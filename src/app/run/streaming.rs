//! The streaming stage of the run runtime (plan D-T3-6, D-T3-9): the
//! runner seam, the execution bundle handed to it, the production
//! [`PreparedStreaming`] implementation, and the pure primary-error
//! selector. Shutdown order encoded here: the frame loop ends via the
//! shared `StopFlag` (at most one frame interval of latency), a pipeline
//! failure cancels the confirmation wait BEFORE the gate join, the gate
//! join costs at most one confirmation poll interval plus any in-flight
//! request (D-T2-16(a): no request-level timeout), and EXACTLY ONE primary
//! error is reported — `GateFinish::Fatal` outranks the pipeline error
//! because the gate-fatal REQUESTED the stop, making any later
//! pipeline/finalize error downstream (D-T3-6 rule 6).

use tracing::info;

use crate::app::pipeline::{prepared_stream_pipeline, FramePipelineError, StopFlag, StreamParts};
use crate::config::AppConfig;
use crate::streaming::RtmpsDestination;
use crate::youtube::WaitCanceller;

use super::error::RunError;
use super::gate::{GateFinish, SpawnedGate};
use super::source::ChatFreeSource;

/// The streaming-stage runner seam (D-T3-9): production starts FFmpeg
/// toward the destination, runs the paced chat-free loop and completes
/// the ordered shutdown; tests inject a recording fake. `&self` (the
/// plan's `self` variant cannot be called through `&dyn StreamingRunner`
/// — object safety; the compiler-arbitrated adaptation keeps every other
/// shape identical).
pub(super) trait StreamingRunner: Send {
    /// Production: starts FFmpeg toward the destination, launches the
    /// startup gate, runs the paced chat-free loop, completes the ordered
    /// shutdown.
    fn run(&self, app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError>;
}

/// Everything the streaming stage needs after the destination reveal
/// (D-T3-9). `destination` carries the redacting `Debug`; there is no url
/// accessor. `stream_id` rides along as non-secret operator diagnostics
/// for recorded-test capture (the gate owns its own copy).
pub(super) struct StreamingExecution {
    pub(super) stop_flag: StopFlag,
    pub(super) canceller: WaitCanceller,
    pub(super) destination: RtmpsDestination,
    pub(super) stream_id: String,
    pub(super) gate: SpawnedGate,
}

/// Production runner: `prepared_stream_pipeline` (the runner sibling
/// constructor) toward the destination, the gate left as provided, the
/// paced loop, then the D-T3-6 shutdown rules.
pub(super) struct PreparedStreaming;

impl StreamingRunner for PreparedStreaming {
    fn run(&self, app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError> {
        let StreamingExecution {
            stop_flag,
            canceller,
            destination,
            // The bundle copy exists for recorded-test capture; the
            // production stage owns its confirmation copy in the gate.
            stream_id: _stream_id,
            gate,
        } = execution;
        let pipeline = prepared_stream_pipeline(
            app_config,
            StreamParts {
                source: Box::new(ChatFreeSource),
                stop_flag: stop_flag.clone(),
                destination,
            },
        )
        .map_err(RunError::Pipeline)?;
        info!("FFmpeg started toward the configured streaming destination");
        let stream_outcome = pipeline.run().map(|_completed| ());
        let gate_finish = join_gate(&canceller, gate, &stream_outcome);
        primary_shutdown_outcome(stream_outcome, gate_finish)
    }
}

/// D-T3-6 rule 2: on a frame-pipeline FATAL the confirmation wait is
/// cancelled BEFORE the gate join, so a mid-interval wait ends on its next
/// attempt check instead of running to its budget.
fn join_gate(
    canceller: &WaitCanceller,
    gate: SpawnedGate,
    stream_outcome: &Result<(), FramePipelineError>,
) -> GateFinish {
    if stream_outcome.is_err() {
        canceller.request_cancel();
    }
    gate.finish()
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
        (Err(error), _) => Err(RunError::Pipeline(error)),
        (Ok(()), GateFinish::Panicked) => Err(RunError::GateThreadStopped),
        (Ok(()), GateFinish::Confirmed | GateFinish::Cancelled) => Ok(()),
    }
}

#[cfg(test)]
#[path = "streaming_stage_tests.rs"]
mod streaming_stage_tests;
