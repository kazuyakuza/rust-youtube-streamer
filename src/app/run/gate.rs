//! The post-start confirmation gate (plan D-T3-2 stage 8 refinement,
//! D-T3-9): the O4-only `wait_until_stream_active` (24 × 5 s,
//! cancellable) starts on a dedicated std thread RIGHT AFTER FFmpeg
//! starts, and the frame pipeline begins immediately after — the literal
//! sequential reading of D-T1-8 is rejected because the existing pipeline
//! contract requires video to flow before YouTube flips `streamStatus` to
//! `active` (FFmpeg opens the RTMPS output only once stdin frames arrive).
//! "Startup success" is MARKED by the confirmation `info!` event, never
//! by a blocking barrier. The thread owns its collaborators (the
//! `'static` spawn requirement): an `Arc` transport clone and cloned
//! stop flags. Joining costs at most one confirmation poll interval plus
//! any in-flight request (D-T2-16(a) carried: NO request-level timeout).
//!
//! Worker duty (record D-T3-A): on `GateFinish::Confirmed` the thread
//! spawns the chat worker from `GatePlan::chat_worker` and records its
//! `JoinHandle`; `finish()` joins the gate thread FIRST, then the worker
//! (§2.3 refinement). `None` spawns nothing.
//!
//! On success the thread also emits the watch-link info event (D-T2W-1).
//!
//! D-T3-12 (recorded, voided behavior): post-start the runtime polls O4
//! ONLY; the broadcast's `live` transition (via autoStart) is never
//! observed, so no ready→live lifecycle log exists — no later agent may
//! assume an O5 poll was forgotten. Resource-fate/monitoring guidance
//! belongs to the Task-4 docs and YouTube Studio.

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use tracing::info;

use crate::app::pipeline::StopFlag;
use crate::youtube::{
    wait_until_stream_active, LiveApiError, LiveApiTransport, StreamObservationRequest,
    WaitCanceller,
};

use super::chat_worker::{spawn_chat_worker, ChatWorkerPlan};

#[path = "gate_diagnostics.rs"]
mod gate_diagnostics;
#[cfg(test)]
pub(super) use gate_diagnostics::{
    observed_status_label, READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE,
    READINESS_TIMEOUT_STREAM_STATUS_MESSAGE,
};

/// The watch-link event's printable line message (D-T2W-1): single
/// source of truth, pinned verbatim by a test.
pub(super) const WATCH_LINK_MESSAGE: &str = "public watch link for the live stream";

/// The gate thread's owned collaborators (the `'static` spawn requirement
/// makes every field owned or `Arc`-shared).
pub(super) struct GatePlan {
    pub(super) transport: Arc<dyn LiveApiTransport + Send + Sync>,
    pub(super) cancellation: WaitCanceller,
    pub(super) stream_id: String,
    pub(super) watch_url: String,
    pub(super) stop_flag: StopFlag,
    pub(super) chat_worker: Option<ChatWorkerPlan>,
}

/// The recorded outcome of the confirmation gate (the primary-error
/// selector's second input).
pub(super) enum GateFinish {
    Confirmed,
    Cancelled,
    Fatal(LiveApiError),
    Panicked,
}

/// Shared state between the gate thread and the joiner; written exactly
/// once by the thread, read once after the join.
struct GateState {
    outcome: Option<GateFinish>,
    chat_worker: Option<JoinHandle<()>>,
}

/// A spawned confirmation gate awaiting its join.
pub(super) struct SpawnedGate {
    handle: JoinHandle<()>,
    state: Arc<Mutex<GateState>>,
}

pub(super) fn spawn_startup_gate(plan: GatePlan) -> SpawnedGate {
    let state = Arc::new(Mutex::new(GateState {
        outcome: None,
        chat_worker: None,
    }));
    let thread_state = Arc::clone(&state);
    let handle = thread::spawn(move || run_confirmation(plan, &thread_state));
    SpawnedGate { handle, state }
}

impl SpawnedGate {
    /// Joins the confirmation thread, then the recorded chat worker, then
    /// reads the recorded outcome. A join failure or a thread that
    /// recorded nothing (a panic before the write) both map to
    /// `Panicked` — never a panic here. The worker join runs even when
    /// the gate thread itself panicked.
    pub(super) fn finish(self) -> GateFinish {
        let gate_panicked = self.handle.join().is_err();
        join_recorded_chat_worker(&self.state);
        if gate_panicked {
            return GateFinish::Panicked;
        }
        match self.state.lock() {
            Ok(mut state) => state.outcome.take().unwrap_or(GateFinish::Panicked),
            Err(_) => GateFinish::Panicked,
        }
    }
}

/// Test-only confirmation budget: ONE attempt at a ZERO-second interval
/// exhausts without any sleep, allowing a zero-sleep ReadinessTimeout
/// test (the gate_tests zero-sleep rule). Production never sees this.
#[cfg(test)]
const TEST_CONFIRMATION_ATTEMPTS: u32 = 1;
#[cfg(test)]
const TEST_CONFIRMATION_INTERVAL_SECONDS: u64 = 0;

#[cfg(test)]
fn confirmation_budget_wait(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
) -> Result<(), LiveApiError> {
    crate::youtube::wait_until_stream_active_with_polling(
        transport,
        stream_pair,
        TEST_CONFIRMATION_ATTEMPTS,
        TEST_CONFIRMATION_INTERVAL_SECONDS,
    )
}

#[cfg(not(test))]
fn confirmation_budget_wait(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
) -> Result<(), LiveApiError> {
    wait_until_stream_active(transport, stream_pair)
}

/// The confirmation wait on its dedicated thread (plan A5): Ok →
/// `Confirmed` + the startup-success event; `WaitCancelled` →
/// `Cancelled`; any other typed failure records `Fatal` AND requests the
/// frame-loop stop so the pipeline ends and the shutdown can report the
/// primary cause. Mutex access uses `if let Ok` — this thread never
/// panics on a poisoned lock (poisoning here means an earlier panic,
/// already a defect).
fn run_confirmation(plan: GatePlan, state: &Mutex<GateState>) {
    let observation = StreamObservationRequest {
        stream_id: &plan.stream_id,
        cancellation: &plan.cancellation,
    };
    let result = confirmation_budget_wait(plan.transport.as_ref(), &observation);
    let outcome = match result {
        Ok(()) => {
            info!("stream confirmed active on YouTube");
            info!(watch_url = %plan.watch_url, "{}", WATCH_LINK_MESSAGE);
            let chat_worker = plan.chat_worker.map(spawn_chat_worker);
            record_chat_worker(state, chat_worker);
            GateFinish::Confirmed
        }
        Err(LiveApiError::WaitCancelled) => GateFinish::Cancelled,
        Err(error) => {
            log_last_stream_status_on_timeout(&plan, &error);
            plan.stop_flag.request_stop();
            GateFinish::Fatal(error)
        }
    };
    if let Ok(mut recorded) = state.lock() {
        recorded.outcome = Some(outcome);
    }
}

fn record_chat_worker(state: &Mutex<GateState>, chat_worker: Option<JoinHandle<()>>) {
    if let Ok(mut recorded) = state.lock() {
        recorded.chat_worker = chat_worker;
    }
}

fn join_recorded_chat_worker(state: &Arc<Mutex<GateState>>) {
    let worker = state
        .lock()
        .ok()
        .and_then(|mut recorded| recorded.chat_worker.take());
    if let Some(worker) = worker {
        let _ = worker.join();
    }
}

fn log_last_stream_status_on_timeout(plan: &GatePlan, error: &LiveApiError) {
    if gate_diagnostics::is_confirmation_timeout(error) {
        gate_diagnostics::log_last_stream_status(plan.transport.as_ref(), &plan.stream_id);
    }
}

#[cfg(test)]
#[path = "gate_tests.rs"]
mod gate_tests;
