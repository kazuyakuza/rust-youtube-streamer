//! Hand-written `Display` for [`LiveApiError`]: stable, end-user-
//! reportable, secret-free lines split into per-variant helpers (the
//! Phase-03 `StreamingError` precedent) so every body stays far below
//! the 50-line cap and no arm can panic.

use std::fmt;

use super::LiveApiError;

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

impl LiveApiError {
    fn display_metadata(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "invalid metadata field '{field}'; fix broadcast config and retry"
        )
    }

    fn display_live_streaming_disabled(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "live streaming is not enabled; see youtube.com/features"
        )
    }

    fn display_channel_blocked(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "this channel cannot stream live now; see youtube.com/features"
        )
    }

    fn display_insufficient_permissions(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "authorization lacks live permissions; run auth again"
        )
    }

    fn display_authorization_rejected(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "access token rejected mid-run; run auth again")
    }

    fn display_quota(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(
            formatter,
            "YouTube quota or rate limit (reason '{reason}'); retry later"
        )
    }

    fn display_state(formatter: &mut fmt::Formatter<'_>, detail: &str) -> fmt::Result {
        write!(
            formatter,
            "broadcast state forbids this operation (reason '{detail}')"
        )
    }

    fn display_not_found(formatter: &mut fmt::Formatter<'_>, resource: &str) -> fmt::Result {
        write!(
            formatter,
            "YouTube {resource} not found; deleted or never created"
        )
    }

    fn display_rejected(
        formatter: &mut fmt::Formatter<'_>,
        status: u16,
        reason: &str,
    ) -> fmt::Result {
        write!(
            formatter,
            "YouTube rejected request (HTTP {status}, reason '{reason}')"
        )
    }

    fn display_backend(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(
            formatter,
            "YouTube backend temporarily unavailable (reason '{reason}')"
        )
    }

    fn display_network(formatter: &mut fmt::Formatter<'_>, reason: &str) -> fmt::Result {
        write!(formatter, "unable to reach the YouTube API: {reason}")
    }

    fn display_malformed(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "malformed YouTube API response (field '{field}')"
        )
    }

    fn display_timeout(formatter: &mut fmt::Formatter<'_>, stage: &str) -> fmt::Result {
        write!(
            formatter,
            "YouTube readiness wait timed out (stage '{stage}')"
        )
    }

    fn display_destination(formatter: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
        write!(
            formatter,
            "ingestion response missing required field '{field}'"
        )
    }
}
