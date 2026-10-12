//! Redaction test for the stored credential type: `Debug` output must
//! never reveal token fixture values. No file I/O happens here — the
//! concrete store is Task 2's work.

use super::*;

#[test]
fn stored_credentials_debug_reveals_no_secrets() {
    let credentials = StoredCredentials {
        access_token: AccessToken::new("access_token_fixture"),
        expires_in_seconds: 3592,
        refresh_token: Some(RefreshToken::new("refresh_token_fixture")),
        granted_scopes: GrantedScopes::from_space_delimited(
            "https://www.googleapis.com/auth/youtube.force-ssl",
        ),
    };
    let rendered = format!("{credentials:?}");
    assert!(!rendered.contains("access_token_fixture"));
    assert!(!rendered.contains("refresh_token_fixture"));
    assert!(rendered.contains("AccessToken(<redacted>)"));
    assert!(rendered.contains("RefreshToken(<redacted>)"));
    assert!(rendered.contains("expires_in_seconds: 3592"));
}
