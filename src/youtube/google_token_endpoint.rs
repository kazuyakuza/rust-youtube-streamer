//! ureq-backed Google token endpoint implementation of
//! [`TokenEndpointTransport`]: the production endpoint is fixed at
//! construction, redirects are disabled (SSRF), and `with_token_url`
//! substitutes an endpoint for offline tests. Response bodies never
//! reach error variants.

use std::{error::Error, fmt};

use oauth2::{
    basic::{BasicClient, BasicErrorResponse, BasicTokenResponse, BasicTokenType},
    AuthUrl, AuthorizationCode as Oauth2AuthorizationCode, ClientId,
    ClientSecret as Oauth2ClientSecret, RedirectUrl, RefreshToken as Oauth2RefreshToken,
    RequestTokenError, TokenResponse as _,
};

use super::authorization::{AUTHORIZATION_ENDPOINT, TOKEN_ENDPOINT};
use super::error::TokenEndpointError;
use super::token::{AccessToken, ClientSecret, GrantedScopes, RefreshToken, TokenResponse};
use super::transport::{
    CodeExchangeRequest, RefreshRequest, TokenEndpointTransport, TokenExchangeResult,
};

/// ureq-backed Google token endpoint client, fixed to the production
/// endpoint unless a test substitutes one via `with_token_url`.
pub(crate) struct GoogleTokenEndpoint {
    agent: ureq::Agent,
    token_url: oauth2::TokenUrl,
}

type TokenHttpResult = Result<oauth2::HttpResponse, TokenRequestFailure>;

impl GoogleTokenEndpoint {
    pub(crate) fn new() -> Result<Self, TokenEndpointError> {
        let token_url = oauth2::TokenUrl::new(TOKEN_ENDPOINT.to_string()).map_err(|_| {
            TokenEndpointError::Transport {
                reason: "invalid Google token endpoint URL".to_string(),
            }
        })?;
        Ok(Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            token_url,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_token_url(token_url: oauth2::TokenUrl) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            token_url,
        }
    }

    fn execute_http(&self, request: oauth2::HttpRequest) -> TokenHttpResult {
        let mut ureq_request = self.agent.post(request.uri().to_string().as_str());
        for (name, value) in request.headers() {
            let header_text = value.to_str().map_err(|_| bridge_error("invalid header"))?;
            ureq_request = ureq_request.set(name.as_str(), header_text);
        }
        match ureq_request.send_bytes(request.body()) {
            Ok(response) => map_ureq_response(response),
            Err(ureq::Error::Status(_, response)) => map_ureq_response(response),
            Err(error) => Err(bridge_error(error)),
        }
    }
}

impl TokenEndpointTransport for GoogleTokenEndpoint {
    fn authorization_code_exchange(&self, request: &CodeExchangeRequest) -> TokenExchangeResult {
        let client = BasicClient::new(ClientId::new(request.client_id.clone()))
            .set_client_secret(oauth2_client_secret(&request.client_secret))
            .set_auth_uri(production_auth_url()?)
            .set_token_uri(self.token_url.clone());
        let code = Oauth2AuthorizationCode::new(request.code.secret().to_string());
        let verifier = oauth2::PkceCodeVerifier::new(request.pkce_verifier.secret().to_string());
        let redirect_url = RedirectUrl::new(request.redirect_uri.clone()).map_err(|_| {
            TokenEndpointError::Transport {
                reason: "invalid redirect URI".to_string(),
            }
        })?;
        let http_client = |http_request: oauth2::HttpRequest| self.execute_http(http_request);
        let response = client
            .exchange_code(code)
            .set_pkce_verifier(verifier)
            .set_redirect_uri(std::borrow::Cow::Owned(redirect_url))
            .request(&http_client)
            .map_err(map_exchange_error)?;
        to_token_response(response)
    }

    fn refresh_token_exchange(&self, request: &RefreshRequest) -> TokenExchangeResult {
        let client = BasicClient::new(ClientId::new(request.client_id.clone()))
            .set_client_secret(oauth2_client_secret(&request.client_secret))
            .set_auth_uri(production_auth_url()?)
            .set_token_uri(self.token_url.clone());
        let http_client = |http_request: oauth2::HttpRequest| self.execute_http(http_request);
        let response = client
            .exchange_refresh_token(&Oauth2RefreshToken::new(
                request.refresh_token.secret().to_string(),
            ))
            .request(&http_client)
            .map_err(map_exchange_error)?;
        to_token_response(response)
    }
}

/// Bridge failure for the oauth2 `SyncHttpClient` closure; carries only a
/// secret-free reason string.
#[derive(Debug)]
struct TokenRequestFailure(String);

impl fmt::Display for TokenRequestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "token endpoint HTTP failure: {}", self.0)
    }
}

impl Error for TokenRequestFailure {}

fn production_auth_url() -> Result<AuthUrl, TokenEndpointError> {
    AuthUrl::new(AUTHORIZATION_ENDPOINT.to_string()).map_err(|_| TokenEndpointError::Transport {
        reason: "invalid Google authorization endpoint URL".to_string(),
    })
}

fn oauth2_client_secret(secret: &ClientSecret) -> Oauth2ClientSecret {
    Oauth2ClientSecret::new(secret.secret().to_string())
}

fn bridge_error(error: impl ToString) -> TokenRequestFailure {
    TokenRequestFailure(error.to_string())
}

fn map_ureq_response(response: ureq::Response) -> TokenHttpResult {
    let status_code = response.status();
    let body = response.into_string().map_err(bridge_error)?.into_bytes();
    oauth2::http::Response::<()>::builder()
        .status(status_code)
        .body(body)
        .map_err(bridge_error)
}

fn map_exchange_error(
    error: RequestTokenError<TokenRequestFailure, BasicErrorResponse>,
) -> TokenEndpointError {
    match error {
        RequestTokenError::ServerResponse(error) => TokenEndpointError::Server {
            code: error.error().to_string(),
            description: error.error_description().map(|text| text.to_string()),
        },
        RequestTokenError::Request(failure) => TokenEndpointError::Transport { reason: failure.0 },
        RequestTokenError::Parse(_, _) => TokenEndpointError::ResponseParse,
        RequestTokenError::Other(message) => TokenEndpointError::Transport { reason: message },
    }
}

fn join_scopes(scopes: &[oauth2::Scope]) -> String {
    let parts = scopes
        .iter()
        .map(|scope| scope.as_ref())
        .collect::<Vec<_>>();
    parts.join(" ")
}

fn to_token_response(response: BasicTokenResponse) -> Result<TokenResponse, TokenEndpointError> {
    if *response.token_type() != BasicTokenType::Bearer {
        return Err(TokenEndpointError::ResponseParse);
    }
    let scopes = response.scopes();
    let scopes_text = scopes.map(|s| join_scopes(s)).unwrap_or_default();
    let expires_in = response.expires_in();
    let expires_in_seconds = expires_in.map_or(0, |duration| duration.as_secs());
    Ok(TokenResponse {
        access_token: AccessToken::new(response.access_token().secret().to_string()),
        expires_in_seconds,
        refresh_token: response
            .refresh_token()
            .map(|token| RefreshToken::new(token.secret().to_string())),
        granted_scopes: GrantedScopes::from_space_delimited(&scopes_text),
    })
}

#[cfg(test)]
#[path = "google_token_endpoint_tests.rs"]
mod google_token_endpoint_tests;
