//! The mockable Phase-06 chat retrieval seam (record D-7, plan §3.3):
//! both chat wire calls — the O6 broadcast chat-id lookup and the O7
//! `liveChatMessages.list` — answer `Result<.., LiveApiError>` so the app
//! funnels ONE typed source; no streaming or concurrency semantics enter
//! the trait (the worker consuming it is Task 3). `Ok(None)` from the
//! lookup means only a verifiably empty list answer; a body that fails to
//! parse is a typed `MalformedResponse`, never a silent `None`.
//!
//! Grounding: S1 (`liveChatMessage` resource), S2 (`liveChatMessages.list`),
//! S5 (`liveBroadcast.snippet.liveChatId`) — citations + access dates in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — fakes/`127.0.0.1` loopback; no test reaches Google.

use super::live_chat_models::{ChatListCommand, LiveChatMessageList};
use super::live_error::LiveApiError;

/// The chat retrieval seam (record D-7): both chat wire calls (O6 lookup
/// and O7 list) return `Result<.., LiveApiError>` so the app funnels ONE
/// typed source; no streaming/concurrency semantics enter the trait.
pub(crate) trait LiveChatTransport {
    /// O6 documented field read: the FIRST item's `snippet.liveChatId`;
    /// `Ok(None)` only for a verifiably empty list answer.
    fn lookup_broadcast_chat_id(&self, broadcast_id: &str) -> Result<Option<String>, LiveApiError>;

    /// O7 `liveChatMessages.list` (D-3 wire form); the typed error covers
    /// every §1.6 lifecycle class.
    fn list_messages(&self, command: &ChatListCommand)
        -> Result<LiveChatMessageList, LiveApiError>;
}
