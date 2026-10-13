# Task 5 Plan — Phase 04: Tests, Documentation, and Completion Evidence (sub-step 4.1b)

- Date: 2026-10-13
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — Task 5 "Tests, Documentation, and Completion Evidence" + ALL 13 acceptance criteria (the task closes the phase).
- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md` §Task 5.
- Branch: `feat/phase04-oauth-credentials`, v0.7.0; HEAD at plan time `ee15453` (clean); Tasks 1–4 `[DONE]`.
- Style source for the report: `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` and `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` (mirror the section structure).
- Deliverable evidence base already recorded (per-task adherence reports): Task 1 `…task1-adherence.md` (27 youtube tests, gate log `20261012T232738Z`), Task 2 `…task2-adherence.md` (195 total, log `20261013T011250Z`), Task 3 `…task3-adherence.md` (218 total, log `20261013T134943Z`), Task 4 `…task4-adherence.md` (247 total, log `20261013T163200Z`).
- Verification commands (Alpine VM MCP; call `vm_status` first):
  - Gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  - Linux build: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`
  - Windows build: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`

## §0 Current-state facts (verified this planning pass)

- Test totals: 147 (Phase 03) → 174 (T1, +27 youtube) → 195 (T2, +21 = 16 youtube store/persisted + 5 config) → 218 (T3, +24 new auth tests − 1 deleted placeholder test) → **247 (T4, +29 = 18 service-clone + 11 verifier)**. `247 passed; 0 failed` at the T4 gate.
- YouTube layer (all under `src/youtube/`, 25 files): `mod, error, client_config, token, transport, google_token_endpoint, authorization, verify, credentials, persisted_credentials, file_credential_store, credential_service, credential_service_error, google_authorization_verifier` + 11 test siblings. Suite per-file known counts: client_config 7, credentials 1, authorization 10, token 8, transport 2, google_token_endpoint 1, error 2, file_credential_store 11, persisted_credentials 5, credential_service_tests (fakes/fixtures only) 0, credential_service_refresh 9, credential_service_verification 9, verifier loopback 7, verifier classify 4 → 76? (implementer must re-derive exact per-file counts from the fresh gate log; totals must reconcile to 247 — arithmetic check below in §2).
- Non-youtube deltas this phase: `src/config/` +5 tests (3 new `validate_oauth` checks: 2 blanks aggregated + distinctness ⇒ 21 validation checks); `src/app/auth/` (8 files: mod/flow/listener/console + mod_tests 5/flow_tests 6/flow_exchange_tests 4/listener_tests 8; CLI description-line test `usage_text_advertises_the_auth_command`); `run` placeholder and `demo` byte-untouched (T3/T4 boundary evidence).
- Gate evidence is container-only; NO live Google authorization was ever performed; native Windows/Linux runtime execution NOT RUN; Windows artifact = cross-compilation only.
- The implemented phase count of `#[allow(dead_code)]` inventory = 14 items (T4 adherence §2.3) — this number goes into `project-structure.md`'s allow note.

## §1 Coverage audit — AC ↔ evidence matrix (do FIRST, record verdicts verbatim in the report)

Map each acceptance criterion (TODO §Acceptance Criteria, rows 1–13) to concrete evidence. The audit verdict: **all 13 criteria are covered** by implementation + tests + recorded adherence evidence, with the standing honest-limitation framing. Per AC:

| # | Criterion (short) | Evidence (cite in report) | Verdict |
| --- | --- | --- | --- |
| 1 | `auth` is a real Google OAuth flow; no not-implemented exit code | Task 3: `modes.rs` placeholder deleted, dispatch rewired (`1dfb231`, D14); tests `app::auth::mod_tests::every_auth_error_maps_to_the_startup_failure_exit` (all 10 variants exit 1), `run_auth_reports_missing_client_file_with_startup_failure_exit`; `cli_tests::usage_text_advertises_the_auth_command`. NO more exit-3 for auth — covered by test + the deleted test in gate history | covered |
| 2 | Browser authorization + local callback, no manual token paste | `app::auth::listener::listener_tests::*` (L1 `bind_uses_loopback_and_ephemeral_port`, L2 `authorized_callback_delivers_the_code`), flow F1–F11 incl. `code_exchange_request_assembles_the_documented_fields` (PKCE S256 redirected-grant, `authorization_url_encodes_all_parameters`); console URL brochure (see §3 NEW test) | covered (+ §3 new test) |
| 3 | State validated; loopback-only binding; listener closed on success AND failure | `state_mismatch_callback_fails_the_flow` (L4), `listener_socket_is_closed_when_dropped` (L8), L1 (127.0.0.1 only), L3/L5/L6/L7 (decline/noise/exhaustion/invalid-utf8); code evidence: bind host constant `127.0.0.1`, `BoundListener` dropped at `run` return | covered |
| 4 | Separate, configurable client-config/credentials paths | `src/config` tests `oauth_paths_require_non_blank_values`, `oauth_paths_must_be_distinct`, `example_mirrors_oauth_schema`, `unknown_oauth_field_rejected`, `oauth_section_missing_rejected`; defaults `credentials/client_secret.json` + `credentials/tokens.json` in `config/config.example.json:9-12`; docs step §4 | covered |
| 5 | Atomic persistence; restrictive permissions where supported; limitations documented | `file_credential_store_tests`: `save_is_atomic_replacing_in_place`, `save_removes_a_stale_temp_file_and_survives_an_interrupted_save_shape`, `unix_mode_is_owner_only_after_save`, `save_creates_a_missing_parent_directory`, `target_directory_collision_maps_to_file_open`… (11 tests) + persisted_credentials 5 (fixed 4-key schema, load round-trips); limitations = task2 plan L1 (Windows rename arm untestable in Linux container) + L4 (no Windows ACL hardening; no encryption claims) — recorded in module docs + §4 docs + report | covered |
| 6 | Refresh from stored refresh token; refreshed data persisted | S-series: `rejected_token_refreshes_saves_and_reverifies`, `refreshed_refresh_token_persists_when_google_respends_one`, `refresh_response_scope_replaces_granted_evidence_when_present`, `refresh_response_without_scope_keeps_the_stored_granted_scopes`, `persistence_failure_maps_to_the_persistence_class`; transport seam tests | covered |
| 7 | Credentials verified against the minimum documented YouTube scope | `REQUIRED_SCOPE = https://www.googleapis.com/auth/youtube.force-ssl` (`authorization.rs:15`; `authorization_url_uses_the_force_ssl_scope`); verifier R-series: `verification_request_url_is_the_documented_channels_call` (`GET /youtube/v3/channels?part=id&mine=true`), `bearer_header_and_request_line_reach_the_fake_api`, `server_200_verifies_with_required_scope_evidence`; scope gate `stored_scopes_missing_the_required_scope_fail_before_any_request` | covered |
| 8 | Missing/malformed/revoked/expired-unrefreshable/insufficient → clear non-zero failures without leaking secrets | `absent_credentials_map_to_the_absent_class`, `unreadable_credentials_map_to_the_unreadable_class`, `load_reports_least_details_for_malformed_json`, `load_reports_fixed_schema_mismatch_message`, `revoked_refresh_token_maps_to_the_distinct_revoked_error`, `missing_stored_refresh_token_cannot_refresh`, `insufficient_scope_from_the_verifier_skips_the_refresh_path`, `verification_failure_after_refresh_maps_to_the_verification_class`; redaction: `auth_error_debug_and_display_reveal_no_secret_fixtures`, `service_error_output_never_carries_secret_fixtures`, `error_bodies_are_never_embedded_into_reasons`, `error_tests::each_variant_has_a_stable_display_line`, `source_chain_exposes_io_errors_only` | covered |
| 9 | Automated tests cover all paths, no real Google account/network | Whole-suite statement: every test offline (`TokenEndpointTransport` fakes, `CredentialStore` fakes, verifier fakes, ureq loopback `127.0.0.1` fake API servers only); mod.rs module-doc seam list; the report states zero non-loopback sockets and zero live-Google calls | covered |
| 10 | `demo` unchanged; `run` placeholder; no broadcast/RTMPS/FFmpeg-production/Live-Chat code | T3/T4 adherence: `git diff` zero over `src/app/demo/**, src/app/pipeline/**, src/chat/**, src/renderer/**, src/streaming/**`; `modes.rs` `run` handler byte-identical (exit 3, four-prerequisites message with 4 tests intact); coupling grep `ffmpeg|broadcast|liveChat|rtmps` over `src/youtube` → doc comment only | covered (evidence re-collected fresh in this task — §6 Step F audit) |
| 11 | Docker dev checks pass + both artifacts rebuilt (or exact blockers) | THIS task produces the fresh evidence: §6 Step A.3 (fresh gate), Step B steps 1–3 (both builds + `dist/` cross-check); cross-compile ≠ native runtime stated | produced by this task |
| 12 | Docs/structure/context/report accurate | THIS task: §4–§8 + Documentation Accuracy Audit table in the report | produced by this task |
| 13 | No credentials/tokens/codes/secrets/videos committed | §6 secret-audit protocol step (fresh greps + gitignore checks + `git ls-files credentials/` = `.gitkeep` only) | produced by this task |

**Gap decision (exactly ONE new test)** — the only uncovered surface is `src/app/auth/console.rs` (19 lines, zero tests as of HEAD); AC #2's "guides the user through browser authorization" rests solely on the §4.5 byte-equal contract without an automated assertion. Everything else is already covered (the youtube suite + config/auth additions per §2's authoritative inventory; no token-exchange, refresh, persistence, permission, or redaction gap exists — see §1 matrix rows). Do NOT add any other test; do NOT touch production code to enable a test (`console.rs` functions are already public-within-module enough to test — see spec).

## §2 Test-Coverage inventory rule for the report (the "cite all 247" requirement)

The report's §Test-Coverage Audit lists the per-file/suite test-count inventory derived from the FRESH gate log (`grep "running N tests"` + `test result:` line), not from stale sources. The authoritative total is the observed gate total — **247 before the §3 test, 248 after it**; the inventory table must sum to that total exactly. Required inventory structure:

- `src/youtube/**` — 72 tests: enumerate each test sibling with its count (re-derive from the gate log; planning-pass spot counts: client_config 7, credentials 1, authorization 10, token 8, transport 2, google_token_endpoint 1, error 2, file_credential_store 11, persisted_credentials 5, credential_service_refresh 9, credential_service_verification 9, verifier_r 7, verifier_classify 4 — sum 76 vs expected 72 ⇒ the implementer MUST take the authoritative per-file numbers from the gate log and correct this plan's spot list where it disagrees; the total must reconcile exactly to 247).
- `src/app/auth/**` — 24 (M×4 incl. per-variant assertions, C1 in `cli_tests` 1, F×11, L×8).
- `src/config/**` — 21 (16 pre-phase + 5 oauth).
- Prior-phase suites — the remainder of the total (baseline 147 from Phase 03, unchanged except the config delta); reconcile on paper as: 147 baseline + 5 config-oauth + (24 auth-and-cli − 1 deleted placeholder) + 29 T4 + 1 (§3 test) = 248; present ALL suite rows as OBSERVED counts with the gate log as the single source.
- Presentation: a two-column table (suite → count) + a paragraph noting `247/248 passed; 0 failed; 0 ignored` verbatim per run, with the arithmetic chain 195→218→247→248 and the 174 waypoint cited from the adherence logs.

## §3 The ONE new test (exact spec)

- File: `src/app/auth/console_tests.rs` (new; `#[cfg(test)]`-mounted per the sibling convention; expected ≤ 80 lines).
- Mounting: in `src/app/auth/console.rs`, append the standard test-foot block `#[cfg(test)] #[path = "console_tests.rs"] mod console_tests;` matching the sibling style used in `flow.rs`/`listener.rs` (check the exact 2-line form in one sibling and copy it).
- Test 1 — `print_authorization_url_uses_the_documented_brochure`: capture stdout (reuse the sibling tests' capture helper method — follow how `flow_tests.rs` captures the success print; if none exists, use the smallest local stdout-capture in the test file via the same mechanism the suites already use; do NOT add a dev-dependency), call `console`'s print function with a fixture authorization URL + redirect URI, assert exact brochure: instruction line, blank line, URL verbatim, blank line, `The application is waiting for the browser callback at {redirect_uri} (loopback only).` line.
- Test 2 — `print_success_names_the_credentials_path_and_scope`: assert `authorization succeeded: YouTube credentials saved at {credentials_path}; the required YouTube scope was granted` with fixture path; fixture path uses a temp-relative fake path like `credentials/tokens.json` (never a real one).
- Redaction: the printed URL contains only the fixture client id — use the same fixture id the flow tests already use (`super::` import; keep it identical to avoid a second secret-shaped constant). No secrets, no tokens in fixtures.
- Expected total after this step: **248** (guidance, not an acceptance criterion).
- Commit: `test: cover the auth console output contract`.

## §4 Documentation deliverables (docs-specialist; outlines locked)

### 4.1 README.md edits (in place)

1. **Current Status** → rewrite to Phase 04 state: OAuth layer shipped under `src/youtube/` + `src/app/auth/`; `auth` performs the real installed-application authorization flow (browser + loopback callback + PKCE + token exchange + scope verification + atomic credential persistence); `run` remains a non-interactive placeholder (exit 3) storing nothing; `demo` unchanged; suite count **248** (fresh), container-only; NO live Google authorization performed in this phase — the flow is validated offline (fakes/loopback), native runtimes NOT RUN.
2. **Key Technologies** → add bullet: **OAuth 2.0 (`oauth2` 5.0 with the `ureq` feature + `ureq` 2.x)** — synchronous, no Tokio; `google-youtube3` remains rejected per brief §3.3.
3. **Commands and Configuration** → replace the `auth` NOT-implemented bullet: `auth` — performs the OAuth authorization flow using the Google OAuth client file and saves tokens to the configured credentials path; exits 0 on verified success; exits 1 on any failure (configuration errors included; never leaks secrets). Exit-code summary line: `3 = mode not implemented yet (run)`. Usage header untouched.
4. NEW section **Authentication (OAuth 2.0 — Key Features)** between Logging and Build Checks + TOC entry:
   - Procedure paragraphs: prereq = a Google OAuth 2.0 "Desktop app" client downloaded as `credentials/client_secret.json` (or the configured path); run `auth`; the console prints the authorization URL — open it in a browser, sign in, authorize; the local callback is bound to `127.0.0.1` on an ephemeral port only; after approval the app validates state (CSRF), exchanges the code, verifies the scope `https://www.googleapis.com/auth/youtube.force-ssl` via the minimal `channels.list(mine=true)` probe, and saves tokens atomically to `credentials/tokens.json`; success = one stdout line + exit 0.
   - Unverified-app screen: first authorization may show the `Google hasn't verified this app` dialog; the docs will note to expect and accept it (Advanced → continue) for a development client.
   - Persistence + permissions: 0600 POSIX created-temp-file mode; honest limitation sentence — Windows uses inherited ACLs (std has no ACL management), there is NO encryption, and the Linux container suite cannot exercise the Windows rename/permission arms (record the limitation verbatim here).
   - Unattended refresh: future sessions load stored tokens and refresh when verification demands it; refreshed tokens are persisted; long-unattended operation depends on the issued refresh credentials and Google's current OAuth policies.
   - Failure/recovery: missing/revoked/expired/unrefreshable/insufficient-scope states each give a distinct redacted error; recovery for revoked or unrefreshable refresh credentials = re-run `auth`.
   - Security: neither file is committed (`credentials/*` gitignored); nothing secret is logged; `client_secret.json` and `tokens.json` must never be copied into the repo tree.
   - Cross-link: full guide → `docs/authentication.md`; note the standing limitation block (no live Google end-to-end run this phase; offline/loopback tests only).
5. **Logging (RUST_LOG)** section: no change (verify the 6 auth info! events are not claimed anywhere).
6. **Build Checks / Release Build sections**: verify accuracy; expected no-op.

### 4.2 NEW `docs/authentication.md` (content outline; ~120–160 lines, TOC + back-links)

1. Overview — what `auth` does in one paragraph; scope authority; the minimal scope justification sentence (`youtube.force-ssl`; later phases reuse the same grant for broadcast/chat — with a pointer to `.kilo/plans/20261012-phase04-task1-oauth-design.md` §1 for the research ledger).
2. Prerequisites — Google Cloud console, Desktop-app client, downloaded JSON at the configured path; distinctness rule (never point both paths at one file; validation rejects it).
3. First authorization (interactive, once) — the 6-step procedure (see §4.1 item 4, more detailed): browser steps, unverified-app screen expectation, callback mechanics (loopback only, ephemeral port, listener closed on every path), what the console prints at each stage, exit codes (0 success; 1 = configuration/persistence/exchange/scope/listener errors — never 3).
4. Credential files — persistence location (`credentials/tokens.json` default), the frozen 4-key JSON schema (`access_token`, `expires_in_seconds`, `refresh_token` (null when absent), `granted_scopes` space-delimited), atomic save (temp + rename in the same directory; stale-tmp cleanup), POSIX 0600, Windows ACL + rename-arm limitations (honest, no encryption claims).
5. Unattended operation — verify-first refresh design (no clock-based refresh decision; the `expires_in_seconds` value is advisory TTL metadata only); refreshed tokens are persisted atomically; the refresh behavior and unattended longevity depend on Google's current OAuth policies.
6. Revoked / unrefreshable / insufficient states — the failure classes and the single recovery path = re-run `auth`.
7. Security notes — redaction guarantees (secret values never in errors/logs; "unformattable" token endpoint body types; no secrets in Display/Debug), gitignore rules table (`credentials/*`, `!credentials/.gitkeep`, `/config/config.json`), what an operator must never commit.
8. Verification evidence + limitations — the offline-test method (fakes + loopback ureq servers, suite count), and the honest statement block: NO live Google authorization was performed in this phase; native Windows/Linux execution NOT RUN; cross-compilation ≠ native runtime.

### 4.3 `docs/configuration.md` updates

- Field Reference: add `### oauth` section — `oauth.client_config_path` (non-blank; default `credentials/client_secret.json`) and `oauth.credentials_path` (non-blank; default `credentials/tokens.json`); state the distinctness rule and point to `docs/authentication.md#first-authorization-interactive-once`.
- Validation Rules: add the three oauth checks (two blanks + distinctness; 18 → 21 checks wording replaced by 21).
- Command-Line Usage and Exit Codes: rewrite the `auth` bullet (real flow now; exit codes 0/1; never 3) and the exit-code summary (`3 = run placeholder only`); remove the stale "authentication is not implemented yet" sentence.
- Sections/`example` bullet: add the oauth section exists in the example template (safe, no secrets).

### 4.4 `credentials/` note — DECISION B (no README file)

- Do NOT create `credentials/README.md`: the `.gitignore` rule `credentials/*` with only `!credentials/.gitkeep` negation would make the file untracked; tracking it would require a new gitignore negation, weakening the blast-radius-minimizing credentials rule for zero added value. The content lives in `docs/authentication.md`.
- `docs/authentication.md` explicitly says `credentials/` is operator-owned and its only tracked content is `.gitkeep`.

### 4.5 `docs/build.md` / `docs/build-windows.md`

- Verify-only: confirm no sentence became stale because of Phase 04 (deps affect nothing they claim). EXPECTED no-op; if a stale sentence is found, require caller approval before editing (no silent changes).

### 4.6 `tech.md`

- Dependencies (policy): add the decided-during-Phase-04 bullet: `oauth2 = { version = "5.0", default-features = false, features = ["ureq"] }` + `ureq = "2"` (locked 2.12.1) — rationale: actively-maintained OAuth 2.0 client covering authorization-code + PKCE + token refresh with a pluggable HTTP transport; `ureq` keeps it synchronous (no Tokio), Rust 1.82 / edition-2021 compatible, Windows+Linux, MIT OR Apache-2.0; `google-youtube3` stays REJECTED (brief §3.3). Add one line: the effective-authorization verifier probes `GET https://www.googleapis.com/youtube/v3/channels?part=id&mine=true` with the Bearer header and maps 200/401/403 (+errors by status-first classification); loopback-test evidence pointer to the T4 adherence report.

### 4.7 `.agent/project-structure.md` full refresh

- `youtube/` block: replace the placeholder paragraph with the real per-file map (14 prod + 11 test files, one concise description line each, matching current mod.rs re-export note); keep the deliberate-staging paragraph about the two `#[allow(dead_code)] mod` declarations (credential_service, google_authorization_verifier) and the run-phase retirement contract.
- `app/` block: add the `auth/` (8 files) map and update `mod.rs`/`modes.rs` lines (auth dispatch real; run placeholder retained; allow-inventory note itemizing the 14 `#[allow(dead_code)]` items with one effectively-grouped sentence).
- Root files: `Cargo.toml` line → v0.7.0 + deps list updated (oauth2, ureq added alphabetically); keep all other entries.
- Other folders: `credentials/` line mentions `.gitkeep` only; `docs/` gains `authentication.md`; `config/` gains the example oauth-section note.

### 4.8 Docs-specialist step placement

- The docs step (sub-step 4.4 in the caller's cycle for this task) authors BOTH public docs (README, configuration, authentication.md, tech.md, project-structure.md) AND the context.md update (per the caller's instruction the context.md writer is assigned to the docs step; project markdown rules allow it).

## §5 Completion report skeleton — `.agent/reports/20261012-phase-04-oauth-credentials.md`

Mirror Phase 03 report structure exactly (TOC first). Sections:

1. **Overview** — branch, plan-time HEAD (`ee15453`), report date, TODO authority, phase summary line, and the standing caveat paragraph (container-only; cross-compilation ≠ native runtime; NO live Google authorization).
2. **Files Created/Changed** — table per task with files and commits, using the observed `git log --oneline main..HEAD` list reproduced verbatim in §Evidence Index; include the 55-commit window from `125430f` (version bump) through the final docs/report commits of this task.
3. **Dependency Rationale** — `oauth2` 5.0 (locked 5.0.0; default-features=false, `ureq` feature; no Tokio enters the tree) + `ureq` 2.x (locked 2.12.1); why NOT `reqwest` (async runtime) and why `google-youtube3` stays rejected (brief §3.3); note Task 2–5 chain added NO further dependencies (Cargo.toml diff ranges cited).
4. **Test Coverage Audit (TODO Task-5 mapping)** — the §1 matrix rows + the §2 per-suite inventory (fresh-gate-derived counts, 247→248 after the new console test) + the arithmetic chain 195→218→247 (+1).
5. **Exact Docker/MCP Commands & Exit Statuses** — grouped FRESH (each with verbatim exit status + gate log path + key summary lines) vs RECORDED-CITED (prior-phase logs and adherence reports); include the secret-audit command block and expected outputs.
6. **Linux Build Result** — exit status, artifact path, size, `ls -l` cross-check, `dist/` ignored proof.
7. **Windows Build Result** — exit status, artifact path, size, cross-check, cross-compilation-only statement.
8. **Manual Platform Tests Actually Performed** — honest block: `Native Windows execution: NOT RUN`, `Native Linux execution: NOT RUN`, `Live Google authorization: NOT RUN (no real Google account was used)`; performed = container-only evidence index (fresh gate runs, builds, audits, prior recorded VM smokes cited from adherence reports).
9. **Acceptance Criteria Verification** — 13-row table (short criterion | evidence | flip decision `ready`).
10. **Documentation Accuracy Audit** — the phase-03-style table: TOC/anchor checks, README auth-bullet/exit-line correctness, configuration.md oauth Field Reference + 21-check count + auth bullet rewrite, authentication.md section completeness, project-structure vs `git ls-files src/youtube src/app/auth`, tech.md freshness, claim-scan row (no "native"/"live authorization"/"encrypted" over-claims anywhere in live docs).
11. **Security Review** — redaction enforced at type level (wrapper types with redacting Debug, no Display; unformattable token-endpoint response body via deserialization-only derive — a stray format attempt is a compile error); gitignore rules intact with fresh `git check-ignore -v` hits recorded verbatim (`credentials/*` line 52 ×2, `/config/config.json` line 48 …); `git ls-files credentials/` = `.gitkeep` only; the five fixture-secret strings (`access_token_fixture`, `refresh_token_fixture`, `client_secret_fixture`, `authorization_code_fixture`, `pkce_verifier_fixture_value_43_chars_min`) confined to `*_tests.rs` (record the grep command + hit list); no token/secret-shaped strings in tracked docs.
12. **Environment Notes** — pinned image, Compose version, autocrlf/.gitattributes notes, owned-volumes, MCP stderr caveat.
13. **Evidence Index** — plans (global + per-task + adherence + this plan), prior reports, fresh gate logs, and the full verbatim commit list.
14. **Known Limitations & Not-Run Checks** — (a) Windows rename arm untestable in the Linux container (task2 L1); (b) Windows ACL limitation (task2 L4, no encryption); (c) NO live end-to-end OAuth with Google was performed — the flow is validated exclusively offline/loopback; (d) native Windows/Linux runtimes NOT RUN; (e) cross-compilation ≠ native runtime; (f) unverified-app screen expectation on first authorization (docs only, not exercised); (g) the persisted token schema is the frozen 4-key JSON (no versioning field this phase); (h) listener read-timeout path untested by design; L-series carried items from T4 (advisory `expires_in_seconds`, no size cap on `into_string()`, ureq Display drift).

## §6 Execution order with commit points and the secret-audit protocol

The implementer runs steps A→F in order. Every step: `git status --porcelain` BEFORE staging; NEVER stage anything matching `.gitignore`; one concern per commit.

### Step A — coverage-gap test (implementer)
1. Implement §3 (console_tests.rs + test-foot). 
2. V0 spot check: `docker compose … run --rm rust cargo test --locked app::auth` → exit 0 with the new tests passing (report the observed total).
3. Fresh gate: `docker compose … run --rm rust sh scripts/dev-checks.sh` → record exit 0 + log path + `248 passed; 0 failed` + four PASS lines.
4. Gitignore compliance: `git status` — stage ONLY `src/app/auth/console.rs` + `src/app/auth/console_tests.rs`.
5. **Commit `test: cover the auth console output contract`**.

### Step B — build evidence (implementer; FRESH, in this order)
1. V-Linux: `docker compose … run --rm rust sh scripts/build-linux.sh` → record exit status + printed size; `ls -l /rust-youtube-streamer/dist/rust-youtube-streamer-service`.
2. V-Windows: `docker compose … run --rm rust sh scripts/build-windows.sh` → record exit status + printed size; `ls -l /rust-youtube-streamer/dist/windows/rust-youtube-streamer-service.exe`.
3. V-dist-hygiene: `git check-ignore -v dist` (expected rule hit `.gitignore:34:dist/`); `git ls-files dist/` → MUST be empty; `git status --porcelain --ignored=matching -- dist/` → `!! dist/`.
4. Record every observed value verbatim (do not copy ANY size from this plan — sizes change with code; write what the build scripts print).

### Step C — public docs (docs step; README + docs/authentication.md + docs/configuration.md + the 4.5 verify-only pass)
- Commit split: commit 1 **`docs: document the oauth authorization flow and credentials`** (README.md, `docs/authentication.md`, `docs/configuration.md`); commit 2 (only if build docs needed touch — currently expected none) as `docs: align build guides with phase 04 evidence`.

### Step D — project metadata + completion report (docs step; planner-owned markdown)
1. `.agent/project-structure.md` per §4.7; `tech.md` per §4.6; `.agent/project-info/context.md` (last; per caller decision the docs step writes it) per Phase-03 precedent: rewrite Current Work Focus (Phase 04 shipping OAuth layer; next = broadcast/RTMPS/chat phases), append Recent-Changes block, add Known Deviations (Phase 04) list = report §14 items, revise Immediate Next Steps.
2. Completion report `.agent/reports/20261012-phase-04-oauth-credentials.md` per §5. NOTE: the report's Files/commit/evidence rows must be finalized AFTER every other commit exists (no speculative hashes); the report may cite its own commit plan without embedding a not-yet-existing hash, and a single `docs: record phase 04 completion report correction` follow-up commit MAY follow. DECISION (avoid the Phase-03 speculative-hash pattern that needed corrections `4d12add`/`b2eb632`): the FIRST report commit is created only after every OTHER hash has landed; aim for ZERO correction commits — the report embeds only already-existing hashes.
3. Commits: **`docs: refresh project structure tech and context for phase 04`**; **`docs: add phase 04 completion report`**.

### Step E — plan tracking (planner-owned; NOT the implementer)
- Commit **`docs: track task 5 planning documents`** staging `.kilo/plans/20261012-phase04-task5-tests-docs-evidence.md` + (as they exist untracked) the task-5 adherence file when 4.5b runs. If the adherence report is produced in this cycle, its tracking commit is `docs: track task 5 adherence document`.

### Step F — 4.6 completion + final hygiene audit (implementer)
1. TODO file: change `### 5. Tests, Documentation, and Completion Evidence` → append ` [DONE]`; flip ALL 13 acceptance-criteria checkboxes `[ ]` → `[x]` (do not alter any other byte; overwrite-prevention rule).
2. **Commit `docs: mark task 5 done in phase 04 todo`**.
3. FINAL audit (record in chat summary, cited in report §Security Review + Evidence Index as "final audit confirmations"):
   - `git status --porcelain` → empty or only untracked planner-owned `.kilo/plans/*.md`.
   - `git ls-files credentials/` → `.gitkeep` only.
   - `git ls-files dist/ videos/ logs/` → empty output means zero tracked files in those trees (expected: only `logs/.gitkeep` should ever appear; confirm it stays).
   - `git check-ignore -v credentials/client_secret.json credentials/tokens.json /config/config.json dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe videos/x.mp4 logs/chat.log` → expected rule hits: `.gitignore:52:credentials/*` ×2, `:48`, `:34:dist/`, `:34:dist/` (windows path also hits `dist/`), `:55:videos/`, `:18:logs/*`; record verbatim.
   - Secret sweep (tracked greps):   - `git grep -n "access_token_fixture"` → hits ONLY in `src/youtube/*_tests.rs` and `src/app/auth/*_tests.rs`.  - `git grep -n "refresh_token_fixture\|client_secret_fixture\|authorization_code_fixture\|pkce_verifier_fixture_value_43_chars_min"` → same file set only.  - `git grep -n "refresh_token\|client_secret" -- config docs README.md .agent` → review each hit: only generic paths/schema-key mentions in docs/example allowed; NO literal secret values. Record the hit inventory in the report.  - `git grep -n "ya29."` and a scan for `Bearer ` + any long random-looking literals in non-test tracked files → zero expectations pinned, recorded.
   - Allow inventory check stays 14 items (no new `#[allow(` anywhere in this task's step-A test commit).

### Step G — out of this sub-step (NOT done here)
- 4.5b adherence report (the NEXT architector cycle owns it).
- Global-plan step 5 (TODO rename/merge/push) — a later caller step.
- Nothing else from Tasks 1–4's cycle.

## §7 Boundary and rule envelopes (hard)

- Source-code changes this task: EXACTLY the §3 console test-foot + `console_tests.rs`. NO production `.rs` behavior change, NO `Cargo.toml`/`Cargo.lock` edits, NO new dependencies, NO `.gitignore` edits, NO `modes.rs`/`run`/`demo`/`config` behavior change.
- Auth behavior frozen: the flow, exit codes, six info! events, and console brochure text are locked by Task 3's byte-equals; test-only additions must not alter any production file except the `#[cfg(test)]` foot lines.
- Every code file ≤ 200 lines; test bodies may unwrap/expect; no panics in non-test paths; hand-written Display, no thiserror; single-section booleans; ≤ 2 params; self-documenting names (no new comments except the required module-doc limitations).
- Documentation must NEVER claim: live Google end-to-end authorization happened, native Windows/Linux runtime validation, Windows ACL hardening, or encryption. Honest-limitation wording (§4 blocks) is mandatory.
- Gitignore compliance before every commit; push only later per global plan step 5 (NOT in this task).

## §8 Self-check against TODO Task 5 bullets

| TODO Task 5 bullet | Plan coverage |
| --- | --- |
| Focused unit tests for oauth config validation / state-callback / exchange ± / refresh ± / atomic persistence / permission restrictions / redacted diagnostics | §1 matrix + §3 gap test (all others landed T1–T4, verified by adherence reports) |
| Integration tests at local fake HTTP/callback boundaries; no Google account/network | §1 AC#9 verdict; new tests offline; report §Test-Coverage states the verified harness method |
| Run dev-checks gate | §6 Step A.3 (fresh, after the last src change) |
| Rebuild Linux + Windows | §6 Step B, observed-only sizes, cross-compile caveat |
| Update README + auth/development docs, project-structure.md, project-info context.md | §4.1, 4.2, 4.3, 4.6, 4.7, 4.8 |
| Tracked report with dependency rationale, changed files, exact commands/exit statuses, tests, builds, security review, manual checks, known limitations | §5 skeleton, all present |
| Confirm no token/secret/code/credential/video committed to git | §6 Step F final audit protocol (recorded verbatim results in the report) |

## §9 Plan self-check (locked)

- All 13 ACs have named evidence (§1); the only gap found = console output contract → exactly 1 new test file, 2 test fns (§3), spec complete.
- Every step has exact commit messages; no speculative hash embedding (§6 Step D.2 decision).
- All verification commands spelled verbatim; observed values are never hardcoded from this plan.
- Honest limitation statements appear in: report §Overview/§Manual/§Known Limitations, README Current Status + Authentication section, docs/authentication.md §8; the standing block is repeated in the docs.
- No scope creep: §7's hard boundary list is the audit reference for 4.5b.

(End of plan.)
