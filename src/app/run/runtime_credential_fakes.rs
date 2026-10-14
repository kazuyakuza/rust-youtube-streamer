//! The credential-side offline fakes of the run runtime (plan D-T3-9;
//! split from `runtime_test_fixtures` per the D-T3-14 line cap): a store,
//! a token endpoint and a verifier implementing the Phase-04 seams, plus
//! the minimal parsed client config the injected-seams runtime requires.
//! Never real-shaped, never reaching Google.

use crate::youtube::{
    parse_client_config, AccessToken, AuthorizationVerifier, ClientConfig, CodeExchangeRequest,
    CredentialStore, CredentialStoreError, GrantedScopes, RefreshRequest, StoredCredentials,
    TokenEndpointError, TokenEndpointTransport, TokenExchangeResult, VerificationError,
    VerifiedAuthorization, REQUIRED_SCOPE,
};

/// Credential-store fake: grants `REQUIRED_SCOPE`; `absent()` fails the
/// load with the typed no-credentials class.
pub(super) struct FakeStore {
    fail_load: bool,
}

impl FakeStore {
    pub(super) fn granting() -> Self {
        Self { fail_load: false }
    }

    pub(super) fn absent() -> Self {
        Self { fail_load: true }
    }
}

impl CredentialStore for FakeStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        if self.fail_load {
            return Err(CredentialStoreError::ContentNotAvailable);
        }
        Ok(granted_stored_credentials())
    }

    fn save(&self, _credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        Ok(())
    }
}

fn granted_stored_credentials() -> StoredCredentials {
    StoredCredentials {
        access_token: AccessToken::new("access-token-fixture"),
        expires_in_seconds: 3600,
        refresh_token: None,
        granted_scopes: GrantedScopes::from_space_delimited(REQUIRED_SCOPE),
    }
}

/// Token-endpoint fake: the happy paths never refresh, so both exchanges
/// answer with the unreadable-response class.
pub(super) struct FakeEndpoint;

impl TokenEndpointTransport for FakeEndpoint {
    fn authorization_code_exchange(&self, _request: &CodeExchangeRequest) -> TokenExchangeResult {
        Err(TokenEndpointError::ResponseParse)
    }

    fn refresh_token_exchange(&self, _request: &RefreshRequest) -> TokenExchangeResult {
        Err(TokenEndpointError::ResponseParse)
    }
}

/// Verifier fake returning scripted scope evidence.
pub(super) struct FakeVerifier {
    evidence: &'static str,
}

impl FakeVerifier {
    pub(super) fn granting_required_scope() -> Self {
        Self {
            evidence: REQUIRED_SCOPE,
        }
    }

    pub(super) fn lacking_required_scope() -> Self {
        Self {
            evidence: "other-scope-fixture",
        }
    }
}

impl AuthorizationVerifier for FakeVerifier {
    fn verify(
        &self,
        _access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        Ok(VerifiedAuthorization::new(
            GrantedScopes::from_space_delimited(self.evidence),
        ))
    }
}

/// A minimal parsed client config (endpoints accepted by the allowlist).
pub(super) fn fixture_client_config() -> ClientConfig {
    parse_client_config(
        r#"{"installed": {
            "client_id": "client-id-fixture",
            "client_secret": "client-secret-fixture",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token"
        }}"#,
    )
    .expect("the fixture client config parses")
}
