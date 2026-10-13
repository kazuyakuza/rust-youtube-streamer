//! Loopback callback listener for the auth flow (plan D6/D7): binds `127.0.0.1` with an
//! ephemeral port (never `localhost`, never a wildcard) and accepts callbacks on the main
//! thread with no wall-clock deadline; each connection gets a 5-second read timeout and
//! an 8 KiB request cap, and up to 8 noise connections are tolerated before the flow
//! abandons. Declined and state-mismatched callbacks are terminal (a foreign state is
//! treated as CSRF); noise is retried; HTTP methods are not checked; browser responses
//! are fixed 200/400 lines that never echo the request. [`BoundListener`] owns the
//! socket and closes it on success AND failure; `CsrfToken` is opaque plumbing.

use oauth2::CsrfToken;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use tracing::info;

use super::AuthError;
use crate::youtube::{
    parse_authorization_callback, AuthorizationCallback, AuthorizationCode, AuthorizationError,
};

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
        let response = outcome_response_parts(&outcome);
        write_callback_response(&mut stream, response);
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

/// Per-connection classification (D7): the four outcomes the accept loop can see.
enum CallbackOutcome {
    Authorized(AuthorizationCode),
    Declined { reason: String },
    StateMismatch,
    Noise,
}
/// The fixed (status line, body) pair for a decided outcome; both halves are static.
struct CallbackResponse {
    status_line: &'static str,
    body: &'static str,
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
fn outcome_response_parts(outcome: &CallbackOutcome) -> CallbackResponse {
    match outcome {
        CallbackOutcome::Authorized(_) => CallbackResponse {
            status_line: CALLBACK_ACCEPTED_STATUS,
            body: CALLBACK_ACCEPTED_BODY,
        },
        _ => CallbackResponse {
            status_line: CALLBACK_REJECTED_STATUS,
            body: CALLBACK_REJECTED_BODY,
        },
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
/// Writes the fixed response; a failed write never changes the decided outcome.
fn write_callback_response(stream: &mut TcpStream, response: CallbackResponse) {
    let content_length = response.body.len();
    let response_text = format!(
        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_line, content_length, response.body
    );
    let _ = stream.write_all(response_text.as_bytes());
}

#[cfg(test)]
#[path = "listener_tests.rs"]
mod listener_tests;
