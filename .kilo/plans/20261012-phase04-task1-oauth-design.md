# Task 1 Plan — Phase 04: OAuth Design and Dependency Selection

- Date: 2026-10-12
- Branch: `feat/phase04-oauth-credentials` (HEAD `125430f`, v0.7.0, tree clean)
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — Task 1 "OAuth Design and Dependency Selection" only. Tasks 2–5 are explicitly OUT of this step.
- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md` (§Task 1, sub-step 4.1b)
- Implementer: JUNIOR developer, 50% restriction — no scope latitude beyond §8 "Explicitly NOT Done".

---

## 0. Task Pre-Analysis (baseline facts, verified)

| Fact | Source |
| --- | --- |
| `auth`/`run` are placeholders (exit 3 after config validation); `demo` is local-only | `src/app/modes.rs` |
| `src/youtube/` contains only a tracked empty `.gitkeep` | `git ls-files src/youtube` |
| Package v0.7.0, edition 2021; deps: serde (derive), serde_json, tracing, tracing-subscriber (env-filter), ab_glyph, ctrlc; dev-dep tempfile (locked 3.14.0) | `Cargo.toml`, context.md |
| No HTTP client, no async runtime in the tree (sync-std Phase-03 decision S2; Tokio reserved) | `tech.md`, context.md |
| `docker-compose.yml` rust service; `CARGO_TARGET_DIR` named volume; command pattern `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>` | docker-compose.yml, context.md |
| `.gitignore` already covers `config/config.json`, `credentials/*` (`!credentials/.gitkeep`), `videos/`, `dist/`, `logs/*` | `.gitignore`, global plan |
| Error convention: hand-written `Display`, `std::error::Error::source`, no `thiserror`, no panics; file ≤200 lines; method ≤50 lines; depth ≤2; ≤2 params (request structs where more) | phase-01/02/03 precedent + `.kilo/rules/*` |

Task 1 is a **design/types-only step**: the OAuth layer's typed surface lands under `src/youtube/`, nothing is wired into CLI/modes, no flow executes, no files are written.

---

## 1. Research Findings (current official Google documentation, verified 2026-10-12)

Sources: `developers.google.com/identity/protocols/oauth2/native-app` (installed apps), `.../web-server` (token/refresh semantics), `developers.google.com/youtube/v3/live/docs/*` (per-method scopes), `developers.google.com/youtube/v3/guides/auth/installed-apps`, `developers.google.com/identity/protocols/oauth2/scopes`.

### 1.1 Installed-application flow (loopback) — AUTHORITATIVE FACTS

- **Endpoints**: authorization `https://accounts.google.com/o/oauth2/v2/auth` (HTTPS only), token `https://oauth2.googleapis.com/token`, programmatic revocation `https://oauth2.googleapis.com/revoke`.
- **Redirect URI — loopback IP is the recommended mechanism for macOS/Linux/Windows desktop**: `http://127.0.0.1:port` or `http://[::1]:port`. The app starts an HTTP listener **on a random available port** and substitutes the actual port. The Google Cloud Console "Desktop app" client type must be used.
- **`localhost` vs `127.0.0.1`**: the official doc says it *"is also possible to use `localhost` in place of the loopback IP, but this configuration may cause issues with client firewalls. Most, but not all, firewalls allow loopback communication."* → **Decision S1: bind `127.0.0.1` only; never use `localhost`.**
- **OOB (manual copy/paste) is DEPRECATED and no longer supported** — the flow must be a real local callback (TODO forbids manual token paste anyway).
- **PKCE**: supported and recommended; verifier = 43–128 chars from `[A-Za-z0-9\-._~]`; `code_challenge_method` supports `S256` (recommended) and `plain` (also the default if `code_challenge` is sent without the method). → **Decision S2: always S256.** Missing/invalid challenge surfaces as `invalid_grant` on the authorization request.
- **`state`**: recommended; **CSRF protection is MANDATORY per the OAuth 2.0 specification §10.12** ("The OAuth client must prevent CSRF"); returned exact-value from the browser must be validated.
- **Authorization-request parameters** (installed apps): `client_id` (required), `redirect_uri` (required), `response_type=code` (required), `scope` (required, space-delimited), `code_challenge` + `code_challenge_method` (recommended), `state` (recommended), `login_hint` (optional). **Incremental authorization is NOT supported for installed apps.**
- **Authorization response**: redirect contains either `code=` or `error=` in the query (e.g. `error=access_denied` when the user declines). Familiar error codes: `access_denied`, `admin_policy_enforced`, `disallowed_useragent`, `org_internal`, `deleted_client`, `invalid_grant`, `redirect_uri_mismatch`, `invalid_request`.
- **Token exchange** (`grant_type=authorization_code`): `client_id`, `client_secret` (**Optional** for installed apps — desktop apps "cannot keep secrets" but a Desktop-type client still carries one), `code`, `code_verifier`, `redirect_uri`.
- **Successful token response fields**: `access_token`, `expires_in` (seconds, ~3592), `id_token` (only when an identity scope was requested — we never request one), `refresh_token` (**"Note that refresh tokens are always returned for installed applications"**), `refresh_token_expires_in` (only for time-based access — not our case), `scope` (space-delimited, case-sensitive granted scopes of *this access token*), `token_type` (always `Bearer`). **"Your application should ignore any unrecognized fields included in the response."**
- **Refresh** (`grant_type=refresh_token`): `client_id`, `client_secret` (optional), `refresh_token`. Response may omit `refresh_token` (only re-issued with `prompt=consent` or after revocation) — code must handle its absence.
- **Refresh-token limits**: *"there are limits on the number of refresh tokens that will be issued; one limit per client/user combination, and another per user across all clients. You should save refresh tokens in long-term storage and continue to use them as long as they remain valid"* → reuse the single stored refresh token; do not re-authorize per run.
- **Revocation**: `POST https://oauth2.googleapis.com/revoke?token=<token>`; revoking an access token also revokes its corresponding refresh token; revocation removes ALL scopes granted to the project; HTTP 200 on success / 400 on error; full effect may be delayed.
- **Errors on refresh**: `invalid_grant` means the refresh token expired/invalidated → re-run interactive authorization.

### 1.2 Minimal YouTube API scope — VERIFIED DECISION

**Decision S3: `https://www.googleapis.com/auth/youtube.force-ssl` is the single requested scope.** Evidence:

| Later-phase operation | Officially accepted scopes (current docs) |
| --- | --- |
| `liveBroadcasts.insert` / lifecycle (YouTube Live Streaming API) | `youtube`, `youtube.force-ssl` |
| `liveStreams.insert` + ingestion inspection | `youtube.readonly`, `youtube`, `youtube.force-ssl` |
| `liveChatMessages.insert` / list (Data API v3 equivalent) | `youtube`, `youtube.force-ssl` |

Justification:
- `youtube.force-ssl` is the **narrowest scope permitting all three operations**; the only alternative set member for the writes is bare `youtube` (broader: full account management).
- `youtube.readonly` cannot perform any broadcast/write/chat-write operation (insert requires `youtube` or `force-ssl`) → insufficient.
- **No `youtube.full.manage` scope exists** in the official Google scopes registry or any YouTube API method page (verified by review of the registry page and method docs; nonexistent scope would break every request with a validation error).
- Note for Task 3 documentation: `youtube.force-ssl` is a **sensitive/restricted-class scope**; first-time authorization on a public app in testing state shows an "unverified app" screen — this is expected behavior, not an implementation defect.

### 1.3 Rust crate evaluation — evidence

| Candidate | Version (today) | MSRV | License | Verdict |
| --- | --- | --- | --- | --- |
| `google-youtube3` | — | — | — | REJECTED by project policy (brief §3.3); not a full auth foundation |
| **`oauth2` (ramosbugs/oauth2-rs)** | **5.0.0** (latest; docs.rs page current) | **1.65** | MIT OR Apache-2.0 | **SELECTED.** Strongly-typed RFC 6749 + RFC 7009 revocation; PKCE (`PkceCodeChallenge::new_random_sha256`, `CsrfToken::new_random`) built in; `SyncHttpClient` seam is `Fn(HttpRequest) -> Result<HttpResponse, E>` (closures auto-implement); re-exports `http` + `url` + `ureq`; actively maintained (MSRV floor policy documented in crate README) |
| `ureq` 3.x | 3.4.2 | **1.85** | MIT OR Apache-2.0 | REJECTED — MSRV exceeds the pinned `rust:1.82` dev/CI image |
| **`ureq` 2.x** | **2.12.1** | **1.71** | MIT OR Apache-2.0 | **SELECTED.** Pure-Rust blocking HTTP client, forbids `unsafe`, rustls TLS (ring provider) by default — no system TLS libs on Windows/Linux; 2.x line is patch-supported to retain MSRV (official changelog: *"2.12.x is MSRV 1.71 … we will release patch version pinning dependencies as needed"*) |
| `reqwest` async | 0.12.28 | pulls `tokio` | MIT OR Apache-2.0 | REJECTED — requires an async runtime that does not exist in the codebase; forces Tokio into the whole dependency graph |
| hand-rolled token exchange over a raw HTTP client | — | — | — | REJECTED — brief §3.3 explicitly forbids manual protocol implementations when a maintained library exists; oauth2 5.0 is that library |

### 1.4 Architecture decision S4 — sync, no Tokio

- **No async runtime in this phase.** The entire OAuth/token layer is synchronous; `ureq 2.x` is blocking; the appeal of `reqwest`/`reqwest::blocking` (which spins an internal Tokio current-thread runtime and bloats the tree) is declined.
- The mockable boundary makes the sync constraint testable and keeps a later Tokio migration (reserved for chat streaming) a swap of the transport impl only — the trait shape stays.
- This mirrors the locked Phase-03 plan's sync decision S2, which is the standing repo baseline.

### 1.5 Decision S5 — mockable boundary + base-URL override

- Our own trait `TokenEndpointTransport` (below) is the seam Google-dependent code talks to. Fakes implement the trait; no network in tests.
- The concrete ureq backend constructs its token endpoint URL from the **constant**; an internal constructor accepts an override URL (`TokenUrl::new(...)`) so an integration test can point the real stack at a fake — the TODO's "base-URL override for fakes" requirement, provided exactly once at the concrete type.
- `oauth2::SyncHttpClient` is closure-compatible: the concrete backend passes a closure backed by its owned `ureq::Agent` (`.redirects(0)` per the crate's SSRF security note).

---

## 2. Cargo.toml — exact dependency edits

Append two lines, keeping the alphabetical order of the existing list (insert after `ctrlc = "3"`):

```toml
oauth2 = { version = "5.0", default-features = false, features = ["ureq"] }
ureq = "2"
```

- `default-features = false` drops oauth2's default `reqwest` + `rustls-tls` reqwest features → **no reqwest, no tokio enter the lockfile** (verified against docs.rs feature metadata: `ureq` feature = `dep:ureq` only).
- `ureq = "2"` is a direct dependency (oauth2's `ureq` feature pulls ureq transitively, but we build our own `Agent` with `redirects(0)`; a direct dep keeps that visible and pinned in our own lockfile entry).
- Resulting `[dependencies]` order: `ab_glyph`, `ctrlc`, `oauth2`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `ureq`.
- `Cargo.lock` is refreshed by a container `cargo check` (see §7 V1). The existing `tempfile = 3.14.0` lock entry MUST NOT move (`cargo check` only adds new package entries; never run `cargo update`). If the diff ever shows tempfile moved, repair with `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update --precise 3.14.0 tempfile` and re-diff.
- Version bump already landed (`125430f`); no further version edit in Task 1.

---

## 3. Module & File Map — `src/youtube/` (exact actions)

```
src/youtube/
  mod.rs                    new   module docs + submodule decls (no re-exports yet)
  ~/.gitkeep                REMOVED (tracked file; see action below)
  error.rs                  new   all redacted error enums + Display/source impls
  client_config.rs          new   Google client_secret.json parse/validate → ClientConfig
  token.rs                  new   secret wrapper types + TokenResponse + GrantedScopes
  authorization.rs          new   scope constants, PKCE/state generators, URL builder, callback parse
  transport.rs              new   TokenEndpointTransport trait + CodeExchangeRequest/RefreshRequest + GoogleTokenEndpoint (ureq)
  credentials.rs            new   StoredCredentials + CredentialStore trait (impl = Task 2)
  verify.rs                 new   VerifiedAuthorization + AuthorizationVerifier trait (impl = Task 4)
  client_config_tests.rs    new
  token_tests.rs            new
  authorization_tests.rs    new
  transport_tests.rs        new   fake-transport tests (offline)
  credentials_tests.rs      new   redaction tests for StoredCredentials (no file I/O)
  error_tests.rs            new   Display contracts + no-secret-leak redaction tests
```

`.gitkeep` action (Decision S6): as soon as real files land, the tracked placeholder is removed with the Phase-03 precedent style — `git rm src/youtube/.gitkeep` in its own commit `chore: remove youtube folder placeholder marker` (mirrors Phase-03 commit `60102fc` for `src/streaming/`). Do NOT delete it before the first real file is added.

`src/main.rs` action (Decision S7): add the module declaration with the deliberate Phase-02-style staging allowance:

```rust
mod app;
mod chat;
mod config;
mod renderer;
#[allow(dead_code)]
mod streaming;
mod youtube;
```

— i.e. add `mod youtube;` (recycling the Phase-02 pattern: the module-level `#[allow(dead_code)]` is intentional staging until Task 3 wires `auth`; it retires there). Keep the existing modules' attributes untouched (no removal from existing modules). Without an `mod youtube;` declaration the module does not compile into the crate at all, so this touch to `main.rs` is inherently in Task 1's scope.

Per-file line budget is enforced (<200 lines): keep each file focused; error Display impls may be split into small `fmt` helper functions (Phase-03 precedent `21189b8`).

### 3.1 Visibility/re-export policy

- Everything is `pub(crate)` or private (`.kilo/rules/prefer-private-members.md`); `mod.rs` adds `pub(crate) use` re-exports ONLY when the first external consumer appears (chat `mod.rs` precedent — none exist yet, so `mod.rs` declares submodules and boundary docs).
- `client_id`/`client_secret` fields are private; accessor methods where needed.

---

## 4. Type Design (skeletons — the junior implementer fills exact bodies)

### 4.1 Secret wrapper types + token response — `token.rs`

```rust
/// Access token for Google API requests. Debug output never reveals the
/// value; Display is intentionally not implemented.
#[derive(Clone)]
pub(crate) struct AccessToken { secret: String }

impl AccessToken {
    pub(crate) fn new(secret: impl Into<String>) -> Self { ... }
    pub(crate) fn secret(&self) -> &str { ... }        // used by the verify/request builder only
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccessToken(<redacted>)")
    }
}
// Same shape for: RefreshToken, AuthorizationCode, ClientSecret.
// PkceCodeVerifier/State: keep oauth2's own `PkceCodeVerifier`/`CsrfToken` types
// (they never print); no own wrapper is needed.
```

Return/refresh response as a serde-parsed struct (mirrors Google's documented fields; unknown fields ignored):

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]   // keep explicit per-field rename instead if clearer
pub(crate) struct TokenEndpointBody {
    access_token: String,
    expires_in: u64,
    #[serde(default)] refresh_token: Option<String>,
    #[serde(default)] scope: Option<String>,
    #[serde(default)] token_type: Option<String>,
}

/// Application-owned, redaction-safe token data constructed from a parsed body.
pub(crate) struct TokenResponse {
    pub(crate) access_token: AccessToken,
    /// Remaining lifetime of `access_token` in seconds, as reported by Google.
    pub(crate) expires_in_seconds: u64,
    pub(crate) refresh_token: Option<RefreshToken>,
    pub(crate) granted_scopes: GrantedScopes,
}
```

`GrantedScopes` — ordered collection of granted scope URL strings:

```rust
pub(crate) struct GrantedScopes { scopes: Vec<String> }

impl GrantedScopes {
    /// Parses Google's space-delimited `scope` field value.
    pub(crate) fn from_space_delimited(value: &str) -> Self { ... }
    /// Exact-match membership (Google scopes are case-sensitive URLs).
    pub(crate) fn grants(&self, scope: &str) -> bool { ... }
}
```

Parsing function (transport layer calls it; tests call it directly):
`pub(crate) fn parse_token_response(body: &str) -> Result<TokenResponse, TokenEndpointError>`
— a valid HTTP 200 body with a missing `access_token` or unparsable JSON maps to `TokenEndpointError::ResponseParse`; a body with `refresh_token` present ever stores it; absent → `None`.

### 4.2 Client configuration model — `client_config.rs`

Parses the Google **downloaded client-secret JSON for a Desktop-app client** (`{"installed": {...}}`):

```rust
/// Client credentials parsed from Google's downloadable client-secret file.
/// `Debug`/`Display` never reveal the secret.
#[derive(Debug)]     // Debug is sound here: ClientSecret's own Debug redacts
pub(crate) struct ClientConfig {
    client_id: String,
    client_secret: ClientSecret,
}

pub(crate) fn parse_client_config(json: &str) -> Result<ClientConfig, ClientConfigError> { ... }
```

- Serde structs mirror Google's real downloadable shape: `ClientSecretFile { installed: InstalledClient }`, `InstalledClient { client_id: String, client_secret: String, #[serde(default)] auth_uri: Option<String>, #[serde(default)] token_uri: Option<String> }` — **tolerant** to the extra fields Google adds (`project_id`, `auth_provider_x509_cert_url`, `redirect_uris`, …): current official guidance for token responses says to ignore unrecognized fields, and Google has been adding fields to these files for years. (`#[serde(bound)]` level detail is the implementer's; no `deny_unknown_fields` here — deviation from the app-config module's strictness is intentional and documented in the file's module docs, because the file is Google-authored, not operator-authored.)
- Validation: `client_id`/`client_secret` non-blank (else `ClientConfigError::MissingField` with the field path string, e.g. `"installed.client_id"`); if `auth_uri`/`token_uri` are present and NOT exactly equal to the Google endpoint constants → `ClientConfigError::EndpointMismatch { field: String }` (prevents tampered redirect targets); absent → treated as the constants.
- The `ClientConfig` stores only id + secret; endpoints are module constants. Task 3 obtains the redirect `127.0.0.1:<port>` at listener-bind time.

### 4.3 Constants + authorization helpers — `authorization.rs`

```rust
pub(crate) const REQUIRED_SCOPE: &str = "https://www.googleapis.com/auth/youtube.force-ssl";
const AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const REVOCATION_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

/// PKCE pair (S256): challenge for the authorization URL, verifier for the code exchange.
pub(crate) struct PkcePair {
    pub(crate) challenge: oauth2::PkceCodeChallenge,
    pub(crate) verifier: oauth2::PkceCodeVerifier,
}

/// Generates a fresh S256 PKCE pair (RFC 7636 entropy via the oauth2 crate).
/// Implemented with the crate's documented tuple construction:
/// let (challenge, verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
pub(crate) fn generate_pkce() -> PkcePair {
    let (challenge, verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    PkcePair { challenge, verifier }
}

/// CSRF `state` value generator (oauth2::CsrfToken::new_random).
pub(crate) fn generate_state() -> oauth2::CsrfToken { ... }

/// Builds the authorization URL the console displays. Pure: no I/O.
pub(crate) fn authorization_url(
    client_config: &ClientConfig,
    redirect_uri: &str,           // http://127.0.0.1:<port>
    state: &oauth2::CsrfToken,
    pkce: &PkcePair,
) -> String { ... }               // https://accounts.google.com/o/oauth2/v2/auth ?
                                  //   client_id=… & response_type=code & scope=<REQUIRED_SCOPE>
                                  //   & redirect_uri=<percent> & state=<value>
                                  //   & code_challenge=<s256> & code_challenge_method=S256
```

URL-building mechanism: `oauth2::url::Url` (re-exported by the oauth2 crate, already in the lockfile) + `url.query_pairs_mut()`; no new URL dependency.

Callback parsing (pure, offline, used by Task 3's listener without changes):

```rust
/// One parsed authorization-endpoint redirect query.
pub(crate) enum AuthorizationCallback {
    Authorized { code: AuthorizationCode, state: String },
    Declined { reason: String },                    // provider error code, never a token
}

/// Parses a callback query string (e.g. "code=…&state=…" or "error=…").
pub(crate) fn parse_authorization_callback(query: &str, expected_state: &oauth2::CsrfToken)
    -> Result<AuthorizationCallback, AuthorizationError> { ... }
```

`AuthorizationError` variants: `MissingParameter { parameter: String }`, `StateMismatch` (CSRF; carries NO attacker-controlled strings), `ProviderDeclined { reason: String }` (`reason` = Google's `error` code string only).

### 4.4 Transport seam — `transport.rs`

```rust
pub(crate) struct CodeExchangeRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) code: AuthorizationCode,
    pub(crate) redirect_uri: String,
    pub(crate) pkce_verifier: oauth2::PkceCodeVerifier,
}

pub(crate) struct RefreshRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: ClientSecret,
    pub(crate) refresh_token: RefreshToken,
}

pub(crate) trait TokenEndpointTransport {
    fn authorization_code_exchange(&self, request: &CodeExchangeRequest)
        -> Result<TokenResponse, TokenEndpointError>;
    fn refresh_token_exchange(&self, request: &RefreshRequest)
        -> Result<TokenResponse, TokenEndpointError>;
}
```

Concrete backend (the only place ureq/oauth2 internals are touched in Task 1):

```rust
/// ureq-backed Google token endpoint client. Fixed to the production Google
/// endpoint; `with_token_url` exists so integration tests can substitute a
/// fake endpoint without any production configuration touching it.
pub(crate) struct GoogleTokenEndpoint {
    agent: ureq::Agent,          // Agent::new_with_config: redirects(0) per oauth2 crate SSRF note
    token_url: oauth2::TokenUrl,
}

impl GoogleTokenEndpoint {
    pub(crate) fn new() -> Self { ... }
    #[cfg(test)]
    pub(crate) fn with_token_url(token_url: oauth2::TokenUrl) -> Self { ... }
}

impl TokenEndpointTransport for GoogleTokenEndpoint { ... }
```

Implementation notes (fixed, no latitude):
- Build requests through `oauth2::BasicClient` (`BasicClient::new(ClientId).set_client_secret(...).set_auth_uri(...).set_token_uri(...)`) exactly like the crate's synchronous example, calling `exchange_code(...).set_pkce_verifier(...).set_redirect_uri(...).request(&http_client)` / `exchange_refresh_token(...).request(&http_client)`.
- The `http_client` argument is a closure over the struct's owned `ureq::Agent` — oauth2's `SyncHttpClient` trait is "automatically implemented for any function/closure that implements `Fn(HttpRequest) -> Result<HttpResponse, E>`", so no extra trait impl files are needed.
- Map crate results: `RequestTokenError::ServerResponse(error)` → `TokenEndpointError::Server { code: error.error().as_str().to_string(), description: error_error_description_option }` (error code + description strings from Google's structured error JSON — these are NOT secrets); `RequestTokenError::Http(status)` variants/timeouts/URL failures → `TokenEndpointError::Transport { reason: String }` where `reason` is a status/description string **with the response body deliberately excluded**; `RequestTokenError::Parse` → `TokenEndpointError::ResponseParse`.
- Success: `StandardTokenResponse` → `TokenResponse` via `.access_token().secret()`, `.expires_in()` (chrono duration → seconds), `.refresh_token()` opt, `.scopes()` list (space-joined for `GrantedScopes::from_space_delimited`), `token_type` ignored except non-Bearer → treat as `ResponseParse` mismatch.
- `StandardTokenResponse::scopes()` returns `Option<&Vec<Scope>>`; missing scope field → `GrantedScopes::from_space_delimited("")` (empty set; verification fails later — never an error path of its own).

### 4.5 Persistence + verification seams (types only; impl = Tasks 2/4)

```rust
// credentials.rs
pub(crate) struct StoredCredentials {
    pub(crate) access_token: AccessToken,
    pub(crate) expires_in_seconds: u64,             // TTL at save time; clock use = Task 4
    pub(crate) refresh_token: Option<RefreshToken>,
    pub(crate) granted_scopes: GrantedScopes,
}

pub(crate) trait CredentialStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError>;
    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError>;
}
pub(crate) struct FileCredentialStore { path: PathBuf } // Task 2 implements; keep the struct minimal or defer it entirely to Task 2 — DEFERRED: declare the TRAIT + STORED TYPE only.
```

Decision S8: `FileCredentialStore` is NOT created in Task 1 (a trait with a single unimplemented struct invites dead-code churn); Task 2 adds the concrete type. Only `StoredCredentials` + `CredentialStore` + `CredentialStoreError` exist now.

```rust
// verify.rs
pub(crate) struct VerifiedAuthorization { granted_scopes: GrantedScopes } // + accessor

pub(crate) trait AuthorizationVerifier {
    /// Confirms the effective authorization carries REQUIRED_SCOPE.
    fn verify(&self, access_token: &AccessToken)
        -> Result<VerifiedAuthorization, VerificationError>;
}
```

Decision S9: the concrete verification request (minimal YouTube API call fitting `force-ssl`) is Task 4's planned decision; Task 1 fixes only the seam above so the credential service stays mockable.

### 4.6 Error enums — `error.rs` (complete variant list)

All four enums in ONE small file, hand-written `Display`, `std::error::Error::source` where a wrapped `io::Error` exists, `Debug` derives safe (wrapper types redact; variant payloads are non-secret):

- `ClientConfigError { MissingField { field: String }, BlankField { field: String }, EndpointMismatch { field: String }, Deserialization { message: String } }` — messages like `invalid OAuth client configuration: <field> must not be empty` / `…token_uri does not match the expected Google token endpoint`.
- `AuthorizationError { MissingParameter { parameter: String }, StateMismatch, ProviderDeclined { reason: String } }`.
- `TokenEndpointError { Transport { reason: String }, Server { code: String, description: Option<String> }, ResponseParse }`.
- `CredentialStoreError { FileOpen { path: PathBuf, source: io::Error }, FileWrite { path: PathBuf, source: io::Error }, ReadParse { path: PathBuf, message: String }, ContentNotAvailable }` (Task 1 adds the enum; Task 2 produces values).
- `VerificationError { NotAuthorized { reason: String }, InsufficientScope { reason: String }, Transport { reason: String } }` (Task 1 adds; Task 4 produces values).

Display wrappers: `ClientConfigError::MissingField` → `invalid OAuth client configuration: field '<f>' is required`; `TokenEndpointError::Server` → `the Google token endpoint rejected the request (code '<code>')` (+ `: <description>` when present); etc. All wording prefixed to be end-user-reportable with zero secrets inlined (§5).

---

## 5. Redaction Rules (type-level decisions, enforced by review + tests)

1. Secrets/tokens/codes/verifiers exist ONLY inside the wrapper types (`ClientSecret`, `AccessToken`, `RefreshToken`, `AuthorizationCode`) and oauth2's own share (`PkceCodeVerifier`, `CsrfToken`).
2. Wrapper types implement `Debug` printing `TypeName(<redacted>)`; **no `Display` for these types** (accidental `format!("{token}")` cannot compile / never stringify a secret).
3. Raw `String`/`&str` values for secrets are never stored outside wrappers: every parse site wraps immediately; `secret()` accessors are used at exactly two call sites (transport request build, Bearer header in Task 3/4) and their results are consumed without logging.
4. Error enums may carry: field paths, I/O `io::Error`s, Google's structured OAuth `error` code + `error_description` strings, HTTP status numbers, and generic transport reasons. They must NEVER carry: token values, the client secret or its raw value string, the PKCE verifier, the authorization code value, or any response body.
5. The transport layer MUST NOT copy response bodies into error variants (status + structured error CODE only; on parse failure even Google's raw JSON is dropped).
6. Redaction tests (§6) construct real errors from fixture strings like `"client_secret_fixture"` / `"access_token_fixture"` and assert those strings never appear in `format!("{e:?}")` or `format!("{e}")`.

---

## 6. Test Plan (deterministic, offline — no Google, no network)

| File / test name (exact) | Asserts |
| --- | --- |
| `client_config_tests.rs::parse_reads_installed_client_fields` | happy-path Google-shaped JSON → `ClientConfig` with matching id/secret |
| `client_config_tests.rs::parse_preserves_unknown_fields` | file with `project_id`/`redirect_uris` extra keys still parses |
| `client_config_tests.rs::parse_reports_missing_installed_client_id` | `"installed"` missing → `MissingField { field: "installed.client_id" }` |
| `client_config_tests.rs::parse_reports_blank_client_secret` | empty secret → `BlankField` |
| `client_config_tests.rs::parse_rejects_foreign_token_uri` | `token_uri: "https://evil.invalid/token"` → `EndpointMismatch { field: "installed.token_uri" }` |
| `client_config_tests.rs::parse_rejects_invalid_json` | garbage → `Deserialization` |
| `client_config_tests.rs::client_config_debug_never_reveals_secret` | `format!("{config:?}")` lacks the secret fixture string |
| `token_tests.rs::parse_full_token_response_with_refresh_token` | documented sample body → all fields incl. `refresh_token: Some(..)` |
| `token_tests.rs::parse_refresh_response_without_refresh_token` | refresh shape (no refresh_token field) → `Some` omitted |
| `token_tests.rs::parse_ignores_unknown_fields` | extra `id_token`/future fields accepted |
| `token_tests.rs::parse_reports_missing_access_token` → `ResponseParse` | |
| `token_tests.rs::granted_scopes_membership_is_exact_and_case_sensitive` | `grants(REQUIRED_SCOPE)` true / partial-URL false |
| `token_tests.rs::token_response_debug_and_display_reveal_no_secrets` | fixture `access_token_fixture`/`refresh_token_fixture` absent from BOTH debug + display strings (TokenResponse gets a hand-written redacted Debug, and modules never `Display` it) |
| `authorization_tests.rs::authorization_url_encodes_all_parameters` | exact URL string: endpoint constant, `response_type=code`, `scope=<REQUIRED_SCOPE>`, `code_challenge_method=S256`, challenging == base64url(sha256(verifier)), percent-encoded values |
| `authorization_tests.rs::authorization_url_uses_the_force_ssl_scope` | contains full scope string |
| `authorization_tests.rs::generate_pkce_produces_valid_pair` | verifier length 43–128, charset; challenge==s256(verifier) |
| `authorization_tests.rs::generate_state_produces_random_value` | two calls differ |
| `authorization_tests.rs::callback_parses_authorized_query` | `code=…&state=…` ↔ expected state |
| `authorization_tests.rs::callback_rejects_state_mismatch` | different state → `StateMismatch`, error Debug/Display contains no codes/values |
| `authorization_tests.rs::callback_parses_provider_declined` | `error=access_denied` → `ProviderDeclined` with exactly `access_denied` |
| `authorization_tests.rs::callback_reports_missing_parameter` | missing `state` → `MissingParameter { parameter: "state" }` |
| `transport_tests.rs::fake_transport_returns_the_stored_token_response` | tiny `FakeTransport` implementing the trait (stubbed exchange/refresh) — proves the seam compiles and drives |
| `transport_tests.rs::fake_transport_propagates_typed_server_error` | returns `TokenEndpointError::Server { code: "invalid_grant", description: None }` unchanged |
| `transport_tests.rs::transport_error_display_never_carries_secret_fixtures` | constructs every `TokenEndpointError` variant embedding fixture secret strings in surrounding context and asserts none appear |
| `credentials_tests.rs::stored_credentials_debug_reveals_no_secrets` | plus `load/save` are the trait's only methods |
| `error_tests.rs::each_variant_has_a_stable_display_line` | snapshot-style `assert_eq`/`starts_with` per variant (Phase-01 error-tests pattern) |
| `error_tests.rs::source_chain_exposes_io_errors_only` | `source()` Some only for io-wrapped variants |

All tests are `#[cfg(test)] #[path = "..."] mod` siblings (existing pattern), `tempfile` only where a path type needs exercising, no `ureq` network calls, no OAuth consent, no wall-clock dependency.

---

## 7. Implementation Order for the Implementer (with git actions)

All steps on `feat/phase04-oauth-credentials`. Host is Windows; every cargo action runs through the Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`). Never stage gitignored paths; before every commit: `git status --porcelain` must show exactly the intended files.

### Step 1 — dependencies
Edit `Cargo.toml` exactly as §2. Then:
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check` → refreshes `Cargo.lock` with the new crates (oauth2 5.0.0, ureq 2.12.1 + rustls/ring/serde/url/… transitive; NO reqwest, NO tokio — verify: `grep -E "^name = \"(reqwest|tokio)\"" Cargo.lock` → no matches).
Then `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` → exit 0.
Commit: `git add Cargo.toml Cargo.lock` → `git commit -m "build: add oauth2 and ureq crates for the youtube module"`.

### Step 2 — module skeleton + typed errors
- Remove placeholder: `git rm src/youtube/.gitkeep`.
- Add `src/youtube/mod.rs`: module docs = boundary rules (owns OAuth/token material; never depends on chat/renderer/streaming; sync transport; no secrets outside wrappers/errors) + `mod client_config; mod token; mod authorization; mod transport; mod credentials; mod verify; mod error;` (each with its `#[cfg(test)] #[path = "..._tests.rs"] mod ..._tests;` line as the file's tests come in).
- Add `src/youtube/error.rs` + `error_tests.rs` per §4.6.
- Edit `src/main.rs`: add `mod youtube;` with `#[allow(dead_code)]` (§3).
- Verify `cargo check --locked` exit 0 (module compiles; dead-code staging silences the unused surface).
- Commit: `git add src/main.rs src/youtube/mod.rs src/youtube/error.rs src/youtube/error_tests.rs` + the already-rm'd `.gitkeep` → wait: `.gitkeep` removal is its own commit per S6 ordering chosen below; do the removal commit FIRST:
  - Commit A: `git rm src/youtube/.gitkeep` → `git commit -m "chore: remove youtube folder placeholder marker"`.
  - Commit B: everything above → `feat: add youtube module skeleton with typed oauth errors`.

### Step 3 — client configuration model
`src/youtube/client_config.rs` + `client_config_tests.rs` (§4.2, §6). Wire `mod client_config;` + its tests into `mod.rs`.
Commit: `feat: parse google oauth client configuration`.

### Step 4 — token types
`src/youtube/token.rs` + `token_tests.rs` (§4.1, §6).
Commit: `feat: add redacted oauth token response types`.

### Step 5 — transport seam + ureq backend
`src/youtube/transport.rs` + `transport_tests.rs` (§4.4, §6). `GoogleTokenEndpoint::with_token_url` is `#[cfg(test)]`-gated so dead-code staging stays clean — if clippy still flags `new()` as unused in dead-code-staged code, the module-level `#[allow(dead_code)]` (S7) already covers it.
Commit: `feat: add oauth token endpoint transport seam with ureq backend`.

### Step 6 — authorization helpers
`src/youtube/authorization.rs` + `authorization_tests.rs` (§4.3, §6).
Commit: `feat: add oauth authorization url builder and callback parser`.

### Step 7 — persistence + verification seams
`src/youtube/credentials.rs`, `src/youtube/verify.rs`, `credentials_tests.rs` (§4.5, §6; S8 defers `FileCredentialStore`; S9 defers the concrete request).
Commit: `feat: add credential persistence and authorization verification seams`.

### Step 8 — full verification (fresh evidence)
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → must exit 0: `PASS fmt-check/check/test/clippy`, `ALL CHECKS PASSED`, test total = 147 + §6 additions (record the observed number in the adherence report).
- `git status --porcelain` → empty or only the fresh plan file; compile is clean with zero `#[allow(` added anywhere except the one deliberate `mod youtube` staging line; `git grep -n "#\[allow(" src/` shows ONLY that one line.
- If rustfmt demands re-wrapping: run `cargo fmt` ONLY inside the container (`... run --rm rust cargo fmt`), inspect the diff, amend into the step's commit BEFORE committing (never a separate formatting commit unless the steps are already committed).

### Commit hygiene (every commit)
```
git status --porcelain          # exactly the intended files only
git diff --cached --stat        # sanity before commit
```
No commit touches `config/`, `credentials/`, `README.md`, docs, or any other module. Only `.kilo/plans/` tracking of this file may appear in a separate docs commit (`docs: track task 1 planning documents`).

---

## 8. Explicitly NOT Done by Task 1 (boundaries)

- No `auth` command implementation/wiring; `modes.rs` untouched; exit codes unchanged.
- No config-file schema change (`src/config/**`, `config/config.example.json` untouched) and no configuration section for OAuth paths (Task 2).
- No credential persistence file work: no `credentials.rs` concrete store, no file I/O, no permission-restriction code (Task 2).
- No callback HTTP listener, no browser launch, no network execution in production paths (Tasks 3–4).
- No broadcast/liveStream/chat-API/RTMPS/FFmpeg code; no scope-verification request implementation (Task 4).
- No removal or modification of code in existing modules except the two encoded touches: `src/main.rs` (module declaration line, §3) and `Cargo.toml`/`Cargo.lock` (§2).
- No logging of anything OAuth; no new `tracing` events (they come with real flows in Task 3).
- No `thiserror`, no panics/unwrap in library paths, no async, no new dev-dependency beyond `tempfile` (already present).

---

## 9. Verification Checklist for the Adherence Reviewer

| # | Check | Evidence marker |
| --- | --- | --- |
| 1 | Typing surfaces exist and compile: `src/youtube/**` 8 source files + test siblings; no `.gitkeep` | `git ls-files src/youtube` |
| 2 | Deps: `oauth2` (default-features=false, `ureq` feature) + `ureq = "2"` in Cargo.toml; lockfile contains `oauth2 5.0.0`, `ureq 2.12.x`; NO `reqwest`/`tokio` entries | `git grep oauth2 Cargo.toml`; lockfile grep |
| 3 | Scope constant is exactly `https://www.googleapis.com/auth/youtube.force-ssl` | `authorization.rs` |
| 4 | Mockable seam: `TokenEndpointTransport` + fakes in tests; `with_token_url` exists | `transport.rs`/`transport_tests.rs` |
| 5 | Redaction: wrapper Debug redacts; error enums carry no secret fields; redaction tests present | §5/§6 tests |
| 6 | Tasks 2–5 boundaries intact | §8 grep checks (`git grep -n "youtube" src/config src/app` → only `main.rs` decl line) |
| 7 | dev-checks gate exit 0 with FRESH log path recorded | §7 Step 8 |
| 8 | Commits match plan names; gitignore-compliant staging; no tmp files left | `git log --oneline main..HEAD` |

---

## 10. Risks / Deviation Log (pre-registered)

- **D1**: oauth2 5.0's `StandardTokenResponse::scopes()` returns `Option<&Vec<Scope>>`; missing `scope` field → empty `GrantedScopes` (documented, never an error) — verification in Task 4 catches the missing scope.
- **D2**: Google's downloadable client file may gain new fields → tolerant parse (S4.2 note); deviation from `config` module's `deny_unknown_fields` strictness, intentional + documented.
- **D3**: oauth2 crate internally uses `thiserror` — allowed (dependency tree; our own code hand-writes Display per policy).
- **D4**: locked `tempfile 3.14.0` must not move during the lockfile refresh (§2 repair path).
- **D5**: `ureq 2.12.x` uses rustls+ring → adds `ring` (C code) to the build; acceptable on the pinned Linux image + Windows GNU cross toolchain (pure-Rust top-level crates policy satisfied: no system TLS libs required).
- **D6**: `oauth2` dependency is v5.0.0 (unchanged latest since its release; actively maintained per repo metadata reviewed at plan time). If lockfile resolves different, record the resolved version in the adherence report.

## 11. Plan Self-Check vs TODO Task 1

- ✔ Read brief/tech/config-error/CLI contracts + Phase-03 completion report before planning (§0, all callers' required reading done).
- ✔ Google installed-app guidance researched (§1.1 — loopback 127.0.0.1, PKCE S256, state/CSRF, endpoints, token/refresh/revocation semantics).
- ✔ Minimal scope researched + verified + justified (§1.2 → S3).
- ✔ Rust crates evaluated + selected with evidence (§1.3 → S4/S5, no Tokio).
- ✔ Typed interfaces/errors for client config, authorization, token exchange/refresh, persistence, verification — mockable (§3–§6).
- ✔ No broadcast/ingestion/chat code (§8).
- Plan file saved at this very path — the only file this sub-step has created/modified.
