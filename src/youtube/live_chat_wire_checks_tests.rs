//! The surviving Phase-06 chat wire pins (plan §C.7): the O6
//! `snippet.liveChatId` read — extended with the REST-side Bearer-only
//! header form (no `key=` anywhere) — and the 403 `liveChatDisabled`
//! classification. The four O7 `liveChatMessages.list` pins retired with
//! the streamList switch. Loopback `127.0.0.1` only; no test reaches
//! Google.

use super::live_chat_test_fixtures::{
    ACCESS_TOKEN_FIXTURE, BROADCAST_ID_FIXTURE, LIVE_CHAT_ID_FIXTURE,
};
use super::*;

#[test]
fn broadcast_chat_lookup_reads_snippet_live_chat_id() {
    let server = ChatWireServer::start(
        "HTTP/1.1 200 OK",
        r#"{"items":[{"snippet":{"liveChatId":"live-chat-id-fixture"}}]}"#,
    );
    let resolved = server
        .transport()
        .lookup_broadcast_chat_id(BROADCAST_ID_FIXTURE)
        .expect("the canned broadcast parses");
    assert_eq!(resolved.as_deref(), Some(LIVE_CHAT_ID_FIXTURE));
    let head = server.head();
    let expected_authorization = format!("Bearer {ACCESS_TOKEN_FIXTURE}");
    assert_eq!(
        header_value(&head, "authorization").as_deref(),
        Some(expected_authorization.as_str())
    );
    assert!(!head.contains("key="), "no API key anywhere in the head");
    server.finish();

    let empty = ChatWireServer::start("HTTP/1.1 200 OK", r#"{"items":[]}"#);
    let absent = empty
        .transport()
        .lookup_broadcast_chat_id(BROADCAST_ID_FIXTURE)
        .expect("the empty list parses");
    assert!(absent.is_none(), "an empty list answer is Ok(None)");
    empty.finish();
}

#[test]
fn chat_disabled_error_body_classifies_as_live_chat_disabled() {
    let body = r#"{"error":{"code":403,"message":"chat disabled",
        "errors":[{"reason":"liveChatDisabled"}]}}"#;
    let server = ChatWireServer::start("HTTP/1.1 403 Forbidden", body);
    let outcome = server
        .transport()
        .lookup_broadcast_chat_id(BROADCAST_ID_FIXTURE);
    let Err(error) = outcome else {
        panic!("a 403 classifies into a typed failure");
    };
    assert!(matches!(error, LiveApiError::ChatDisabled));
    server.finish();
}
