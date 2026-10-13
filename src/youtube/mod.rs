//! YouTube integration boundary: OAuth 2.0 design, typed interfaces and
//! mockable seams for the Phase 04 credential layer (TODO Task 1 scope
//! authority `.agent/todos/20261012/20261012-todo-1.md`).
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
//!   offline fakes live in the matching `*_tests.rs` files.
//! - Landed vs deferred: the concrete credential store is implemented
//!   (`file_credential_store` for the atomic file IO and 0600 permission
//!   handling, `persisted_credentials` for the on-disk JSON schema); its
//!   atomicity, POSIX 0600, Windows ACL/rename and redaction limits are
//!   documented in those two module docs, and the `oauth.*` path defaults
//!   in `config/config.example.json`. The concrete YouTube verification
//!   request remains deferred Task 4 work (plan decision S9); do not add
//!   it while this layer stays unwired.
//! - The researched Google OAuth facts (endpoints, loopback redirect,
//!   PKCE S256, refresh semantics, the force-ssl scope decision) are
//!   recorded in `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1.

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
