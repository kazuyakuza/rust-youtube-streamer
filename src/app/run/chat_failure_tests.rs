//! The pure failure-policy pins retained after the streamList rewire
//! (plan §C.7): the CLOSED transient matrix (Network / backend / malformed
//! / the rate-limit quota reason transient; every other class — including
//! the two Phase-07 stream classes — not) and the three-way terminal
//! mapping. Offline only — pure classifiers, no transport.

use crate::app::pipeline::ChatRetrievalError;
use crate::youtube::LiveApiError;

use super::{is_transient_failure, terminal_retrieval_error};

#[test]
fn closed_transient_matrix_is_pinned() {
    assert!(is_transient_failure(&LiveApiError::Network {
        reason: "network-fixture".to_string(),
    }));
    assert!(is_transient_failure(&LiveApiError::BackendUnavailable {
        reason: "unavailable-fixture".to_string(),
    }));
    assert!(is_transient_failure(&LiveApiError::MalformedResponse {
        field: "stream response".to_string(),
    }));
    assert!(is_transient_failure(&LiveApiError::QuotaLike {
        reason: "rateLimitExceeded".to_string(),
    }));
    assert!(!is_transient_failure(&LiveApiError::QuotaLike {
        reason: "quotaExceeded".to_string(),
    }));
    assert!(!is_transient_failure(&LiveApiError::ChatPrecondition {
        detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED".to_string(),
    }));
    assert!(!is_transient_failure(&LiveApiError::InvalidChatRequest {
        detail: "invalid argument".to_string(),
    }));
}

#[test]
fn terminal_mapping_is_three_way_with_the_stream_classes() {
    let quota = terminal_retrieval_error(&LiveApiError::QuotaLike {
        reason: "quotaExceeded".to_string(),
    });
    assert!(matches!(quota, ChatRetrievalError::Quota { .. }));

    for error in [
        LiveApiError::ChatPrecondition {
            detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED".to_string(),
        },
        LiveApiError::InvalidChatRequest {
            detail: "invalid argument".to_string(),
        },
        LiveApiError::InsufficientPermissions,
    ] {
        let mapped = terminal_retrieval_error(&error);
        assert!(
            matches!(mapped, ChatRetrievalError::ChatUnavailable { .. }),
            "every non-quota class maps into ChatUnavailable: {error}"
        );
    }
}
