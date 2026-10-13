//! Loopback callback listener for the auth flow (plan D6/D7): binds
//! `127.0.0.1` with an OS-ephemeral port — never `localhost`, never a
//! wildcard address — and accepts browser callbacks on the main thread
//! with no spawned threads and no async runtime. Each connection is
//! classified as authorized, declined, state-mismatched or noise;
//! declined and state-mismatch outcomes are terminal, noise is retried
//! under a bounded budget. HTTP method names are not checked: any
//! request whose request line carries a parseable query string is
//! classified per that query. The [`BoundListener`] owns the socket;
//! dropping it closes the listener deterministically on success AND on
//! every failure. `oauth2::CsrfToken` appears only as opaque plumbing
//! passed through signatures; all Google-protocol logic stays in
//! `src/youtube/`.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use oauth2::CsrfToken;
use tracing::info;

use crate::youtube::{
    parse_authorization_callback, AuthorizationCallback, AuthorizationCode, AuthorizationError,
};

use super::AuthError;

const CALLBACK_HOST: &str = "127.0.0.1";
const NOISE_CONNECTION_LIMIT: usize = 8;
const REQUEST_READ_LIMIT: usize = 8192;
const REQUEST_READ_CHUNK: usize = 1024;
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(5);
const CALLBACK_ACCEPTED_STATUS: &str = "HTTP/1.1 200 OK";
const CALLBACK_REJECTED_STATUS: &str = "HTTP/1.1 400 Bad Request";
const CALLBACK_ACCEPTED_BODY: &str =
    "Authorization complete. You can close this window and return to the application.";
const CALLBACK_REJECTED_BODY: &str =
    "This callback port is only used by the running authorization flow.";
const HEADER_TERMINATOR_BYTES: &[u8] = b"\r\n\r\n";

/// A loopback-bound listener plus the redirect URI it serves. Dropping this
/// value closes the socket; the auth flow holds it exactly for the duration
/// of the authorization wait.
pub(super) struct BoundListener {
    listener: TcpListener,
    redirect_uri: String,
}

/// Binds on 127.0.0.1 with an ephemeral port (D6); emits the bound event.
pub(super) fn bind() -> Result<BoundListener, AuthError> {
    let listener = TcpListener::bind((CALLBACK_HOST, 0)).map_err(AuthError::ListenerBind)?;
    let address: SocketAddr = listener.local_addr().map_err(AuthError::ListenerBind)?;
    info!(redirect_port = %address.port(), "callback listener bound");
    Ok(BoundListener {
        listener,
        redirect_uri: format!("http://{CALLBACK_HOST}:{}", address.port()),
    })
}

impl BoundListener {
    pub(super) fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Blocking accept loop: returns the authorized code on success, a typed
    /// AuthError for terminal outcomes, and keeps waiting through noise up to
    /// the budget (D7). The budget counter is local state incrementing once
    /// per noise-classified connection (or transient accept error).
    pub(super) fn wait_for_authorized_code(
        &self,
        expected_state: &CsrfToken,
    ) -> Result<AuthorizationCode, AuthError> {
        let mut noise_count = 0usize;
        loop {
            if let Some(code) = self.handle_next_connection(expected_state, &mut noise_count)? {
                return Ok(code);
            }
        }
    }

    /// Accepts one connection, classifies it, responds, and maps the result.
    /// Ok(None) = the connection was noise and the budget still holds.
    fn handle_next_connection(
        &self,
        expected_state: &CsrfToken,
        noise_count: &mut usize,
    ) -> Result<Option<AuthorizationCode>, AuthError> {
        let mut stream = match self.listener.accept() {
            Ok((stream, _)) => stream,
            Err(_) => return discard_benign_connection(noise_count),
        };
        match classify_connection(&mut stream, expected_state) {
            CallbackOutcome::Authorized(code) => {
                info!("authorization callback accepted");
                write_callback_response(
                    &mut stream,
                    CALLBACK_ACCEPTED_STATUS,
                    CALLBACK_ACCEPTED_BODY,
                );
                Ok(Some(code))
            }
            CallbackOutcome::Declined { reason } => {
                write_callback_response(
                    &mut stream,
                    CALLBACK_REJECTED_STATUS,
                    CALLBACK_REJECTED_BODY,
                );
                Err(AuthError::ProviderDeclined { reason })
            }
            CallbackOutcome::StateMismatch => {
                write_callback_response(
                    &mut stream,
                    CALLBACK_REJECTED_STATUS,
                    CALLBACK_REJECTED_BODY,
                );
                Err(AuthError::StateMismatch)
            }
            CallbackOutcome::Noise => {
                write_callback_response(
                    &mut stream,
                    CALLBACK_REJECTED_STATUS,
                    CALLBACK_REJECTED_BODY,
                );
                discard_benign_connection(noise_count)
            }
        }
    }
}

/// Bumps the noise counter and fails the flow when the budget is exhausted.
fn discard_benign_connection(
    noise_count: &mut usize,
) -> Result<Option<AuthorizationCode>, AuthError> {
    *noise_count += 1;
    if is_noise_budget_exhausted(*noise_count) {
        return Err(AuthError::CallbackAbandoned {
            attempts: *noise_count,
        });
    }
    Ok(None)
}

/// Single-section boolean predicate (rule).
fn is_noise_budget_exhausted(noise_count: usize) -> bool {
    noise_count >= NOISE_CONNECTION_LIMIT
}

enum CallbackOutcome {
    Authorized(AuthorizationCode),
    Declined { reason: String },
    StateMismatch,
    Noise,
}

/// Classifies one already-accepted connection (D7).
fn classify_connection(stream: &mut TcpStream, expected_state: &CsrfToken) -> CallbackOutcome {
    let request_text = match read_request_text(stream) {
        Some(text) => text,
        None => return CallbackOutcome::Noise,
    };
    let query = match callback_query(&request_text) {
        Some(query) => query,
        None => return CallbackOutcome::Noise,
    };
    match parse_authorization_callback(query, expected_state) {
        Ok(AuthorizationCallback::Authorized { code, .. }) => CallbackOutcome::Authorized(code),
        Ok(AuthorizationCallback::Declined { reason }) => CallbackOutcome::Declined { reason },
        Err(AuthorizationError::StateMismatch) => CallbackOutcome::StateMismatch,
        Err(AuthorizationError::MissingParameter { .. }) => CallbackOutcome::Noise,
        Err(AuthorizationError::ProviderDeclined { reason }) => {
            CallbackOutcome::Declined { reason }
        }
    }
}

/// Reads the request head bytes with the read timeout set; returns None for
/// anything that never terminates its headers within the size limit.
fn read_request_text(stream: &mut TcpStream) -> Option<String> {
    let _ = stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT));
    let mut received: Vec<u8> = Vec::new();
    let mut chunk = [0u8; REQUEST_READ_CHUNK];
    loop {
        if head_is_complete(&received) {
            return String::from_utf8(received).ok();
        }
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(read_count) => received.extend_from_slice(&chunk[..read_count]),
            Err(_) => return None,
        }
        if received.len() > REQUEST_READ_LIMIT {
            return None;
        }
    }
}

/// Single-section predicate over the buffered bytes.
fn head_is_complete(received: &[u8]) -> bool {
    received.ends_with(HEADER_TERMINATOR_BYTES)
}

/// Extracts the query part of the request line ("GET /?a=b HTTP/1.1").
fn callback_query(request_text: &str) -> Option<&str> {
    let request_line = request_text.lines().next()?;
    let path = request_line.split_whitespace().nth(1)?;
    let separation = path.find('?')?;
    Some(&path[separation + 1..])
}

/// Writes one fixed-content HTTP response; failures are intentionally
/// ignored (the parse outcome was already decided before this call).
fn write_callback_response(stream: &mut TcpStream, status_line: &str, body: &str) {
    let content_length = body.len();
    let response = format!(
        "{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}"
    );
    let _ = stream.write_all(response.as_bytes());
}

#[cfg(test)]
#[path = "listener_tests.rs"]
mod listener_tests;
