//! Transport loopback tests (plan §4 roster A, request-assembly subset;
//! the reply/error outcomes live in the
//! `google_live_api_transport_outcomes_tests` sibling): real-ureq request
//! assembly against canned `127.0.0.1` servers — request line, Bearer
//! header, Content-Type, POST-body capture — with the O1/O2 wire forms
//! asserted against the locked request-body shapes. No Google contact.

use std::io::{Read, Write};
use std::sync::mpsc;

use serde_json::json;

use super::*;

pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";
pub(super) const STREAM_ID_FIXTURE: &str = "stream-id-fixture";
pub(super) const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";

pub(super) const STREAM_RESOURCE_BODY: &str = r#"{"id":"stream-id-fixture","cdn":{"ingestionInfo":{"streamName":"stream-name-fixture","rtmpsIngestionAddress":"rtmps://ingest.example.test/ingest"}}}"#;
pub(super) const BOUND_BROADCAST_BODY: &str =
    r#"{"id":"broadcast-id-fixture","contentDetails":{"boundStreamId":"stream-id-fixture"}}"#;

pub(super) struct RecordedRequest {
    pub(super) request_line: String,
    pub(super) authorization: Option<String>,
    pub(super) content_type: Option<String>,
    pub(super) body: String,
}

/// One canned JSON exchange per `start` (the Phase-04 `FakeApiServer`
/// precedent, plus the POST-body capture).
pub(super) struct RecordingJsonServer {
    port: u16,
    recorded: mpsc::Receiver<RecordedRequest>,
    handle: std::thread::JoinHandle<()>,
}

impl RecordingJsonServer {
    pub(super) fn start(status_line: &str, body: &str) -> Self {
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

    pub(super) fn transport(&self) -> GoogleLiveApiTransport {
        GoogleLiveApiTransport::with_base_url(
            format!("http://127.0.0.1:{}/youtube/v3/", self.port),
            AccessToken::new(ACCESS_TOKEN_FIXTURE),
        )
    }

    pub(super) fn recorded_request(&self) -> RecordedRequest {
        self.recorded.recv().expect("the request was recorded")
    }

    pub(super) fn finish(self) {
        self.handle.join().expect("the fake API thread completes");
    }
}

fn record_request(stream: &mut std::net::TcpStream) -> RecordedRequest {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 512];
    while headers_end(&bytes).is_none() {
        let read = stream.read(&mut chunk).expect("the request is readable");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let text = String::from_utf8_lossy(&bytes).to_string();
    RecordedRequest {
        request_line: text.lines().next().unwrap_or_default().to_string(),
        authorization: header_value(&text, "authorization"),
        content_type: header_value(&text, "content-type"),
        body: read_body(stream, &bytes, &text),
    }
}

fn read_body(stream: &mut std::net::TcpStream, head: &[u8], text: &str) -> String {
    let content_length = header_value(text, "content-length")
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

#[test]
fn stream_insert_posts_documented_path_with_bearer_and_json() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", STREAM_RESOURCE_BODY);
    let command = CreateStreamCommand {
        title: "Fixture stream".to_string(),
        resolution: "1080p".to_string(),
        frame_rate: "30fps".to_string(),
    };
    server
        .transport()
        .create_stream(&command)
        .expect("the fake 200 resource parses");
    let recorded = server.recorded_request();
    assert!(recorded
        .request_line
        .starts_with("POST /youtube/v3/liveStreams?part=snippet,cdn "));
    assert_eq!(
        recorded.authorization.as_deref(),
        Some("Bearer access_token_fixture")
    );
    assert_eq!(recorded.content_type.as_deref(), Some("application/json"));
    let sent: serde_json::Value = serde_json::from_str(&recorded.body).expect("JSON body");
    assert_eq!(
        sent,
        json!({
            "snippet": {"title": "Fixture stream"},
            "cdn": {"ingestionType": "rtmp", "resolution": "1080p", "frameRate": "30fps"}
        })
    );
    server.finish();
}

#[test]
fn broadcast_insert_posts_documented_path_with_locked_body() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", BOUND_BROADCAST_BODY);
    let command = CreateBroadcastCommand {
        title: "Fixture broadcast".to_string(),
        description: "Fixture description".to_string(),
        privacy_status: "unlisted".to_string(),
        scheduled_start_time: "2026-10-14T12:01:00Z".to_string(),
        made_for_kids: None,
    };
    server
        .transport()
        .create_broadcast(&command)
        .expect("the fake 200 resource parses");
    let recorded = server.recorded_request();
    assert!(recorded
        .request_line
        .starts_with("POST /youtube/v3/liveBroadcasts?part=snippet,contentDetails,status "));
    let sent: serde_json::Value = serde_json::from_str(&recorded.body).expect("JSON body");
    assert_eq!(
        sent,
        json!({
            "snippet": {
                "title": "Fixture broadcast",
                "description": "Fixture description",
                "scheduledStartTime": "2026-10-14T12:01:00Z"
            },
            "contentDetails": {
                "enableAutoStart": true,
                "enableAutoStop": true,
                "monitorStream": {"enableMonitorStream": false}
            },
            "status": {"privacyStatus": "unlisted"}
        })
    );
    server.finish();
}
