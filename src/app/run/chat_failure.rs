//! The worker's typed failure policy (record D-6/D-8; transcribed from
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §3):
//! the CLOSED transient class — Network / 5xx-backend / malformed
//! responses plus the chat rate-limit `QuotaLike` reason — decides retry
//! with backoff, and the three-way terminal mapping funnels every other
//! classified `LiveApiError` into the typed `ChatRetrievalError` (detail
//! = the redacted inner Display only). `ChatEnded` never reaches here:
//! the poll cycle handles it first as the graceful end-of-chat signal.

use crate::app::pipeline::ChatRetrievalError;
use crate::youtube::LiveApiError;

const RATE_LIMIT_EXCEEDED_REASON: &str = "rateLimitExceeded";

/// D-6's CLOSED transient class: Network / 5xx-backend / malformed
/// responses, plus the chat rate-limit QuotaLike reason; everything else
/// on a chat call is terminal (unknown quota reasons included — the app
/// never hammers a quota-shaped failure).
pub(super) fn is_transient_failure(error: &LiveApiError) -> bool {
    match error {
        LiveApiError::Network { .. }
        | LiveApiError::BackendUnavailable { .. }
        | LiveApiError::MalformedResponse { .. } => true,
        LiveApiError::QuotaLike { reason } => reason == RATE_LIMIT_EXCEEDED_REASON,
        _ => false,
    }
}

/// D-8's three-way terminal mapping. `detail` = the inner Display only.
pub(super) fn terminal_retrieval_error(error: &LiveApiError) -> ChatRetrievalError {
    match error {
        LiveApiError::QuotaLike { .. } => ChatRetrievalError::Quota {
            detail: error.to_string(),
        },
        _ => ChatRetrievalError::ChatUnavailable {
            detail: error.to_string(),
        },
    }
}

#[cfg(test)]
#[path = "chat_failure_tests.rs"]
mod chat_failure_tests;
