//! Provider decode views and the fixed wire form of the Phase-06 chat
//! retrieval (record D-1/D-3, plan §3.1): tolerant `liveChatMessages.list`
//! response models — unknown fields ignored, a missing `items` key decodes
//! empty (the Phase-05 frozen precedent) — plus the two path builders with
//! the documented query order: `liveChatId`, `maxResults`, `part` fixed,
//! `pageToken` appended LAST only when a token exists; never `hl`,
//! `profileImageSize` or `key=` (Bearer only, S2/S3).
//!
//! Grounding: S1/S2/S5 citations + access dates live in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — fakes/`127.0.0.1` loopback; no test reaches Google.

use serde::Deserialize;

use super::api_request::encode_query_value;

/// Poll budget of ONE list call (documented minimum, S2; per-call 5-unit cost).
pub(crate) const MAX_CHAT_RESULTS: u64 = 200;
/// The `MalformedResponse` field label of the chat list answer (transport parse call).
pub(super) const CHAT_MESSAGES_RESOURCE: &str = "liveChatMessageList";

/// One `liveChatMessages.list` page (O7); `items` defaults to empty.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageList {
    #[serde(default)]
    pub(crate) items: Vec<LiveChatMessageRecord>,
    pub(crate) next_page_token: Option<String>,
    pub(crate) polling_interval_millis: Option<u64>,
    pub(crate) offline_at: Option<String>,
}

/// One `liveChatMessage` resource view (S1); documented fields only.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageRecord {
    pub(crate) id: Option<String>,
    pub(crate) snippet: Option<LiveChatMessageSnippet>,
    pub(crate) author_details: Option<LiveChatAuthorDetails>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageSnippet {
    #[serde(rename = "type")]
    pub(crate) message_type: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(super) text_message_details: Option<TextMessageDetails>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextMessageDetails {
    pub(crate) message_text: Option<String>,
}

/// `authorDetails` (S1): `displayName`/`channelId` read; unknown fields ignored.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatAuthorDetails {
    pub(crate) display_name: Option<String>,
    pub(crate) channel_id: Option<String>,
}

/// The O7 list command (record D-3): id + optional continuation token.
#[derive(Clone)]
pub(crate) struct ChatListCommand {
    pub(crate) live_chat_id: String,
    pub(crate) page_token: Option<String>,
}

/// The O7 wire path+query (D-3 fixed order; `pageToken` last, only when present).
pub(crate) fn chat_messages_path(command: &ChatListCommand) -> String {
    let mut path = format!(
        "liveChat/messages?liveChatId={}&maxResults={}&part=snippet,authorDetails,id",
        encode_query_value(&command.live_chat_id),
        MAX_CHAT_RESULTS,
    );
    if let Some(token) = command.page_token.as_deref() {
        path.push_str("&pageToken=");
        path.push_str(&encode_query_value(token));
    }
    path
}

/// The O6 broadcast lookup path+query (`part=snippet` owns `liveChatId`, S5).
pub(super) fn broadcast_chat_lookup_path(broadcast_id: &str) -> String {
    format!(
        "liveBroadcasts?part=snippet&id={}",
        encode_query_value(broadcast_id)
    )
}

#[cfg(test)]
#[path = "live_chat_models_tests.rs"]
mod live_chat_models_tests;
