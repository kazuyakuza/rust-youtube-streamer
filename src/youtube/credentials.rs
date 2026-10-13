//! Credential persistence seam: the application-owned stored credential
//! type and the mockable [`CredentialStore`] boundary. The concrete file
//! store lives in `file_credential_store` (atomic temp-file + rename
//! writes, POSIX 0600) with its on-disk JSON schema in
//! `persisted_credentials`; this module owns types and the trait only.

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

/// Credential persistence boundary; test fakes implement it and the
/// concrete `FileCredentialStore` writes the on-disk JSON credential file.
pub(crate) trait CredentialStore {
    /// Reads the previously persisted credentials.
    // Load-side consumers arrive with the Task 4 credential service (D13).
    #[allow(dead_code)]
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError>;

    /// Persists credentials atomically; the store's temp-file + rename
    /// sequence and platform limits are documented in `file_credential_store`.
    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError>;
}

#[cfg(test)]
#[path = "credentials_tests.rs"]
mod credentials_tests;
