//! Redaction tests for the Google token endpoint backend: mapped token
//! endpoint errors never carry secret fixture strings.

use oauth2::basic::BasicErrorResponse;

use super::*;

const ACCESS_FIXTURE: &str = "access_token_fixture";
const REFRESH_FIXTURE: &str = "refresh_token_fixture";
const SECRET_FIXTURE: &str = "client_secret_fixture";

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
