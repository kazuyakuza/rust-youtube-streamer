//! Google OAuth client configuration: parses the downloadable Desktop-app
//! client-secret JSON (`{"installed": {...}}`) into [`ClientConfig`].
//!
//! Deviation note (plan D2): this file is Google-authored, not
//! operator-authored, so unknown fields are tolerated instead of rejected
//! (`deny_unknown_fields` is intentionally absent) — Google has added
//! fields to this document for years. A declared endpoint must be one of the
//! accepted Google endpoint forms (legacy or v2; see the `authorization`
//! constants) or the file is rejected (tampered redirect guard).

use serde::Deserialize;

use super::authorization::{ACCEPTED_AUTHORIZATION_ENDPOINTS, ACCEPTED_TOKEN_ENDPOINTS};
use super::error::ClientConfigError;
use super::token::ClientSecret;

/// Client credentials parsed from Google's downloadable client-secret file.
/// `Debug` never reveals the secret (the wrapper redacts itself).
#[derive(Debug)]
pub(crate) struct ClientConfig {
    client_id: String,
    client_secret: ClientSecret,
}

impl ClientConfig {
    /// OAuth client identifier as issued by Google.
    pub(crate) fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Client secret wrapper; its `Debug` redacts the value.
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
    if !endpoint_is_recognized(&installed.auth_uri, &ACCEPTED_AUTHORIZATION_ENDPOINTS) {
        return Err(ClientConfigError::EndpointMismatch {
            field: "installed.auth_uri".to_string(),
        });
    }
    if !endpoint_is_recognized(&installed.token_uri, &ACCEPTED_TOKEN_ENDPOINTS) {
        return Err(ClientConfigError::EndpointMismatch {
            field: "installed.token_uri".to_string(),
        });
    }
    let client_id = required_non_blank(installed.client_id, "installed.client_id")?;
    let client_secret = required_non_blank(installed.client_secret, "installed.client_secret")?;
    Ok(ClientConfig {
        client_id,
        client_secret: ClientSecret::new(client_secret),
    })
}

fn endpoint_is_recognized(declared: &Option<String>, accepted: &[&str]) -> bool {
    declared
        .as_ref()
        .map_or(true, |actual| accepted.contains(&actual.as_str()))
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
