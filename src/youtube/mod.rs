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
//!   keeps tests offline.
//! - Typed failures surface through the enums in [`error`] with
//!   hand-written `Display` implementations and no panics.
//!
//! Guidance for future agents:
//! - Access path: submodules are private; re-exports are added here only
//!   when the first external consumer appears.
//! - Mockable seams: [`transport::TokenEndpointTransport`],
//!   [`credentials::CredentialStore`] and [`verify::AuthorizationVerifier`];
//!   the credential-service scenarios fake ALL three seams offline
//!   (`credential_service_tests` plus the `_refresh`/`_verification`
//!   scenario siblings); the concrete verifier is tested with the real
//!   ureq stack against loopback fake servers
//!   (`google_authorization_verifier_tests`) and its pure status+body
//!   mapping in `google_authorization_verifier_classify_tests`. No test
//!   ever reaches Google.
//! - Landed vs deferred: the concrete credential store is implemented
//!   (`file_credential_store` for the atomic file IO and 0600 permission
//!   handling, `persisted_credentials` for the on-disk JSON schema); its
//!   atomicity, POSIX 0600, Windows ACL/rename and redaction limits are
//!   documented in those two module docs, and the `oauth.*` path defaults
//!   in `config/config.example.json`. The concrete YouTube verification
//!   request is landed in `google_authorization_verifier` (the minimal
//!   channels.list(mine=true) probe for the force-ssl scope) and the
//!   reusable load/refresh/verify pipeline in `credential_service`
//!   (verify-first refresh decision, typed failure classes).
//! - The researched Google OAuth facts (endpoints, loopback redirect,
//!   PKCE S256, refresh semantics, the force-ssl scope decision) are
//!   recorded in `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1;
//!   the verify-first refresh decision, the channels.list(mine=true)
//!   probe evidence and the status→error mapping table are in
//!   `.kilo/plans/20261012-phase04-task4-credential-service.md`.
//!
//! Phase-05 live runtime (TODO Tasks 2–3, plan
//! `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! plus Amendments 1–2 and
//! `.kilo/plans/20261013-phase05-task3-run-orchestration.md`): the five
//! Task-1 staged design modules (`live_models`, `live_lifecycle`,
//! `live_error`, `scheduled_start_time`, `api_transport`) are consumed by
//! the concrete backend (`google_live_api_transport` + its `api_request`
//! wire sibling), the orchestrator (`live_ingestion`), the readiness and
//! confirmation waits (`readiness_wait`), the destination wrapper
//! (`rtmps_ingestion`) and the cancellation seam (`wait_cancellation`);
//! their whole-module staging allows were deleted in the Task-2
//! retirement act. The Task-3 run wiring has now landed: the remaining
//! module-level staging allows (`api_request`, `credential_service`,
//! `google_authorization_verifier`, `google_live_api_transport`,
//! `live_ingestion`, `readiness_wait`, `rtmps_ingestion`,
//! `wait_cancellation`, and the D-T2-14-transcribed `live_lifecycle` /
//! `live_models` pair) were deleted there and the production re-exports
//! below added. Residual staging (V2 contract, compiler-arbitrated at the
//! Task-3 checkpoint): whichever of `live_models` / `live_lifecycle`
//! still exposes documented-but-unread resource fields / the pending-state
//! predicate carries a module-level allow whose comment names the future
//! consuming phase; every item-level allow retired with its consumer.
//! The only remaining `#[allow(dead_code)]` items anywhere in `src/` are
//! the pre-existing Phase-03/04 reservations (`AuthError::CallbackAbandoned`,
//! `ChatMessage::with_timestamp`, `ChatStore::is_empty`,
//! `REVOCATION_ENDPOINT`, `Authorized.state`,
//! `AuthorizationError::ProviderDeclined`).
//!
//! Standing rule for all of this module's tests: no network, no Google
//! contact, no real credentials — every boundary is a fake or a
//! `127.0.0.1` loopback server (fixtures only).

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
mod google_token_endpoint;
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
mod token;
mod transport;
mod verify;
mod wait_cancellation;

// Shared offline test fixtures for the Phase-05 Task-2 runtime tests (the
// `config::test_fixtures` precedent; Amendment 1 ruling 1); never built in
// production and never reaching Google.
#[cfg(test)]
mod live_test_fixtures;

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
pub(crate) use live_error::LiveApiError;
pub(crate) use live_ingestion::{LiveBroadcastPreparer, PreparedBroadcast};
pub(crate) use readiness_wait::{wait_until_stream_active, StreamObservationRequest};
pub(crate) use token::{AccessToken, AuthorizationCode, TokenResponse};
pub(crate) use transport::{CodeExchangeRequest, TokenEndpointTransport};
pub(crate) use verify::AuthorizationVerifier;
pub(crate) use wait_cancellation::WaitCanceller;
// Test-only re-exports: the auth flow's offline test fakes hand-build
// `TokenResponse` values and implement `TokenEndpointTransport`, which
// requires naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
};
#[cfg(test)]
pub(crate) use token::{GrantedScopes, RefreshToken};
#[cfg(test)]
pub(crate) use transport::RefreshRequest;
// Test-only re-exports: the run runtime's offline fakes implement the
// transport/verifier seams and script wait outcomes, which requires
// naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use error::VerificationError;
#[cfg(test)]
pub(crate) use transport::TokenExchangeResult;
#[cfg(test)]
pub(crate) use verify::VerifiedAuthorization;
