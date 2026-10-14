//! Typed Live API failures for the Phase-05 runtime (plan F4, D-T1-4):
//! one enum covering the documented §1.5 failure table plus the bounded
//! wait timeout and locally produced invalid metadata. Messages are
//! end-user-reportable and stable; the RTMPS destination (D-T1-11) never
//! enters this enum, so no variant can leak the stream key or URL.
//! Classification is status-FIRST (Phase-04 precedent: statuses outrank
//! reason strings); the non-secret server `reason` code is the only body
//! content ever referenced, and bodies are never embedded.

use std::error::Error;
use std::fmt;

/// Canonical reason for a rejection whose body carried no reason code.
const UNSPECIFIED_REASON: &str = "unspecified";
/// API resource-name spellings for the documented not-found reasons.
const BROADCAST_RESOURCE: &str = "liveBroadcast";
const STREAM_RESOURCE: &str = "liveStream";

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

/// Pure canonical §1.5 mapping; exactly two arguments (status, reason).
/// Bare `required` (a generic Google missing-field reason) cannot name
/// its field, so the server's own reason text becomes the field value.
pub(crate) fn classify_api_error(status: u16, reason: Option<&str>) -> LiveApiError {
    let reason_text = reason.unwrap_or(UNSPECIFIED_REASON);
    match status {
        401 => LiveApiError::AuthorizationRejected,
        403 => classify_forbidden(status, reason_text),
        404 => classify_not_found(status, reason_text),
        400 => classify_bad_request(status, reason_text),
        500..=599 => LiveApiError::BackendUnavailable {
            reason: reason_text.to_string(),
        },
        _ => LiveApiError::ApiRejected {
            status,
            reason: reason_text.to_string(),
        },
    }
}

fn classify_forbidden(status: u16, reason: &str) -> LiveApiError {
    match reason {
        "liveStreamingNotEnabled" => LiveApiError::LiveStreamingNotEnabled,
        "livePermissionBlocked" => LiveApiError::ChannelLiveBlocked,
        "insufficientLivePermissions" => LiveApiError::InsufficientPermissions,
        "liveBroadcastBindingNotAllowed"
        | "errorStreamInactive"
        | "invalidTransition"
        | "redundantTransition" => LiveApiError::BroadcastState {
            detail: reason.to_string(),
        },
        "concurrentBroadcastsExceedLimit"
        | "sharedIngestionBroadcastsExceedLimit"
        | "userRequestsExceedRateLimit"
        | "rateLimitExceeded" => LiveApiError::QuotaLike {
            reason: reason.to_string(),
        },
        _ => LiveApiError::ApiRejected {
            status,
            reason: reason.to_string(),
        },
    }
}

fn classify_not_found(status: u16, reason: &str) -> LiveApiError {
    match reason {
        "liveBroadcastNotFound" => LiveApiError::ApiNotFound {
            resource: BROADCAST_RESOURCE.to_string(),
        },
        "liveStreamNotFound" => LiveApiError::ApiNotFound {
            resource: STREAM_RESOURCE.to_string(),
        },
        _ => LiveApiError::ApiRejected {
            status,
            reason: reason.to_string(),
        },
    }
}

fn classify_bad_request(status: u16, reason: &str) -> LiveApiError {
    if let Some(field) = invalid_metadata_field(reason) {
        return LiveApiError::InvalidMetadata {
            field: field.to_string(),
        };
    }
    match reason {
        "limitExceeded" | "userBroadcastsExceedLimit" => LiveApiError::QuotaLike {
            reason: reason.to_string(),
        },
        _ => LiveApiError::ApiRejected {
            status,
            reason: reason.to_string(),
        },
    }
}

/// Documented 400 invalid-value/required reasons → API resource-path
/// spellings (plan F4 mapping column; locally produced invalid metadata
/// uses config paths instead, e.g. "youtube.broadcast.title").
fn invalid_metadata_field(reason: &str) -> Option<&'static str> {
    let field = match reason {
        "invalidTitle" | "titleRequired" => "snippet.title",
        "invalidDescription" => "snippet.description",
        "invalidPrivacyStatus" | "privacyStatusRequired" => "status.privacyStatus",
        "invalidScheduledStartTime" | "scheduledStartTimeRequired" => "snippet.scheduledStartTime",
        "invalidScheduledEndTime" => "snippet.scheduledEndTime",
        "invalidAutoStart" => "contentDetails.enableAutoStart",
        "invalidAutoStop" => "contentDetails.enableAutoStop",
        "invalidFrameRate" | "frameRateRequired" => "cdn.frameRate",
        "invalidResolution" | "resolutionRequired" => "cdn.resolution",
        "invalidIngestionType" | "ingestionTypeRequired" => "cdn.ingestionType",
        "invalidEmbedSetting" => "status.enableEmbeddedPlayer",
        "cdnRequired" => "cdn",
        "required" => "required",
        _ => return None,
    };
    Some(field)
}

impl fmt::Display for LiveApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMetadata { field } => Self::display_metadata(formatter, field),
            Self::LiveStreamingNotEnabled => Self::display_live_streaming_disabled(formatter),
            Self::ChannelLiveBlocked => Self::display_channel_blocked(formatter),
            Self::InsufficientPermissions => Self::display_insufficient_permissions(formatter),
            Self::AuthorizationRejected => Self::display_authorization_rejected(formatter),
            Self::QuotaLike { reason } => Self::display_quota(formatter, reason),
            Self::BroadcastState { detail } => Self::display_state(formatter, detail),
            Self::ApiNotFound { resource } => Self::display_not_found(formatter, resource),
            Self::ApiRejected { status, reason } => {
                Self::display_rejected(formatter, *status, reason)
            }
            Self::BackendUnavailable { reason } => Self::display_backend(formatter, reason),
            Self::Network { reason } => Self::display_network(formatter, reason),
            Self::MalformedResponse { field } => Self::display_malformed(formatter, field),
            Self::ReadinessTimeout { stage } => Self::display_timeout(formatter, stage),
            Self::DestinationMissing { field } => Self::display_destination(formatter, field),
        }
    }
}

// Display split per the Phase-03 StreamingError precedent: every helper
// body stays far below the 50-line cap and no arm can panic.
impl LiveApiError {
    fn display_metadata(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "invalid broadcast metadata field '{field}'; fix the configured broadcast/video values and retry"
        )
    }

    fn display_live_streaming_disabled(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "live streaming is not enabled for this YouTube channel; see youtube.com/features"
        )
    }

    fn display_channel_blocked(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "this YouTube channel cannot stream live at this time; see youtube.com/features"
        )
    }

    fn display_insufficient_permissions(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the stored authorization lacks live permissions; run the auth command again"
        )
    }

    fn display_authorization_rejected(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the YouTube API rejected the access token mid-run; run the auth command again"
        )
    }

    fn display_quota(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(
            formatter,
            "the YouTube API reported a quota or rate limit (reason '{reason}'); retry later"
        )
    }

    fn display_state(formatter: &mut fmt::Formatter<'_>, detail: &str) -> fmt::Result {
        write!(
            formatter,
            "the current broadcast state does not allow this operation (reason '{detail}')"
        )
    }

    fn display_not_found(formatter: &mut fmt::Formatter<'_>, resource: &str) -> fmt::Result {
        write!(
            formatter,
            "the YouTube {resource} resource was not found; it was deleted or never existed"
        )
    }

    fn display_rejected(
        formatter: &mut fmt::Formatter<'_>,
        status: u16,
        reason: &str,
    ) -> fmt::Result {
        write!(
            formatter,
            "the YouTube API rejected the request (HTTP {status}, reason '{reason}')"
        )
    }

    fn display_backend(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(
            formatter,
            "the YouTube API backend is temporarily unavailable (reason '{reason}'); retry shortly"
        )
    }

    fn display_network(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(formatter, "unable to reach the YouTube API: {reason}")
    }

    fn display_malformed(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "the YouTube API response is malformed (field '{field}')"
        )
    }

    fn display_timeout(formatter: &mut fmt::Formatter<'_>, stage: &str) -> fmt::Result {
        write!(
            formatter,
            "the YouTube readiness wait timed out (stage '{stage}')"
        )
    }

    fn display_destination(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "the ingestion response is missing the required field '{field}'"
        )
    }
}

impl Error for LiveApiError {}

#[cfg(test)]
#[path = "live_error_tests.rs"]
mod live_error_tests;
