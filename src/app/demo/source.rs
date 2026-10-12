//! Delivers the fixed demo timeline through the existing MessageSource
//! seam. The pipeline loop calls next_message exactly once per frame
//! iteration, so counting calls IS frame counting — no clock of any kind
//! lives here, and the schedule can never drift from the pacing loop. The
//! cloned StopFlag is the timeline's END REQUEST through the pipeline's
//! existing shutdown seam: requesting it during the final drain call makes
//! the loop render the last frame, break at the next iteration top, and
//! finalize the sink exactly once.

use crate::app::demo::schedule::DemoTimeline;
use crate::app::pipeline::{MessageSource, StopFlag};
use crate::chat::{ChatError, ChatMessage};

pub(crate) struct DemoMessageSource {
    timeline: DemoTimeline,
    served: usize,
    current_frame_index: u64,
    stop_flag: StopFlag,
}

impl DemoMessageSource {
    /// ≤2-arguments rule; every counter starts empty/cleared.
    pub(crate) fn new(timeline: DemoTimeline, stop_flag: StopFlag) -> Self {
        Self {
            timeline,
            served: 0,
            current_frame_index: 0,
            stop_flag,
        }
    }

    /// Consumer-side accounting for `due_at`.
    fn served_count(&self) -> usize {
        self.served
    }
}

impl MessageSource for DemoMessageSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> {
        let frame_index = self.current_frame_index;
        let is_end = frame_index >= self.timeline.final_frame_index();
        self.current_frame_index += 1;
        if is_end {
            self.stop_flag.request_stop();
            return Ok(None);
        }
        let due = self.timeline.due_at(self.served_count(), frame_index);
        if due.is_some() {
            self.served += 1;
        }
        Ok(due)
    }
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod source_tests;
