# Simplification Plan — Phase 04 Task 1 (sub-step 4.3, code-simplifier)

- Date: 2026-10-12
- Branch / commits reviewed: `feat/phase04-oauth-credentials`, baseline `125430f` → HEAD `851b994`
- Scope reviewed (Task 1 changes ONLY): `src/youtube/{mod,error,client_config,token,authorization,transport,credentials,verify}.rs` + the six `*_tests.rs` siblings, `Cargo.toml` dependency block, `src/main.rs` (staging line only)
- Contract authority (frozen surfaces): `.kilo/plans/20261012-phase04-task1-oauth-design.md` §3 file map, §4 type design, §5 redaction rules, §6 test plan; TODO authority `.agent/todos/20261012/20261012-todo-1.md` Task 1
- Implementer profile: **JUNIOR, 50% restriction.** Every step below is atomic with exact code; no judgment calls. Do not reorder steps, do not "improve" anything outside the listed edits.
- Verdict: **SIMPLIFICATION REQUIRED** (2 hard rule violations + 1 visibility defect + bounded duplication).

## 0. Baseline state (verified this session, not assumed)

| Item | Observation | Tool |
| --- | --- | --- |
| Line counts | authorization 119, authorization_tests 156, client_config 101, client_config_tests 112, credentials 30, credentials_tests 23, error 194, error_tests 171, mod 26, token 199, token_tests 108, transport 198, transport_tests 143, verify 32 — **1612 total** | `wc -l` in Alpine VM |
| `#[test]` count | authorization_tests 8, client_config_tests 7, credentials_tests 1, error_tests 2, token_tests 6, transport_tests 3 — **27 total** (MUST stay 27) | `grep -c` in Alpine VM |
| Cap status | No file over the 200-line hard cap, but `token.rs` = 199 and `transport.rs` = 198 (1–2 lines of headroom) | same |
| Rule violations | `error_tests.rs` `each_variant_has_a_stable_display_line` spans lines 15-138 → **body = 122 lines** (> 50-line method cap). `client_config.rs:70` `validate_endpoint` has **3 parameters** (> 2-param cap; the only 3-param free function in non-test `src/` — verified by repo-wide grep over both single-line and multi-line signatures; the sole precedent is a test helper `src/streaming/ffmpeg_tests.rs:70`) | read + grep |
| Visibility defect | `error.rs:15,24,32,45,54` declare `pub enum`; plan §3.1 and `.kilo/rules/prefer-private-members.md` require `pub(crate)` (every other Task 1 type uses `pub(crate)`) | read |
| Duplication | 4 near-identical secret wrappers (`token.rs:16-122`, ~96 lines); duplicated `BasicClient` build (`transport.rs:84-87` vs `106-109`); duplicated token-body JSON literals (`token_tests.rs:13-21` vs `92-100`); duplicated URL-build blocks and a duplicated S256 recomputation (`authorization_tests.rs:41-46`/`70-75`, `59-65`); fake-transport method-body duplication (`transport_tests.rs:18-56`); 4 copy-pasted `match … other => panic!` blocks (`client_config_tests.rs`), repeated JSON literals/`"client_secret_fixture"` strings, `PathBuf::from("credentials/tokens.json")` ×5 (`error_tests.rs`) | read |
| Verified externally (claims used below) | (a) `http::Uri` **1.5.0 has no `as_str()`** → the `request.uri().to_string()` allocation in `transport.rs:69` is REQUIRED, do not "optimize" it (docs.rs `http 1.5.0` Uri page). (b) `oauth2 5.0.0 CsrfToken::secret(&self) -> &String` → the `.as_str()` at `authorization.rs:89` is REQUIRED, do not remove it. (c) `redacted_secret!` macro shape (S4) and `#[derive(Debug)]`-on-`TokenResponse` produce byte-identical output `TokenResponse { access_token: AccessToken(<redacted>), expires_in_seconds: 3592, refresh_token: Some(RefreshToken(<redacted>)) }`; verified by compiling and running an isolated snippet with `rustc --edition 2021 -D warnings` in the pinned `rust:1.82` image. (d) The `let Pattern = &error else { panic!("…{error}") }` idiom compiles and is the established repo style (`src/streaming/ffmpeg_tests.rs:108-117`, `src/app/pipeline/runner_tests.rs:128-131`) | Alpine VM docker run + docs.rs |

Contract surfaces that MUST NOT change anywhere in this plan: module/file map, all type/trait/variant/constant names, all 27 `#[test]` names and their assertion coverage, redaction semantics (wrapper `Debug` strings, no `Display` on wrappers, no secret payloads in error variants), offline determinism (no network, no clock, no real credentials), `Cargo.toml`/`Cargo.lock`, `src/main.rs` `#[allow(dead_code)] mod youtube;` staging line.

---

## 1. Steps

Order: **S1 → S2 → S3** (MUST, independent), then **S5 → S6 → S7 → S8 → S9 → S10** (RECOMMENDED, independent), with **S4 gated on caller approval** (see §3). If S4 is approved, apply it immediately BEFORE S5 (both edit `token.rs`; keep them as two consecutive commits). Run §6 verification after EVERY step; stop and report on any failure. One commit per step, messages given.

### S1 — MUST — Split the 122-line Display test in `error_tests.rs` (≤50-line method rule)

File: `src/youtube/error_tests.rs`. Test name `each_variant_has_a_stable_display_line` is plan-mandated (§6) and MUST remain the `#[test]` fn. Its five per-enum assertion blocks are moved verbatim into five private helpers; nothing is added, removed, or reworded inside any `assert_eq!`.

S1.1 — add this helper directly below the existing `fn permission_denied()` (current lines 10-12), and use it for the five repeated `PathBuf::from("credentials/tokens.json")` occurrences in the blocks being moved (same value, so every expected Display string is unchanged):

```rust
fn credential_path() -> PathBuf {
    PathBuf::from("credentials/tokens.json")
}
```

S1.2 — replace the whole current `each_variant_has_a_stable_display_line` (lines 15-138) with this delegating test plus the five helpers, each helper receiving the verbatim assertion block listed with it:

```rust
#[test]
fn each_variant_has_a_stable_display_line() {
    assert_client_config_display_lines();
    assert_authorization_display_lines();
    assert_token_endpoint_display_lines();
    assert_credential_store_display_lines();
    assert_verification_display_lines();
}

fn assert_client_config_display_lines() {
    // current lines 16-43 verbatim (the four ClientConfigError asserts)
}

fn assert_authorization_display_lines() {
    // current lines 44-61 verbatim (the three AuthorizationError asserts)
}

fn assert_token_endpoint_display_lines() {
    // current lines 62-88 verbatim (the four TokenEndpointError asserts)
}

fn assert_credential_store_display_lines() {
    // current lines 89-116 verbatim (the four CredentialStoreError asserts),
    // with PathBuf::from("credentials/tokens.json") replaced by credential_path()
}

fn assert_verification_display_lines() {
    // current lines 117-137 verbatim (the three VerificationError asserts)
}
```

The `// current lines …` markers above are instructions for the implementer to paste the existing code — they MUST NOT appear in the source file (no commented code). Resulting helper body sizes: 28 / 18 / 27 / 28 / 21 lines — all ≤ 50.

S1.3 — in `source_chain_exposes_io_errors_only` (lines 141-171), replace the four `PathBuf::from("credentials/tokens.json")` occurrences with `credential_path()`. Assertions unchanged.

Commit: `refactor: split the youtube error display test into per-enum helpers`

### S2 — MUST — Remove the 3-parameter helper in `client_config.rs` (≤2-param rule)

File: `src/youtube/client_config.rs`. Behavior preserved exactly: same field paths (`"installed.auth_uri"`, `"installed.token_uri"`), same Google constants compared, same validation ORDER (installed block → endpoints → client_id → client_secret), same error variants.

S2.1 — replace `validate_endpoint` (current lines 70-81) with this private guard type plus a 2-parameter validator:

```rust
/// One endpoint declared in Google's client file, paired with the Google
/// constant it must equal (tampered redirect guard, plan §4.2).
struct DeclaredEndpoint {
    field: &'static str,
    expected: &'static str,
    declared: Option<String>,
}

fn validate_endpoint(endpoint: &DeclaredEndpoint) -> Result<(), ClientConfigError> {
    match &endpoint.declared {
        Some(actual) if actual != endpoint.expected => {
            Err(ClientConfigError::EndpointMismatch {
                field: endpoint.field.to_string(),
            })
        }
        _ => Ok(()),
    }
}
```

S2.2 — replace the body of `parse_client_config` (current lines 48-68) with the destructured version. `auth_uri`/`token_uri` move into the guard structs before `client_id`/`client_secret` are read (partial moves of distinct fields are legal; cargo check confirms):

```rust
pub(crate) fn parse_client_config(json: &str) -> Result<ClientConfig, ClientConfigError> {
    let file: ClientSecretFile =
        serde_json::from_str(json).map_err(|error| ClientConfigError::Deserialization {
            message: error.to_string(),
        })?;
    let installed = file
        .installed
        .ok_or_else(|| missing_field("installed.client_id"))?;
    let InstalledClient {
        client_id,
        client_secret,
        auth_uri,
        token_uri,
    } = installed;
    validate_declared_endpoints(auth_uri, token_uri)?;
    let client_id = required_non_blank(client_id, "installed.client_id")?;
    let client_secret = required_non_blank(client_secret, "installed.client_secret")?;
    Ok(ClientConfig {
        client_id,
        client_secret: ClientSecret::new(client_secret),
    })
}

fn validate_declared_endpoints(
    auth_uri: Option<String>,
    token_uri: Option<String>,
) -> Result<(), ClientConfigError> {
    let endpoints = [
        DeclaredEndpoint {
            field: "installed.auth_uri",
            expected: AUTHORIZATION_ENDPOINT,
            declared: auth_uri,
        },
        DeclaredEndpoint {
            field: "installed.token_uri",
            expected: TOKEN_ENDPOINT,
            declared: token_uri,
        },
    ];
    for endpoint in &endpoints {
        validate_endpoint(endpoint)?;
    }
    Ok(())
}
```

S2.3 — keep `required_non_blank` and `missing_field` exactly as they are (2 params / 1 param, single-use, self-documenting). No serde struct change: `InstalledClient` fields stay `Option<String>` (tolerant parse per plan §4.2/D2 — `client_id`/`client_secret` must stay `Option` so a missing key produces `MissingField`, not a serde error).

Constraints: `validate_endpoint` = 1 param; `validate_declared_endpoints` = 2 params; `parse_client_config` body = 15 lines; max depth 2; `DeclaredEndpoint` is private (prefer-private-members).

Commit: `refactor: encapsulate declared endpoint validation inputs`

### S3 — MUST — Correct `pub` → `pub(crate)` visibility in `error.rs` (plan §3.1 + prefer-private-members)

File: `src/youtube/error.rs`. Five declarations only, no other change in the file:

- line 15 `pub enum ClientConfigError` → `pub(crate) enum ClientConfigError`
- line 24 `pub enum AuthorizationError` → `pub(crate) enum AuthorizationError`
- line 32 `pub enum TokenEndpointError` → `pub(crate) enum TokenEndpointError`
- line 45 `pub enum CredentialStoreError` → `pub(crate) enum CredentialStoreError`
- line 54 `pub enum VerificationError` → `pub(crate) enum VerificationError`

All consumers live inside `src/youtube/` (imported as `super::error::…`) and in the `#[cfg(test)]` sibling modules, so the crate still compiles with zero signature/Display/variant changes. Do NOT change any `impl` block, variant, message string, or the `Error`/`source` impls.

Commit: `refactor: scope youtube oauth error enums to the crate`

### S4 — RECOMMENDED (GATED — caller decision, see §3) — Collapse the four duplicated secret wrappers in `token.rs`

File: `src/youtube/token.rs`. Contract preserved: the four type names, `new(impl Into<String>)`, `secret() -> &str`, `#[derive(Clone)]`, no `Display`, and the exact `Debug` strings `AccessToken(<redacted>)` / `RefreshToken(<redacted>)` / `AuthorizationCode(<redacted>)` / `ClientSecret(<redacted>)` (plan §4.1/§5; asserted by `token_tests.rs`, `client_config_tests.rs`, `credentials_tests.rs`).

S4.1 — replace lines 16-122 (the four wrapper struct/impl/Debug blocks) with:

```rust
macro_rules! redacted_secret {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Clone)]
        pub(crate) struct $name {
            secret: String,
        }

        impl $name {
            pub(crate) fn new(secret: impl Into<String>) -> Self {
                Self {
                    secret: secret.into(),
                }
            }

            /// Raw value; consumed only when building requests, never
            /// formatted or logged.
            pub(crate) fn secret(&self) -> &str {
                &self.secret
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

redacted_secret!(
    /// Access token for Google API requests. Debug never reveals the value;
    /// Display is intentionally not implemented.
    AccessToken
);

redacted_secret!(
    /// Refresh token for long-term credential reuse. Debug never reveals the
    /// value; Display is intentionally not implemented.
    RefreshToken
);

redacted_secret!(
    /// Single-use authorization code returned by the callback. Debug never
    /// reveals the value; Display is intentionally not implemented.
    AuthorizationCode
);

redacted_secret!(
    /// OAuth client secret from Google's downloadable client file. Debug never
    /// reveals the value; Display is intentionally not implemented.
    ClientSecret
);
```

S4.2 — nothing else in the file changes in this step (`use std::fmt;` stays: it is used by the macro body and by `TokenResponse`'s Debug until S5). If the caller rejects S4, skip it entirely and apply S5 only — token.rs then ends at ~186 lines, still under the 200-line cap.

Verified before writing this plan: the `($(#[$attribute:meta])* $name:ident)` matcher with doc comments passed to the invocation compiles under `rustc --edition 2021 -D warnings` (pinned 1.82 toolchain) and emits exactly the `TypeName(<redacted>)` strings listed above.

Commit: `refactor: generate the redacted secret wrappers from one macro`

### S5 — RECOMMENDED — `token.rs`: derive `TokenResponse` Debug and simplify the scope default

File: `src/youtube/token.rs`. Two independent edits.

S5.1 — replace the hand-written `impl fmt::Debug for TokenResponse` (current lines 148-158) with a derive, i.e. change lines 138-146 to:

```rust
/// Application-owned, redaction-safe token data built from a parsed
/// token endpoint body. `Debug` never reveals secrets: the token wrappers
/// redact themselves (plan §5).
#[derive(Debug)]
pub(crate) struct TokenResponse {
    pub(crate) access_token: AccessToken,
    /// Remaining lifetime of `access_token` in seconds, as reported by Google.
    pub(crate) expires_in_seconds: u64,
    pub(crate) refresh_token: Option<RefreshToken>,
    pub(crate) granted_scopes: GrantedScopes,
}
```

and delete the whole `impl fmt::Debug for TokenResponse { … }` block. The derive is byte-identical to the deleted impl (same struct name, same field names, same declaration order) — verified in the 1.82 toolchain (§0 item c) — so `token_response_debug_and_display_reveal_no_secrets` keeps passing unchanged.

S5.2 — in `parse_token_response` (current lines 183-195), replace the double-closure `map_or_else` with one explicit default, keeping identical semantics (absent `scope` → empty granted set, never an error — plan D1):

```rust
pub(crate) fn parse_token_response(body: &str) -> Result<TokenResponse, TokenEndpointError> {
    let parsed: TokenEndpointBody =
        serde_json::from_str(body).map_err(|_| TokenEndpointError::ResponseParse)?;
    let scope_text = parsed.scope.unwrap_or_default();
    Ok(TokenResponse {
        access_token: AccessToken::new(parsed.access_token),
        expires_in_seconds: parsed.expires_in,
        refresh_token: parsed.refresh_token.map(RefreshToken::new),
        granted_scopes: GrantedScopes::from_space_delimited(&scope_text),
    })
}
```

This mirrors the shape already used by `transport.rs::to_token_response` (build the space-delimited text, hand it to `from_space_delimited`), so the two TokenResponse producers now read the same way.

Commit: `refactor: derive the token response debug and simplify scope parsing`

### S6 — RECOMMENDED — `transport.rs`: extract the duplicated `BasicClient` construction

File: `src/youtube/transport.rs`. The identical 4-line client build at lines 84-87 and 106-109 becomes one method; request/verifier/redirect handling stays per method. Behavior identical (same `ClientId` string, same secret conversion, same auth URI constant, same `self.token_url.clone()`).

S6.1 — add this method to the existing `impl GoogleTokenEndpoint` block (after `execute_http`, current line 79):

```rust
/// Both exchanges use the same client identity: caller-supplied id and
/// secret, the fixed Google authorization URI, and this endpoint's token
/// URL (test-substitutable via `with_token_url`).
fn token_client(&self, client_id: &str, client_secret: &ClientSecret) -> BasicClient {
    BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(oauth2_client_secret(client_secret))
        .set_auth_uri(production_auth_url())
        .set_token_uri(self.token_url.clone())
}
```

(`&self` + 2 parameters follows the receiver-excluded convention already used in the repo, e.g. `src/renderer/text.rs:55`.)

S6.2 — in `authorization_code_exchange`, replace lines 84-87 with:

```rust
        let client = self.token_client(&request.client_id, &request.client_secret);
```

S6.3 — in `refresh_token_exchange`, replace lines 106-109 with the identical single line above. Leave `code`, `verifier`, `redirect_url`, `http_client`, the `.request(&http_client)` chains, `to_token_response`, `map_exchange_error`, `bridge_error`, `map_ureq_response`, `join_scopes`, `production_auth_url`, `oauth2_client_secret` and `TokenRequestFailure` untouched.

S6.4 — DO NOT "remove the `to_string()` allocation" at current line 69 (`request.uri().to_string().as_str()`): `http::Uri` 1.5.0 exposes no `as_str()` (§0 item a), so the `String` is required by `ureq::Agent::post(&str)`.

Commit: `refactor: share the oauth client build between both token exchanges`

### S7 — RECOMMENDED — `client_config_tests.rs`: replace the four copy-pasted `match … panic!` blocks with the repo let-else idiom

File: `src/youtube/client_config_tests.rs`. Test names, JSON documents (values AND layout), and every assertion stay identical; scenario count stays 7. The JSON literals are deliberate readable fixtures and stay untouched — see §4 for why a `format!`-based fixture builder was rejected here.

S7.1 — `parse_reports_missing_installed_client_id` (lines 45-63). Both 6-line `match` blocks become the let-else idiom (verified compilable pattern, §0 item d; the same idiom is already used in `src/streaming/ffmpeg_tests.rs:108-117`), keeping each `expect_err` message and each field-path assertion. Final body:

```rust
#[test]
fn parse_reports_missing_installed_client_id() {
    let error = parse_client_config("{}").expect_err("missing installed block must fail");
    let ClientConfigError::MissingField { field } = &error else {
        panic!("expected MissingField, got {error}");
    };
    assert_eq!(field, "installed.client_id");

    let json = r#"{"installed": {"client_secret": "client_secret_fixture"}}"#;
    let error = parse_client_config(json).expect_err("missing client_id must fail");
    let ClientConfigError::MissingField { field } = &error else {
        panic!("expected MissingField, got {error}");
    };
    assert_eq!(field, "installed.client_id");
}
```

S7.2 — `parse_reports_blank_client_secret` (lines 65-80): keep the JSON literal and the blank `"   "` value exactly as they are; replace only the `match` block with:

```rust
    let ClientConfigError::BlankField { field } = &error else {
        panic!("expected BlankField, got {error}");
    };
    assert_eq!(field, "installed.client_secret");
```

S7.3 — `parse_rejects_foreign_token_uri` (lines 82-98): keep the JSON literal (including `"token_uri": "https://evil.invalid/token"`) exactly as it is; replace only the `match` block with:

```rust
    let ClientConfigError::EndpointMismatch { field } = &error else {
        panic!("expected EndpointMismatch, got {error}");
    };
    assert_eq!(field, "installed.token_uri");
```

S7.4 — `google_shaped_json`, `parse_reads_installed_client_fields`, `parse_preserves_unknown_fields`, `parse_rejects_invalid_json`, `client_config_debug_never_reveals_secret`: NO edits.

Note (S2 interaction): the endpoint-tampering test still reaches `EndpointMismatch` because `validate_declared_endpoints` runs before the `client_id`/`client_secret` checks — the order preserved by S2.

Commit: `test: use the let-else idiom in the client config error cases`

### S8 — RECOMMENDED — `token_tests.rs`: share the documented token body, reuse the scope constant

File: `src/youtube/token_tests.rs`. Six tests stay six; each keeps its own assertions.

S8.1 — replace line 9 (`const FORCE_SSL_SCOPE: &str = "https://www.googleapis.com/auth/youtube.force-ssl";`) with

```rust
use super::super::authorization::REQUIRED_SCOPE as FORCE_SSL_SCOPE;
```

so the fixture stops restating the production constant. `granted_scopes_membership_is_exact_and_case_sensitive` and the negative-case literals (bare `youtube` scope, uppercase variant, `""`) stay exactly as they are; the scope string itself remains pinned by `authorization_tests.rs` (lines 82-85), which is the single authority for the literal value.

S8.2 — add the shared body helper below the constants:

```rust
/// Google's documented token response body for an installed application
/// (plan §1.1): access token, lifetime, refresh token, scope, Bearer type.
fn documented_token_body() -> String {
    format!(
        r#"{{
            "access_token": "{ACCESS_FIXTURE}",
            "expires_in": 3592,
            "refresh_token": "{REFRESH_FIXTURE}",
            "scope": "{FORCE_SSL_SCOPE}",
            "token_type": "Bearer"
        }}"#
    )
}
```

S8.3 — `parse_full_token_response_with_refresh_token` (lines 11-31) and `token_response_debug_and_display_reveal_no_secrets` (lines 90-107) now start with `let body = documented_token_body();`; delete their duplicated literals. All their `assert`/`expect` lines stay verbatim.

S8.4 — `parse_refresh_response_without_refresh_token` (33-46) and `parse_ignores_unknown_fields` (48-63) keep their own distinct bodies (they are the deliberate shape variations), unchanged except the shared `FORCE_SSL_SCOPE` name from S8.1.

Commit: `test: deduplicate the youtube token response fixtures`

### S9 — RECOMMENDED — `transport_tests.rs`: one respond path per fake, alias + constant reuse

File: `src/youtube/transport_tests.rs`. Three tests stay three; fakes keep their names (`FakeTransport`, `ServerErrorFake`) — do NOT merge them (two distinctly named fakes read better than one enum-driven fake).

S9.1 — add alongside the existing import block (lines 5-8):

```rust
use super::super::authorization::REQUIRED_SCOPE;
```

S9.2 — `FakeTransport` (lines 14-32): add the shared path and shorten both trait methods, using the module's existing private alias `TokenExchangeResult` (defined at `transport.rs:34`, reachable through `use super::*`):

```rust
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
```

S9.3 — `ServerErrorFake` (lines 34-56): same shape with a `fn respond(&self) -> TokenExchangeResult` returning `Err(TokenEndpointError::Server { code: "invalid_grant".to_string(), description: None })`, and both trait methods calling `self.respond()`.

S9.4 — in `fake_transport_returns_the_stored_token_response` (lines 78-100): delete the local `let scope = "https://www.googleapis.com/auth/youtube.force-ssl";` (line 80) and use `REQUIRED_SCOPE` in the body `format!` and in `assert!(exchanged.granted_scopes.grants(REQUIRED_SCOPE));`. All other assertions unchanged.

S9.5 — in `fake_transport_propagates_typed_server_error` (lines 102-115): replace the `match … other => panic!` block with the let-else idiom:

```rust
    let TokenEndpointError::Server { code, description } = &error else {
        panic!("expected Server, got {error}");
    };
    assert_eq!(code, "invalid_grant");
    assert!(description.is_none());
```

S9.6 — `code_request`, `refresh_request`, `transport_error_display_never_carries_secret_fixtures` (including its fixture string list and the double loop) stay unchanged — the redaction assertion coverage must not be touched.

Commit: `test: deduplicate the youtube fake transports`

### S10 — RECOMMENDED — `authorization_tests.rs`: share the URL build, reuse the PKCE fixture

File: `src/youtube/authorization_tests.rs`. Eight tests stay eight.

S10.1 — add after `fixture_state()` (current lines 35-37):

```rust
/// The authorization URL built from the fixtures, used by the two URL tests.
fn built_authorization_url() -> String {
    authorization_url(
        &fixture_client_config(),
        "http://127.0.0.1:42123",
        &fixture_state(),
        &rfc7636_pkce(),
    )
}
```

S10.2 — `authorization_url_encodes_all_parameters` (lines 39-66): `let url = built_authorization_url();` and keep the exact-URL `assert_eq!` unchanged. Then replace the duplicated S256 recomputation (lines 59-65) with the fixture reuse — same guarantee (`challenge == S256(verifier)` against the RFC 7636 vector):

```rust
    assert_eq!(rfc7636_pkce().challenge.as_str(), RFC7636_CHALLENGE);
```

S10.3 — `authorization_url_uses_the_force_ssl_scope` (lines 68-86): `let url = built_authorization_url();`; the `Url::parse`/`query_pairs` lookup and both asserts (including the literal `REQUIRED_SCOPE` value pin, lines 82-85) stay verbatim.

S10.4 — `callback_parses_authorized_query` (106-118), `callback_parses_provider_declined` (131-139): replace the two-variant `match … panic!` blocks with the let-else idiom **binding by reference (`&callback`) exactly as written below** so the `else` branch may render the original value (the repo's proven pattern, §0 item d), keeping every assertion:

```rust
    let AuthorizationCallback::Authorized { code, state } = &callback else {
        panic!("expected Authorized, got {callback:?}");
    };
    assert_eq!(code.secret(), "code_fixture");
    assert_eq!(state, "state_fixture");
```

```rust
    let AuthorizationCallback::Declined { reason } = &callback else {
        panic!("expected Declined, got {callback:?}");
    };
    assert_eq!(reason, "access_denied");
```

S10.5 — `callback_reports_missing_parameter` (141-156): both cases use let-else on `AuthorizationError::MissingParameter { parameter }` with `assert_eq!(parameter, "state")` / `assert_eq!(parameter, "code")` unchanged. `callback_rejects_state_mismatch` (120-129) and `generate_pkce_produces_valid_pair` / `generate_state_produces_random_value` stay byte-for-byte unchanged — the state-mismatch redaction asserts are contractual.

Commit: `test: share the youtube authorization url fixture`

---

## 2. Projected state after S1-S10

| File | Before | After (with S4) | After (S4 rejected) |
| --- | --- | --- | --- |
| token.rs | 199 | ~118 | ~186 |
| transport.rs | 198 | ~190 | ~190 |
| error.rs | 194 | 194 (S3 only) | 194 |
| error_tests.rs | 171 | ~182 (rule-compliant bodies) | ~182 |
| authorization_tests.rs | 156 | ~142 | ~142 |
| transport_tests.rs | 143 | ~126 | ~126 |
| client_config_tests.rs | 112 | ~104 | ~104 |
| token_tests.rs | 108 | ~96 | ~96 |
| client_config.rs | 101 | ~112 (rule-compliant params) | ~112 |
| others untouched by S1-S10 (authorization 119, credentials 30, credentials_tests 23, mod 26, verify 32) | 230 | 230 | 230 |
| **youtube total** | **1612** | **~1494** | **~1562** |

Invariant checks: 27 `#[test]` functions before and after; zero files > 200 lines; zero function bodies > 50 lines; zero free functions with > 2 parameters (the plan-mandated 4-parameter `authorization_url` is exempt — see §4); max depth ≤ 2; no commented-out code; no new `#[allow(...)]` anywhere in `src/youtube/` (the single staging allow in `src/main.rs` stays the only one); no new dependency; `Cargo.toml`/`Cargo.lock`/`src/main.rs`/`Cargo` untouched by every step.

## 3. Caller decision required before S4 (macro introduction)

S4 is the only step that adds a language construct with **no precedent in `src/`** (`git grep macro_rules -- src` → zero hits today; a previous simplification plan also rejected macro extraction on taste grounds). It is behavior-preserving and toolchain-verified, but it is a *style* decision, not a rule fix, and the junior implementer has no authority to make it. **Caller/reviewer must instruct "apply S4" or "skip S4" before dispatching.** Everything else in this plan is either a rule-violation fix (S1, S2, S3) or a mechanical local dedup (S5-S10) and may proceed regardless.

## 4. Considered and REJECTED (do NOT change — recorded to prevent helper churn)

- A `format!`-based fixture builder for `client_config_tests.rs` JSON documents (wrapping an `"installed"` member list, reusing `CLIENT_ID`/`SECRET_FIXTURE`/the endpoint constants) — rejected after drafting it: nesting `{"installed": {…}}` inside interpolated raw strings forces `{{`/`}}` double-escaping in every test, which is harder to read than the current plain JSON literals, and it would rewrite fixtures that already serve as readable Google-shape documentation. S7 keeps the literals and dedups only the assertion blocks. If a future task adds more client-config cases, revisit with `String::from(r#"{"installed": {"#)` + `push_str` composition instead of `format!`.
- `transport.rs:69` `request.uri().to_string().as_str()` — allocation is REQUIRED: `http` 1.5.0 `Uri` has no `as_str()` (§0 item a). Could be written `&request.uri().to_string()`; cosmetic only, skip.
- `authorization.rs:89` `.as_str()` after `expected_state.secret()` — REQUIRED: `oauth2 5.0 CsrfToken::secret()` returns `&String` (§0 item b).
- `authorization.rs` `authorization_url` 4 parameters — mandated verbatim by the design plan §4.3, together with the `REQUIRED_SCOPE`/endpoint constants, `PkcePair`, `generate_pkce`, `generate_state`, `AuthorizationCallback` and `parse_authorization_callback` shapes. Rule deviation owned by the design plan/reviewer, not the simplifier. Do not introduce a request struct.
- `authorization.rs` `REVOCATION_ENDPOINT` and `transport.rs` `GoogleTokenEndpoint::with_token_url` are currently unused — plan-mandated surfaces (plan §4.3; plan §9 checklist item 4). Keep; the module-level staging allow already silences them.
- `TokenEndpointBody` `#[derive(Debug)]` (token.rs:126) prints its raw `access_token` field if ever formatted. Today the value never escapes `parse_token_response` (function-local), so no leak exists or is asserted; removing/altering the derive contradicts plan §4.1. **Flagged to caller/reviewer as a Task 2/3 hardening observation, NOT part of this plan.**
- Merging `FakeTransport` and `ServerErrorFake` into one enum-driven fake — abstraction grows while clarity drops; S9 dedups their bodies instead.
- A shared test-fixture module for the repeated `access_token_fixture` / `client_secret_fixture` strings across test files — would add a file to the plan-§3 fixed module map (scope/architecture decision). Out of scope here; propose to the caller only if a later task wants it.
- `error.rs` `CredentialStoreError` FileOpen/FileWrite arm dedup — the verbs differ ("open"/"write"), so any shared helper needs 3 parameters (worse under the ≤2-param rule). Leave.
- `error.rs` ClientConfigError shared message prefix — the four messages share `"invalid OAuth client configuration: "`; extracting a constant would require writing the prefix separately from the `match` arms, restructuring five contractual Display lines for ~2 saved lines and no rule gain. Leave (message strings are snapshot-tested).
- `join_scopes` `Vec` + `join` and the scopes `String`→`split_whitespace` round trip in `transport.rs::to_token_response` — plan §4.4 explicitly specifies the space-joined hand-off to `GrantedScopes::from_space_delimited`; the two extra small allocations are contractual and negligible (one response per exchange). Leave.
- `GrantedScopes` `Vec<String>` → `Vec<Box<str>>`/set, `write_optional_description` inlining, `use std::{error::Error, fmt};` style unification, `mod.rs` doc "TODO Task 1 scope" phrasing — cosmetic noise, no rule or clarity gain. Leave.
- `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `credentials.rs`, `verify.rs`, `credentials_tests.rs`, `mod.rs` — reviewed, already minimal; NO edits in this plan.

## 5. Out of scope for this step (report to caller, not planned here)

- Nothing rises to a medium/big redesign: all ten steps are local, behavior-preserving edits inside `src/youtube/`. No TODO re-scope and no architector routing needed. The only decision items are the S4 macro approval (§3) and the `TokenEndpointBody` Debug observation (§4).
- Task 1 sub-step 4.3 does not run the plan-deviation review (concurrent code-reviewer owns it) and does not implement these edits.

## 6. Verification protocol (Alpine VM Docker workflow — every step)

Always `alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`. Fast loop after each step:

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt` (reflow only; inspect that the diff is confined to the step's file)
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked`

Stop conditions (report to caller, do not improvise): any compile/clippy/test failure, a test count other than 27 in the youtube module, a redaction assertion failure, or a clippy lint triggered by the S4 macro (then revert S4 and continue with S5 — the gated fallback).

Final gate after the last step (must print four `PASS` lines + `ALL CHECKS PASSED`, exit 0):

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`

Post-gate hygiene (gitignore compliance + staging):

- `git grep -n "#\[allow(" -- src` shows exactly one line (`src/main.rs` youtube staging).
- `git status --porcelain` shows only `src/youtube/**` files of the current step; stage explicitly per step (never `git add -A`), no `Cargo.lock`/`logs/`/`dist/`/`target/` paths, no tmp files created (host or container mounted volume).
- Record the final per-file `wc -l` and `grep -c "#\[test\]"` numbers in the implementing step's report for the reviewer.

## 7. This step's status

- DONE: full read of all 14 Task 1 sources + `Cargo.toml` + `main.rs` touch; rule/cap/duplication audit; external claims verified in the pinned 1.82 toolchain and against current crate docs; this plan written.
- NOT DONE (by design): no source file modified, nothing committed, no test run of the proposed edits, no deviation review, no S4 approval. Implementation belongs to the next workflow sub-step.
