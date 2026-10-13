//! File credential store tests: atomic replacement, parent-directory
//! creation, typed error mapping, Unix permission SET bits, and
//! redaction guarantees — offline and deterministic (tempfile only; no
//! Google, no network, no wall clock). Container tests run as root, so
//! permissions assert set bits, never denial.

use std::error::Error;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::Value;

use super::super::authorization::REQUIRED_SCOPE;
use super::super::token::{AccessToken, GrantedScopes, RefreshToken};
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

fn store_at(dir: &tempfile::TempDir) -> FileCredentialStore {
    FileCredentialStore::new(dir.path().join("tokens.json"))
}

fn read_document(store: &FileCredentialStore) -> Value {
    let text = fs::read_to_string(&store.path).expect("credential file must be readable");
    serde_json::from_str(&text).expect("credential file must be valid JSON")
}

#[test]
fn save_is_atomic_replacing_in_place() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = store_at(&dir);
    store
        .save(&stored_credentials())
        .expect("first save must succeed");

    let first_document = read_document(&store);
    assert_eq!(first_document["access_token"], ACCESS_TOKEN_FIXTURE);
    assert_eq!(first_document["refresh_token"], REFRESH_TOKEN_FIXTURE);

    let mut newer = stored_credentials();
    newer.access_token = AccessToken::new("new_access_value");
    newer.refresh_token = Some(RefreshToken::new("new_refresh_value"));
    store.save(&newer).expect("second save must succeed");

    let document = read_document(&store);
    assert_eq!(document["access_token"], "new_access_value");
    assert_eq!(document["refresh_token"], "new_refresh_value");
    let text = fs::read_to_string(&store.path).expect("credential file must be readable");
    assert!(!text.contains(ACCESS_TOKEN_FIXTURE));
    assert!(!text.contains(REFRESH_TOKEN_FIXTURE));
}

#[test]
fn save_removes_a_stale_temp_file_and_survives_an_interrupted_save_shape() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = store_at(&dir);
    let temp_path = store.temp_path().expect("temp path must be valid");
    fs::write(&temp_path, b"garbage left by an interrupted save").expect("temp must be writable");

    store
        .save(&stored_credentials())
        .expect("save must succeed");

    assert!(!temp_path.exists());
    let document = read_document(&store);
    assert_eq!(document["access_token"], ACCESS_TOKEN_FIXTURE);
}

#[test]
fn load_returns_content_not_available_for_missing_file() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = store_at(&dir);
    let error = store.load().expect_err("missing file must not load");
    assert!(matches!(error, CredentialStoreError::ContentNotAvailable));
    let message = error.to_string();
    assert!(message.contains("no stored credentials are available"));
}

#[test]
fn load_maps_directory_read_to_file_open() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = FileCredentialStore::new(dir.path().to_path_buf());
    let error = store.load().expect_err("directory must not load");
    match error {
        CredentialStoreError::FileOpen { .. } => {}
        other => panic!("expected FileOpen, got: {other}"),
    }
    assert!(error.source().is_some());
}

#[test]
fn save_creates_a_missing_parent_directory() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let nested_path = dir
        .path()
        .join("nested")
        .join("credentials")
        .join("tokens.json");
    let store = FileCredentialStore::new(nested_path);

    store
        .save(&stored_credentials())
        .expect("save must succeed");

    assert!(store.path.is_file());
}

#[test]
fn save_fails_with_file_open_for_a_directory_parent_colliding_with_a_file() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let blocking_file = dir.path().join("parent");
    fs::write(&blocking_file, b"blocking file").expect("blocking file must be writable");
    let store = FileCredentialStore::new(blocking_file.join("tokens.json"));

    let error = store
        .save(&stored_credentials())
        .expect_err("save must fail");
    match &error {
        CredentialStoreError::FileOpen { path, .. } => assert_eq!(path, &store.path),
        other => panic!("expected FileOpen, got: {other:?}"),
    }
}

#[test]
fn target_directory_collision_maps_to_file_write() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let target = dir.path().join("tokens.json");
    fs::create_dir(&target).expect("target directory must be created");
    let store = FileCredentialStore::new(target);

    let error = store
        .save(&stored_credentials())
        .expect_err("save must fail");
    assert!(matches!(error, CredentialStoreError::FileWrite { .. }));
    let temp_path = store.temp_path().expect("temp path must be valid");
    assert!(!temp_path.exists());
}

#[test]
fn stored_credentials_debug_reveals_no_secrets() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = store_at(&dir);
    store
        .save(&stored_credentials())
        .expect("save must succeed");

    let loaded = store.load().expect("load must succeed");
    let rendered = format!("{loaded:?}");
    assert!(!rendered.contains(ACCESS_TOKEN_FIXTURE));
    assert!(!rendered.contains(REFRESH_TOKEN_FIXTURE));
    assert!(rendered.contains("AccessToken(<redacted>)"));
    assert!(rendered.contains("RefreshToken(<redacted>)"));
}

#[cfg(unix)]
#[test]
fn unix_mode_is_owner_only_after_save() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let store = store_at(&dir);
    store
        .save(&stored_credentials())
        .expect("save must succeed");

    let metadata = fs::metadata(&store.path).expect("credential file metadata must be readable");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[test]
fn replace_overwrites_an_existing_target() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let target = dir.path().join("tokens.json");
    fs::write(&target, b"previous credential content").expect("target must be writable");
    let temp_path = dir.path().join(".tokens.json.tmp");
    fs::write(&temp_path, b"fresh credential content").expect("temp must be writable");

    sync_replace(&temp_path, &target).expect("replace must succeed");

    assert_eq!(
        fs::read_to_string(&target).expect("target must be readable"),
        "fresh credential content"
    );
    assert!(!temp_path.exists());
}
