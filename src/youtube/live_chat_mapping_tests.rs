//! Mapping pins for the Phase-06 provider→carrier page mapping (plan
//! §5.1): the D-4 skip/accept policy in order, author fallback, id/
//! timestamp carriers, API-order preservation, the chat-ended page marker
//! and the documented RFC 3339 UTC parse/reject forms. Offline only.

use serde_json::json;

use super::*;

fn list_from_items(items: serde_json::Value) -> LiveChatMessageList {
    serde_json::from_value(json!({ "items": items })).expect("the canned page parses")
}

fn text_message_record() -> serde_json::Value {
    json!({
        "id": "msg-1",
        "snippet": {
            "type": "textMessageEvent",
            "publishedAt": "2026-10-25T12:30:45Z",
            "textMessageDetails": {"messageText": "Hello stream"}
        },
        "authorDetails": {"displayName": "Alice", "channelId": "UC-fixture"}
    })
}

#[test]
fn text_message_maps_full_shape_with_id_and_epoch_timestamp() {
    let outcome = map_page(&list_from_items(json!([text_message_record()])));
    assert_eq!(outcome.messages.len(), 1);
    let message = &outcome.messages[0];
    assert_eq!(message.author, "Alice");
    assert_eq!(message.text, "Hello stream");
    assert_eq!(message.message_id.as_deref(), Some("msg-1"));
    assert_eq!(message.published_epoch, Some(1_792_931_445));
    assert!(!outcome.chat_ended);
}

#[test]
fn missing_display_name_falls_back_to_channel_id() {
    let record = json!({
        "id": "msg-2",
        "snippet": {
            "type": "textMessageEvent",
            "textMessageDetails": {"messageText": "fallback author"}
        },
        "authorDetails": {"channelId": "UC-fallback"}
    });
    let outcome = map_page(&list_from_items(json!([record])));
    assert_eq!(outcome.messages.len(), 1);
    assert_eq!(outcome.messages[0].author, "UC-fallback");
    assert_eq!(outcome.messages[0].published_epoch, None);
}

#[test]
fn missing_both_author_fields_is_skipped() {
    let record = json!({
        "snippet": {
            "type": "textMessageEvent",
            "textMessageDetails": {"messageText": "no author"}
        },
        "authorDetails": {}
    });
    let outcome = map_page(&list_from_items(json!([record])));
    assert!(outcome.messages.is_empty());
}

#[test]
fn non_text_event_types_are_skipped_including_tombstone_and_unknown() {
    let records = json!([
        {"snippet": {"type": "superChatEvent"}},
        {"snippet": {"type": "tombstone"}},
        {"snippet": {"type": "futureEvent"}}
    ]);
    let outcome = map_page(&list_from_items(records));
    assert!(outcome.messages.is_empty());
}

#[test]
fn malformed_text_message_payload_is_skipped_not_mapped_as_ordinary() {
    let record = json!({
        "id": "msg-3",
        "snippet": {"type": "textMessageEvent"},
        "authorDetails": {"displayName": "Alice"}
    });
    let outcome = map_page(&list_from_items(json!([record])));
    assert!(outcome.messages.is_empty());
}

#[test]
fn blank_or_missing_message_text_is_skipped() {
    let blank = json!({
        "snippet": {
            "type": "textMessageEvent",
            "textMessageDetails": {"messageText": "   "}
        },
        "authorDetails": {"displayName": "Alice"}
    });
    let empty = json!({
        "snippet": {
            "type": "textMessageEvent",
            "textMessageDetails": {"messageText": ""}
        },
        "authorDetails": {"displayName": "Bob"}
    });
    let outcome = map_page(&list_from_items(json!([blank, empty])));
    assert!(outcome.messages.is_empty());
}

#[test]
fn page_mapping_preserves_api_order() {
    let author = |name: &str| {
        json!({
            "snippet": {
                "type": "textMessageEvent",
                "textMessageDetails": {"messageText": "hi"}
            },
            "authorDetails": {"displayName": name}
        })
    };
    let outcome = map_page(&list_from_items(json!([
        author("first"),
        author("second"),
        author("third")
    ])));
    let authors: Vec<&str> = outcome
        .messages
        .iter()
        .map(|message| message.author.as_str())
        .collect();
    assert_eq!(authors, ["first", "second", "third"]);
}

#[test]
fn chat_ended_event_marks_page_ended_without_a_message() {
    let outcome = map_page(&list_from_items(json!([
        {"snippet": {"type": "chatEndedEvent"}}
    ])));
    assert!(outcome.messages.is_empty());
    assert!(outcome.chat_ended);
}

#[test]
fn offline_at_marks_page_ended() {
    let list: LiveChatMessageList =
        serde_json::from_value(json!({"items": [], "offlineAt": "2026-10-25T12:05:00Z"}))
            .expect("the offline page parses");
    let outcome = map_page(&list);
    assert!(outcome.messages.is_empty());
    assert!(outcome.chat_ended);
}

#[test]
fn absent_or_unparseable_published_at_maps_without_timestamp() {
    let absent = json!({
        "id": "msg-4",
        "snippet": {
            "type": "textMessageEvent",
            "textMessageDetails": {"messageText": "no timestamp"}
        },
        "authorDetails": {"displayName": "Alice"}
    });
    let unparseable = json!({
        "id": "msg-5",
        "snippet": {
            "type": "textMessageEvent",
            "publishedAt": "not-a-timestamp",
            "textMessageDetails": {"messageText": "bad timestamp"}
        },
        "authorDetails": {"displayName": "Bob"}
    });
    let outcome = map_page(&list_from_items(json!([absent, unparseable])));
    assert_eq!(outcome.messages.len(), 2);
    assert_eq!(outcome.messages[0].published_epoch, None);
    assert_eq!(outcome.messages[1].published_epoch, None);
}

#[test]
fn rfc3339_parsing_covers_documented_utc_forms() {
    assert_eq!(
        parse_rfc3339_seconds("2026-10-25T12:30:45Z"),
        Some(1_792_931_445)
    );
    assert_eq!(
        parse_rfc3339_seconds("2026-10-25T12:30:45.123Z"),
        Some(1_792_931_445)
    );
    assert_eq!(parse_rfc3339_seconds("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse_rfc3339_seconds("2026-02-29T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-04-31T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-13-01T00:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T24:00:00Z"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T12:30:45"), None);
    assert_eq!(parse_rfc3339_seconds("2026-10-25T12:30:45.1234Z"), None);
}
