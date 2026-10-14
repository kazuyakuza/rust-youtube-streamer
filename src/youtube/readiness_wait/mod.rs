//! Bounded, cancellable readiness polling (plan F-T2-3, D-T2-9; layout
//! per Amendment 2): the attempt-count-bounded poll loops gating `prepare`
//! on stream+broadcast readiness ([`ReadinessWait`], this file) and (Task
//! 3) on post-start ingestion confirmation (`confirmation_wait`). The
//! shared observation mappings and the transient/fatal attempt matrix
//! live in `observation`. Cancellation is checked at the top of every
//! attempt (`WaitCancelled`, D-T2-11); the constants are the D-T2-9
//! contract (180 s / 5 s → 36 readiness attempts; 120 s → 24 confirmation
//! attempts). No transition/delete calls exist anywhere.
//!
//! Plan: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (+ Amendments 1–2); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2.

mod confirmation_wait;
mod observation;

#[cfg(test)]
pub(super) use confirmation_wait::wait_until_stream_active_with_polling;
pub(super) use observation::StreamObservationRequest;

use super::api_transport::LiveApiTransport;
use super::live_error::LiveApiError;
use super::live_lifecycle::{BroadcastLifecycle, StreamStatusKind};
use super::live_models::LiveStream;
use super::wait_cancellation::WaitCanceller;
use observation::{
    attempt_exhausted, broadcast_observation, broadcast_state, is_transient, stream_observation,
};

const READINESS_BUDGET_SECONDS: u64 = 180;
const POLL_INTERVAL_SECONDS: u64 = 5;
const MAX_READINESS_ATTEMPTS: u32 = (READINESS_BUDGET_SECONDS / POLL_INTERVAL_SECONDS) as u32;
const READINESS_STAGE: &str = "readiness";
const STREAM_STATUS_ERROR_DETAIL: &str = "streamStatus error";
const BROADCAST_REVOKED_DETAIL: &str = "lifeCycleStatus revoked";

/// The readiness wait of `prepare` (D-T2-8 step 4): bounded O4+O5 polling
/// until each resource has been observed ready at least once.
pub(crate) struct ReadinessWait<'a> {
    transport: &'a dyn LiveApiTransport,
    stream_id: &'a str,
    broadcast_id: &'a str,
    cancellation: &'a WaitCanceller,
    max_attempts: u32,
    poll_interval_seconds: u64,
}

impl<'a> ReadinessWait<'a> {
    pub(crate) fn new(
        transport: &'a dyn LiveApiTransport,
        stream_id: &'a str,
        broadcast_id: &'a str,
        cancellation: &'a WaitCanceller,
    ) -> Self {
        Self {
            transport,
            stream_id,
            broadcast_id,
            cancellation,
            max_attempts: MAX_READINESS_ATTEMPTS,
            poll_interval_seconds: POLL_INTERVAL_SECONDS,
        }
    }

    /// Test-only attempt/interval re-injection; production never calls it.
    #[cfg(test)]
    pub(crate) fn with_polling(
        transport: &'a dyn LiveApiTransport,
        stream_id: &'a str,
        broadcast_id: &'a str,
        cancellation: &'a WaitCanceller,
        max_attempts: u32,
        poll_interval_seconds: u64,
    ) -> Self {
        Self {
            transport,
            stream_id,
            broadcast_id,
            cancellation,
            max_attempts,
            poll_interval_seconds,
        }
    }

    pub(crate) fn run(&self) -> Result<LiveStream, LiveApiError> {
        let mut readiness = ReadinessFlags::default();
        for _attempt in 0..self.max_attempts {
            if self.cancellation.is_cancelled() {
                return Err(LiveApiError::WaitCancelled);
            }
            if let Some(confirmed_stream) = self.poll_cycle(&mut readiness)? {
                return Ok(confirmed_stream);
            }
            std::thread::sleep(std::time::Duration::from_secs(self.poll_interval_seconds));
        }
        Err(attempt_exhausted(READINESS_STAGE))
    }

    /// One cycle: `Ok(Some(stream))` when both are observed ready (the
    /// CURRENT O4 observation); `Ok(None)` when the attempt is consumed.
    fn poll_cycle(
        &self,
        readiness: &mut ReadinessFlags,
    ) -> Result<Option<LiveStream>, LiveApiError> {
        let observed = self
            .transport
            .lookup_stream(self.stream_id)
            .and_then(stream_observation);
        let (stream, stream_status) = match observed {
            Ok(observation) => observation,
            Err(error) if is_transient(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        match stream_status {
            StreamStatusKind::Error => return Err(broadcast_state(STREAM_STATUS_ERROR_DETAIL)),
            StreamStatusKind::Ready => readiness.stream_ready = true,
            _ => {}
        }
        let lifecycle = self
            .transport
            .lookup_broadcast(self.broadcast_id)
            .and_then(broadcast_observation);
        let broadcast_lifecycle = match lifecycle {
            Ok(observed_lifecycle) => observed_lifecycle,
            Err(error) if is_transient(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        if broadcast_lifecycle == BroadcastLifecycle::Revoked {
            return Err(broadcast_state(BROADCAST_REVOKED_DETAIL));
        }
        if broadcast_lifecycle.is_streamable_setup() {
            readiness.broadcast_ready = true;
        }
        if readiness.is_both_ready() {
            return Ok(Some(stream));
        }
        Ok(None)
    }
}

#[derive(Default)]
struct ReadinessFlags {
    stream_ready: bool,
    broadcast_ready: bool,
}

impl ReadinessFlags {
    fn is_both_ready(&self) -> bool {
        self.stream_ready && self.broadcast_ready
    }
}

#[cfg(test)]
#[path = "../readiness_wait_tests.rs"]
mod readiness_wait_tests;

#[cfg(test)]
#[path = "../readiness_wait_confirmation_tests.rs"]
mod readiness_wait_confirmation_tests;
