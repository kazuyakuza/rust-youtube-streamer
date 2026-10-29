//! The mockable Phase-06 chat lookup seam (record D-7, plan §3.3), now
//! O6-only after the Phase-07 rewire: the broadcast chat-id lookup answers
//! `Result<.., LiveApiError>` so the app funnels ONE typed source. The O7
//! `liveChatMessages.list` operation retired with the streamList switch
//! (record `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
//! §12; user ruling 2026-10-29); production retrieval runs through the
//! Phase-07 `ChatStreamTransport`. No streaming or concurrency semantics
//! enter this trait. `Ok(None)` from the lookup means only a verifiably
//! empty list answer; a body that fails to parse is a typed
//! `MalformedResponse`, never a silent `None`.
//!
//! Grounding: S5 (`liveBroadcast.snippet.liveChatId`) — citations + access
//! dates in `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md`
//! §1. Offline tests only — fakes/`127.0.0.1` loopback; no test reaches
//! Google.

use super::live_error::LiveApiError;

/// The chat lookup seam (record D-7): the O6 wire call returns
/// `Result<.., LiveApiError>` so the app funnels ONE typed source; no
/// streaming/concurrency semantics enter the trait.
pub(crate) trait LiveChatTransport {
    /// O6 documented field read: the FIRST item's `snippet.liveChatId`;
    /// `Ok(None)` only for a verifiably empty list answer.
    fn lookup_broadcast_chat_id(&self, broadcast_id: &str) -> Result<Option<String>, LiveApiError>;
}
