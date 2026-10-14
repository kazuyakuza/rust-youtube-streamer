//! The shared observation mappings of the readiness waits (Amendment 2
//! split of F-T2-3): exactly one mapping from a `liveStreams` lookup
//! answer and one from a `liveBroadcasts` answer to the typed status /
//! lifecycle (D-T2-9), plus the binding within-wait attempt matrix —
//! `Network` and 5xx `BackendUnavailable` are transient (one attempt
//! consumed), every other classified API error is immediate and fatal;
//! `Ok(None)` lookups are `ApiNotFound` and missing/unknown labels are
//! `MalformedResponse`.

use crate::youtube::live_error::LiveApiError;
use crate::youtube::live_lifecycle::{BroadcastLifecycle, StreamStatusKind};
use crate::youtube::live_models::{LiveBroadcast, LiveStream};
use crate::youtube::wait_cancellation::WaitCanceller;

/// The O4-only parameter object of the confirmation wait (Task 3 fills
/// `cancellation` from its Ctrl+C-linked clone).
pub(crate) struct StreamObservationRequest<'a> {
    pub(crate) stream_id: &'a str,
    pub(crate) cancellation: &'a WaitCanceller,
}

/// The one stream-status mapping: `None` → not-found, missing/unknown
/// label → malformed, otherwise the typed status kind.
pub(super) fn stream_observation(
    stream: Option<LiveStream>,
) -> Result<(LiveStream, StreamStatusKind), LiveApiError> {
    let stream = stream.ok_or_else(|| not_found("liveStream"))?;
    let label = stream
        .status
        .as_ref()
        .and_then(|status| status.stream_status.as_deref())
        .ok_or_else(|| malformed("status.streamStatus"))?;
    let status = StreamStatusKind::from_api_label(label)?;
    Ok((stream, status))
}

pub(super) fn broadcast_observation(
    broadcast: Option<LiveBroadcast>,
) -> Result<BroadcastLifecycle, LiveApiError> {
    let broadcast = broadcast.ok_or_else(|| not_found("liveBroadcast"))?;
    let label = broadcast
        .status
        .as_ref()
        .and_then(|status| status.life_cycle_status.as_deref())
        .ok_or_else(|| malformed("status.lifeCycleStatus"))?;
    BroadcastLifecycle::from_api_label(label)
}

fn is_transient(error: &LiveApiError) -> bool {
    matches!(
        error,
        LiveApiError::Network { .. } | LiveApiError::BackendUnavailable { .. }
    )
}

/// Attempt-matrix gate (D-T2-9): a transient failure consumes one attempt
/// (`Ok(None)`); any other error is immediate and fatal.
pub(super) fn tolerate_transient<T>(
    outcome: Result<T, LiveApiError>,
) -> Result<Option<T>, LiveApiError> {
    match outcome {
        Ok(observed) => Ok(Some(observed)),
        Err(error) if is_transient(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Sleep between poll attempts — shared by both attempt-bounded waits
/// (plan F-T2-3 `sleep_between_polls`).
pub(super) fn sleep_between_polls(seconds: u64) {
    std::thread::sleep(std::time::Duration::from_secs(seconds));
}

pub(super) fn attempt_exhausted(stage: &str) -> LiveApiError {
    LiveApiError::ReadinessTimeout {
        stage: stage.to_string(),
    }
}

pub(super) fn broadcast_state(detail: &str) -> LiveApiError {
    LiveApiError::BroadcastState {
        detail: detail.to_string(),
    }
}

fn not_found(resource: &str) -> LiveApiError {
    LiveApiError::ApiNotFound {
        resource: resource.to_string(),
    }
}

fn malformed(field: &str) -> LiveApiError {
    LiveApiError::MalformedResponse {
        field: field.to_string(),
    }
}
