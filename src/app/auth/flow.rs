//! The ordered auth flow (plan §1/D8): read + parse the configured OAuth
//! client file FIRST, bind the loopback listener, open the authorization
//! (URL print + callback wait), exchange the code, validate the granted
//! scope and refresh token OFFLINE (D9 — the network verifier stays
//! Task 4), persist credentials through the injected store, print the
//! success line. `oauth2::PkceCodeVerifier` appears only as opaque
//! plumbing carried through [`OpenedFlow`]; all Google-protocol logic
//! stays in `src/youtube/`.

use std::fs;
use std::path::Path;

use oauth2::PkceCodeVerifier;
use tracing::info;

use crate::config::AppConfig;
use crate::youtube::{
    authorization_url, generate_pkce, generate_state, parse_client_config, AuthorizationCode,
    ClientConfig, CodeExchangeRequest, CredentialStore, StoredCredentials, TokenEndpointTransport,
    TokenResponse, REQUIRED_SCOPE,
};

use super::console;
use super::listener;
use super::AuthError;

/// One parameterization of the auth flow: the already-validated app config,
/// the token-endpoint transport (real in production, fake in tests) and the
/// credential store. The struct exists because three inputs would exceed the
/// two-argument function cap.
pub(super) struct AuthFlow<'a> {
    pub(super) config: &'a AppConfig,
    pub(super) transport: &'a dyn TokenEndpointTransport,
    pub(super) store: &'a dyn CredentialStore,
}

/// Code + PKCE verifier + redirect URI delivered by the listener, with the
/// listener ALREADY closed (the value moves here only after the wait ended).
/// It is consumed by value at the exchange: `oauth2::PkceCodeVerifier`
/// deliberately does not implement Clone, and the verifier must be used
/// exactly once.
struct OpenedFlow {
    code: AuthorizationCode,
    pkce_verifier: PkceCodeVerifier,
    redirect_uri: String,
}

impl AuthFlow<'_> {
    /// Production entry: reads + parses the client config FIRST, then binds
    /// the loopback listener, then runs the ordered flow (D5 ordering). The
    /// listener drops deterministically at this function's return, whatever
    /// the outcome is (D6).
    pub(super) fn run(&self) -> Result<(), AuthError> {
        let client_config = read_client_config(&self.config.oauth.client_config_path)?;
        let bound = listener::bind()?;
        self.run_with_listener(&client_config, &bound)
    }

    /// Test-visible seam: the whole ordered flow against an already-parsed
    /// client config and a caller-supplied listener. Production calls only
    /// `run`; tests call this directly with a prebound listener and a
    /// tempdir-parsed client config (F11). The body order is the flow
    /// contract: open authorization, exchange, validate, persist, print.
    pub(super) fn run_with_listener(
        &self,
        client_config: &ClientConfig,
        bound: &listener::BoundListener,
    ) -> Result<(), AuthError> {
        info!("authorization flow started");
        let opened = self.open_authorization(client_config, bound)?;
        let response = self.exchange_authorization_code(client_config, opened)?;
        let stored = storeable_credentials(response)?;
        self.save_validated_credentials(&stored)?;
        info!("credentials persisted");
        console::print_success(&self.config.oauth.credentials_path);
        Ok(())
    }

    /// Builds the URL, prints it, waits for the authorized callback. The
    /// client config was READ+PARSED before this runs (D5 ordering); `bound`
    /// is borrowed and stays alive for the accept loop, dropping when `run`
    /// returns (success AND failure) — D6 shutdown semantics.
    fn open_authorization(
        &self,
        client_config: &ClientConfig,
        bound: &listener::BoundListener,
    ) -> Result<OpenedFlow, AuthError> {
        let state = generate_state();
        let pkce = generate_pkce();
        let url = authorization_url(client_config, bound.redirect_uri(), &state, &pkce);
        console::print_authorization_url(url.as_str(), bound.redirect_uri());
        let code = bound.wait_for_authorized_code(&state)?;
        Ok(OpenedFlow {
            code,
            pkce_verifier: pkce.verifier,
            redirect_uri: bound.redirect_uri().to_string(),
        })
    }

    fn exchange_authorization_code(
        &self,
        client_config: &ClientConfig,
        opened: OpenedFlow,
    ) -> Result<TokenResponse, AuthError> {
        let request = code_exchange_request(client_config, opened);
        let response = self
            .transport
            .authorization_code_exchange(&request)
            .map_err(AuthError::TokenExchange)?;
        info!("authorization code exchanged");
        Ok(response)
    }

    fn save_validated_credentials(&self, stored: &StoredCredentials) -> Result<(), AuthError> {
        self.store.save(stored).map_err(AuthError::Persistence)
    }
}

/// Reads + parses the configured client-secret file (D5; one param).
pub(super) fn read_client_config(client_config_path: &str) -> Result<ClientConfig, AuthError> {
    let client_path = Path::new(client_config_path);
    let text = fs::read_to_string(client_path).map_err(|source| AuthError::ClientConfigFile {
        path: client_config_path.to_string(),
        source,
    })?;
    parse_client_config(&text).map_err(AuthError::ClientConfig)
}

/// Verifies the exchanged response carries the required scope and a refresh
/// token, then moves its fields into the persisted credential shape (D8).
pub(super) fn storeable_credentials(
    response: TokenResponse,
) -> Result<StoredCredentials, AuthError> {
    if !response.granted_scopes.grants(REQUIRED_SCOPE) {
        return Err(AuthError::InsufficientScope);
    }
    let refresh_token = match response.refresh_token {
        Some(refresh_token) => refresh_token,
        None => return Err(AuthError::MissingRefreshToken),
    };
    info!("required YouTube scope granted");
    Ok(StoredCredentials {
        access_token: response.access_token,
        expires_in_seconds: response.expires_in_seconds,
        refresh_token: Some(refresh_token),
        granted_scopes: response.granted_scopes,
    })
}

/// Builds the code-exchange request from the validated client config and the
/// consumed opened flow (D8; two params — captured by the F7 fake-transport
/// test through `exchange_authorization_code`; the PKCE verifier moves in
/// exactly once). Private: its `OpenedFlow` parameter type is private to
/// this module.
fn code_exchange_request(client_config: &ClientConfig, opened: OpenedFlow) -> CodeExchangeRequest {
    CodeExchangeRequest {
        client_id: client_config.client_id().to_string(),
        client_secret: client_config.client_secret().clone(),
        code: opened.code.clone(),
        redirect_uri: opened.redirect_uri.clone(),
        pkce_verifier: opened.pkce_verifier,
    }
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod flow_tests;

#[cfg(test)]
#[path = "flow_exchange_tests.rs"]
mod flow_exchange_tests;
