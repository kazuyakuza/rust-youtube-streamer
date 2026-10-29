//! YouTube integration boundary: OAuth 2.0 design, typed interfaces and
//! mockable seams for the Phase 04 credential layer (scope authority
//! `.agent/todos/20261012/20261012-todo-1-DONE.md`, Tasks 1–4 landed).
//!
//! Boundary rules:
//! - This module owns all OAuth client and token material; secret values
//!   live only inside the redacting wrapper types and are never stored in
//!   error payloads, `Display`/`Debug` output, or logs.
//! - It must never depend on `chat`, `renderer`, `streaming`, or the `demo`
//!   scheduling code; the dependency direction is one-way into this module.
//! - The transport layer is synchronous (no async runtime); the
//!   [`transport::TokenEndpointTransport`] trait is the mockable seam that
//!   keeps tests offline. The Phase-07 stream seam is the sanctioned
//!   exception: it sits behind [`stream_transport::ChatStreamTransport`]
//!   and its async runtime is confined to Task 3's worker thread.
//! - Typed failures surface through the enums in [`error`] with
//!   hand-written `Display` implementations and no panics.
//!
//! Mockable seams: [`transport::TokenEndpointTransport`],
//! [`credentials::CredentialStore`], [`verify::AuthorizationVerifier`],
//! [`api_transport::LiveApiTransport`],
//! [`api_chat_transport::LiveChatTransport`], and the Phase-07
//! [`stream_transport::ChatStreamTransport`] /
//! [`stream_transport::LiveChatStreamSession`] pair. Every boundary is a
//! fake or a `127.0.0.1` loopback server; the tonic TLS handshake is the
//! sole offline gap (operator manual E2E).
//!
//! Phase-05 run wiring: the staged design modules (`live_models`,
//! `live_lifecycle`, `live_error`, `scheduled_start_time`, `api_transport`)
//! are consumed by the concrete backend (`google_live_api_transport` plus
//! its `api_request` wire sibling), the orchestrator (`live_ingestion`),
//! the readiness and confirmation waits (`readiness_wait`), the destination
//! wrapper (`rtmps_ingestion`) and the cancellation seam
//! (`wait_cancellation`); the run-mode re-exports below name their
//! consumers.
//!
//! Phase-06 chat layer (record D-1..D-4/D-7, plan
//! `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`): the
//! chat seam, concrete transport, O6 resolution, mapping, dedup and models
//! plus 2 `LiveApiError` chat classes; youtube stays chat-free (`crate::chat`
//! belongs to the app side).
//!
//! Phase-07 streaming layer (Task 2, plan
//! `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md`): the
//! streamList gRPC seam + prost wire models + Bearer-only client + grpc-status
//! mapper; the resumable loop and the confined runtime thread are Task 3's.
//!
//! Standing rule for all of this module's tests: no network, no Google
//! contact, no real credentials — every boundary is a fake or a
//! `127.0.0.1` loopback server (fixtures only).

mod api_chat_transport;
mod api_request;
mod api_transport;
mod authorization;
mod client_config;
mod credential_service;
mod credential_service_error;
mod credentials;
mod error;
mod file_credential_store;
mod google_authorization_verifier;
mod google_live_api_transport;
mod google_live_chat_transport;
mod google_token_endpoint;
mod live_chat_dedup;
mod live_chat_mapping;
mod live_chat_models;
mod live_chat_resolution;
mod live_chat_stream_mapping;
mod live_error;
mod live_ingestion;
// Wired by the Phase-05 run runtime; the residual item is the
// pending-state predicate (`is_transition_pending`) reserved for a future
// phase that reads it — retire when consumption lands.
#[allow(dead_code)]
mod live_lifecycle;
// Wired by the Phase-05 run runtime; the residual items are the
// documented-but-unread resource fields reserved for a future phase that
// reads them — retire when consumption lands.
#[allow(dead_code)]
mod live_models;
mod persisted_credentials;
mod readiness_wait;
mod rtmps_ingestion;
mod scheduled_start_time;
mod stream_client;
mod stream_concrete;
mod stream_proto;
mod stream_transport;
mod token;
mod transport;
mod verify;
mod wait_cancellation;

// Shared offline test fixtures for the Phase-05 Task-2 runtime tests (the
// `config::test_fixtures` precedent; Amendment 1 ruling 1); never built in
// production and never reaching Google.
#[cfg(test)]
mod live_test_fixtures;

// Shared offline stream fixtures for the Phase-07 rosters (Task 2) and the
// Task-3 worker rosters; never built in production.
#[cfg(test)]
mod stream_test_fixtures;

pub(crate) use api_chat_transport::LiveChatTransport;
pub(crate) use api_transport::LiveApiTransport;
pub(crate) use authorization::{
    authorization_url, generate_pkce, generate_state, parse_authorization_callback,
    AuthorizationCallback, REQUIRED_SCOPE,
};
pub(crate) use client_config::{parse_client_config, ClientConfig};
pub(crate) use credential_service::{CredentialService, VerifiedCredentials};
pub(crate) use credential_service_error::CredentialServiceError;
pub(crate) use credentials::{CredentialStore, StoredCredentials};
pub(crate) use error::{
    AuthorizationError, ClientConfigError, CredentialStoreError, TokenEndpointError,
};
pub(crate) use file_credential_store::FileCredentialStore;
pub(crate) use google_authorization_verifier::GoogleAuthorizationVerifier;
pub(crate) use google_live_api_transport::GoogleLiveApiTransport;
pub(crate) use google_token_endpoint::GoogleTokenEndpoint;
pub(crate) use live_chat_dedup::ChatDedupWindow;
pub(crate) use live_chat_resolution::resolve_live_chat_id;
#[cfg(test)]
pub(crate) use live_chat_resolution::resolve_live_chat_id_with_interval;
pub(crate) use live_error::LiveApiError;
pub(crate) use live_ingestion::{LiveBroadcastPreparer, PreparedBroadcast};
pub(crate) use live_models::LiveStream;
pub(crate) use readiness_wait::{wait_until_stream_active, StreamObservationRequest};
// Phase-07 landed surface (Task-3 rewire; record P7-12): the run chat
// worker consumes the stream seam, the concrete transport, and the
// mapped carrier.
pub(crate) use live_chat_mapping::ChatTextMessage;
pub(crate) use stream_concrete::GoogleLiveChatStreamTransport;
pub(crate) use stream_transport::{
    ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession, YieldedChatStreamBatch,
};
// Test-only crate export for the confirmation-gate zero-sleep timeout
// tests (production uses the D-T2-9 budget; no signature changes).
#[cfg(test)]
pub(crate) use readiness_wait::wait_until_stream_active_with_polling;
pub(crate) use token::{AccessToken, AuthorizationCode, TokenResponse};
pub(crate) use transport::{CodeExchangeRequest, TokenEndpointTransport};
pub(crate) use verify::AuthorizationVerifier;
pub(crate) use wait_cancellation::WaitCanceller;
// Test-only re-exports: the auth-flow and run-runtime offline fakes
// hand-build token/scope values and token-endpoint requests, which
// requires naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use token::{GrantedScopes, RefreshToken};
#[cfg(test)]
pub(crate) use transport::RefreshRequest;
// Test-only re-exports: the run runtime's offline transport fake scripts
// live resources through these models; production code never uses them.
#[cfg(test)]
pub(crate) use live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast,
};
// Test-only re-exports: the run runtime's offline fakes implement the
// transport/verifier seams and script wait outcomes, which requires
// naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use error::VerificationError;
#[cfg(test)]
pub(crate) use transport::TokenExchangeResult;
#[cfg(test)]
pub(crate) use verify::VerifiedAuthorization;
// Test-only stream fixture re-exports: the Phase-07 rosters build carriers
// and scripted sessions through these names; production never uses them.
#[cfg(test)]
pub(crate) use stream_test_fixtures::{
    ended_item, response_offline, response_plain, response_with_token, stream_test_block_on,
    text_item, tombstone_item, yielded, RunChatStreamLedger, ScriptedChatStreamTransport,
    ScriptedRunStreamTransport, ScriptedSessionSpec, ACCESS_TOKEN_FIXTURE, LIVE_CHAT_ID_FIXTURE,
    PUBLISHED_AT_FIXTURE, RESUME_TOKEN_FIXTURE,
};
