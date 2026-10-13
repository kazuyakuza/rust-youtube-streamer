# Task 3 Adherence Report — Phase 04 `auth` Command (sub-step 4.5b, Overall Plan Adherence)

- Date: 2026-10-13
- Reviewer: Architector sub-agent (plan-adherence audit only; NO source files modified, NO commits made)
- Branch: `feat/phase04-oauth-credentials`, HEAD `f0b969f`, v0.7.0; tree clean except the 2 pre-existing untracked planner plans (`.kilo/plans/20261012-phase04-task3-review-fix.md`, `.kilo/plans/20261012-phase04-task3-simplify.md`) and this report
- Review range: `9ebf13f..f0b969f` (Task 3 implementation + review-fix + simplification + docs cycle)
- Authorities: plan `.kilo/plans/20261012-phase04-task3-auth-command.md` (D1–D16, §3–§11); review-fix plan `20261012-phase04-task3-review-fix.md`; simplify plan `20261012-phase04-task3-simplify.md`; TODO `.agent/todos/20261012/20261012-todo-1.md` Task 3 bullets

## VERDICT: ADHERENT

The implemented `auth` command satisfies every TODO Task 3 bullet and every locked plan decision D1–D16 with zero behavior deviations. All 15 observed deviations are planner-sanctioned adaptations or doc/test-shape equivalents; each is adjudicated **ACCEPTABLE** in §4 below. **Unacceptable findings: NONE.** No proposed changes.

---

## 1. Fresh container evidence (Alpine VM MCP, this session)

`alpine-vm_vm_status` → `running | sshReachable: true` checked BEFORE every command.

### 1.1 Dev-checks gate (V6)

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
- **Exit code: 0**
- Summary block (verbatim from log): `PASS fmt-check exit=0 (4 s)` / `PASS check exit=0 (21 s)` / `PASS test exit=0 (28 s)` / `PASS clippy exit=0 (1 s)` / `ALL CHECKS PASSED`
- **Test total: 218** — verbatim `test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s`
- Arithmetic: 195 (pre-Task-3) − 1 (deleted `auth_reports_not_implemented_exit_code`) + 24 (new auth tests) = 218 ✓ (plan §5.5 guidance met exactly; observed, not adjusted)
- Fresh log path: **`logs/checks/20261013T134943Z.log`** (UTC `Tue Oct 13 13:49:43 UTC 2026`, cargo 1.82.0)
- Note (pre-existing, sanctioned): one test-profile-only warning `associated function with_token_url is never used` (`src/youtube/google_token_endpoint.rs:50`, Task-1 `#[cfg(test)]` item) — does not affect any gate check (clippy `-D warnings` passes on its targets); see deviation 11.

### 1.2 The 24 new test names vs plan §5 (extracted from the fresh gate log; `grep '^test (app::auth|app::cli::cli_tests::usage_text_advertises_the_auth_command)'` → exactly 24 rows)

| Plan § | Test (observed full path) | Present |
| --- | --- | --- |
| M1 | `app::auth::mod_tests::run_auth_reports_missing_client_file_with_startup_failure_exit` | ✓ |
| M2 | `app::auth::mod_tests::every_auth_error_maps_to_the_startup_failure_exit` | ✓ |
| M3 | `app::auth::mod_tests::auth_error_display_lines_are_stable` | ✓ |
| M4 | `app::auth::mod_tests::auth_error_debug_and_display_reveal_no_secret_fixtures` | ✓ |
| C1 | `app::cli::cli_tests::usage_text_advertises_the_auth_command` | ✓ |
| F1 | `app::auth::flow::flow_tests::client_file_happy_parse_returns_client_config` | ✓ |
| F2 | `app::auth::flow::flow_tests::client_file_read_failure_is_the_client_config_class` | ✓ |
| F3 | `app::auth::flow::flow_tests::client_file_invalid_content_is_the_client_config_class` | ✓ |
| F4 | `app::auth::flow::flow_tests::stored_response_yields_storeable_credentials` | ✓ |
| F5 | `app::auth::flow::flow_tests::storeable_credentials_rejects_missing_required_scope` | ✓ |
| F6 | `app::auth::flow::flow_tests::storeable_credentials_rejects_missing_refresh_token` | ✓ |
| F7 | `app::auth::flow::flow_exchange_tests::code_exchange_request_assembles_the_documented_fields` | ✓ |
| F8 | `app::auth::flow::flow_exchange_tests::failing_exchange_maps_to_token_exchange_error` | ✓ |
| F9 | `app::auth::flow::flow_exchange_tests::saved_credentials_reach_the_store_seam` | ✓ |
| F10 | `app::auth::flow::flow_exchange_tests::store_errors_map_to_the_persistence_class` | ✓ |
| F11 | `app::auth::flow::flow_tests::declined_provider_flow_ends_without_persisting_end_to_end` | ✓ |
| L1 | `app::auth::listener::listener_tests::bind_uses_loopback_and_ephemeral_port` | ✓ |
| L2 | `app::auth::listener::listener_tests::authorized_callback_delivers_the_code` | ✓ |
| L3 | `app::auth::listener::listener_tests::declined_callback_fails_the_flow` | ✓ |
| L4 | `app::auth::listener::listener_tests::state_mismatch_callback_fails_the_flow` | ✓ |
| L5 | `app::auth::listener::listener_tests::noise_connections_are_ignored_until_a_valid_callback` | ✓ |
| L6 | `app::auth::listener::listener_tests::exhausted_noise_budget_abandons_the_flow` | ✓ |
| L7 | `app::auth::listener::listener_tests::invalid_utf8_request_is_treated_as_noise` | ✓ |
| L8 | `app::auth::listener::listener_tests::listener_socket_is_closed_when_dropped` | ✓ |

24/24 names byte-identical to plan §5.1–§5.4. No test added, removed, or renamed; the deleted placeholder test is gone from the suite.

### 1.3 Per-file `wc -l` (container run; cap ≤200)

| File | Lines | Note |
| --- | --- | --- |
| src/app/auth/mod.rs | 192 | |
| src/app/auth/flow.rs | 173 | |
| src/app/auth/listener.rs | **200** | exactly at cap — compliant (≤200); noted |
| src/app/auth/console.rs | 19 | |
| src/app/auth/mod_tests.rs | 146 | |
| src/app/auth/flow_tests.rs | 197 | |
| src/app/auth/flow_exchange_tests.rs | **200** | exactly at cap — compliant; noted |
| src/app/auth/listener_tests.rs | 192 | |
| src/youtube/mod.rs | 65 | |
| src/youtube/authorization.rs | 130 | |
| src/youtube/client_config.rs | 98 | |
| src/youtube/credentials.rs | 37 | |
| src/youtube/error.rs | **200** | exactly at cap (compression commit `8a50093`) |
| src/youtube/file_credential_store.rs | 199 | |
| src/youtube/google_token_endpoint.rs | 198 | |
| src/youtube/persisted_credentials.rs | 114 | |
| src/youtube/token.rs | **200** | exactly at cap (compression commit `8a50093`) |
| src/youtube/transport.rs | 52 | |
| src/youtube/verify.rs | 43 | |
| src/app/modes.rs | 83 | |
| src/app/cli.rs | 153 | |
| src/app/cli_tests.rs | 156 | |
| src/app/mod.rs | 177 | |
| src/main.rs | 16 | |

**All 24 files ≤200.** Four files sit exactly at 200 (listener.rs, flow_exchange_tests.rs, youtube/error.rs, youtube/token.rs) — within the rule, flagged for Task 4 awareness.

### 1.4 `#[allow(` allow-set equality — 24 hits (host grep over `src/`, matches `git grep "#\[allow(" src/`)

| # | Category | Items |
| --- | --- | --- |
| A | Phase-03 legacy (pre-existing, untouched by Task 3) — 3 | `chat/model.rs:44` (with_timestamp), `chat/store.rs:123` (is_empty), `streaming/video_output.rs:27` (OutputKind::Session) |
| B | Auth submodule — 1 | `app/auth/mod.rs:87` (CallbackAbandoned `attempts` field) |
| C | Task-3 youtube staging allows — 20 | `authorization.rs:23` (REVOCATION_ENDPOINT), `:81` (Authorized.state field); `credentials.rs:27` (CredentialStore::load); `error.rs:24` (AuthorizationError enum-level), `:47` (CredentialStoreError enum-level), `:58` (VerificationError); `file_credential_store.rs:124` (read_credential_text), `:146` (map_read_error); `token.rs:132` (TokenEndpointBody), `:185` (parse_token_response); `transport.rs:23` (RefreshRequest), `:46` (refresh_token_exchange); `verify.rs:12/20/28/36` (VerifiedAuthorization, new, granted_scopes, AuthorizationVerifier); `persisted_credentials.rs:64/79/90/100` (parse_credentials, stored_from_persisted, parse_failure_message, is_malformed_json) |

3 + 1 + 20 = **24** — equal to the on-record count in simplify plan §0 ("24 hits incl. the 21 Task-3/youtube allows"). No module-wide allow anywhere; every allow is item- or enum-level with a D13-referencing reason comment.

### 1.5 Boundary greps (diff range `9ebf13f..f0b969f`)

- Full `git diff --stat 9ebf13f..f0b969f` lists exactly 23 files: 8 × `src/app/auth/*` (new), `src/app/cli.rs`, `src/app/cli_tests.rs`, `src/app/mod.rs`, `src/app/modes.rs`, `src/main.rs`, 9 × `src/youtube/*`, plus the tracked plan file `.kilo/plans/20261012-phase04-task3-auth-command.md` (Step-0 commit `7d75989`).
- **Zero diffs** (verified by empty `git diff` pathspecs and absence from the stat): `src/config/**`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, `README.md`, `config/`, `docs/`, `Dockerfile`, `docker-compose.yml`, `scripts/`, `fonts/`, `.gitattributes`, `src/app/demo/**`, `src/app/pipeline/**`, `src/chat/**`, `src/renderer/**`, `src/streaming/**`.
- `Cargo.toml` byte-identical at both endpoints (v0.7.0, edition 2021, deps unchanged: oauth2 5.0/ureq, serde, serde_json, tracing, tracing-subscriber, ctrlc, ab_glyph; dev-dep tempfile 3).
- Gitignore compliance (host): `git check-ignore -v credentials/client_secret.json credentials/tokens.json config/config.json` → `.gitignore:52:credentials/*` ×2 and `.gitignore:48:/config/config.json`; `git ls-files credentials/` → `.gitkeep` only. No secret material anywhere in the range.

---

## 2. Evidence matrix — plan §-by-§

Legend: ✓ = verified against the HEAD `f0b969f` tree (file:line cites) and/or the fresh 218-test gate.

### §2 D1 — `AuthError` enum (`src/app/auth/mod.rs:68-104`)
✓ All ten variants present in plan order with exact payloads: ClientConfigFile{path,source}, ClientConfig(ClientConfigError), ListenerBind(io), CallbackAbandoned{attempts}, StateMismatch, ProviderDeclined{reason}, TokenExchange(TokenEndpointError), Persistence(CredentialStoreError), InsufficientScope, MissingRefreshToken. `Debug` derived; payloads non-secret.

### §2 D2 — Display contracts (`mod.rs:56-64, 114-149`)
✓ `STATE_MISMATCH_MESSAGE` and `ABANDONED_MESSAGE` constants verbatim (incl. the literal "8" in ABANDONED). All ten Display arms match the D2 table byte-for-byte — proven by the green M3 snapshot test (all ten lines asserted with `assert_eq!`). `attempts` NOT formatted (D2 ✓). `Error::source()` returns Some only for ClientConfigFile + ListenerBind (`mod.rs:151-159`).

### §2 D3 — Exit codes (all failures → 1; success → 0)
✓ `exit_code()` returns `STARTUP_FAILURE_EXIT` for every variant (`mod.rs:106-112`); `AUTH_SUCCESS_EXIT = 0`. M2 asserts all ten variants; M1 asserts the full `run_auth` entry returns 1 on a missing client file. `run` keeps 3 (`modes.rs:14,36-39` byte-identical, §2.3 below).

### §2 D4 — File decomposition (`src/app/auth/`)
✓ mod.rs 192 / flow.rs 173 / listener.rs 200 / console.rs 19 / mod_tests.rs 146 / flow_tests.rs 197 / flow_exchange_tests.rs 200 / listener_tests.rs 192. The D4 split hook was exercised exactly as specified (F7–F10 in `flow_exchange_tests.rs`, fakes shared with `flow_tests.rs` via `super::` imports) — see deviation 8.

### §2 D5 — Client-config failure classes; read+parse BEFORE bind
✓ `flow.rs:53-57` (`AuthFlow::run`): `read_client_config(...)?` executes BEFORE `listener::bind()?` — the TODO "validated before opening the flow" ordering holds. Read failure → `ClientConfigFile { path, source }` (`flow.rs:123-130`); parse failure → `ClientConfig(inner)` verbatim. F2/F3 cover both; M1 covers the wired entry.

### §2 D6 — Listener lifecycle (loopback-only, deterministic shutdown)
✓ Constants exact (`listener.rs:21-32`): CALLBACK_HOST "127.0.0.1", NOISE_CONNECTION_LIMIT 8, REQUEST_READ_LIMIT 8192, REQUEST_READ_CHUNK 1024, REQUEST_READ_TIMEOUT 5 s, both status lines, both fixed bodies, `HEADER_TERMINATOR_BYTES: &[u8] = b"\r\n\r\n"`. Bind `TcpListener::bind((CALLBACK_HOST, 0))` + `local_addr()` both mapped to ListenerBind (`listener.rs:40-48`); redirect_uri `http://127.0.0.1:<port>`. Accept loop runs on the MAIN thread; no threads/async/`Drop` impl in production code (grep-verified: threads exist only in `*_tests.rs`); the `BoundListener` is owned by `run`'s local scope and drops at return on success AND every failure. No Ctrl+C handler installed (D6 documented decision). Response format: status + Content-Type/Content-Length/Connection: close + body, written best-effort with `let _ =` (`listener.rs:189-196`). L1/L8 prove the loopback facts and the drop-closes-socket guarantee.

### §2 D7 — Callback outcome classification
✓ `CallbackOutcome { Authorized, Declined, StateMismatch, Noise }` (`listener.rs:99-104`). Authorized → 200 + accepted body; Declined → 400 + TERMINAL ProviderDeclined; StateMismatch → 400 + TERMINAL (CSRF); MissingParameter/unreadable/no-`?`/accept-error → Noise with budget decrement; exhaustion → `CallbackAbandoned { attempts: 8 }` (L6 asserts `attempts == NOISE_CONNECTION_LIMIT`). Methods not checked (documented in module docs). The extra `Err(AuthorizationError::ProviderDeclined)` arm is the sanctioned exhaustive-match adaptation (deviation 3). L2–L7 all green.

### §2 D8 — Exchange, validation, persistence ordering
✓ `code_exchange_request` assembles exactly the five documented fields (`flow.rs:157-165`). Validation BEFORE persisting, in order: `granted_scopes.grants(REQUIRED_SCOPE)` → `InsufficientScope` (`flow.rs:137-139`), then refresh-token presence → `MissingRefreshToken` (`flow.rs:140-142`), then `store.save` via the injected store (`flow.rs:117-119`), then the success print (`flow.rs:75`). F4–F10 cover every step; F11 proves nothing persists on the declined path (`store.recorded_count() == 0`, plus exchange never attempted).

### §2 D9 — Offline granted-scope verification only; verify.rs unwired
✓ Scope check lives in the app flow (`storeable_credentials`); `src/youtube/verify.rs` received ONLY narrow allows + reason comments (diff-verified, 8 added lines, zero logic changes). No network verifier this task.

### §2 D10 — Transport construction timing
✓ `auth_outcome` calls `create_transport()` FIRST (`mod.rs:170-171`), before the client-file read and before binding; failure maps to `AuthError::TokenExchange` with zero resources acquired.

### §2 D11 — No auto-launch, no stdin
✓ `console.rs` prints instructions + URL only; no process spawn, no stdin read anywhere in the submodule (grep-verified).

### §2 D12 — Test seams
✓ L1–L8 drive real `127.0.0.1:0` sockets with browser threads (`listener_tests.rs`); transport fakes (`ScriptedTransport` implementing `TokenEndpointTransport`) and store fakes (`RecordingStore`/`FailingStore`) are pure; F11 is the sanctioned declined-path e2e through `run_with_listener`; the success path is deliberately piecewise (documented). Layering note present in both module docs (`flow.rs:6-8`, `listener.rs:8`): oauth2 types are opaque plumbing only.

### §2 D13 — Re-exports + staging retirement
✓ `src/youtube/mod.rs:45-65`: the production re-export block contains EXACTLY the D13 item set (grouping of AuthorizationError under `error::*` differs from the sketch — same items; trivial). `#[cfg(test)]` re-exports added for the four test-only names (deviation 2). `src/main.rs`: `#[allow(dead_code)]` removed from `mod youtube;` (diff-verified, 1 line). The allow set grew to the sanctioned 24-hit envelope via D13's own escape hatch (deviation 6).

### §2 D14 — Modes/dispatch/CLI wiring
✓ `modes.rs`: `auth` handler + doc comment + `auth_reports_not_implemented_exit_code` DELETED; doc-header rewritten (D14) and later extended doc-only at `f0b969f` (deviation log, minor note 2). `app/mod.rs`: `mod auth;` inserted; dispatch arm exactly `cli::Mode::Auth => auth::run_auth(app_config),`; module-doc exit-code sentence updated to the D14 wording. `cli.rs`: ONLY the auth description line changed → `  auth                 authorize YouTube access and store OAuth credentials` (byte-verified diff); CLI syntax (commands, flags, `--config`/`--help` forms) unchanged. `cli_tests.rs`: C1 added asserting the exact new description.

### §2 D15 — Logging (six events, whitelisted fields)
✓ Grep-verified: the submodule contains exactly six `info!` sites — E1 `authorization flow started` (flow.rs:69), E2 `redirect_port = %address.port()` (listener.rs:43, the only fielded event), E3 `authorization callback accepted` (listener.rs:118), E4 `authorization code exchanged` (flow.rs:112), E5 `required YouTube scope granted` (flow.rs:143), E6 `credentials persisted` (flow.rs:74). No `warn!`/`error!`/`debug!`; no URL/token/code/query/credential-path ever logged. Failures are reported ONLY by the single stderr `error:` line (`report_auth_failure`, mod.rs:189-192).

### §2 D16 — Redaction guarantees
✓ No `.secret()` call in any production auth file (grep: secret() appears only in `*_tests.rs` and inside `src/youtube/`). M4 asserts Debug+Display of all ten variants contain none of the five fixture secrets and no `code=`/`state=` echo; F8 re-checks the exchange path; L4 re-checks the state-mismatch path; HTTP bodies are fixed constants.

### §3.1–§3.4 — Exact signatures
✓ `run_auth`/`auth_outcome`/`create_transport`/`create_store`/`report_auth_failure` match §3.1. `AuthFlow`/`OpenedFlow`/`run`/`run_with_listener`/`open_authorization`/`exchange_authorization_code`/`save_validated_credentials`/`read_client_config`/`storeable_credentials` match §3.2 modulo the sanctioned ownership adaptation (deviation 1: `opened` consumed by value; `code_exchange_request` private — deviation 4). `BoundListener`/`bind`/`wait_for_authorized_code`/helpers match §3.3 modulo the review-fix `CallbackResponse` restructure (deviation 7). `console.rs` matches §3.4.

### §4.5 — Console output spec
✓ `print_authorization_url` emits, in order: the instruction line, blank, the URL, blank, `The application is waiting for the browser callback at {redirect_uri} (loopback only).` — byte-equal to §4.5. `print_success` emits `authorization succeeded: YouTube credentials saved at {credentials_path}; the required YouTube scope was granted` — byte-equal; no secrets (path is the operator-configured value, sanctioned by D15's stdout-print note).

### §5 — Test plan
✓ 24/24 names exact (§1.2). Fixture-secret conventions honored; `assert!(a.secret() == b)` style preserved per simplify §0; M3 retains the per-variant snapshot blocks (S4 skipped); read-timeout path untested by documented design (listener_tests.rs module docs); L8 failure assertion tolerance-style (`is_err`).

### §6 — Implementation order / commits
✓ Observed commit chain in range: `7d75989` (Step 0 plan track) → `1dfb231` (Step 1 feat) → `1ae8ec5` (Step 2 staging retirement) → `8a50093` (cap compression) → `bd9bfc1` (review fix) → `be45d70` (S1) → `aeed51d` (S2) → `bd5cc4a` (S3) → `f0b969f` (docs alignment). Commit messages match the planned convention; one commit per step; no stray commits.

### §8 — Rule caps
✓ Files ≤200 (§1.3). Fn bodies ≤50 (longest production fn ≈ 17 lines). Depth ≤2. ≤2 non-receiver params everywhere (3-input contexts encapsulated in `AuthFlow`/`CodeExchangeRequest`/`CallbackResponse`; `write_callback_response` now exactly 2 params). Single-section booleans via `is_noise_budget_exhausted`/`head_is_complete`. Hand-written Display, no thiserror. Zero unwrap/expect/panic in production auth files (grep: all 53 hits in `*_tests.rs`). No commented-out code (all comments explanatory). No new dependencies (Cargo.toml byte-identical). Fresh `fmt --check` PASS.

### §9 — Registered risks
✓ L1–L9 all honored as documented (blocking accept without deadline; read-timeout path untested by design; L8 tolerance-style; F11 backlog reliance; oauth2 plumbing types documented; `with_token_url` untouched; 218 as guidance met; no native-interactive validation claimed anywhere).

### §10 — Explicitly NOT done
✓ Zero diffs in demo/pipeline/chat/renderer/streaming/config; no refresh path; no verify.rs logic; no config/schema/.gitignore/README/docs-of-record changes; no Cargo.toml/Cargo.lock edits; no version bump; no push; no branch creation.

---

## 3. TODO Task 3 bullet matrix

| TODO Task 3 bullet | Evidence |
| --- | --- |
| Replace the `auth` placeholder with the installed-app authorization-code flow | modes.rs handler deleted; dispatch rewired; full flow in `src/app/auth/`; exit 3 retired for auth (D3) — `run_auth` never returns 3 (M1/M2) |
| Load and validate OAuth client data before opening the flow | flow.rs:53-57 read+parse BEFORE bind; F1–F3, M1 |
| Display the authorization URL in the console; guide the user | console.rs §4.5-exact lines; no auto-launch (D11) |
| Supported local callback mechanism; validate state (CSRF) | loopback listener + `parse_authorization_callback` state equality; StateMismatch TERMINAL; L2/L4 |
| Bind only to a loopback interface | `127.0.0.1` only; L1 (never `localhost`, ephemeral port > 0) |
| Shut the listener down after completion or failure | `BoundListener` dropped at `run` return on both paths; no background tasks/threads in production; L8 |
| Exchange code for tokens; provider errors actionable but redacted | exchange via injected transport; `TokenExchange`/`ProviderDeclined` Displays redacted; F8, M4 |
| Persist credentials securely | Task-2 `FileCredentialStore` (atomic temp+rename, 0600 POSIX) only AFTER validation; F9 |
| Verify minimum required scope; non-zero exit if missing/revoked/insufficient | `granted_scopes.grants(REQUIRED_SCOPE)` + refresh-token presence before persist; exit 1 via InsufficientScope/MissingRefreshToken; F5/F6/M2; network verifier deferred to Task 4 (D9) |
| Exit after success; no background listeners or tasks | single-threaded mode returns `u8` immediately after the success print; L8 |
| Preserve non-interactive `run` placeholder and local-only `demo` | `run` handler byte-identical (§2 D14; `git diff` shows only doc-header/auth-deletion hunks); `src/app/demo/**` zero-diff; `run` keeps exit 3 |
| Wire `auth` CLI mode preserving syntax; documented exit-contract change | cli.rs description-line-only diff; D3 change documented in module docs + app/mod.rs doc sentence |

---

## 4. Sanctioned-deviation log — all 11 on-record adaptations + workflow rulings, each adjudicated

| # | Deviation | Ruling |
| --- | --- | --- |
| 1 | Non-Clone verifier consumption: `exchange_authorization_code`/`code_exchange_request` take `OpenedFlow` BY VALUE (plan §3.2 sketched `&OpenedFlow` + clones; S1 `be45d70` also moved code/redirect_uri) | **ACCEPTABLE** — `oauth2::PkceCodeVerifier` does not implement Clone, so the sketch was infeasible; by-value move enforces single-use PKCE per RFC 7636; values identical (F7 asserts captured client_id/redirect_uri/wrappers); doc comments record the rationale (flow.rs:39-41, 100-101) |
| 2 | `#[cfg(test)]` re-exports in `youtube/mod.rs` (`AccessToken`, `GrantedScopes`, `RefreshToken`, `RefreshRequest`) beyond the D13 block | **ACCEPTABLE** — youtube submodules are private; without these the app-layer test fakes cannot name the types; widening modules to `pub(crate) mod` would enlarge the real API surface (worse); test-only, simplify §0 explicitly preserved |
| 3 | `Err(AuthorizationError::ProviderDeclined)` arm in `classify_connection` (listener.rs:154-156) absent from the plan §3.3 sketch | **ACCEPTABLE** — the enum match must be exhaustive; the arm maps the variant identically to the Declined outcome; zero behavior change on the exercised paths (L3/F11 go through `Ok(Declined)`) |
| 4 | `code_exchange_request` made private (plan §3.2: `pub(super)`) | **ACCEPTABLE** — smaller API surface (prefer-private-members); plan F7's own text drives the capture through `exchange_authorization_code`, which the final test does |
| 5 | `#[allow(dead_code)]` on `CallbackAbandoned { attempts }` (mod.rs:87) | **ACCEPTABLE** — D2 mandates `attempts` NOT appear in the Display sentence (message hardcodes "8"), so the field is never read outside tests; narrowest possible allow with explanatory comment; simplify §0 preserved it |
| 6 | Enlarged allow set incl. enum-level allows (error.rs AuthorizationError + CredentialStoreError at enum level; plus credentials.rs `load`, file_credential_store.rs read-path ×2, persisted_credentials.rs ×4, transport.rs `refresh_token_exchange`) beyond the D13 six-file table | **ACCEPTABLE** — D13's own escape hatch ("if cargo check reports MORE dead items, add item-level allows in the same narrow pattern and REPORT the deviation"); all 24 hits enumerated in §1.4; equality with the simplify-plan on-record count; nothing module-wide |
| 7 | 3-param `write_callback_response` | **FIXED** as required by the review-fix plan — commit `bd9bfc1` introduced `struct CallbackResponse { status_line, body }`, `outcome_response_parts` returns it, writer is exactly `(stream, response)` (listener.rs:106-138, 189-196); bodies remain fixed constants; write failures still ignored |
| 8 | Test-layout merges: flow tests split into `flow_tests.rs` (F1–F6, F11 + shared fakes) + `flow_exchange_tests.rs` (F7–F10); fakes shared cross-module; single `ScriptedTransport` fake merges the plan's CapturingOk/CapturingFailing pair (Option-driven) | **ACCEPTABLE** — exactly the D4 split hook ("if a single test file would exceed 200 lines, split…"); test NAMES and assertions unchanged; the merged fake is behavior-identical and was required to hold the 200-line cap |
| 9 | Cap compression commit `8a50093` (flow_exchange_tests 232→200, flow_tests 225→197, listener.rs 222→200; error.rs/token.rs/file_credential_store.rs doc-comment compression) | **ACCEPTABLE** — mechanical rewrap/compression only, no behavior or assertion change; enforces plan §8's ≤200 cap; four files now sit exactly at 200 (flagged in §1.3) |
| 10 | Fix-up commit `bd9bfc1` landed separately ("fix: keep the callback writer within two parameters") | **ACCEPTABLE** — matches the review-fix plan's own acceptance/commit convention; one concern per commit |
| 11 | Pre-existing `with_token_url` note: Task-1 `#[cfg(test)]` constructor emits a test-profile-only dead-code warning (observed fresh in §1.1) | **ACCEPTABLE** — untouched by Task 3 (file not in the diff; plan risk L6 declares it out of scope); the warning does not fail any gate check (clippy `-D warnings` PASS, all four PASS lines green) |
| R10 | Same-commit doc compression in listener.rs (`8a50093`) | **ACCEPTABLE** — doc-comment rewrap that holds the file at 200/200; semantics preserved |
| R11 | Simplify step S4 (M3 table collapse) SKIPPED | **ACCEPTABLE** — S4 was caller-gated with "SKIP by default"; M3 retains the ten per-variant `assert_eq!` blocks (mod_tests.rs:77-130) exactly as plan §5.1 specifies |
| Step-A | fmt reflow during the review cycle (no commit; matched HEAD) | **ACCEPTABLE** — fresh `fmt --check` PASS at HEAD; no pending reflow |
| S1–S3 | Simplify steps landed: `be45d70` (moved exchange fields), `aeed51d` (`ok_or` for refresh token), `bd5cc4a` (shared `parsed_client_fixture` helper) | **ACCEPTABLE** — verified in final code (flow.rs:161-162; flow.rs:140-142; flow_tests.rs:92-96); behavior-equivalence argument in simplify §6 holds (F4/F6/F7/F8/F9/F10 + L2–L7 all green) |

Minor on-record notes (no ruling required, logged for completeness):

1. L6 sends exactly `NOISE_CONNECTION_LIMIT` noise connections (plan text said `+ 1`); the terminal assertion is byte-identical (`CallbackAbandoned { attempts: 8 }`) and a 9th connection could never be processed — the budget exhausts on the 8th. Test-internal shape, contract preserved.
2. `modes.rs` doc-header gained the auth-contract cross-reference at the docs step `f0b969f` (beyond D14's run-only rewrite) — doc-only, zero code impact, improves accuracy.
3. Re-export grouping places `AuthorizationError` under `error::*` instead of the sketch's `authorization::*` group — identical item set.
4. F11 adds the strengthening assertion `!transport.exchange_was_attempted()` (declined path never reaches the exchange) — an addition, never a weakening.
5. E3 (`authorization callback accepted`) fires inside `map_callback_outcome` after the response write rather than inside the plan's match arm — same event, same outcome, equivalent ordering.

## 5. Unacceptable findings

**NONE.** No behavior, contract, security, scope, or rule-cap violation found. No proposed changes.

## 6. Observed evidence index

| Evidence | Result |
| --- | --- |
| `git status` (pre/post) | clean; only the 2 pre-existing untracked planner plans (+ this report after writing) |
| `git log --oneline 9ebf13f..f0b969f` | 9 commits, mapped in §2 §6 row |
| `git diff --stat 9ebf13f..f0b969f` | 23 files; boundary files absent (§1.5) |
| `git diff` modes.rs / mod.rs / cli.rs / cli_tests.rs / main.rs / youtube/* | all hunks match D13/D14 exactly; `run` handler byte-identical |
| `git diff 9ebf13f..f0b969f -- src/app/demo/` (+ pipeline/chat/renderer/streaming/config) | empty — untouched |
| `Cargo.toml` at both endpoints | byte-identical, v0.7.0, deps unchanged |
| `git check-ignore -v` + `git ls-files credentials/` | `.gitignore:52` / `.gitignore:48`; `.gitkeep` only |
| `#\[allow(` grep over `src/` | 24 hits, categories in §1.4 |
| unwrap/expect/panic grep over `src/app/auth/` | all 53 hits inside `*_tests.rs`; zero in production files |
| tracing-event grep over `src/app/auth/` | exactly the six D15 events; no other levels |
| Alpine VM `vm_status` | running, SSH reachable (checked before every VM command) |
| Alpine VM dev-checks gate | exit 0; 4 × PASS; `ALL CHECKS PASSED`; **218 passed / 0 failed**; log `logs/checks/20261013T134943Z.log` |
| Alpine VM gate-log grep | 24 auth test names = plan §5 (§1.2) |
| Alpine VM `wc -l` (24 files) | all ≤200 (§1.3) |

Report saved at `.kilo/plans/20261012-phase04-task3-adherence.md` — the only file this sub-step created. No source files were modified; no commits were made.
