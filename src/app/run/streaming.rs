//! The streaming stage of the run runtime (plan D-T3-6, D-T3-9): the
//! runner seam, the execution bundle handed to it, the production
//! [`PreparedStreaming`] implementation, and the pure primary-error
//! selector. Shutdown order encoded here: the frame loop ends via the
//! shared `StopFlag` (at most one frame interval of latency; the chat
//! worker SHARES that flag and is flag-stopped concurrently, joined in
//! `gate.finish()` after the sink EOF — the §2.3 refinement), a pipeline
//! failure cancels the confirmation wait BEFORE the gate join, the gate
//! join costs at most one confirmation poll interval plus any in-flight
//! request (D-T2-16(a): no request-level timeout), and EXACTLY ONE primary
//! error is reported — `GateFinish::Fatal` outranks the pipeline error
//! because the gate-fatal REQUESTED the stop, making any later
//! pipeline/finalize error downstream (D-T3-6 rule 6).

use tracing::{info, warn};

use crate::app::pipeline::{prepared_stream_pipeline, FramePipelineError, StopFlag, StreamParts};
use crate::config::AppConfig;
use crate::streaming::{CompletedOutput, RtmpsDestination, StreamExit, StreamingError};
use crate::youtube::WaitCanceller;

use super::error::RunError;
use super::gate::{GateFinish, SpawnedGate};
use super::source::ChatSource;

/// The streaming-stage runner seam (D-T3-9): production starts FFmpeg
/// toward the destination, runs the paced chat-backed loop and completes
/// the ordered shutdown; tests inject a recording fake. `&self` (the
/// plan's `self` variant cannot be called through `&dyn StreamingRunner`
/// — object safety; the compiler-arbitrated adaptation keeps every other
/// shape identical).
pub(super) trait StreamingRunner: Send {
    /// Production: starts FFmpeg toward the destination, launches the
    /// startup gate, runs the paced chat-backed loop, completes the
    /// ordered shutdown.
    fn run(&self, app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError>;
}

/// Everything the streaming stage needs after the destination reveal
/// (D-T3-9). `destination` carries the redacting `Debug`; there is no url
/// accessor. `stream_id` and `watch_url` ride along as non-secret operator
/// diagnostics for recorded-test capture (the gate owns its own copy).
/// The pre-built production `source` rides the bundle (constructed in
/// stage 7.5).
pub(super) struct StreamingExecution {
    pub(super) stop_flag: StopFlag,
    pub(super) source: ChatSource,
    pub(super) canceller: WaitCanceller,
    pub(super) destination: RtmpsDestination,
    pub(super) stream_id: String,
    pub(super) watch_url: String,
    pub(super) gate: SpawnedGate,
}

/// The info event at the END of a streaming run when FFmpeg exited
/// cleanly (exit status 0): the operator log records the exit AND the
/// redacted stderr tail (brief §11.1). Pinned verbatim by a unit test.
pub(super) const FFMPEG_CLEAN_EXIT_MESSAGE: &str =
    "FFmpeg streaming process exited cleanly (exit status 0)";

/// The warn event when FFmpeg exited non-zero during the streaming run
/// (signal death included): NEVER a second `error:` line — the single
/// `error:`-line contract holds (plan-encoded choice). Pinned verbatim.
pub(super) const FFMPEG_EXIT_FAILURE_MESSAGE: &str =
    "FFmpeg streaming process exited with a failure status";

/// The note substituted when the captured stderr tail is EMPTY so the
/// event text stays truthful without an awkward empty suffix.
pub(super) const FFMPEG_NO_DIAGNOSTICS_NOTE: &str = "none captured";

/// Production runner: `prepared_stream_pipeline` (the runner sibling
/// constructor) toward the destination, the gate left as provided, the
/// paced loop, then the D-T3-6 shutdown rules.
pub(super) struct PreparedStreaming;

impl StreamingRunner for PreparedStreaming {
    fn run(&self, app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError> {
        let StreamingExecution {
            stop_flag,
            source,
            canceller,
            destination,
            // The bundle copies exist for recorded-test capture; the
            // production stage owns its confirmation copy in the gate.
            stream_id: _stream_id,
            watch_url: _watch_url,
            gate,
        } = execution;
        let pipeline = prepared_stream_pipeline(
            app_config,
            StreamParts {
                source: Box::new(source),
                stop_flag: stop_flag.clone(),
                destination,
            },
        )
        .map_err(RunError::Pipeline)?;
        info!("FFmpeg started toward the configured streaming destination");
        let stream_outcome = pipeline.run();
        log_stream_shutdown_evidence(&stream_outcome);
        // D-T3-6 rule 2: on a frame-pipeline FATAL the confirmation wait is
        // cancelled BEFORE the gate join, so a mid-interval wait ends on its
        // next attempt check instead of running to its budget. The stop-flag
        // flip (§2.3) ends the shared-flag chat worker during the sink EOF.
        if stream_outcome.is_err() {
            stop_flag.request_stop();
            canceller.request_cancel();
        }
        let gate_finish = gate.finish();
        primary_shutdown_outcome(stream_outcome.map(|_completed| ()), gate_finish)
    }
}

/// D-8/D-9 funnel: a chat-retrieval pipeline failure maps to the typed
/// chat variant; every other pipeline failure keeps the existing variant.
pub(super) fn pipeline_error(error: FramePipelineError) -> RunError {
    match error {
        FramePipelineError::ChatRetrieval(retrieval) => RunError::ChatRetrieval(retrieval),
        other => RunError::Pipeline(other),
    }
}

/// The pure primary-error selector (D-T3-6 rule 6; unit-tested matrix):
/// `GateFinish::Fatal` always outranks a pipeline error; a Panicked gate
/// is only reported when the pipeline itself completed cleanly.
pub(super) fn primary_shutdown_outcome(
    stream_outcome: Result<(), FramePipelineError>,
    gate_finish: GateFinish,
) -> Result<(), RunError> {
    match (stream_outcome, gate_finish) {
        (_, GateFinish::Fatal(error)) => Err(RunError::Broadcast(error)),
        (Err(error), _) => Err(pipeline_error(error)),
        (Ok(()), GateFinish::Panicked) => Err(RunError::GateThreadStopped),
        (Ok(()), GateFinish::Confirmed | GateFinish::Cancelled) => Ok(()),
    }
}

/// The end-of-run FFmpeg evidence (incident 2026-10-22): a clean exit
/// and its stderr tail are surfaced on the gate-timeout shutdown path
/// where the gate error outranks the (healthy) pipeline outcome, and a
/// non-zero exit gets the encoded warn line. `Ok(Recording)` is
/// unreachable on this production path (demo owns it) and stays silent.
fn log_stream_shutdown_evidence(stream_outcome: &Result<CompletedOutput, FramePipelineError>) {
    match stream_outcome {
        Ok(CompletedOutput::Stream(stream_exit)) => log_clean_stream_exit(stream_exit),
        Err(FramePipelineError::Sink(StreamingError::ExitFailure {
            exit_code,
            diagnostics,
        })) => log_failed_stream_exit(exit_code, diagnostics),
        _ => {}
    }
}

fn log_clean_stream_exit(stream_exit: &StreamExit) {
    info!(
        exit_code = ?stream_exit.exit_code(),
        "{}; captured diagnostics follow: {}",
        FFMPEG_CLEAN_EXIT_MESSAGE,
        diagnostics_text(stream_exit.stderr_tail())
    );
}

fn log_failed_stream_exit(exit_code: &Option<i32>, stderr_tail: &str) {
    warn!(
        exit_code = ?exit_code,
        "{}; captured diagnostics follow: {}",
        FFMPEG_EXIT_FAILURE_MESSAGE,
        diagnostics_text(stderr_tail)
    );
}

/// The test-pinned classification predicate for the stream-exit
/// classification contract; the production warn arm decides via the
/// `ExitFailure` match.
#[cfg(test)]
pub(super) fn stream_exit_failed(exit_code: Option<i32>) -> bool {
    !matches!(exit_code, Some(0))
}

pub(super) fn diagnostics_text(stderr_tail: &str) -> &str {
    if stderr_tail.is_empty() {
        FFMPEG_NO_DIAGNOSTICS_NOTE
    } else {
        stderr_tail
    }
}

#[cfg(test)]
#[path = "streaming_stage_tests.rs"]
mod streaming_stage_tests;

#[cfg(test)]
#[path = "streaming_diagnostics_tests.rs"]
mod streaming_diagnostics_tests;
