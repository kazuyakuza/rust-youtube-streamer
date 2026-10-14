//! The O4-only post-start confirmation wait (D-T1-8 step 8; Task-3-
//! consumed): polls `liveStreams.list` until `status.streamStatus` is
//! observed `active`, within the D-T2-9 post-start budget (120 s / 5 s →
//! 24 attempts). Transient tolerance and the terminal mappings are the
//! shared `observation` matrix; the readiness budget lives in `super`.

use super::observation::{
    attempt_exhausted, broadcast_state, sleep_between_polls, stream_observation,
    tolerate_transient,
};
use super::*;

const CONFIRMATION_BUDGET_SECONDS: u64 = 120;
const MAX_CONFIRMATION_ATTEMPTS: u32 =
    (CONFIRMATION_BUDGET_SECONDS / super::POLL_INTERVAL_SECONDS) as u32;
const CONFIRMATION_STAGE: &str = "ingestion-confirmation";

/// O4-only confirmation wait (D-T1-8 step 8; Task-3-consumed): polls
/// until the stream status is observed `active`.
#[allow(dead_code)]
pub(crate) fn wait_until_stream_active(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
) -> Result<(), LiveApiError> {
    confirmation_poll(
        transport,
        stream_pair,
        MAX_CONFIRMATION_ATTEMPTS,
        super::POLL_INTERVAL_SECONDS,
    )
}

/// Test-only attempt/interval re-injection for the confirmation wait.
#[cfg(test)]
pub(crate) fn wait_until_stream_active_with_polling(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
    max_attempts: u32,
    poll_interval_seconds: u64,
) -> Result<(), LiveApiError> {
    confirmation_poll(transport, stream_pair, max_attempts, poll_interval_seconds)
}

fn confirmation_poll(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
    max_attempts: u32,
    poll_interval_seconds: u64,
) -> Result<(), LiveApiError> {
    for _attempt in 0..max_attempts {
        if stream_pair.cancellation.is_cancelled() {
            return Err(LiveApiError::WaitCancelled);
        }
        let observed = transport
            .lookup_stream(stream_pair.stream_id)
            .and_then(stream_observation)
            .map(|(_stream, status)| status);
        match tolerate_transient(observed)? {
            Some(StreamStatusKind::Active) => return Ok(()),
            Some(StreamStatusKind::Error) => return Err(broadcast_state(STREAM_STATUS_ERROR_DETAIL)),
            Some(_) | None => {}
        }
        sleep_between_polls(poll_interval_seconds);
    }
    Err(attempt_exhausted(CONFIRMATION_STAGE))
}
