# Task 3 Plan — Phase 04: Implement the `auth` Command

- Date: 2026-10-12
- Branch: `feat/phase04-oauth-credentials` (HEAD `9ebf13f`, v0.7.0, tree clean, 195 tests green)
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — **Task 3 "Implement the `auth` Command" ONLY**. Tasks 1–2 are DONE and provide the typed layer; Task 4 owns the reusable credential service; Task 5 owns docs/report. Tasks 1/2/4/5 boundaries declared in §15.
- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md` §Task 3 (pre-analysis).
- Sanctioned OAuth facts (reuse, NO re-research): `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1.1/§1.2/§4.3/§5 — loopback `127.0.0.1` + random port (decision S1), PKCE S256 (S2), scope `https://www.googleapis.com/auth/youtube.force-ssl` (S3), sync-std (S4), mockable `TokenEndpointTransport` (S5), refresh tokens ALWAYS returned for installed apps, token endpoint rejects with structured `error` code + description.
- Task 2 layer facts: `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md` (D1 config paths, D3/D4/D7 store, `FileCredentialStore` atomic + 0600 POSIX).
- Implementer: JUNIOR developer, 50% restriction — no latitude beyond §15 "Explicitly NOT Done". Every structural/architectural decision below is locked; none is open.

## 0. Task Pre-Analysis (baseline facts, verified this session)

| Fact | Source (verified reads) |
| --- | --- |
| `auth` placeholder: `modes::auth(config) -> u8` prints one stderr `error:` line, returns `NOT_IMPLEMENTED_EXIT` (3). `modes::run` placeholder + its 3 tests must stay BYTE-IDENTICAL | `src/app/modes.rs` |
| Startup funnel: parse then logging then load+validate config then dispatch; three info events with whitelisted fields (`command`, `config_path`, `exit_code`); `report_failure` = single stderr `error:` line | `src/app/mod.rs` |
| Dispatch arm to replace: `cli::Mode::Auth => modes::auth(app_config)` | `src/app/mod.rs` dispatch |
| CLI usage text labels `auth` "not implemented yet"; `cli_tests.rs` has NO assertion on the auth line (only default-path + demo-line assertions) | `src/app/cli.rs`, `src/app/cli_tests.rs` |
| `src/main.rs` staging line: `#[allow(dead_code)] mod youtube;` | `src/main.rs` |
| `src/youtube/` surface (all read): `parse_client_config`/`ClientConfig` (`client_id()`/`client_secret()` accessors, tolerant Google-file parse, endpoint check), `REQUIRED_SCOPE` const, `generate_pkce` returning `PkcePair { challenge, verifier }`, `generate_state` returning `oauth2::CsrfToken`, `authorization_url(client_config, redirect_uri, state, pkce) -> String`, `parse_authorization_callback(query, expected_state) -> Result<AuthorizationCallback, AuthorizationError>` with `AuthorizationCallback { Authorized { code, state }, Declined { reason } }` and `AuthorizationError { MissingParameter { parameter }, StateMismatch, ProviderDeclined { reason } }`, `GoogleTokenEndpoint::new() -> Result<Self, TokenEndpointError>` implementing `TokenEndpointTransport` (`authorization_code_exchange(&CodeExchangeRequest)`/`refresh_token_exchange`), `CodeExchangeRequest { client_id: String, client_secret: ClientSecret, code: AuthorizationCode, redirect_uri: String, pkce_verifier: oauth2::PkceCodeVerifier }`, `FileCredentialStore::new(PathBuf)` implementing `CredentialStore { load, save }` with `CredentialStoreError {..PathBuf, io}`, `StoredCredentials { access_token, expires_in_seconds, refresh_token: Option<RefreshToken>, granted_scopes }` (all fields `pub(crate)`), `TokenResponse { access_token, expires_in_seconds, refresh_token, granted_scopes }`, `GrantedScopes::grants(&str) -> bool` (exact match), hand-written redacted `Display` on every error enum | `src/youtube/*.rs` |
| Submodule items are NOT re-exported: `youtube/mod.rs` policy says re-exports are added only when the first external consumer appears; `src/app::auth` IS that first consumer | `src/youtube/mod.rs` docs |
| `oauth.client_config_path` + `oauth.credentials_path` exist in `AppConfig`, validated non-blank + distinct by the config module | `src/config/model.rs`, Task 2 plan D2 |
| `demo` exit contract: success 0; ANY `DemoError` = one stderr `error:` line + `STARTUP_FAILURE_EXIT` (1); demo never exits 3 | `src/app/demo/mod.rs` |
| Deps locked: serde (derive), serde_json, tracing, tracing-subscriber, ab_glyph, ctrlc, oauth2 5.0 (default-features=false, `ureq` feature), ureq 2.12.x; dev-dep tempfile 3.14.0 (must not move). NO new deps allowed this task | `Cargo.toml`, context.md |
| `crate::config::valid_config()` cfg-test fixture exists (used by modes tests) | `src/config/test_fixtures.rs` |
| Container verification via Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`, command prefix `docker`); container runs as root | context.md |
| Client-secret JSON fixture shape (Task 1/2 test precedent): `{"installed":{"client_id":"...","client_secret":"...","auth_uri":"https://accounts.google.com/o/oauth2/v2/auth","token_uri":"https://oauth2.googleapis.com/token"}}` | `client_config_tests.rs` |
| Rule caps: file within 200 lines, fn body within 50, indentation depth within 2, max 2 non-receiver params, single-section boolean conditions (extract predicate fns), hand-written `Display`, no `thiserror`, no panics/unwrap in non-test library paths, no commented-out code | `.kilo/rules/*` |

Task 3 is the **wiring + interactive-flow step**: the first production consumer of the `src/youtube/` layer. No refresh path, no network verifier, no config/schema change.

## 1. High-Level Approach (locked)

`auth` executes a synchronous, single-threaded, std-only installed-application authorization-code flow:

```
read client file (oauth.client_config_path) then parse_client_config   [fail = ClientConfigFile / ClientConfig]
        |
bind TcpListener on 127.0.0.1:0 (loopback only, ephemeral port)
        |  redirect_uri = http://127.0.0.1:<port>                      [fail = ListenerBind]
        |
generate state + PKCE(S256), build authorization_url,
print instructions + URL to stdout (NO auto browser launch)
        |
blocking accept loop on the MAIN thread (no spawned threads, no async):
   each connection: bounded read, extract query, parse_authorization_callback
   - Authorized { code }  -> respond 200, exit loop (state validated inside parse)
   - Declined { reason }  -> respond 400, TERMINAL fail
   - StateMismatch        -> respond 400, TERMINAL fail (CSRF)
   - Noise (unparseable/garbage/missing-params/oversized/non-UTF8/timeout)
                          -> respond 400, decrement budget; 8 exhausted = TERMINAL fail
   listener DROPS here on both success and failure (Rust ownership)
                                                                       [fail = ProviderDeclined / StateMismatch / CallbackAbandoned]
        |
GoogleTokenEndpoint::authorization_code_exchange with
CodeExchangeRequest { redirect_uri, pkce verifier }                    [fail = TokenExchange]
        |
validate: granted_scopes.grants(REQUIRED_SCOPE)                        [fail = InsufficientScope]
          refresh_token present (installed apps always get one)        [fail = MissingRefreshToken]
        |
FileCredentialStore(oauth.credentials_path).save(StoredCredentials)    [fail = Persistence]
        |
print success line (stdout, no secrets), exit 0
```

Every failure: exactly one stderr `error:` line (redacted, §2 D2) and exit code 1.

## 2. Locked Decisions (numbered; all pre-made, none open)

### D1 — Error enum `AuthError` (lives in `src/app/auth/mod.rs`)

```rust
#[derive(Debug)]
pub(super) enum AuthError {
    /// The configured OAuth client-configuration file could not be read
    /// (missing, unreadable, directory target, invalid UTF-8).
    ClientConfigFile { path: String, source: std::io::Error },
    /// The file was read but its content is invalid (schema, blank or
    /// endpoint mismatch); carries the youtube-layer error verbatim.
    ClientConfig(ClientConfigError),
    /// The loopback callback listener could not be bound (or its address
    /// could not be read).
    ListenerBind(std::io::Error),
    /// The listener closed without receiving a usable callback: the bounded
    /// noise budget (8 non-callback connection attempts) was exhausted.
    CallbackAbandoned { attempts: usize },
    /// The callback failed its CSRF state validation (always terminal).
    StateMismatch,
    /// The provider declined the authorization request in the browser
    /// (terminal; `reason` is Google's error code string only, non-secret).
    ProviderDeclined { reason: String },
    /// The authorization-code exchange failed; the youtube-layer error
    /// Display is already redacted.
    TokenExchange(TokenEndpointError),
    /// Credential persistence failed; verbatim delegation.
    Persistence(CredentialStoreError),
    /// The granted scopes do not include the required YouTube scope.
    InsufficientScope,
    /// The token response did not include a refresh token, which later
    /// unattended runs require (brief section 6.3).
    MissingRefreshToken,
}
```

Justification for `MissingRefreshToken` (addition to the caller's example variant list): Task 4's credential service and brief section 6.3 depend on stored refresh credentials; Google's documented installed-app behavior (Task 1 plan section 1.1) is "refresh tokens are always returned for installed applications", so absence is a hard certification failure. Persisting a token pair without refresh capability would silently break the phase premise. The variant is unit (no payload) so nothing can leak.

`Debug` is derived safely: all payloads are non-secret (paths, io errors, provider code strings, redacted youtube-layer errors).

### D2 — Display contracts (exact strings, snapshot-tested)

All variants produce ONE end-user-reportable line; inner youtube-layer errors delegate verbatim; terminal-certification variants state the operator's next action. Fixed message constants (module-level in `mod.rs`):

```
STATE_MISMATCH_MESSAGE = "authorization callback state did not match the expected value (possible cross-site request forgery); run the authorization command again"

ABANDONED_MESSAGE = "no usable authorization callback was received within the allowed 8 connection attempts; run the authorization command again and re-open the printed URL"
```

Display arms (exact):

| Variant | Display text |
| --- | --- |
| `ClientConfigFile` | `unable to read the OAuth client configuration file '{path}': {source}` |
| `ClientConfig(error)` | `{error}` verbatim (inner text already starts "invalid OAuth client configuration: ") |
| `ListenerBind(source)` | `unable to bind the OAuth callback listener on 127.0.0.1: {source}` |
| `CallbackAbandoned { .. }` | `ABANDONED_MESSAGE` (the `attempts` field is NOT formatted into the sentence; the budget number is architectural) |
| `StateMismatch` | `STATE_MISMATCH_MESSAGE` |
| `ProviderDeclined { reason }` | `the authorization was declined by the provider (error code '{reason}'); run the authorization command again if this was unintended` |
| `TokenExchange(error)` | `{error}` verbatim |
| `Persistence(error)` | `{error}` verbatim |
| `InsufficientScope` | `the granted authorization does not include the required YouTube scope ({REQUIRED_SCOPE}); no credentials were stored; run the authorization command again and approve the requested YouTube permission` |
| `MissingRefreshToken` | `the token response did not include a refresh token, which later unattended runs require; no credentials were stored; run the authorization command again and approve the requested YouTube permission` |

`InsufficientScope` Display embeds the `REQUIRED_SCOPE` constant by value (it is a public scope URL, not a secret).

`impl std::error::Error for AuthError` with `source()` returning `Some(source)` only for `ClientConfigFile` and `ListenerBind` (io-cause transport per the Phase-01 error convention); `None` otherwise.

### D3 — Exit-code mapping: ALL auth failures exit 1 (option A); placeholder exit 3 retired for `auth` only

| # | Failure cause | Variant | Exit code |
| --- | --- | --- | --- |
| 1 | client file unreadable | `ClientConfigFile` | 1 |
| 2 | client content invalid | `ClientConfig` | 1 |
| 3 | listener bind failed | `ListenerBind` | 1 |
| 4 | noise budget exhausted | `CallbackAbandoned` | 1 |
| 5 | CSRF state mismatch | `StateMismatch` | 1 |
| 6 | provider declined | `ProviderDeclined` | 1 |
| 7 | exchange failed | `TokenExchange` | 1 |
| 8 | persistence failed | `Persistence` | 1 |
| 9 | scope missing | `InsufficientScope` | 1 |
| 10 | refresh token absent | `MissingRefreshToken` | 1 |
| - | success | - | 0 |

```rust
impl AuthError {
    /// Every auth-origin failure is a post-startup fatal mode failure and
    /// exits with the startup-failure class (code 1), exactly like demo.
    pub(super) fn exit_code(&self) -> u8 { STARTUP_FAILURE_EXIT }
}
```

Justification (documented contract change; TODO bullet 6 sanctions "unless a justified, documented change is required"):

- Usage errors exit 2 and startup-funnel config/log failures exit 1 are PRESERVED (untouched code paths).
- After successful config validation, `auth` is in exactly the same class as `demo`: a post-startup mode-origin failure reported by one stderr line and exit 1 (`DemoError` precedent).
- Exit 3 semantically means "mode not implemented" (`NOT_IMPLEMENTED_EXIT` doc comment states it); retiring it for `auth` is pre-decided in the global plan section Task 3 ("the placeholder exit-3 must be retired for auth only; run keeps exit 3").
- Distinct code 4 was REJECTED: every auth failure ends in the same operator action (fix configuration or retry authorization); the stderr line already differentiates causes; a new exit class would add docs/tests burden with no operator benefit.
- `run` DOES NOT change: `modes::run` stays byte-identical; `NOT_IMPLEMENTED_EXIT` remains defined and used by `run` only.

### D4 — App-layer placement and file decomposition (`src/app/auth/`)

| File | Content | Projected lines |
| --- | --- | --- |
| `src/app/auth/mod.rs` | module docs (flow contract, redaction rules, no-auto-launch, listener lifecycle, exit-code contract D3); `run_auth` entry; `auth_outcome`; transport/store construction; `AuthError` + Display + Error source impl + message consts + `exit_code` + `report_auth_failure`; submodule + test decls | ~175 |
| `src/app/auth/flow.rs` | `AuthFlow` struct; `run()`; `run_with_listener()`; `open_authorization`; `OpenedFlow`; `exchange_authorization_code`; free fns `read_client_config`, `storeable_credentials`, `code_exchange_request`; `save_validated_credentials` | ~180 |
| `src/app/auth/listener.rs` | `BoundListener`; `bind()`; `wait_for_authorized_code`; per-connection classify/read/extract/respond helpers; `CallbackOutcome`; all socket constants | ~195 |
| `src/app/auth/console.rs` | the two exact print functions (section 4.5) | ~30 |
| `src/app/auth/mod_tests.rs` | M1-M4 (test plan) | ~150 |
| `src/app/auth/flow_tests.rs` | F1-F6 + F11 (test plan) | keep within 200; if a single test file would exceed 200 lines, split into `flow_tests.rs` (client/validation/flow tests) + new sibling `flow_exchange_tests.rs` (transport-fake + store-seam tests F7/F8/F9/F10) with the standard `#[cfg(test)] #[path = "..."]` decl pair |
| `src/app/auth/listener_tests.rs` | L1-L8 (test plan) | keep within 200 |

Rule-cap enforcement in every file: fn bodies within 50 lines (split helpers instead), depth within 2 (3rd level goes to a helper), max 2 non-receiver params (request structs where more), single-section boolean conditions via named predicate helpers, no panics/unwrap/expect in non-test paths (tests may unwrap/expect), no commented-out code, self-documenting names.

### D5 — Client-config file failure classes (exact)

Reading happens FIRST, before any listener is bound (TODO bullet: "Load and validate the configured OAuth client data before opening the flow"):

- `std::fs::read_to_string(Path::new(path))` failure maps to `AuthError::ClientConfigFile { path: path.to_string(), source }`. Covers missing file, permission denied, directory target, invalid UTF-8.
- Content parse failure (`parse_client_config` returning `ClientConfigError`) maps to `AuthError::ClientConfig(error)` (Display delegates verbatim).
- Both exit 1 (D3). No existence pre-check is added to the config module (Task 2 ruling stands: existence belongs to the use site).

Free function (one param, testable):

```rust
pub(super) fn read_client_config(client_config_path: &str) -> Result<ClientConfig, AuthError>
```

### D6 — Listener lifecycle (loopback-only, deterministic shutdown)

- Bind `TcpListener::bind((CALLBACK_HOST, 0))` with `CALLBACK_HOST = "127.0.0.1"` — loopback ONLY per Task 1 decision S1 (never `localhost`, never `::`, never `0.0.0.0`). Port is OS-ephemeral.
- `redirect_uri = format!("http://127.0.0.1:{port}")` from `local_addr().port()`. A `local_addr()` failure maps to `ListenerBind` too (same class: the flow cannot learn its own address).
- Binding and the accept loop run on the MAIN thread. No threads, no tokio, no background tasks. The mode does NOT install the Ctrl+C handler (documented decision: the mode is interactive and short-lived; default OS termination is acceptable and always closes the socket).
- Shutdown semantics: the `BoundListener` (owning the `TcpListener`) is owned by `AuthFlow::run`'s local scope and dropped deterministically at that function's return, on BOTH the success path and every error path (Rust ownership; std closes the socket on drop; no explicit `Drop` impl needed). Nothing survives the mode function.
- Accept is BLOCKING with no wall-clock deadline (the process waits for the user to open the printed URL; the TODO's "shut the callback listener down after completion or failure" is satisfied by completion/failure-driven shutdown, not a timer). Bounded protections: per-connection read timeout (5 seconds), request size cap (8 KiB), and a noise-connection budget of 8 after which the flow abandons (`CallbackAbandoned`).
- Socket constants (exact, in `listener.rs`):

```rust
const CALLBACK_HOST: &str = "127.0.0.1";
const NOISE_CONNECTION_LIMIT: usize = 8;
const REQUEST_READ_LIMIT: usize = 8192;
const REQUEST_READ_CHUNK: usize = 1024;
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(5);
const CALLBACK_ACCEPTED_STATUS: &str = "HTTP/1.1 200 OK";
const CALLBACK_REJECTED_STATUS: &str = "HTTP/1.1 400 Bad Request";
const CALLBACK_ACCEPTED_BODY: &str = "Authorization complete. You can close this window and return to the application.";
const CALLBACK_REJECTED_BODY: &str = "This callback port is only used by the running authorization flow.";
```

  plus one private byte constant holding the 4-byte HTTP header terminator (carriage return, line feed, carriage return, line feed), written as the corresponding Rust byte-string literal.

- HTTP responses sent to the browser (best-effort writes; a failed write never changes the already-decided parse outcome):

```rust
// status line, then exactly these headers, then the body:
//   Content-Type: text/plain
//   Content-Length: <body byte length>
//   Connection: close
// followed by one blank line and the body text.
```

Implemented as one helper `write_callback_response(stream: &mut TcpStream, status_line: &str, body: &str)` (2 params); write errors ignored via `let _ =`.

### D7 — Callback outcome classification (retry-vs-fail, exact)

Private enum in `listener.rs`:

```rust
enum CallbackOutcome {
    Authorized(AuthorizationCode),
    Declined { reason: String },
    StateMismatch,
    Noise,
}
```

Per-connection classification (`classify_connection(stream: &mut TcpStream, expected_state: &oauth2::CsrfToken) -> CallbackOutcome`, 2 params):

| Incoming outcome | Classification | Action in the accept loop |
| --- | --- | --- |
| `parse_authorization_callback` gives `Ok(Authorized { code, .. })` | `Authorized(code)` | respond 200 + accepted body; **success** |
| `Ok(Declined { reason })` | `Declined` | respond 400 + rejected body; **TERMINAL FAIL** (`ProviderDeclined`) |
| `Err(AuthorizationError::StateMismatch)` | `StateMismatch` | respond 400 + rejected body; **TERMINAL FAIL** |
| `Err(AuthorizationError::MissingParameter { .. })` | `Noise` | respond 400; budget decrement |
| request unreadable (timeout, EOF before header terminator, reset, over-size, non-UTF-8) | `Noise` | best-effort 400; budget decrement |
| request line has no `?` (e.g. `GET /favicon.ico ...`) | `Noise` | 400; budget decrement |
| `accept()` returns a transient error | `Noise` | budget decrement |

Encoded decisions:

- **StateMismatch is TERMINAL, never retried.** The only legitimate client (the user's browser following THIS run's URL) always carries the issued state; a parseable callback with a foreign state is attacker-controlled or stale traffic. Silently waiting for "the real callback" would mask the condition and keep the socket open, against the TODO shutdown rule. Failing fast closes the listener immediately.
- **ProviderDeclined is TERMINAL.** No authorization code will ever arrive for a declined request; the stderr names Google's error code (non-secret) and offers re-running the command.
- **Noise is retried under a bounded budget of 8.** Benign noise (port probes, favicon/robot requests, half-sent requests) carries no parseable callback query for this run; failing on the first probe would make the flow unusable on shared machines. At exhaustion the flow returns `CallbackAbandoned { attempts: NOISE_CONNECTION_LIMIT }`.
- HTTP method names are NOT checked; any request whose request line carries a parseable query string is classified per the table (documented in the module docs).

### D8 — Exchange, validation, persistence ordering (brief section 6.2 steps 6-9)

1. `code_exchange_request(client_config: &ClientConfig, opened: &OpenedFlow) -> CodeExchangeRequest` builds exactly:

```rust
CodeExchangeRequest {
    client_id: client_config.client_id().to_string(),
    client_secret: client_config.client_secret().clone(),
    code: opened.code.clone(),
    redirect_uri: opened.redirect_uri.clone(),
    pkce_verifier: opened.pkce_verifier.clone(),
}
```

   (the exchange runs AFTER the listener handling returned; the PKCE verifier moves through `OpenedFlow`).
2. Offline validation BEFORE persisting (never store insufficient credentials): `response.granted_scopes.grants(REQUIRED_SCOPE)` else `InsufficientScope`; then `response.refresh_token` present else `MissingRefreshToken`.
3. Persist via the injected store: `save_validated_credentials(&self, stored: &StoredCredentials) -> Result<(), AuthError> { self.store.save(stored).map_err(AuthError::Persistence) }`. The `StoredCredentials` is built by verbatim field moves from `TokenResponse` (persisted file schema = Task 2 D4: access token + TTL + refresh token + granted scopes; no clock use here — Task 4 owns expiry logic).
4. Print success line and return Ok (exit 0).

### D9 — Verification approach: granted-scope check ONLY this task; network verifier stays Task 4

- The flow verifies authorization offline: the token endpoint response's `scope` field lists the granted scopes of THIS access token (Task 1 plan section 1.1), so `granted_scopes.grants(REQUIRED_SCOPE)` IS scope verification: the minimal YouTube scope (`youtube.force-ssl`, researched and justified in Task 1 section 1.2) is confirmed present in what Google actually issued. The TODO bullet "verify the resulting credentials against the minimum YouTube API scope" is satisfied by this check.
- The Task-1 `verify.rs` seam (`AuthorizationVerifier`/`VerifiedAuthorization`, plan decision S9) remains UNWIRED: the seam's `verify(&AccessToken)` signature cannot express an offline granted-scope check (scopes live in `TokenResponse`, not in the access token); the concrete network verifier is Task 4's designed decision. The scope-evidence check lives in the app flow (`storeable_credentials` in `flow.rs`); NO code is added to `src/youtube/verify.rs` this task.

### D10 — Transport construction timing

`GoogleTokenEndpoint::new()` runs FIRST inside `auth_outcome` (before the client file read and before binding). Its failure (`TokenEndpointError`, e.g. endpoint-URL validation) maps to `AuthError::TokenExchange` and aborts with zero resources acquired. Justification: fail-fast; construction cannot observe network health anyway (ureq connects lazily), so early construction maximizes the config-surface coverage before anything opens.

```rust
fn create_transport() -> Result<GoogleTokenEndpoint, AuthError> {
    GoogleTokenEndpoint::new().map_err(AuthError::TokenExchange)
}
```

### D11 — No browser auto-launch, no interactive console input

The console prints instructions + URL (section 4.5) and nothing else. No `explorer`/`xdg-open`/`start` spawn (no new dependency, no cross-platform process handling; brief section 6.2 step 3 only says "Display the authorization URL in the console"). No stdin is ever read.

### D12 — Test-seam strategy (which seams use a real loopback HTTP GET vs pure mocks)

- **Real loopback HTTP GET** (sanctioned: tests bind only `127.0.0.1:0`, never leave the machine): the LISTENER tests (L1-L8) drive `bind()`/`wait_for_authorized_code` with `std::thread` browser threads that connect and send canned requests. This covers the full socket-plus-parse integration: happy path, state mismatch, declined, noise tolerance, budget exhaustion, invalid UTF-8, close-on-drop.
- **Pure fakes**: token transport fakes implementing `TokenEndpointTransport` (exchange success + typed failure, F7/F8); credential-store fakes implementing `CredentialStore` (recording fake = save reached, failing fake = persistence error, F9/F10); `tempfile::TempDir` client fixtures for the client-file failure classes and happy parse (F1-F3).
- **One full-flow integration test** (F11): a DECLINED provider callback end-to-end through `AuthFlow::run_with_listener` (section 3.2 seam). The test parses the tempdir client file (F1 fixture), binds its own listener (knows the port), spawns a browser thread dialing `GET /?error=access_denied` (no state knowledge needed for a declined query), calls `run_with_listener(&parsed_client, &bound)`, and asserts `Err(AuthError::ProviderDeclined { reason == "access_denied" })` plus that the recording store recorded NOTHING (no persistence on failure). The success path is deliberately NOT e2e-tested: the oauth2 `CsrfToken`/PKCE entropy cannot be pre-seeded offline without a production-type test hook (rejected); the success sequence is covered piecewise by L2 (happy listener with the test's own state), F7 (request assembly against a capturing fake transport), F1 (client file parse), F9 (store seam), and the Task-2 store round-trip tests.
- **Layering note (documented in listener/flow module docs)**: `src/app/auth` may reference `oauth2::CsrfToken` and `oauth2::PkceCodeVerifier` ONLY as opaque plumbing types passed through signatures; all Google-protocol logic stays in `src/youtube/`.

### D13 — Re-exports + staging retirement (exact)

`src/youtube/mod.rs` gains the re-export block (first-external-consumer record; per the module's documented policy). Keep ONLY re-exports the auth layer actually names (avoid unused-import warnings; if clippy flags one, trim it and record the deviation):

```rust
pub(crate) use authorization::{
    authorization_url, generate_pkce, generate_state, parse_authorization_callback,
    AuthorizationCallback, AuthorizationError, REQUIRED_SCOPE,
};
pub(crate) use client_config::{parse_client_config, ClientConfig};
pub(crate) use credentials::{CredentialStore, StoredCredentials};
pub(crate) use error::{ClientConfigError, CredentialStoreError, TokenEndpointError};
pub(crate) use file_credential_store::FileCredentialStore;
pub(crate) use google_token_endpoint::GoogleTokenEndpoint;
pub(crate) use token::{AuthorizationCode, TokenResponse};
pub(crate) use transport::{CodeExchangeRequest, TokenEndpointTransport};
```

Note: `AuthorizationError` is re-exported because the listener matches on its variants. `error.rs`'s `AuthorizationError` stays where it is (single enum source, Display untouched). `verify.rs` items are NOT re-exported (Task 4).

`src/main.rs`: REMOVE the `#[allow(dead_code)]` line; `mod youtube;` becomes plain. The Task-4-reserved surface then requires NARROW item-level allows (Phase-03 retirement precedent, `eaa12dc`-style):

| File | Item-level `#[allow(dead_code)]` on |
| --- | --- |
| `src/youtube/authorization.rs` | `const REVOCATION_ENDPOINT` (revocation belongs to a later phase) |
| `src/youtube/authorization.rs` | the `state: String` field of `AuthorizationCallback::Authorized` (the parser re-checks state equality internally; the flow never re-reads it) |
| `src/youtube/transport.rs` | `struct RefreshRequest` (Task 4 refresh path) |
| `src/youtube/token.rs` | `struct TokenEndpointBody` and `fn parse_token_response` (the ureq path parses via oauth2's BasicTokenResponse; the serde parser is the offline/Task-4 seam) |
| `src/youtube/error.rs` | `enum VerificationError` (Task 4 verifier errors) |
| `src/youtube/verify.rs` | `struct VerifiedAuthorization`, its `fn new`, its `fn granted_scopes`, and `trait AuthorizationVerifier` (Task 4 seam) |

Verification: `git grep -n "#\[allow(" src/` must then show ONLY those narrow allows (no module-wide allow anywhere). If `cargo check` reports MORE dead items, add item-level allows in the same narrow pattern and REPORT the deviation in the implementing summary (never re-add a module-wide allow).

### D14 — Modes/dispatch/CLI wiring (exact edits)

- `src/app/modes.rs`: DELETE `pub(super) fn auth` + its doc comment; DELETE the test `auth_reports_not_implemented_exit_code`; REWRITE the file doc-header to describe only the `run` placeholder. **`run` handler, `RUN_NOT_IMPLEMENTED_MESSAGE`, `NOT_IMPLEMENTED_EXIT`, `run_reports_not_implemented_exit_code`, `exit_codes_remain_distinct_and_stable`, and both run-message tests stay BYTE-IDENTICAL** (the header rewrite is the only file-level doc change).
- `src/app/mod.rs`:
  - add `mod auth;` between `mod cli;` and `mod demo;`;
  - dispatch arm becomes `cli::Mode::Auth => auth::run_auth(app_config),` (the `run`/`demo` arms unchanged);
  - update the module-doc exit-code sentence so it no longer says auth is a placeholder; new wording: exit codes are `1` startup/config failures and `1` for demo- and auth-origin fatal failures reported through their handlers, `2` usage, `3` the `run` mode not implemented.
- `src/app/cli.rs`: in `USAGE`, replace ONLY the description text after the existing `  auth                 ` padding with `authorize YouTube access and store OAuth credentials`; every other usage line stays byte-identical. CLI SYNTAX unchanged (command names, flag forms; TODO bullet 6 satisfied).
- `src/app/cli_tests.rs`: add `usage_text_advertises_the_auth_command` asserting the new description (no existing test references the auth line).
- `src/main.rs`: only the staging-line retirement (D13).

### D15 — Logging (bounded; this list IS the submodule's field whitelist)

All `tracing::info!` (default filter). NO token/code/URL/secret values, and NO credential-path values in log FIELDS (paths appear on STDOUT in the success print only — demo `print_final_path` precedent; the startup funnel's field whitelist is untouched).

| # | Event | Site | Fields |
| --- | --- | --- | --- |
| E1 | `info!("authorization flow started")` | first line of `AuthFlow::run_with_listener` | none |
| E2 | `info!(redirect_port = %port, "callback listener bound")` | `listener::bind` after a successful bind | `redirect_port` (u16 runtime value) |
| E3 | `info!("authorization callback accepted")` | `Authorized` outcome | none |
| E4 | `info!("authorization code exchanged")` | after transport success | none |
| E5 | `info!("required YouTube scope granted")` | after scope + refresh validation passes | none |
| E6 | `info!("credentials persisted")` | after `save` success | none |

NEVER logged: the authorization URL (contains state + client_id query values), the callback request line, any exchange result values, token/secret values, the credential file path as a field. Failures are NOT tracing events: the single stderr `error:` line via `report_auth_failure` is the report (funnel precedent; no duplicate cross-layer reporting).

### D16 — Redaction guarantees

- Secrets exist only inside youtube-layer wrapper types; `src/app/auth` never calls `.secret()` on `AccessToken`/`RefreshToken`/`AuthorizationCode` (the exchange request receives clones of the wrappers; nothing formats them).
- Every `AuthError` variant Debug + Display must never contain the fixture secret strings (`client_secret_fixture`, `access_token_fixture`, `refresh_token_fixture`, `authorization_code_fixture`, the PKCE verifier fixture) or any callback query echo. `StateMismatch` has no payload; `ProviderDeclined` carries Google's error code only; the HTTP response bodies are fixed constants that never echo the request.

## 3. Exact type/signature design (signatures ARE the design; implementer fills bodies per section 2)

### 3.1 `src/app/auth/mod.rs` — entry + error type

```rust
//! The `auth` mode handler: runs the installed-application OAuth flow with a
//! loopback-only callback listener. Module docs must cover: the ordered flow
//! (section 1 diagram), the exit-code contract (D3), the listener lifecycle
//! guarantees (D6), redaction rules (D16), the no-auto-launch rule (D11).

use std::fmt;
use std::path::PathBuf;

use crate::app::error::STARTUP_FAILURE_EXIT;
use crate::config::AppConfig;
use crate::youtube::{CredentialStore, FileCredentialStore, GoogleTokenEndpoint, TokenEndpointError};

mod console;
mod flow;
mod listener;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;

/// Exit code returned by the auth mode on success.
const AUTH_SUCCESS_EXIT: u8 = 0;

// ... AuthError enum (D1) + the two fixed message consts + Display + Error impl ...

/// Runs the OAuth authorization flow: 0 after verified persistence, or one
/// redacted stderr `error:` line plus exit 1 on any failure.
pub(super) fn run_auth(config: &AppConfig) -> u8 {
    match auth_outcome(config) {
        Ok(()) => AUTH_SUCCESS_EXIT,
        Err(error) => report_auth_failure(&error),
    }
}

fn auth_outcome(config: &AppConfig) -> Result<(), AuthError> {
    let transport = create_transport()?;
    let store = create_store(config);
    let flow = AuthFlow {
        config,
        transport: &transport,
        store: &store,
    };
    flow.run()
}

fn create_transport() -> Result<GoogleTokenEndpoint, AuthError> {
    GoogleTokenEndpoint::new().map_err(AuthError::TokenExchange)
}

fn create_store(config: &AppConfig) -> FileCredentialStore {
    FileCredentialStore::new(PathBuf::from(&config.oauth.credentials_path))
}

fn report_auth_failure(error: &AuthError) -> u8 {
    eprintln!("error: {error}");
    error.exit_code()
}
```

### 3.2 `src/app/auth/flow.rs` — the ordered flow (FINAL shape; no cfg-gated struct fields)

```rust
use std::fs;
use std::path::Path;

use oauth2::PkceCodeVerifier;    // plumbing type per D12
use tracing::info;

use crate::config::AppConfig;
use crate::youtube::{
    authorization_url, generate_pkce, generate_state, parse_client_config,
    AuthorizationCode, ClientConfig, CodeExchangeRequest, CredentialStore,
    REQUIRED_SCOPE, StoredCredentials, TokenEndpointTransport, TokenResponse,
};

use super::console;
use super::listener;
use super::AuthError;

/// One parameterization of the auth flow: the already-validated app config,
/// the token-endpoint transport (real in production, fake in tests) and the
/// credential store. The struct exists because three inputs would exceed the
/// two-argument function cap.
pub(super) struct AuthFlow<'a> {
    pub(super) config: &'a AppConfig,
    pub(super) transport: &'a dyn TokenEndpointTransport,
    pub(super) store: &'a dyn CredentialStore,
}

/// Code + PKCE verifier + redirect URI delivered by the listener, with the
/// listener ALREADY closed (the value moves here only after the wait ended).
struct OpenedFlow {
    code: AuthorizationCode,
    pkce_verifier: PkceCodeVerifier,
    redirect_uri: String,
}

impl AuthFlow<'_> {
    /// Production entry: reads + parses the client config FIRST, then binds
    /// the loopback listener, then runs the ordered flow (D5 ordering). The
    /// listener drops deterministically at this function's return, whatever
    /// the outcome is (D6).
    pub(super) fn run(&self) -> Result<(), AuthError> {
        let client_config = read_client_config(&self.config.oauth.client_config_path)?;
        let bound = listener::bind()?;
        self.run_with_listener(&client_config, &bound)
    }

    /// Test-visible seam: the whole ordered flow against an already-parsed
    /// client config and a caller-supplied listener. Production calls only
    /// `run`; tests call this directly with a prebound listener and a
    /// tempdir-parsed client config (F11). The body order is the flow
    /// contract: open authorization, exchange, validate, persist, print.
    pub(super) fn run_with_listener(
        &self,
        client_config: &ClientConfig,
        bound: &listener::BoundListener,
    ) -> Result<(), AuthError> {
        info!("authorization flow started");
        let opened = self.open_authorization(client_config, bound)?;
        let response = self.exchange_authorization_code(client_config, &opened)?;
        let stored = storeable_credentials(response)?;
        self.save_validated_credentials(&stored)?;
        info!("credentials persisted");
        console::print_success(&self.config.oauth.credentials_path);
        Ok(())
    }

    /// Builds the URL, prints it, waits for the authorized callback. The
    /// client config was READ+PARSED before this runs (D5 ordering); `bound`
    /// is borrowed and stays alive for the accept loop, dropping when `run`
    /// returns (success AND failure) — D6 shutdown semantics.
    fn open_authorization(
        &self,
        client_config: &ClientConfig,
        bound: &listener::BoundListener,
    ) -> Result<OpenedFlow, AuthError> {
        let state = generate_state();
        let pkce = generate_pkce();
        let url = authorization_url(
            client_config,
            bound.redirect_uri(),
            &state,
            &pkce,
        );
        console::print_authorization_url(url.as_str(), bound.redirect_uri());
        let code = bound.wait_for_authorized_code(&state)?;
        Ok(OpenedFlow {
            code,
            pkce_verifier: pkce.verifier,
            redirect_uri: bound.redirect_uri().to_string(),
        })
    }

    fn exchange_authorization_code(
        &self,
        client_config: &ClientConfig,
        opened: &OpenedFlow,
    ) -> Result<TokenResponse, AuthError> {
        let request = code_exchange_request(client_config, opened);
        let response = self
            .transport
            .authorization_code_exchange(&request)
            .map_err(AuthError::TokenExchange)?;
        info!("authorization code exchanged");
        Ok(response)
    }

    fn save_validated_credentials(&self, stored: &StoredCredentials) -> Result<(), AuthError> {
        self.store.save(stored).map_err(AuthError::Persistence)
    }
}

/// Reads + parses the configured client-secret file (D5; one param).
pub(super) fn read_client_config(client_config_path: &str) -> Result<ClientConfig, AuthError> {
    let client_path = Path::new(client_config_path);
    let text = fs::read_to_string(client_path).map_err(|source| AuthError::ClientConfigFile {
        path: client_config_path.to_string(),
        source,
    })?;
    parse_client_config(&text).map_err(AuthError::ClientConfig)
}

/// Verifies the exchanged response carries the required scope and a refresh
/// token, then moves its fields into the persisted credential shape (D8).
pub(super) fn storeable_credentials(response: TokenResponse) -> Result<StoredCredentials, AuthError> {
    if !response.granted_scopes.grants(REQUIRED_SCOPE) {
        return Err(AuthError::InsufficientScope);
    }
    let refresh_token = match response.refresh_token {
        Some(refresh_token) => refresh_token,
        None => return Err(AuthError::MissingRefreshToken),
    };
    info!("required YouTube scope granted");
    Ok(StoredCredentials {
        access_token: response.access_token,
        expires_in_seconds: response.expires_in_seconds,
        refresh_token: Some(refresh_token),
        granted_scopes: response.granted_scopes,
    })
}

/// Builds the code-exchange request from the validated client config and the
/// opened flow (D8; two params — captured by the F7 fake-transport test).
pub(super) fn code_exchange_request(
    client_config: &ClientConfig,
    opened: &OpenedFlow,
) -> CodeExchangeRequest {
    CodeExchangeRequest {
        client_id: client_config.client_id().to_string(),
        client_secret: client_config.client_secret().clone(),
        code: opened.code.clone(),
        redirect_uri: opened.redirect_uri.clone(),
        pkce_verifier: opened.pkce_verifier.clone(),
    }
}
```

`OpenedFlow` stays private to `flow.rs` (the sibling `flow_tests` module can construct it for F7/F8).

### 3.3 `src/app/auth/listener.rs` — socket boundary

```rust
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use oauth2::CsrfToken;    // plumbing type per D12
use tracing::info;

use crate::youtube::{
    parse_authorization_callback, AuthorizationCallback, AuthorizationCode, AuthorizationError,
};

use super::AuthError;

// ... constants from D6 ...

/// A loopback-bound listener plus the redirect URI it serves. Dropping this
/// value closes the socket; the auth flow holds it exactly for the duration
/// of the authorization wait.
pub(super) struct BoundListener {
    listener: TcpListener,
    redirect_uri: String,
}

/// Binds on 127.0.0.1 with an ephemeral port (D6); emits the bound event.
pub(super) fn bind() -> Result<BoundListener, AuthError> {
    let listener = TcpListener::bind((CALLBACK_HOST, 0)).map_err(AuthError::ListenerBind)?;
    let address: SocketAddr = listener.local_addr().map_err(AuthError::ListenerBind)?;
    info!(redirect_port = %address.port(), "callback listener bound");
    Ok(BoundListener {
        listener,
        redirect_uri: format!("http://{CALLBACK_HOST}:{}", address.port()),
    })
}

impl BoundListener {
    pub(super) fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Blocking accept loop: returns the authorized code on success, a typed
    /// AuthError for terminal outcomes, and keeps waiting through noise up to
    /// the budget (D7). The budget counter is local state incrementing once
    /// per noise-classified connection (or transient accept error).
    pub(super) fn wait_for_authorized_code(
        &self,
        expected_state: &CsrfToken,
    ) -> Result<AuthorizationCode, AuthError> {
        let mut noise_count = 0usize;
        loop {
            if let Some(code) = self.handle_next_connection(expected_state, &mut noise_count)? {
                return Ok(code);
            }
        }
    }

    /// Accepts one connection, classifies it, responds, and maps the result.
    /// Ok(None) = the connection was noise and the budget still holds.
    fn handle_next_connection(
        &self,
        expected_state: &CsrfToken,
        noise_count: &mut usize,
    ) -> Result<Option<AuthorizationCode>, AuthError> {
        let mut stream = match self.listener.accept() {
            Ok((stream, _)) => stream,
            Err(_) => return discard_benign_connection(noise_count),
        };
        let classification = classify_connection(&mut stream, expected_state);
        match classification {
            CallbackOutcome::Authorized(code) => {
                info!("authorization callback accepted");
                write_callback_response(&mut stream, CALLBACK_ACCEPTED_STATUS, CALLBACK_ACCEPTED_BODY);
                Ok(Some(code))
            }
            CallbackOutcome::Declined { reason } => {
                write_callback_response(&mut stream, CALLBACK_REJECTED_STATUS, CALLBACK_REJECTED_BODY);
                Err(AuthError::ProviderDeclined { reason })
            }
            CallbackOutcome::StateMismatch => {
                write_callback_response(&mut stream, CALLBACK_REJECTED_STATUS, CALLBACK_REJECTED_BODY);
                Err(AuthError::StateMismatch)
            }
            CallbackOutcome::Noise => {
                write_callback_response(&mut stream, CALLBACK_REJECTED_STATUS, CALLBACK_REJECTED_BODY);
                discard_benign_connection(noise_count)
            }
        }
    }
}

// helpers (each NOT exceeding 50 lines, 2 params max):

/// Bumps the noise counter and fails the flow when the budget is exhausted.
fn discard_benign_connection(noise_count: &mut usize) -> Result<Option<AuthorizationCode>, AuthError> {
    *noise_count += 1;
    if is_noise_budget_exhausted(*noise_count) {
        return Err(AuthError::CallbackAbandoned { attempts: *noise_count });
    }
    Ok(None)
}

/// Single-section boolean predicate (rule).
fn is_noise_budget_exhausted(noise_count: usize) -> bool {
    noise_count >= NOISE_CONNECTION_LIMIT
}

enum CallbackOutcome {
    Authorized(AuthorizationCode),
    Declined { reason: String },
    StateMismatch,
    Noise,
}

/// classifies one already-accepted connection (D7).
fn classify_connection(stream: &mut TcpStream, expected_state: &CsrfToken) -> CallbackOutcome {
    let request_text = match read_request_text(stream) {
        Some(text) => text,
        None => return CallbackOutcome::Noise,
    };
    let query = match callback_query(&request_text) {
        Some(query) => query,
        None => return CallbackOutcome::Noise,
    };
    match parse_authorization_callback(query, expected_state) {
        Ok(AuthorizationCallback::Authorized { code, .. }) => CallbackOutcome::Authorized(code),
        Ok(AuthorizationCallback::Declined { reason }) => CallbackOutcome::Declined { reason },
        Err(AuthorizationError::StateMismatch) => CallbackOutcome::StateMismatch,
        Err(AuthorizationError::MissingParameter { .. }) => CallbackOutcome::Noise,
    }
}

/// Reads the request head bytes with the read timeout set; returns None for
/// anything that never terminates its headers within the size limit.
fn read_request_text(stream: &mut TcpStream) -> Option<String> {
    let _ = stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT));
    let mut received: Vec<u8> = Vec::new();
    let mut chunk = [0u8; REQUEST_READ_CHUNK];
    loop {
        if head_is_complete(&received) {
            return String::from_utf8(received).ok();
        }
        match stream.read(&mut chunk) {
            Ok(0) => return None,               // EOF before the header terminator
            Ok(read_count) => received.extend_from_slice(&chunk[..read_count]),
            Err(_) => return None,              // timeout, reset or other I/O failure
        }
        if received.len() > REQUEST_READ_LIMIT {
            return None;
        }
    }
}

/// Single-section predicate over the buffered bytes.
fn head_is_complete(received: &[u8]) -> bool {
    received.ends_with(HEADER_TERMINATOR_BYTES)
}

/// Extracts the query part of the request line ("GET /?a=b HTTP/1.1").
fn callback_query(request_text: &str) -> Option<&str> {
    let request_line = request_text.lines().next()?;
    let path = request_line.split_whitespace().nth(1)?;
    let separation = path.find('?')?;
    Some(&path[separation + 1..])
}

/// Writes one fixed-content HTTP response; failures are intentionally
/// ignored (the parse outcome was already decided before this call).
fn write_callback_response(stream: &mut TcpStream, status_line: &str, body: &str) {
    let response = format!(
        "{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}
```

Implementation notes (LOCKED):

- `HEADER_TERMINATOR_BYTES` is the private byte constant from D6 (the four-byte CRLF CRLF sequence, written as the corresponding Rust byte-string literal).
- `IpAddr`/unused imports must be pruned to keep clippy clean at `-D warnings`.
- `format!` with two exponent inputs (the implicit `format_args` capture for `{status_line}`/`{body}` and the positional `body.len()`) must be expressed however the 1.82 compiler accepts — if the mixed implicit/positional form errors, switch the content-length pair to a pre-computed `let content_length = body.len();` and inline `{content_length}`.
- No method-enforcement, no Host validation, no keep-alive handling (D7 note); the response always uses `Connection: close` so the browser closes its side.
- The read-timeout path (a connection that never completes its headers) is exempt from automated tests (a 5-second wait would slow the suite) — documented limitation in the test module docs.

### 3.4 `src/app/auth/console.rs` — exact output (see section 4.5)

```rust
/// Console lines of the auth flow. Deliberately NOT tracing events and NOT
/// test-assertable through the tracing layer: plain stdout prints so the
/// instructions reach the interactive operator.

pub(super) fn print_authorization_url(url: &str, redirect_uri: &str) {
    println!("To authorize YouTube access, open the following URL in a browser and approve the authorization prompt:");
    println!();
    println!("{url}");
    println!();
    println!("The application is waiting for the browser callback at {redirect_uri} (loopback only).");
}

pub(super) fn print_success(credentials_path: &str) {
    println!("authorization succeeded: YouTube credentials saved at {credentials_path}; the required YouTube scope was granted");
}
```

## 4. Remaining behavioral specs

### 4.1 Exit-code table — see D3 (table governs; `run` keeps 3)

### 4.2 Listener lifecycle — see D6/D7 (bind, budget, shutdown, constants govern)

### 4.3 Logging — see D15 (event list governs)

### 4.4 Test-seam strategy — see D12

### 4.5 Console output spec (verbatim; the operator sees exactly these lines)

Descriptor output printed to STDOUT before waiting:

```
To authorize YouTube access, open the following URL in a browser and approve the authorization prompt:

<the authorization URL, e.g. https://accounts.google.com/o/oauth2/v2/auth?client_id=...&response_type=code&scope=...&redirect_uri=http%3A%2F%2F127.0.0.1%3A52144&state=...&code_challenge=...&code_challenge_method=S256>

The application is waiting for the browser callback at http://127.0.0.1:52144 (loopback only).
```

Descriptor output printed to STDOUT after verified persistence (exit 0 follows):

```
authorization succeeded: YouTube credentials saved at <oauth.credentials_path value>; the required YouTube scope was granted
```

Fatal-path output (STDERR, single line, exit 1): the generic `error: ` prefix from `report_auth_failure` plus the D2 Display of the `AuthError` variant. Nothing else is ever printed; the authorization URL is NEVER a tracing field (D15).

## 5. Test plan (deterministic, offline; NO real Google, NO external network, NO wall clock)

All tests live as `#[cfg(test)] #[path = "..."]` sibling modules of their source file (repo pattern). Tests may use `unwrap`/`expect`, `std::thread`, `tempfile::TempDir`, and `127.0.0.1:0` sockets. All existing 195 tests must keep passing UNCHANGED except the one deleted placeholder test (`auth_reports_not_implemented_exit_code`).

Browser-thread helper (shared inside `listener_tests.rs`; exact helper, the implementer writes the two real CRLF byte pairs where marked):

```rust
fn send_request(port: u16, request_bytes: &[u8]) {
    std::thread::spawn(move || {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port))
            .expect("the callback listener is accepting");
        std::io::Write::write_all(&mut stream, request_bytes).expect("the request is sent");
        // optionally read the response on the test side to assert status lines
    });
}

fn get_request_bytes(query: &str) -> Vec<u8> {
    // request line + CRLF + "Host: 127.0.0.1" + CRLF + CRLF
    // e.g. for query "code=x" the line is "GET /?code=x HTTP/1.1"
}
```

Tests asserting responses may read the reply bytes from the browser-thread side (`read_to_end` after `Connection: close`) and assert the exact status line constants.

Fixture secrets (module consts mirroring the Task-1/2 conventions): `access_token_fixture`, `refresh_token_fixture`, `client_secret_fixture`, `authorization_code_fixture`, pkce verifier fixture string of 43+ chars.

### 5.1 `src/app/auth/mod_tests.rs`

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| M1 | `run_auth_reports_missing_client_file_with_startup_failure_exit` | fixture config (from `crate::config::valid_config()` with `oauth.client_config_path` redirected to a nonexistent tempdir path) passed to `run_auth` returns exactly `STARTUP_FAILURE_EXIT` (1) |
| M2 | `every_auth_error_maps_to_the_startup_failure_exit` | all ten `AuthError` variants constructed return `exit_code() == STARTUP_FAILURE_EXIT` |
| M3 | `auth_error_display_lines_are_stable` | per-variant `assert_eq`/`starts_with` snapshot of every Display line (exact D2 texts incl. prefix `error: ` behavior delegated to the reporter — assertions here are on `format!("{e}")` WITHOUT the prefix) |
| M4 | `auth_error_debug_and_display_reveal_no_secret_fixtures` | variants built with fixture-secret-bearing payloads (e.g. `TokenExchange(Server { code: "invalid_grant", .. })` next to a fake response string containing `access_token_fixture`; `ClientConfigFile` with a path) produce Debug + Display strings that contain NONE of the fixture secret strings, no callback query echo |

### 5.2 `src/app/cli_tests.rs` (added case)

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| C1 | `usage_text_advertises_the_auth_command` | `USAGE` contains `authorize YouTube access and store OAuth credentials` |

### 5.3 `src/app/auth/flow_tests.rs` (+ optional `flow_exchange_tests.rs` split per D4)

Fakes (final shapes; hand-built values, no serde in the fakes):

```rust
struct RecordingStore { records: std::cell::RefCell<Vec<StoredCredentials>> }  // CredentialStore impl records every save
struct FailingStore;                                                          // save returns Err(FileWrite { fixed path + fixed io error })
struct OkTransport { response: TokenResponse }                                // authorization_code_exchange answers Ok(response fields moved)
struct CapturingOkTransport { response: TokenResponse,                       // also captures the CodeExchangeRequest it received
                              captured: std::cell::RefCell<Option<CapturedRequest>> }
struct CapturingFailingTransport { captured: std::cell::RefCell<Option<CapturedRequest>> } // captures, then Err(Server { code: "invalid_grant", description: None })
```

`CapturedRequest` is a small test struct holding the non-secret captured facts (`client_id: String`, `redirect_uri: String`, a bool marking code/verifier wrappers present — never their values). The transports implement `TokenEndpointTransport` by moving field pairs out of the response. `load()` on both stores returns the fixed `Err(CredentialStoreError::ContentNotAvailable)` (load is unused by the auth flow; the fake only needs the trait satisfied).

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| F1 | `client_file_happy_parse_returns_client_config` | tempdir file with the Task-1/2 Google-shaped JSON (fixture seen in section 0) through `read_client_config` is `Ok(ClientConfig)` |
| F2 | `client_file_read_failure_is_the_client_config_class` | nonexistent tempdir path returns `Err(AuthError::ClientConfigFile { .. })` and its Display starts `unable to read the OAuth client configuration file` |
| F3 | `client_file_invalid_content_is_the_client_config_class` | file containing invalid JSON returns `Err(AuthError::ClientConfig(_))`; Display is the inner verbatim line |
| F4 | `stored_response_yields_storeable_credentials` | a hand-built `TokenResponse` (access + refresh + required scope + TTL 3592) maps to `Ok(StoredCredentials)` with verbatim field moves (fields compared via the wrapper-free pub(crate) fields; secrets compared only in unsensitive ways — see note) |
| F5 | `storeable_credentials_rejects_missing_required_scope` | response whose granted scopes lack the required URL gives `Err(AuthError::InsufficientScope)`; Display contains the full required scope URL |
| F6 | `storeable_credentials_rejects_missing_refresh_token` | response without refresh token gives `Err(AuthError::MissingRefreshToken)` |
| F7 | `code_exchange_request_assembles_the_documented_fields` | `exchange_authorization_code` on an `AuthFlow` holding a parsed-`ClientConfig` + hand-built `OpenedFlow` against `CapturingOkTransport` produces a captured request whose `client_id`/`redirect_uri` strings equal the source values and whose code/pkce_verifier wrappers are present; the method result is the transport's Ok response |
| F8 | `failing_exchange_maps_to_token_exchange_error` | the same machinery but the fake returns `Err(TokenEndpointError::Server { code: "invalid_grant", description: None })`; the flow method result is `Err(AuthError::TokenExchange(_))` and the inner Display contains `invalid_grant` but NOT any fixture secret |
| F9 | `saved_credentials_reach_the_store_seam` | `save_validated_credentials` against the `RecordingStore` records exactly one save whose stored credentials equal the prepared one (equality expressed per-field) |
| F10 | `store_errors_map_to_the_persistence_class` | `save_validated_credentials` against `FailingStore` gives `Err(AuthError::Persistence(_))` |
| F11 | `declined_provider_flow_ends_without_persisting_end_to_end` | full `run_with_listener(&parsed_tempdir_client_config, &prebound_listener)` + browser thread sending `GET /?error=access_denied HTTP/1.1 (+CRLF+CRLF)`: result is `Err(AuthError::ProviderDeclined { reason == "access_denied" })`, the `RecordingStore` recorded nothing, and the browser could read the 400 status line |

Secrecy note for F7/F8: assertions compare NON-secret facts (client_id, redirect_uri, captured-wrapper presence) or wrapper-internal equality via `.secret()`; never embed a fixture secret inside an assert message.

### 5.4 `src/app/auth/listener_tests.rs` (real loopback sockets)

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| L1 | `bind_uses_loopback_and_ephemeral_port` | `bind()` gives `redirect_uri()` exactly starting with `http://127.0.0.1:`; the port parses as u16 greater than 0; NEVER contains `localhost` |
| L2 | `authorized_callback_delivers_the_code` | test generates its own state (`generate_state()`), browser thread sends `GET /?code=authorization_code_fixture&state=<state>`; `wait_for_authorized_code` returns the code whose secret equals the fixture (assert without formatting into a message); the browser saw `HTTP/1.1 200 OK` and the accepted body text |
| L3 | `declined_callback_fails_the_flow` | browser sends `GET /?error=access_denied`; wait returns `Err(AuthError::ProviderDeclined { reason == "access_denied" })` |
| L4 | `state_mismatch_callback_fails_the_flow` | browser sends code + a foreign state; wait returns `Err(AuthError::StateMismatch)`; its Debug/Display contains no fixture values |
| L5 | `noise_connections_are_ignored_until_a_valid_callback` | two noise connections first (`GET /favicon.ico HTTP/1.1` and garbage bytes forming a complete head), then a valid authorized callback with the right state; wait still returns the code |
| L6 | `exhausted_noise_budget_abandons_the_flow` | `NOISE_CONNECTION_LIMIT + 1` sequential noise connections; wait returns `Err(AuthError::CallbackAbandoned { attempts: NOISE_CONNECTION_LIMIT })` |
| L7 | `invalid_utf8_request_is_treated_as_noise` | a connection whose bytes include an invalid UTF-8 sequence (composed with `0xFF` raw byte) completes as noise, and (optionally validated in the same test) a follow-up valid callback still succeeds |
| L8 | `listener_socket_is_closed_when_dropped` | bind; take port; `TcpStream::connect` succeeds while bound; drop the `BoundListener`; a fresh connect to the same port fails (assert refused/errored — port reused only with astronomically-unlikely OS collision); this is the listener-shutdown-guarantee test |

Test-module doc note (required): the read-timeout path (D7 noise) is untested on purpose (a 5-second suite slowdown); port-collision flakiness risk acknowledged for L8/L1 as negligible on the containerized Linux runner.

### 5.5 Expected suite arithmetic

195 (current) minus 1 (deleted modes placeholder test) plus 24 (M1-M4, C1, F1-F11, L1-L8) = **218 expected tests**. Record the OBSERVED total; never adjust tests to hit the arithmetic (Task-2 L6 precedent). If a planned test is impossible to implement as specified, STOP and ask the caller instead of inventing a substitute.

## 6. Implementation order for the implementer (with git actions + commit points)

All work happens on `feat/phase04-oauth-credentials`. Host is Windows: every cargo action runs through the Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`). Host git is used ONLY for status/add/commit/diff. Standard fast loop per step: `cargo fmt` (fold any rewrap into that step's commit), then fmt --check, check --locked, targeted test, clippy.

### Step 0 — plan-tracking commit

- `git status --porcelain` must show ONLY this plan file (untracked). Commit: `docs: track task 3 implementation plan`.

### Step 1 — app submodule + youtube re-exports + wiring (ONE commit; staging line stays this commit)

Edits/files (exactly):
- NEW `src/app/auth/mod.rs`, `flow.rs`, `listener.rs`, `console.rs`, `mod_tests.rs`, `flow_tests.rs`, `listener_tests.rs` per sections 2-5.
- `src/youtube/mod.rs`: append the re-export block (D13) after the submodule declarations.
- `src/app/modes.rs`: delete the `auth` handler + its doc comment + `auth_reports_not_implemented_exit_code`; rewrite the file doc-header (run-only).
- `src/app/mod.rs`: `mod auth;` + dispatch arm replacement + module-doc exit-code sentence update (D14).
- `src/app/cli.rs`: USAGE auth description line (D14).
- `src/app/cli_tests.rs`: add C1.
- `src/main.rs`: UNTOUCHED this step (staging line stays while the re-exported-but-not-yet-narrowed surface is still covered).

Verify (each via the VM MCP, exit status authoritative):
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` (fix by running `cargo fmt` in-container first and folding the diff in)
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked app::` then `cargo test --locked youtube`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`

Gitignore compliance pre-commit: `git status --porcelain` lists EXACTLY the intended files (no `credentials/`, no `config/config.json`, no logs). Commit: `feat: replace the auth placeholder with the oauth authorization flow`.

### Step 2 — staging-line retirement + narrow allows (ONE commit)

- `src/main.rs`: remove the `#[allow(dead_code)]` line from `mod youtube;`.
- Run `docker ... cargo check --locked`; the reported dead items MUST match the D13 table exactly. Add the narrow item-level `#[allow(dead_code)]` there (with one-line reasons in the attribute's doc comment position or plain comments adjacent — no commented-out code means real explanatory comments are fine).
- If additional dead items appear beyond the table: STOP-style doc — add the item-level allow in the same pattern AND record the deviation in the implementing summary.
- Verify again: fmt --check, check --locked, test --locked (full), clippy -D warnings (in-container), plus host `git grep -n "#\[allow(" src/` producing ONLY the narrow set.
- Commit: `chore: retire the youtube module staging allowance`.

### Step 3 — full gate + hygiene (no code change; evidence step)

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` must exit 0 with four PASS lines and `ALL CHECKS PASSED`; record the observed test total (expect ~218) and the fresh log path.
- Host checks (gitignore-compliance rule): `git status --porcelain` empty (or only caller-owned plan/adherence docs); `git check-ignore -v credentials/client_secret.json credentials/tokens.json config/config.json` unchanged versus the Task-2 evidence; `git ls-files credentials/` still only `.gitkeep`.
- Working-tree rule caps self-check: `wc -l` on every touched file within 200; no `#[allow(` beyond the D13 narrow set; no leftover tmp files anywhere in the repo tree.

## 7. Verification command series (V-series; Alpine VM MCP unless noted; exit status is authoritative)

| # | Command | Expected |
| --- | --- | --- |
| V1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` | exit 0 |
| V2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` | exit 0 |
| V3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` | exit 0, all pass, observed total recorded (~218) |
| V4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked app::youtube -- --test-threads=4` | optional targeted fast loop during development |
| V5 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` | exit 0 |
| V6 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | exit 0, PASS fmt-check/check/test/clippy, `ALL CHECKS PASSED`, fresh log path recorded |
| V7 | (host) `git status --porcelain` | exactly the intended files per step |
| V8 | (host) `git grep -n "#\[allow(" src/` | only the D13 narrow set |
| V9 | (host) `git check-ignore -v credentials/client_secret.json credentials/tokens.json config/config.json` | unchanged rule hits (gitignore-compliance) |
| V10 | (host) `git ls-files credentials/` | `.gitkeep` only |

BUILD artifacts are NOT rebuilt by this task (build-linux/build-windows belong to Task 5's evidence step).

## 8. Rule-cap checklist (enforced at every step + final)

| Rule | Compliance action |
| --- | --- |
| File within 200 lines | per-file projections D4; `wc -l` after steps 1-2; split hook defined in D4 for test files |
| Fn body within 50 lines | split into helpers per section 3; count excludes signature |
| Depth within 2 | match arms 1 level deep; 3rd level into helpers |
| Params within 2 non-receiver | helpers shaped per section 3; three-input contexts encapsulated in `AuthFlow`/`CodeExchangeRequest` |
| Single-section booleans | `is_noise_budget_exhausted`, `head_is_complete` predicate helpers |
| Hand-written Display, no thiserror | D2; no `Debug` formatting leaks (D16) |
| No panics/unwrap/expect outside tests | section 3 code uses `map_err`/`ok_or` patterns; NO `?`-missing sites, no `unwrap_or_default` on secrets |
| No commented-out code | explanatory comments only; removals use git history |
| No new dependencies | oauth2/ureq/tempfile already locked; no Cargo.toml edit |
| Newline rule (real newlines) | all multi-line file writes use real line breaks; no literal escape sequences in written files |

## 9. Risks / documented limitations (pre-registered)

- L1: the blocking accept waits indefinitely for the user (D6 justification; caller-sanctioned "until a successful parse"); no wall-clock deadline exists.
- L2: the read-timeout noise path is untested (5-second suite cost); covered by design review only (test module docs record it).
- L3: L8's after-drop connect-failure assertion has a theoretical port-reuse race on the shared container; declared negligible plus the assertion is written tolerance-style (`is_err`, not a specific `ConnectionRefused` kind).
- L4: the F11 e2e test dials the prebound listener BEFORE `run_with_listener` begins accepting; the kernel accept backlog holds the connection deterministically (loopback). If observed flaky on the container runner, the fix pattern is: the browser thread performs a bounded local retry connect loop (max 25 attempts, 20 ms sleep) — allowed as a TEST-ONLY helper change, never in production code.
- L5: oauth2 `PkceCodeVerifier`/`CsrfToken` referenced from `src/app/auth` (plumbing types, D12 note) — a deliberate, documented narrow exception to pure layering; no Google-protocol logic exists in the app layer.
- L6: `GoogleTokenEndpoint`'s `with_token_url` remains cfg-test-gated (Task 1); untouched by this task.
- L7: expected test total 218 is arithmetic guidance, not an acceptance criterion (record observed).
- L8: rustfmt may reflow snippets; hard caps still govern (Task-3 Phase-03 precedent).
- L9: this task NEVER claims native Windows/Linux interactive authorization validation — no browser is exerciseable in the container; the manual end-to-end authorization run belongs to a later operator step and Task 5's evidence discipline.

## 10. Explicitly NOT done by Task 3 (boundaries)

- NO `run` changes: `modes::run` byte-identical; exit 3 retained for `run`.
- NO `demo` changes of any kind.
- NO refresh path, no `RefreshRequest` production caller, no refresh-token rotation.
- NO network-based authorization verification: `src/youtube/verify.rs` untouched (code-comment level only); granted-scope offline check only (D9).
- NO config schema/example/validation change; NO `.gitignore` change; NO `credentials/README`; NO README/docs/module-map/project-info edits (Task 5 + docs step 4.4 own them).
- NO new dependencies; no `Cargo.toml`/`Cargo.lock` edit; tempfile version frozen.
- NO `src/youtube/` logic changes beyond the mod.rs re-export block + narrow item-level `#[allow(dead_code)]` attributes (D13). No google_token_endpoint/transport/authorization/client_config/file_credential_store behavioral changes.
- NO logging of anything beyond the D15 event list; no logging of the URL/token/code/query.
- NO broadcast/RTMPS/FFmpeg/Live-Chat code; no interactive stdin; no browser spawn.
- NO version bump (step 3 already bumped to 0.7.0 at phase start).
- NO git push (restricted to workflow step 5), NO branch creation (already on the feature branch).

## 11. Self-check vs TODO Task 3 bullets

| TODO Task 3 bullet | Where satisfied |
| --- | --- |
| Replace the current `auth` placeholder with the installed-application authorization-code flow | section 1 + section 3; modes.rs auth REMOVED; dispatch rewired (D14) |
| Load and validate the configured OAuth client data before opening the flow | D5 (read+parse before bind) |
| Display the authorization URL in the console and guide the user | section 3.4 + 4.5 console spec; no auto browser launch, no stdin (D11) |
| Supported local callback mechanism; validate state against CSRF | D6/D7 + `parse_authorization_callback` reuse; D7 state handling |
| Bind only to a loopback interface | D6 (`127.0.0.1` only, S1) |
| Shut the callback listener down after completion or failure | D6 shutdown semantics + L8 test |
| Exchange the returned authorization code for tokens, handle provider errors with actionable but redacted diagnostics | D8 + D2 + D16 (Server error code + description are non-secret per Task 1 section 4.4) |
| Persist credentials securely | D8 step 3 via Task-2 `FileCredentialStore` (atomic + 0600) |
| Verify authorization with the minimum required YouTube API permission/scope; clear non-zero exit if missing/revoked/insufficient | D9 + D1/D3 (`InsufficientScope` = exit 1; "revoked"/missing credentials class is Task 4's load-side concern, auth's insufficient/declined classes are here) |
| Exit after successful verification; no background listeners or tasks | D6 (single-threaded, deterministic drop) + M-test/exit table |
| Preserve the existing non-interactive `run` placeholder and local-only `demo` behavior | section 10 first two bullets |
| Wire existing `auth` CLI mode preserving CLI syntax and exit-code contracts unless a justified, documented change | D3 (documented justification) + D14 (syntax preserved) |

Plan saved at `.kilo/plans/20261012-phase04-task3-auth-command.md` — the only file this sub-step created.
