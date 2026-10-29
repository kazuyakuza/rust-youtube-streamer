//! The provider→carrier page mapping of Phase 06 (record D-4, plan §3.7):
//! the documented skip/accept policy per item, in order — non-`textMessageEvent`
//! types skip (tombstones, `chatEndedEvent`, every paid/membership/gift/ban/
//! poll event AND any unknown string), missing/blank message text skips
//! (never ordinary chat), author = `displayName` else `channelId` else skip
//! — then the carrier keeps the verbatim id and the optional epoch second
//! from `parse_rfc3339_seconds` (the sibling `live_chat_rfc3339` module,
//! caller-authorized split). Page end = any `chatEndedEvent` or a present
//! `offlineAt`; page order preserves `items[]` API order.
//!
//! Grounding: S1/S2 citations + access dates in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — no test reaches Google.

use super::live_chat_models::{LiveChatMessageList, LiveChatMessageRecord, LiveChatMessageSnippet};

#[path = "live_chat_rfc3339.rs"]
mod live_chat_rfc3339;

pub(crate) use self::live_chat_rfc3339::parse_rfc3339_seconds;

const TEXT_MESSAGE_TYPE: &str = "textMessageEvent";
const CHAT_ENDED_TYPE: &str = "chatEndedEvent";

/// The youtube-owned carrier the mapping returns (record D-4): the app
/// side constructs `ChatMessage` in Task 3, so `src/youtube/` stays
/// free of any `crate::chat` dependency. Debug only — never `Display`.
#[derive(Debug)]
pub(crate) struct ChatTextMessage {
    pub(crate) author: String,
    pub(crate) text: String,
    pub(crate) message_id: Option<String>,
    pub(crate) published_epoch: Option<u64>,
}

/// One mapped page: accepted carriers in API order + the graceful
/// end-of-chat marker (`chatEndedEvent` or `offlineAt`, record D-4 step 5).
#[derive(Debug)]
pub(crate) struct ChatPageOutcome {
    pub(crate) messages: Vec<ChatTextMessage>,
    pub(crate) chat_ended: bool,
}

/// Maps one page; order-preserving, 1-arg (plan §3.7 "2-arg-free").
pub(crate) fn map_page(list: &LiveChatMessageList) -> ChatPageOutcome {
    let messages = list.items.iter().filter_map(map_record).collect();
    ChatPageOutcome {
        messages,
        chat_ended: page_marked_ended(list),
    }
}

/// D-4 steps 1–4 in order; `None` = skipped by policy (never ordinary chat).
fn map_record(record: &LiveChatMessageRecord) -> Option<ChatTextMessage> {
    let snippet = record.snippet.as_ref()?;
    if !is_text_message(snippet) {
        return None;
    }
    let text = non_blank_message_text(snippet)?;
    let author = resolve_author(record)?;
    Some(ChatTextMessage {
        author,
        text,
        message_id: record.id.clone(),
        published_epoch: snippet
            .published_at
            .as_deref()
            .and_then(parse_rfc3339_seconds),
    })
}

fn is_text_message(snippet: &LiveChatMessageSnippet) -> bool {
    snippet.message_type.as_deref() == Some(TEXT_MESSAGE_TYPE)
}

/// Blank/missing text is a malformed payload — skipped, never ordinary chat;
/// the accepted text is preserved verbatim (untrimmed).
fn non_blank_message_text(snippet: &LiveChatMessageSnippet) -> Option<String> {
    snippet
        .text_message_details
        .as_ref()
        .and_then(|details| non_blank_text(&details.message_text))
}

/// D-4 step 3: `displayName` else `channelId` else skip (no stable id).
fn resolve_author(record: &LiveChatMessageRecord) -> Option<String> {
    let author = record.author_details.as_ref()?;
    non_blank_text(&author.display_name).or_else(|| non_blank_text(&author.channel_id))
}

pub(super) fn non_blank_text(value: &Option<String>) -> Option<String> {
    let text = value.as_ref()?;
    if text.trim().is_empty() {
        return None;
    }
    Some(text.clone())
}

fn page_marked_ended(list: &LiveChatMessageList) -> bool {
    list_shows_offline(list) || list.items.iter().any(marks_ended)
}

fn list_shows_offline(list: &LiveChatMessageList) -> bool {
    list.offline_at.is_some()
}

fn marks_ended(record: &LiveChatMessageRecord) -> bool {
    let Some(snippet) = record.snippet.as_ref() else {
        return false;
    };
    snippet.message_type.as_deref() == Some(CHAT_ENDED_TYPE)
}

#[cfg(test)]
#[path = "live_chat_mapping_tests.rs"]
mod live_chat_mapping_tests;
