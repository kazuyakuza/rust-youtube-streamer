//! The proto-side page mapping of Phase 07 (record §12; the D-4 policy over
//! the prost types): for every delivered item, in API order — the snippet
//! type must be TEXT (the resumable enum value), the text arm must carry a
//! non-blank `messageText`, the author is `displayName` else `channelId`
//! else skip; the carrier keeps the verbatim id and the RFC3339→epoch
//! second via the REUSED sibling parser. A `chatEndedEvent` item or a
//! present `offline_at` marks the batch as gracefully ended. Provider
//! payloads are never logged; `display_message` is NEVER promoted to text
//! (the text arm is the only sanctioned text source for textMessageEvent).

use super::live_chat_mapping::{non_blank_text, parse_rfc3339_seconds, ChatTextMessage};
use super::stream_proto::{
    DisplayedContent, LiveChatMessage, LiveChatMessageListResponse, LiveChatMessageSnippet,
    SnippetType,
};
use super::stream_transport::YieldedChatStreamBatch;

pub(crate) fn map_stream_batch(response: &LiveChatMessageListResponse) -> YieldedChatStreamBatch {
    YieldedChatStreamBatch {
        messages: response.items.iter().filter_map(map_item).collect(),
        next_page_token: response.next_page_token.clone(),
        offline_ended: batch_ended(response),
    }
}

/// The D-4 accept/skip policy in order; `None` = skipped (never ordinary
/// chat); API order preserved.
fn map_item(item: &LiveChatMessage) -> Option<ChatTextMessage> {
    let snippet = item.snippet.as_ref()?;
    if !is_text(snippet) {
        return None;
    }
    let text = message_text(snippet)?;
    let author = author_name(item)?;
    Some(ChatTextMessage {
        author,
        text,
        message_id: item.id.clone(),
        published_epoch: snippet
            .published_at
            .as_deref()
            .and_then(parse_rfc3339_seconds),
    })
}

fn is_text(snippet: &LiveChatMessageSnippet) -> bool {
    snippet.message_type == Some(SnippetType::TextMessageEvent as i32)
}

/// Text comes ONLY from the oneof text arm; blank/missing text skips.
fn message_text(snippet: &LiveChatMessageSnippet) -> Option<String> {
    snippet
        .displayed_content
        .as_ref()
        .and_then(|content| match content {
            DisplayedContent::TextMessage(details) => non_blank_text(&details.message_text),
        })
}

/// `displayName` else `channelId` else skip (no stable id) — the D-4 rule.
fn author_name(item: &LiveChatMessage) -> Option<String> {
    let author = item.author_details.as_ref()?;
    non_blank_text(&author.display_name).or_else(|| non_blank_text(&author.channel_id))
}

fn batch_ended(response: &LiveChatMessageListResponse) -> bool {
    response.offline_at.is_some() || response.items.iter().any(is_chat_ended_item)
}

fn is_chat_ended_item(item: &LiveChatMessage) -> bool {
    item.snippet
        .as_ref()
        .is_some_and(|snippet| snippet.message_type == Some(SnippetType::ChatEndedEvent as i32))
}

#[cfg(test)]
#[path = "live_chat_stream_mapping_tests.rs"]
mod live_chat_stream_mapping_tests;
