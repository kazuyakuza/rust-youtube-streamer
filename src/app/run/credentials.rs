//! The credentials/authorization stage of the run runtime (plan D-T3-2
//! stages 2–3, A3): client file read+parse FIRST (the auth-flow D5
//! ordering), the concrete Phase-04 stops, the credential service
//! consumed VERBATIM (D-G4: no OAuth duplication, no second credential
//! format), and the run-side permission gate that consumes the FRESH
//! scope evidence BEFORE any YouTube resource is created.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AppConfig;
use crate::youtube::{
    parse_client_config, ClientConfig, ClientConfigError, CredentialService, FileCredentialStore,
    GoogleAuthorizationVerifier, GoogleTokenEndpoint, VerifiedCredentials, REQUIRED_SCOPE,
};

use super::error::RunError;

/// The concrete Phase-04 stops, owned so `run_outcome` can hand borrows
/// of them to the injected-seams runtime (the borrow-legitimate shape of
/// D-T3-2 stage 2's "construct the concrete stops").
pub(super) struct ProductionCredentials {
    pub(super) client_config: ClientConfig,
    pub(super) store: FileCredentialStore,
    pub(super) endpoint: GoogleTokenEndpoint,
    pub(super) verifier: GoogleAuthorizationVerifier,
}

/// D-T3-2 stage 2: read+parse the configured OAuth client file FIRST,
/// then construct the concrete stops (`FileCredentialStore`,
/// `GoogleTokenEndpoint`, `GoogleAuthorizationVerifier`, the parsed
/// `ClientConfig`).
pub(super) fn establish_credentials(
    app_config: &AppConfig,
) -> Result<ProductionCredentials, RunError> {
    Ok(ProductionCredentials {
        client_config: read_client_config(app_config)?,
        store: FileCredentialStore::new(PathBuf::from(&app_config.oauth.credentials_path)),
        endpoint: GoogleTokenEndpoint::new().map_err(RunError::EndpointInit)?,
        verifier: GoogleAuthorizationVerifier::new(),
    })
}

/// Reads + parses the configured client-secret file. Private duplicate of
/// `auth/flow.rs`'s helper (the duplication-over-cross-module-contract
/// precedent): the auth error type is never widened for the run path.
fn read_client_config(app_config: &AppConfig) -> Result<ClientConfig, RunError> {
    let path_text = &app_config.oauth.client_config_path;
    let text = fs::read_to_string(Path::new(path_text)).map_err(unreadable_client_file)?;
    parse_client_config(&text).map_err(RunError::ClientConfig)
}

fn unreadable_client_file(source: std::io::Error) -> RunError {
    RunError::ClientConfig(ClientConfigError::Deserialization {
        message: format!("the configured client file could not be read: {source}"),
    })
}

/// The Phase-04 service consumed VERBATIM: load → offline persisted-scope
/// gate → verify-first probe → refresh exactly once → atomic persist →
/// one re-verify.
pub(super) fn authorize_from_service(
    service: &CredentialService<'_>,
) -> Result<VerifiedCredentials, RunError> {
    service
        .establish_authorization()
        .map_err(RunError::Credential)
}

/// D-T3-2 stage 3: the run-side permission gate BEFORE any resource
/// creation — the FRESH verification evidence must grant REQUIRED_SCOPE
/// (exact membership; the service's own offline gate covered the
/// PERSISTED scopes). Failure → `RunError::InsufficientScope` (exit 1).
pub(super) fn authorize_scope_evidence(verified: &VerifiedCredentials) -> Result<(), RunError> {
    if verified
        .authorization
        .granted_scopes()
        .grants(REQUIRED_SCOPE)
    {
        return Ok(());
    }
    Err(RunError::InsufficientScope)
}
