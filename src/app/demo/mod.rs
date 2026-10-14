//! The `demo` mode handler (TODO Task 3): a local-only demonstration that
//! never contacts YouTube, never attempts OAuth, never creates a broadcast,
//! and never connects to a chat API. It installs the app's single Ctrl+C
//! interrupt handler, builds the demo message source, and runs the shared
//! chat → renderer → FFmpeg pipeline against a `demo-`-prefixed MP4 beside
//! the executable. Exit contract: success 0; ANY demo-origin failure —
//! interrupt-install failure or any FramePipelineError — is reported by
//! exactly one stderr `error:` line and exits 1 (the startup-failure class;
//! see app::error). `demo` never exits 3.

mod schedule;
mod source;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;

use std::fmt;

use crate::app::demo::schedule::DemoTimeline;
use crate::app::demo::source::DemoMessageSource;
use crate::app::error::STARTUP_FAILURE_EXIT;
use crate::app::pipeline::{
    install_interrupt_handler, prepared_pipeline, FramePipelineError, InterruptHandlerError,
    PipelineConstruction, StopFlag,
};
use crate::config::AppConfig;
use crate::streaming::{CompletedOutput, OutputKind};

/// Exit code returned by the demo mode on success.
const DEMO_SUCCESS_EXIT: u8 = 0;

/// All fatal demo-origin failures; every variant exits code 1.
#[derive(Debug)]
pub(super) enum DemoError {
    /// The Ctrl+C interrupt handler could not be installed.
    Interrupt(InterruptHandlerError),
    /// Building or running the frame pipeline failed (chat message stage,
    /// rendering, setup guards, or the FFmpeg sink lifecycle).
    Pipeline(FramePipelineError),
}

impl DemoError {
    /// Scheduled exit-code mapping: both variants are fatal failures after
    /// process start → the startup-failure class (code 1); `demo` never
    /// returns a placeholder code.
    fn exit_code(&self) -> u8 {
        STARTUP_FAILURE_EXIT
    }
}

/// Verbatim delegation keeps the single stderr line naming the cause: the
/// inner Display strings are already actionable ("unable to install the
/// Ctrl+C interrupt handler: …", "chat message stage failed: …", "frame
/// rendering failed: …", "frame output failed: …").
impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Interrupt(error) => write!(formatter, "{error}"),
            Self::Pipeline(error) => write!(formatter, "{error}"),
        }
    }
}

/// Runs the demo to completion: 0 on a verified finalized recording, 1 with
/// one stderr `error:` line on any failure.
pub(super) fn run_demo(app_config: &AppConfig) -> u8 {
    match demo_outcome(app_config) {
        Ok(()) => DEMO_SUCCESS_EXIT,
        Err(error) => report_demo_failure(&error),
    }
}

/// Ordered demo lifecycle: interrupt handler FIRST (a Ctrl+C during the
/// remaining setup is already observed), then the shared pipeline against
/// the `OutputKind::Demo` output, then the success-only print.
fn demo_outcome(app_config: &AppConfig) -> Result<(), DemoError> {
    let stop_flag = install_interrupt_handler().map_err(DemoError::Interrupt)?;
    let construction = PipelineConstruction {
        source: Box::new(build_demo_source(stop_flag.clone())),
        stop_flag,
        output_kind: OutputKind::Demo,
    };
    let pipeline = prepared_pipeline(app_config, construction).map_err(DemoError::Pipeline)?;
    let completed = pipeline.run().map_err(DemoError::Pipeline)?;
    print_final_path(&completed);
    Ok(())
}

/// User-facing completion output, printed ONLY after
/// `FramePipeline::run` returned a verified local recording (FFmpeg exited
/// 0 and the MP4 was confirmed present and non-empty on disk); the demo
/// never completes as a stream, so the `Recording` guard always holds.
/// Standard println!, deliberately NOT a tracing event: the tracing
/// whitelist (command, config_path, exit_code) stays untouched and no
/// configuration or path value ever enters a log field.
fn print_final_path(completed: &CompletedOutput) {
    if let CompletedOutput::Recording(recording) = completed {
        println!("demo recording finalized: {}", recording.path().display());
    }
}

fn report_demo_failure(error: &DemoError) -> u8 {
    eprintln!("error: {error}");
    error.exit_code()
}

fn build_demo_source(stop_flag: StopFlag) -> DemoMessageSource {
    DemoMessageSource::new(DemoTimeline::fixed_timeline(), stop_flag)
}
