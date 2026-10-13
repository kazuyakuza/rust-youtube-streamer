//! Verifier loopback tests (R2-R6, R10, R11): request assembly through
//! the REAL ureq agent against fake `127.0.0.1` servers, and the
//! live status→mapping outcomes. Loopback only; no live Google access.
//! The pure-classifier tests live in the `..._classify_tests` sibling.

use std::io::{Read, Write};
use std::sync::mpsc;

use super::*;

pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";

struct RecordedRequest {
    request_line: String,
    authorization: Option<String>,
}

struct FakeApiServer {
    port: u16,
    recorded: mpsc::Receiver<RecordedRequest>,
    handle: std::thread::JoinHandle<()>,
}

impl FakeApiServer {
    fn start(status_line: &str, body: &str) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let port = listener.local_addr().expect("a bound address").port();
        let (sender, recorded) = mpsc::channel();
        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("the fake API accepts a request");
            let request = record_request(&mut stream);
            sender.send(request).expect("the recording arrives");
            stream
                .write_all(response.as_bytes())
                .expect("the canned reply is sent");
        });
        Self {
            port,
            recorded,
            handle,
        }
    }

    fn verifier(&self) -> GoogleAuthorizationVerifier {
        GoogleAuthorizationVerifier::with_request_url(format!(
            "http://127.0.0.1:{}/youtube/v3/channels?part=id&mine=true",
            self.port
        ))
    }

    fn recorded_request(&self) -> RecordedRequest {
        self.recorded.recv().expect("the request was recorded")
    }

    fn finish(self) {
        self.handle.join().expect("the fake API thread completes");
    }
}

fn record_request(stream: &mut std::net::TcpStream) -> RecordedRequest {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 512];
    while !request_headers_complete(&bytes) {
        let read = stream.read(&mut chunk).expect("the request is readable");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let text = String::from_utf8_lossy(&bytes);
    RecordedRequest {
        request_line: text.lines().next().unwrap_or_default().to_string(),
        authorization: text.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("authorization")
                .then(|| value.trim().to_string())
        }),
    }
}

fn request_headers_complete(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|window| window == b"\r\n\r\n")
}

pub(super) fn fixture_access_token() -> AccessToken {
    AccessToken::new(ACCESS_TOKEN_FIXTURE)
}

pub(super) fn error_body(reason: &str, message: &str) -> String {
    format!(
        r#"{{"error":{{"code":401,"message":"{message}","errors":[{{"reason":"{reason}"}}]}}}}"#
    )
}

/// Failure extraction without `expect_err` (VerifiedAuthorization is Debug-free).
pub(super) fn verification_failure(
    outcome: Result<VerifiedAuthorization, VerificationError>,
) -> VerificationError {
    let Err(error) = outcome else {
        panic!("expected a verification failure");
    };
    error
}

#[test]
fn bearer_header_and_request_line_reach_the_fake_api() {
    let server = FakeApiServer::start("HTTP/1.1 200 OK", "{}");
    server
        .verifier()
        .verify(&fixture_access_token())
        .expect("the fake 200 verifies");
    let recorded = server.recorded_request();
    assert!(recorded
        .request_line
        .starts_with("GET /youtube/v3/channels?part=id&mine=true "));
    let authorization = recorded.authorization.expect("the Authorization header");
    assert_eq!(
        format!("Authorization: {authorization}"),
        "Authorization: Bearer access_token_fixture"
    );
    server.finish();
}

#[test]
fn server_200_verifies_with_required_scope_evidence() {
    let server = FakeApiServer::start("HTTP/1.1 200 OK", "{}");
    let verified = server
        .verifier()
        .verify(&fixture_access_token())
        .expect("a 200 verifies");
    assert!(verified.granted_scopes().grants(REQUIRED_SCOPE));
    server.finish();
}

#[test]
fn server_401_maps_to_not_authorized() {
    let body = error_body("authorizationRequired", "Invalid credentials.");
    let server = FakeApiServer::start("HTTP/1.1 401 Unauthorized", &body);
    let error = verification_failure(server.verifier().verify(&fixture_access_token()));
    assert!(matches!(error, VerificationError::NotAuthorized { .. }));
    let rendered = format!("{error}");
    assert!(rendered.contains("HTTP 401"));
    assert!(rendered.contains("authorizationRequired"));
    assert!(!rendered.contains("Invalid credentials"));
    server.finish();
}

#[test]
fn server_403_insufficient_permissions_maps_to_the_scope_class() {
    let body = error_body("insufficientPermissions", "Scopes insufficient.");
    let server = FakeApiServer::start("HTTP/1.1 403 Forbidden", &body);
    let error = verification_failure(server.verifier().verify(&fixture_access_token()));
    let VerificationError::InsufficientScope { reason } = &error else {
        panic!("expected the InsufficientScope class, got {error}");
    };
    assert!(reason.contains("insufficientPermissions"));
    server.finish();
}

#[test]
fn server_403_other_reason_still_maps_to_the_scope_class() {
    let body = error_body("accountSuspended", "The account is suspended.");
    let server = FakeApiServer::start("HTTP/1.1 403 Forbidden", &body);
    let error = verification_failure(server.verifier().verify(&fixture_access_token()));
    let VerificationError::InsufficientScope { reason } = &error else {
        panic!("expected the InsufficientScope class, got {error}");
    };
    assert!(reason.contains("accountSuspended"));
    server.finish();
}

#[test]
fn redirects_are_not_followed() {
    let server = FakeApiServer::start("HTTP/1.1 301 Moved Permanently", "");
    let error = verification_failure(server.verifier().verify(&fixture_access_token()));
    let VerificationError::Transport { reason } = &error else {
        panic!("expected the Transport class, got {error}");
    };
    assert!(reason.contains("HTTP 301"));
    server.finish();
}

#[test]
fn connection_failures_map_to_transport() {
    let dropped = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port = dropped.local_addr().expect("a bound address").port();
    drop(dropped);
    let verifier = GoogleAuthorizationVerifier::with_request_url(format!(
        "http://127.0.0.1:{port}/youtube/v3/channels?part=id&mine=true"
    ));
    let error = verification_failure(verifier.verify(&fixture_access_token()));
    assert!(matches!(error, VerificationError::Transport { .. }));
}
