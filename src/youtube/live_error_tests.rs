//! Classifier and Display-contract tests for the typed Live API failures:
//! the documented §1.5 status-first mapping, per-reason metadata fields,
//! and stable secret-free Display lines (plan §5 T3). No variant output
//! may ever carry an RTMPS URL, a stream key, or any URL text.

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
    let broadcast = classify(404, "liveBroadcastNotFound");
    let LiveApiError::ApiNotFound { resource } = &broadcast else {
        panic!("expected ApiNotFound, got {broadcast}");
    };
    assert_eq!(resource, "liveBroadcast");

    let stream = classify(404, "liveStreamNotFound");
    let LiveApiError::ApiNotFound { resource } = &stream else {
        panic!("expected ApiNotFound, got {stream}");
    };
    assert_eq!(resource, "liveStream");
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

    let unknown = classify(403, "brandNewReason");
    let LiveApiError::ApiRejected {
        status: mapped_status,
        reason: mapped_reason,
    } = &unknown
    else {
        panic!("expected ApiRejected, got {unknown}");
    };
    assert_eq!(*mapped_status, 403);
    assert_eq!(mapped_reason, "brandNewReason");

    assert!(matches!(
        classify(401, "unauthorized"),
        LiveApiError::AuthorizationRejected
    ));
    assert!(matches!(
        classify_api_error(401, None),
        LiveApiError::AuthorizationRejected
    ));
    assert!(matches!(
        classify_api_error(503, None),
        LiveApiError::BackendUnavailable { .. }
    ));
}

#[test]
fn every_variant_display_stable_and_secret_free() {
    let cases: Vec<(LiveApiError, &str)> = vec![
        (
            LiveApiError::InvalidMetadata {
                field: "snippet.title".to_string(),
            },
            "invalid broadcast metadata field 'snippet.title'; fix the configured broadcast/video values and retry",
        ),
        (
            LiveApiError::LiveStreamingNotEnabled,
            "live streaming is not enabled for this YouTube channel; see youtube.com/features",
        ),
        (
            LiveApiError::ChannelLiveBlocked,
            "this YouTube channel cannot stream live at this time; see youtube.com/features",
        ),
        (
            LiveApiError::InsufficientPermissions,
            "the stored authorization lacks live permissions; run the auth command again",
        ),
        (
            LiveApiError::AuthorizationRejected,
            "the YouTube API rejected the access token mid-run; run the auth command again",
        ),
        (
            LiveApiError::QuotaLike {
                reason: "userRequestsExceedRateLimit".to_string(),
            },
            "the YouTube API reported a quota or rate limit (reason 'userRequestsExceedRateLimit'); retry later",
        ),
        (
            LiveApiError::BroadcastState {
                detail: "invalidTransition".to_string(),
            },
            "the current broadcast state does not allow this operation (reason 'invalidTransition')",
        ),
        (
            LiveApiError::ApiNotFound {
                resource: "liveBroadcast".to_string(),
            },
            "the YouTube liveBroadcast resource was not found; it was deleted or never existed",
        ),
        (
            LiveApiError::ApiRejected {
                status: 403,
                reason: "brandNewReason".to_string(),
            },
            "the YouTube API rejected the request (HTTP 403, reason 'brandNewReason')",
        ),
        (
            LiveApiError::BackendUnavailable {
                reason: "backendError".to_string(),
            },
            "the YouTube API backend is temporarily unavailable (reason 'backendError'); retry shortly",
        ),
        (
            LiveApiError::Network {
                reason: "connection refused".to_string(),
            },
            "unable to reach the YouTube API: connection refused",
        ),
        (
            LiveApiError::MalformedResponse {
                field: "status.lifeCycleStatus".to_string(),
            },
            "the YouTube API response is malformed (field 'status.lifeCycleStatus')",
        ),
        (
            LiveApiError::ReadinessTimeout {
                stage: "readiness".to_string(),
            },
            "the YouTube readiness wait timed out (stage 'readiness')",
        ),
        (
            LiveApiError::DestinationMissing {
                field: "rtmpsIngestionAddress".to_string(),
            },
            "the ingestion response is missing the required field 'rtmpsIngestionAddress'",
        ),
    ];
    assert_eq!(
        cases.len(),
        14,
        "every variant pins exactly one Display line"
    );
    for (error, expected) in cases {
        let rendered = error.to_string();
        assert_eq!(rendered, expected);
        assert!(
            !rendered.contains("rtmps://"),
            "no RTMPS URL in: {rendered}"
        );
        assert!(!rendered.contains("://"), "no URL text in: {rendered}");
        assert!(
            !rendered.contains("stream key"),
            "no stream-key text in: {rendered}"
        );
    }
}
