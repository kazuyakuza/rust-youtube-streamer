//! Loopback wire roster for the concrete Phase-06 chat transport (plan
//! §4, caller-authorized sibling split): the full-head capturing
//! [`ChatWireServer`] and its capture helpers live here so both chat
//! wire rosters share ONE server definition; the six prescribed pins
//! inhabit the `live_chat_wire_checks_tests` child roster. Loopback
//! `127.0.0.1` only; no test reaches Google.

use std::io::{Read, Write};
use std::sync::mpsc;

use crate::youtube::token::AccessToken;

use self::live_chat_test_fixtures::ACCESS_TOKEN_FIXTURE;
use super::*;

/// One canned exchange per `start`: the FULL request head is recorded so
/// the request target and the Authorization header stay assertable.
pub(super) struct ChatWireServer {
    port: u16,
    served: mpsc::Receiver<String>,
    handle: std::thread::JoinHandle<()>,
}

impl ChatWireServer {
    fn start(status_line: &str, body: &str) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let port = listener.local_addr().expect("a bound address").port();
        let (sender, served) = mpsc::channel();
        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("the chat wire server accepts a request");
            let head = record_head(&mut stream);
            sender.send(head).expect("the head was recorded");
            stream
                .write_all(response.as_bytes())
                .expect("the canned reply is sent");
        });
        Self {
            port,
            served,
            handle,
        }
    }

    fn transport(&self) -> GoogleLiveApiTransport {
        GoogleLiveApiTransport::with_base_url(
            format!("http://127.0.0.1:{}/youtube/v3/", self.port),
            AccessToken::new(ACCESS_TOKEN_FIXTURE),
        )
    }

    fn head(&self) -> String {
        self.served.recv().expect("the request head was recorded")
    }

    fn finish(self) {
        self.handle.join().expect("the chat wire server completes");
    }
}

pub(super) fn record_head(stream: &mut std::net::TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 512];
    while headers_end(&bytes).is_none() {
        let read = stream.read(&mut chunk).expect("the head is readable");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn headers_end(bytes: &[u8]) -> Option<usize> {
    let index = bytes.windows(4).position(|window| window == b"\r\n\r\n")?;
    Some(index + 4)
}

pub(super) fn header_value(head: &str, name: &str) -> Option<String> {
    head.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_string())
    })
}

pub(super) fn request_target(head: &str) -> &str {
    let request_line = head.lines().next().unwrap_or_default();
    let without_version = request_line.split(" HTTP/").next().unwrap_or_default();
    let Some((_method, target)) = without_version.split_once(' ') else {
        return without_version;
    };
    target
}

#[cfg(test)]
#[path = "live_chat_test_fixtures.rs"]
mod live_chat_test_fixtures;

#[cfg(test)]
#[path = "live_chat_wire_checks_tests.rs"]
mod live_chat_wire_checks_tests;
