//! Token endpoint body parsing, granted-scope membership and redaction
//! tests: full/refresh-shaped/unknown-field bodies, missing access
//! token, exact case-sensitive scope matching, and secret-free Debug.

use super::*;

const ACCESS_FIXTURE: &str = "access_token_fixture";
const REFRESH_FIXTURE: &str = "refresh_token_fixture";
const FORCE_SSL_SCOPE: &str = "https://www.googleapis.com/auth/youtube.force-ssl";

#[test]
fn parse_full_token_response_with_refresh_token() {
    let body = format!(
        r#"{{
            "access_token": "{ACCESS_FIXTURE}",
            "expires_in": 3592,
            "refresh_token": "{REFRESH_FIXTURE}",
            "scope": "{FORCE_SSL_SCOPE}",
            "token_type": "Bearer"
        }}"#
    );
    let response = parse_token_response(&body).expect("documented sample body parses");
    assert_eq!(response.access_token.secret(), ACCESS_FIXTURE);
    assert_eq!(response.expires_in_seconds, 3592);
    let refresh_token = response
        .refresh_token
        .as_ref()
        .expect("refresh token present");
    assert_eq!(refresh_token.secret(), REFRESH_FIXTURE);
    assert!(response.granted_scopes.grants(FORCE_SSL_SCOPE));
}

#[test]
fn parse_refresh_response_without_refresh_token() {
    let body = format!(
        r#"{{
            "access_token": "{ACCESS_FIXTURE}",
            "expires_in": 3592,
            "scope": "{FORCE_SSL_SCOPE}",
            "token_type": "Bearer"
        }}"#
    );
    let response = parse_token_response(&body).expect("refresh-shaped body parses");
    assert!(response.refresh_token.is_none());
    assert_eq!(response.access_token.secret(), ACCESS_FIXTURE);
}

#[test]
fn parse_ignores_unknown_fields() {
    let body = format!(
        r#"{{
            "access_token": "{ACCESS_FIXTURE}",
            "expires_in": 3592,
            "id_token": "future_identity_field",
            "refresh_token_expires_in": 604800,
            "future_field": {{ "nested": true }},
            "scope": "{FORCE_SSL_SCOPE}",
            "token_type": "Bearer"
        }}"#
    );
    let response = parse_token_response(&body).expect("unknown fields must not break parsing");
    assert_eq!(response.access_token.secret(), ACCESS_FIXTURE);
}

#[test]
fn parse_reports_missing_access_token() {
    let body = r#"{"expires_in": 3592, "token_type": "Bearer"}"#;
    let error = parse_token_response(body).expect_err("missing access_token must fail");
    assert!(matches!(error, TokenEndpointError::ResponseParse));

    let error = parse_token_response("not json at all").expect_err("garbage body must fail");
    assert!(matches!(error, TokenEndpointError::ResponseParse));
}

#[test]
fn granted_scopes_membership_is_exact_and_case_sensitive() {
    let scopes = GrantedScopes::from_space_delimited(
        "https://www.googleapis.com/auth/youtube.readonly https://www.googleapis.com/auth/youtube.force-ssl",
    );
    assert!(scopes.grants(FORCE_SSL_SCOPE));
    assert!(scopes.grants("https://www.googleapis.com/auth/youtube.readonly"));
    assert!(!scopes.grants("https://www.googleapis.com/auth/youtube"));
    assert!(!scopes.grants("HTTPS://WWW.GOOGLEAPIS.COM/AUTH/YOUTUBE.FORCE-SSL"));
    assert!(!scopes.grants(""));

    let empty = GrantedScopes::from_space_delimited("");
    assert!(!empty.grants(FORCE_SSL_SCOPE));
}

#[test]
fn token_response_debug_and_display_reveal_no_secrets() {
    let body = format!(
        r#"{{
            "access_token": "{ACCESS_FIXTURE}",
            "expires_in": 3592,
            "refresh_token": "{REFRESH_FIXTURE}",
            "scope": "{FORCE_SSL_SCOPE}",
            "token_type": "Bearer"
        }}"#
    );
    let response = parse_token_response(&body).expect("documented sample body parses");
    let debug_output = format!("{response:?}");
    assert!(!debug_output.contains(ACCESS_FIXTURE));
    assert!(!debug_output.contains(REFRESH_FIXTURE));
    assert!(debug_output.contains("AccessToken(<redacted>)"));
    assert!(debug_output.contains("RefreshToken(<redacted>)"));
    assert!(debug_output.contains("<redacted>"));
}
