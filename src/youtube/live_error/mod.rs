//! Typed Live API failures for the Phase-05 runtime (plan F4, D-T1-4):
//! one enum covering the documented §1.5 failure table plus the bounded
//! wait timeout and locally produced invalid metadata. Messages are
//! end-user-reportable and stable; the RTMPS destination (D-T1-11) never
//! enters this enum, so no variant can leak the stream key or URL.
//! Classification is status-FIRST (Phase-04 precedent: statuses outrank
//! reason strings); only the non-secret server `reason` code is read from
//! bodies, and bodies are never embedded.

// Re-exported for Task 2's concrete transport and for offline tests.
// Retire when Task 2's concrete transport consumes the classifier.
#[allow(unused_imports)]
pub(crate) use classify::classify_api_error;

mod classify;
mod display;

/// Failures of the documented YouTube Live operations (O1–O5) and the
/// readiness waits around them.
#[derive(Debug)]
pub(crate) enum LiveApiError {
    InvalidMetadata { field: String },
    LiveStreamingNotEnabled,
    ChannelLiveBlocked,
    InsufficientPermissions,
    AuthorizationRejected,
    QuotaLike { reason: String },
    BroadcastState { detail: String },
    ApiNotFound { resource: String },
    ApiRejected { status: u16, reason: String },
    BackendUnavailable { reason: String },
    Network { reason: String },
    MalformedResponse { field: String },
    ReadinessTimeout { stage: String },
    DestinationMissing { field: String },
}

#[cfg(test)]
#[path = "../live_error_tests.rs"]
mod live_error_tests;

#[cfg(test)]
#[path = "../live_error_display_tests.rs"]
mod live_error_display_tests;
