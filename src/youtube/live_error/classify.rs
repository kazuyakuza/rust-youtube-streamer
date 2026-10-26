//! The pure status-first §1.5 mapping from HTTP status + server reason
//! code to the typed enum. Bodies are consulted only for the non-secret
//! `reason` string and never embedded.
//!
//! Grounding: the reason codes and their HTTP statuses are the live
//! Data API error list at
//! https://developers.google.com/youtube/v3/live/docs/errors (with the
//! channel-eligibility `insufficientPermissions` case per
//! https://developers.google.com/youtube/v3/live/authentication); the
//! `field` spellings mirror the resource paths of the liveBroadcasts and
//! liveStreams references cited in `live_models`.
//!
//! Staging: dead until Task 2's concrete transport routes every non-2xx
//! response through it.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.5, F4); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

use super::LiveApiError;

/// Reason used when a rejection body carried none (or was unreadable).
const UNSPECIFIED_REASON: &str = "unspecified";
/// API resource-name spellings for the documented not-found reasons.
const BROADCAST_RESOURCE: &str = "liveBroadcast";
const STREAM_RESOURCE: &str = "liveStream";
/// The chat-side not-found spelling (record D-7): shared with the O6
/// resolution budget's typed absence via the `live_error` re-export.
pub(crate) const LIVE_CHAT_RESOURCE: &str = "live chat";

/// Pure canonical §1.5 mapping; exactly two arguments (status, reason).
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
        "liveChatDisabled" => LiveApiError::ChatDisabled,
        "liveChatEnded" => LiveApiError::ChatEnded,
        "forbidden" => LiveApiError::InsufficientPermissions,
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
        "liveChatNotFound" => LiveApiError::ApiNotFound {
            resource: LIVE_CHAT_RESOURCE.to_string(),
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
/// uses config paths instead, e.g. "youtube.broadcast.title"). The bare
/// `required` reason names no field, so the server text becomes the field.
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
