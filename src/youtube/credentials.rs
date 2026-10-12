//! Credential persistence seam: the application-owned stored credential
//! type and the mockable [`CredentialStore`] boundary. The concrete file
//! store (atomic writes, permission restrictions) is Task 2's work
//! (plan decision S8); this module owns types and the trait only.

use super::error::CredentialStoreError;
use super::token::{AccessToken, GrantedScopes, RefreshToken};

/// Redaction-safe credential material as persisted between runs.
/// `Debug` never reveals token values (the wrappers redact themselves).
#[derive(Debug)]
pub(crate) struct StoredCredentials {
    pub(crate) access_token: AccessToken,
    /// Remaining lifetime of `access_token` in seconds at save time;
    /// clock use belongs to the credential service (Task 4).
    pub(crate) expires_in_seconds: u64,
    pub(crate) refresh_token: Option<RefreshToken>,
    pub(crate) granted_scopes: GrantedScopes,
}

/// Credential persistence boundary; fakes implement it in tests.
pub(crate) trait CredentialStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError>;

    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError>;
}

#[cfg(test)]
#[path = "credentials_tests.rs"]
mod credentials_tests;
