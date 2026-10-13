//! The `auth` mode handler: the installed-application OAuth 2.0
//! authorization-code flow with a loopback-only callback listener.
//!
//! Ordered flow (plan §1): read + parse the configured OAuth client file,
//! bind a `127.0.0.1` listener on an ephemeral port, generate CSRF state +
//! PKCE (S256), print the authorization URL to stdout (never auto-launched;
//! no interactive input is ever read), accept the browser callback on the
//! main thread, exchange the code, verify the granted scope offline, persist
//! credentials, print the success line, exit 0. Every failure — client file,
//! listener bind, CSRF state, provider decline, noise-budget exhaustion,
//! token exchange, persistence, insufficient scope, missing refresh token —
//! is reported as exactly one redacted stderr `error:` line and exits with
//! code 1 (the startup-failure class, `demo` precedent); exit 3 stays
//! reserved for the `run` placeholder.
//!
//! Listener lifecycle (D6): the bound listener lives only inside the flow
//! call; the socket closes deterministically on success AND on every
//! failure. No threads, tasks, or listeners survive the mode.
//!
//! Redaction (D16): no token, authorization code, client secret, PKCE
//! verifier, URL, or callback query value ever reaches a log field, a
//! tracing event, or an error message; the only logged field in this
//! submodule is the bound `redirect_port`. Credentials are persisted only
//! after the granted-scope and refresh-token validation passes.
//!
//! Design authority: `.kilo/plans/20261012-phase04-task3-auth-command.md`
//! (decisions D1-D16; §4.5 console spec; §9 limitations). Fake-tested seams
//! for future agents: the listener runs against real loopback sockets in
//! `listener_tests.rs` (L1-L8); `run_with_listener` is exercised end-to-end
//! on the declined path in `flow_tests.rs` (F11); transport and store seams
//! use pure fakes in `flow_exchange_tests.rs` (F7-F10).

use std::fmt;
use std::path::PathBuf;

use crate::app::error::STARTUP_FAILURE_EXIT;
use crate::config::AppConfig;
use crate::youtube::{
    ClientConfigError, CredentialStoreError, FileCredentialStore, GoogleTokenEndpoint,
    TokenEndpointError, REQUIRED_SCOPE,
};

mod console;
mod flow;
mod listener;

use flow::AuthFlow;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;

/// Exit code returned by the auth mode on success.
const AUTH_SUCCESS_EXIT: u8 = 0;

/// Fixed Display line for the terminal CSRF state-mismatch failure.
const STATE_MISMATCH_MESSAGE: &str = "authorization callback state did not match \
the expected value (possible cross-site request forgery); run the authorization \
command again";

/// Fixed Display line for the terminal noise-budget-exhaustion failure.
const ABANDONED_MESSAGE: &str = "no usable authorization callback was received \
within the allowed 8 connection attempts; run the authorization command again \
and re-open the printed URL";

/// Every fatal failure the auth flow can produce. Each variant is one
/// operator-facing failure class; all exit with code 1 (D3).
#[derive(Debug)]
pub(super) enum AuthError {
    /// The configured OAuth client-configuration file could not be read
    /// (missing, unreadable, directory target, invalid UTF-8).
    ClientConfigFile {
        path: String,
        source: std::io::Error,
    },
    /// The file was read but its content is invalid (schema, blank or
    /// endpoint mismatch); carries the youtube-layer error verbatim.
    ClientConfig(ClientConfigError),
    /// The loopback callback listener could not be bound (or its address
    /// could not be read).
    ListenerBind(std::io::Error),
    /// The listener closed without receiving a usable callback: the bounded
    /// noise budget (8 non-callback connection attempts) was exhausted.
    // The attempts count is validated by the listener tests but deliberately
    // kept out of the Display line (D2); the Debug impl alone does not count
    // as a read for dead-code analysis.
    #[allow(dead_code)]
    CallbackAbandoned { attempts: usize },
    /// The callback failed its CSRF state validation (always terminal).
    StateMismatch,
    /// The provider declined the authorization request in the browser
    /// (terminal; `reason` is Google's error code string only, non-secret).
    ProviderDeclined { reason: String },
    /// The authorization-code exchange failed; the youtube-layer error
    /// Display is already redacted.
    TokenExchange(TokenEndpointError),
    /// Credential persistence failed; verbatim delegation.
    Persistence(CredentialStoreError),
    /// The granted scopes do not include the required YouTube scope.
    InsufficientScope,
    /// The token response did not include a refresh token, which later
    /// unattended runs require (brief section 6.3).
    MissingRefreshToken,
}

impl AuthError {
    /// Every auth-origin failure is a post-startup fatal mode failure and
    /// exits with the startup-failure class (code 1), exactly like demo.
    pub(super) fn exit_code(&self) -> u8 {
        STARTUP_FAILURE_EXIT
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::ClientConfigFile { path, source } => write!(
                formatter,
                "unable to read the OAuth client configuration file '{path}': {source}"
            ),
            AuthError::ClientConfig(error) => write!(formatter, "{error}"),
            AuthError::ListenerBind(source) => write!(
                formatter,
                "unable to bind the OAuth callback listener on 127.0.0.1: {source}"
            ),
            AuthError::CallbackAbandoned { .. } => formatter.write_str(ABANDONED_MESSAGE),
            AuthError::StateMismatch => formatter.write_str(STATE_MISMATCH_MESSAGE),
            AuthError::ProviderDeclined { reason } => write!(
                formatter,
                "the authorization was declined by the provider (error code '{reason}'); \
run the authorization command again if this was unintended"
            ),
            AuthError::TokenExchange(error) => write!(formatter, "{error}"),
            AuthError::Persistence(error) => write!(formatter, "{error}"),
            AuthError::InsufficientScope => write!(
                formatter,
                "the granted authorization does not include the required YouTube scope \
({REQUIRED_SCOPE}); no credentials were stored; run the authorization command \
again and approve the requested YouTube permission"
            ),
            AuthError::MissingRefreshToken => write!(
                formatter,
                "the token response did not include a refresh token, which later \
unattended runs require; no credentials were stored; run the authorization \
command again and approve the requested YouTube permission"
            ),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthError::ClientConfigFile { source, .. } => Some(source),
            AuthError::ListenerBind(source) => Some(source),
            _ => None,
        }
    }
}

/// Runs the OAuth authorization flow: 0 after verified persistence, or one
/// redacted stderr `error:` line plus exit 1 on any failure.
pub(super) fn run_auth(config: &AppConfig) -> u8 {
    match auth_outcome(config) {
        Ok(()) => AUTH_SUCCESS_EXIT,
        Err(error) => report_auth_failure(&error),
    }
}

fn auth_outcome(config: &AppConfig) -> Result<(), AuthError> {
    let transport = create_transport()?;
    let store = create_store(config);
    let flow = AuthFlow {
        config,
        transport: &transport,
        store: &store,
    };
    flow.run()
}

fn create_transport() -> Result<GoogleTokenEndpoint, AuthError> {
    GoogleTokenEndpoint::new().map_err(AuthError::TokenExchange)
}

fn create_store(config: &AppConfig) -> FileCredentialStore {
    FileCredentialStore::new(PathBuf::from(&config.oauth.credentials_path))
}

fn report_auth_failure(error: &AuthError) -> u8 {
    eprintln!("error: {error}");
    error.exit_code()
}
