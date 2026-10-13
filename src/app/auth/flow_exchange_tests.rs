//! Code-exchange and store-seam tests (F7-F10): request assembly against
//! a capturing token-transport fake, exchange failure mapping, and the
//! credential-store seam (store fakes live with the flow-test siblings).
//! Fully offline.

use std::cell::RefCell;

use oauth2::PkceCodeVerifier;

use super::flow_tests::{
    parsed_client_config, recording_store, token_response_with, FailingStore, ACCESS_TOKEN_FIXTURE,
    CLIENT_ID_FIXTURE, CLIENT_SECRET_FIXTURE, FIXTURE_EXPIRES_IN_SECONDS, REFRESH_TOKEN_FIXTURE,
};
use super::{storeable_credentials, AuthError, AuthFlow};
use crate::config::AppConfig;
use crate::youtube::{
    AuthorizationCode, CodeExchangeRequest, CredentialStore, RefreshRequest, RefreshToken,
    TokenEndpointError, TokenEndpointTransport, TokenResponse, REQUIRED_SCOPE,
};

const REDIRECT_URI_FIXTURE: &str = "http://127.0.0.1:52144";
const AUTHORIZATION_CODE_FIXTURE: &str = "authorization_code_fixture";
const PKCE_VERIFIER_FIXTURE: &str = "pkce_verifier_fixture_abcdefghijklmnopqrstuvwxyz012345";

struct CapturedRequest {
    client_id: String,
    redirect_uri: String,
    wrappers_match_fixtures: bool,
}

impl CapturedRequest {
    fn from_request(request: &CodeExchangeRequest) -> Self {
        Self {
            client_id: request.client_id.clone(),
            redirect_uri: request.redirect_uri.clone(),
            wrappers_match_fixtures: request.code.secret() == AUTHORIZATION_CODE_FIXTURE
                && request.pkce_verifier.secret() == PKCE_VERIFIER_FIXTURE,
        }
    }
}

pub(super) struct ScriptedTransport {
    succeeds_with: Option<TokenResponse>,
    captured: RefCell<Option<CapturedRequest>>,
}

impl ScriptedTransport {
    fn scripted(succeeds_with: Option<TokenResponse>) -> Self {
        Self {
            succeeds_with,
            captured: RefCell::new(None),
        }
    }

    pub(super) fn exchange_was_attempted(&self) -> bool {
        self.captured.borrow().is_some()
    }
}

impl TokenEndpointTransport for ScriptedTransport {
    fn authorization_code_exchange(
        &self,
        request: &CodeExchangeRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        *self.captured.borrow_mut() = Some(CapturedRequest::from_request(request));
        match &self.succeeds_with {
            Some(response) => Ok(TokenResponse {
                access_token: response.access_token.clone(),
                expires_in_seconds: response.expires_in_seconds,
                refresh_token: response.refresh_token.clone(),
                granted_scopes: response.granted_scopes.clone(),
            }),
            None => Err(TokenEndpointError::Server {
                code: "invalid_grant".to_string(),
                description: None,
            }),
        }
    }

    fn refresh_token_exchange(
        &self,
        _request: &RefreshRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        Err(TokenEndpointError::ResponseParse)
    }
}

pub(super) fn capturing_failing_transport() -> ScriptedTransport {
    ScriptedTransport::scripted(None)
}

fn fixture_refresh_response() -> TokenResponse {
    token_response_with(
        Some(RefreshToken::new(REFRESH_TOKEN_FIXTURE)),
        REQUIRED_SCOPE,
    )
}

fn capturing_ok_transport() -> ScriptedTransport {
    ScriptedTransport::scripted(Some(fixture_refresh_response()))
}

/// AuthFlow under test against the given seams and the shared config.
pub(super) fn flow_under_test<'a>(
    transport: &'a dyn TokenEndpointTransport,
    store: &'a dyn CredentialStore,
) -> AuthFlow<'a> {
    static CONFIG: std::sync::OnceLock<AppConfig> = std::sync::OnceLock::new();
    let config = CONFIG.get_or_init(crate::config::valid_config);
    AuthFlow {
        config,
        transport,
        store,
    }
}

fn opened_flow() -> super::OpenedFlow {
    super::OpenedFlow {
        code: AuthorizationCode::new(AUTHORIZATION_CODE_FIXTURE),
        pkce_verifier: PkceCodeVerifier::new(PKCE_VERIFIER_FIXTURE.to_string()),
        redirect_uri: REDIRECT_URI_FIXTURE.to_string(),
    }
}

fn assert_no_fixture_secrets(haystack: &str) {
    for fixture in [
        CLIENT_SECRET_FIXTURE,
        ACCESS_TOKEN_FIXTURE,
        REFRESH_TOKEN_FIXTURE,
        AUTHORIZATION_CODE_FIXTURE,
        PKCE_VERIFIER_FIXTURE,
    ] {
        assert!(!haystack.contains(fixture), "leaked fixture in {haystack}");
    }
}

#[test]
fn code_exchange_request_assembles_the_documented_fields() {
    let client_config = parsed_client_config();
    let transport = capturing_ok_transport();
    let store = recording_store();
    let flow = flow_under_test(&transport, &store);
    let response = flow
        .exchange_authorization_code(&client_config, opened_flow())
        .expect("the fake transport succeeds");
    assert_eq!(response.expires_in_seconds, FIXTURE_EXPIRES_IN_SECONDS);
    let captured = transport.captured.borrow();
    let captured = captured.as_ref().expect("the request was captured");
    assert_eq!(captured.client_id, CLIENT_ID_FIXTURE);
    assert_eq!(captured.redirect_uri, REDIRECT_URI_FIXTURE);
    assert!(captured.wrappers_match_fixtures);
}

#[test]
fn failing_exchange_maps_to_token_exchange_error() {
    let client_config = parsed_client_config();
    let transport = capturing_failing_transport();
    let store = recording_store();
    let flow = flow_under_test(&transport, &store);
    let error = flow
        .exchange_authorization_code(&client_config, opened_flow())
        .expect_err("the fake transport fails");
    let AuthError::TokenExchange(inner) = &error else {
        panic!("expected the TokenExchange class, got {error}");
    };
    let message = format!("{inner}");
    assert!(message.contains("invalid_grant"));
    assert_no_fixture_secrets(&message);
    assert_no_fixture_secrets(&format!("{error:?}"));
}

#[test]
fn saved_credentials_reach_the_store_seam() {
    let stored = storeable_credentials(fixture_refresh_response()).expect("full response");
    let store = recording_store();
    let transport = capturing_failing_transport();
    let flow = flow_under_test(&transport, &store);
    flow.save_validated_credentials(&stored)
        .expect("the recording store saves");
    assert_eq!(store.recorded_count(), 1);
    let records = store.records.borrow();
    let recorded = records.first().expect("one recorded save");
    assert_eq!(recorded.expires_in_seconds, FIXTURE_EXPIRES_IN_SECONDS);
    assert!(recorded.access_token.secret() == ACCESS_TOKEN_FIXTURE);
    let refresh = recorded.refresh_token.as_ref().expect("present");
    assert!(refresh.secret() == REFRESH_TOKEN_FIXTURE);
    assert!(recorded.granted_scopes.grants(REQUIRED_SCOPE));
}

#[test]
fn store_errors_map_to_the_persistence_class() {
    let stored = storeable_credentials(fixture_refresh_response()).expect("full response");
    let store = FailingStore;
    let transport = capturing_failing_transport();
    let flow = flow_under_test(&transport, &store);
    let error = flow
        .save_validated_credentials(&stored)
        .expect_err("the failing store errors");
    assert!(matches!(error, AuthError::Persistence(_)));
}
