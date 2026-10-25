# Task 1 Stop-Path Isolation — 4.3 Review Fix Plan

Plan date: 2026-10-25 · Author: Code Reviewer (4.3) · Binding plan: `.kilo/plans/20261025-stop-path-task1-implementation.md`

## Review Verdict

ALMOST CONFORMANT. One minor naming correction is required in the new isolation roster so the test name and its survival-window constants truthfully reflect the caller-authorized redesign (direct-pid delivery, not a parent-group SIGINT broadcast).

The implementation otherwise satisfies every binding decision (D1–D12), the byte-stability ledger, the rule set, and the gate evidence (389 passed / 0 failed; `logs/checks/20261025T034206Z.log`).

## Required Correction

### R1 — Rename the misleading test-1 name and survival constants

**File:** `src/streaming/ffmpeg_isolation_tests.rs`

**Why:** The caller authorized the §8 STOP redesign: the container puts every process in pgid 1, so a group-directed SIGINT is rejected/unsafe; test 1 now delivers the stop signal by direct pid (`/bin/kill -s INT <test-pid>`). The existing function name and constants still say `parent_group_sigint`, which is misleading even though the module-level and test-level doc comments correctly describe the honest proof structure.

**Exact edits (junior-executable):**

1. Rename the constants on lines 33–34:

   - `PARENT_GROUP_SIGINT_PROOF_WINDOW_MS` → `STOP_SIGNAL_SURVIVAL_WINDOW_MS`
   - `PARENT_GROUP_SIGINT_PROOF_ATTEMPTS` → `STOP_SIGNAL_SURVIVAL_ATTEMPTS`

2. Rename the test function on line 124:

   - `stream_child_survives_parent_group_sigint_and_finalizes_on_stdin_eof`
   - → `stream_child_survives_direct_stop_signal_and_finalizes_on_stdin_eof`

3. Update the two references inside the test body (the for-loop on line 145 and the `steps`/`proof_window` computation on lines 156–157) to use the new constant names.

4. No other behavior changes. Do not touch the 4-point honest doc comment, the direct-pid sender, the handler flag logic, the structural test, or any production code.

**Verification after fix:**

- `cargo fmt --check` passes.
- `cargo test --locked -- isolation` passes (2 tests, including the renamed test).
- `cargo test --locked streaming` passes (54 tests).
- `cargo test --locked interrupt` passes (3 tests, unchanged count).
- Full gate count remains 389 / 0.

## Conformance Summary (what was verified and found correct)

### (i) spawn.rs isolation
- Two cfg-gated helpers (`#[cfg(target_family = "windows")]` and `#[cfg(target_family = "unix")]`) are present, each ≤2 args and single-section.
- Named constants `CREATE_NEW_PROCESS_GROUP = 0x0000_0200` and `ISOLATED_PROCESS_GROUP = 0` are documented.
- `prepare_piped_command` calls `isolate_from_console_control(command)` as the last builder step.
- Module doc truthfully names the isolation duty and the CTRL_BREAK caveat is covered in `interrupt.rs`.

### (ii) ffmpeg.rs roster declaration
- `#[cfg(all(test, unix))] #[path = "ffmpeg_isolation_tests.rs"] mod ffmpeg_isolation_tests;` is correctly placed immediately after `ffmpeg_tests`.
- No production-shape changes beyond the isolation.

### (iii) New isolation roster
- Both tests are present.
- Test-2 structural pin (`stream_child_leads_its_own_process_group`) asserts `child pgid == child pid`.
- Test-1 uses the caller-authorized direct-pid delivery and asserts handler flag flip + child survival + EOF exit 0.
- The 4-point honest doc comment is present and accurate.
- All four collateral `#[cfg(unix)] process_group(0)` patches are present on the out-of-spawn child sites (`console_tests.rs`, `ffmpeg_real_tests.rs`, `ffmpeg_real_audio_tests.rs` ×2).

### (iv) interrupt.rs
- Limitation item 6 added verbatim (doc-only).

### (v) Byte-stability
- `git diff` confirms only the 7 expected files changed in the feat commit; no edits to `args.rs`, `stream_args.rs`, `error.rs`, run exit mapping, `RUN_STOP_MESSAGE`, diagnostics constants, auth/YouTube surfaces.
- Demo `Interrupt → 1` mapping untouched.
- Single-`error:`-line contract untouched.

### (vi) Rules
- `#[allow(` count remains 8 attribute lines + 1 comment mention; zero new.
- No commented-out code.
- Private-by-default helpers/constants in `spawn.rs`.
- All changed functions respect ≤2 args.
- All changed file bodies respect ≤50 lines; files respect ≤200 lines (`spawn.rs` 110, roster 197, `console_tests.rs` 88, `ffmpeg_real_tests.rs` 113, `ffmpeg_real_audio_tests.rs` 169).
- Max depth respected.

### (vii) Docs commit
- `0cc59df` changes only `.agent/project-info/context.md` and `.agent/project-structure.md`.
- Structure map truthfully records 23 streaming files, spawn.rs isolation duty, the new roster, interrupt limitations 6 items, v0.11.1.
- No real-Google claims; no secrets.
- README test-count drift (387→389) correctly left for 4.4 scope.

### (viii) Gate evidence
- Full gate `logs/checks/20261025T034206Z.log`: 389 passed / 0 failed, `ALL CHECKS PASSED`.
- Targeted re-runs confirmed:
  - `cargo test --locked -- isolation` → 2 passed.
  - `cargo test --locked streaming` → 54 passed.
  - `cargo test --locked interrupt` → 3 passed.
  - `cargo clippy --locked -- -D warnings` → exit 0.
- The 6 pre-existing test-profile warnings are unchanged.
- `.agent/STOP` marker is absent.

## After the Fix

The implementer should apply only the R1 rename, run the verification commands above, and report back. No other source changes are authorized.
