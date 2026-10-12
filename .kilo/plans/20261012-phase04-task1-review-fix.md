# Task 1 Code Review Fix Plan

## Review context

- Branch: `feat/phase04-oauth-credentials`
- Review range: `125430f..851b994`
- Plan authority: `.kilo/plans/20261012-phase04-task1-oauth-design.md`
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md`
- Verdict: **DEVIATIONS FOUND (2 fixable)**

This file lists only the findings that require source changes. All other
judgment calls raised by the implementer were adjudicated against the plan
and accepted; they are summarized in the review report, not here.

---

## Finding 1 — `expect()` panics in non-test library paths

- **Severity:** high
- **Files / lines:**
  - `src/youtube/authorization.rs:54`
  - `src/youtube/transport.rs:56`
  - `src/youtube/transport.rs:136`
- **Description:** Google endpoint constants are parsed with `Url::parse(...).expect(...)`, which panics in library code. The implementation plan §8 explicitly forbids panics/unwrap in library paths, and the project error convention (§0) states "no panics".
- **Required change:** Remove every `expect()`/`unwrap()` from non-test `src/youtube/*.rs` code. Keep the exact URL output and query-parameter order so existing tests pass unchanged.

### 1a — `src/youtube/authorization.rs:54`

Replace the `Url::parse(...).expect(...)` construction with `oauth2::url::form_urlencoded::Serializer` applied to the constant base string. The function signature `authorization_url(...) -> String` must stay unchanged (plan-mandated).

```rust
pub(crate) fn authorization_url(
    client_config: &ClientConfig,
    redirect_uri: &str,
    state: &oauth2::CsrfToken,
    pkce: &PkcePair,
) -> String {
    let query = oauth2::url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_config.client_id())
        .append_pair("response_type", "code")
        .append_pair("scope", REQUIRED_SCOPE)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("state", state.secret())
        .append_pair("code_challenge", pkce.challenge.as_str())
        .append_pair("code_challenge_method", "S256")
        .finish();
    format!("{}?{}", AUTHORIZATION_ENDPOINT, query)
}
```

### 1b — `src/youtube/transport.rs:56` and `:136`

Make endpoint URL construction fallible and propagate parse failures as typed `TokenEndpointError::Transport` reasons.

- Change `production_auth_url()` to return `Result<AuthUrl, TokenEndpointError>`:

```rust
fn production_auth_url() -> Result<AuthUrl, TokenEndpointError> {
    AuthUrl::new(AUTHORIZATION_ENDPOINT.to_string())
        .map_err(|_| TokenEndpointError::Transport {
            reason: "invalid Google authorization endpoint URL".to_string(),
        })
}
```

- Change `GoogleTokenEndpoint::new()` to return `Result<Self, TokenEndpointError>`:

```rust
pub(crate) fn new() -> Result<Self, TokenEndpointError> {
    let token_url = oauth2::TokenUrl::new(TOKEN_ENDPOINT.to_string())
        .map_err(|_| TokenEndpointError::Transport {
            reason: "invalid Google token endpoint URL".to_string(),
        })?;
    Ok(Self {
        agent: ureq::AgentBuilder::new().redirects(0).build(),
        token_url,
    })
}
```

- Update `authorization_code_exchange` and `refresh_token_exchange` to call `production_auth_url()?` instead of `production_auth_url()`.

- `with_token_url` already receives a parsed `oauth2::TokenUrl`, so it remains `-> Self`.

- Note: changing `new() -> Self` to `new() -> Result<Self, TokenEndpointError>` deviates from the plan's signature sketch, but it is required to honor the plan's own "no panics/unwrap in library paths" boundary (§8). Later consumers (Task 3) must handle the `Result`.

### Acceptance check

```text
grep -E "expect\(|unwrap\(" src/youtube/*.rs | grep -v "_tests.rs"
```
returns zero matches, and `cargo check --locked` plus `sh scripts/dev-checks.sh` both exit 0.

---

## Finding 2 — `validate_endpoint` exceeds the max-arguments limit

- **Severity:** medium
- **File / line:** `src/youtube/client_config.rs:70`
- **Description:** `validate_endpoint(declared, expected, field)` takes three arguments, violating `.kilo/rules/max-arguments-per-method.md` (methods/functions must have ≤2 params; otherwise encapsulate in a struct/object).
- **Required change:** Replace the three-argument helper with a two-argument boolean helper and inline the field-specific error in `parse_client_config`.

Replace:

```rust
fn validate_endpoint(
    declared: &Option<String>,
    expected: &str,
    field: &str,
) -> Result<(), ClientConfigError> {
    match declared {
        Some(actual) if actual != expected => Err(ClientConfigError::EndpointMismatch {
            field: field.to_string(),
        }),
        _ => Ok(()),
    }
}
```

with:

```rust
fn endpoint_matches(declared: &Option<String>, expected: &str) -> bool {
    declared.as_ref().map_or(true, |actual| actual == expected)
}
```

Then in `parse_client_config`, replace:

```rust
validate_endpoint(
    &installed.auth_uri,
    AUTHORIZATION_ENDPOINT,
    "installed.auth_uri",
)?;
validate_endpoint(&installed.token_uri, TOKEN_ENDPOINT, "installed.token_uri")?;
```

with:

```rust
if !endpoint_matches(&installed.auth_uri, AUTHORIZATION_ENDPOINT) {
    return Err(ClientConfigError::EndpointMismatch {
        field: "installed.auth_uri".to_string(),
    });
}
if !endpoint_matches(&installed.token_uri, TOKEN_ENDPOINT) {
    return Err(ClientConfigError::EndpointMismatch {
        field: "installed.token_uri".to_string(),
    });
}
```

### Acceptance check

No function in `src/youtube/client_config.rs` has more than two parameters; `cargo check --locked` and `sh scripts/dev-checks.sh` both exit 0; the existing `client_config_tests.rs` tests still pass unchanged.

---

## Notes on accepted judgment calls (no source changes)

- **4-argument `authorization_url`:** Acceptable. The plan §4.3 mandates the exact four-argument signature; this is a documented exception to the max-arguments rule.
- **oauth2 5.0 error mapping:** Acceptable. The plan sketched oauth2 4.x variant names (`Http`), but the selected crate is 5.0. The implementer correctly mapped `ServerResponse`, `Request`, `Parse`, and `Other` to the plan's `TokenEndpointError` variants.
- **Inline `BasicClient` construction:** Acceptable. The plan §4.4 requires building requests through `BasicClient`; the typestate builder makes extraction impractical without generics, so inline construction is justified.
- **`#[allow(dead_code)]` on `mod youtube`:** Acceptable. It silences the staged dead code in the new module without touching the pre-existing `chat`/`streaming` allows; baseline allows remain untouched.

---

## Final verification after fixes

1. `cargo check --locked` exits 0.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` exits 0 with all checks passing.
3. No `expect()`/`unwrap()` remains in non-test `src/youtube/*.rs` files.
4. No function in `src/youtube/*.rs` has more than two parameters (except the plan-mandated `authorization_url`).
5. `git status --porcelain` shows only intended files; no gitignored paths are staged.
