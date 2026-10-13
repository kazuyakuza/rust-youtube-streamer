//! YouTube integration boundary: OAuth 2.0 design, typed interfaces and
//! mockable seams for the Phase 04 credential layer (scope authority
//! `.agent/todos/20261012/20261012-todo-1.md`, Tasks 1–4 landed).
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

mod authorization;
mod client_config;
mod credentials;
mod error;
mod file_credential_store;
mod google_token_endpoint;
mod persisted_credentials;
mod token;
mod transport;
mod verify;

// Staged until the future run-phase wiring constructs the load/refresh/
// verify service and its concrete verifier (deliberate staging, the
// Phase-02 `#[allow(dead_code)] mod chat;` precedent; retire there).
#[allow(dead_code)]
mod credential_service;
mod credential_service_error;
// Staged until the future run-phase wiring injects the concrete verifier
// into the credential service (same staging contract as above).
#[allow(dead_code)]
mod google_authorization_verifier;

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
