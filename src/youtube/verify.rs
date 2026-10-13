//! Authorization verification seam: the result type and the mockable
//! [`AuthorizationVerifier`] boundary. The concrete minimal YouTube API
//! check that fits the force-ssl scope is Task 4's decision (plan
//! decision S9); Task 1 fixes only the seam so the credential service
//! stays mockable.

use super::error::VerificationError;
use super::token::{AccessToken, GrantedScopes};

/// Evidence that the effective authorization carries the required scope.
pub(crate) struct VerifiedAuthorization {
    // Staged for the future run-phase wiring (the scope-evidence reader).
    #[allow(dead_code)]
    granted_scopes: GrantedScopes,
}

impl VerifiedAuthorization {
    /// Wraps the scopes observed while verifying an effective authorization.
    pub(crate) fn new(granted_scopes: GrantedScopes) -> Self {
        Self { granted_scopes }
    }

    /// The granted-scope evidence, for exact-membership checks against
    /// [`super::authorization::REQUIRED_SCOPE`].
    // Staged for the future run-phase wiring (the scope-evidence reader).
    #[allow(dead_code)]
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
