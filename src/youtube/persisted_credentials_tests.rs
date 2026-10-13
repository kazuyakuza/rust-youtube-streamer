//! Persisted-credential schema tests: the documented on-disk document
//! shape, the fixed redaction-safe parse messages, and the full
//! serialize/parse round trip — direct calls to the module's two
//! `pub(crate)` entry points (offline, deterministic; no Google, no
//! network, no wall clock; file IO is tempfile-only).

use std::fs;

use serde_json::Value;

use super::super::authorization::REQUIRED_SCOPE;
use super::*;

const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";
const REFRESH_TOKEN_FIXTURE: &str = "refresh_token_fixture";

fn stored_credentials() -> StoredCredentials {
    StoredCredentials {
        access_token: AccessToken::new(ACCESS_TOKEN_FIXTURE),
        expires_in_seconds: 3592,
        refresh_token: Some(RefreshToken::new(REFRESH_TOKEN_FIXTURE)),
        granted_scopes: GrantedScopes::from_space_delimited(REQUIRED_SCOPE),
    }
}

#[test]
fn save_writes_the_documented_schema() {
    let text = serialized_document(&stored_credentials()).expect("save must succeed");
    let access_key_at = text
        .find("\"access_token\"")
        .expect("access_token key present");
    let expires_key_at = text
        .find("\"expires_in_seconds\"")
        .expect("expires_in_seconds key present");
    let refresh_key_at = text
        .find("\"refresh_token\"")
        .expect("refresh_token key present");
    let scopes_key_at = text
        .find("\"granted_scopes\"")
        .expect("granted_scopes key present");
    assert!(access_key_at < expires_key_at);
    assert!(expires_key_at < refresh_key_at);
    assert!(refresh_key_at < scopes_key_at);

    let document: Value = serde_json::from_str(&text).expect("credential file must be valid JSON");
    let mut keys = document
        .as_object()
        .expect("document must be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "access_token",
            "expires_in_seconds",
            "granted_scopes",
            "refresh_token"
        ]
    );
    assert_eq!(document["expires_in_seconds"], Value::from(3592));
    assert_eq!(document["access_token"], ACCESS_TOKEN_FIXTURE);
    assert_eq!(document["refresh_token"], REFRESH_TOKEN_FIXTURE);
    assert_eq!(document["granted_scopes"], REQUIRED_SCOPE);
}

#[test]
fn load_reports_least_details_for_malformed_json() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("tokens.json");
    let malformed = format!("{{ not json {ACCESS_TOKEN_FIXTURE}");
    fs::write(&path, malformed).expect("credential file must be writable");
    let text = fs::read_to_string(&path).expect("credential file must be readable");

    let error = parse_credentials(&text, &path).expect_err("malformed JSON must not load");
    match &error {
        CredentialStoreError::ReadParse { message, .. } => {
            assert_eq!(message, "the credential file is not valid JSON");
            assert!(!message.contains(ACCESS_TOKEN_FIXTURE));
        }
        other => panic!("expected ReadParse, got: {other:?}"),
    }
}

#[test]
fn load_reports_fixed_schema_mismatch_message() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("tokens.json");
    let wrong_shape = format!(r#"{{"nope": 1, "leak": "{ACCESS_TOKEN_FIXTURE}"}}"#);
    fs::write(&path, wrong_shape).expect("credential file must be writable");
    let text = fs::read_to_string(&path).expect("credential file must be readable");

    let error = parse_credentials(&text, &path).expect_err("wrong schema must not load");
    match &error {
        CredentialStoreError::ReadParse { message, .. } => {
            assert_eq!(
                message,
                "the credential file does not match the expected credential schema"
            );
            assert!(!message.contains(ACCESS_TOKEN_FIXTURE));
        }
        other => panic!("expected ReadParse, got: {other:?}"),
    }
}

#[test]
fn load_round_trips_the_full_credential_shape() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("tokens.json");
    let text = serialized_document(&stored_credentials()).expect("save must succeed");

    let loaded = parse_credentials(&text, &path).expect("load must succeed");
    assert_eq!(loaded.access_token.secret(), ACCESS_TOKEN_FIXTURE);
    let refresh_token = loaded
        .refresh_token
        .as_ref()
        .expect("refresh token present");
    assert_eq!(refresh_token.secret(), REFRESH_TOKEN_FIXTURE);
    assert_eq!(loaded.expires_in_seconds, 3592);
    assert!(loaded.granted_scopes.grants(REQUIRED_SCOPE));
    assert_eq!(loaded.granted_scopes.to_space_delimited(), REQUIRED_SCOPE);
}

#[test]
fn load_round_trips_null_refresh_token_and_empty_scope() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("tokens.json");
    let mut credentials = stored_credentials();
    credentials.refresh_token = None;
    credentials.granted_scopes = GrantedScopes::from_space_delimited("");
    let text = serialized_document(&credentials).expect("save must succeed");

    let loaded = parse_credentials(&text, &path).expect("load must succeed");
    assert!(loaded.refresh_token.is_none());
    assert!(!loaded.granted_scopes.grants(REQUIRED_SCOPE));
}
