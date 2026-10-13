//! Credential service class and redaction tests (S1, S2, S11, S12,
//! S14-S18): the verify-first success path, the offline scope gate,
//! the load-error classes, verifier-class routing and secret redaction.

use super::credential_service_refresh_tests::{
    answering, file_write_error, refresh_ok, service_failure, REFRESHED_ACCESS_TOKEN_FIXTURE,
    REFRESHED_REFRESH_TOKEN_FIXTURE,
};
use super::credential_service_tests::*;
use super::*;

fn denied_scope() -> Result<VerifiedAuthorization, VerificationError> {
    Err(VerificationError::InsufficientScope {
        reason: "fixture scope denial".to_string(),
    })
}

fn transport_failure() -> Result<VerifiedAuthorization, VerificationError> {
    Err(VerificationError::Transport {
        reason: "fixture transport failure".to_string(),
    })
}

fn store_error_text(path: &str, message: &str) -> CredentialStoreError {
    CredentialStoreError::ReadParse {
        path: std::path::PathBuf::from(path),
        message: message.to_string(),
    }
}

#[test]
fn valid_stored_token_verifies_without_refresh() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_verifier(QueueVerifier::queued(vec![verified_evidence()]));
    let verified = service.establish().expect("a valid stored token verifies");
    assert!(verified.access_token.secret() == ACCESS_TOKEN_FIXTURE);
    assert!(verified
        .authorization
        .granted_scopes()
        .grants(REQUIRED_SCOPE));
    assert_eq!(service.endpoint.refresh_calls.get(), 0);
    assert_eq!(service.store.saved.borrow().len(), 0);
}

#[test]
fn stored_scopes_missing_the_required_scope_fail_before_any_request() {
    let mut stored = stored_refreshable();
    stored.granted_scopes =
        GrantedScopes::from_space_delimited("https://www.googleapis.com/auth/youtube.readonly");
    let service = ServiceUnderTest::loading(stored);
    let error = service_failure(service.establish());
    assert!(matches!(
        error,
        CredentialServiceError::Verification(VerificationError::InsufficientScope { .. })
    ));
    assert_eq!(service.verifier.calls.get(), 0);
}

#[test]
fn absent_credentials_map_to_the_absent_class() {
    let service = ServiceUnderTest::unavailable(CredentialStoreError::ContentNotAvailable);
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Absent));
    assert_eq!(service.verifier.calls.get(), 0);
}

#[test]
fn unreadable_credentials_map_to_the_unreadable_class() {
    let load_errors = [
        store_error_text(
            "credentials/tokens.json",
            "the credential file is not valid JSON",
        ),
        CredentialStoreError::FileOpen {
            path: std::path::PathBuf::from("credentials/tokens.json"),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        },
    ];
    for load_error in load_errors {
        let service = ServiceUnderTest::unavailable(load_error);
        let error = service_failure(service.establish());
        let CredentialServiceError::Unreadable(inner) = &error else {
            panic!("expected the Unreadable class, got {error}");
        };
        assert_eq!(format!("{error}"), format!("{inner}"));
    }
}

#[test]
fn missing_stored_refresh_token_cannot_refresh() {
    let service = ServiceUnderTest::loading(stored_with(ACCESS_TOKEN_FIXTURE, None))
        .with_verifier(QueueVerifier::queued(vec![not_authorized()]));
    let error = service_failure(service.establish());
    assert!(matches!(error, CredentialServiceError::Unrefreshable));
    assert_eq!(service.endpoint.refresh_calls.get(), 0);
}

#[test]
fn insufficient_scope_from_the_verifier_skips_the_refresh_path() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_verifier(QueueVerifier::queued(vec![denied_scope()]));
    let error = service_failure(service.establish());
    assert!(matches!(
        error,
        CredentialServiceError::Verification(VerificationError::InsufficientScope { .. })
    ));
    assert_eq!(service.endpoint.refresh_calls.get(), 0);
}

#[test]
fn transport_from_the_verifier_skips_the_refresh_path() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_verifier(QueueVerifier::queued(vec![transport_failure()]));
    let error = service_failure(service.establish());
    assert!(matches!(
        error,
        CredentialServiceError::Verification(VerificationError::Transport { .. })
    ));
    assert_eq!(service.endpoint.refresh_calls.get(), 0);
}

#[test]
fn verification_failure_after_refresh_maps_to_the_verification_class() {
    let service = ServiceUnderTest::loading(stored_refreshable())
        .with_endpoint(answering(refresh_ok()))
        .with_verifier(QueueVerifier::queued(vec![
            not_authorized(),
            transport_failure(),
        ]));
    let error = service_failure(service.establish());
    assert!(matches!(
        error,
        CredentialServiceError::Verification(VerificationError::Transport { .. })
    ));
    assert_eq!(service.endpoint.refresh_calls.get(), 1);
}

#[test]
fn service_error_output_never_carries_secret_fixtures() {
    for error in fixture_bearing_service_errors() {
        for rendered in [format!("{error}"), format!("{error:?}")] {
            for fixture in [
                ACCESS_TOKEN_FIXTURE,
                REFRESH_TOKEN_FIXTURE,
                CLIENT_SECRET_FIXTURE,
                REFRESHED_ACCESS_TOKEN_FIXTURE,
                REFRESHED_REFRESH_TOKEN_FIXTURE,
            ] {
                assert!(
                    !rendered.contains(fixture),
                    "leaked {fixture} in {rendered}"
                );
            }
        }
    }
}

/// One instance of every `CredentialServiceError` variant, produced
/// from fixture-bearing service contexts where a context exists.
fn fixture_bearing_service_errors() -> Vec<CredentialServiceError> {
    let mut errors = vec![
        CredentialServiceError::Absent,
        CredentialServiceError::Unrefreshable,
        CredentialServiceError::Persistence(file_write_error()),
    ];
    let parse_failure = store_error_text(
        "credentials/tokens.json",
        "the credential file is not valid JSON",
    );
    errors.push(service_failure(
        ServiceUnderTest::unavailable(parse_failure).establish(),
    ));
    errors.push(service_failure(
        ServiceUnderTest::loading(stored_refreshable())
            .with_endpoint(answering(Err(TokenEndpointError::Transport {
                reason: "fixture transport failure".to_string(),
            })))
            .with_verifier(QueueVerifier::queued(vec![not_authorized()]))
            .establish(),
    ));
    errors.push(service_failure(
        ServiceUnderTest::loading(stored_refreshable())
            .with_endpoint(answering(Err(TokenEndpointError::Server {
                code: "invalid_grant".to_string(),
                description: None,
            })))
            .with_verifier(QueueVerifier::queued(vec![not_authorized()]))
            .establish(),
    ));
    errors.push(service_failure(
        ServiceUnderTest::loading(stored_refreshable())
            .with_endpoint(answering(refresh_ok()))
            .with_verifier(QueueVerifier::queued(vec![
                not_authorized(),
                not_authorized(),
            ]))
            .establish(),
    ));
    errors
}
