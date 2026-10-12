use crate::chat::{ChatError, ChatMessage};

/// Supplies chat messages into the pipeline. The pacing loop calls
/// `next_message` exactly at the TOP of each frame iteration (after the stop
/// check, before rendering) and consumes at most one due message per frame,
/// so message arrival is paced frame-by-frame: the demo command (TODO Task
/// 3) feeds a deterministic schedule; a future production run mode feeds
/// live chat messages without changing this loop (TODO Task 4 bullet).
/// `ChatStore`'s bounded visible queue keeps any unbounded arrival from
/// accumulating unbounded state.
pub(crate) trait MessageSource {
    /// Returns the next message due now, or `None` when nothing is due.
    /// The only failure class available in this phase is the chat-domain
    /// error; TODO Task 4 may widen this error type when a real network
    /// listener exists (documented evolution point).
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError>;
}
