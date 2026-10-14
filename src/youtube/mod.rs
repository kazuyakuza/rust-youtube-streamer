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
//! - Staging contract: the module-level `#[allow(dead_code)]` declarations
//!   for `credential_service` and `google_authorization_verifier` are
//!   deliberate staging; the future run-phase wiring removes BOTH
//!   attributes, adds the re-exports its consumers need, and constructs
//!   the pipeline — until then the `run` placeholder behavior (exit 3)
//!   is unchanged.
//! - Landed vs deferred: the concrete credential store is implemented
//!   (`file_credential_store` for the atomic file IO and 0600 permission
//!   handling, `persisted_credentials` for the on-disk JSON schema); its
//!   atomicity, POSIX 0600, Windows ACL/rename and redaction limits are
//!   documented in those two module docs, and the `oauth.*` path defaults
//!   in `config/config.example.json`. The concrete YouTube verification
//!   request is landed in `google_authorization_verifier` (the minimal
//!   channels.list(mine=true) probe for the force-ssl scope) and the
//!   reusable load/refresh/verify pipeline in `credential_service`
//!   (verify-first refresh decision, typed failure classes); both stay
//!   staged, unwired until the future run-phase wiring.
//! - The researched Google OAuth facts (endpoints, loopback redirect,
//!   PKCE S256, refresh semantics, the force-ssl scope decision) are
//!   recorded in `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1;
//!   the verify-first refresh decision, the channels.list(mine=true)
//!   probe evidence and the status→error mapping table are in
//!   `.kilo/plans/20261012-phase04-task4-credential-service.md`.
//!
//! Phase-05 live runtime (TODO Task 2, plan
//! `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! plus Amendments 1–2): the five Task-1 staged design modules
//! (`live_models`, `live_lifecycle`, `live_error`,
//! `scheduled_start_time`, `api_transport`) are now consumed by the
//! concrete backend (`google_live_api_transport` + its `api_request`
//! wire sibling), the orchestrator (`live_ingestion`), the readiness
//! and confirmation waits (`readiness_wait`), the destination wrapper
//! (`rtmps_ingestion`) and the cancellation seam (`wait_cancellation`);
//! their whole-module staging allows were deleted in that retirement
//! act. The task-2 modules themselves keep staged `#[allow(dead_code)]`
//! declarations (D-T2-14: nothing prod-reachable roots the chain until
//! Task 3's `run` wiring — whichever residual allows the compiler
//! demanded retire there, exactly like the Phase-04
//! `credential_service`/`google_authorization_verifier` staging).
//!
//! Standing rule for all of this module's tests: no network, no Google
//! contact, no real credentials — every boundary is a fake or a
//! `127.0.0.1` loopback server (fixtures only).

// Staged for the Task 3 run wiring; retire the allow there (the wire
// plumbing is consumed by the staged transport until that wiring roots
// the chain).
#[allow(dead_code)]
mod api_request;
// Consumed by Task 2's concrete transport/orchestrator/waits; truly live
// when Task 3's run wiring roots the chain.
mod api_transport;
mod authorization;
mod client_config;
// Staged until the future run-phase wiring constructs the load/refresh/
// verify service (deliberate Phase-04 staging; retire there).
#[allow(dead_code)]
mod credential_service;
mod credential_service_error;
mod credentials;
mod error;
mod file_credential_store;
// Staged until the future run-phase wiring injects the concrete verifier
// into the credential service (same staging contract as above).
#[allow(dead_code)]
mod google_authorization_verifier;
// Staged for the Task 3 run wiring; retire the allow there (nothing
// prod-reachable constructs the transport until that wiring roots the
// chain).
#[allow(dead_code)]
mod google_live_api_transport;
mod google_token_endpoint;
// Consumed by Task 2's concrete transport; truly live when Task 3's run
// wiring roots the chain.
mod live_error;
// Staged for the Task 3 run wiring; retire the allow there (nothing
// prod-reachable runs the orchestrator until that wiring roots the
// chain).
#[allow(dead_code)]
mod live_ingestion;
// Consumed by Task 2's backend/orchestrator/waits; truly live when
// Task 3's run wiring roots the chain.
mod live_lifecycle;
mod live_models;
mod persisted_credentials;
// Staged for the Task 3 run wiring; retire the allow there (nothing
// prod-reachable constructs the wait until that wiring roots the chain).
#[allow(dead_code)]
mod readiness_wait;
// Staged for the Task 3 run wiring; retire the allow there (the app glue
// that consumes the destination reveal point is Task 3).
#[allow(dead_code)]
mod rtmps_ingestion;
// Consumed by Task 2's orchestrator; truly live when Task 3's run wiring
// roots the chain.
mod scheduled_start_time;
mod token;
mod transport;
mod verify;
// Staged for the Task 3 run wiring; retire the allow there (the poll
// loops consuming this seam are themselves dead until that wiring roots
// the chain).
#[allow(dead_code)]
mod wait_cancellation;

// Shared offline test fixtures for the Phase-05 Task-2 runtime tests (the
// `config::test_fixtures` precedent; Amendment 1 ruling 1); never built in
// production and never reaching Google.
#[cfg(test)]
mod live_test_fixtures;

pub(crate) use authorization::{
    authorization_url, generate_pkce, generate_state, parse_authorization_callback,
    AuthorizationCallback, REQUIRED_SCOPE,
};
pub(crate) use client_config::{parse_client_config, ClientConfig};
pub(crate) use credentials::{CredentialStore, StoredCredentials};
pub(crate) use error::{
    AuthorizationError, ClientConfigError, CredentialStoreError, TokenEndpointError,
};
pub(crate) use file_credential_store::FileCredentialStore;
pub(crate) use google_token_endpoint::GoogleTokenEndpoint;
pub(crate) use token::{AuthorizationCode, TokenResponse};
pub(crate) use transport::{CodeExchangeRequest, TokenEndpointTransport};
// Test-only re-exports: the auth flow's offline test fakes hand-build
// `TokenResponse` values and implement `TokenEndpointTransport`, which
// requires naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use token::{AccessToken, GrantedScopes, RefreshToken};
#[cfg(test)]
pub(crate) use transport::RefreshRequest;
