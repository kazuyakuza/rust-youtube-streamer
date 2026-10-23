//! The confirmation-gate timeout diagnostics (run-diagnostics Task 1):
//! the single-source-of-truth message constants for the tolerated final
//! status lookup and the pure helpers the gate's fatal arm consumes at
//! the `ReadinessTimeout`. Every outcome is logged as a fact — never
//! fatal, never a RunError variant.

use tracing::info;

use crate::youtube::{LiveApiError, LiveApiTransport, LiveStream};

/// The info event at the confirmation-gate timeout AFTER the tolerated
/// final stream lookup observed a label: the last observed streamStatus
/// is the key discriminator (never left `ready` = YouTube never
/// registered ingest). Non-secret; pinned verbatim by a unit test.
pub(in crate::app::run) const READINESS_TIMEOUT_STREAM_STATUS_MESSAGE: &str =
    "confirmation gate timed out; last observed streamStatus";

/// The info event when the tolerated final lookup itself yields no
/// label (lookup error, not-found, or a label-less resource): the FACT
/// is logged, never fatal. Pinned verbatim by a unit test.
pub(in crate::app::run) const READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE: &str =
    "confirmation gate timed out; the stream status could not be observed";

/// Single-section boolean rule: the timeout-only diagnostic decision is
/// a named predicate (the gate path is the only producer of this
/// variant, and the stage argument stays enum-internal).
pub(super) fn is_confirmation_timeout(error: &LiveApiError) -> bool {
    matches!(error, LiveApiError::ReadinessTimeout { .. })
}

/// ONE additional tolerated stream lookup at the timeout (plan-encoded):
/// any outcome is consumed as a fact — an observed label is named, any
/// other outcome logs the unavailable message; a Network/classified
/// failure is NEVER fatal here and never changes `GateFinish::Fatal`.
pub(super) fn log_last_stream_status(transport: &dyn LiveApiTransport, stream_id: &str) {
    let observed_stream = transport.lookup_stream(stream_id).ok().flatten();
    match observed_stream.as_ref().and_then(observed_status_label) {
        Some(label) => {
            info!(
                stream_status = label,
                "{}", READINESS_TIMEOUT_STREAM_STATUS_MESSAGE
            );
        }
        None => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
    }
}

pub(in crate::app::run) fn observed_status_label(stream: &LiveStream) -> Option<&str> {
    stream
        .status
        .as_ref()
        .and_then(|status| status.stream_status.as_deref())
}
