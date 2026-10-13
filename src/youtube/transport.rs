//! Google token endpoint transport: the mockable [`TokenEndpointTransport`]
//! seam and its request/response contracts. The ureq backend lives in
//! [`super::google_token_endpoint`].

use super::error::TokenEndpointError;
use super::token::{AuthorizationCode, ClientSecret, RefreshToken, TokenResponse};

/// Authorization-code exchange input for [`TokenEndpointTransport`].
/// Secret-bearing fields are wrapper types; the struct derives no
/// `Debug`, so its contents cannot leak through formatting.
pub(crate) struct CodeExchangeRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) code: AuthorizationCode,
    /// Must equal the redirect URI used in the authorization request
    /// (loopback `http://127.0.0.1:<port>`).
    pub(crate) redirect_uri: String,
    pub(crate) pkce_verifier: oauth2::PkceCodeVerifier,
}

/// Refresh-grant exchange input for [`TokenEndpointTransport`].
pub(crate) struct RefreshRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) refresh_token: RefreshToken,
}

/// Result shared by both token-exchange operations of the seam.
pub(crate) type TokenExchangeResult = Result<TokenResponse, TokenEndpointError>;

/// The single mockable seam Google-dependent code talks to. Offline
/// fakes implement it (see `transport_tests.rs`); the real ureq-backed
/// implementation lives in [`super::google_token_endpoint`].
pub(crate) trait TokenEndpointTransport {
    /// Exchanges a callback authorization code with its PKCE verifier
    /// and the exact redirect URI used, returning application-owned
    /// token material.
    fn authorization_code_exchange(&self, request: &CodeExchangeRequest) -> TokenExchangeResult;

    /// Trades a stored refresh token for a fresh access token. Google
    /// may omit the refresh token from a refresh response: callers must
    /// then keep the previously stored one (plan §1.1).
    fn refresh_token_exchange(&self, request: &RefreshRequest) -> TokenExchangeResult;
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod transport_tests;
