//! Credential service refresh-path tests (S3-S10, S13): the
//! refresh-decision outcomes, merge semantics (kept refresh token,
//! scope replacement), endpoint failure classes, and the persistence
//! failure mapping. Pure fakes; no network or clock.

use std::cell::RefCell;

use super::credential_service_tests::*;
use super::*;
use crate::youtube::transport::TokenExchangeResult;

pub(super) const REFRESHED_ACCESS_TOKEN_FIXTURE: &str = "refreshed_access_token_fixture";
pub(super) const REFRESHED_REFRESH_TOKEN_FIXTURE: &str = "refreshed_refresh_token_fixture";

/// Failure extraction without `expect_err` (`VerifiedCredentials` is
/// Debug-free by design, plan D6).
pub(super) fn service_failure(
    outcome: Result<VerifiedCredentials, CredentialServiceError>,
) -> CredentialServiceError {
    let Err(error) = outcome else {
        panic!("expected a credential service failure");
    };
    error
}

pub(super) fn refresh_response_with(
    access_token: &str,
    refresh_token: Option<&str>,
    scopes: &str,
) -> TokenResponse {
    TokenResponse {
        access_token: AccessToken::new(access_token),
        expires_in_seconds: FIXTURE_EXPIRES_IN_SECONDS,
        refresh_token: refresh_token.map(RefreshToken::new),
        granted_scopes: GrantedScopes::from_space_delimited(scopes),
    }
}

pub(super) fn refresh_ok() -> TokenExchangeResult {
    Ok(refresh_response_with(
        REFRESHED_ACCESS_TOKEN_FIXTURE,
        None,
        REQUIRED_SCOPE,
    ))
}

pub(super) fn answering(result: TokenExchangeResult) -> FakeEndpoint {
    FakeEndpoint {
        refresh_result: RefCell::new(Some(result)),
        ..FakeEndpoint::default()
    }
}

pub(super) fn file_write_error() -> CredentialStoreError {
    CredentialStoreError::FileWrite {
        path: std::path::PathBuf::from("credentials/tokens.json"),
        source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
    }
}

#[test]
fn rejected_token_refreshes_saves_and_reverifies() {
    let outcomes = vec![not_authorized(), verified_evidence()];
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(refresh_ok()))
        .with_verifier(QueueVerifier::queued(outcomes));
    service
        .establish()
        .expect("the refresh recovers the rejected token");
    assert_eq!(service.endpoint.refresh_calls.get(), 1);
    let captured = service.endpoint.captured_client_id.borrow();
    assert!(captured.as_deref() == Some("1234567890.apps.googleusercontent.com"));
    let last_token = service.verifier.last_token.borrow().clone();
    assert_eq!(last_token.as_deref(), Some(REFRESHED_ACCESS_TOKEN_FIXTURE));
    let saved = service.store.saved.borrow();
    assert_eq!(saved.len(), 1);
    let stored = saved.first().expect("one saved record");
    assert!(stored.access_token.secret() == REFRESHED_ACCESS_TOKEN_FIXTURE);
    assert!(stored.refresh_token.as_ref().expect("kept").secret() == REFRESH_TOKEN_FIXTURE);
    assert!(stored.granted_scopes.grants(REQUIRED_SCOPE));
}

#[test]
fn refreshed_refresh_token_persists_when_google_respends_one() {
    let response = refresh_response_with(
        REFRESHED_ACCESS_TOKEN_FIXTURE,
        Some(REFRESHED_REFRESH_TOKEN_FIXTURE),
        REQUIRED_SCOPE,
    );
    let outcomes = vec![not_authorized(), verified_evidence()];
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Ok(response)))
        .with_verifier(QueueVerifier::queued(outcomes));
    service.establish().expect("the refresh succeeds");
    let saved = service.store.saved.borrow();
    let stored = saved.first().expect("one saved record");
    let refresh = stored.refresh_token.as_ref().expect("replaced");
    assert!(refresh.secret() == REFRESHED_REFRESH_TOKEN_FIXTURE);
}

#[test]
fn refresh_response_scope_replaces_granted_evidence_when_present() {
    let response_scopes =
        format!("{REQUIRED_SCOPE} https://www.googleapis.com/auth/youtube.readonly");
    let response = refresh_response_with(REFRESHED_ACCESS_TOKEN_FIXTURE, None, &response_scopes);
    let outcomes = vec![not_authorized(), verified_evidence()];
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Ok(response)))
        .with_verifier(QueueVerifier::queued(outcomes));
    service.establish().expect("the refresh succeeds");
    let saved = service.store.saved.borrow();
    let stored = saved.first().expect("one saved record");
    assert!(stored.granted_scopes.grants(REQUIRED_SCOPE));
    assert!(stored
        .granted_scopes
        .grants("https://www.googleapis.com/auth/youtube.readonly"));
}

#[test]
fn refresh_response_without_scope_keeps_the_stored_granted_scopes() {
    let response = refresh_response_with(REFRESHED_ACCESS_TOKEN_FIXTURE, None, "");
    let outcomes = vec![not_authorized(), verified_evidence()];
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Ok(response)))
        .with_verifier(QueueVerifier::queued(outcomes));
    service.establish().expect("the refresh succeeds");
    let saved = service.store.saved.borrow();
    let stored = saved.first().expect("one saved record");
    assert!(stored.granted_scopes.grants(REQUIRED_SCOPE));
}

#[test]
fn revoked_refresh_token_maps_to_the_distinct_revoked_error() {
    let server_error = TokenEndpointError::Server {
        code: "invalid_grant".to_string(),
        description: Some("Token has been expired or revoked.".to_string()),
    };
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Err(server_error)))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]));
    let error = service_failure(service.establish());
    let CredentialServiceError::Revoked { code } = &error else {
        panic!("expected the Revoked class, got {error}");
    };
    assert!(code == "invalid_grant");
    let rendered = format!("{error}");
    assert!(rendered.contains("invalid_grant"));
    assert!(rendered.contains("run the authorization command again"));
    assert_eq!(service.store.saved.borrow().len(), 0);
}

#[test]
fn other_refresh_server_codes_stay_in_the_refresh_class() {
    let server_error = TokenEndpointError::Server {
        code: "unauthorized_client".to_string(),
        description: None,
    };
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Err(server_error)))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]));
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Refresh(_)));
    assert!(format!("{error}").contains("unauthorized_client"));
}

#[test]
fn refresh_transport_failure_stays_in_the_refresh_class() {
    let endpoint_error = TokenEndpointError::Transport {
        reason: "connection failed".to_string(),
    };
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Err(endpoint_error)))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]));
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Refresh(_)));
    assert!(format!("{error}").contains("connection failed"));
}

#[test]
fn refresh_parse_failure_stays_in_the_refresh_class() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(Err(TokenEndpointError::ResponseParse)))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]));
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Refresh(_)));
    assert!(format!("{error}").contains("unreadable response"));
}

#[test]
fn persistence_failure_maps_to_the_persistence_class() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(refresh_ok()))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]))
        .failing_save(file_write_error());
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Persistence(_)));
    assert_eq!(service.verifier.calls.get(), 1);
}
