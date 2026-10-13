# Task 5 Review Fix Plan — Phase 04 OAuth Credentials

## Review Verdict

ADHERENT with **1 documentation fix required**.

## Required Fix

| Severity | File | Line | Current text | Required change | Acceptance check |
| --- | --- | --- | --- | --- | --- |
| low | `.agent/project-structure.md` | 17 | `... / console_tests.rs - 29 unit tests (...)` | `... / console_tests.rs - 25 unit tests (mod_tests 4 + flow_tests 7 + flow_exchange_tests 4 + listener_tests 8 + console_tests 2)` | After the edit, the `app/auth/` test count in the structure map matches the observed suite total (`cargo test --locked app::auth` → `25 passed`). A fresh grep count of the five `*_tests.rs` files sums to 25, and the per-suite inventory in `.agent/reports/20261012-phase-04-oauth-credentials.md` (25 auth tests) stays consistent. |

## Rationale

The actual `src/app/auth/**` test inventory is:

- `mod_tests.rs` — 4
- `flow_tests.rs` — 7
- `flow_exchange_tests.rs` — 4
- `listener_tests.rs` — 8
- `console_tests.rs` — 2
- **Total — 25**

`.agent/project-structure.md` currently states 29, which conflicts with the fresh gate log and the report's own per-suite table. All other structure-map facts (file lists, line counts, the 11-item `#[allow(dead_code)]` inventory, `Cargo.toml` v0.7.0 deps, `credentials/` content) were verified and are correct.

## Adjudicated Deviations (no code/docs changes)

1. **Suite total 249 vs plan guidance 248** — accepted. The plan's §3 spec explicitly defines two test functions in `console_tests.rs`; the observed total governs per plan §2 and is accurately recorded.
2. **rustfmt rewraps** — accepted. Mechanical formatting-only changes, behavior-preserving, required to pass the `fmt-check` gate.
3. **Console test child-process stdout capture (`Command` + `--exact` + env-gated print mode)** — accepted. Offline, no dev-deps, no production behavior change, and both tests pass.
4. **Build-guide factual-drift correction (2 sentences)** — accepted. The caller's Step-C instruction authorized correcting stale "auth is a placeholder" wording; the edits are factual and limited.
5. **`#[allow(dead_code)]` inventory 11 vs T4 report's 14** — accepted. Fresh `git grep -n -F "#[allow(dead_code)]" src/` shows 11 attribute lines; the T4 report's total was an internal arithmetic slip as already noted in `context.md`.
6. **`console_tests.rs` 81 lines vs plan §3 ≤80 guidance** — accepted. The hard 200-line source-file cap holds; the one-line overshoot is a guidance-only projection and requires no change.

## What Was Done

- Read the governing TODO, Task 5 plan, completion report, README, all touched docs, `src/app/auth/console.rs` + `console_tests.rs`, `src/config/model.rs`, `config/config.example.json`, `Cargo.toml`, `.gitignore`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, and `.agent/project-info/context.md`.
- Verified the fresh gate in the Alpine VM: `249 passed; 0 failed; 0 ignored; ALL CHECKS PASSED` (new log `logs/checks/20261013T181405Z.log`).
- Cross-checked artifact sizes (`dist/rust-youtube-streamer-service` 4956728 bytes; `dist/windows/rust-youtube-streamer-service.exe` 8382035 bytes) against the report.
- Verified per-suite test-count arithmetic sums to 249 and matches the gate log.
- Verified the 11-item `#[allow(dead_code)]` inventory and that Task 5 added no new `#[allow(` attributes.
- Verified all source files remain ≤200 lines.
- Ran secret-sweep greps: fixture strings are confined to `*_tests.rs`; no `ya29.` literals; `Bearer` mentions in docs are generic; `client_secret`/`refresh_token` mentions in docs are paths/schema keys only.
- Verified git hygiene: working tree clean, `credentials/` contains only `.gitkeep`, `dist/`/`videos/` tracked-file lists empty except `logs/.gitkeep`.

## What Was NOT Done

- No source files, docs, or metadata were modified (this is a review-only sub-step).
- The TODO file acceptance-criteria flips and final 4.6 secret audit were not performed (caller-owned 4.6).
- No build scripts, `Cargo.toml`, `.gitignore`, `modes.rs`, `run`, `demo`, or config behavior were changed.
