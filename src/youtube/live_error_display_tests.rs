//! Display-contract tests for the typed Live API failures: every variant
//! pins exactly one stable, end-user-reportable line, and no rendered
//! line may ever carry an RTMPS URL, a stream key, or any URL text
//! (the D-T1-11 secret-surface regression pair; sibling of
//! `live_error_tests.rs` under the line-cap rule).

use super::*;

/// One instance of every `LiveApiError` variant, in the pinned line order.
fn all_error_variants() -> Vec<LiveApiError> {
    vec![
        LiveApiError::InvalidMetadata {
            field: "snippet.title".to_string(),
        },
        LiveApiError::LiveStreamingNotEnabled,
        LiveApiError::ChannelLiveBlocked,
        LiveApiError::InsufficientPermissions,
        LiveApiError::AuthorizationRejected,
        LiveApiError::QuotaLike {
            reason: "userRequestsExceedRateLimit".to_string(),
        },
        LiveApiError::BroadcastState {
            detail: "invalidTransition".to_string(),
        },
        LiveApiError::ApiNotFound {
            resource: "liveBroadcast".to_string(),
        },
        LiveApiError::ApiRejected {
            status: 403,
            reason: "brandNewReason".to_string(),
        },
        LiveApiError::BackendUnavailable {
            reason: "backendError".to_string(),
        },
        LiveApiError::Network {
            reason: "connection refused".to_string(),
        },
        LiveApiError::MalformedResponse {
            field: "status.lifeCycleStatus".to_string(),
        },
        LiveApiError::ReadinessTimeout {
            stage: "readiness".to_string(),
        },
        LiveApiError::DestinationMissing {
            field: "rtmpsIngestionAddress".to_string(),
        },
        LiveApiError::ChatDisabled,
        LiveApiError::ChatEnded,
    ]
}

/// The pinned stable Display line for each variant of `all_error_variants`.
fn expected_display_lines() -> [&'static str; 16] {
    [
        "invalid metadata field 'snippet.title'; fix broadcast config and retry",
        "live streaming is not enabled; see youtube.com/features",
        "this channel cannot stream live now; see youtube.com/features",
        "authorization lacks live permissions; run auth again",
        "access token rejected mid-run; run auth again",
        "YouTube quota or rate limit (reason 'userRequestsExceedRateLimit'); retry later",
        "broadcast state forbids this operation (reason 'invalidTransition')",
        "YouTube liveBroadcast not found; deleted or never created",
        "YouTube rejected request (HTTP 403, reason 'brandNewReason')",
        "YouTube backend temporarily unavailable (reason 'backendError')",
        "unable to reach the YouTube API: connection refused",
        "malformed YouTube API response (field 'status.lifeCycleStatus')",
        "YouTube readiness wait timed out (stage 'readiness')",
        "ingestion response missing required field 'rtmpsIngestionAddress'",
        "live chat is disabled for this broadcast; enable chat and reuse its id",
        "the live chat has ended",
    ]
}

#[test]
fn every_variant_display_stable_and_secret_free() {
    let rendered: Vec<String> = all_error_variants()
        .iter()
        .map(ToString::to_string)
        .collect();
    let expected = expected_display_lines();
    assert_eq!(rendered.len(), 16, "every variant pins exactly one line");
    for (index, (actual, wanted)) in rendered.iter().zip(&expected).enumerate() {
        assert_eq!(actual, wanted, "variant #{index} Display line drifted");
        assert!(!actual.contains("rtmps://"), "no RTMPS URL in: {actual}");
        assert!(!actual.contains("://"), "no URL text in: {actual}");
        assert!(!actual.contains("stream key"), "no stream key in: {actual}");
    }
}

#[test]
fn wait_cancelled_display_is_stable_and_secret_free() {
    let rendered = LiveApiError::WaitCancelled.to_string();
    assert_eq!(
        rendered,
        "the YouTube readiness wait was cancelled by the stop request"
    );
    assert!(!rendered.contains("rtmps"), "no RTMPS text in: {rendered}");
    assert!(!rendered.contains("Bearer"), "no token text in: {rendered}");
    assert!(!rendered.contains("url"), "no URL text in: {rendered}");
}

#[test]
fn chat_disabled_display_is_exact_line() {
    let rendered = LiveApiError::ChatDisabled.to_string();
    assert_eq!(
        rendered,
        "live chat is disabled for this broadcast; enable chat and reuse its id"
    );
    assert!(!rendered.contains("://"), "no URL text in: {rendered}");
    assert!(!rendered.contains("Bearer"), "no token text in: {rendered}");
}

#[test]
fn chat_ended_display_is_exact_line() {
    let rendered = LiveApiError::ChatEnded.to_string();
    assert_eq!(rendered, "the live chat has ended");
    assert!(!rendered.contains("://"), "no URL text in: {rendered}");
}
