//! Redacted OAuth token material for the YouTube module: secret wrapper
//! types whose `Debug` prints `<redacted>` (never `Display`), the
//! serde-parsed Google token endpoint body, the application-owned
//! [`TokenResponse`], and the granted-scope collection (plan §5).
//!
//! Raw secret strings exist only inside the wrappers; `secret()`
//! accessors are consumed only at request-build sites (transport today,
//! the Bearer header with Tasks 3–4), never formatted or logged.

use std::fmt;

use serde::Deserialize;

use super::error::TokenEndpointError;

/// Access token for Google API requests. Debug never reveals the value;
/// Display is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct AccessToken {
    secret: String,
}

impl AccessToken {
    /// Wraps a raw access-token string; the wrapper owns the secret.
    pub(crate) fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Raw value; consumed only when building API requests, never
    /// logged or formatted.
    pub(crate) fn secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessToken(<redacted>)")
    }
}

/// Refresh token for long-term credential reuse. Debug never reveals
/// the value; Display is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct RefreshToken {
    secret: String,
}

impl RefreshToken {
    /// Wraps a raw refresh-token string; the wrapper owns the secret.
    pub(crate) fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Raw value; consumed only when building token refresh requests,
    /// never logged or formatted.
    pub(crate) fn secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for RefreshToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RefreshToken(<redacted>)")
    }
}

/// Single-use authorization code returned by the callback. Debug never
/// reveals the value; Display is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct AuthorizationCode {
    secret: String,
}

impl AuthorizationCode {
    /// Wraps a raw authorization-code string; the wrapper owns the secret.
    pub(crate) fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Raw value; consumed only when exchanging the code for tokens,
    /// never logged or formatted.
    pub(crate) fn secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for AuthorizationCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AuthorizationCode(<redacted>)")
    }
}

/// OAuth client secret from Google's downloadable client file. Debug
/// never reveals the value; Display is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct ClientSecret {
    secret: String,
}

impl ClientSecret {
    /// Wraps a raw client-secret string; the wrapper owns the secret.
    pub(crate) fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Raw value; consumed only when building token endpoint requests,
    /// never logged or formatted.
    pub(crate) fn secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for ClientSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClientSecret(<redacted>)")
    }
}

/// Google token endpoint response fields we recognize; unknown fields
/// (e.g. `id_token`) are ignored per Google's response guidance.
/// The raw-string secret fields intentionally make `Debug` unavailable:
/// accidental formatting in a future call site would print tokens; a
/// compile error is the stronger guard (Task 2 hardening, simplify §4 flag).
// The serde parser is the offline/Task-4 seam; the ureq path parses via
// oauth2's BasicTokenResponse (D13).
#[allow(dead_code)]
#[derive(Deserialize)]
pub(crate) struct TokenEndpointBody {
    access_token: String,
    expires_in: u64,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
}

/// Application-owned, redaction-safe token data built from a parsed
/// token endpoint body. `Debug` never reveals secrets: the token wrappers
/// redact themselves (plan §5).
#[derive(Debug)]
pub(crate) struct TokenResponse {
    pub(crate) access_token: AccessToken,
    /// Remaining lifetime of `access_token` in seconds, as reported by Google.
    pub(crate) expires_in_seconds: u64,
    pub(crate) refresh_token: Option<RefreshToken>,
    pub(crate) granted_scopes: GrantedScopes,
}

/// Ordered collection of granted scope URL strings (case-sensitive).
#[derive(Clone, Debug)]
pub(crate) struct GrantedScopes {
    scopes: Vec<String>,
}

impl GrantedScopes {
    /// Parses Google's space-delimited `scope` field value.
    pub(crate) fn from_space_delimited(value: &str) -> Self {
        Self {
            scopes: value.split_whitespace().map(str::to_string).collect(),
        }
    }

    /// Exact-match membership (Google scopes are case-sensitive URLs).
    pub(crate) fn grants(&self, scope: &str) -> bool {
        self.scopes.iter().any(|granted| granted == scope)
    }

    /// Serializes back to Google's space-delimited scope format; pairs with
    /// `from_space_delimited` as the credential store's persistence round trip.
    pub(crate) fn to_space_delimited(&self) -> String {
        self.scopes.join(" ")
    }
}

/// Parses a Google token endpoint response body; a missing
/// `access_token` or unparsable JSON maps to
/// [`TokenEndpointError::ResponseParse`].
// Offline/Task-4 seam; the ureq path parses via oauth2's token types (D13).
#[allow(dead_code)]
pub(crate) fn parse_token_response(body: &str) -> Result<TokenResponse, TokenEndpointError> {
    let parsed: TokenEndpointBody =
        serde_json::from_str(body).map_err(|_| TokenEndpointError::ResponseParse)?;
    let scope_text = parsed.scope.unwrap_or_default();
    Ok(TokenResponse {
        access_token: AccessToken::new(parsed.access_token),
        expires_in_seconds: parsed.expires_in,
        refresh_token: parsed.refresh_token.map(RefreshToken::new),
        granted_scopes: GrantedScopes::from_space_delimited(&scope_text),
    })
}

#[cfg(test)]
#[path = "token_tests.rs"]
mod token_tests;
