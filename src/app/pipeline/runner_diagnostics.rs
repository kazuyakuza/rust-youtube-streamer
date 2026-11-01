//! Inherent diagnostics hooks for the frame pipeline: registered as a child
//! of `runner` so the parent's private `diagnostics` field stays reachable
//! without widening the aggregate's visibility. Every hook is guarded by the
//! `Option` read that IS the entire disabled-mode cost.

use crate::chat::ChatMessage;
use crate::streaming::FrameSink;

use super::FramePipeline;

impl<S: FrameSink> FramePipeline<S> {
    /// Records one dequeued message into the shared aggregate.
    pub(super) fn note_message_dequeued(&self, message: &ChatMessage) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_message_dequeued(message);
        }
    }

    pub(super) fn note_render_started(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_render_started();
        }
    }

    pub(super) fn note_render_completed(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_render_completed();
        }
    }

    pub(super) fn note_write_started(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_write_started();
        }
    }

    pub(super) fn note_write_completed(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_write_completed();
        }
    }

    /// Records one non-emitting frame-loop tick (interval and jitter).
    pub(super) fn note_frame_tick(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_frame_tick();
        }
    }

    /// Emits one aggregate summary row when the configured period elapsed.
    pub(super) fn maybe_emit_summary(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.maybe_emit_summary();
        }
    }
}

#[cfg(test)]
#[path = "runner_diagnostics_tests.rs"]
mod runner_diagnostics_tests;
