//! Client-file, validation and end-to-end flow tests (F1-F6, F11):
//! tempdir client fixtures for the read/parse classes, the offline
//! scope/refresh validation of `storeable_credentials`, and one full
//! declined-provider flow through `run_with_listener` on a prebound
//! loopback listener. Offline except loopback; no real Google, no
//! external network. Also hosts the credential-store fakes shared with
//! the exchange tests.

use std::cell::RefCell;

use super::flow_exchange_tests::capturing_failing_transport;
use super::{read_client_config, storeable_credentials, AuthError, AuthFlow};
use crate::youtube::{
    parse_client_config, AccessToken, ClientConfig, CredentialStore, CredentialStoreError,
    GrantedScopes, RefreshToken, StoredCredentials, TokenResponse, REQUIRED_SCOPE,
};

pub(super) const CLIENT_ID_FIXTURE: &str = "1234567890.apps.googleusercontent.com";
pub(super) const CLIENT_SECRET_FIXTURE: &str = "client_secret_fixture";
pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";
pub(super) const REFRESH_TOKEN_FIXTURE: &str = "refresh_token_fixture";
pub(super) const FIXTURE_EXPIRES_IN_SECONDS: u64 = 3592;

pub(super) fn google_shaped_client_json() -> String {
    format!(
        r#"{{
            "installed": {{
                "client_id": "{CLIENT_ID_FIXTURE}",
                "client_secret": "{CLIENT_SECRET_FIXTURE}",
                "auth_uri": "https://accounts.google.com/o/oauth2/v2/auth",
                "token_uri": "https://oauth2.googleapis.com/token"
            }}
        }}"#
    )
}

pub(super) fn parsed_client_config() -> ClientConfig {
    parse_client_config(&google_shaped_client_json()).expect("the fixture document parses")
}

pub(super) fn token_response_with(
    refresh_token: Option<RefreshToken>,
    scopes: &str,
) -> TokenResponse {
    TokenResponse {
        access_token: AccessToken::new(ACCESS_TOKEN_FIXTURE),
        expires_in_seconds: FIXTURE_EXPIRES_IN_SECONDS,
        refresh_token,
        granted_scopes: GrantedScopes::from_space_delimited(scopes),
    }
}

pub(super) struct RecordingStore {
    pub(super) records: RefCell<Vec<StoredCredentials>>,
}

impl RecordingStore {
    pub(super) fn recorded_count(&self) -> usize {
        self.records.borrow().len()
    }
}

impl CredentialStore for RecordingStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        Err(CredentialStoreError::ContentNotAvailable)
    }

    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        self.records.borrow_mut().push(StoredCredentials {
            access_token: credentials.access_token.clone(),
            expires_in_seconds: credentials.expires_in_seconds,
            refresh_token: credentials.refresh_token.clone(),
            granted_scopes: credentials.granted_scopes.clone(),
        });
        Ok(())
    }
}

pub(super) fn recording_store() -> RecordingStore {
    RecordingStore {
        records: RefCell::new(Vec::new()),
    }
}

pub(super) struct FailingStore;

impl CredentialStore for FailingStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        Err(CredentialStoreError::ContentNotAvailable)
    }

    fn save(&self, _credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        Err(CredentialStoreError::FileWrite {
            path: std::path::PathBuf::from("credentials/tokens.json"),
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fixture write failure",
            ),
        })
    }
}

fn write_fixture_client_file(directory: &tempfile::TempDir) -> std::path::PathBuf {
    let client_path = directory.path().join("client_secret.json");
    std::fs::write(&client_path, google_shaped_client_json()).expect("the fixture file is written");
    client_path
}

fn spawn_declined_browser(port: u16) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut stream =
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("the listener accepts");
        std::io::Write::write_all(
            &mut stream,
            b"GET /?error=access_denied HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        )
        .expect("the request is sent");
        let mut response = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut response).expect("the response is read");
        response
    })
}

#[test]
fn client_file_happy_parse_returns_client_config() {
    let temp_dir = tempfile::tempdir().expect("a temp directory");
    let client_path = write_fixture_client_file(&temp_dir);
    let client_config =
        read_client_config(&client_path.to_string_lossy()).expect("the fixture file parses");
    assert_eq!(client_config.client_id(), CLIENT_ID_FIXTURE);
    assert!(client_config.client_secret().secret() == CLIENT_SECRET_FIXTURE);
}

#[test]
fn client_file_read_failure_is_the_client_config_class() {
    let temp_dir = tempfile::tempdir().expect("a temp directory");
    let missing_path = temp_dir.path().join("missing_client_secret.json");
    let error =
        read_client_config(&missing_path.to_string_lossy()).expect_err("a missing file must fail");
    assert!(matches!(error, AuthError::ClientConfigFile { .. }));
    assert!(format!("{error}").starts_with("unable to read the OAuth client configuration file"));
}

#[test]
fn client_file_invalid_content_is_the_client_config_class() {
    let temp_dir = tempfile::tempdir().expect("a temp directory");
    let client_path = temp_dir.path().join("client_secret.json");
    std::fs::write(&client_path, "not json").expect("the fixture file is written");
    let error =
        read_client_config(&client_path.to_string_lossy()).expect_err("invalid content must fail");
    let AuthError::ClientConfig(inner) = &error else {
        panic!("expected the ClientConfig class, got {error}");
    };
    assert_eq!(format!("{error}"), format!("{inner}"));
}

#[test]
fn stored_response_yields_storeable_credentials() {
    let response = token_response_with(
        Some(RefreshToken::new(REFRESH_TOKEN_FIXTURE)),
        REQUIRED_SCOPE,
    );
    let stored = storeable_credentials(response).expect("a full response is storeable");
    assert_eq!(stored.expires_in_seconds, FIXTURE_EXPIRES_IN_SECONDS);
    assert!(stored.access_token.secret() == ACCESS_TOKEN_FIXTURE);
    let refresh = stored
        .refresh_token
        .as_ref()
        .expect("the refresh token is kept");
    assert!(refresh.secret() == REFRESH_TOKEN_FIXTURE);
    assert!(stored.granted_scopes.grants(REQUIRED_SCOPE));
}

#[test]
fn storeable_credentials_rejects_missing_required_scope() {
    let response = token_response_with(
        Some(RefreshToken::new(REFRESH_TOKEN_FIXTURE)),
        "https://www.googleapis.com/auth/youtube.readonly",
    );
    let error = storeable_credentials(response).expect_err("a missing scope must fail");
    assert!(matches!(error, AuthError::InsufficientScope));
    assert!(format!("{error}").contains(REQUIRED_SCOPE));
}

#[test]
fn storeable_credentials_rejects_missing_refresh_token() {
    let response = token_response_with(None, REQUIRED_SCOPE);
    let error = storeable_credentials(response).expect_err("a missing refresh token must fail");
    assert!(matches!(error, AuthError::MissingRefreshToken));
}

#[test]
fn declined_provider_flow_ends_without_persisting_end_to_end() {
    let temp_dir = tempfile::tempdir().expect("a temp directory");
    let client_path = write_fixture_client_file(&temp_dir);
    let client_config =
        read_client_config(&client_path.to_string_lossy()).expect("the fixture file parses");
    let bound = super::listener::bind().expect("the loopback listener binds");
    let port = bound
        .redirect_uri()
        .rsplit_once(':')
        .expect("a port suffix")
        .1
        .parse()
        .expect("a numeric port");
    let store = recording_store();
    let transport = capturing_failing_transport();
    let config = crate::config::valid_config();
    let flow = AuthFlow {
        config: &config,
        transport: &transport,
        store: &store,
    };
    let browser = spawn_declined_browser(port);
    let outcome = flow.run_with_listener(&client_config, &bound);
    let response = browser.join().expect("the browser thread completes");
    let error = outcome.expect_err("the declined callback must fail the flow");
    let AuthError::ProviderDeclined { reason } = error else {
        panic!("expected ProviderDeclined, got {error}");
    };
    assert!(reason == "access_denied");
    assert_eq!(store.recorded_count(), 0);
    assert!(!transport.exchange_was_attempted());
    assert!(response.starts_with(b"HTTP/1.1 400 Bad Request"));
}
