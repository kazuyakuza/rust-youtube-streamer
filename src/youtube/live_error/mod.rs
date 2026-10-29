//! Typed Live API failures for the Phase-05 runtime (plan F4, D-T1-4):
//! one enum covering the documented §1.5 failure table plus the bounded
//! wait timeout and locally produced invalid metadata. Messages are
//! end-user-reportable and stable; the RTMPS destination (D-T1-11) never
//! enters this enum, so no variant can leak the stream key or URL.
//! Classification is status-FIRST (Phase-04 precedent: statuses outrank
//! reason strings); only the non-secret server `reason` code is read from
//! bodies, and bodies are never embedded.
//!
//! Grounding: the Google JSON error shape `{"error": {"code", "reason"}}`
//! and the per-reason live-streaming error list are documented by
//! https://developers.google.com/youtube/v3/live/docs/errors; the
//! channel-eligibility failure mode (`insufficientPermissions` when Live
//! Streaming is not enabled) is documented by
//! https://developers.google.com/youtube/v3/live/authentication.
//!
//! Staging: the classifier is consumed by Task 2's concrete transport
//! (`google_live_api_transport`); the Display lines become live when
//! Task 3's `run` path reports them.
//!
//! Phase-06 chat classes (record D-7, plan §3.2): the enum gains
//! `ChatDisabled` (403 `liveChatDisabled`, terminal) and `ChatEnded`
//! (403 `liveChatEnded`, the worker's GRACEFUL end signal), the
//! classifier gains the `liveChatDisabled`/`liveChatEnded`/bare
//! `forbidden` 403 arms plus the 404 `liveChatNotFound` arm resolving to
//! `ApiNotFound { resource: "live chat" }`; every added line is
//! secret-free and carries only the server `reason` string.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.5, D-T1-4, F4); TODO `.agent/todos/20261013/20261013-todo-1.md`
//! Task 1.

pub(crate) use classify::{classify_api_error, LIVE_CHAT_RESOURCE};
pub(crate) use grpc_status::classify_stream_status;

mod classify;
mod display;
mod grpc_status;

/// Failures of the documented YouTube Live operations (O1–O5) and the
/// readiness waits around them.
#[derive(Debug)]
pub(crate) enum LiveApiError {
    InvalidMetadata {
        field: String,
    },
    LiveStreamingNotEnabled,
    ChannelLiveBlocked,
    InsufficientPermissions,
    AuthorizationRejected,
    QuotaLike {
        reason: String,
    },
    BroadcastState {
        detail: String,
    },
    ApiNotFound {
        resource: String,
    },
    ApiRejected {
        status: u16,
        reason: String,
    },
    BackendUnavailable {
        reason: String,
    },
    Network {
        reason: String,
    },
    MalformedResponse {
        field: String,
    },
    ReadinessTimeout {
        stage: String,
    },
    /// The readiness wait observed a stop request (D-T2-11): a typed
    /// cancellation — not a timeout, not a transport/backend failure —
    /// that Task 3 maps to the normal-shutdown path.
    WaitCancelled,
    DestinationMissing {
        field: String,
    },
    /// 403 liveChatDisabled (record D-7): chat is not enabled on the
    /// broadcast — a terminal retrieval failure class.
    ChatDisabled,
    /// 403 liveChatEnded (record D-6/D-7): the chat is no longer live —
    /// the worker's GRACEFUL end-of-chat signal, never a fatal.
    ChatEnded,
    /// gRPC FAILED_PRECONDITION (record §A.4): the chat is DISABLED or
    /// ENDED — the officially documented code-level ambiguity carried in
    /// `detail`; the establishment vs mid-session worker branch (Task 3)
    /// decides fatal vs graceful.
    ChatPrecondition {
        detail: String,
    },
    /// gRPC INVALID_ARGUMENT (record §A.4): a request-shape failure —
    /// terminal (never retried; brief §12.2 posture).
    InvalidChatRequest {
        detail: String,
    },
}

#[cfg(test)]
#[path = "../live_error_tests.rs"]
mod live_error_tests;

#[cfg(test)]
#[path = "../live_error_display_tests.rs"]
mod live_error_display_tests;
