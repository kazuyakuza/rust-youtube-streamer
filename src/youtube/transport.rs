//! Google token endpoint transport: the mockable [`TokenEndpointTransport`]
//! seam and its request/response contracts. The ureq backend lives in
//! [`super::google_token_endpoint`].

use super::error::TokenEndpointError;
use super::token::{AuthorizationCode, ClientSecret, RefreshToken, TokenResponse};

pub(crate) struct CodeExchangeRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) code: AuthorizationCode,
    pub(crate) redirect_uri: String,
    pub(crate) pkce_verifier: oauth2::PkceCodeVerifier,
}

pub(crate) struct RefreshRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) refresh_token: RefreshToken,
}

pub(crate) type TokenExchangeResult = Result<TokenResponse, TokenEndpointError>;

pub(crate) trait TokenEndpointTransport {
    fn authorization_code_exchange(&self, request: &CodeExchangeRequest) -> TokenExchangeResult;

    fn refresh_token_exchange(&self, request: &RefreshRequest) -> TokenExchangeResult;
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod transport_tests;
