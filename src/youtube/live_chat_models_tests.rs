//! Decode pins for the tolerant Phase-06 chat models (plan §5.4): the
//! documented page fields, the `type` rename, author details, the empty/
//! missing-`items` tolerance and the unknown-type verbatim decode the
//! skip policy relies on. Offline only — canned bodies, no network.

use super::*;

#[test]
fn full_list_response_decodes_token_interval_and_items() {
    let body = r#"{
        "items": [{"id": "msg-1"}],
        "nextPageToken": "next-token-fixture",
        "pollingIntervalMillis": 13000,
        "offlineAt": "2026-10-25T12:00:00Z"
    }"#;
    let list: LiveChatMessageList = serde_json::from_str(body).expect("the canned page parses");
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.next_page_token.as_deref(), Some("next-token-fixture"));
    assert_eq!(list.polling_interval_millis, Some(13000));
    assert_eq!(list.offline_at.as_deref(), Some("2026-10-25T12:00:00Z"));
}

#[test]
fn empty_items_list_decodes_without_records() {
    let list: LiveChatMessageList =
        serde_json::from_str(r#"{"items":[]}"#).expect("the empty page parses");
    assert!(list.items.is_empty());
    assert!(list.next_page_token.is_none());
    assert!(list.polling_interval_millis.is_none());
    assert!(list.offline_at.is_none());
}

#[test]
fn missing_items_key_decodes_as_empty_items() {
    let list: LiveChatMessageList = serde_json::from_str("{}").expect("the tolerant page parses");
    assert!(
        list.items.is_empty(),
        "a missing items key is an empty list"
    );
}

#[test]
fn text_message_record_decodes_type_rename_and_text() {
    let list: LiveChatMessageList = serde_json::from_str(
        r#"{"items":[{"snippet":{"type":"textMessageEvent",
        "textMessageDetails":{"messageText":"hello"}}}]}"#,
    )
    .expect("the text record parses");
    let snippet = list.items[0].snippet.as_ref().expect("the snippet part");
    assert_eq!(snippet.message_type.as_deref(), Some("textMessageEvent"));
    let details = snippet
        .text_message_details
        .as_ref()
        .expect("the text details part");
    assert_eq!(details.message_text.as_deref(), Some("hello"));
}

#[test]
fn author_details_decodes_display_name_and_channel_id() {
    let list: LiveChatMessageList = serde_json::from_str(
        r#"{"items":[{"authorDetails":{"displayName":"Alice","channelId":"UC-fixture"}}]}"#,
    )
    .expect("the author record parses");
    let author = list.items[0]
        .author_details
        .as_ref()
        .expect("the author part");
    assert_eq!(author.display_name.as_deref(), Some("Alice"));
    assert_eq!(author.channel_id.as_deref(), Some("UC-fixture"));
}

#[test]
fn unknown_type_strings_decode_verbatim_for_the_skip_policy() {
    let list: LiveChatMessageList =
        serde_json::from_str(r#"{"items":[{"snippet":{"type":"futureEvent"}}]}"#)
            .expect("the unknown type parses");
    let snippet = list.items[0].snippet.as_ref().expect("the snippet part");
    assert_eq!(snippet.message_type.as_deref(), Some("futureEvent"));
}
