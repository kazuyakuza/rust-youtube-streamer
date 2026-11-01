//! The continuous frame pipeline: drains one due message, renders the
//! complete visible chat text, writes exactly one full RGB24 frame per
//! paced iteration, and finalizes the sink on stop (TODO Task 2; plan
//! decisions S9/S10). It never names `FfmpegProcess` — the sink arrives as
//! `S: FrameSink` from the caller. The diagnostics hook methods live in the
//! `runner_diagnostics` child module (private-field access) and the
//! production constructors in the `runner_prepare` sibling (line-cap split).

use crate::app::diagnostics::DiagnosticsCollector;
use crate::app::pipeline::clock::FrameClock;
use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::pacing::{PacingSchedule, PipelinePacing};
use crate::app::pipeline::{source::MessageSource, source_error::pipeline_source_failure};
use crate::chat::ChatStore;
use crate::renderer::{Frame, RenderInput, Renderer};
use crate::streaming::{CompletedOutput, FrameSink, OutputKind, RtmpsDestination};

#[path = "runner_diagnostics.rs"]
mod runner_diagnostics;

/// Assembled parts for one frame pipeline. Callers (TODO Task 3 demo mode,
/// tests) populate the struct literal directly; fields are `pub(crate)`
/// because this crate-internal aggregate's single job is field collection
/// (keeps `FramePipeline::new` within the max-2-arguments rule).
pub(crate) struct PipelineParts<S> {
    pub(crate) chat_store: ChatStore,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) sink: S,
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) clock: Box<dyn FrameClock>,
    pub(crate) pacing: PipelinePacing,
    pub(crate) stop_flag: StopFlag,
    pub(crate) diagnostics: Option<DiagnosticsCollector>,
}

/// Cell types for `prepared_pipeline` (production convenience constructor;
/// ≤2-arguments rule via parameter object).
pub(crate) struct PipelineConstruction {
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) stop_flag: StopFlag,
    pub(crate) output_kind: OutputKind,
}

/// Cell types for `prepared_stream_pipeline` (the production streaming
/// constructor; ≤2-arguments rule via parameter object).
pub(crate) struct StreamParts {
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) stop_flag: StopFlag,
    pub(crate) destination: RtmpsDestination,
    pub(crate) diagnostics: Option<DiagnosticsCollector>,
}

/// The continuous frame pipeline: accepts at most one due message per frame
/// iteration, renders the complete visible text, writes one exact RGB24
/// frame per paced iteration, and finalizes the sink on stop. Never names
/// `FfmpegProcess` (S9).
pub(crate) struct FramePipeline<S> {
    chat_store: ChatStore,
    renderer: Box<dyn Renderer>,
    sink: S,
    source: Box<dyn MessageSource>,
    clock: Box<dyn FrameClock>,
    pacing: PipelinePacing,
    stop_flag: StopFlag,
    diagnostics: Option<DiagnosticsCollector>,
}

impl<S: FrameSink> FramePipeline<S> {
    pub(crate) fn new(parts: PipelineParts<S>) -> Self {
        Self {
            chat_store: parts.chat_store,
            renderer: parts.renderer,
            sink: parts.sink,
            source: parts.source,
            clock: parts.clock,
            pacing: parts.pacing,
            stop_flag: parts.stop_flag,
            diagnostics: parts.diagnostics,
        }
    }

    /// The paced loop. STOP LATENCY: at most one frame interval.
    pub(crate) fn run(mut self) -> Result<CompletedOutput, FramePipelineError> {
        let mut schedule =
            PacingSchedule::started(self.pacing.interval_nanos(), self.clock.now_nanos());
        loop {
            if self.stop_flag.is_requested() {
                break;
            }
            self.accept_next_message()?;
            self.render_and_write_frame()?;
            let gap_nanos = schedule.next_gap(self.clock.now_nanos());
            self.note_frame_tick();
            self.maybe_emit_summary();
            self.clock.sleep_nanos(gap_nanos);
        }
        let sink = self.sink;
        sink.finish().map_err(FramePipelineError::Sink)
    }

    /// Message seam call point (S5): top of every iteration, before render;
    /// at most one message per frame so arrival never outruns the pacing.
    fn accept_next_message(&mut self) -> Result<(), FramePipelineError> {
        let due = self
            .source
            .next_message()
            .map_err(pipeline_source_failure)?;
        if let Some(message) = due {
            self.note_message_dequeued(&message);
            self.chat_store
                .accept(message)
                .map_err(FramePipelineError::Chat)?;
        }
        Ok(())
    }

    fn render_and_write_frame(&mut self) -> Result<(), FramePipelineError> {
        let input = RenderInput {
            text: self.chat_store.visible_text(),
        };
        self.note_render_started();
        let frame = self
            .renderer
            .render(&input)
            .map_err(FramePipelineError::Render)?;
        self.note_render_completed();
        self.write_frame_bytes(frame)
    }

    /// Defense-in-depth size gate (S8), then Task-1's exact-byte `write_all`
    /// semantics via the sink (S10).
    fn write_frame_bytes(&mut self, frame: Frame) -> Result<(), FramePipelineError> {
        let expected = self.pacing.frame_byte_count();
        if !frame_has_expected_size(&frame, expected) {
            return Err(FramePipelineError::FrameSizeMismatch {
                expected,
                actual: frame.pixels().len(),
            });
        }
        self.note_write_started();
        self.sink
            .write_frame(frame.pixels())
            .map_err(FramePipelineError::Sink)?;
        self.note_write_completed();
        Ok(())
    }
}

/// Single-section boolean rule: the comparison lives behind a named helper.
fn frame_has_expected_size(frame: &Frame, expected: usize) -> bool {
    frame.pixels().len() == expected
}
