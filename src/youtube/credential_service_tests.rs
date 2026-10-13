//! Credential service test seams (D8): the scripted store/endpoint/
//! verifier fakes, the fixture client config and the service builder
//! shared by the scenario test siblings. Pure fakes; no network/clock.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::OnceLock;

use super::*;
use crate::youtube::client_config::parse_client_config;
use crate::youtube::transport::{CodeExchangeRequest, TokenExchangeResult};

pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access_token_fixture";
pub(super) const REFRESH_TOKEN_FIXTURE: &str = "refresh_token_fixture";
pub(super) const CLIENT_SECRET_FIXTURE: &str = "client_secret_fixture";
pub(super) const FIXTURE_EXPIRES_IN_SECONDS: u64 = 3592;

pub(super) fn stored_with(access_token: &str, refresh_token: Option<&str>) -> StoredCredentials {
    StoredCredentials {
        access_token: AccessToken::new(access_token),
        expires_in_seconds: FIXTURE_EXPIRES_IN_SECONDS,
        refresh_token: refresh_token.map(RefreshToken::new),
        granted_scopes: GrantedScopes::from_space_delimited(REQUIRED_SCOPE),
    }
}

pub(super) fn stored_refreshable() -> StoredCredentials {
    stored_with(ACCESS_TOKEN_FIXTURE, Some(REFRESH_TOKEN_FIXTURE))
}

pub(super) fn verified_evidence() -> Result<VerifiedAuthorization, VerificationError> {
    Ok(VerifiedAuthorization::new(
        GrantedScopes::from_space_delimited(REQUIRED_SCOPE),
    ))
}

pub(super) fn not_authorized() -> Result<VerifiedAuthorization, VerificationError> {
    Err(VerificationError::NotAuthorized {
        reason: "fixture rejection".to_string(),
    })
}

/// The store seam fake (`RefCell` take-once: `&self` trait, non-Clone errors).
#[derive(Default)]
pub(super) struct FakeStore {
    pub(super) loaded: RefCell<Option<StoredCredentials>>,
    pub(super) load_error: RefCell<Option<CredentialStoreError>>,
    pub(super) saved: RefCell<Vec<StoredCredentials>>,
    pub(super) save_error: RefCell<Option<CredentialStoreError>>,
}

impl CredentialStore for FakeStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        if let Some(error) = self.load_error.borrow_mut().take() {
            return Err(error);
        }
        self.loaded
            .borrow_mut()
            .take()
            .ok_or(CredentialStoreError::ContentNotAvailable)
    }

    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        if let Some(error) = self.save_error.borrow_mut().take() {
            return Err(error);
        }
        self.saved.borrow_mut().push(StoredCredentials {
            access_token: credentials.access_token.clone(),
            expires_in_seconds: credentials.expires_in_seconds,
            refresh_token: credentials.refresh_token.clone(),
            granted_scopes: credentials.granted_scopes.clone(),
        });
        Ok(())
    }
}

/// The token endpoint seam fake: one scripted refresh result, captured
/// request facts, call count; a missing result fails the test.
#[derive(Default)]
pub(super) struct FakeEndpoint {
    pub(super) refresh_result: RefCell<Option<TokenExchangeResult>>,
    pub(super) refresh_calls: Cell<usize>,
    pub(super) captured_refresh_token: RefCell<Option<String>>,
    pub(super) captured_client_id: RefCell<Option<String>>,
}

impl TokenEndpointTransport for FakeEndpoint {
    fn authorization_code_exchange(&self, _request: &CodeExchangeRequest) -> TokenExchangeResult {
        panic!("the credential service never exchanges authorization codes");
    }

    fn refresh_token_exchange(&self, request: &RefreshRequest) -> TokenExchangeResult {
        self.refresh_calls.set(self.refresh_calls.get() + 1);
        *self.captured_refresh_token.borrow_mut() =
            Some(request.refresh_token.secret().to_string());
        *self.captured_client_id.borrow_mut() = Some(request.client_id.clone());
        self.refresh_result
            .borrow_mut()
            .take()
            .expect("the refresh was scripted by the test")
    }
}

/// The verifier seam fake: queued outcomes, call count, last verified token.
#[derive(Default)]
pub(super) struct QueueVerifier {
    pub(super) outcomes: RefCell<VecDeque<Result<VerifiedAuthorization, VerificationError>>>,
    pub(super) calls: Cell<usize>,
    pub(super) last_token: RefCell<Option<String>>,
}

impl QueueVerifier {
    pub(super) fn queued(outcomes: Vec<Result<VerifiedAuthorization, VerificationError>>) -> Self {
        Self {
            outcomes: RefCell::new(VecDeque::from(outcomes)),
            calls: Cell::new(0),
            last_token: RefCell::new(None),
        }
    }
}

impl AuthorizationVerifier for QueueVerifier {
    fn verify(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        self.calls.set(self.calls.get() + 1);
        *self.last_token.borrow_mut() = Some(access_token.secret().to_string());
        self.outcomes
            .borrow_mut()
            .pop_front()
            .expect("a queued verify outcome")
    }
}

pub(super) fn fixture_client_config() -> &'static ClientConfig {
    static CONFIG: OnceLock<ClientConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let json = format!(
            r#"{{"installed":{{"client_id":"1234567890.apps.googleusercontent.com","client_secret":"{CLIENT_SECRET_FIXTURE}","auth_uri":"https://accounts.google.com/o/oauth2/v2/auth","token_uri":"https://oauth2.googleapis.com/token"}}}}"#
        );
        parse_client_config(&json).expect("the fixture document parses")
    })
}

/// The service under test (D6 struct literal behind a 2-arg-capped builder).
pub(super) struct ServiceUnderTest {
    pub(super) store: FakeStore,
    pub(super) endpoint: FakeEndpoint,
    pub(super) verifier: QueueVerifier,
}

impl ServiceUnderTest {
    pub(super) fn new(store: FakeStore) -> Self {
        Self {
            store,
            endpoint: FakeEndpoint::default(),
            verifier: QueueVerifier::default(),
        }
    }

    pub(super) fn loading(stored: StoredCredentials) -> Self {
        Self::new(FakeStore {
            loaded: RefCell::new(Some(stored)),
            ..FakeStore::default()
        })
    }

    pub(super) fn unavailable(load_error: CredentialStoreError) -> Self {
        Self::new(FakeStore {
            load_error: RefCell::new(Some(load_error)),
            ..FakeStore::default()
        })
    }

    pub(super) fn with_endpoint(mut self, endpoint: FakeEndpoint) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub(super) fn with_verifier(mut self, verifier: QueueVerifier) -> Self {
        self.verifier = verifier;
        self
    }

    pub(super) fn failing_save(self, error: CredentialStoreError) -> Self {
        *self.store.save_error.borrow_mut() = Some(error);
        self
    }

    pub(super) fn establish(&self) -> Result<VerifiedCredentials, CredentialServiceError> {
        CredentialService {
            store: &self.store,
            endpoint: &self.endpoint,
            verifier: &self.verifier,
            client_config: fixture_client_config(),
        }
        .establish_authorization()
    }
}
