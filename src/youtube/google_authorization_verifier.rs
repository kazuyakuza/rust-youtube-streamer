//! The concrete `AuthorizationVerifier`: the minimal YouTube API probe for
//! the force-ssl scope — `GET /youtube/v3/channels?part=id&mine=true`
//! (plan §1.1 evidence; no broadcast/chat operations). Responses are
//! mapped status-first; bodies are consulted only for Google's
//! non-secret error `reason` code strings and never embedded in errors.

use serde::Deserialize;

use super::authorization::REQUIRED_SCOPE;
use super::error::VerificationError;
use super::token::{AccessToken, GrantedScopes};
use super::verify::{AuthorizationVerifier, VerifiedAuthorization};

/// The exact verification request URL: the minimal documented
/// channels.list(mine=true) call for the required scope (plan §1.1).
const VERIFICATION_ENDPOINT: &str =
    "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true";

/// Fixed 401 class text (D3); the observed reason code is appended when
/// the reply body carries one.
const REJECTED_TOKEN_TEXT: &str = "the Google server rejected the access token (HTTP 401)";

/// Fixed 403 class text (D3); same observed-reason append rule.
const DENIED_ACCESS_TEXT: &str = "the Google server denied access (HTTP 403)";

/// Fixed text for the unreadable-body Transport row (D3).
const UNREADABLE_BODY_TEXT: &str = "the verification response body could not be read";

pub(crate) struct GoogleAuthorizationVerifier {
    agent: ureq::Agent,
    request_url: String,
}

impl GoogleAuthorizationVerifier {
    /// Builds the verifier against the production endpoint.
    pub(crate) fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            request_url: VERIFICATION_ENDPOINT.to_string(),
        }
    }

    /// Test-only substitute URL (the `with_token_url` precedent); the
    /// production code never calls it.
    #[cfg(test)]
    pub(crate) fn with_request_url(request_url: String) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            request_url,
        }
    }

    /// Executes the Bearer request; maps transport-level failures (the
    /// D3 ureq-level row) and hands status+body to the classifier.
    fn executed_verification(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        let bearer_value = format!("Bearer {}", access_token.secret());
        let reply = self
            .agent
            .get(self.request_url.as_str())
            .set("Authorization", bearer_value.as_str())
            .call();
        self.reading_verification(reply)
    }

    /// Reads the HTTP reply body once and classifies through the ONE
    /// shared read-then-classify helper (no per-branch mapping logic).
    fn reading_verification(
        &self,
        reply: Result<ureq::Response, ureq::Error>,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        match reply {
            Ok(response) => classify_with_body_read(response.status(), response),
            Err(ureq::Error::Status(status, response)) => classify_with_body_read(status, response),
            Err(other) => Err(VerificationError::Transport {
                reason: other.to_string(),
            }),
        }
    }
}

impl AuthorizationVerifier for GoogleAuthorizationVerifier {
    fn verify(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        self.executed_verification(access_token)
    }
}

/// Reads the reply body (fixed-text Transport row when unreadable) and
/// delegates the (status, body) pair to the pure classifier.
fn classify_with_body_read(
    status: u16,
    response: ureq::Response,
) -> Result<VerifiedAuthorization, VerificationError> {
    match response.into_string() {
        Ok(body) => classify_verification_reply(status, Some(&body)),
        Err(_) => Err(VerificationError::Transport {
            reason: UNREADABLE_BODY_TEXT.to_string(),
        }),
    }
}

/// The mapping table (D3) as ONE pure function over status + body.
fn classify_verification_reply(
    status: u16,
    body: Option<&str>,
) -> Result<VerifiedAuthorization, VerificationError> {
    match status {
        200 => Ok(verified_scope_evidence()),
        401 => Err(VerificationError::NotAuthorized {
            reason: reason_with_observed_code(REJECTED_TOKEN_TEXT, body),
        }),
        403 => Err(VerificationError::InsufficientScope {
            reason: reason_with_observed_code(DENIED_ACCESS_TEXT, body),
        }),
        other => Err(VerificationError::Transport {
            reason: format!("the YouTube API server replied HTTP {other}"),
        }),
    }
}

/// The non-secret error `reason` code string of a Google API error body.
fn extracted_reason(body: Option<&str>) -> Option<String> {
    let document: ApiErrorDocument = serde_json::from_str(body?).ok()?;
    let detail = document.error?;
    detail.errors.into_iter().find_map(|item| item.reason)
}

/// Appends the extracted reason code to the fixed class text (D3 payload
/// column); the body itself is never included.
fn reason_with_observed_code(fixed_text: &str, body: Option<&str>) -> String {
    match extracted_reason(body) {
        Some(code) => format!("{fixed_text} (server reason '{code}')"),
        None => fixed_text.to_string(),
    }
}

/// The verified evidence for an accepted request (D3 row 200): the
/// scope set the server just confirmed effective for this token.
fn verified_scope_evidence() -> VerifiedAuthorization {
    VerifiedAuthorization::new(GrantedScopes::from_space_delimited(REQUIRED_SCOPE))
}

/// The typed serde view of the documented Google error shape; serde
/// defaults everywhere (absent fields tolerated).
#[derive(Deserialize)]
struct ApiErrorDocument {
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    #[serde(default)]
    errors: Vec<ApiErrorItem>,
}

#[derive(Deserialize)]
struct ApiErrorItem {
    #[serde(default)]
    reason: Option<String>,
}

#[cfg(test)]
#[path = "google_authorization_verifier_tests.rs"]
mod google_authorization_verifier_tests;
