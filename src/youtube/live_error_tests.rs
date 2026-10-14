//! Classifier and Display-contract tests for the typed Live API failures:
//! the documented §1.5 status-first mapping, per-reason metadata fields,
//! and stable secret-free Display lines (plan §5 T3).

use super::*;

fn classify(status: u16, reason: &str) -> LiveApiError {
    classify_api_error(status, Some(reason))
}

fn assert_invalid_metadata(error: LiveApiError, expected_field: &str) {
    let LiveApiError::InvalidMetadata { field } = error else {
        panic!("expected InvalidMetadata, got {error}");
    };
    assert_eq!(field, expected_field);
}

fn assert_not_found(error: LiveApiError, expected_resource: &str) {
    let LiveApiError::ApiNotFound { resource } = &error else {
        panic!("expected ApiNotFound, got {error}");
    };
    assert_eq!(resource, expected_resource);
}

#[test]
fn live_streaming_not_enabled_maps_from_forbidden_reason() {
    let error = classify(403, "liveStreamingNotEnabled");
    assert!(matches!(error, LiveApiError::LiveStreamingNotEnabled));
}

#[test]
fn channel_live_blocked_maps_from_live_permission_blocked() {
    let error = classify(403, "livePermissionBlocked");
    assert!(matches!(error, LiveApiError::ChannelLiveBlocked));
}

#[test]
fn insufficient_permissions_map() {
    let error = classify(403, "insufficientLivePermissions");
    assert!(matches!(error, LiveApiError::InsufficientPermissions));
}

#[test]
fn broadcast_state_reasons_map() {
    for reason in [
        "liveBroadcastBindingNotAllowed",
        "errorStreamInactive",
        "invalidTransition",
        "redundantTransition",
    ] {
        let error = classify(403, reason);
        let LiveApiError::BroadcastState { detail } = &error else {
            panic!("expected BroadcastState for '{reason}', got {error}");
        };
        assert_eq!(detail, reason);
    }
}

#[test]
fn invalid_metadata_reasons_map_by_field() {
    assert_invalid_metadata(classify(400, "invalidTitle"), "snippet.title");
    assert_invalid_metadata(
        classify(400, "invalidPrivacyStatus"),
        "status.privacyStatus",
    );
    assert_invalid_metadata(
        classify(400, "invalidScheduledStartTime"),
        "snippet.scheduledStartTime",
    );
    assert_invalid_metadata(
        classify(400, "invalidAutoStart"),
        "contentDetails.enableAutoStart",
    );
    assert_invalid_metadata(classify(400, "invalidFrameRate"), "cdn.frameRate");
    assert_invalid_metadata(classify(400, "titleRequired"), "snippet.title");
    assert_invalid_metadata(classify(400, "required"), "required");
}

#[test]
fn not_found_reasons_map() {
    assert_not_found(classify(404, "liveBroadcastNotFound"), "liveBroadcast");
    assert_not_found(classify(404, "liveStreamNotFound"), "liveStream");
}

#[test]
fn quota_reasons_map() {
    for (status, reason) in [
        (403, "userRequestsExceedRateLimit"),
        (403, "rateLimitExceeded"),
        (403, "concurrentBroadcastsExceedLimit"),
        (403, "sharedIngestionBroadcastsExceedLimit"),
        (400, "limitExceeded"),
        (400, "userBroadcastsExceedLimit"),
    ] {
        let error = classify(status, reason);
        let LiveApiError::QuotaLike { reason: mapped } = &error else {
            panic!("expected QuotaLike for '{reason}', got {error}");
        };
        assert_eq!(mapped, reason);
    }
}

#[test]
fn backend_and_unknown_map_status_first() {
    for (status, reason) in [
        (500, "backendError"),
        (502, "internalError"),
        (503, "serviceUnavailable"),
        (504, "errorExecutingTransition"),
    ] {
        let error = classify(status, reason);
        assert!(
            matches!(error, LiveApiError::BackendUnavailable { .. }),
            "HTTP {status} '{reason}' must map to BackendUnavailable, got {error}"
        );
    }

    let LiveApiError::ApiRejected {
        status: mapped_status,
        reason: mapped_reason,
    } = classify(403, "brandNewReason")
    else {
        panic!("expected ApiRejected for an unknown 403 reason");
    };
    assert_eq!(mapped_status, 403);
    assert_eq!(mapped_reason, "brandNewReason");
    assert!(matches!(
        classify(401, "unauthorized"),
        LiveApiError::AuthorizationRejected
    ));
    assert!(matches!(
        classify_api_error(503, None),
        LiveApiError::BackendUnavailable { .. }
    ));
}
