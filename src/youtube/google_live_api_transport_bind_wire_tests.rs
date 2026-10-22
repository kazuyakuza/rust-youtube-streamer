//! Enforcing loopback server for the bodyless-POST wire pins (the HTTP
//! 411 fix): request-fact capture with the `Content-Length` header plus
//! Google-like POST enforcement — a POST WITHOUT `Content-Length` is
//! rejected 411, every other form (the fixed zero-length bind POST, any
//! GET) is accepted. Loopback `127.0.0.1` only; no test reaches Google.
//!
//! Plan: `.kilo/plans/20261022-fix-http-411-task1-implementation.md`;
//! TODO `.agent/todos/20261022/20261022-todo-1.md` Task 1.

use std::io::{Read, Write};
use std::sync::mpsc;

use crate::youtube::api_request::response_reason_error;

use super::google_live_api_transport_tests::ACCESS_TOKEN_FIXTURE;
pub(super) use super::google_live_api_transport_tests::{
    BOUND_BROADCAST_BODY, BROADCAST_ID_FIXTURE, STREAM_ID_FIXTURE,
};
use super::*;

const ENFORCED_STATUS_LINE: &str = "HTTP/1.1 411 Length Required";
const ACCEPTED_STATUS_LINE: &str = "HTTP/1.1 200 OK";
const ENFORCED_BODY: &str = "Length Required";

pub(super) struct ServedRequest {
    pub(super) request_line: String,
    pub(super) content_length: Option<String>,
    pub(super) body: String,
}

pub(super) struct EnforcingBindServer {
    pub(super) port: u16,
    served: mpsc::Receiver<ServedRequest>,
    handle: std::thread::JoinHandle<()>,
}

impl EnforcingBindServer {
    pub(super) fn start(accept_body: &str) -> Self {
        let accept_body = accept_body.to_string();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let port = listener.local_addr().expect("a bound address").port();
        let (sender, served) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("the enforcing server accepts a request");
            let served_request = record_request(&mut stream);
            let reply = reply_for(&served_request, &accept_body);
            sender
                .send(served_request)
                .expect("the request fact was recorded");
            stream
                .write_all(reply.as_bytes())
                .expect("the canned reply is sent");
        });
        Self {
            port,
            served,
            handle,
        }
    }

    pub(super) fn transport(&self) -> GoogleLiveApiTransport {
        GoogleLiveApiTransport::with_base_url(
            format!("http://127.0.0.1:{}/youtube/v3/", self.port),
            AccessToken::new(ACCESS_TOKEN_FIXTURE),
        )
    }

    pub(super) fn served_request(&self) -> ServedRequest {
        self.served.recv().expect("the request fact was recorded")
    }

    pub(super) fn finish(self) {
        self.handle.join().expect("the enforcing server completes");
    }
}

fn reply_for(served_request: &ServedRequest, accept_body: &str) -> String {
    if is_enforced_request(served_request) {
        canned_reply(ENFORCED_STATUS_LINE, ENFORCED_BODY)
    } else {
        canned_reply(ACCEPTED_STATUS_LINE, accept_body)
    }
}

/// The Google-like enforcing rule for this probe: a POST arriving
/// WITHOUT a Content-Length header is 411-class; every other form —
/// the fixed zero-length bind POST or any GET — is accepted.
fn is_enforced_request(served_request: &ServedRequest) -> bool {
    served_request.request_line.starts_with("POST") && served_request.content_length.is_none()
}

fn canned_reply(status_line: &str, body: &str) -> String {
    format!(
        "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn record_request(stream: &mut std::net::TcpStream) -> ServedRequest {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 512];
    while headers_end(&bytes).is_none() {
        let read = stream.read(&mut chunk).expect("the request is readable");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let text = String::from_utf8_lossy(&bytes);
    ServedRequest {
        request_line: text.lines().next().unwrap_or_default().to_string(),
        content_length: header_value(&text, "content-length"),
        body: read_body_tail(stream, &bytes),
    }
}

fn read_body_tail(stream: &mut std::net::TcpStream, head: &[u8]) -> String {
    let head_text = String::from_utf8_lossy(head);
    let content_length = header_value(&head_text, "content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body_bytes = head[headers_end(head).unwrap_or(head.len())..].to_vec();
    let mut chunk = [0_u8; 512];
    while body_bytes.len() < content_length {
        let read = stream.read(&mut chunk).expect("the body is readable");
        if read == 0 {
            break;
        }
        body_bytes.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8_lossy(&body_bytes).to_string()
}

fn headers_end(bytes: &[u8]) -> Option<usize> {
    let index = bytes.windows(4).position(|window| window == b"\r\n\r\n")?;
    Some(index + 4)
}

fn header_value(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_string())
    })
}

/// Reproduces the INCIDENT wire form in the test only: a POST whose
/// transmission is the ureq no-body `call()` — the exact shape that
/// produced `HTTP 411` at Google before the fix. Never production code.
pub(super) fn pre_fix_lengthless_call(url: &str) -> Result<ureq::Response, ureq::Error> {
    ureq::AgentBuilder::new()
        .redirects(0)
        .build()
        .post(url)
        .set("Authorization", &format!("Bearer {ACCESS_TOKEN_FIXTURE}"))
        .call()
}

/// Classifies a raw ureq reply through the SAME arm the transport's ONE
/// `respond` applies to non-2xx statuses (D-T2-5), keeping the pinned
/// classification honest without widening production visibility.
pub(super) fn classify_through_status_arm(
    reply: Result<ureq::Response, ureq::Error>,
) -> LiveApiError {
    match reply {
        Err(ureq::Error::Status(status, response)) => response_reason_error(status, response),
        _ => panic!("expected the enforcing rejection to arrive as a status error"),
    }
}
