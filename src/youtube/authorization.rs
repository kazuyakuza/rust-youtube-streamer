//! OAuth authorization helpers: Google endpoint constants, the required
//! YouTube scope, PKCE/CSRF-state generators, the authorization URL
//! builder and the callback query parser (plan §4.3). Pure functions —
//! no I/O; the CSRF state is validated before any callback value is kept.

use std::collections::HashMap;

use super::client_config::ClientConfig;
use super::error::AuthorizationError;
use super::token::AuthorizationCode;

/// The single requested YouTube API scope: the narrowest scope covering
/// the later broadcast lifecycle, stream ingestion and live-chat write
/// operations (plan §1.2 decision S3).
pub(crate) const REQUIRED_SCOPE: &str = "https://www.googleapis.com/auth/youtube.force-ssl";

/// Authorization endpoint Google uses for installed applications.
pub(crate) const AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
/// Token endpoint Google uses for installed applications.
pub(crate) const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
/// Revocation endpoint Google uses for installed applications.
// Reserved for a later phase (revocation is out of Task 3 scope; D13).
#[allow(dead_code)]
pub(crate) const REVOCATION_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

/// PKCE pair (S256): challenge for the authorization URL, verifier for
/// the code exchange.
pub(crate) struct PkcePair {
    /// Public half, sent with the authorization request.
    pub(crate) challenge: oauth2::PkceCodeChallenge,
    /// Secret half: held in memory for the code exchange only, never
    /// persisted or logged.
    pub(crate) verifier: oauth2::PkceCodeVerifier,
}

/// Generates a fresh S256 PKCE pair (RFC 7636 entropy via the oauth2 crate).
pub(crate) fn generate_pkce() -> PkcePair {
    let (challenge, verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    PkcePair {
        challenge,
        verifier,
    }
}

/// CSRF `state` value generator.
pub(crate) fn generate_state() -> oauth2::CsrfToken {
    oauth2::CsrfToken::new_random()
}

/// Builds the authorization URL the console displays. Pure: no I/O.
/// `redirect_uri` must be a loopback `http://127.0.0.1:<port>` value
/// obtained at callback-listener bind time — never `localhost` (plan
/// decision S1).
pub(crate) fn authorization_url(
    client_config: &ClientConfig,
    redirect_uri: &str,
    state: &oauth2::CsrfToken,
    pkce: &PkcePair,
) -> String {
    let query = oauth2::url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_config.client_id())
        .append_pair("response_type", "code")
        .append_pair("scope", REQUIRED_SCOPE)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("state", state.secret())
        .append_pair("code_challenge", pkce.challenge.as_str())
        .append_pair("code_challenge_method", "S256")
        .finish();
    format!("{}?{}", AUTHORIZATION_ENDPOINT, query)
}

/// One parsed authorization-endpoint redirect query.
#[derive(Debug)]
pub(crate) enum AuthorizationCallback {
    /// User approved; the state was validated before this variant is
    /// produced, and the code is single-use material.
    Authorized {
        code: AuthorizationCode,
        // The parser re-checks state equality internally; the auth flow
        // never re-reads it (D13).
        #[allow(dead_code)]
        state: String,
    },
    /// Provider error code, never a token.
    Declined { reason: String },
}

/// Parses a callback query string (e.g. `code=…&state=…` or `error=…`).
pub(crate) fn parse_authorization_callback(
    query: &str,
    expected_state: &oauth2::CsrfToken,
) -> Result<AuthorizationCallback, AuthorizationError> {
    let parameters = collect_parameters(query);
    if let Some(reason) = parameters.get("error") {
        return Ok(AuthorizationCallback::Declined {
            reason: reason.clone(),
        });
    }
    let state = required_parameter(&parameters, "state")?;
    if state != expected_state.secret().as_str() {
        return Err(AuthorizationError::StateMismatch);
    }
    let code = required_parameter(&parameters, "code")?;
    Ok(AuthorizationCallback::Authorized {
        code: AuthorizationCode::new(code),
        state,
    })
}

fn collect_parameters(query: &str) -> HashMap<String, String> {
    oauth2::url::form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn required_parameter(
    parameters: &HashMap<String, String>,
    name: &str,
) -> Result<String, AuthorizationError> {
    parameters
        .get(name)
        .cloned()
        .ok_or_else(|| AuthorizationError::MissingParameter {
            parameter: name.to_string(),
        })
}

#[cfg(test)]
#[path = "authorization_tests.rs"]
mod authorization_tests;
