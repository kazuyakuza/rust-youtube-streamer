//! Typed OAuth errors for the YouTube module: one enum per boundary
//! (client configuration, authorization callback, token endpoint,
//! credential persistence, authorization verification). Every message is
//! end-user-reportable and carries field paths, I/O causes, HTTP/OAuth
//! error codes or generic reasons only — never token values, client
//! secrets, PKCE verifiers or authorization codes (plan §5 redaction).

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Failures parsing Google's downloadable client-secret JSON file.
#[derive(Debug)]
pub enum ClientConfigError {
    MissingField { field: String },
    BlankField { field: String },
    EndpointMismatch { field: String },
    Deserialization { message: String },
}

/// Failures parsing the authorization-endpoint callback query.
#[derive(Debug)]
pub enum AuthorizationError {
    MissingParameter { parameter: String },
    StateMismatch,
    ProviderDeclined { reason: String },
}

/// Failures from the Google token endpoint exchange/refresh boundary.
#[derive(Debug)]
pub enum TokenEndpointError {
    Transport {
        reason: String,
    },
    Server {
        code: String,
        description: Option<String>,
    },
    ResponseParse,
}

/// Failures from the credential persistence boundary.
#[derive(Debug)]
pub enum CredentialStoreError {
    FileOpen { path: PathBuf, source: io::Error },
    FileWrite { path: PathBuf, source: io::Error },
    ReadParse { path: PathBuf, message: String },
    ContentNotAvailable,
}

/// Failures from the authorization verification boundary.
#[derive(Debug)]
pub enum VerificationError {
    NotAuthorized { reason: String },
    InsufficientScope { reason: String },
    Transport { reason: String },
}

impl fmt::Display for ClientConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientConfigError::MissingField { field } => {
                write!(formatter, "invalid OAuth client configuration: field '{field}' is required")
            }
            ClientConfigError::BlankField { field } => {
                write!(formatter, "invalid OAuth client configuration: field '{field}' must not be empty")
            }
            ClientConfigError::EndpointMismatch { field } => write!(
                formatter,
                "invalid OAuth client configuration: field '{field}' does not match the expected Google endpoint"
            ),
            ClientConfigError::Deserialization { message } => {
                write!(formatter, "invalid OAuth client configuration: {message}")
            }
        }
    }
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthorizationError::MissingParameter { parameter } => {
                write!(formatter, "authorization callback is missing the '{parameter}' parameter")
            }
            AuthorizationError::StateMismatch => write!(
                formatter,
                "authorization callback state did not match the expected value (possible cross-site request forgery)"
            ),
            AuthorizationError::ProviderDeclined { reason } => {
                write!(formatter, "the authorization request was declined (provider code '{reason}')")
            }
        }
    }
}

impl fmt::Display for TokenEndpointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenEndpointError::Transport { reason } => write!(
                formatter,
                "unable to reach the Google token endpoint: {reason}"
            ),
            TokenEndpointError::Server { code, description } => {
                write!(
                    formatter,
                    "the Google token endpoint rejected the request (code '{code}')"
                )?;
                write_optional_description(formatter, description)
            }
            TokenEndpointError::ResponseParse => write!(
                formatter,
                "the Google token endpoint returned an unreadable response"
            ),
        }
    }
}

fn write_optional_description(
    formatter: &mut fmt::Formatter<'_>,
    description: &Option<String>,
) -> fmt::Result {
    match description {
        Some(text) => write!(formatter, ": {text}"),
        None => Ok(()),
    }
}

impl fmt::Display for CredentialStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialStoreError::FileOpen { path, source } => write!(
                formatter,
                "unable to open the credential file '{}': {}",
                path.display(),
                source
            ),
            CredentialStoreError::FileWrite { path, source } => write!(
                formatter,
                "unable to write the credential file '{}': {}",
                path.display(),
                source
            ),
            CredentialStoreError::ReadParse { path, message } => write!(
                formatter,
                "unable to parse the credential file '{}': {}",
                path.display(),
                message
            ),
            CredentialStoreError::ContentNotAvailable => write!(
                formatter,
                "no stored credentials are available; run authorization first"
            ),
        }
    }
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::NotAuthorized { reason } => {
                write!(formatter, "the stored authorization is not valid: {reason}")
            }
            VerificationError::InsufficientScope { reason } => write!(
                formatter,
                "the stored authorization lacks the required YouTube scope: {reason}"
            ),
            VerificationError::Transport { reason } => write!(
                formatter,
                "unable to verify the stored authorization: {reason}"
            ),
        }
    }
}

impl Error for ClientConfigError {}
impl Error for AuthorizationError {}
impl Error for TokenEndpointError {}
impl Error for VerificationError {}

impl Error for CredentialStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CredentialStoreError::FileOpen { source, .. } => Some(source),
            CredentialStoreError::FileWrite { source, .. } => Some(source),
            CredentialStoreError::ReadParse { .. } => None,
            CredentialStoreError::ContentNotAvailable => None,
        }
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
