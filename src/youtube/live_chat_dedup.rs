//! Provider-ID dedup window (record D-5, plan §3.8): the LAST
//! `CHAT_DEDUP_WINDOW` seen message ids, `record` answering "unseen" with
//! `true` exactly once per id. The worker owns a per-session in-memory
//! instance (Task 3); tombstone/gift id re-use never conflicts because
//! the D-4 skip policy runs BEFORE the window inside the worker flow.
//! Pure id-store — no ordering, no persistence, no logging.
//!
//! Grounding: S1 notes gift events may re-use an id (citation + access
//! date in `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md`
//! §1.5). Offline tests only — no network anywhere.

use std::collections::VecDeque;

const CHAT_DEDUP_WINDOW: usize = 512;

/// Provider-ID dedup window (record D-5): the LAST `CHAT_DEDUP_WINDOW`
/// seen ids; the worker owns a per-session instance (Task 3). Tombstones
/// and gift-event re-use never conflict because the skip policy runs
/// BEFORE the window inside the worker flow.
pub(crate) struct ChatDedupWindow {
    seen: VecDeque<String>,
}

impl ChatDedupWindow {
    pub(crate) fn new() -> Self {
        Self {
            seen: VecDeque::new(),
        }
    }

    /// `true` exactly when the id was UNSEEN; then remembers it and keeps
    /// the window bounded at `CHAT_DEDUP_WINDOW` ("last 512" eviction).
    pub(crate) fn record(&mut self, id: &str) -> bool {
        if self.seen.iter().any(|seen_id| seen_id == id) {
            return false;
        }
        self.seen.push_back(id.to_string());
        if self.seen.len() > CHAT_DEDUP_WINDOW {
            self.seen.pop_front();
        }
        true
    }
}

#[cfg(test)]
#[path = "live_chat_dedup_tests.rs"]
mod live_chat_dedup_tests;
