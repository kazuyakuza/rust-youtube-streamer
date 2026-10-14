//! Credential-stage tests (plan §8 TC): the service failure delegates its
//! Display verbatim, the run-side scope gate consumes the FRESH
//! verification evidence, and required-scope evidence passes.

use crate::app::run::error::RunError;
use crate::app::run::runtime_credential_fakes::{
    fixture_client_config, FakeEndpoint, FakeStore, FakeVerifier,
};
use crate::youtube::{ClientConfig, CredentialService};

use super::{authorize_from_service, authorize_scope_evidence};

fn service<'a>(
    store: &'a FakeStore,
    verifier: &'a FakeVerifier,
    endpoint: &'a FakeEndpoint,
    client_config: &'a ClientConfig,
) -> CredentialService<'a> {
    CredentialService {
        store,
        endpoint,
        verifier,
        client_config,
    }
}

#[test]
fn credential_service_failure_maps_to_run_error() {
    let store = FakeStore::absent();
    let verifier = FakeVerifier::granting_required_scope();
    let endpoint = FakeEndpoint;
    let client_config = fixture_client_config();

    // `let Err` instead of `expect_err` (`VerifiedCredentials` is
    // Debug-free by the redaction contract).
    let Err(error) = authorize_from_service(&service(&store, &verifier, &endpoint, &client_config))
    else {
        panic!("an absent credential store must surface as a run error");
    };

    assert!(matches!(error, RunError::Credential(_)));
    assert_eq!(
        error.to_string(),
        "no stored credentials are available; run the authorization command first",
        "the service Display delegates verbatim (delegation contract)"
    );
}

#[test]
fn missing_scope_evidence_maps_to_run_insufficient_scope_error() {
    let store = FakeStore::granting();
    let verifier = FakeVerifier::lacking_required_scope();
    let endpoint = FakeEndpoint;
    let client_config = fixture_client_config();
    let verified = authorize_from_service(&service(&store, &verifier, &endpoint, &client_config))
        .expect("the service itself succeeds (the store grants the scope)");

    let error = authorize_scope_evidence(&verified)
        .expect_err("fresh evidence lacking the scope must fail the run gate");

    assert!(matches!(error, RunError::InsufficientScope));
    assert_eq!(error.exit_code(), 1);
}

#[test]
fn verified_scope_evidence_passes_the_run_gate() {
    let store = FakeStore::granting();
    let verifier = FakeVerifier::granting_required_scope();
    let endpoint = FakeEndpoint;
    let client_config = fixture_client_config();
    let verified = authorize_from_service(&service(&store, &verifier, &endpoint, &client_config))
        .expect("the service succeeds");

    authorize_scope_evidence(&verified).expect("required-scope evidence must pass the gate");
}
