//! Authorization URL construction, PKCE/state generation and callback
//! parsing tests: exact encoded URL, force-ssl scope, RFC 7636 PKCE
//! shape, CSRF state validation, provider declines and missing
//! parameters — all offline.

use oauth2::url::Url;

use super::super::client_config::parse_client_config;
use super::*;

const RFC7636_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const RFC7636_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

fn fixture_client_config() -> ClientConfig {
    let json = r#"{
        "installed": {
            "client_id": "client_id_fixture",
            "client_secret": "client_secret_fixture",
            "auth_uri": "https://accounts.google.com/o/oauth2/v2/auth",
            "token_uri": "https://oauth2.googleapis.com/token"
        }
    }"#;
    parse_client_config(json).expect("fixture client configuration parses")
}

fn rfc7636_pkce() -> PkcePair {
    PkcePair {
        challenge: oauth2::PkceCodeChallenge::from_code_verifier_sha256(
            &oauth2::PkceCodeVerifier::new(RFC7636_VERIFIER.to_string()),
        ),
        verifier: oauth2::PkceCodeVerifier::new(RFC7636_VERIFIER.to_string()),
    }
}

fn fixture_state() -> oauth2::CsrfToken {
    oauth2::CsrfToken::new("state_fixture".to_string())
}

/// The authorization URL built from the fixtures, used by the two URL tests.
fn built_authorization_url() -> String {
    authorization_url(
        &fixture_client_config(),
        "http://127.0.0.1:42123",
        &fixture_state(),
        &rfc7636_pkce(),
    )
}

#[test]
fn authorization_url_encodes_all_parameters() {
    let url = built_authorization_url();
    assert_eq!(
        url,
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id=client_id_fixture\
             &response_type=code\
             &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.force-ssl\
             &redirect_uri=http%3A%2F%2F127.0.0.1%3A42123\
             &state=state_fixture\
             &code_challenge={RFC7636_CHALLENGE}\
             &code_challenge_method=S256"
        )
    );
    assert_eq!(rfc7636_pkce().challenge.as_str(), RFC7636_CHALLENGE);
}

#[test]
fn authorization_url_uses_the_force_ssl_scope() {
    let url = built_authorization_url();
    let parsed = Url::parse(&url).expect("authorization URL parses");
    let scope = parsed
        .query_pairs()
        .find(|(key, _)| key == "scope")
        .map(|(_, value)| value.to_string());
    assert_eq!(scope.as_deref(), Some(REQUIRED_SCOPE));
    assert_eq!(
        REQUIRED_SCOPE,
        "https://www.googleapis.com/auth/youtube.force-ssl"
    );
}

#[test]
fn generate_pkce_produces_valid_pair() {
    let pair = generate_pkce();
    let verifier = pair.verifier.secret();
    assert!(verifier.len() >= 43);
    assert!(verifier.len() <= 128);
    assert!(verifier
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-._~".contains(character)));
    let expected_challenge = oauth2::PkceCodeChallenge::from_code_verifier_sha256(&pair.verifier);
    assert_eq!(pair.challenge.as_str(), expected_challenge.as_str());
}

#[test]
fn generate_state_produces_random_value() {
    assert_ne!(generate_state().secret(), generate_state().secret());
}

#[test]
fn callback_parses_authorized_query() {
    let callback =
        parse_authorization_callback("code=code_fixture&state=state_fixture", &fixture_state())
            .expect("authorized query parses");
    let AuthorizationCallback::Authorized { code, state } = &callback else {
        panic!("expected Authorized, got {callback:?}");
    };
    assert_eq!(code.secret(), "code_fixture");
    assert_eq!(state, "state_fixture");
}

#[test]
fn callback_rejects_state_mismatch() {
    let error =
        parse_authorization_callback("code=code_fixture&state=attacker_state", &fixture_state())
            .expect_err("mismatched state must fail");
    assert!(matches!(error, AuthorizationError::StateMismatch));
    let rendered = format!("{error}{error:?}");
    assert!(!rendered.contains("attacker_state"));
    assert!(!rendered.contains("code_fixture"));
}

#[test]
fn callback_parses_provider_declined() {
    let callback = parse_authorization_callback("error=access_denied", &fixture_state())
        .expect("declined query parses");
    let AuthorizationCallback::Declined { reason } = &callback else {
        panic!("expected Declined, got {callback:?}");
    };
    assert_eq!(reason, "access_denied");
}

#[test]
fn callback_reports_missing_parameter() {
    let error = parse_authorization_callback("code=code_fixture", &fixture_state())
        .expect_err("missing state must fail");
    let AuthorizationError::MissingParameter { parameter } = &error else {
        panic!("expected MissingParameter, got {error}");
    };
    assert_eq!(parameter, "state");

    let error = parse_authorization_callback("state=state_fixture", &fixture_state())
        .expect_err("missing code must fail");
    let AuthorizationError::MissingParameter { parameter } = &error else {
        panic!("expected MissingParameter, got {error}");
    };
    assert_eq!(parameter, "code");
}
