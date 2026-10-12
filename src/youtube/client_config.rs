//! Google OAuth client configuration: parses the downloadable Desktop-app
//! client-secret JSON (`{"installed": {...}}`) into [`ClientConfig`].
//!
//! Deviation note (plan D2): this file is Google-authored, not
//! operator-authored, so unknown fields are tolerated instead of rejected
//! (`deny_unknown_fields` is intentionally absent) — Google has added
//! fields to this document for years. Declared endpoints must equal the
//! Google constants or the file is rejected (tampered redirect guard).

use std::fmt;

use serde::Deserialize;

use super::error::ClientConfigError;

/// Authorization endpoint Google uses for installed applications.
pub(crate) const AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
/// Token endpoint Google uses for installed applications.
pub(crate) const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
/// Revocation endpoint Google uses for installed applications.
pub(crate) const REVOCATION_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

/// OAuth client secret. `Debug` output never reveals the value and
/// `Display` is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct ClientSecret {
    secret: String,
}

impl ClientSecret {
    pub(crate) fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Raw value; consumed only when building token endpoint requests and
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

/// Client credentials parsed from Google's downloadable client-secret file.
/// `Debug` never reveals the secret (the wrapper redacts itself).
#[derive(Debug)]
pub(crate) struct ClientConfig {
    client_id: String,
    client_secret: ClientSecret,
}

impl ClientConfig {
    pub(crate) fn client_id(&self) -> &str {
        &self.client_id
    }

    pub(crate) fn client_secret(&self) -> &ClientSecret {
        &self.client_secret
    }
}

#[derive(Debug, Deserialize)]
struct ClientSecretFile {
    installed: Option<InstalledClient>,
}

#[derive(Debug, Deserialize)]
struct InstalledClient {
    client_id: Option<String>,
    client_secret: Option<String>,
    auth_uri: Option<String>,
    token_uri: Option<String>,
}

/// Parses Google's downloadable client-secret document.
pub(crate) fn parse_client_config(json: &str) -> Result<ClientConfig, ClientConfigError> {
    let file: ClientSecretFile =
        serde_json::from_str(json).map_err(|error| ClientConfigError::Deserialization {
            message: error.to_string(),
        })?;
    let installed = file
        .installed
        .ok_or_else(|| missing_field("installed.client_id"))?;
    validate_endpoint(
        &installed.auth_uri,
        AUTHORIZATION_ENDPOINT,
        "installed.auth_uri",
    )?;
    validate_endpoint(&installed.token_uri, TOKEN_ENDPOINT, "installed.token_uri")?;
    let client_id = required_non_blank(installed.client_id, "installed.client_id")?;
    let client_secret = required_non_blank(installed.client_secret, "installed.client_secret")?;
    Ok(ClientConfig {
        client_id,
        client_secret: ClientSecret::new(client_secret),
    })
}

fn validate_endpoint(
    declared: &Option<String>,
    expected: &str,
    field: &str,
) -> Result<(), ClientConfigError> {
    match declared {
        Some(actual) if actual != expected => Err(ClientConfigError::EndpointMismatch {
            field: field.to_string(),
        }),
        _ => Ok(()),
    }
}

fn required_non_blank(value: Option<String>, field: &str) -> Result<String, ClientConfigError> {
    match value {
        None => Err(missing_field(field)),
        Some(text) if text.trim().is_empty() => Err(ClientConfigError::BlankField {
            field: field.to_string(),
        }),
        Some(text) => Ok(text),
    }
}

fn missing_field(field: &str) -> ClientConfigError {
    ClientConfigError::MissingField {
        field: field.to_string(),
    }
}

#[cfg(test)]
#[path = "client_config_tests.rs"]
mod client_config_tests;
