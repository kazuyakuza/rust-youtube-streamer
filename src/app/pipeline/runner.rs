//! The continuous frame pipeline: drains one due message, renders the
//! complete visible chat text, writes exactly one full RGB24 frame per
//! paced iteration, and finalizes the sink on stop (TODO Task 2; plan
//! decisions S9/S10). It never names `FfmpegProcess` — the sink arrives as
//! `S: FrameSink` from the caller.

use crate::app::pipeline::clock::{FrameClock, SystemFrameClock};
use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::pacing::{PacingSchedule, PipelinePacing};
use crate::app::pipeline::{source::MessageSource, source_error::pipeline_source_failure};
use crate::chat::{ChatStore, ChatStoreSettings};
use crate::config::model::AppConfig;
use crate::renderer::{Frame, RenderInput, Renderer, RendererSettings, TextRenderer};
use crate::streaming::{
    resolve_output_path, CompletedOutput, FfmpegProcess, FrameSink, OutputKind, RtmpsDestination,
};

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
        let frame = self
            .renderer
            .render(&input)
            .map_err(FramePipelineError::Render)?;
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
        self.sink
            .write_frame(frame.pixels())
            .map_err(FramePipelineError::Sink)
    }
}

/// Single-section boolean rule: the comparison lives behind a named helper.
fn frame_has_expected_size(frame: &Frame, expected: usize) -> bool {
    frame.pixels().len() == expected
}

/// Production convenience constructor for TODO Task 3's demo command: builds
/// output path, FFmpeg process, chat store, and text renderer from validated
/// config; a `SystemFrameClock` is fixed inside. Tests do NOT use this (they
/// assemble `PipelineParts` directly).
pub(crate) fn prepared_pipeline(
    config: &AppConfig,
    construction: PipelineConstruction,
) -> Result<FramePipeline<FfmpegProcess>, FramePipelineError> {
    let pacing = PipelinePacing::from_config(&config.video)?;
    let output_path =
        resolve_output_path(construction.output_kind).map_err(FramePipelineError::Sink)?;
    let process = FfmpegProcess::start(config, output_path).map_err(FramePipelineError::Sink)?;
    let parts = PipelineParts {
        chat_store: build_chat_store(config)?,
        renderer: build_renderer(config)?,
        sink: process,
        source: construction.source,
        clock: Box::new(SystemFrameClock::new()),
        pacing,
        stop_flag: construction.stop_flag,
    };
    Ok(FramePipeline::new(parts))
}

/// Production streaming constructor for the run mode (plan D-T3-10): the
/// existing production assembly (`PipelinePacing::from_config`,
/// `build_chat_store`, `build_renderer`, `SystemFrameClock`) with the ONLY
/// difference that the sink is `FfmpegProcess::start_stream` toward the
/// RTMPS destination (no local output file). `prepared_pipeline`,
/// `PipelineConstruction` and `OutputKind` stay untouched.
pub(crate) fn prepared_stream_pipeline(
    config: &AppConfig,
    parts: StreamParts,
) -> Result<FramePipeline<FfmpegProcess>, FramePipelineError> {
    let pacing = PipelinePacing::from_config(&config.video)?;
    let process =
        FfmpegProcess::start_stream(config, parts.destination).map_err(FramePipelineError::Sink)?;
    let parts = PipelineParts {
        chat_store: build_chat_store(config)?,
        renderer: build_renderer(config)?,
        sink: process,
        source: parts.source,
        clock: Box::new(SystemFrameClock::new()),
        pacing,
        stop_flag: parts.stop_flag,
    };
    Ok(FramePipeline::new(parts))
}

fn build_chat_store(config: &AppConfig) -> Result<ChatStore, FramePipelineError> {
    let settings = ChatStoreSettings::from_config(config).map_err(FramePipelineError::Chat)?;
    ChatStore::new(settings).map_err(FramePipelineError::Chat)
}

fn build_renderer(config: &AppConfig) -> Result<Box<dyn Renderer>, FramePipelineError> {
    let settings = RendererSettings::from_config(&config.renderer, &config.video)
        .map_err(FramePipelineError::Render)?;
    let renderer = TextRenderer::new(settings).map_err(FramePipelineError::Render)?;
    Ok(Box::new(renderer))
}

#[cfg(test)]
#[path = "runner_retrieval_tests.rs"]
mod runner_retrieval_tests;
