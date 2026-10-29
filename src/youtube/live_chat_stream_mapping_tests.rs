//! Policy pins for the proto-side stream mapping (plan §E.2; record D-4/
//! §12): text-only acceptance in API order, author fallback, blank/missing
//! text skips, tombstone/ended/unknown types skip, the graceful end marker,
//! the carried id + epoch and the resume-token pass-through. Offline only.

use super::*;
use crate::youtube::stream_proto::{LiveChatMessageAuthorDetails, LiveChatTextMessageDetails};
use crate::youtube::{
    ended_item, response_offline, response_plain, response_with_token, text_item, tombstone_item,
    PUBLISHED_AT_FIXTURE,
};

/// The epoch the EXISTING parser roster pins for the shared fixture literal.
const PUBLISHED_EPOCH: u64 = 1_792_931_445;

fn blank_text_item() -> LiveChatMessage {
    LiveChatMessage {
        snippet: Some(LiveChatMessageSnippet {
            message_type: Some(SnippetType::TextMessageEvent as i32),
            displayed_content: Some(DisplayedContent::TextMessage(LiveChatTextMessageDetails {
                message_text: Some("   ".to_string()),
            })),
            ..Default::default()
        }),
        author_details: Some(LiveChatMessageAuthorDetails {
            display_name: Some("author-fixture".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn text_message_event_maps_to_a_carrier() {
    let batch = map_stream_batch(&response_plain(vec![text_item("msg-1")]));
    assert_eq!(batch.messages.len(), 1);
    let message = &batch.messages[0];
    assert_eq!(message.author, "author-fixture");
    assert_eq!(message.text, "text-fixture");
    assert!(batch.messages[0].message_id.is_some());
    assert!(!batch.offline_ended);
}

#[test]
fn display_name_missing_falls_back_to_channel_id() {
    let mut item = text_item("msg-2");
    let author = item
        .author_details
        .as_mut()
        .expect("the fixture has an author");
    author.display_name = None;
    let batch = map_stream_batch(&response_plain(vec![item]));
    assert_eq!(batch.messages.len(), 1);
    assert_eq!(batch.messages[0].author, "channel-id-fixture");
}

#[test]
fn both_author_sources_missing_skips_the_item() {
    let mut item = text_item("msg-3");
    let author = item
        .author_details
        .as_mut()
        .expect("the fixture has an author");
    author.display_name = None;
    author.channel_id = None;
    let batch = map_stream_batch(&response_plain(vec![item]));
    assert!(batch.messages.is_empty());
}

#[test]
fn blank_message_text_is_skipped() {
    let batch = map_stream_batch(&response_plain(vec![blank_text_item()]));
    assert!(batch.messages.is_empty());
}

#[test]
fn tombstone_items_are_skipped() {
    let batch = map_stream_batch(&response_plain(vec![tombstone_item("msg-4")]));
    assert!(batch.messages.is_empty());
    assert!(!batch.offline_ended);
}

#[test]
fn chat_ended_event_marks_the_batch_ended_and_skips_its_item() {
    let batch = map_stream_batch(&response_plain(vec![ended_item()]));
    assert!(batch.messages.is_empty());
    assert!(batch.offline_ended);
}

#[test]
fn offline_at_marks_the_batch_ended() {
    let batch = map_stream_batch(&response_offline(vec![]));
    assert!(batch.messages.is_empty());
    assert!(batch.offline_ended);
}

#[test]
fn unknown_type_values_are_skipped() {
    let mut item = text_item("msg-5");
    let snippet = item.snippet.as_mut().expect("the fixture has a snippet");
    snippet.message_type = Some(99);
    let batch = map_stream_batch(&response_plain(vec![item]));
    assert!(batch.messages.is_empty());
}

#[test]
fn items_keep_the_api_order() {
    let mut first = text_item("msg-1");
    let mut second = text_item("msg-2");
    let mut third = text_item("msg-3");
    for (item, author) in [
        (&mut first, "first"),
        (&mut second, "second"),
        (&mut third, "third"),
    ] {
        let details = item
            .author_details
            .as_mut()
            .expect("the fixture has an author");
        details.display_name = Some(author.to_string());
    }
    let batch = map_stream_batch(&response_plain(vec![first, second, third]));
    let authors: Vec<&str> = batch
        .messages
        .iter()
        .map(|message| message.author.as_str())
        .collect();
    assert_eq!(authors, ["first", "second", "third"]);
}

#[test]
fn the_carrier_records_ids_and_epochs() {
    let item = text_item("msg-6");
    let expected_epoch = parse_rfc3339_seconds(PUBLISHED_AT_FIXTURE);
    assert_eq!(expected_epoch, Some(PUBLISHED_EPOCH));
    let batch = map_stream_batch(&response_plain(vec![item]));
    let message = &batch.messages[0];
    assert_eq!(message.message_id.as_deref(), Some("msg-6"));
    assert_eq!(message.published_epoch, expected_epoch);
}

#[test]
fn the_next_page_token_carries_forward_when_present_and_stays_none_when_absent() {
    let present = map_stream_batch(&response_with_token(vec![], "token-fixture"));
    assert_eq!(present.next_page_token.as_deref(), Some("token-fixture"));
    let absent = map_stream_batch(&response_plain(vec![]));
    assert!(absent.next_page_token.is_none());
}

#[test]
fn missing_id_maps_to_none_message_id() {
    let mut item = text_item("msg-7");
    item.id = None;
    let batch = map_stream_batch(&response_plain(vec![item]));
    assert_eq!(batch.messages.len(), 1);
    assert!(batch.messages[0].message_id.is_none());
}
