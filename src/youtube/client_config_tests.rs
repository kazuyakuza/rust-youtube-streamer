//! Parsing, validation and redaction tests for Google's client-secret
//! document: required fields, blank values, accepted/foreign endpoint forms,
//! unknown field tolerance, malformed JSON and Debug redaction.

use super::*;

const SECRET_FIXTURE: &str = "client_secret_fixture";

/// Google's downloadable Desktop-client file ships the legacy `auth_uri`
/// (without `/v2`), so the fixture mirrors Google's real document.
fn google_shaped_json() -> String {
    format!(
        r#"{{
            "installed": {{
                "client_id": "1234567890.apps.googleusercontent.com",
                "client_secret": "{SECRET_FIXTURE}",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token"
            }}
        }}"#
    )
}

#[test]
fn parse_reads_installed_client_fields() {
    let config = parse_client_config(&google_shaped_json()).expect("valid document parses");
    assert_eq!(config.client_id(), "1234567890.apps.googleusercontent.com");
    assert_eq!(config.client_secret().secret(), SECRET_FIXTURE);
}

#[test]
fn parse_preserves_unknown_fields() {
    let json = r#"{
        "installed": {
            "project_id": "example-project",
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "client_secret_fixture",
            "auth_uri": "https://accounts.google.com/o/oauth2/v2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "redirect_uris": ["http://127.0.0.1"]
        }
    }"#;
    let config = parse_client_config(json).expect("unknown fields must not break parsing");
    assert_eq!(config.client_id(), "1234567890.apps.googleusercontent.com");
}

#[test]
fn parse_reports_missing_installed_client_id() {
    let error = parse_client_config("{}").expect_err("missing installed block must fail");
    let ClientConfigError::MissingField { field } = &error else {
        panic!("expected MissingField, got {error}");
    };
    assert_eq!(field, "installed.client_id");

    let json = r#"{"installed": {"client_secret": "client_secret_fixture"}}"#;
    let error = parse_client_config(json).expect_err("missing client_id must fail");
    let ClientConfigError::MissingField { field } = &error else {
        panic!("expected MissingField, got {error}");
    };
    assert_eq!(field, "installed.client_id");
}

#[test]
fn parse_reports_blank_client_secret() {
    let json = r#"{
        "installed": {
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "   "
        }
    }"#;
    let error = parse_client_config(json).expect_err("blank secret must fail");
    let ClientConfigError::BlankField { field } = &error else {
        panic!("expected BlankField, got {error}");
    };
    assert_eq!(field, "installed.client_secret");
}

#[test]
fn parse_accepts_legacy_authorization_endpoint() {
    let config = parse_client_config(&google_shaped_json()).expect("legacy auth_uri accepted");
    assert_eq!(config.client_id(), "1234567890.apps.googleusercontent.com");
}

#[test]
fn parse_accepts_v2_authorization_endpoint() {
    let json = r#"{
        "installed": {
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "client_secret_fixture",
            "auth_uri": "https://accounts.google.com/o/oauth2/v2/auth"
        }
    }"#;
    parse_client_config(json).expect("v2 auth_uri accepted");
}

#[test]
fn parse_accepts_legacy_token_endpoint() {
    let json = r#"{
        "installed": {
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "client_secret_fixture",
            "token_uri": "https://www.googleapis.com/oauth2/v4/token"
        }
    }"#;
    parse_client_config(json).expect("legacy token_uri accepted");
}

#[test]
fn parse_accepts_absent_endpoints() {
    let json = r#"{"installed": {
        "client_id": "1234567890.apps.googleusercontent.com",
        "client_secret": "client_secret_fixture"
    }}"#;
    parse_client_config(json).expect("absent endpoints accepted");
}

#[test]
fn parse_rejects_foreign_authorization_endpoint() {
    let json = r#"{
        "installed": {
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "client_secret_fixture",
            "auth_uri": "https://evil.invalid/auth"
        }
    }"#;
    let error = parse_client_config(json).expect_err("foreign auth_uri must fail");
    let ClientConfigError::EndpointMismatch { field } = &error else {
        panic!("expected EndpointMismatch, got {error}");
    };
    assert_eq!(field, "installed.auth_uri");
}

#[test]
fn parse_rejects_foreign_token_uri() {
    let json = r#"{
        "installed": {
            "client_id": "1234567890.apps.googleusercontent.com",
            "client_secret": "client_secret_fixture",
            "token_uri": "https://evil.invalid/token"
        }
    }"#;
    let error = parse_client_config(json).expect_err("foreign token_uri must fail");
    let ClientConfigError::EndpointMismatch { field } = &error else {
        panic!("expected EndpointMismatch, got {error}");
    };
    assert_eq!(field, "installed.token_uri");
}

#[test]
fn parse_rejects_invalid_json() {
    let error = parse_client_config("not json at all").expect_err("garbage must fail");
    assert!(matches!(error, ClientConfigError::Deserialization { .. }));
}

#[test]
fn client_config_debug_never_reveals_secret() {
    let config = parse_client_config(&google_shaped_json()).expect("valid document parses");
    let debug_output = format!("{config:?}");
    assert!(!debug_output.contains(SECRET_FIXTURE));
    assert!(debug_output.contains("<redacted>"));
}
