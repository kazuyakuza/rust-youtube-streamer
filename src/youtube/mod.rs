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
//! - Deliberately deferred: the concrete `FileCredentialStore` is Task 2
//!   work (plan decision S8) and the concrete YouTube verification
//!   request is Task 4 work (plan decision S9); do not implement either
//!   while this layer stays unwired.
//! - The researched Google OAuth facts (endpoints, loopback redirect,
//!   PKCE S256, refresh semantics, the force-ssl scope decision) are
//!   recorded in `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1.

mod authorization;
mod client_config;
mod credentials;
mod error;
mod google_token_endpoint;
mod token;
mod transport;
mod verify;
