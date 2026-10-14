//! Authorization verification seam: the result type and the mockable
//! [`AuthorizationVerifier`] boundary. The production implementation is
//! `google_authorization_verifier` (the read-only channels.list(mine=true)
//! probe for the force-ssl scope); the credential service consumes this
//! trait and its scenario tests drive it with scripted offline fakes.

use super::error::VerificationError;
use super::token::{AccessToken, GrantedScopes};

/// Evidence that the effective authorization carries the required scope.
pub(crate) struct VerifiedAuthorization {
    granted_scopes: GrantedScopes,
}

impl VerifiedAuthorization {
    /// Wraps the scopes observed while verifying an effective authorization.
    pub(crate) fn new(granted_scopes: GrantedScopes) -> Self {
        Self { granted_scopes }
    }

    /// The granted-scope evidence, for exact-membership checks against
    /// [`super::authorization::REQUIRED_SCOPE`] (the run-side permission
    /// gate consumes it before any resource creation).
    pub(crate) fn granted_scopes(&self) -> &GrantedScopes {
        &self.granted_scopes
    }
}

/// Authorization verification boundary; fakes implement it in tests.
pub(crate) trait AuthorizationVerifier {
    /// Confirms the effective authorization carries `REQUIRED_SCOPE`.
    fn verify(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError>;
}
