//! The reusable credential service for the future production startup
//! (deliberately NOT wired into the `run` placeholder; a later run phase
//! owns the wiring). Pipeline: load stored credentials → offline gate on
//! the persisted scope evidence → verify with the read-only
//! channels.list(mine=true) probe → refresh EXACTLY once on a rejected
//! token → merge (an omitted response refresh token keeps the stored one)
//! → atomic save → one re-verify. Verify-first (T4-D1): TTL math cannot
//! detect revocation, no clock is read, the schema is frozen —
//! `expires_in_seconds` is advisory only. Tokens NEVER reach errors,
//! logs, `Display` or `Debug`; probe bodies never surface (statuses +
//! non-secret reason codes). Initial authorization is interactive
//! (`auth`); unattended runs rely on stored refresh credentials + policy.

use tracing::info;

use super::authorization::REQUIRED_SCOPE;
use super::client_config::ClientConfig;
use super::credential_service_error::CredentialServiceError;
use super::credentials::{CredentialStore, StoredCredentials};
use super::error::{CredentialStoreError, TokenEndpointError, VerificationError};
use super::token::{AccessToken, GrantedScopes, RefreshToken, TokenResponse};
use super::transport::{RefreshRequest, TokenEndpointTransport};
use super::verify::{AuthorizationVerifier, VerifiedAuthorization};

/// The verified-authorization outcome: proven-usable token + evidence (D6).
pub(crate) struct VerifiedCredentials {
    /// Proven usable by the final verification; consumed by future run wiring.
    pub(crate) access_token: AccessToken,
    /// Seam-verifiable scope evidence of the verified authorization.
    pub(crate) authorization: VerifiedAuthorization,
}

/// The service seams (D6): store, endpoint, verifier, client_config; struct literal only.
pub(crate) struct CredentialService<'a> {
    pub(crate) store: &'a dyn CredentialStore,
    pub(crate) endpoint: &'a dyn TokenEndpointTransport,
    pub(crate) verifier: &'a dyn AuthorizationVerifier,
    pub(crate) client_config: &'a ClientConfig,
}

impl CredentialService<'_> {
    /// The one entry operation (D1): load, gate, verify, refresh-once.
    pub(crate) fn establish_authorization(
        &self,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        let stored = self.load_verified_scope_evidence()?;
        self.verify_loaded(&stored)
    }

    /// Load + map store errors (D5) + the offline gate: credentials granting REQUIRED_SCOPE.
    fn load_verified_scope_evidence(&self) -> Result<StoredCredentials, CredentialServiceError> {
        let stored = self.store.load().map_err(mapped_load_failure)?;
        if !grants_required_scope(&stored) {
            return Err(CredentialServiceError::Verification(
                VerificationError::InsufficientScope {
                    reason: "the persisted granted scopes do not include it".to_string(),
                },
            ));
        }
        info!("stored credentials loaded");
        Ok(stored)
    }

    /// First verification; routes 401 into the refresh path (D3/D4).
    fn verify_loaded(
        &self,
        stored: &StoredCredentials,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        match self.verifier.verify(&stored.access_token) {
            Ok(authorization) => self.report_with_verification_evidence(stored, authorization),
            Err(VerificationError::NotAuthorized { .. }) => self.refresh_and_reverify(stored),
            Err(other) => Err(CredentialServiceError::Verification(other)),
        }
    }

    /// The refresh path: require a stored refresh token, exchange, merge, save, re-verify.
    fn refresh_and_reverify(
        &self,
        stored: &StoredCredentials,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        let refresh_token = stored
            .refresh_token
            .as_ref()
            .ok_or(CredentialServiceError::Unrefreshable)?;
        let request = refresh_request(self.client_config, refresh_token);
        let response = self
            .endpoint
            .refresh_token_exchange(&request)
            .map_err(mapped_refresh_failure)?;
        let refreshed = merge_refresh_response(stored, response);
        self.store
            .save(&refreshed)
            .map_err(CredentialServiceError::Persistence)?;
        info!("access token refreshed and persisted");
        self.verify_latest(&refreshed)
    }

    /// Single re-verification with the fresh token (D4-step 7: no loops).
    fn verify_latest(
        &self,
        refreshed: &StoredCredentials,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        match self.verifier.verify(&refreshed.access_token) {
            Ok(authorization) => self.report_with_verification_evidence(refreshed, authorization),
            Err(error) => Err(CredentialServiceError::Verification(error)),
        }
    }

    /// Shared success construction for BOTH Ok arms; logs once.
    fn report_with_verification_evidence(
        &self,
        stored: &StoredCredentials,
        authorization: VerifiedAuthorization,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        info!("stored authorization verified");
        Ok(VerifiedCredentials {
            access_token: stored.access_token.clone(),
            authorization,
        })
    }
}

fn mapped_load_failure(error: CredentialStoreError) -> CredentialServiceError {
    // `FileWrite` cannot be produced by `load`; folded into the readable class (D5).
    match error {
        CredentialStoreError::ContentNotAvailable => CredentialServiceError::Absent,
        other => CredentialServiceError::Unreadable(other),
    }
}

fn mapped_refresh_failure(error: TokenEndpointError) -> CredentialServiceError {
    if is_revoked_refresh(&error) {
        return revoked_from(error);
    }
    CredentialServiceError::Refresh(error)
}

/// Single-section predicate for the invalid_grant exact match (D4-4).
fn is_revoked_refresh(error: &TokenEndpointError) -> bool {
    matches!(error, TokenEndpointError::Server { code, .. } if code == "invalid_grant")
}

/// Shape-matches `Server { code }` to `Revoked`; folds other shapes into Refresh (panic-free).
fn revoked_from(error: TokenEndpointError) -> CredentialServiceError {
    match error {
        TokenEndpointError::Server { code, .. } => CredentialServiceError::Revoked { code },
        other => CredentialServiceError::Refresh(other),
    }
}

fn grants_required_scope(stored: &StoredCredentials) -> bool {
    stored.granted_scopes.grants(REQUIRED_SCOPE)
}

fn response_scopes_replace(response_scopes: &GrantedScopes) -> bool {
    !response_scopes.to_space_delimited().is_empty()
}

/// Merges a refresh response over the stored credentials (D4-5): the response
/// token + TTL always move; omitted response fields keep the stored values.
fn merge_refresh_response(
    stored: &StoredCredentials,
    response: TokenResponse,
) -> StoredCredentials {
    let refresh_token = response
        .refresh_token
        .or_else(|| stored.refresh_token.clone());
    let granted_scopes = if response_scopes_replace(&response.granted_scopes) {
        response.granted_scopes
    } else {
        stored.granted_scopes.clone()
    };
    StoredCredentials {
        access_token: response.access_token,
        expires_in_seconds: response.expires_in_seconds,
        refresh_token,
        granted_scopes,
    }
}

fn refresh_request(client_config: &ClientConfig, refresh_token: &RefreshToken) -> RefreshRequest {
    RefreshRequest {
        client_id: client_config.client_id().to_string(),
        client_secret: client_config.client_secret().clone(),
        refresh_token: refresh_token.clone(),
    }
}

#[cfg(test)]
#[path = "credential_service_tests.rs"]
mod credential_service_tests;

#[cfg(test)]
#[path = "credential_service_refresh_tests.rs"]
mod credential_service_refresh_tests;

#[cfg(test)]
#[path = "credential_service_verification_tests.rs"]
mod credential_service_verification_tests;
