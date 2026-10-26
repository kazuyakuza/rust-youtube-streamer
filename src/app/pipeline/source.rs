use crate::chat::ChatMessage;

use super::source_error::ChatSourceError;

/// Supplies chat messages into the pipeline. The seam feeds the `demo`
/// command (the deterministic schedule) and the production `run` mode (the
/// worker-backed `ChatSource`); the pacing loop calls `next_message` exactly
/// at the TOP of each frame iteration (after the stop check, before
/// rendering) and consumes at most one due message per frame, so message
/// arrival is paced frame-by-frame and `ChatStore`'s bounded visible queue
/// keeps any burst from accumulating unbounded state.
pub(crate) trait MessageSource {
    /// Returns the next message due now, or `None` when nothing is due.
    /// Failure type (record D-8 widening): `Chat` = the chat-domain store
    /// error behind the log-first acceptance contract; `Retrieval` = the
    /// typed YouTube/chat retrieval failure from the production worker —
    /// never collapsed into a chat-log error.
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError>;
}
