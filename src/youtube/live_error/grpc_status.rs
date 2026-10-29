//! The pure gRPC-status → typed-LiveApiError mapping for the streaming call
//! (Task-1 record §A.4/§A.5): status-FIRST (only `status.code()` is read —
//! NEVER the server message, record §8); existing `LiveApiError` classes
//! are reused; the two NEW classes carry fixed documented detail labels.
//! Establishment/mid-session branching for `ChatPrecondition` is a worker
//! policy (Task 3), not this classifier (record: `is_transient_failure`
//! stays CLOSED).

use tonic::Code;

use super::classify::LIVE_CHAT_RESOURCE;
use super::LiveApiError;

/// The officially documented DISABLED/ENDED ambiguity (the S1 footnote),
/// carried verbatim as the ONLY detail text (record §8).
pub(crate) const CHAT_PRECONDITION_DETAIL: &str = "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED";
/// The official web-traffic rate-limit reason (record §1.3): the SAME loader
/// literal that keeps the CLOSED transient matcher in `chat_failure`.
pub(crate) const STREAM_RATE_LIMIT_REASON: &str = "rateLimitExceeded";
pub(crate) const INVALID_CHAT_ARGUMENT_DETAIL: &str = "invalid argument";
pub(crate) const STREAM_UNAVAILABLE_REASON: &str = "unavailable";
pub(crate) const STREAM_INTERNAL_FIELD: &str = "stream response";
pub(crate) const STREAM_CODE_PREFIX: &str = "gRPC code";

pub(crate) fn classify_stream_status(status: &tonic::Status) -> LiveApiError {
    match status.code() {
        Code::PermissionDenied => LiveApiError::InsufficientPermissions,
        Code::InvalidArgument => LiveApiError::InvalidChatRequest {
            detail: INVALID_CHAT_ARGUMENT_DETAIL.to_string(),
        },
        Code::FailedPrecondition => LiveApiError::ChatPrecondition {
            detail: CHAT_PRECONDITION_DETAIL.to_string(),
        },
        Code::NotFound => LiveApiError::ApiNotFound {
            resource: LIVE_CHAT_RESOURCE.to_string(),
        },
        Code::ResourceExhausted => LiveApiError::QuotaLike {
            reason: STREAM_RATE_LIMIT_REASON.to_string(),
        },
        Code::Unauthenticated => LiveApiError::AuthorizationRejected,
        Code::Unavailable => LiveApiError::BackendUnavailable {
            reason: STREAM_UNAVAILABLE_REASON.to_string(),
        },
        Code::Internal => LiveApiError::MalformedResponse {
            field: STREAM_INTERNAL_FIELD.to_string(),
        },
        _ => LiveApiError::Network {
            reason: stream_code_reason(status.code()),
        },
    }
}

/// The sanitized class shape for any remaining code: the sanctioned gRPC
/// code NUMBER only (record §8 — "the gRPC status code number").
fn stream_code_reason(code: Code) -> String {
    format!("{STREAM_CODE_PREFIX} {}", i32::from(code))
}

#[cfg(test)]
#[path = "../live_error_grpc_status_tests.rs"]
mod live_error_grpc_status_tests;
