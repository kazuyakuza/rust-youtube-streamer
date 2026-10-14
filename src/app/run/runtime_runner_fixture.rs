//! The recorded streaming-runner fake (D-T3-9; split from
//! `runtime_test_fixtures` per the D-T3-14 line cap): captures the
//! execution bundle, mirrors the production shutdown sequence per
//! scripted behavior, and reports the outcome through the REAL
//! `primary_shutdown_outcome` selector (production/test parity, TD-5's
//! documented drift risk).

use std::sync::Mutex;

use crate::app::pipeline::{FramePipelineError, StopFlag};
use crate::config::AppConfig;
use crate::streaming::StreamingError;

use super::error::RunError;
use super::gate::GateFinish;
use super::streaming::{primary_shutdown_outcome, StreamingExecution, StreamingRunner};

/// What the recorded runner captured from one execution bundle.
pub(super) struct RecordedExecution {
    pub(super) destination_debug: String,
    pub(super) stream_id: String,
    pub(super) gate_finish: &'static str,
}

pub(super) struct RecordingStreamingRunner {
    script: RunnerScript,
    recorded: Mutex<Option<RecordedExecution>>,
}

pub(super) enum RunnerScript {
    /// TD-1: the streaming stage reports a clean stop.
    ReportOk,
    /// TD-4: mirror the production gate-drive sequence — the loop ends
    /// when the gate flips the stop flag, then cancel + join.
    MirrorProductionShutdown,
    /// TD-5: simulate a frame-pipeline failure, then apply the PRODUCTION
    /// order (cancel the wait BEFORE joining the gate).
    PipelineFailureThenProductionOrder,
}

const STOP_FLAG_SPIN_CAP: u32 = 5_000;
const STOP_FLAG_SPIN_MILLIS: u64 = 1;

impl RecordingStreamingRunner {
    pub(super) fn scripted(script: RunnerScript) -> Self {
        Self {
            script,
            recorded: Mutex::new(None),
        }
    }

    pub(super) fn take_recorded(&self) -> RecordedExecution {
        self.recorded
            .lock()
            .expect("fixture mutex")
            .take()
            .expect("the recorded runner must run")
    }

    pub(super) fn was_invoked(&self) -> bool {
        self.recorded.lock().expect("fixture mutex").is_some()
    }

    fn scripted_stream_outcome(&self, stop_flag: &StopFlag) -> Result<(), FramePipelineError> {
        match self.script {
            RunnerScript::ReportOk => Ok(()),
            RunnerScript::MirrorProductionShutdown => {
                wait_for_requested(stop_flag);
                Ok(())
            }
            RunnerScript::PipelineFailureThenProductionOrder => {
                Err(FramePipelineError::Sink(StreamingError::WaitFailed {
                    source: std::io::Error::other("frame pipeline failure fixture"),
                }))
            }
        }
    }
}

impl StreamingRunner for RecordingStreamingRunner {
    fn run(&self, _app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError> {
        let destination_debug = format!("{:?}", execution.destination);
        let stream_id = execution.stream_id.clone();
        let stream_outcome = self.scripted_stream_outcome(&execution.stop_flag);
        if stream_outcome.is_err() {
            execution.canceller.request_cancel();
        }
        let gate_finish = execution.gate.finish();
        *self.recorded.lock().expect("fixture mutex") = Some(RecordedExecution {
            destination_debug,
            stream_id,
            gate_finish: gate_label(&gate_finish),
        });
        primary_shutdown_outcome(stream_outcome, gate_finish)
    }
}

fn wait_for_requested(stop_flag: &StopFlag) {
    for _ in 0..STOP_FLAG_SPIN_CAP {
        if stop_flag.is_requested() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(STOP_FLAG_SPIN_MILLIS));
    }
}

fn gate_label(gate_finish: &GateFinish) -> &'static str {
    match gate_finish {
        GateFinish::Confirmed => "confirmed",
        GateFinish::Cancelled => "cancelled",
        GateFinish::Fatal(_) => "fatal",
        GateFinish::Panicked => "panicked",
    }
}
