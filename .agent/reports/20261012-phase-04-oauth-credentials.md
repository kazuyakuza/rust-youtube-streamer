# Phase 04 Completion Report — OAuth 2.0 and Credential Management

## Table of Contents

- [Overview](#overview)
- [Files Created/Changed](#files-createdchanged)
- [Dependency Rationale](#dependency-rationale)
- [Test Coverage Audit (TODO Task-5 mapping)](#test-coverage-audit-todo-task-5-mapping)
- [Exact Docker/MCP Commands & Exit Statuses](#exact-dockermcp-commands--exit-statuses)
- [Linux Build Result](#linux-build-result)
- [Windows Build Result](#windows-build-result)
- [Manual Platform Tests Actually Performed](#manual-platform-tests-actually-performed)
- [Acceptance Criteria Verification](#acceptance-criteria-verification)
- [Documentation Accuracy Audit](#documentation-accuracy-audit)
- [Security Review](#security-review)
- [Environment Notes](#environment-notes)
- [Evidence Index](#evidence-index)
- [Known Limitations & Not-Run Checks](#known-limitations--not-run-checks)

## Overview

- Branch: `feat/phase04-oauth-credentials`; package `rust-youtube-streamer-service` v0.7.0; plan-time HEAD `ee15453` (clean tree).
- Report date: 2026-10-13. TODO authority: `.agent/todos/20261012/20261012-todo-1.md` (Tasks 1–4 `[DONE]`; Task 5 completed by this task's cycle, its `[DONE]`/checkbox flips land in the caller-owned 4.6 step).
- Phase summary: the `auth` command now performs a real Google OAuth 2.0 installed-application authorization flow (browser + loopback-only callback + PKCE S256 + token exchange + minimum-scope verification + atomic credential persistence), backed by a reusable credential layer (load → verify-first → refresh → persist) under `src/youtube/` for the future production `run` mode. `run` remains a non-interactive placeholder (exit 3); `demo` is byte-unchanged.
- **Standing caveat (applies to every claim in this report): verification is container-only (Linux, Alpine VM). Cross-compilation of the Windows `.exe` is not native runtime validation. NO live Google authorization was ever performed in this phase — the entire flow is validated offline with injected fakes and loopback (`127.0.0.1`) HTTP servers; no real Google account or live client secret was used anywhere.**

## Files Created/Changed

55 commits from the version bump `125430f` through this task's Step-C commits (`471ba7a`); the full verbatim list is in [Evidence Index](#evidence-index). Per-task grouping:

| Task | Files (created/changed) | Commits |
| --- | --- | --- |
| Version bump | `Cargo.toml` (0.6.0 → 0.7.0) | `125430f` |
| 1 — OAuth design + youtube skeleton | `Cargo.toml`/`Cargo.lock` (oauth2 + ureq), `src/youtube/{mod,error,client_config,token,transport,google_token_endpoint,authorization,verify,credentials,persisted_credentials}.rs` + 10 test siblings, removed `src/youtube/.gitkeep` | `9aa7bdd`, `1a943b3`, `f1aa085`, `d27fb15`, `b4251b2`, `cd7124a`, `0482310`, `851b994`, `3a1c8f4`, `6ce1ee6`, `1d10522`, `d1009e0`, `ae4e866`, `3671d5e`, `bd8f8f0`, `8ca6149`, `11f05ed`, `6ba5fe1`, `b3460f9`, `5596ce4`, `98d93fe`, `ae37a62` |
| 2 — OAuth config + credential storage | `src/config/{model,validation,validation_tests}.rs`, `config/config.example.json`, `src/youtube/file_credential_store.rs` + tests, `src/youtube/token.rs` (unformattable body), docs | `cfc0cc4`, `4d0d637`, `c133827`, `24d05fd`, `42d33fd`, `3997eec`, `09833cc`, `a84b877`, `9ebf13f` |
| 3 — the `auth` command | `src/app/auth/{mod,flow,listener,console}.rs` + 4 test siblings, `src/app/{mod,modes}.rs` (dispatch rewired, placeholder deleted), `src/app/cli_tests.rs`, youtube staging retirement | `7d75989`, `1dfb231`, `1ae8ec5`, `8a50093`, `bd9bfc1`, `be45d70`, `aeed51d`, `bd5cc4a`, `f0b969f`, `9141bbd`, `2829840` |
| 4 — credential service + verifier | `src/youtube/{credential_service,credential_service_error,google_authorization_verifier}.rs` + 3 test siblings, staging allows adjusted, verifier classify-test split | `8416c28`, `f82f4fc`, `fe697e8`, `6f05869`, `4b1eae6`, `e788ed7`, `6d6c79f`, `ee15453` |
| 5 — tests, docs, evidence (this task) | `src/app/auth/{console.rs,console_tests.rs}` (test-only), `README.md`, `docs/authentication.md` (new), `docs/configuration.md`, `docs/build.md`, `docs/build-windows.md`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`, this report | `50b6858`, `57a059d`, `471ba7a`, + the structure/tech/context and report commits listed in Evidence Index |

Source-code changes in Task 5 are EXACTLY the plan §7 envelope: the `#[cfg(test)]` test foot in `console.rs` plus the new `console_tests.rs`. No production `.rs` behavior change, no `Cargo.toml`/`Cargo.lock` edit, no new dependency, no `.gitignore` edit.

## Dependency Rationale

- `oauth2 = { version = "5.0", default-features = false, features = ["ureq"] }` (locked 5.0.0) — actively maintained OAuth 2.0 client covering the authorization-code flow, PKCE (S256), and token refresh with a pluggable HTTP transport; `default-features = false` + the `ureq` feature keeps the whole OAuth path synchronous, so **no Tokio enters the dependency tree**; Rust 1.82 / edition 2021 compatible; Windows+Linux; MIT OR Apache-2.0.
- `ureq = "2"` (locked 2.12.1) — synchronous HTTP client used as the token-endpoint transport backend and by the authorization verifier; same compatibility/license profile.
- Why NOT `reqwest`: it pulls an async runtime, conflicting with the project's synchronous std-based pipeline architecture (tech.md decision S2 lineage).
- Why `google-youtube3` stays REJECTED: brief §3.3 (unsuitable maintenance status); the YouTube layer is hand-written behind this project's own module interfaces.
- Tasks 2–5 added NO further dependencies: `Cargo.toml` diff ranges cover only the Task-1 lines (`oauth2`, `ureq`); dev-deps remain `tempfile` only.

## Test Coverage Audit (TODO Task-5 mapping)

### Acceptance-criterion ↔ evidence matrix (all 13 covered)

| # | Criterion (short) | Evidence | Verdict |
| --- | --- | --- | --- |
| 1 | `auth` is a real Google OAuth flow; no not-implemented exit code | Task 3: `modes.rs` placeholder deleted, dispatch rewired (`1dfb231`); tests `app::auth::mod_tests::every_auth_error_maps_to_the_startup_failure_exit` (all 10 variants exit 1), `run_auth_reports_missing_client_file_with_startup_failure_exit`, `cli_tests::usage_text_advertises_the_auth_command`; auth never exits 3 | covered |
| 2 | Browser authorization + local callback, no manual token paste | `listener_tests` L1-L8 (`bind_uses_loopback_and_ephemeral_port`, `authorized_callback_delivers_the_code`, …), flow F1-F11 incl. `code_exchange_request_assembles_the_documented_fields` and `authorization_url_encodes_all_parameters`; console brochure locked by the new `console_tests` pair | covered |
| 3 | State validated; loopback-only binding; listener closed on success AND failure | `state_mismatch_callback_fails_the_flow` (L4), `listener_socket_is_closed_when_dropped` (L8), L1 (127.0.0.1 only); code: bind host constant `127.0.0.1`, `BoundListener` dropped at `run` return on every path | covered |
| 4 | Separate, configurable client-config/credentials paths | `src/config` tests `oauth_paths_require_non_blank_values`, `oauth_paths_must_be_distinct`, `example_mirrors_oauth_schema`, `unknown_oauth_field_rejected`, `oauth_section_missing_rejected`; defaults `credentials/client_secret.json` + `credentials/tokens.json` in `config/config.example.json:9-12`; docs §Field Reference | covered |
| 5 | Atomic persistence; restrictive permissions where supported; limitations documented | `file_credential_store_tests` (10 tests: `save_is_atomic_replacing_in_place`, interrupted-save shape, `unix_mode_is_owner_only_after_save`, missing-parent creation, directory collision, …) + `persisted_credentials_tests` (5: frozen 4-key schema, round-trips); limitations in module docs + `docs/authentication.md` §Credential Files + [Known Limitations](#known-limitations--not-run-checks) | covered |
| 6 | Refresh from stored refresh token; refreshed data persisted | S-series: `rejected_token_refreshes_saves_and_reverifies`, `refreshed_refresh_token_persists_when_google_respends_one`, `refresh_response_scope_replaces_granted_evidence_when_present`, `refresh_response_without_scope_keeps_the_stored_granted_scopes`, `persistence_failure_maps_to_the_persistence_class`; transport seam tests | covered |
| 7 | Credentials verified against the minimum documented YouTube scope | `REQUIRED_SCOPE = https://www.googleapis.com/auth/youtube.force-ssl` (`authorization.rs`; test `authorization_url_uses_the_force_ssl_scope`); verifier R-series: `verification_request_url_is_the_documented_channels_call` (`GET /youtube/v3/channels?part=id&mine=true`), `bearer_header_and_request_line_reach_the_fake_api`, `server_200_verifies_with_required_scope_evidence`; scope gate `stored_scopes_missing_the_required_scope_fail_before_any_request` | covered |
| 8 | Missing/malformed/revoked/expired-unrefreshable/insufficient → clear non-zero failures without leaking secrets | `absent_credentials_map_to_the_absent_class`, `unreadable_credentials_map_to_the_unreadable_class`, `load_reports_least_details_for_malformed_json`, `load_reports_fixed_schema_mismatch_message`, `revoked_refresh_token_maps_to_the_distinct_revoked_error`, `missing_stored_refresh_token_cannot_refresh`, `insufficient_scope_from_the_verifier_skips_the_refresh_path`, `verification_failure_after_refresh_maps_to_the_verification_class`; redaction: `auth_error_debug_and_display_reveal_no_secret_fixtures`, `service_error_output_never_carries_secret_fixtures`, `error_bodies_are_never_embedded_into_reasons`, `error_tests::each_variant_has_a_stable_display_line` | covered |
| 9 | Automated tests cover all paths, no real Google account/network | Every test is offline: `TokenEndpointTransport`/`CredentialStore`/verifier fakes and ureq loopback `127.0.0.1` servers only; `src/youtube/mod.rs` module-doc seam list; zero non-loopback sockets, zero live-Google calls | covered |
| 10 | `demo` unchanged; `run` placeholder; no broadcast/RTMPS/FFmpeg-production/Live-Chat code | T3/T4 boundary evidence: `git diff` zero over `src/app/demo/**`, `src/app/pipeline/**`, `src/chat/**`, `src/renderer/**`, `src/streaming/**`; `modes.rs` `run` handler intact (exit 3, four-prerequisites message, 4 tests); coupling grep `ffmpeg\|broadcast\|liveChat\|rtmps` over `src/youtube` → doc comment only (T4 adherence) | covered |
| 11 | Docker dev checks pass + both artifacts rebuilt | Produced by THIS task: fresh gate 249 passed (below), both builds exit 0 (below); cross-compile ≠ native runtime stated everywhere | covered (this task) |
| 12 | Docs/structure/context/report accurate | [Documentation Accuracy Audit](#documentation-accuracy-audit) below | covered (this task) |
| 13 | No credentials/tokens/codes/secrets/videos committed | [Security Review](#security-review) below (fresh observations + recorded T4 audit citations); final 4.6 audit appended by the caller's next sub-step | covered (this task; final audit in 4.6) |

### Per-suite inventory (derived from the FRESH gate log `logs/checks/20261013T171123Z.log`)

The authoritative total is the observed gate total: **249 passed; 0 failed; 0 ignored** (verbatim `test result: ok. 249 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s`). The plan's "248" guidance assumed a single test function; the locked §3 spec contains two, and per the plan's own rule (observed totals are authoritative) the recorded figure is 249.

| Suite | Count |
| --- | --- |
| `src/youtube/**` — authorization 8, client_config 7, credential_service (refresh 11 + verification 7, incl. shared fakes) 18, credentials 1, error 2, file_credential_store 10, google_authorization_verifier (loopback + classify) 11, google_token_endpoint 1, persisted_credentials 5, token 7, transport 2 | **72** |
| `src/app/auth/**` — mod_tests 4, flow 11 (flow_tests 7 + flow_exchange_tests 4), listener 8, console 2 (NEW this task) | **25** |
| `src/config/**` — loader 10, validation 11 (16 pre-phase + 5 oauth) | **21** |
| `src/app/cli` 16 (incl. the auth description-line test) | **16** |
| `src/app/tests` 6, `app/error` 2, `app/logging` 7, `app/modes` 4 | **19** |
| `src/app/demo/**` 14 | **14** |
| `src/app/pipeline/**` 15 | **15** |
| `src/chat/**` (model 5 + store 11) | **16** |
| `src/renderer/**` (frame 4 + settings 4 + text 11) | **19** |
| `src/streaming/**` (args 4 + error 4 + ffmpeg 7 + utc_timestamp 7 + video_output 10) | **32** |
| **Total** | **249** |

Arithmetic chain (observed at every gate): 147 (Phase 03) → 174 (T1, +27 youtube) → 195 (T2, +21 = 16 youtube store/persisted + 5 config) → 218 (T3, +24 new auth-and-cli − 1 deleted placeholder test) → 247 (T4, +29 = 18 service + 11 verifier) → **249 (T5, +2 console-brochure tests)**; the 174 waypoint is cited from the Task-1 adherence log (`20261012T232738Z`). The plan §2 spot list disagreed with the log on three youtube files (authorization 10→8, token 8→7, file_credential_store 11→10); the table above uses the log-derived numbers, which still reconcile exactly to the plan's expected youtube total of 72.

## Exact Docker/MCP Commands & Exit Statuses

All commands ran through the Alpine VM MCP (`vm_status` consulted first each time; pattern `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust …`).

### FRESH — Step A.2: V0 spot check of the new console tests

- `docker compose … run --rm rust cargo test --locked app::auth` → **exit 0**; `test result: ok. 25 passed; 0 failed; … (224 filtered out)`; both new tests green (`app::auth::console::console_tests::print_authorization_url_uses_the_documented_brochure`, `…::print_success_names_the_credentials_path_and_scope`).

### FRESH — Step A.3, first gate attempt (fmt drift found and fixed)

- `docker compose … run --rm rust sh scripts/dev-checks.sh` → **exit 1**; log `logs/checks/20261013T171023Z.log`; `FAIL fmt-check exit=1` with two rustfmt diffs (single-line `assert!` wraps in `console_tests.rs`); check/test/clippy still ran (test: 249 passed). Fix: the two `assert!` calls were re-wrapped exactly as rustfmt demanded (mechanical, behavior-preserving).

### FRESH — Step A.3: the authoritative gate (post-fix, code-complete tree)

- `docker compose … run --rm rust sh scripts/dev-checks.sh` → **exit 0**; log `logs/checks/20261013T171123Z.log`; verbatim summary lines:
  - `PASS fmt-check exit=0 (1 s)`
  - `PASS check exit=0 (5 s)`
  - `PASS test exit=0 (12 s)`
  - `PASS clippy exit=0 (0 s)`
  - `ALL CHECKS PASSED`
  - `running 249 tests` / `test result: ok. 249 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s`

### FRESH — Step B: builds and artifact cross-check

- Linux: `docker compose … run --rm rust sh scripts/build-linux.sh` → **exit 0**; printed `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (4956728 bytes).`
- `ls -l /rust-youtube-streamer/dist/rust-youtube-streamer-service` → exit 0; `-rwxrwx--- 1 root vboxsf 4956728 Oct 13 17:14 …` (size matches the printed byte count).
- Windows: `docker compose … run --rm rust sh scripts/build-windows.sh` → **exit 0**; printed `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (8382035 bytes).`
- `ls -l /rust-youtube-streamer/dist/windows/rust-youtube-streamer-service.exe` → exit 0; `-rwxrwx--- 1 root vboxsf 8382035 Oct 13 17:16 …` (size matches).
- Host hygiene (Step B.3, verbatim):
  - `git check-ignore -v dist` → `.gitignore:34:dist/\tdist`
  - `git ls-files dist/` → empty output (zero tracked files)
  - `git status --porcelain --ignored=matching -- dist/` → `!! dist/`

### RECORDED-CITED (per-task adherence reports; not re-executed this task)

- Task 1 gate: 27 youtube tests, log `logs/checks/20261012T232738Z.log` (`…task1-adherence.md`).
- Task 2 gate: 195 total, log `logs/checks/20261013T011250Z.log` (`…task2-adherence.md`).
- Task 3 gate: 218 total, log `logs/checks/20261013T134943Z.log` (`…task3-adherence.md`).
- Task 4 gate: 247 total, log `logs/checks/20261013T163200Z.log` (`…task4-adherence.md`), plus its boundary/host-hygiene/secret-audit observations at `ee15453`.
- The secret-audit command block (fresh `git check-ignore` credential hits, `git ls-files credentials/`, fixture-string greps, `ya29.` scan) runs as the caller-owned 4.6 final audit; this report cites the T4-adherence-recorded values below and marks the 4.6 append point.

## Linux Build Result

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` — **exit 0**.
- Artifact: `dist/rust-youtube-streamer-service`, **4956728 bytes** (printed by the script; `ls -l` cross-check identical: `4956728 Oct 13 17:14`, mode `-rwxrwx---`, `root:vboxsf` on the shared mount).
- `dist/` ignored proof: `git check-ignore -v dist` → `.gitignore:34:dist/`; `git ls-files dist/` empty; `git status --porcelain --ignored=matching -- dist/` → `!! dist/`. The artifact was never staged or committed.

## Windows Build Result

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` — **exit 0**.
- Artifact: `dist/windows/rust-youtube-streamer-service.exe`, **8382035 bytes** (printed by the script; `ls -l` cross-check identical: `8382035 Oct 13 17:16`).
- **Cross-compilation only**: the `.exe` targets `x86_64-pc-windows-gnu` and was produced inside the Linux container; native Windows runtime behavior was NOT validated by this build (the script's own printed caveat line is authoritative).

## Manual Platform Tests Actually Performed

Honest ledger:

- `Native Windows execution: NOT RUN` — no Windows host was used at any point in this phase.
- `Native Linux execution: NOT RUN` — all verification ran inside the Linux container on the Alpine VM (checks/builds/tests only; the produced Linux binary was never executed natively).
- `Live Google authorization: NOT RUN` — no real Google account, no live OAuth client secret, no non-loopback network endpoint was ever contacted.

Performed (container-only evidence): the four fresh Step-A/B commands above (spot run, failed-fmt gate, authoritative 249-test gate, both builds + `ls -l` cross-checks + dist hygiene), plus the RECORDED-CITED per-task gate runs and audits (tasks 1–4 adherence reports at their logged heads).

## Acceptance Criteria Verification

| # | Criterion | Evidence | Flip decision |
| --- | --- | --- | --- |
| 1 | Real Google OAuth flow; no not-implemented exit code | `1dfb231`; mod_tests exit-mapping; cli description test | ready |
| 2 | Browser + local callback, no manual token paste | listener L1-L8; flow F1-F11; console brochure tests | ready |
| 3 | State validated; loopback-only; listener closed on success AND failure | L4/L8/L1; `BoundListener` drop at `run` return | ready |
| 4 | Separate, configurable paths | 5 config oauth tests; example json:9-12; docs | ready |
| 5 | Atomic persistence; permissions where supported; limitations documented | 10 file-store tests + 5 schema tests; docs + module docs | ready |
| 6 | Refresh + persisted refresh data | S-series tests | ready |
| 7 | Minimum documented scope verified | REQUIRED_SCOPE tests; channels-mine verifier R-series | ready |
| 8 | Distinct clear failures, no secret leakage | class-mapping + redaction tests | ready |
| 9 | Offline automated tests only | fake/loopback harness; zero live-Google calls | ready |
| 10 | `demo` unchanged; `run` placeholder; no out-of-scope code | zero-diff boundaries; `run` exit-3 tests | ready |
| 11 | Dev checks pass + both artifacts rebuilt | 249-test gate `20261013T171123Z`; both builds exit 0 | ready |
| 12 | Docs/structure/context/report accurate | Documentation Accuracy Audit below | ready |
| 13 | No secrets/tokens/videos committed | Security Review below; 4.6 final audit appends | ready |

All 13 criteria are `ready` for the caller-owned 4.6 checkbox flips.

## Documentation Accuracy Audit

| Check | Result |
| --- | --- |
| README TOC ↔ sections | PASS — new `Authentication (OAuth 2.0 — Key Features)` section + TOC entry `#authentication-oauth-20--key-features`; all other entries unchanged |
| README auth bullet + exit-code line | PASS — real-flow bullet (exit 0 verified success / exit 1 any failure, never secrets) and exit-code summary `3 = mode not implemented yet (run)` |
| README Current Status | PASS — Phase 04 state, 249 fresh count, honest limitations (no live Google, native runtimes NOT RUN) |
| README Logging section | PASS — unchanged; the 6 auth `info!` events are claimed nowhere |
| configuration.md oauth Field Reference | PASS — `oauth.client_config_path` / `oauth.credentials_path` with defaults, distinctness rule, link to `docs/authentication.md#first-authorization-interactive-once` |
| configuration.md validation wording | PASS — 21 checks (18 + 2 blanks + 1 distinctness) |
| configuration.md auth bullet + exit summary | PASS — rewritten; stale "authentication is not implemented yet" sentence removed; `3 = the run placeholder only` |
| configuration.md example bullet | PASS — oauth section in the example template noted (safe, no secrets) |
| authentication.md completeness | PASS — 8 sections incl. unverified-app screen, persistence/permissions + platform limitations, unattended refresh under Google policies, revoked/unrefreshable recovery, evidence/limitations with the mandatory honest wording |
| `credentials/README.md` | NOT CREATED (Decision B, per plan §4.4) — content lives in `docs/authentication.md`, which states `credentials/` is operator-owned with `.gitkeep` as the only tracked content |
| build docs drift | CORRECTED (2 sentences): `docs/build.md` and `docs/build-windows.md` still claimed `auth` is a not-implemented placeholder — factually stale after Task 3; fixed in commit `471ba7a` (`docs: align build guides with phase 04 evidence`). This is the plan-§4.5-anticipated commit-2 case (factual drift found → correction authorized by the caller's Step-C instruction) |
| project-structure vs `git ls-files src/youtube src/app/auth` | PASS — 28 youtube files + 9 app/auth files, all enumerated; the stale `src/youtube/.gitkeep` note replaced (marker removed in `f1aa085`) |
| tech.md freshness | PASS — oauth2/ureq decision + verifier probe summary appended |
| context.md freshness | PASS — Current Work Focus, Recent Changes (Phase 04 block), Known Deviations (Phase 04), Immediate Next Steps rewritten |
| Claim-scan | PASS — no "native runtime", "live authorization performed", "Windows ACL hardening", or "encrypted" over-claim exists in `README.md`, `docs/*.md`, this report, or the touched `.agent` files; the honest-limitation wording appears in README Current Status + Authentication section, `docs/authentication.md` §Verification Evidence, and this report's Overview/Manual/Limitations |

## Security Review

- Redaction is enforced at the type level: token/secret wrapper types carry a redacting `Debug` and no `Display`; the token-endpoint response body deserializes only — a stray format attempt is a compile error (commit `c133827`).
- Gitignore rules intact (fresh this task for dist; credential rules re-verified in the T4 audit at `ee15453`, unchanged since — no `.gitignore` edit exists in this phase's 55-commit window): `git check-ignore -v` hits `.gitignore:52:credentials/*` (×2 paths), `:48:/config/config.json`; `git check-ignore -v dist` → `.gitignore:34:dist/` (fresh).
- `git ls-files credentials/` → `credentials/.gitkeep` only (fresh this task).
- `git ls-files dist/` → empty (fresh this task).
- The five fixture-secret strings (`access_token_fixture`, `refresh_token_fixture`, `client_secret_fixture`, `authorization_code_fixture`, `pkce_verifier_fixture_value_43_chars_min`) are confined to `*_tests.rs` files (T4-adherence grep inventory; unchanged by this task — `console_tests.rs` uses only the non-secret client-id fixture `1234567890.apps.googleusercontent.com`, identical to the flow-tests fixture value).
- No token/secret-shaped strings in tracked docs; no `ya29.`-style access-token literal exists in the tree (T4 audit; unchanged this task — Task 5 touched docs contain only generic path/schema-key mentions).
- The final 4.6 secret audit (fresh re-runs of the full grep block + `git status`/`ls-files` sweep at the final tree) is appended by the caller's next sub-step; nothing this task changed touches credential material (test-only + docs-only diff).

## Environment Notes

- Pinned image: official `rust:1.82` (rustfmt + clippy + `gcc-mingw-w64-x86-64` + `x86_64-pc-windows-gnu` target + ffmpeg dev tooling); Docker Compose v2.31.0 observed on the Alpine VM.
- Repo-host git `core.autocrlf=true`; tracked `.gitattributes` forces LF for `*.sh` and `Dockerfile` (Phase 01.2 incident; still the operative guard). Git prints benign `LF will be replaced by CRLF` warnings on staging — expected.
- Cargo intermediates live in the `rust-streamer-target` named volume; no root `target/` ever appears; `Cargo.lock` is the sole tracked root artifact.
- MCP caveat: cargo stderr (`Finished` lines) may be partially captured in VM output; exit status is the authoritative evidence (in this task stderr was fully captured, but the policy was applied).

## Evidence Index

### Plans and adherence documents

- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md`
- Task plans: `…task1-oauth-design.md`, `…task2-oauth-config-storage.md`, `…task3-auth-command.md`, `…task4-credential-service.md`, `…task5-tests-docs-evidence.md`
- Adherence: `…task1-adherence.md`, `…task2-adherence.md`, `…task3-adherence.md`, `…task4-adherence.md` (+ review/simplify docs tracked alongside)

### Fresh logs (this task)

- Gate (fmt-fail first attempt): `logs/checks/20261013T171023Z.log` (exit 1, FAIL fmt-check)
- Gate (authoritative): `logs/checks/20261013T171123Z.log` (exit 0, ALL CHECKS PASSED, 249 passed)

### Prior reports

- `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (style source + Phase 03 baseline evidence)
- `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` (Windows build-workflow evidence)

### Full verbatim commit list (`git log --oneline main..HEAD`, 55 commits at report time)

```
471ba7a docs: align build guides with phase 04 evidence
57a059d docs: document the oauth authorization flow and credentials
50b6858 test: cover the auth console output contract
ee15453 docs: track task 4 adherence document
6d6c79f docs: mark task 4 done
e788ed7 docs: document the credential service and verification contract
4b1eae6 test: split the verifier classifier tests to respect the file cap
6f05869 chore: retire the task 4 reserved staging allowances
fe697e8 feat: add the reusable credential service with refresh
f82f4fc feat: add the channels-mine authorization verifier
8416c28 docs: track task 4 implementation plan
2829840 docs: track task 3 review simplification and adherence documents
9141bbd docs: mark task 3 done
f0b969f docs: document the auth command flow and listener contract
bd5cc4a test: merge the client file fixture writer into the parsed helper
aeed51d refactor: require stored refresh token with ok_or
be45d70 refactor: move code exchange fields instead of cloning
bd9bfc1 fix: keep the callback writer within two parameters
8a50093 chore: compress new and touched files to the 200-line rule cap
1ae8ec5 chore: retire the youtube module staging allowance
1dfb231 feat: replace the auth placeholder with the oauth authorization flow
7d75989 docs: track task 3 implementation plan
9ebf13f docs: track task 2 review and adherence documents
a84b877 docs: mark task 2 done
09833cc docs: document oauth configuration and credential storage
3997eec test: assert oauth blank paths aggregate in one error
42d33fd docs: track task 2 plan amendments
24d05fd feat: add the atomic file credential store
c133827 fix: make the token endpoint body unformattable
4d0d637 feat: add oauth path settings to the application configuration
cfc0cc4 docs: track task 2 planning documents
ae37a62 docs: track task 1 planning review and adherence documents
98d93fe docs: mark task 1 done
5596ce4 docs: document youtube oauth module boundaries and redaction contract
b3460f9 test: share the youtube authorization url fixture
6ba5fe1 test: deduplicate the youtube fake transports
11f05ed test: deduplicate the youtube token response fixtures
8ca6149 test: use the let-else idiom in the client config error cases
bd8f8f0 refactor: share the oauth client build between both token exchanges
3671d5e refactor: split the google token endpoint backend from the transport seam
ae4e866 refactor: derive the token response debug and simplify scope parsing
d1009e0 refactor: scope youtube oauth error enums to the crate
1d10522 refactor: split the youtube error display test into per-enum helpers
6ce1ee6 refactor: keep endpoint validation helpers within two parameters
3a1c8f4 fix: parse google oauth endpoint constants without panics
851b994 feat: add credential persistence and authorization verification seams
aa83e9b feat: add oauth authorization url builder and callback parser
0482310 feat: add oauth token endpoint transport seam with ureq backend
cd7124a feat: add redacted oauth token response types
b4251b2 feat: parse google oauth client configuration
d27fb15 feat: add youtube module skeleton with typed oauth errors
f1aa085 chore: remove youtube folder placeholder marker
1a943b3 build: add oauth2 and ureq crates for the youtube module
9aa7bdd docs: track task 1 planning documents
125430f chore: bump version to 0.7.0
```

(Task-5 note: the structure/tech/context commit and this report's commit land after the three listed Task-5 commits; by plan §6-D2 the report embeds only already-existing hashes — its own commit hash is intentionally absent, and the plan-tracking commit follows it.)

## Known Limitations & Not-Run Checks

1. **Windows rename arm untestable in the Linux container** (Task-2 L1): the atomic save's rename-over-target behavior on Windows cannot be exercised by the container suite; the POSIX 0600 mode arm IS covered.
2. **Windows ACL limitation** (Task-2 L4): credential files on Windows inherit directory ACLs only (std has no ACL management); there is NO encryption at rest on any platform — documented, never claimed otherwise.
3. **NO live end-to-end OAuth with Google was performed** — the flow is validated exclusively offline/loopback; no real Google account or live client secret was used.
4. **Native Windows/Linux runtimes NOT RUN** — the Linux artifact was built but never executed natively; the Windows `.exe` is cross-compilation only.
5. **Cross-compilation ≠ native runtime** — stated wherever the Windows artifact is claimed.
6. **Unverified-app screen** (`Google hasn't verified this app`) is documented as an operator expectation but was never exercised against Google's live pages.
7. **Frozen 4-key token schema** (`access_token`, `expires_in_seconds`, `refresh_token` nullable, `granted_scopes`) has no versioning field this phase.
8. **Listener read-timeout path untested by design** (Task-3 limitation).
9. **T4 carried items**: `expires_in_seconds` is advisory TTL metadata only (verify-first refresh, no clock-based decision); no size cap on the token-endpoint `into_string()` read; ureq Display-format drift is a tolerated future-maintenance risk; test-profile warn-only dead-code warnings (`with_token_url`, `token_type`) adjudicated ACCEPTABLE in the T4 review.
