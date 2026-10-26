//! The six prescribed Phase-06 chat wire pins (plan §4, caller-authorized
//! sibling roster): documented O7 query order + Bearer-only form (no
//! `key=` anywhere), the pageToken rule, page-field decoding, the typed
//! malformed outcome, the 403 `liveChatDisabled` classification, and the
//! O6 `snippet.liveChatId` read — all through the shared [`ChatWireServer`]
//! of the parent roster. Loopback `127.0.0.1` only; no test reaches Google.

use super::live_chat_test_fixtures::{
    live_chat_list_body, BROADCAST_ID_FIXTURE, LIVE_CHAT_ID_FIXTURE,
};
use super::*;

fn list_command(page_token: Option<&str>) -> ChatListCommand {
    ChatListCommand {
        live_chat_id: LIVE_CHAT_ID_FIXTURE.to_string(),
        page_token: page_token.map(str::to_string),
    }
}

#[test]
fn chat_list_sends_bearer_only_with_documented_query() {
    let server = ChatWireServer::start("HTTP/1.1 200 OK", &live_chat_list_body("[]"));
    let page = server
        .transport()
        .list_messages(&list_command(None))
        .expect("the canned page parses");
    assert!(page.items.is_empty());
    let expected_target = concat!(
        "/youtube/v3/liveChat/messages?liveChatId=live-chat-id-fixture",
        "&maxResults=200&part=snippet,authorDetails,id"
    );
    let head = server.head();
    assert_eq!(request_target(&head), expected_target);
    assert_eq!(
        header_value(&head, "authorization").as_deref(),
        Some("Bearer access-token-fixture")
    );
    assert!(!head.contains("key="), "no API key anywhere in the head");
    server.finish();
}

#[test]
fn chat_list_appends_page_token_only_when_present() {
    let with_token = ChatWireServer::start("HTTP/1.1 200 OK", &live_chat_list_body("[]"));
    with_token
        .transport()
        .list_messages(&list_command(Some("token-1-fixture")))
        .expect("the continuation page parses");
    assert!(request_target(&with_token.head()).ends_with("&pageToken=token-1-fixture"));
    with_token.finish();

    let first_page = ChatWireServer::start("HTTP/1.1 200 OK", &live_chat_list_body("[]"));
    first_page
        .transport()
        .list_messages(&list_command(None))
        .expect("the first page parses");
    assert!(!request_target(&first_page.head()).contains("pageToken="));
    first_page.finish();
}

#[test]
fn chat_list_response_decodes_page_fields() {
    let body = r#"{"items":[{"id":"msg-1","snippet":{"type":"textMessageEvent",
        "textMessageDetails":{"messageText":"hello"}},"authorDetails":{"displayName":"Alice"}}],
        "nextPageToken":"next-token-fixture","pollingIntervalMillis":13000}"#;
    let server = ChatWireServer::start("HTTP/1.1 200 OK", body);
    let page = server
        .transport()
        .list_messages(&list_command(None))
        .expect("a 200 reply decodes");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.next_page_token.as_deref(), Some("next-token-fixture"));
    assert_eq!(page.polling_interval_millis, Some(13000));
    assert!(page.offline_at.is_none());
    server.finish();
}

#[test]
fn chat_list_malformed_body_is_typed_error() {
    let server = ChatWireServer::start("HTTP/1.1 200 OK", "<html>");
    let outcome = server.transport().list_messages(&list_command(None));
    let Err(error) = outcome else {
        panic!("a non-JSON body is typed, never silent");
    };
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { ref field } if field == "liveChatMessageList"
    ));
    server.finish();
}

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
