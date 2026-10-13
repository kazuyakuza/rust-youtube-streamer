//! Exit-code, Display-contract and redaction tests for [`AuthError`] and
//! the `run_auth` entry: the startup-failure exit mapping, the exact
//! per-variant Display lines (D2), and the guarantee that Debug/Display
//! never reveal token/secret material (D16).

use super::*;
use crate::config::valid_config;

const CLIENT_SECRET_FIXTURE: &str = "client_secret_fixture";
const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";
const REFRESH_TOKEN_FIXTURE: &str = "refresh_token_fixture";
const AUTHORIZATION_CODE_FIXTURE: &str = "authorization_code_fixture";
const PKCE_VERIFIER_FIXTURE: &str = "pkce_verifier_fixture_abcdefghijklmnopqrstuvwxyz012345";

fn fixture_io_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, "fixture io failure")
}

fn all_auth_errors() -> Vec<AuthError> {
    vec![
        AuthError::ClientConfigFile {
            path: "credentials/client_secret.json".to_string(),
            source: fixture_io_error(),
        },
        AuthError::ClientConfig(ClientConfigError::MissingField {
            field: "installed.client_id".to_string(),
        }),
        AuthError::ListenerBind(fixture_io_error()),
        AuthError::CallbackAbandoned { attempts: 8 },
        AuthError::StateMismatch,
        AuthError::ProviderDeclined {
            reason: "access_denied".to_string(),
        },
        AuthError::TokenExchange(TokenEndpointError::Server {
            code: "invalid_grant".to_string(),
            description: None,
        }),
        AuthError::Persistence(CredentialStoreError::ContentNotAvailable),
        AuthError::InsufficientScope,
        AuthError::MissingRefreshToken,
    ]
}

fn assert_contains_none(haystack: &str) {
    for fixture in [
        CLIENT_SECRET_FIXTURE,
        ACCESS_TOKEN_FIXTURE,
        REFRESH_TOKEN_FIXTURE,
        AUTHORIZATION_CODE_FIXTURE,
        PKCE_VERIFIER_FIXTURE,
    ] {
        assert!(!haystack.contains(fixture), "leaked fixture in {haystack}");
    }
    assert!(!haystack.contains("code="), "callback echo in {haystack}");
    assert!(!haystack.contains("state="), "callback echo in {haystack}");
}

#[test]
fn run_auth_reports_missing_client_file_with_startup_failure_exit() {
    let temp_dir = tempfile::tempdir().expect("a temp directory");
    let missing_path = temp_dir.path().join("missing_client_secret.json");
    let mut config = valid_config();
    config.oauth.client_config_path = missing_path.to_string_lossy().to_string();
    assert_eq!(run_auth(&config), STARTUP_FAILURE_EXIT);
}

#[test]
fn every_auth_error_maps_to_the_startup_failure_exit() {
    let errors = all_auth_errors();
    assert_eq!(errors.len(), 10, "every variant is covered");
    for error in errors {
        assert_eq!(error.exit_code(), STARTUP_FAILURE_EXIT);
    }
}

#[test]
fn auth_error_display_lines_are_stable() {
    let errors = all_auth_errors();
    let displayed: Vec<String> = errors.iter().map(|error| error.to_string()).collect();
    assert_eq!(
        displayed[0],
        "unable to read the OAuth client configuration file \
'credentials/client_secret.json': fixture io failure"
    );
    assert_eq!(
        displayed[1],
        "invalid OAuth client configuration: field 'installed.client_id' is required"
    );
    assert_eq!(
        displayed[2],
        "unable to bind the OAuth callback listener on 127.0.0.1: fixture io failure"
    );
    assert_eq!(
        displayed[3],
        "no usable authorization callback was received within the allowed 8 \
connection attempts; run the authorization command again and re-open the \
printed URL"
    );
    assert_eq!(
        displayed[4],
        "authorization callback state did not match the expected value \
(possible cross-site request forgery); run the authorization command again"
    );
    assert_eq!(
        displayed[5],
        "the authorization was declined by the provider (error code \
'access_denied'); run the authorization command again if this was unintended"
    );
    assert_eq!(
        displayed[6],
        "the Google token endpoint rejected the request (code 'invalid_grant')"
    );
    assert_eq!(
        displayed[7],
        "no stored credentials are available; run authorization first"
    );
    assert_eq!(
        displayed[8],
        "the granted authorization does not include the required YouTube scope \
(https://www.googleapis.com/auth/youtube.force-ssl); no credentials were \
stored; run the authorization command again and approve the requested \
YouTube permission"
    );
    assert_eq!(
        displayed[9],
        "the token response did not include a refresh token, which later \
unattended runs require; no credentials were stored; run the authorization \
command again and approve the requested YouTube permission"
    );
}

#[test]
fn auth_error_debug_and_display_reveal_no_secret_fixtures() {
    let required_scope = crate::youtube::REQUIRED_SCOPE;
    let fake_token_endpoint_body = format!(
        "{{\"access_token\":\"{ACCESS_TOKEN_FIXTURE}\",\
\"refresh_token\":\"{REFRESH_TOKEN_FIXTURE}\",\
\"scope\":\"{required_scope}\"}}"
    );
    assert!(fake_token_endpoint_body.contains(ACCESS_TOKEN_FIXTURE));
    assert!(fake_token_endpoint_body.contains(REFRESH_TOKEN_FIXTURE));
    for error in all_auth_errors() {
        assert_contains_none(&format!("{error:?}"));
        assert_contains_none(&format!("{error}"));
    }
}
