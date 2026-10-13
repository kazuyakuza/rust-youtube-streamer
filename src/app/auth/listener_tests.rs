//! Loopback listener tests (L1-L8): bind address facts, the authorized /
//! declined / state-mismatch outcomes, noise tolerance and budget
//! exhaustion, invalid-UTF-8 handling, and the drop-closes-socket
//! guarantee. Real sockets, but only `127.0.0.1` ephemeral ports — never
//! leaving the machine. Documented limitations: the 5-second read-timeout
//! noise path is deliberately untested (it would slow the suite by 5 s;
//! covered by design review only), and L1/L8's port facts carry a
//! theoretical OS port-reuse race that is negligible on the containerized
//! Linux runner (L8's failure assertion is written tolerance-style).

use std::io::{Read, Write};

use crate::youtube::generate_state;

use super::{AuthError, BoundListener, NOISE_CONNECTION_LIMIT};

const AUTHORIZATION_CODE_FIXTURE: &str = "authorization_code_fixture";
const ACCEPTED_BODY_TEXT: &str = "Authorization complete";

fn get_request_bytes(query: &str) -> Vec<u8> {
    format!("GET /?{query} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").into_bytes()
}

fn get_path_request_bytes(path: &str) -> Vec<u8> {
    format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").into_bytes()
}

/// Spawns a browser thread that connects, sends the request, then reads
/// the reply until the server closes; the handle is joined AFTER the wait
/// call so the server-side response can be asserted.
fn spawn_browser(port: u16, request_bytes: Vec<u8>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut stream =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the listener accepts");
        stream
            .write_all(&request_bytes)
            .expect("the request is sent");
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .expect("the response is read");
        response
    })
}

/// Sends one noise request and closes without waiting for the reply, so
/// sequential noise connections queue deterministically before the wait
/// loop starts accepting.
fn spawn_noise(port: u16, request_bytes: Vec<u8>) {
    std::thread::spawn(move || {
        let mut stream =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the listener accepts");
        stream
            .write_all(&request_bytes)
            .expect("the request is sent");
    })
    .join()
    .expect("the noise request is sent");
}

fn bound_port(bound: &BoundListener) -> u16 {
    bound
        .redirect_uri()
        .rsplit_once(':')
        .expect("a port suffix")
        .1
        .parse()
        .expect("a numeric port")
}

fn authorized_query(state: &oauth2::CsrfToken) -> String {
    format!("code={AUTHORIZATION_CODE_FIXTURE}&state={}", state.secret())
}

#[test]
fn bind_uses_loopback_and_ephemeral_port() {
    let bound = super::bind().expect("the loopback listener binds");
    let redirect_uri = bound.redirect_uri();
    assert!(redirect_uri.starts_with("http://127.0.0.1:"));
    assert!(!redirect_uri.contains("localhost"));
    let port = bound_port(&bound);
    assert!(port > 0);
}

#[test]
fn authorized_callback_delivers_the_code() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    let state = generate_state();
    let browser = spawn_browser(port, get_request_bytes(&authorized_query(&state)));
    let code = bound
        .wait_for_authorized_code(&state)
        .expect("the authorized callback succeeds");
    assert!(code.secret() == AUTHORIZATION_CODE_FIXTURE);
    let response = browser.join().expect("the browser thread completes");
    let reply = String::from_utf8_lossy(&response);
    assert!(reply.starts_with("HTTP/1.1 200 OK"));
    assert!(reply.contains(ACCEPTED_BODY_TEXT));
}

#[test]
fn declined_callback_fails_the_flow() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    let state = generate_state();
    let browser = spawn_browser(port, get_request_bytes("error=access_denied"));
    let error = bound
        .wait_for_authorized_code(&state)
        .expect_err("the declined callback must fail");
    let AuthError::ProviderDeclined { reason } = error else {
        panic!("expected ProviderDeclined, got {error}");
    };
    assert!(reason == "access_denied");
    let response = browser.join().expect("the browser thread completes");
    assert!(String::from_utf8_lossy(&response).starts_with("HTTP/1.1 400 Bad Request"));
}

#[test]
fn state_mismatch_callback_fails_the_flow() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    let expected_state = generate_state();
    let foreign_state = generate_state();
    let browser = spawn_browser(port, get_request_bytes(&authorized_query(&foreign_state)));
    let error = bound
        .wait_for_authorized_code(&expected_state)
        .expect_err("the foreign state must fail");
    assert!(matches!(error, AuthError::StateMismatch));
    assert!(!format!("{error:?}").contains(AUTHORIZATION_CODE_FIXTURE));
    assert!(!format!("{error}").contains(AUTHORIZATION_CODE_FIXTURE));
    browser.join().expect("the browser thread completes");
}

#[test]
fn noise_connections_are_ignored_until_a_valid_callback() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    spawn_noise(port, get_path_request_bytes("/favicon.ico"));
    spawn_noise(port, b"GARBAGE /x?junk=1\r\n\r\n".to_vec());
    let state = generate_state();
    let browser = spawn_browser(port, get_request_bytes(&authorized_query(&state)));
    let code = bound
        .wait_for_authorized_code(&state)
        .expect("the valid callback still succeeds");
    assert!(code.secret() == AUTHORIZATION_CODE_FIXTURE);
    browser.join().expect("the browser thread completes");
}

#[test]
fn exhausted_noise_budget_abandons_the_flow() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    for _ in 0..NOISE_CONNECTION_LIMIT {
        spawn_noise(port, get_path_request_bytes("/favicon.ico"));
    }
    let state = generate_state();
    let error = bound
        .wait_for_authorized_code(&state)
        .expect_err("the exhausted budget must fail");
    assert!(matches!(
        error,
        AuthError::CallbackAbandoned { attempts } if attempts == NOISE_CONNECTION_LIMIT
    ));
}

#[test]
fn invalid_utf8_request_is_treated_as_noise() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    spawn_noise(
        port,
        b"GET /\xff HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n".to_vec(),
    );
    let state = generate_state();
    let browser = spawn_browser(port, get_request_bytes(&authorized_query(&state)));
    let code = bound
        .wait_for_authorized_code(&state)
        .expect("the valid callback still succeeds");
    assert!(code.secret() == AUTHORIZATION_CODE_FIXTURE);
    browser.join().expect("the browser thread completes");
}

#[test]
fn listener_socket_is_closed_when_dropped() {
    let bound = super::bind().expect("the loopback listener binds");
    let port = bound_port(&bound);
    let first_connection = std::net::TcpStream::connect(("127.0.0.1", port));
    assert!(first_connection.is_ok(), "the bound listener accepts");
    drop(bound);
    let second_connection = std::net::TcpStream::connect(("127.0.0.1", port));
    assert!(second_connection.is_err(), "the dropped listener refuses");
}
