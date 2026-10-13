//! On-disk credential-file schema: the private `PersistedCredentials`
//! document and its conversions to/from [`StoredCredentials`]
//! (space-delimited `granted_scopes` round trip per plan §4.6 and
//! amendment-1's `GrantedScopes::to_space_delimited`). Serialization and
//! parse mapping live here; the store module owns only file IO
//! (temp-file creation, permissions, rename) and io-error mapping.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::credentials::StoredCredentials;
use super::error::CredentialStoreError;
use super::token::{AccessToken, GrantedScopes, RefreshToken};

/// JSON document shape of the persisted credential file. Secret VALUES
/// are the file's legitimate content (the file itself is the vault),
/// mirrored one-to-one from StoredCredentials; this type exists so
/// stored credentials serialize with fixed field names and never via a
/// derived Debug/serde path that could drift from the documented schema.
#[derive(Deserialize, Serialize)]
struct PersistedCredentials {
    access_token: String,
    /// Remaining lifetime of `access_token` in seconds at save time
    /// (TTL semantics preserved from Task 1: no absolute timestamps are
    /// persisted; clock use belongs to the Task 4 credential service).
    expires_in_seconds: u64,
    /// Serialized as `null` when absent (Option semantics, no skip).
    refresh_token: Option<String>,
    /// Google's space-delimited scope list, exactly as the token
    /// endpoint reports it (empty string when no scope was granted).
    granted_scopes: String,
}

impl PersistedCredentials {
    fn from_stored(credentials: &StoredCredentials) -> Self {
        Self {
            access_token: credentials.access_token.secret().to_string(),
            expires_in_seconds: credentials.expires_in_seconds,
            refresh_token: credentials
                .refresh_token
                .as_ref()
                .map(|token| token.secret().to_string()),
            granted_scopes: credentials.granted_scopes.to_space_delimited(),
        }
    }
}

/// Serializes stored credentials to the persisted JSON document text.
pub(crate) fn serialized_document(credentials: &StoredCredentials) -> Result<String, io::Error> {
    serde_json::to_string(&PersistedCredentials::from_stored(credentials))
        .map_err(|_| serialization_failure())
}

/// Parses persisted file text into stored credentials; malformed or
/// wrong-schema JSON maps to ReadParse with the fixed redacted message.
pub(crate) fn parse_credentials(
    text: &str,
    path: &Path,
) -> Result<StoredCredentials, CredentialStoreError> {
    let parsed = serde_json::from_str::<PersistedCredentials>(text).map_err(|error| {
        CredentialStoreError::ReadParse {
            path: path.to_path_buf(),
            message: parse_failure_message(&error),
        }
    })?;
    Ok(stored_from_persisted(parsed))
}

fn stored_from_persisted(parsed: PersistedCredentials) -> StoredCredentials {
    StoredCredentials {
        access_token: AccessToken::new(parsed.access_token),
        expires_in_seconds: parsed.expires_in_seconds,
        refresh_token: parsed.refresh_token.map(RefreshToken::new),
        granted_scopes: GrantedScopes::from_space_delimited(&parsed.granted_scopes),
    }
}

fn parse_failure_message(error: &serde_json::Error) -> String {
    if is_malformed_json(error) {
        "the credential file is not valid JSON".to_string()
    } else {
        "the credential file does not match the expected credential schema".to_string()
    }
}

fn is_malformed_json(error: &serde_json::Error) -> bool {
    error.is_syntax() || error.is_eof()
}

fn serialization_failure() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "credential serialization failed",
    )
}

#[cfg(test)]
#[path = "persisted_credentials_tests.rs"]
mod persisted_credentials_tests;
