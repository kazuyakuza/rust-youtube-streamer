//! Fake-transport seam tests: the trait drives with in-memory fakes (no
//! network) and typed server errors propagate unchanged.

use super::super::authorization::REQUIRED_SCOPE;
use super::super::token::parse_token_response;
use super::*;

const ACCESS_FIXTURE: &str = "access_token_fixture";
const REFRESH_FIXTURE: &str = "refresh_token_fixture";
const SECRET_FIXTURE: &str = "client_secret_fixture";

struct FakeTransport {
    body: String,
}

impl FakeTransport {
    /// Both seam operations answer from the same stored body: the fake
    /// proves the trait shape, not Google's protocol details.
    fn respond(&self) -> TokenExchangeResult {
        parse_token_response(&self.body)
    }
}

impl TokenEndpointTransport for FakeTransport {
    fn authorization_code_exchange(&self, _request: &CodeExchangeRequest) -> TokenExchangeResult {
        self.respond()
    }

    fn refresh_token_exchange(&self, _request: &RefreshRequest) -> TokenExchangeResult {
        self.respond()
    }
}

struct ServerErrorFake;

impl ServerErrorFake {
    /// The typed server error both seam operations propagate unchanged.
    fn respond(&self) -> TokenExchangeResult {
        Err(TokenEndpointError::Server {
            code: "invalid_grant".to_string(),
            description: None,
        })
    }
}

impl TokenEndpointTransport for ServerErrorFake {
    fn authorization_code_exchange(&self, _request: &CodeExchangeRequest) -> TokenExchangeResult {
        self.respond()
    }

    fn refresh_token_exchange(&self, _request: &RefreshRequest) -> TokenExchangeResult {
        self.respond()
    }
}

fn code_request() -> CodeExchangeRequest {
    CodeExchangeRequest {
        client_id: "client_id_fixture".to_string(),
        client_secret: ClientSecret::new(SECRET_FIXTURE),
        code: AuthorizationCode::new("authorization_code_fixture"),
        redirect_uri: "http://127.0.0.1:42123".to_string(),
        pkce_verifier: oauth2::PkceCodeVerifier::new(
            "pkce_verifier_fixture_value_43_chars_min".to_string(),
        ),
    }
}

fn refresh_request() -> RefreshRequest {
    RefreshRequest {
        client_id: "client_id_fixture".to_string(),
        client_secret: ClientSecret::new(SECRET_FIXTURE),
        refresh_token: RefreshToken::new(REFRESH_FIXTURE),
    }
}

#[test]
fn fake_transport_returns_the_stored_token_response() {
    let body = format!(
        r#"{{"access_token":"{ACCESS_FIXTURE}","expires_in":3592,"refresh_token":"{REFRESH_FIXTURE}","scope":"{REQUIRED_SCOPE}","token_type":"Bearer"}}"#
    );
    let fake = FakeTransport { body };

    let exchanged = fake
        .authorization_code_exchange(&code_request())
        .expect("stored response drives the seam");
    assert_eq!(exchanged.access_token.secret(), ACCESS_FIXTURE);
    assert_eq!(exchanged.expires_in_seconds, 3592);
    assert!(exchanged.granted_scopes.grants(REQUIRED_SCOPE));

    let refreshed = fake
        .refresh_token_exchange(&refresh_request())
        .expect("stored response drives the seam");
    assert_eq!(
        refreshed.refresh_token.as_ref().expect("present").secret(),
        REFRESH_FIXTURE
    );
}

#[test]
fn fake_transport_propagates_typed_server_error() {
    let fake = ServerErrorFake;
    let error = fake
        .refresh_token_exchange(&refresh_request())
        .expect_err("typed error must propagate");
    let TokenEndpointError::Server { code, description } = &error else {
        panic!("expected Server, got {error}");
    };
    assert_eq!(code, "invalid_grant");
    assert!(description.is_none());
}
