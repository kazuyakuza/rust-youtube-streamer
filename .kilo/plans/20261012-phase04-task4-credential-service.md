# Task 4 Plan — Phase 04: Credential Loading, Refresh, and Verification

- Date: 2026-10-12 (planned; execution date = implementer's step dates)
- Branch: `feat/phase04-oauth-credentials`, HEAD `2829840`, v0.7.0, tree clean, 218 tests green
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — **Task 4 "Credential Loading, Refresh, and Verification" ONLY**. Tasks 1–3 are DONE and provide the layer; Task 5 owns docs/tests-evidence/report. Other-task boundaries are restated in §12.
- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md` §Task 4 (pre-analysis).
- Sanctioned facts (reuse, NO re-research): `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1.1 (endpoints, refresh-response omission semantics, `invalid_grant` → re-run auth, revocation), §1.2 (scope S3 = `https://www.googleapis.com/auth/youtube.force-ssl`), §1.3–§1.5 (oauth2 5.0 / ureq 2.12 sync stack, S5 seams); `.kilo/plans/20261012-phase04-task3-auth-command.md` §2 D13 (staging allow inventory), D9 (auth flow verifies offline via granted scopes only).
- Implementer: JUNIOR developer, 50% restriction — no latitude beyond §13 "Explicitly NOT Done". Every structural/architectural decision below is locked; none is open.

---

## 0. Task Pre-Analysis (baseline facts, verified this session)

| # | Fact | Source (verified reads) |
| --- | --- | --- |
| F1 | `StoredCredentials { access_token, expires_in_seconds (TTL-at-save), refresh_token: Option<RefreshToken>, granted_scopes }`, all fields `pub(crate)`; wrappers are redacting; secrets only behind `.secret()` | `src/youtube/credentials.rs`, `token.rs` |
| F2 | `CredentialStore { load, save }`; concrete `FileCredentialStore` (atomic temp+rename, POSIX 0600, Windows remove+rename fallback documented-untested); on-disk JSON schema is the **FROZEN 4-key** document (`access_token`, `expires_in_seconds`, `refresh_token` nullable, `granted_scopes` space-delimited) | `file_credential_store.rs`, `persisted_credentials.rs` |
| F3 | `AuthorizationVerifier` trait: `verify(&AccessToken) -> Result<VerifiedAuthorization, VerificationError>`; `VerifiedAuthorization { granted_scopes }` with `new(...)` + `granted_scopes()` accessors; concrete verifier deliberately deferred to Task 4 (decision S9) | `verify.rs` |
| F4 | `TokenEndpointTransport::refresh_token_exchange(&RefreshRequest)` implemented by `GoogleTokenEndpoint` (ureq2 + oauth2 5.0, `redirects(0)`); `RefreshRequest { client_id, client_secret, refresh_token }`; `TokenExchangeResult = Result<TokenResponse, TokenEndpointError>`; refresh responses with a non-Bearer token type map to `ResponseParse` | `transport.rs`, `google_token_endpoint.rs` |
| F5 | `VerificationError { NotAuthorized { reason }, InsufficientScope { reason }, Transport { reason } }`; `CredentialStoreError { FileOpen, FileWrite, ReadParse, ContentNotAvailable }`; `TokenEndpointError { Transport, Server { code, description }, ResponseParse }`; `error.rs` is EXACTLY 200 lines (adding variants would breach the cap) | `error.rs` |
| F6 | Staging inventory: 20 item-level `#[allow(dead_code)]` across `src/youtube/**` (authorization 2, verify 4, transport 2, token 2, file_credential_store 2, persisted_credentials 4, credentials 1, error 3), all Task-4-reserved per Task-3 D13 | `git grep -n "#\[allow(" src/` |
| F7 | `auth` flow (Task 3) is the wired consumer: it saves credentials whose granted scope + refresh token were validated OFFLINE (D9); `run` is still the placeholder handler — `src/app/modes.rs::run`, its message constant and its 3 tests must stay BYTE-IDENTICAL | `src/app/auth/flow.rs`, `src/app/modes.rs` |
| F8 | The auth flow's `storeable_credentials` (flow.rs) guarantees a persisted `refresh_token: Some(..)` with `grants(REQUIRED_SCOPE)` at save-time | `flow.rs` lines 134–150 |
| F9 | Client configuration carries `client_id()` + `client_secret()` accessors (the refresh grant request needs these) | `client_config.rs` |
| F10 | Deps locked: `oauth2 5.0` (default-features=false, `ureq`), `ureq 2.12.x`, `serde/serde_json`, `tracing`; dev-dep `tempfile 3.14.0` frozen. NO new dependencies this task | `Cargo.toml`, context.md |
| F11 | Config surface: `config.oauth.client_config_path` + `config.oauth.credentials_path` exist (Task 2); untouched this task | `src/config/model.rs` |
| F12 | Rule caps: file ≤200 lines, fn body ≤50, depth ≤2, ≤2 non-receiver params, single-section booleans, hand-written `Display`, no `thiserror`, no panics/unwrap in non-test paths, no commented-out code | `.kilo/rules/*` |
| F13 | `parse_token_response` + `TokenEndpointBody` in `token.rs` are the serde parse seam used ONLY by `token_tests.rs`; production exchange/refresh parse via the oauth2 crate | `token.rs` 132–196 |

Task 4 is the **load-side service step**: one reusable credential service + one concrete verifier, fully offline-testable, NOT wired into any CLI mode.

## 1. Research Findings (current official Google/YouTube documentation, verified 2026-10-13 planning session)

### 1.1 The minimal verification request — AUTHORITATIVE EVIDENCE

1. **The exact request shape is Google's own documented example.** The official *OAuth 2.0 for Mobile & Desktop Apps* guide for the YouTube Data API (`developers.google.com/youtube/v3/guides/auth/installed-apps`), in its "Calling a Google API" section, shows verbatim:
   > *A call to the youtube.channels endpoint (the YouTube Data API) using the `Authorization: Bearer` HTTP header might look like the following. Note that you need to specify your own access token:*
   > `GET /youtube/v3/channels?part=snippet&mine=true HTTP/1.1` / `Host: www.googleapis.com` / `Authorization: Bearer <access_token>`

2. **`mine=true` is documented as the authorization probe.** The `channels.list` reference (`developers.google.com/youtube/v3/docs/channels/list`) states:
   > *mine | boolean | This parameter can only be used in a properly authorized request. Set this parameter's value to true to instruct the API to only return channels owned by the authenticated user.*

   The channels implementation guide (`.../guides/implementation/channels`) repeats: *"Your request must be authorized using OAuth 2.0"* when `mine=true` is set.
3. **`part=id` is the minimal valid part.** `part` is the required parameter listing channel-resource top-level properties; `id` is a documented channel resource part (channels reference). Decision: use `part=id` (smaller response than the guide's `snippet` example — payload is ignored anyway) — the request remains the documented `channels.list(mine=true)` call with the least payload.
4. **Scope pairing is valid.** The same installed-apps guide lists `https://www.googleapis.com/auth/youtube.force-ssl` among the scopes usable with the YouTube Data API ("See, edit, and permanently delete your YouTube videos, ratings, comments and captions."), and the Task-1 §1.2 evidence established force-ssl covers the later broadcast/stream/chat-write operations (read-only channels.list access is implied by force-ssl; the Task 1 spec-scope table shows `force-ssl` in the accepted scope sets of the write operations that also sit behind this read call).
5. **Quota cost is negligible:** `channels.list` is a read/list call (1 unit of the 10,000-unit default daily allocation; see Getting Started). One verification call per startup (at most two when refresh happens) fits any budget.

### 1.2 Response/error semantics — AUTHORITATIVE EVIDENCE

From `developers.google.com/youtube/v3/docs/errors` (Channels request-context errors + global errors) and `.../docs/core_errors` (global domain errors):

| HTTP status | Google error `reason` (from the JSON `error.errors[].reason` field) | Meaning (docs) |
| --- | --- | --- |
| 401 | `authorizationRequired` | *"The request uses the mine parameter but is not properly authorized."* |
| 401 | `unauthorized` | *"The user is not authorized to make the request."* |
| 401 | `authError` | *"The authorization credentials provided for the request are invalid."* |
| 401 | `expired` | *"Session Expired."* |
| 403 | `insufficientPermissions` | *"The OAuth 2.0 token provided for the request specifies scopes that are insufficient for accessing the requested data."* |
| 403 | `forbidden` | *"The request is not properly authorized."* |
| 403 | `accountSuspended` (and other account/limit reasons) | account-status / limit errors |
| other | — | RFC 7231 global domain errors; transport-level for our purposes |

A 401 on this request is therefore the **authoritative "this access token is not usable right now" signal** (expired OR individually revoked — Google states revoked tokens rejected with 401), and a 403 is the **"authenticated but denied" signal** (scope insufficiency docs mention by name). Tokens expired/invalidated such that refresh is impossible surface at the token endpoint as `invalid_grant` (Task-1 §1.1).

### 1.3 Refresh facts carried over (Task-1 §1.1 — no new research)

- Refresh grant (`grant_type=refresh_token`) needs `client_id`, `client_secret` (desktop clients carry one), `refresh_token`.
- **The refresh response MAY omit `refresh_token`** → callers must keep the previously stored one.
- `invalid_grant` on refresh means the refresh token expired/invalidated/**revoked** → re-run interactive authorization.
- Refresh tokens should be kept and reused long-term (issued-token limits); never re-authorize per run.

## 2. Locked Decisions (numbered; all made here, none open)

### T4-D1 — Refresh decision = VERIFY-FIRST (option b); NO schema change

The service determines whether refresh is required by **asking Google**, not by clock math:

```
load stored credentials
  → offline granted-scope gate on the stored evidence
  → verifier.verify(access_token)          [the effective-authorization probe]
      200 (authorized)                    → success, NO refresh, NO store writes
      401 (token rejected/expired/revoked)→ refresh path: refresh_token_exchange
                                            → merge response over stored credentials
                                            → store.save (atomic) → re-verify ONCE
      403 / transport-class               → typed failure, never refreshes
```

Justification (option (a) TTL/clock math REJECTED):

- (a) requires adding an absolute-issued-at timestamp to the **FROZEN Task-2 4-key JSON schema** — a D4-class schema amendment with migration semantics (existing credential files on disk) plus a clock-injection seam purely for tests. The persisted `expires_in_seconds` is TTL-AT-SAVE and even combined with a newly persisted wall-clock field breaks nothing less than a schema change would.
- Clock math cannot detect **revocation** (a revoked-but-young token passes TTL checks and fails later, possibly mid-stream); the verify-first probe detects both expired and revoked in one signal, matching TODO outcome wording ("determine whether access-token refresh is required") and brief §6.3 ("Refresh expired access tokens when possible; Fail clearly if authorization is missing, revoked, or insufficient").
- Verify-first is trivially offline-testable with fake verifier/transport — no clock seam anywhere; `expires_in_seconds` stays recorded metadata (its doc comments change wording, not semantics; see §9 Step 3).
- One-once cost: at most one extra GET (1 quota unit) when the token is still valid.

Cost accepted (documented limitation §9-L1): the stored TTL is only advisory metadata this task; a "token almost certainly expired" optimization belongs to the future run-phase wiring if wanted.

### T4-D2 — Concrete verifier: `GoogleAuthorizationVerifier`

- New file `src/youtube/google_authorization_verifier.rs`; implements the EXISTING `AuthorizationVerifier` trait unchanged (seam shape frozen by Task 1 §4.5).
- Request: `GET https://www.googleapis.com/youtube/v3/channels?part=id&mine=true` with `Authorization: Bearer <access_token>` (§1.1 evidence). The URL is ONE private constant `VERIFICATION_ENDPOINT` (query embedded — never reassembled from parts); a unit test asserts the exact constant string.
- HTTP client: the **`ureq 2.x` Agent directly, with `redirects(0)`** (SSRF-hardening precedent set by `GoogleTokenEndpoint`). It does NOT go through `TokenEndpointTransport` — that trait is the token-endpoint boundary (token exchange/refresh); the verification request is a Google Data API boundary with a different request/response shape. Two seams, two boundaries — decided.
- Mockable seams for the concrete type (D8 test strategy): (1) the existing `AuthorizationVerifier` trait fakes the verifier entirely for ALL service tests; (2) `#[cfg(test)] with_request_url(url: String)` constructor substitutes the full URL for the concrete-class tests, following the `GoogleTokenEndpoint::with_token_url` precedent (Task-1 S5).

### T4-D3 — Verification response → error mapping table (LOCKED)

The mapper consults the response ONLY through (a) the HTTP status and (b) Google's structured error-`reason` code string from the JSON body's `error.errors[].reason` field. Response bodies, messages, tokens or any other payload content are never embedded in error text (§5 redaction). Mapping (evidence §1.2):

| Condition | Mapping | Payload |
| --- | --- | --- |
| HTTP 200 | `Ok(VerifiedAuthorization::new(GrantedScopes::from_space_delimited(REQUIRED_SCOPE)))` | Payload IGNORED (body read toString then dropped). Documented semantics: the evidence scope set is what the server just confirmed effective — `mine=true` is only usable in a properly authorized request, so a 200 proves the required scope is granted for THIS token. Not a transcription of OAuth scope strings. |
| HTTP 401 | `Err(VerificationError::NotAuthorized { reason })` — the refresh-decision signal | `reason` = `"<fixed text> (server reason '<code>')"` — fixed text: `the Google server rejected the access token (HTTP 401)`; `<code>` = extracted reason string when present (`authorizationRequired`/`authError`/`expired`/`unauthorized`/other), omitted when absent/unparsable |
| HTTP 403 (ANY reason) | `Err(VerificationError::InsufficientScope { reason })` | fixed text: `the Google server denied access (HTTP 403)` + observed reason code when present (`insufficientPermissions` is the documented insufficient-scope code; other reasons such as `forbidden`/account-limit codes are reported verbatim in the reason for actionability) |
| any other HTTP status (400, 404, 5xx, 301…) | `Err(VerificationError::Transport { reason })` | `reason` = `the YouTube API server replied HTTP <status>` (number only) |
| ureq-level network failure (DNS/TLS/timeout/refused/reset) | `Err(VerificationError::Transport { reason })` | `reason` = the ureq error's own `Display` text (same bridge precedent as `TokenRequestFailure`; never the body) |
| response body unreadable (`into_string` failure) | `Err(VerificationError::Transport { reason })` | fixed text: `the verification response body could not be read` |

Rationale for "403 → InsufficientScope ALWAYS": refresh never helps any 403 (the access token itself was just accepted — a refreshed token changes nothing); `invalid_grant`-class revocation arrives at 401/refresh instead; a type-only branch keeps the mapping single-section and deterministic. Limitation recorded in §9.

Never trusted for authorization decisions beyond status + reason code strings: everything else in the body. Payment/quota-style 403s land in the scope class with the observed code named — documented, not silently mislabeled.

### T4-D4 — Refresh + persistence semantics (LOCKED)

1. Refresh triggers ONLY on `VerificationError::NotAuthorized` from the first verify.
2. No stored refresh token → distinct `CredentialServiceError::Unrefreshable` (exhaustive, §Project errors); the endpoint is never called.
3. `RefreshRequest { client_id: client_config.client_id(), client_secret, refresh_token }` — built from the INJECTED `ClientConfig` (F9) and the stored refresh token (moved; wrappers are `Clone`).
4. Endpoint failure mapping (distinct-class priority): `TokenEndpointError::Server { code == "invalid_grant" }` → `CredentialServiceError::Revoked { code }` (**exact match** on `invalid_grant` only; Google's documented fatal code for expired/invalidated/revoked refresh tokens); ALL other token endpoint errors (Transport, ResponseParse, Server with any other code) → `CredentialServiceError::Refresh(error)` — the transient/recoverable class per brief §12.1. Justification for the exact match: only `invalid_grant` is documented as "re-run authorization"; every other server code is either client/config trouble or transient, and no variant rewriting breathes beyond the rule-tank `Revoked { code }` payload carries the observed code verbatim.
5. Merge rule (response over stored): `access_token` + `expires_in_seconds` ALWAYS from the response; `refresh_token` = response's when present, ELSE the STORED one (§1.3 omission semantics; the merge never drops a stored refresh token); `granted_scopes` = the response's when its scope field is non-empty, ELSE the stored set (draft downgrades avoided); the merged offline scope gate is NOT re-checked before save (the second verification is the server-side gate).
6. Persist the merged credentials via `store.save(...)` BEFORE the second verification (atomic store; an interrupted save/re-verify leaves the freshest usable token persisted). A failing save short-circuits (no re-verify; §6 test 13).
7. Re-verify EXACTLY ONCE with the fresh access token; no loops/retries in the service. Second result: `Ok` → success; `Err(any)` → `CredentialServiceError::Verification(error)` (including a second `NotAuthorized` — operator text remains "run the authorization command again").
8. On success WITHOUT refresh, the store is never written (no save-call).

### T4-D5 — Service-level error enum (LOCKED; error.rs stays byte-stable at 200 lines)

`error.rs` cannot accept variants (F5). The service boundary therefore gets its OWN error enum in its OWN file — the enum-per-boundary convention preserved:

```rust
// src/youtube/credential_service_error.rs
#[derive(Debug)]
pub(crate) enum CredentialServiceError {
    /// No stored credentials exist; the operator must run the auth command first.
    Absent,
    /// Stored credentials exist but could not be read or parsed (FileOpen,
    /// ReadParse; `FileWrite` is impossible from `load` and folded into this
    /// same readable class when a future store change produces it).
    Unreadable(CredentialStoreError),
    /// The refresh succeeded but persisting the refreshed data failed.
    Persistence(CredentialStoreError),
    /// The refresh attempt failed at the token endpoint (transport failure,
    /// unparsable response, or a server code other than invalid_grant) —
    /// the transient/recoverable failure class (brief §12.1).
    Refresh(TokenEndpointError),
    /// Google answered `invalid_grant`: the stored refresh token is expired,
    /// invalidated or revoked. Only interactive re-authorization fixes this.
    Revoked { code: String },
    /// Stored credentials carry no refresh token, so a rejected access token
    /// cannot be refreshed; re-authorization is required.
    Unrefreshable,
    /// Verification of the effective authorization failed (insufficient
    /// scope, unauthorized-after-refresh, or transport); the inner error
    /// names the exact class.
    Verification(VerificationError),
}
```

- Display (hand-written, one end-user line each; inner enum Displays delegate verbatim — the AuthError pattern):
  - `Absent` → `no stored credentials are available; run the authorization command first`
  - `Unreadable(e)` / `Persistence(e)` → `{e}` verbatim (inner StoreError lines already name the file + cause)
  - `Refresh(e)` → `credential refresh failed: {e}`
  - `Revoked { code }` → `the stored refresh token is no longer valid (Google error code '{code}'); run the authorization command again to obtain new credentials`
  - `Unrefreshable` → `the stored credentials cannot be refreshed because no refresh token is stored; run the authorization command again`
  - `Verification(e)` → `{e}` verbatim
- `impl Error` with `source()` returning `Some` only for the io-carrying inner values via the inner `CredentialStoreError::source` chain: `Unreadable`/`Persistence` → `Some(inner.source())`; all others `None`.
- Exit-code/actionable-class ownership happens in the FUTURE run wiring; this task defines the classes only.

Mapping table (service level):

| TODO failure wording | Producing condition | Variant |
| --- | --- | --- |
| absent | `store.load()` → `ContentNotAvailable` (file never authorized) | `Absent` |
| malformed / unreadable / partial | `store.load()` → `ReadParse` / `FileOpen` | `Unreadable` |
| expired-but-refreshable | first verify 401 + refresh succeeds + second verify OK | SUCCESS |
| revoked | refresh exchange answers `Server { code: "invalid_grant" }` | `Revoked { code }` |
| expired/unrefreshable | first verify 401 + no stored refresh token | `Unrefreshable` |
| insufficient-scope | stored-scope gate fails, OR verify 403 | `Verification(InsufficientScope)` |
| transient network/endpoint trouble | verify Transport, or refresh Transport/ResponseParse/other Server code | `Verification(Transport)` / `Refresh` |

### T4-D6 — Service type design (parent struct = the parameter object; ≤2-arg rule honored by construction)

```rust
// src/youtube/credential_service.rs
pub(crate) struct VerifiedCredentials {
    /// The access token proven usable by the final verification; the future
    /// run wiring consumes it for the YouTube API calls.
    pub(crate) access_token: AccessToken,
    /// Seam-verifiable scope evidence of the verified effective authorization.
    pub(crate) authorization: VerifiedAuthorization,
}

pub(crate) struct CredentialService<'a> {
    pub(crate) store: &'a dyn CredentialStore,
    pub(crate) endpoint: &'a dyn TokenEndpointTransport,
    pub(crate) verifier: &'a dyn AuthorizationVerifier,
    pub(crate) client_config: &'a ClientConfig,
}
```

- NO `new()` method: constructed by struct literal only (the `AuthFlow` precedent, Task-3 §3.1) — four `pub(crate)` fields would break the ≤2-param constructor rule otherwise; the struct itself is the requested parameter object.
- Single entry operation `pub(crate) fn establish_authorization(&self) -> Result<VerifiedCredentials, CredentialServiceError>` (name locked — matches the caller's suggested `establish_authorization`).
- Orchestration contract (helper-first, each method ≤50 lines / ≤2 params / ≤2 depth):

```rust
impl CredentialService<'_> {
    /// The one entry operation (D1 diagram). Loads stored credentials,
    /// applies the offline scope gate, verifies the effective authorization,
    /// and refreshes exactly once on a rejected access token.
    pub(crate) fn establish_authorization(&self)
        -> Result<VerifiedCredentials, CredentialServiceError>;

    /// load + map load errors (D5 table) + offline gate: returns the stored
    /// credentials whose granted scopes include REQUIRED_SCOPE.
    fn load_verified_scope_evidence(&self)
        -> Result<StoredCredentials, CredentialServiceError>;

    /// First verification with the loaded access token; routes 401 into the
    /// refresh path and every other outcome straight through (D3/D4).
    fn verify_loaded(&self, stored: &StoredCredentials)
        -> Result<VerifiedCredentials, CredentialServiceError>;

    /// The refresh-only path: requires the stored refresh token (else
    /// Unrefreshable), exchanges, merges, persists, re-verifies once.
    fn refresh_and_reverify(&self, stored: &StoredCredentials)
        -> Result<VerifiedCredentials, CredentialServiceError>;
}
```

- Free helpers in the same file (2-param cap respected):
  - `fn grants_required_scope(stored: &StoredCredentials) -> bool` (single-section predicate; uses the existing `GrantedScopes::grants` — an exact-membership consumer that retires the last staging allow on that, §8).
  - `fn merge_refresh_response(stored: &StoredCredentials, response: TokenResponse) -> StoredCredentials` (2 params; moves response material per D4-step 5).
  - `fn refresh_request(client_config: &ClientConfig, refresh_token: &RefreshToken) -> RefreshRequest` (2 params).
- Gate failure message (offline scope gate): `VerificationError::InsufficientScope { reason: "the persisted granted scopes do not include it".to_string() }` (reason = short statement; the fixed hint belongs to the Display).
- Bearer build: `format!("Bearer {}", token.secret())` — the only secret() consumption of this task (token.rs doc rule honored).

### T4-D7 — Files, line projections and error.rs constraint

| File | Action | Projected lines | Notes |
| --- | --- | --- | --- |
| `src/youtube/google_authorization_verifier.rs` | NEW | ~150 | verifier + ureq execution + pure classifier + reason extraction (serde structs for the error document) |
| `src/youtube/credential_service.rs` | NEW | ~135 | service + `VerifiedCredentials` + orchestration fns |
| `src/youtube/credential_service_error.rs` | NEW | ~110 | the 7-variant enum + Display + Error sources |
| `src/youtube/google_authorization_verifier_tests.rs` | NEW | ≤200 (loopback server + classifier; split hook: `google_authorization_verifier_classify_tests.rs` if needed, mirroring the flow_tests split precedent) | |
| `src/youtube/credential_service_tests.rs` | NEW | ≤200 (split hook: `credential_service_refresh_tests.rs`) | |
| `src/youtube/mod.rs` | EDIT | +6 net (module docs paragraph refresh + 2 decls with the staging attribute/comment) | NO re-exports |
| `src/youtube/{credentials,transport,error,verify,persisted_credentials,file_credential_store}.rs` | EDIT | deletes only (`#[allow]` retirements ≤ comment rewording; NO additions) | see §8 |

- **error.rs ends EXACTLY at 200 lines: this plan adds ZERO lines to it (and to token.rs).** The only mutation is attribute/comment removal/rewording NET of at most −0 lines (attribute line REPLACED by nothing; comment swapped, not appended). `wc -l` proof per V-series.
- Explanatory comments are plain descriptive text only (no commented-out code).

### T4-D8 — Test-seam strategy

- **Pure offline fakes** (service tests): fake `CredentialStore` (fixed load result/error, records saves, fixed save result), fake `TokenEndpointTransport` (refresh result, call count, captured request facts), fake `AuthorizationVerifier` (queued verify outcomes + call count). No network, no clock, no serde in fakes (TokenResponse/is built by hand with its `pub(crate)` fields).
- **Exact classifier tests** (verifier tests): the mapping/`reason`-extraction functions are PURE and receive synthesized `(status, body)` pairs — the largest behavior surface is unit-tested without sockets.
- **Real loopback HTTP** (verifier tests ONLY, sanctioned by Task-3 D12): `TcpListener` fakes on `127.0.0.1:0` serving canned responses through the REAL ureq Agent with `with_request_url`. Prove the request assembly (Bearer header attached, method/URL) and live status→mapping through the production stack.
- The `with_token_url` / `with_request_url` cfg(test) constructors stay production-invisible.

### T4-D9 — Redaction guarantees (Task-1 §5 extended)

- Error payloads carry: statuses (numbers), FIXED reason texts, Google reason/error code strings (non-secret API codes), inner error Displays (already redacted). NO bodies, NO message fields, NO token values.
- `CredentialServiceError` derives `Debug` safely (inner enums' Debug already redact the secret material; `Revoked.code` and the `reason` strings are non-secret codes/texts).
- The fake test fixtures reuse the established names: `access_token_fixture`, `refresh_token_fixture`, `client_secret_fixture`; redaction tests assert the fixtures never appear in `format!("{e}")` or `format!("{e:?}")`.
- Refreshed-credential values never reach tracing: the only new strings logged (D10) carry no values.

### T4-D10 — Logging (bounded whitelist; new events ONLY here)

| # | Event (info!) | Site | Fields |
| --- | --- | --- | --- |
| E1 | `info!("stored credentials loaded")` | after a successful `store.load()` | none |
| E2 | `info!("access token refreshed and persisted")` | after successful save of the refreshed credentials | none |
| E3 | `info!("stored authorization verified")` | after the successful verification (both paths) | none |

NEVER logged: tokens, refresh tokens, scopes strings, statuses, reasons (the values all reach the operator through the returned typed error at the future wiring site), URLs. The verifier module emits NO tracing events. The 3 events are `info` (default filter compliant).

### T4-D11 — Staging contract for the NEW production-unconsumed surface

The service and the concrete verifier have NO production consumer this task (`run` stays a placeholder; `src/app/**` untouched). Staging follows the Phase-02 deliberate precedent (`#[allow(dead_code)] mod chat;` until Phase-03 wiring retired it):

```rust
// src/youtube/mod.rs — after the existing mod declarations:

// Staged until the future run-phase wiring constructs the load/refresh/
// verify service and its concrete verifier (deliberate staging, the
// Phase-02 `#[allow(dead_code)] mod chat;` precedent; retire there).
#[allow(dead_code)]
mod credential_service;
// Staged until the future run-phase wiring injects the concrete verifier
// into the credential service (same staging contract as above).
#[allow(dead_code)]
mod google_authorization_verifier;
```

- NO new `pub(crate) use` re-exports for the new items: the `youtube/mod.rs` policy is re-exports ONLY at the first external consumer — the future run-phase adds them when it consumes (`ClientConfig`, `StoredCredentials`, etc. are already re-exported and reach the service via `super::` internally, so nothing is needed now).
- Retirement contract written into the mod.rs staging comment; the future run-phase must remove BOTH module-level attributes when wiring.

### T4-D12 — Persistence-less behaviors that remain OUT (run/demo/CLI/config)

`src/app/modes.rs` (and every other `src/app/**` and `src/config/**` file) are UNTOUCHED. The service is reusable-but-not-yet-wired; the future run wiring (a later phase) will: re-export + construct `GoogleTokenEndpoint::new()`, `GoogleAuthorizationVerifier::new()`, `FileCredentialStore::new(credentials_path)`, load + parse the client file, build `CredentialService { .. }`, call `establish_authorization`, map its errors to the documented exit behavior. NONE of that happens here.

## 3. Exact Type/Signature Design (signatures ARE the design; the implementer fills bodies per §2)

### 3.1 `src/youtube/google_authorization_verifier.rs`

```rust
//! The concrete `AuthorizationVerifier`: the minimal YouTube API probe for
//! the force-ssl scope — `GET /youtube/v3/channels?part=id&mine=true`
//! (plan §1.1 evidence; no broadcast/chat operations). Responses are
//! mapped status-first; bodies are consulted only for Google's
//! non-secret error `reason` code strings and never embedded in errors.

use serde::Deserialize;

use super::error::VerificationError;
use super::token::{AccessToken, GrantedScopes};
use super::verify::{AuthorizationVerifier, VerifiedAuthorization};
use super::authorization::REQUIRED_SCOPE;

/// The exact verification request URL: the minimal documented
/// channels.list(mine=true) call for the required scope (plan §1.1).
const VERIFICATION_ENDPOINT: &str =
    "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true";

/// Fixed reason texts (D3)... (one `const &str` per per-mapping fixed text;
/// never appended with bodies)

pub(crate) struct GoogleAuthorizationVerifier {
    agent: ureq::Agent,
    request_url: String,
}

impl GoogleAuthorizationVerifier {
    /// Builds the verifier against the production endpoint.
    pub(crate) fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            request_url: VERIFICATION_ENDPOINT.to_string(),
        }
    }

    /// Test-only substitute URL (the `with_token_url` precedent); the
    /// production code never calls it.
    #[cfg(test)]
    pub(crate) fn with_request_url(request_url: String) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            request_url,
        }
    }

    /// Executes the Bearer request; maps transport-level failures (D3 row:
    /// ureq-level) and hands status+body to the classifier.
    fn executed_verification(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> { ... }

    /// Records the HTTP reply as (status, body-or-None) and delegates.
    fn reading_verification(...)  // internal texture: the implementer keeps
                                  // body reading inside the same 50-line body
}

impl AuthorizationVerifier for GoogleAuthorizationVerifier {
    fn verify(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerifiedAuthorization, VerificationError> {
        self.executed_verification(access_token)
    }
}

/// The mapping table (D3) as ONE pure function over status + body.
fn classify_verification_reply(
    status: u16,
    body: Option<&str>,
) -> Result<VerifiedAuthorization, VerificationError> { ... }

/// The non-secret error `reason` code string of a Google API error body.
fn extracted_reason(body: Option<&str>) -> Option<String> { ... }

/// The typed serde view of the documented Google error shape; serde
/// defaults everywhere (absent fields tolerated).
#[derive(Deserialize)]
struct ApiErrorDocument { error: Option<ApiErrorDetail> }

#[derive(Deserialize)]
struct ApiErrorDetail {
    #[serde(default)]
    errors: Vec<ApiErrorItem>,
}

#[derive(Deserialize)]
struct ApiErrorItem {
    #[serde(default)]
    reason: Option<String>,
}

/// Appends the extracted reason code to the fixed class text (D3 payload
/// column); the body itself is never included.
fn reason_with_observed_code(fixed_text: &str, body: Option<&str>) -> String { ... }

/// The verified evidence for an accepted request (D3 row 200).
fn verified_scope_evidence() -> VerifiedAuthorization {
    VerifiedAuthorization::new(GrantedScopes::from_space_delimited(REQUIRED_SCOPE))
}
```

Locked execution details:

- ureq 2.x call shape (mirror of `google_token_endpoint::execute_http`):
  `let response = self.agent.get(self.request_url.as_str()).set("Authorization", bearer_value.as_str()).call();`
  - `Ok(response)` (2xx) → read `response.into_string()` (failure → Transport fixed text) → `classify_verification_reply(response.status(), Some(&body))`.
  - `Err(ureq::Error::Status(status, response))` → read body (same read rule) → `classify_verification_reply(status, Some(&body))`. NOTE (locked fact): ureq 2.12 returns `Ok(response)` ONLY for 2xx; non-2xx arrives as `Error::Status`. The mapping flow handles both through ONE shared read-then-classify helper producing `(status, body)` — the implementer must NOT duplicate the classification logic across the two branches.
  - `Err(other)` → `Err(VerificationError::Transport { reason: other.to_string() })`.
  - Redirect responses can never be followed (`redirects(0)`): a 3xx arrives as `Error::Status` → classifier Transport branch.
- The body reader: `response.into_string()` may allocate arbitrarily large bodies — acceptable here (channels.list part=id responses are tiny; the Google TLS channel is the only source). No size guard this task (§9 limitation).
- The bearer header string `bearer_value` is consumed at the ureq call site and dropped with the local scope; never stored/returned.

### 3.2 `src/youtube/credential_service.rs`

```rust
//! The reusable credential service for the future production startup:
//! loads stored credentials, verifies the effective authorization with the
//! minimal YouTube API probe (the verify-first refresh decision, plan
//! D1/D3), refreshes exactly once when the access token is rejected
//! (keeping an omitted refresh token), and persists refreshed data through
//! the same atomic store. Typed failures distinguish absent/unreadable/
//! persistence/refresh/revoked/unrefreshable/verification classes per §2-D5.

use super::client_config::ClientConfig;
use super::credentials::{CredentialStore, StoredCredentials};
use super::credential_service_error::CredentialServiceError;
use super::error::VerificationError;
use super::token::{AccessToken, RefreshToken};
use super::transport::{RefreshRequest, TokenEndpointTransport};
use super::verify::{AuthorizationVerifier, VerifiedAuthorization};
use super::authorization::REQUIRED_SCOPE;

// The two public types (T4-D6) — constructed by struct literal only (no new()).
pub(crate) struct VerifiedCredentials {
    /// The access token proven usable by the final verification; the future
    /// run wiring consumes it for the YouTube API calls.
    pub(crate) access_token: AccessToken,
    /// Seam-verifiable scope evidence of the verified effective authorization.
    pub(crate) authorization: VerifiedAuthorization,
}

pub(crate) struct CredentialService<'a> {
    pub(crate) store: &'a dyn CredentialStore,
    pub(crate) endpoint: &'a dyn TokenEndpointTransport,
    pub(crate) verifier: &'a dyn AuthorizationVerifier,
    pub(crate) client_config: &'a ClientConfig,
}
```

Implementation-order bodies (locked):

```rust
impl CredentialService<'_> {
    pub(crate) fn establish_authorization(&self)
        -> Result<VerifiedCredentials, CredentialServiceError> {
        let stored = self.load_verified_scope_evidence()?;
        self.verify_loaded(&stored)
    }

    fn load_verified_scope_evidence(&self)
        -> Result<StoredCredentials, CredentialServiceError> {
        let stored = self.store.load().map_err(mapped_load_failure)?;
        if !grants_required_scope(&stored) {
            return Err(CredentialServiceError::Verification(
                VerificationError::InsufficientScope {
                    reason: "the persisted granted scopes do not include it".to_string(),
                },
            ));
        }
        info!("stored credentials loaded");
        Ok(stored)
    }

    fn verify_loaded(&self, stored: &StoredCredentials)
        -> Result<VerifiedCredentials, CredentialServiceError> {
        match self.verifier.verify(&stored.access_token) {
            Ok(authorization) => self.report_with_verification_evidence(stored, authorization),
            Err(VerificationError::NotAuthorized { .. }) => self.refresh_and_reverify(stored),
            Err(other) => Err(CredentialServiceError::Verification(other)),
        }
    }

    fn refresh_and_reverify(&self, stored: &StoredCredentials)
        -> Result<VerifiedCredentials, CredentialServiceError> {
        let refresh_token = stored.refresh_token
            .as_ref()
            .ok_or(CredentialServiceError::Unrefreshable)?;
        let request = refresh_request(self.client_config, refresh_token);
        let response = self.endpoint
            .refresh_token_exchange(&request)
            .map_err(|error| mapped_refresh_failure(error))?;
        let refreshed = merge_refresh_response(stored, response);
        self.store.save(&refreshed).map_err(CredentialServiceError::Persistence)?;
        info!("access token refreshed and persisted");
        self.verify_latest(&refreshed)
    }

    fn verify_latest(&self, refreshed: &StoredCredentials)
        -> Result<VerifiedCredentials, CredentialServiceError> {
        match self.verifier.verify(&refreshed.access_token) {
            Ok(authorization) => self.report_with_verification_evidence(refreshed, authorization),
            Err(error) => Err(CredentialServiceError::Verification(error)),
        }
    }

    /// The shared success construction for BOTH Ok arms (first verify and
    /// re-verify after refresh); emits the verification-verified event once.
    fn report_with_verification_evidence(
        &self,
        stored: &StoredCredentials,
        authorization: VerifiedAuthorization,
    ) -> Result<VerifiedCredentials, CredentialServiceError> {
        info!("stored authorization verified");
        Ok(VerifiedCredentials {
            access_token: stored.access_token.clone(),
            authorization,
        })
    }
}
```

```rust
fn mapped_load_failure(error: CredentialStoreError) -> CredentialServiceError {
    // `FileWrite` cannot be produced by `load`; folding it into the same
    // readable class keeps the match exhaustive (D5).
    match error {
        CredentialStoreError::ContentNotAvailable => CredentialServiceError::Absent,
        other => CredentialServiceError::Unreadable(other),
    }
}

fn mapped_refresh_failure(error: TokenEndpointError) -> CredentialServiceError {
    if is_revoked_refresh(&error) {
        return revoked_from(&error);   // extracts Server { code }
    }
    CredentialServiceError::Refresh(error)
}

/// Single-section predicate for the invalid_grant exact match (D4-4).
fn is_revoked_refresh(error: &TokenEndpointError) -> bool { ... }

/// Shape-matches Server { code } to `Revoked`; unreachable for
/// non-Server shapes (guard the false-match with a Transport fold to keep
/// the impl panic-free if Google's error shape shifts).
fn revoked_from(error: &TokenEndpointError) -> CredentialServiceError { ... }

fn grants_required_scope(stored: &StoredCredentials) -> bool {
    stored.granted_scopes.grants(REQUIRED_SCOPE)
}

fn merge_refresh_response(stored: &StoredCredentials, response: TokenResponse) -> StoredCredentials { ... }
fn refresh_request(client_config: &ClientConfig, refresh_token: &RefreshToken) -> RefreshRequest { ... }
```

Body-fill constraints for the implementer (LOCKED):

- `merge_refresh_response` moves response fields (D4-step 5); `granted_scopes` replacement uses a response scopes non-empty predicate (single-section boolean predicate extracted: `fn granted_evidence_replaces(stored_or_response...)`).
- The match-depth cap (≤2) governs: any deeper match (e.g. shape-matching `Server { code }` inside `mapped_refresh_failure`) goes into `is_revoked_refresh` + `revoked_from` helpers exactly as sketched.
- NO `unwrap`/`expect`/`panic` in non-test code; `ok_or`/`map_err` only.

### 3.3 `src/youtube/credential_service_error.rs`

Enum per D5 + Display per D5 + `impl Error` per D5 (AuthError shape precedent, `src/app/auth/mod.rs`). No message constants beyond the short variant texts inlined into the matching arms (they are one line each; keep them inline like existing error.rs arms).

### 3.4 Test fakes (locked shapes; the implementer writes bodies)

```rust
struct FakeStore {
    loaded: Option<StoredCredentials>,                     // moved out on first load
    load_error: Option<CredentialStoreError>,              // returned on EVERY load when set
    saved: std::cell::RefCell<Vec<StoredCredentials>>,     // one entry per save call
    save_error: Option<CredentialStoreError>,              // returned by every save when set
}

struct FakeEndpoint {
    refresh_result: Option<TokenExchangeResult>,           // moved out on first refresh call
    refresh_calls: std::cell::Cell<usize>,
    captured_refresh_token: std::cell::RefCell<Option<String>>, // refresh token fixture string (tests own the fixture)
    captured_client_id: std::cell::RefCell<Option<String>>,
}

struct QueueVerifier {
    outcomes: std::cell::RefCell<VecDeque<Result<VerifiedAuthorization, VerificationError>>>,
    calls: std::cell::Cell<usize>,
    last_token: std::cell::RefCell<Option<String>>,        // LAST access token verified (fixture string)
}
```

Hand-built values: a fixture `StoredCredentials` builder helper `fn stored_with(...)` (access/refresh fixture tokens, TTL 3592, REQUIRED_SCOPE granted scope set) and a fixture `TokenResponse` builder for refresh replies. `VerifiedAuthorization::new(GrantedScopes::from_space_delimited(REQUIRED_SCOPE))` is the fake-Ok value; fakes may also enqueue a NON-matching evidence set where a test demands it — NONE of the S-series depends on it (kept out; the evidence-membership re-check happens only through the verifier mapping itself and the stored-scope gate).

## 4. Status→Error Mapping Summary (authoritative, merged view)

| HTTP on verify | Verifier output | Service treatment | Final service error / outcome |
| --- | --- | --- | --- |
| 200 | `Ok(VerifiedAuthorization)` | success return, no store write | `Ok(VerifiedCredentials)` |
| 401 (+ stored refresh token) | `Err(NotAuthorized)` | refresh → merge → save → re-verify once | SUCCESS or §second-verify row |
| 401 (no stored refresh token) | `Err(NotAuthorized)` | endpoint NOT called | `Err(Unrefreshable)` |
| 403 (any reason) | `Err(InsufficientScope)` | no refresh | `Err(Verification(InsufficientScope))` |
| 400/404/5xx/3xx | `Err(Transport)` | no refresh | `Err(Verification(Transport))` |
| ureq network failure | `Err(Transport)` | no refresh | `Err(Verification(Transport))` |

| Refresh token exchange result | Service mapping | Variant |
| --- | --- | --- |
| `Ok(TokenResponse)` | merge + save + one re-verify | — |
| `Err(Server { code: "invalid_grant" })` | DIST | `Err(Revoked { code: "invalid_grant" })` |
| `Err(Server { code: <other> })` | transient/server class | `Err(Refresh(_))` |
| `Err(Transport)` | transient class | `Err(Refresh(_))` |
| `Err(ResponseParse)` | transient class | `Err(Refresh(_))` |
| any + `save` failure | persistence class, no re-verify | `Err(Persistence(_))` |

## 5. Redaction Rules (reviewer checklist)

1. New error payloads carry only: HTTP status numbers, fixed reason texts, extracted Google `reason`/`code` strings, inner error values (already redacted), io errors, paths (operator-facing credential-file path only, as existing).
2. Response bodies, JSON `message` fields, and any token/secret values never reach an error string, `Debug`, `Display`, or a tracing field.
3. `AccessToken`/`RefreshToken` wrappers: `.secret()` appears at exactly FOUR call sites in Task 4 files: (a) verifier bearer build, (b) refresh-request build, (c) merged-credentials field moves, (d) assertion sites in tests. Never formatted into anything else.
4. Redaction tests (D9 fixtures) assert absence of the fixture strings from Display and Debug of `CredentialServiceError`, and from the verifier classifier outputs, and from `StoredCredentials` (existing coverage stays).
5. The service RuntimeError-free guarantee: no panics/unwrap/expect in non-test paths.

## 6. Test Plan (deterministic, offline, NO Google, NO network beyond loopback)

All tests are `#[cfg(test)] #[path = "...—tests.rs"]` sibling modules (repo pattern). Tests may `unwrap`/`expect`. All existing 218 tests keep passing UNCHANGED.

### 6.1 `credential_service_tests.rs` (S-series)

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| S1 | `valid_stored_token_verifies_without_refresh` | store loaded with fixture credentials granted the required scope; fake verifier returns Ok → `Ok(VerifiedCredentials)`; access token secret == stored fixture; authorization grants REQUIRED_SCOPE; endpoint refresh calls == 0; store saves == 0 |
| S2 | `stored_scopes_missing_the_required_scope_fail_before_any_request` | stored granted scope WITHOUT force-ssl → `Err(Verification(InsufficientScope))`; verifier calls == 0 |
| S3 | `rejected_token_refreshes_saves_and_reverifies` | verifier queue `[Err(NotAuthorized), Ok]`; refresh response WITHOUT refresh_token, new access fixture, TTL 3592 → `Ok`; store saved EXACTLY once whose refresh token secret == ORIGINAL fixture (omission rule), access token secret == new fixture, granted scopes preserved; endpoint captured client_id == fixture; LAST verified token == new fixture |
| S4 | `refreshed_refresh_token_persists_when_google_respends_one` | same queue, response carries a NEW refresh token → saved refresh secret == NEW fixture (replacement rule, D4-5) |
| S5 | `refresh_response_scope_replaces_granted_evidence_when_present` | response carries a non-empty scope set incl. REQUIRED_SCOPE → saved granted set == the response's scope set (membership asserted per scope) |
| S6 | `refresh_response_without_scope_keeps_the_stored_granted_scopes` | response with EMPTY scope → saved granted set == the stored one |
| S7 | `revoked_refresh_token_maps_to_the_distinct_revoked_error` | endpoint `Err(Server { code: "invalid_grant", description: Some("Token has been expired or revoked.") })` → `Err(Revoked { code == "invalid_grant" })`; Display contains `invalid_grant` + "run the authorization command again"; store saves == 0 |
| S8 | `other_refresh_server_codes_stay_in_the_refresh_class` | `Err(Server { code: "unauthorized_client" })` → `Err(Refresh(_))`; inner Display text unchanged |
| S9 | `refresh_transport_failure_stays_in_the_refresh_class` | `Err(Transport { reason: "connection failed" })` → `Err(Refresh(_))` |
| S10 | `refresh_parse_failure_stays_in_the_refresh_class` | `Err(ResponseParse)` → `Err(Refresh(_))` |
| S11 | `absent_credentials_map_to_the_absent_class` | store load error = `ContentNotAvailable` → `Err(Absent)`; verifier calls == 0 |
| S12 | `unreadable_credentials_map_to_the_unreadable_class` | load errors `ReadParse { .. }` and `FileOpen { .. }` (two runs of one loop) → `Err(Unreadable(_))`; Display delegates the inner line verbatim |
| S13 | `persistence_failure_maps_to_the_persistence_class` | verifier queue `[Err(NotAuthorized)]` only; refresh Ok; store save error = `FileWrite` → `Err(Persistence(_))`; verifier calls == 1 (NO re-verify after a failed save) |
| S14 | `missing_stored_refresh_token_cannot_refresh` | verifier queue `[Err(NotAuthorized)]`; stored refresh_token None → `Err(Unrefreshable)`; endpoint calls == 0 |
| S15 | `insufficient_scope_from_the_verifier_skips_the_refresh_path` | verifier queue `[Err(InsufficientScope)]` → `Err(Verification(_))`; endpoint calls == 0 |
| S16 | `transport_from_the_verifier_skips_the_refresh_path` | verifier queue `[Err(Transport)]` → `Err(Verification(_))`; endpoint calls == 0 |
| S17 | `verification_failure_after_refresh_maps_to_the_verification_class` | verifier queue `[Err(NotAuthorized), Err(Transport)]` → `Err(Verification(Transport))`; endpoint calls == 1 (single retry rule) |
| S18 | `service_error_output_never_carries_secret_fixtures` | construct EVERY `CredentialServiceError` variant from fixture-bearing contexts (Refresh with Transport reason; Unreadable ReadParse path; Verification NotAuthorized; Revoked) → `format!("{e}")` and `format!("{e:?}")` contain NONE of `access_token_fixture` / `refresh_token_fixture` / `client_secret_fixture` |

### 6.2 `google_authorization_verifier_tests.rs` (R-series)

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| R1 | `verification_request_url_is_the_documented_channels_call` | `VERIFICATION_ENDPOINT` exactly `"https://www.googleapis.com/youtube/v3/channels?part=id&mine=true"` (accessible via the test module's same-file visibility) |
| R2 | `bearer_header_and_request_line_reach_the_fake_api` | loopback fake upstream server (TcpListener on `127.0.0.1:0`) records the first request line (`GET`) and path (`/youtube/v3/channels` + query or the given override URL path); through `with_request_url` REAL ureq Agent, result is `Ok(...)` and the captured header is exactly `Authorization: Bearer access_token_fixture` |
| R3 | `server_200_verifies_with_required_scope_evidence` | fake server replies 200 + any JSON body → `Ok` + `granted scopes grants(REQUIRED_SCOPE)` |
| R4 | `server_401_maps_to_not_authorized` | fake 401 + the documented `authorizationRequired` error body → `Err(NotAuthorized)`; Display contains `HTTP 401` and `authorizationRequired`; body's other content absent |
| R5 | `server_403_insufficient_permissions_maps_to_the_scope_class` | fake 403 + body reason `insufficientPermissions` → `Err(InsufficientScope)`; reason text contains `insufficientPermissions` |
| R6 | `server_403_other_reason_still_maps_to_the_scope_class` | fake 403 + body reason `accountSuspended` → `Err(InsufficientScope)`; reason text carries the observed code |
| R7 | `other_statuses_map_to_transport` | 400/404/500 synthesized statuses → `Err(Transport)`; reason contains the number |
| R8 | `error_bodies_are_never_embedded_into_reasons` | classifier with a body whose `message` embeds `access_token_fixture` (hostile-shape fixture) → the mapped error contains NO fixture string, only the reason code |
| R9 | `unparsable_error_bodies_use_the_fixed_reason_text` | 401 with non-JSON body (random sentence) → `Err(NotAuthorized)`; reason == exactly the fixed 401 text |
| R10 | `redirects_are_not_followed` | fake server replies `HTTP/1.1 301 Moved Permanently` → result `Err(Transport)` with reason containing `HTTP 301` |
| R11 | `connection_failures_map_to_transport` | override URL pointing at a port whose listener was dropped → `Err(Transport)` |

Loopback fake-server helper (locked shape): one thread, one accept, bounded `read_to_end` (or read-line loop up to the header terminator + connection close detection), responds with the canned status line + fixed headers + body of the test's specification, records the request line + the Authorization header into the caller-driven cell. Deterministic; the browser-style helper pattern of `listener_tests.rs` applies (bind first, dial after, small Connect-retry tolerance if flaky on the shared runner — the L4 note).

### 6.3 Suite arithmetic

218 (current) + 18 (S) + 11 (R) = **246 expected tests** — arithmetic guidance, NOT an acceptance criterion (Task-2/3 precedent: record the OBSERVED total; if a planned test is impossible as specified, STOP and ask the caller instead of inventing a substitute). No existing test may be modified or deleted.

## 7. Implementation Order for the Implementer (git actions + commit points)

All work happens on `feat/phase04-oauth-credentials`. Host is Windows: every cargo action runs through the Alpine VM MCP (`alpine-vm_vm_status` FIRST, then `alpine-vm_vm_run_command`, command prefix allowed set e.g. `docker compose ... ...`). Host git for status/add/commit/diff only. Fast loop per step: in-container `cargo fmt` (fold the diff into the step's commit), then `fmt --check`, `check --locked`, targeted `test --locked youtube`, `clippy --locked -- -D warnings`.

### Step 0 — plan-tracking commit

- `git status --porcelain` must show ONLY this plan file (untracked). Commit: `docs: track task 4 implementation plan`.

### Step 1 — concrete verifier

- NEW `src/youtube/google_authorization_verifier.rs` + `google_authorization_verifier_tests.rs` (§3.1, §6.2).
- EDIT `src/youtube/mod.rs`: add `#[allow(dead_code)] mod google_authorization_verifier;` (+ staging comment, D11) and update the module-docs "Landed vs deferred" sentence (the concrete verification request is now LANDED; the sentence points at the new module instead of "Task 4"). NO re-exports (D11).
- Verify: V1, V2, `cargo test --locked google_authorization_verifier`, V5 (in-container).
- Gitignore compliance pre-commit: `git status --porcelain` lists EXACTLY the intended files. Commit: `feat: add the channels-mine authorization verifier`.

### Step 2 — credential service

- NEW `src/youtube/credential_service.rs`, `credential_service_error.rs`, `credential_service_tests.rs` (§3.2/§3.3, §6.1; split hook per D7 if a test file would exceed 200 lines). The service module docs MUST state the interactive-vs-unattended contract: the initial authorization is interactive (the `auth` command), future unattended operation depends on the stored refresh credentials and Google's current OAuth policies (refresh tokens can be invalidated — `invalid_grant` requires the auth command again; the `auth` command remains the recovery path).
- EDIT `src/youtube/mod.rs`: add `#[allow(dead_code)] mod credential_service;` (+ staging comment, D11) and extend the module-docs paragraph with the service landing.
- Verify: V1, V2, `cargo test --locked credential_service`, V5.
- Commit: `feat: add the reusable credential service with refresh`.

### Step 3 — staging retirement + comment alignment (ONE commit)

Retire (EXACT list — delete the attribute lines; where a D13 comment line exists, delete it with the attribute; keep real code identical):

| File | Line (approx) | Item | Replacement note |
| --- | --- | --- | --- |
| `credentials.rs` | 26–27 | `CredentialStore::load` | delete both lines (the service through the trait now consumes load) |
| `transport.rs` | 22–23 | `RefreshRequest` struct | delete (the service builds it) |
| `transport.rs` | 45–46 | `refresh_token_exchange` trait method | delete (the service calls it) |
| `token.rs` | 132 | `TokenEndpointBody` | REPLACE `#[allow(dead_code)]` with `#[cfg(test)]` (line-count neutral; all fields already cfg(test)-reachable through the tests) |
| `token.rs` | 185 | `parse_token_response` | REPLACE `#[allow(dead_code)]` with `#[cfg(test)]` (line-count neutral) |
| `error.rs` | 46–47 | `CredentialStoreError` enum | delete both |
| `error.rs` | 57–58 | `VerificationError` enum | delete both |
| `verify.rs` | 11–12 / 19–20 / 27–28 / 35–36 | `VerifiedAuthorization`, `new`, `granted_scopes`, `AuthorizationVerifier` | delete all four (service + verifier consume them) |
| `persisted_credentials.rs` | 63–64 / 78–79 / 89–90 / 99–100 | `parse_credentials`, `stored_from_persisted`, `parse_failure_message`, `is_malformed_json` | delete all four attribute+comment pairs |
| `file_credential_store.rs` | 124 / 146 | `read_credential_text`, `map_read_error` | delete both |

STAY (untouched; still true after Task 4): `authorization.rs` line 23 (`REVOCATION_ENDPOINT`, no caller: revocation is a later-phase concern), `authorization.rs` line 81 (`Authorized.state` field), `error.rs` line 24 (`AuthorizationError` enum — `ProviderDeclined` still never constructed by the youtube layer).

Also in this commit (comment-only): `credentials.rs` `expires_in_seconds` doc + `persisted_credentials.rs` doc drop the "clock use belongs to the credential service (Task 4)" phrasing → "recorded TTL metadata at save time; the credential service verifies by asking the API (verify-first), never by clock math".

- Expected post-state: `git grep -n "#\[allow(" src/` shows EXACTLY: `authorization.rs` ×2, `app/auth/mod.rs` ×1, `chat/store.rs` ×1, `chat/model.rs` ×1, `streaming/video_output.rs` ×1, `youtube/mod.rs` ×2 (the staging decls). 
- Run V2 first; the compiler is the judge: any item reported dead that the table above expected live (a possible dyn-trait-call analysis gap: check the load/impl chain) gets a narrow item-level `#[allow(dead_code)]` with a `staged for the future run-phase wiring` adjacency comment, the deviation goes into the implementing summary. NEVER re-add module-wide allows; NEVER report a guess — the accumulated set is recorded.
- Commit: `chore: retire the task 4 reserved staging allowances`.

### Step 4 — full gate + hygiene evidence

- V6 dev-checks.sh gate (`ALL CHECKS PASSED`, observed total ~246 and the fresh log path recorded).
- V7–V11 host checks (gitignore-compliance rule: nothing ignored is staged; `credentials/` still only `.gitkeep`; no `target/`; no tmp leftovers).
- `wc -l` on every touched/new source file ≤200 (report per-file numbers in the implementing summary).

## 8. Staging Allow Inventory Before → After (authoritative summary of §7 Step 3)

| Before (commit `2829840`): 20 youtube allows | After Task 4 |
| --- | --- |
| `credentials.rs::CredentialStore::load` | RETIRED |
| `transport.rs::RefreshRequest` | RETIRED |
| `transport.rs::refresh_token_exchange` | RETIRED |
| `token.rs::TokenEndpointBody` | → `#[cfg(test)]` |
| `token.rs::parse_token_response` | → `#[cfg(test)]` |
| `error.rs::CredentialStoreError` | RETIRED |
| `error.rs::VerificationError` | RETIRED |
| `verify.rs` ×4 | RETIRED |
| `persisted_credentials.rs` ×4 | RETIRED |
| `file_credential_store.rs` ×2 | RETIRED |
| `authorization.rs` ×2 | KEEP |

Plus 2 NEW module-level staging allows (`youtube/mod.rs` decls, T4-D11). Net youtube count: 20 → 6.

## 9. Verification Command Series (V-series; Alpine VM MCP unless noted; exit status authoritative)

| # | Command | Expected |
| --- | --- | --- |
| V1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` | exit 0 |
| V2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` | exit 0 |
| V3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` | exit 0; observed total recorded (~246 expected) |
| V4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked youtube` | targeted development loop |
| V5 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` | exit 0 |
| V6 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | exit 0, 4 × PASS, `ALL CHECKS PASSED`, fresh log path recorded |
| V7 | (host) `git status --porcelain` | exactly the intended files per step |
| V8 | (host) `git grep -n "#\[allow(" src/` | exactly the §7 Step 3 expected set |
| V9 | (host) `git check-ignore -v credentials/client_secret.json credentials/tokens.json config/config.json` | unchanged rule hits |
| V10 | (host) `git ls-files credentials/` | `.gitkeep` only |
| V11 | (host) `wc -l` per touched/new src file | ≤200 each |

Build artifacts (build-linux/build-windows) are NOT rebuilt by Task 4 (Task 5's evidence step owns them).

## 10. Rule-Cap Checklist (every step + final)

| Rule | Compliance action |
| --- | --- |
| File ≤200 | new-file projections (D7); the two 200-cap files error.rs/token.rs get only deletions/attribute swaps; V11 proves |
| fn body ≤50 | orchestration helpers per §3.2; each method's body audited at review |
| depth ≤2 | match trees inside helpers (§3.2 sketch) |
| ≤2 non-receiver params | parameter-object structs (`CredentialService` literal, `RefreshRequest`, 2-param free helpers) |
| single-section booleans | `grants_required_scope`, `is_revoked_refresh`, and the response-scope replacement predicate |
| hand-written Display, no thiserror | D5 (AuthError pattern); no panics in library paths |
| no commented-out code | only descriptive comments; removals via git history |
| no new deps | nothing touches `Cargo.toml`/`Cargo.lock` (V12-implicit via `--locked` checks) |
| real newlines | standard file tooling; no literal `\n` escapes in written content |

## 11. Risks / Pre-Registered Limitations

- L1: The stored `expires_in_seconds` stays advisory metadata (TTL-at-save); refresh timing is decided by the live verifier probe (T4-D1). An "optimistically early refresh" policy would need a schema change — explicitly out.
- L2: dyn-trait dead-code analysis could flag the retired load chain (§7 step-3 stragglers protocol; narrow allows with `staged for the future run-phase wiring` comments + summary deviations; never module-wide).
- L3: The 200→`VerifiedAuthorization::new(REQUIRED_SCOPE)` evidence construction is by design (server-confirmed-effective); any FUTURE verifier implementation must keep the same semantic (documented in the verifier module docs).
- L4: ureq `into_string()` has no size cap (channels responses are small; documented).
- L5: R10/R11 (301/conn-refused-through-ureq) rely on ureq's error surface; if ureq's Display text changes between patch versions the fixed-prefix assertions could drift — tests assert only the mapped variant + status substring (tolerant of wording drift).
- L6: expected test total 246 is arithmetic guidance, not an acceptance criterion.
- L7: NO live-network or native-Windows validation occurs; all verification is the container gate with loopback locals only.
- L8: `GoogleTokenEndpoint::refresh_token_exchange` return contract is already fixed (Task 1): non-Bearer token-type refresh responses map to `ResponseParse` — service behavior tested through fakes only (S10), the real endpoint's Bearer guard is Task-1-owned (covered by its module tests), NOT re-tested here.

## 12. Explicitly NOT Done by Task 4 (boundaries)

- NO `src/app/**` changes of ANY kind: `run` stays the byte-identical placeholder (exit 3); its three tests untouchable; no `run` wiring of the service; NO `demo` changes; NO CLI/usage changes; NO dispatch changes.
- NO config schema/example/validation/v-example changes; NO `.gitignore` changes; NO `credentials/README` or docs/README/project-structure/project-info edits (Task 5 + docs step own them).
- NO broadcast/lifecycle/RTMPS/ingestion/chat-API code (verify request is channels-only read); no revocation-flow code (endpoint URL stays staged); NO interactive stdin; NO browser interaction.
- NO new dependencies; `Cargo.toml`/`Cargo.lock` untouched; tempfile frozen.
- NO persistence schema change: the frozen 4-key JSON shape stays EXACT (T4-D1 rejected option (a), so no TimeStamp field, no migration).
- NO existing-test modifications or deletions; no test requires live Google or non-loopback network.
- NO changes to the existing seams: the `AuthorizationVerifier` trait contract in `verify.rs` is FROZEN (T4-D2) — the concrete verifier implements it as-is; no existing type, trait or constant is resigned or re-shaped anywhere in `src/youtube/**` (the only touches are the §5/§8/§9 allow/comments retirements).
- NO logging beyond T4-D10's 3 events; no secret material anywhere in commits (gitignore compliance per commit).
- NO git push (workflow step 5); no branch changes.

## 13. Self-Check vs TODO Task 4 (mission record for the adherence reviewer)

| TODO Task 4 bullet | Where satisfied |
| --- | --- |
| "Implement a reusable credential service for later production startup: load stored credentials, determine whether access-token refresh is required, refresh using a stored refresh token when possible, and persist updated token data." | §3.2 service (`establish_authorization`), D1/D4; saved via the same atomic `CredentialStore::save`; NOT wired (D10/§12) |
| "Handle absent, malformed, unreadable, expired/unrefreshable, revoked, and insufficient-scope credentials with distinct actionable errors." | D5 mapping table — every class a separate variant/Display |
| "Verify the effective authorization through a minimal YouTube API request appropriate to the selected scope; avoid adding broadcast operations." | D2/D3: `channels?part=id&mine=true` (§1.1 evidence, non-secret status+reason mapping, §1.2) |
| "Ensure refresh and verification logic can be unit-tested without live Google credentials or network access." | §6 fakes + pure classifier + loopback-only concrete verifier tests (T4-D8) |
| "Document that initial authorization requires user interaction and that future unattended operation depends on the issued refresh credentials and Google's current OAuth policies." | They go in the SERVICE module docs (D6 docs contract) + this plan's D4 justification (Task-5 docs own the public-facing documentation; module-doc requirement listed in §7 step 2 bullet) |

Plan saved at `.kilo/plans/20261012-phase04-task4-credential-service.md` — the only file this sub-step has created or modified.
