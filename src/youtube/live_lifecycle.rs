//! Broadcast lifecycle and stream status state machines for the Phase-05
//! runtime (plan F2, D-T1-7/D-T1-13): the documented API labels of
//! `status.lifeCycleStatus` and `status.streamStatus` mapped to typed
//! enums. Unknown labels are `MalformedResponse` — a typed readiness
//! failure, never a wait-spin. No chat variants exist (Live Chat is
//! Phase 06); the enums carry nothing sensitive, so a derived `Debug`
//! suffices and readiness-poll code consumes booleans and labels only.

use super::live_error::LiveApiError;

/// The documented `status.lifeCycleStatus` values of a live broadcast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BroadcastLifecycle {
    Created,
    Ready,
    LiveStarting,
    Live,
    Complete,
    Revoked,
    TestStarting,
    Testing,
}

impl BroadcastLifecycle {
    /// Parses one documented label exactly (case-sensitive, per the API);
    /// an unknown label is a typed contract violation.
    pub(crate) fn from_api_label(label: &str) -> Result<Self, LiveApiError> {
        match label {
            "created" => Ok(Self::Created),
            "ready" => Ok(Self::Ready),
            "liveStarting" => Ok(Self::LiveStarting),
            "live" => Ok(Self::Live),
            "complete" => Ok(Self::Complete),
            "revoked" => Ok(Self::Revoked),
            "testStarting" => Ok(Self::TestStarting),
            "testing" => Ok(Self::Testing),
            _ => Err(unknown_label_error(LIFECYCLE_STATUS_FIELD)),
        }
    }

    /// PENDING observations while YouTube completes a transition.
    pub(crate) fn is_transition_pending(self) -> bool {
        matches!(self, Self::LiveStarting | Self::TestStarting)
    }

    /// The Phase-05 readiness-gate stage: settings complete, pre-data.
    pub(crate) fn is_streamable_setup(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// The documented `status.streamStatus` values of a live stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamStatusKind {
    Active,
    Created,
    Error,
    Inactive,
    Ready,
}

impl StreamStatusKind {
    /// Parses one documented label exactly; unknown → `MalformedResponse`.
    pub(crate) fn from_api_label(label: &str) -> Result<Self, LiveApiError> {
        match label {
            "active" => Ok(Self::Active),
            "created" => Ok(Self::Created),
            "error" => Ok(Self::Error),
            "inactive" => Ok(Self::Inactive),
            "ready" => Ok(Self::Ready),
            _ => Err(unknown_label_error(STREAM_STATUS_FIELD)),
        }
    }
}

const LIFECYCLE_STATUS_FIELD: &str = "status.lifeCycleStatus";
const STREAM_STATUS_FIELD: &str = "status.streamStatus";

fn unknown_label_error(field: &'static str) -> LiveApiError {
    LiveApiError::MalformedResponse {
        field: field.to_string(),
    }
}

#[cfg(test)]
#[path = "live_lifecycle_tests.rs"]
mod live_lifecycle_tests;
