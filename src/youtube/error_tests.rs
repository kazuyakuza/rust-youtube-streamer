//! Display-contract and redaction tests for the YouTube OAuth error enums:
//! every variant renders its exact stable message, and only the variants
//! wrapping an I/O cause chain it through `source()`.

use std::io;
use std::path::PathBuf;

use super::*;

fn permission_denied() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "permission denied")
}

#[test]
fn each_variant_has_a_stable_display_line() {
    assert_eq!(
        ClientConfigError::MissingField {
            field: "installed.client_id".to_string()
        }
        .to_string(),
        "invalid OAuth client configuration: field 'installed.client_id' is required"
    );
    assert_eq!(
        ClientConfigError::BlankField {
            field: "installed.client_secret".to_string()
        }
        .to_string(),
        "invalid OAuth client configuration: field 'installed.client_secret' must not be empty"
    );
    assert_eq!(
        ClientConfigError::EndpointMismatch {
            field: "installed.token_uri".to_string()
        }
        .to_string(),
        "invalid OAuth client configuration: field 'installed.token_uri' does not match the expected Google endpoint"
    );
    assert_eq!(
        ClientConfigError::Deserialization {
            message: "expected value at line 1 column 1".to_string()
        }
        .to_string(),
        "invalid OAuth client configuration: expected value at line 1 column 1"
    );
    assert_eq!(
        AuthorizationError::MissingParameter {
            parameter: "state".to_string()
        }
        .to_string(),
        "authorization callback is missing the 'state' parameter"
    );
    assert_eq!(
        AuthorizationError::StateMismatch.to_string(),
        "authorization callback state did not match the expected value (possible cross-site request forgery)"
    );
    assert_eq!(
        AuthorizationError::ProviderDeclined {
            reason: "access_denied".to_string()
        }
        .to_string(),
        "the authorization request was declined (provider code 'access_denied')"
    );
    assert_eq!(
        TokenEndpointError::Transport {
            reason: "connection timed out".to_string()
        }
        .to_string(),
        "unable to reach the Google token endpoint: connection timed out"
    );
    assert_eq!(
        TokenEndpointError::Server {
            code: "invalid_grant".to_string(),
            description: Some("Bad Request".to_string()),
        }
        .to_string(),
        "the Google token endpoint rejected the request (code 'invalid_grant'): Bad Request"
    );
    assert_eq!(
        TokenEndpointError::Server {
            code: "invalid_client".to_string(),
            description: None,
        }
        .to_string(),
        "the Google token endpoint rejected the request (code 'invalid_client')"
    );
    assert_eq!(
        TokenEndpointError::ResponseParse.to_string(),
        "the Google token endpoint returned an unreadable response"
    );
    assert_eq!(
        CredentialStoreError::FileOpen {
            path: PathBuf::from("credentials/tokens.json"),
            source: permission_denied(),
        }
        .to_string(),
        "unable to open the credential file 'credentials/tokens.json': permission denied"
    );
    assert_eq!(
        CredentialStoreError::FileWrite {
            path: PathBuf::from("credentials/tokens.json"),
            source: permission_denied(),
        }
        .to_string(),
        "unable to write the credential file 'credentials/tokens.json': permission denied"
    );
    assert_eq!(
        CredentialStoreError::ReadParse {
            path: PathBuf::from("credentials/tokens.json"),
            message: "expected value at line 1 column 1".to_string(),
        }
        .to_string(),
        "unable to parse the credential file 'credentials/tokens.json': expected value at line 1 column 1"
    );
    assert_eq!(
        CredentialStoreError::ContentNotAvailable.to_string(),
        "no stored credentials are available; run authorization first"
    );
    assert_eq!(
        VerificationError::NotAuthorized {
            reason: "endpoint rejected the token".to_string()
        }
        .to_string(),
        "the stored authorization is not valid: endpoint rejected the token"
    );
    assert_eq!(
        VerificationError::InsufficientScope {
            reason: "granted scopes do not include the required scope".to_string()
        }
        .to_string(),
        "the stored authorization lacks the required YouTube scope: granted scopes do not include the required scope"
    );
    assert_eq!(
        VerificationError::Transport {
            reason: "connection refused".to_string()
        }
        .to_string(),
        "unable to verify the stored authorization: connection refused"
    );
}

#[test]
fn source_chain_exposes_io_errors_only() {
    let file_open = CredentialStoreError::FileOpen {
        path: PathBuf::from("credentials/tokens.json"),
        source: permission_denied(),
    };
    let file_write = CredentialStoreError::FileWrite {
        path: PathBuf::from("credentials/tokens.json"),
        source: permission_denied(),
    };
    assert!(file_open.source().is_some());
    assert!(file_write.source().is_some());

    let read_parse = CredentialStoreError::ReadParse {
        path: PathBuf::from("credentials/tokens.json"),
        message: "expected value".to_string(),
    };
    assert!(read_parse.source().is_none());
    assert!(CredentialStoreError::ContentNotAvailable.source().is_none());
    assert!(ClientConfigError::Deserialization {
        message: "expected value".to_string()
    }
    .source()
    .is_none());
    assert!(AuthorizationError::StateMismatch.source().is_none());
    assert!(TokenEndpointError::ResponseParse.source().is_none());
    assert!(VerificationError::Transport {
        reason: "connection refused".to_string()
    }
    .source()
    .is_none());
}
