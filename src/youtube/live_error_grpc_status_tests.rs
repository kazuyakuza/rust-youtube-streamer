//! Status-mapping pins for the streaming call (plan §E.3; record §A.4/A.5):
//! every documented gRPC code maps to its typed class, the catch-all keeps
//! only the code NUMBER, and no mapped variant ever embeds the server
//! status message. Statuses are constructed in-process; nothing dials out.

use tonic::Code;

use super::*;
use crate::youtube::live_error::LiveApiError;

const MARKER: &str = "stream-message-marker-MUST-NOT-LEAK";

fn mapped(code: Code) -> LiveApiError {
    classify_stream_status(&tonic::Status::new(code, MARKER))
}

#[test]
fn permission_denied_becomes_insufficient_permissions() {
    assert!(matches!(
        mapped(Code::PermissionDenied),
        LiveApiError::InsufficientPermissions
    ));
}

#[test]
fn invalid_argument_becomes_invalid_chat_request() {
    let LiveApiError::InvalidChatRequest { detail } = mapped(Code::InvalidArgument) else {
        panic!("expected InvalidChatRequest");
    };
    assert_eq!(detail, INVALID_CHAT_ARGUMENT_DETAIL);
}

#[test]
fn failed_precondition_carries_the_documented_disabled_or_ended_detail() {
    let LiveApiError::ChatPrecondition { detail } = mapped(Code::FailedPrecondition) else {
        panic!("expected ChatPrecondition");
    };
    assert_eq!(detail, CHAT_PRECONDITION_DETAIL);
}

#[test]
fn not_found_resolves_to_the_live_chat_resource() {
    let LiveApiError::ApiNotFound { resource } = mapped(Code::NotFound) else {
        panic!("expected ApiNotFound");
    };
    assert_eq!(resource, "live chat");
}

#[test]
fn resource_exhausted_carries_the_rate_limit_reason() {
    let LiveApiError::QuotaLike { reason } = mapped(Code::ResourceExhausted) else {
        panic!("expected QuotaLike");
    };
    assert_eq!(reason, STREAM_RATE_LIMIT_REASON);
}

#[test]
fn unauthenticated_becomes_authorization_rejected() {
    assert!(matches!(
        mapped(Code::Unauthenticated),
        LiveApiError::AuthorizationRejected
    ));
}

#[test]
fn unavailable_becomes_backend_unavailable() {
    let LiveApiError::BackendUnavailable { reason } = mapped(Code::Unavailable) else {
        panic!("expected BackendUnavailable");
    };
    assert_eq!(reason, STREAM_UNAVAILABLE_REASON);
}

#[test]
fn internal_becomes_malformed_stream_response() {
    let LiveApiError::MalformedResponse { field } = mapped(Code::Internal) else {
        panic!("expected MalformedResponse");
    };
    assert_eq!(field, STREAM_INTERNAL_FIELD);
}

#[test]
fn cancelled_deadline_exceeded_and_unknown_become_network() {
    for (code, number) in [
        (Code::Cancelled, 1),
        (Code::DeadlineExceeded, 4),
        (Code::Unknown, 2),
    ] {
        let LiveApiError::Network { reason } = mapped(code) else {
            panic!("expected Network for {code:?}");
        };
        assert_eq!(reason, format!("gRPC code {number}"));
    }
}

#[test]
fn unmapped_codes_become_network() {
    let LiveApiError::Network { reason } = mapped(Code::DataLoss) else {
        panic!("expected Network for DataLoss");
    };
    assert_eq!(reason, "gRPC code 15");
}

#[test]
fn chat_precondition_and_invalid_request_display_lines_are_secret_free() {
    let precondition = mapped(Code::FailedPrecondition).to_string();
    assert_eq!(
        precondition,
        "live chat precondition failed (reason 'LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED')"
    );
    let invalid = mapped(Code::InvalidArgument).to_string();
    assert_eq!(
        invalid,
        "invalid stream chat request parameters (reason 'invalid argument')"
    );
    assert!(!precondition.contains(MARKER));
    assert!(!invalid.contains(MARKER));
}

#[test]
fn mapped_stream_statuses_never_embed_the_server_message() {
    let codes = [
        Code::Ok,
        Code::Cancelled,
        Code::Unknown,
        Code::InvalidArgument,
        Code::DeadlineExceeded,
        Code::NotFound,
        Code::AlreadyExists,
        Code::PermissionDenied,
        Code::ResourceExhausted,
        Code::FailedPrecondition,
        Code::Aborted,
        Code::OutOfRange,
        Code::Unimplemented,
        Code::Internal,
        Code::Unavailable,
        Code::DataLoss,
        Code::Unauthenticated,
    ];
    for code in codes {
        let error = mapped(code);
        assert!(
            !error.to_string().contains(MARKER),
            "Display leaked at {code:?}"
        );
        assert!(
            !format!("{error:?}").contains(MARKER),
            "Debug leaked at {code:?}"
        );
    }
}
