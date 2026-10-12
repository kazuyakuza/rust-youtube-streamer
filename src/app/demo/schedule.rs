//! The fixed, documented demo schedule: a static ordered set of chat
//! messages, each due at an exact frame index of the demo timeline. Pure
//! data + pure frame-index arithmetic: unit-testable without any clock,
//! sleep, or pipeline (TODO Task 3; plan §4.2). Deltas: the timeline leads
//! with LEAD_IN_FRAMES black frames, admits at most one message every
//! MESSAGE_GAP_FRAMES frames, holds a TAIL_FRAMES tail after the last
//! delivery, and carries more messages than the example config's derived
//! visible capacity so FIFO eviction is exercised in the shipped demo.

use crate::chat::ChatMessage;

pub(crate) const DEMO_MESSAGE_TOTAL: usize = 28;
pub(crate) const LEAD_IN_FRAMES: u64 = 30;
pub(crate) const MESSAGE_GAP_FRAMES: u64 = 15;
pub(crate) const TAIL_FRAMES: u64 = 60;
/// 1-based message index whose text is the long line that classes through
/// the renderer's horizontal clipping path (pixel-level assertions for
/// clipping live in the renderer test suite, brief §9.5).
pub(crate) const LONG_LINE_MESSAGE_INDEX: usize = 15;

const MESSAGE_AUTHORS: [&str; 4] = ["alice", "bob", "charlie", "dana"];
const TEXT_PREFIX: &str = "demo message";
const LONG_LINE_PIECE: &str = "0123456789";
const LONG_LINE_PIECES: usize = 25;
const MESSAGE_ID_PREFIX: &str = "demo-";

pub(crate) struct ScheduledMessage {
    pub(crate) due_frame_index: u64,
    pub(crate) message: ChatMessage,
}

pub(crate) struct DemoTimeline {
    deliveries: Vec<ScheduledMessage>,
}

impl DemoTimeline {
    /// The only production constructor: the fixed message set of the
    /// shipped demo timeline.
    pub(crate) fn fixed_timeline() -> Self {
        Self {
            deliveries: build_deliveries(),
        }
    }

    /// Test-only constructor injecting the deliveries.
    #[cfg(test)]
    pub(crate) fn raw(deliveries: Vec<ScheduledMessage>) -> Self {
        Self { deliveries }
    }

    /// The next not-yet-served message once its due frame has arrived; at
    /// most ONE message per call (the pacing loop's one-per-frame rule).
    pub(crate) fn due_at(&self, already_served: usize, frame_index: u64) -> Option<ChatMessage> {
        let scheduled = self.deliveries.get(already_served)?;
        delivery_due(scheduled, frame_index)
    }

    /// Index of the LAST rendered frame: lead-in, one gap between each pair
    /// of the DEMO_MESSAGE_TOTAL deliveries, then the tail. Example config
    /// (fps 30): 30 + 27*15 + 60 = 495.
    pub(crate) fn final_frame_index(&self) -> u64 {
        LEAD_IN_FRAMES + ((DEMO_MESSAGE_TOTAL as u64) - 1) * MESSAGE_GAP_FRAMES + TAIL_FRAMES
    }

    /// Frames rendered 0..=final_frame_index; the example config renders
    /// 496 frames (≈16.5 s at 30 FPS).
    // Test-only helper: the production stop path uses final_frame_index.
    #[cfg(test)]
    pub(crate) fn rendered_frame_count(&self) -> u64 {
        self.final_frame_index() + 1
    }

    // Test-only helper: the production pipeline never counts deliveries.
    #[cfg(test)]
    pub(crate) fn total_deliveries(&self) -> usize {
        self.deliveries.len()
    }
}

/// Production message set, exposed to the demo test suites only.
#[cfg(test)]
pub(crate) fn scheduled_messages_for_tests() -> Vec<ScheduledMessage> {
    build_deliveries()
}

fn build_deliveries() -> Vec<ScheduledMessage> {
    (0..DEMO_MESSAGE_TOTAL).map(scheduled_message).collect()
}

fn scheduled_message(index: usize) -> ScheduledMessage {
    let number = index + 1;
    let author = MESSAGE_AUTHORS[index % MESSAGE_AUTHORS.len()];
    let message = ChatMessage::new(author, &message_text(number))
        .with_message_id(&format!("{MESSAGE_ID_PREFIX}{number:02}"));
    ScheduledMessage {
        due_frame_index: LEAD_IN_FRAMES + (index as u64) * MESSAGE_GAP_FRAMES,
        message,
    }
}

fn message_text(number: usize) -> String {
    if is_long_line_slot(number) {
        return LONG_LINE_PIECE.repeat(LONG_LINE_PIECES);
    }
    format!("{TEXT_PREFIX} {number:02}")
}

fn is_long_line_slot(number: usize) -> bool {
    number == LONG_LINE_MESSAGE_INDEX
}

fn delivery_due(scheduled: &ScheduledMessage, frame_index: u64) -> Option<ChatMessage> {
    if scheduled.due_frame_index <= frame_index {
        return Some(scheduled.message.clone());
    }
    None
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod schedule_tests;
