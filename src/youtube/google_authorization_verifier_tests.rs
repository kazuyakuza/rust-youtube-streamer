//! Verifier tests (R1-R11): the documented endpoint constant, request
//! assembly through the REAL ureq agent against fake `127.0.0.1` servers,
//! the status→error mapping table and the body-redaction guarantees of
//! the pure classifier. Loopback only; no live Google access.

use std::io::{Read, Write};
use std::sync::mpsc;

use super::*;

const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";

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

fn fixture_access_token() -> AccessToken {
    AccessToken::new(ACCESS_TOKEN_FIXTURE)
}

fn error_body(reason: &str, message: &str) -> String {
    format!(
        r#"{{"error":{{"code":401,"message":"{message}","errors":[{{"reason":"{reason}"}}]}}}}"#
    )
}

/// `expect_err` would demand `Debug` on `VerifiedAuthorization` (a type
/// the plan keeps Debug-free by design), so failure extraction uses this.
fn verification_failure(
    outcome: Result<VerifiedAuthorization, VerificationError>,
) -> VerificationError {
    let Err(error) = outcome else {
        panic!("expected a verification failure");
    };
    error
}

#[test]
fn verification_request_url_is_the_documented_channels_call() {
    assert_eq!(
        VERIFICATION_ENDPOINT,
        "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true"
    );
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
    let server = FakeApiServer::start(
        "HTTP/1.1 200 OK",
        r#"{"kind":"youtube#channelListResponse"}"#,
    );
    let verified = server
        .verifier()
        .verify(&fixture_access_token())
        .expect("a 200 verifies");
    assert!(verified.granted_scopes().grants(REQUIRED_SCOPE));
    server.finish();
}

#[test]
fn server_401_maps_to_not_authorized() {
    let body = error_body(
        "authorizationRequired",
        "Request had invalid authentication credentials.",
    );
    let server = FakeApiServer::start("HTTP/1.1 401 Unauthorized", &body);
    let error = verification_failure(server.verifier().verify(&fixture_access_token()));
    assert!(matches!(error, VerificationError::NotAuthorized { .. }));
    let rendered = format!("{error}");
    assert!(rendered.contains("HTTP 401"));
    assert!(rendered.contains("authorizationRequired"));
    assert!(!rendered.contains("invalid authentication credentials"));
    server.finish();
}

#[test]
fn server_403_insufficient_permissions_maps_to_the_scope_class() {
    let body = error_body(
        "insufficientPermissions",
        "The OAuth 2.0 token scopes are insufficient.",
    );
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
fn other_statuses_map_to_transport() {
    for status in [400_u16, 404, 500] {
        let error = verification_failure(classify_verification_reply(status, None));
        let VerificationError::Transport { reason } = &error else {
            panic!("expected the Transport class, got {error}");
        };
        assert!(reason.contains(&status.to_string()));
    }
}

#[test]
fn error_bodies_are_never_embedded_into_reasons() {
    let hostile = format!(
        r#"{{"error":{{"code":401,"message":"token {ACCESS_TOKEN_FIXTURE} was rejected","errors":[{{"reason":"authError"}}]}}}}"#
    );
    let error = verification_failure(classify_verification_reply(401, Some(&hostile)));
    let rendered = format!("{error}");
    assert!(rendered.contains("authError"));
    assert!(!rendered.contains(ACCESS_TOKEN_FIXTURE));
}

#[test]
fn unparsable_error_bodies_use_the_fixed_reason_text() {
    let error = verification_failure(classify_verification_reply(
        401,
        Some("Connection refused by the operator."),
    ));
    let VerificationError::NotAuthorized { reason } = &error else {
        panic!("expected the NotAuthorized class, got {error}");
    };
    assert_eq!(reason, REJECTED_TOKEN_TEXT);
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
