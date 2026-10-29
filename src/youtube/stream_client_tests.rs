//! Offline pins for the streamList adapter surface (plan §E.4): Bearer-only
//! metadata, the official host/path constants, the tls + bounded-timeout
//! endpoint shape (construction only — nothing dials), and the rejected
//! non-ASCII token class. No test reaches Google.

use std::time::Duration;

use super::*;
use crate::youtube::ACCESS_TOKEN_FIXTURE;

#[test]
fn bearer_token_metadata_is_bearer_only() {
    let metadata = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    assert_eq!(
        metadata.to_str().expect("the value is ASCII"),
        "Bearer access-token-fixture"
    );
}

#[test]
fn bearer_token_metadata_holds_the_authorization_key_not_an_api_key() {
    let metadata = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    let rendered = metadata.to_str().expect("the value is ASCII");
    assert_eq!(AUTHORIZATION_METADATA_KEY, "authorization");
    assert!(rendered.starts_with("Bearer "));
    assert!(!rendered.contains("key="));
}

#[test]
fn non_ascii_access_token_fails_as_authorization_rejected() {
    let error = bearer_token_metadata(&AccessToken::new("non-ascii-token-Θ"))
        .expect_err("a non-ASCII token cannot become ASCII metadata");
    assert!(matches!(error, LiveApiError::AuthorizationRejected));
}

#[test]
fn stream_wire_constants_are_the_official_host_and_path() {
    assert_eq!(STREAM_ENDPOINT_URL, "https://youtube.googleapis.com");
    assert_eq!(
        STREAM_LIST_PATH,
        "/youtube.api.v3.V3DataLiveChatMessageService/StreamList"
    );
    tonic::codegen::http::uri::PathAndQuery::from_static(STREAM_LIST_PATH);
}

#[test]
fn stream_endpoint_builds_offline_with_connect_timeout_and_tls_shape() {
    let endpoint = stream_endpoint().expect("the endpoint builds offline");
    let rendered_uri = endpoint.uri().to_string();
    assert_eq!(rendered_uri.trim_end_matches('/'), STREAM_ENDPOINT_URL);
    assert_eq!(
        endpoint.get_connect_timeout(),
        Some(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS))
    );
}

#[test]
fn stream_request_carries_the_bearer_metadata_only() {
    let bearer = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    let request = stream_request(LiveChatMessageListRequest::default(), bearer);
    let metadata = request.metadata();
    assert_eq!(metadata.len(), 1);
    let rendered = metadata
        .get(AUTHORIZATION_METADATA_KEY)
        .expect("the authorization key is present")
        .to_str()
        .expect("the value is ASCII");
    assert!(rendered.starts_with("Bearer "));
}
