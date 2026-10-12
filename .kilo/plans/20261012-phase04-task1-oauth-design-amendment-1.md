# Plan Amendment 1 — Task 1 review cycle (sub-step 4.3-fix), transport cap breach

- Date: 2026-10-12
- Base plan: `.kilo/plans/20261012-phase04-task1-oauth-design.md` (unchanged except where amended here)
- Branch: `feat/phase04-oauth-credentials`, HEAD `ae4e866` (tree clean)
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — Task 1 only
- Applies to: reviewer sub-step 4.3-fix; simplifier steps S6–S10

---

## A0. Why this amendment exists (record, not action)

- Review-fix commit `3a1c8f4` made endpoint parsing fallible and removed every `expect()` from library paths (plan §8: "no panics in library paths"). That mandatory panic removal added fallible URL construction (`TokenUrl::new`/`RedirectUrl::new` mapping into `TokenEndpointError::Transport`) and grew `src/youtube/transport.rs` to 203 lines — over the 200-line hard cap (`.kilo/rules/max-lines-per-file.md`).
- The pending simplifier step S6 (extract `token_client`) was empirically verified to leave a fmt-stable floor of ~209–212 lines: dedup removes only the duplicated 4-line client-construction pair, while rustfmt re-wrapping of the moved fallible blocks restores the mass. The cap is unreachable via dedup alone inside the currently allowed scope.
- Decision (this amendment): split the Google token-endpoint backend out of `transport.rs` into a NEW file `src/youtube/google_token_endpoint.rs`, leaving the seam contract in `transport.rs`. The module already registered one file per concern (`error.rs`, `token.rs`, …); a second backend file under `src/youtube/` is an in-scope structural dedup, consistent with plan §3's per-file focus budget.

## A1. Exact movement map (anchored to `transport.rs` @ HEAD `ae4e866`, 203 lines)

STAY in `src/youtube/transport.rs` (the seam contract; target ≈ 45 lines incl. test-mod decl):

| Item | Current lines |
| --- | --- |
| Module doc comment — REWRITTEN to describe only the seam (drop the ureq-backend sentence, redirect readers to `google_token_endpoint`) | 1–3 |
| `use` lines needed by the contract only: `super::token::{AccessToken→REMOVED, AuthorizationCode, ClientSecret, RefreshToken, TokenResponse}`, `super::error::TokenEndpointError` | 16–18, 15 — trimmed |
| `pub(crate) struct CodeExchangeRequest` (with `oauth2::PkceCodeVerifier` field, qualified path) | 20–26 |
| `pub(crate) struct RefreshRequest` | 28–32 |
| `type TokenExchangeResult = Result<TokenResponse, TokenEndpointError>;` | 34 |
| `pub(crate) trait TokenEndpointTransport` (both methods) | 36–40 |
| `#[cfg(test)] #[path = "transport_tests.rs"] mod transport_tests;` | 201–203 |

MOVE to NEW `src/youtube/google_token_endpoint.rs` (target ≈ 160–185 lines incl. imports and test-mod decl):

| Item | Current lines |
| --- | --- |
| `GoogleTokenEndpoint` struct + its doc comment (agent + resolved `token_url` fields) | 42–47 |
| `type TokenHttpResult = Result<oauth2::HttpResponse, TokenRequestFailure>;` | 49 |
| `impl GoogleTokenEndpoint` ( `new`, `#[cfg(test)] with_token_url`, `execute_http` ) | 51–84 |
| `impl TokenEndpointTransport for GoogleTokenEndpoint` (both exchange methods) | 86–123 |
| `struct TokenRequestFailure(String)` + `Display` + `Error` impls + doc comment | 125–136 |
| `fn production_auth_url` | 138–142 |
| `fn oauth2_client_secret` | 144–146 |
| `fn bridge_error` | 148–150 |
| `fn map_ureq_response` | 152–159 |
| `fn map_exchange_error` | 161–173 |
| `fn join_scopes` | 175–181 |
| `fn to_token_response` | 183–199 |

No item is renamed, re-typed, or re-ordered; bodies move verbatim. The oauth2/ureq `use` block (lines 5–14: `BasicClient, BasicErrorResponse, BasicTokenResponse, BasicTokenType, AuthUrl, AuthorizationCode as Oauth2AuthorizationCode, ClientId, ClientSecret as Oauth2ClientSecret, RedirectUrl, RefreshToken as Oauth2RefreshToken, RequestTokenError, TokenResponse as _, plus ureq`) moves to the backend file verbatim; `transport.rs` keeps nothing of it except the qualified `oauth2::PkceCodeVerifier` in `CodeExchangeRequest`.

## A2. Import / visibility adjustments

- `google_token_endpoint.rs` header: `use std::{error::Error, fmt};`, the moved oauth2/ureq `use` block, then:
  - `use super::authorization::{AUTHORIZATION_ENDPOINT, TOKEN_ENDPOINT};`
  - `use super::error::TokenEndpointError;`
  - `use super::token::{AccessToken, ClientSecret, GrantedScopes, RefreshToken, TokenResponse};` (AuthorizationCode stays a qualified `oauth2` alias usage inside the file; add it to this `use` list only if the compiler requires it — it does NOT, the moved code uses `Oauth2AuthorizationCode`)
  - `use super::transport::{CodeExchangeRequest, RefreshRequest, TokenEndpointTransport, TokenExchangeResult};`
- Visibility: unchanged — everything keeps its current `pub(crate)`; new file locals (`TokenRequestFailure`, `production_auth_url`, `oauth2_client_secret`, `bridge_error`, `map_ureq_response`, `join_scopes`, `to_token_response`, `TokenHttpResult`) become **private** (they were already private; file-local status is preserved by keeping the same module level — they are NOT `pub`).
- `src/youtube/mod.rs`: insert `mod google_token_endpoint;` between `mod error;` and `mod token;` (alphabetical). No re-export added (§3.1 policy: none until the first external consumer). No `#[allow(`.
- No `main.rs`, `Cargo.toml`, `Cargo.lock` change.

## A3. Test-file decision (explicit)

- `transport_tests.rs` keeps the 3 test NAMES, but only the two fake-seam tests belong with the trait: `fake_transport_returns_the_stored_token_response` and `fake_transport_propagates_typed_server_error` STAY in `transport_tests.rs` unchanged — they exercise the `TokenEndpointTransport` seam via fakes and are trait-scope.
- The third test `transport_error_display_never_carries_secret_fixtures` depends on `map_exchange_error` + `TokenRequestFailure`, which MOVE to the backend file → it MUST move. Create NEW `src/youtube/google_token_endpoint_tests.rs` holding exactly that one test, name unchanged, with `use oauth2::basic::BasicErrorResponse;`, `use super::*;`, and the fixture consts `ACCESS_FIXTURE`/`REFRESH_FIXTURE`/`SECRET_FIXTURE` (fixture strings verbatim from `transport_tests.rs` lines 10–12; drop the consts the moved test does not reference). `transport_tests.rs` then simplifies to `use super::super::token::parse_token_response; use super::*;` keeping only const names its 2 remaining tests reference.
- Wiring: `google_token_endpoint.rs` gains the standard foot block (mirrors current lines 201–203):
  `#[cfg(test)]`
  `#[path = "google_token_endpoint_tests.rs"]`
  `mod google_token_endpoint_tests;`
- Zero new test names; total count 27 youtube test names preserved; names unchanged count as preserved.

## A4. Order of application for the implementer (both commits, then S7–S10 untouched)

1. **Split commit** — create `google_token_endpoint.rs` + `google_token_endpoint_tests.rs` per A1–A3; edit `transport.rs`, `transport_tests.rs`, `mod.rs` per A2–A3. Verify in container: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked youtube` exit 0 (names unchanged) and `cargo fmt --check` exit 0 (run `cargo fmt` inside the container first and fold into the commit if it re-wraps). Stage ONLY `src/youtube/{transport.rs,transport_tests.rs,google_token_endpoint.rs,google_token_endpoint_tests.rs,mod.rs}` after `git status --porcelain` confirmation. Commit message: `refactor: split the google token endpoint backend from the transport seam`.
2. **S6 re-run, Option B shape, inside the new backend file** (two-step dedup of the identical builder pair at former lines 88–91 and 110–113):
   - Add to `impl GoogleTokenEndpoint` a private helper:
     `fn token_client(&self, client_id: &str, client_secret: &ClientSecret) -> Result<BasicClient, TokenEndpointError>`
     building ONLY `Ok(BasicClient::new(ClientId::new(client_id.to_string())).set_client_secret(oauth2_client_secret(client_secret)))`.
   - `authorization_code_exchange` becomes `let client = self.token_client(&request.client_id, &request.client_secret)?;` and KEEPS its own `.set_auth_uri(production_auth_url()?)` + `.set_token_uri(self.token_url.clone())` tail (single-section rule: the `?` sits on the helper call).
   - `refresh_token_exchange` gets the identical two-line substitution. No `Permission`, no scope change, no error-mapping change.
   - Verify: `cargo test --locked youtube` exit 0, names unchanged; fmt applied as above. Commit: `refactor: share the oauth client build between both token exchanges`.
3. S7–S10 proceed UNCHANGED from the review-cycle instructions.

## A5. Hard acceptance (both commits)

| Check | Evidence |
| --- | --- |
| `src/youtube/transport.rs` ≤ 200 lines | `wc -l` / `Get-Content … | Measure-Object -Line` after commit 1 AND stays after commit 2 (expected ≈ 45–50) |
| `src/youtube/google_token_endpoint.rs` ≤ 200 lines | same, after commit 2 (expected ≈ 160–185; S6 adds ~+3) |
| All 27 youtube test names preserved | `cargo test --locked youtube` output compares name-for-name (2 remain in transport_tests, 1 in google_token_endpoint_tests, 24 untouched elsewhere) |
| Redaction semantics unchanged | redaction test body moved verbatim; fixture strings identical |
| No new `#[allow(` | `git grep -n "#\[allow(" src/` unchanged from HEAD `ae4e866` |
| No behavior change, tests pass unchanged | dev-checks gate `… run --rm rust sh scripts/dev-checks.sh` exit 0 after commit 2; log recorded |

## A6. Boundary rules for this amend (nothing new)

- No `with_token_url` removal, no error-enum changes (`error.rs` untouched), no token.rs changes, no re-exports added to `mod.rs`, no dependency changes, no comment audit beyond the rewritten transport module doc.
- Both new/edited files obey all house rules: methods ≤ 50 lines, depth ≤ 2, ≤ 2 params — renegotiated only where the task names the exact 3-param `token_client` helper signature (named exception, recorded here as the sanctioned deviation).
- `.gitignore` compliance: `git status --porcelain` before each commit must show only the five intended `src/youtube/` files (commit 1) / two backend files (commit 2); nothing from `dist/`, `logs/`, `target/` staged.
