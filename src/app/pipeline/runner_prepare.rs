//! Production frame-pipeline constructors (split from `runner` per the
//! 200-line cap precedent): the demo convenience constructor and the run
//! streaming constructor share the same assembly helpers, and only differ in
//! the sink they start.

use crate::app::diagnostics::DiagnosticsCollector;
use crate::app::pipeline::clock::SystemFrameClock;
use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::pacing::PipelinePacing;
use crate::chat::{ChatStore, ChatStoreSettings};
use crate::config::model::AppConfig;
use crate::renderer::{Renderer, RendererSettings, TextRenderer};
use crate::streaming::{resolve_output_path, FfmpegProcess};

use super::runner::{FramePipeline, PipelineConstruction, PipelineParts, StreamParts};

/// Production convenience constructor for TODO Task 3's demo command: builds
/// output path, FFmpeg process, chat store, and text renderer from validated
/// config; a `SystemFrameClock` is fixed inside. Tests do NOT use this (they
/// assemble `PipelineParts` directly). Demo diagnostics are built from the
/// same config and stay disabled by default.
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
        diagnostics: DiagnosticsCollector::from_config(config),
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
        diagnostics: parts.diagnostics,
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
