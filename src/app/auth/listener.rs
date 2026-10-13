//! Loopback callback listener for the auth flow (plan D6/D7): binds
//! `127.0.0.1` with an ephemeral port (never `localhost`, never a
//! wildcard) and accepts callbacks on the main thread. Connections
//! classify as authorized, declined, state-mismatched or noise; the
//! latter two fail terminally, noise retries under a bounded budget,
//! and HTTP methods are not checked. [`BoundListener`] owns the socket
//! and closes it on success AND failure; CsrfToken is opaque plumbing.

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

/// A loopback-bound listener plus its redirect URI; drop closes the socket.
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

    /// Blocking accept loop (D7): the code on success, a typed error otherwise.
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

    /// Accepts one connection, classifies it, responds, maps it; None = noise.
    fn handle_next_connection(
        &self,
        expected_state: &CsrfToken,
        noise_count: &mut usize,
    ) -> Result<Option<AuthorizationCode>, AuthError> {
        let mut stream = match self.listener.accept() {
            Ok((stream, _)) => stream,
            Err(_) => return discard_benign_connection(noise_count),
        };
        let outcome = classify_connection(&mut stream, expected_state);
        let (status_line, body) = outcome_response_parts(&outcome);
        write_callback_response(&mut stream, status_line, body);
        map_callback_outcome(outcome, noise_count)
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

fn is_noise_budget_exhausted(noise_count: usize) -> bool {
    noise_count >= NOISE_CONNECTION_LIMIT
}

enum CallbackOutcome {
    Authorized(AuthorizationCode),
    Declined { reason: String },
    StateMismatch,
    Noise,
}

/// Maps a classified outcome to the accept-loop result (D7).
fn map_callback_outcome(
    outcome: CallbackOutcome,
    noise_count: &mut usize,
) -> Result<Option<AuthorizationCode>, AuthError> {
    match outcome {
        CallbackOutcome::Authorized(code) => {
            info!("authorization callback accepted");
            Ok(Some(code))
        }
        CallbackOutcome::Declined { reason } => Err(AuthError::ProviderDeclined { reason }),
        CallbackOutcome::StateMismatch => Err(AuthError::StateMismatch),
        CallbackOutcome::Noise => discard_benign_connection(noise_count),
    }
}

/// Fixed (status, body) pair for an already-decided outcome.
fn outcome_response_parts(outcome: &CallbackOutcome) -> (&'static str, &'static str) {
    match outcome {
        CallbackOutcome::Authorized(_) => (CALLBACK_ACCEPTED_STATUS, CALLBACK_ACCEPTED_BODY),
        _ => (CALLBACK_REJECTED_STATUS, CALLBACK_REJECTED_BODY),
    }
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

/// Reads the request head under the read timeout; None if it never ends.
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

/// Writes one fixed-content HTTP response; write failures are ignored.
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
