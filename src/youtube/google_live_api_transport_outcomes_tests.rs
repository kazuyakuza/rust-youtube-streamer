//! Transport reply/error-outcome tests (plan §4 roster A; the O1/O2
//! request-assembly subset lives in the `google_live_api_transport_tests`
//! sibling, the Phase-04 verifier-pair pattern): the O3 bodyless bind
//! wire form, the O1 reply parsing, status-first classification through
//! the staged `classify_api_error`, reason-only surfacing (bodies never
//! embedded), redirect refusal, connection failures, body-parse failures
//! and the query-encoding unit. NOTE on roster item 11: the staged list
//! models carry `#[serde(default)]` on `items` (Task-1 contract, pinned
//! by `empty_items_fall_back_to_default`), so a `{}` body verifiably
//! equals an empty list → `Ok(None)`; the roster's "absent body is
//! error" case is realized as an EMPTY body, which fails to parse →
//! typed `MalformedResponse`. Loopback only; no test reaches Google.

use crate::youtube::api_request::encode_query_value;

use super::google_live_api_transport_tests::*;
use super::*;

const EMPTY_LIST_BODY: &str = r#"{"items":[]}"#;

fn closed_port() -> u16 {
    let dropped = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port = dropped.local_addr().expect("a bound address").port();
    drop(dropped);
    port
}

/// Failure extraction without `expect_err` (resources are Debug-free).
fn api_failure<T>(outcome: Result<T, LiveApiError>) -> LiveApiError {
    let Err(error) = outcome else {
        panic!("expected a live API failure");
    };
    error
}

fn error_body(reason: &str, message: &str) -> String {
    format!(
        r#"{{"error":{{"code":403,"message":"{message}","errors":[{{"reason":"{reason}"}}]}}}}"#
    )
}

#[test]
fn bind_sends_bodyless_post_with_both_query_ids() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", BOUND_BROADCAST_BODY);
    server
        .transport()
        .bind(&BindCommand {
            broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
            stream_id: STREAM_ID_FIXTURE.to_string(),
        })
        .expect("the fake 200 resource parses");
    let recorded = server.recorded_request();
    assert!(recorded.request_line.starts_with(
        "POST /youtube/v3/liveBroadcasts/bind?id=broadcast-id-fixture&streamId=stream-id-fixture "
    ));
    assert_eq!(recorded.content_type, None, "bind sends no Content-Type");
    assert!(recorded.body.is_empty(), "bind sends no body payload");
    server.finish();
}

#[test]
fn stream_insert_reply_resource_parses() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", STREAM_RESOURCE_BODY);
    let command = CreateStreamCommand {
        title: "Fixture stream".to_string(),
        resolution: "1080p".to_string(),
        frame_rate: "30fps".to_string(),
    };
    let stream = server
        .transport()
        .create_stream(&command)
        .expect("the canned resource parses");
    assert_eq!(stream.id.as_deref(), Some(STREAM_ID_FIXTURE));
    let info = stream
        .cdn
        .expect("cdn part")
        .ingestion_info
        .expect("ingestionInfo part");
    assert_eq!(
        info.rtmps_ingestion_address.as_deref(),
        Some("rtmps://ingest.example.test/ingest")
    );
    assert_eq!(info.stream_name.as_deref(), Some("stream-name-fixture"));
    server.finish();
}

#[test]
fn forbidden_live_streaming_not_enabled_classifies_from_body_reason() {
    let body = error_body(
        "liveStreamingNotEnabled",
        "Live Streaming API access is disabled for this account.",
    );
    let server = RecordingJsonServer::start("HTTP/1.1 403 Forbidden", &body);
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(error, LiveApiError::LiveStreamingNotEnabled));
    assert!(!format!("{error}").contains("disabled for this account"));
    assert!(!format!("{error:?}").contains("disabled for this account"));
    server.finish();
}

#[test]
fn unauthorized_maps_to_authorization_rejected() {
    let body = error_body("authorizationRequired", "Invalid credentials.");
    let server = RecordingJsonServer::start("HTTP/1.1 401 Unauthorized", &body);
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(error, LiveApiError::AuthorizationRejected));
    assert!(!format!("{error}").contains("Invalid credentials"));
    server.finish();
}

#[test]
fn unknown_forbidden_reason_maps_to_api_rejected_reason_only() {
    let body = error_body("brandNewProviderReason", "A secret provider detail.");
    let server = RecordingJsonServer::start("HTTP/1.1 403 Forbidden", &body);
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    let LiveApiError::ApiRejected { status, reason } = &error else {
        panic!("expected ApiRejected, got {error}");
    };
    assert_eq!(*status, 403);
    assert_eq!(reason, "brandNewProviderReason");
    assert!(!format!("{error:?}").contains("secret provider detail"));
    assert!(!format!("{error}").contains("secret provider detail"));
    server.finish();
}

#[test]
fn redirects_are_not_followed() {
    let server = RecordingJsonServer::start("HTTP/1.1 301 Moved Permanently", "");
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(
        matches!(error, LiveApiError::ApiRejected { status: 301, .. }),
        "expected ApiRejected 301, got {error:?}"
    );
    server.finish();
}

#[test]
fn connection_failure_maps_to_network() {
    let transport = GoogleLiveApiTransport::with_base_url(
        format!("http://127.0.0.1:{}/youtube/v3/", closed_port()),
        AccessToken::new(ACCESS_TOKEN_FIXTURE),
    );
    let error = api_failure(transport.lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(error, LiveApiError::Network { .. }));
}

#[test]
fn malformed_insert_body_maps_to_malformed_response() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", "<html>");
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { ref field } if field == "liveStreamList"
    ));
    server.finish();
}

#[test]
fn empty_stream_list_is_none_while_absent_body_is_error() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", EMPTY_LIST_BODY);
    let stream = server
        .transport()
        .lookup_stream(STREAM_ID_FIXTURE)
        .expect("an empty list parses");
    assert!(stream.is_none());
    server.finish();

    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", "");
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { ref field } if field == "liveStreamList"
    ));
    server.finish();
}

#[test]
fn malformed_list_body_maps_to_malformed_response() {
    let server = RecordingJsonServer::start("HTTP/1.1 200 OK", r#"{"items": 5}"#);
    let error = api_failure(server.transport().lookup_stream(STREAM_ID_FIXTURE));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { ref field } if field == "liveStreamList"
    ));
    server.finish();
}

#[test]
fn query_values_are_percent_encoded() {
    assert_eq!(encode_query_value("abcXYZ09-_."), "abcXYZ09-_.");
    assert_eq!(encode_query_value("46-4.5/_~"), "46-4.5%2F_~");
    assert_eq!(encode_query_value("/?&="), "%2F%3F%26%3D");
    assert_eq!(encode_query_value("stream id+plus"), "stream%20id%2Bplus");
}
