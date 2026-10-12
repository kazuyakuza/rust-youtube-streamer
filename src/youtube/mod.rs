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
//! Access path note for future agents: submodules are private; re-exports
//! are added here only when the first external consumer appears.

mod error;
