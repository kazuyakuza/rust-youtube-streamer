//! Fake-transport seam tests: the trait drives with in-memory fakes (no
//! network), typed server errors propagate unchanged, and mapped errors
//! never carry secret fixture strings.

use oauth2::basic::BasicErrorResponse;

use super::super::token::parse_token_response;
use super::*;

const ACCESS_FIXTURE: &str = "access_token_fixture";
const REFRESH_FIXTURE: &str = "refresh_token_fixture";
const SECRET_FIXTURE: &str = "client_secret_fixture";

struct FakeTransport {
    body: String,
}

impl TokenEndpointTransport for FakeTransport {
    fn authorization_code_exchange(
        &self,
        _request: &CodeExchangeRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        parse_token_response(&self.body)
    }

    fn refresh_token_exchange(
        &self,
        _request: &RefreshRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        parse_token_response(&self.body)
    }
}

struct ServerErrorFake;

impl TokenEndpointTransport for ServerErrorFake {
    fn authorization_code_exchange(
        &self,
        _request: &CodeExchangeRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        Err(TokenEndpointError::Server {
            code: "invalid_grant".to_string(),
            description: None,
        })
    }

    fn refresh_token_exchange(
        &self,
        _request: &RefreshRequest,
    ) -> Result<TokenResponse, TokenEndpointError> {
        Err(TokenEndpointError::Server {
            code: "invalid_grant".to_string(),
            description: None,
        })
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
    let scope = "https://www.googleapis.com/auth/youtube.force-ssl";
    let body = format!(
        r#"{{"access_token":"{ACCESS_FIXTURE}","expires_in":3592,"refresh_token":"{REFRESH_FIXTURE}","scope":"{scope}","token_type":"Bearer"}}"#
    );
    let fake = FakeTransport { body };

    let exchanged = fake
        .authorization_code_exchange(&code_request())
        .expect("stored response drives the seam");
    assert_eq!(exchanged.access_token.secret(), ACCESS_FIXTURE);
    assert_eq!(exchanged.expires_in_seconds, 3592);
    assert!(exchanged.granted_scopes.grants(scope));

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
    match error {
        TokenEndpointError::Server { code, description } => {
            assert_eq!(code, "invalid_grant");
            assert!(description.is_none());
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn transport_error_display_never_carries_secret_fixtures() {
    let basic_error: BasicErrorResponse =
        serde_json::from_str(r#"{"error":"invalid_grant"}"#).expect("error body parses");
    let errors = [
        map_exchange_error(RequestTokenError::Request(TokenRequestFailure(
            "status code 400".to_string(),
        ))),
        map_exchange_error(RequestTokenError::ServerResponse(basic_error)),
        map_exchange_error(RequestTokenError::Other(
            "unexpected protocol failure".to_string(),
        )),
        TokenEndpointError::ResponseParse,
    ];
    for error in errors {
        let rendered = format!("{error}{error:?}");
        for fixture in [
            ACCESS_FIXTURE,
            REFRESH_FIXTURE,
            SECRET_FIXTURE,
            "authorization_code_fixture",
            "pkce_verifier_fixture_value_43_chars_min",
        ] {
            assert!(!rendered.contains(fixture));
        }
    }
}
