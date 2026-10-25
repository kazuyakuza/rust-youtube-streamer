# Task 1 Simplification Plan — Stop-Path Isolation (4.3 simplifier → 4.2 junior)

Date: 2026-10-25 · Author: Simplifier (4.3) · Scope: the **Task-1-changed files only**
(`spawn.rs`, `ffmpeg.rs`, `ffmpeg_isolation_tests.rs`, `console_tests.rs`,
`ffmpeg_real_audio_tests.rs`, `ffmpeg_real_tests.rs`, `interrupt.rs`).
Branch `fix/stop-path-and-stream-health`, HEAD `0cc59df`. Binding plan:
`.kilo/plans/20261025-stop-path-task1-implementation.md`. Parent global plan unchanged.

## §0 Caller rulings that are BINDING — do NOT undo

- The test-1 **direct-pid** redesign + honest-doc (§113–122 block, `/bin/kill -s INT <pid>`).
- Removal of the sender's `process_group(0)` (the sender no longer carries it).
- The 7 recorded deviations; the docs-commit derivation.
- GUARDRAILS: never touch the pinned vectors/tests, demo exit codes, the exit mapping,
  `RUN_STOP_MESSAGE`, diagnostics constants, **auth/YouTube surfaces**, Cargo files, `.gitignore`.
  No new `#[allow(`. Offline tests only.

## §1 Verdict: exactly ONE applied simplification (single atomic step)

`src/streaming/ffmpeg_isolation_tests.rs` — the two `#[test]` functions end with a
**byte-identical** 7-line "finalize at stdin EOF + assert clean Stream exit 0" epilogue.
The sibling roster `ffmpeg_stream_tests.rs` does **not** fold its epilogues because each
occurrence differs (distinct `expect`/`panic` text, extra assertions); here the two copies are
identical, so the fold is genuine duplication removal, explicitly within the binding plan's
"test-shape latitude" (S4 / §5 "repetitive assert blocks foldable into small file-local helpers").
Zero behavior change; both test names, both assertion outcomes, the roster count (2 tests),
and the ≤200-line cap are preserved (the file moves 197 → ~193 lines).

### Step S1 — fold the duplicated finalization epilogue into a file-local helper

All edits are in `src/streaming/ffmpeg_isolation_tests.rs`. Do them in this order.

**S1.1 — Add the helper.** Insert the following function immediately **after** the closing
brace of `fn wait_for_handler_flag(...)` (currently ends at the `}` on line 111) and **before**
the `/// Honest proof structure ...` doc comment. (Anchors are unique; keep one blank line on
each side of the new fn.)

```rust
/// Shared epilogue of both isolation tests: finalize the isolated fake at stdin
/// EOF and pin the clean Stream exit 0.
fn assert_finalizes_at_stdin_eof_with_success(process: FfmpegProcess) {
    let completed = process
        .finish()
        .expect("stdin EOF must finalize the isolated child");
    let CompletedOutput::Stream(stream_exit) = completed else {
        panic!("the isolated fake must complete as a Stream carrier");
    };
    assert_eq!(stream_exit.exit_code(), Some(0));
}
```

**S1.2 — Replace BOTH epilogue copies with the call.** The following 7-line block appears
**twice** (once at the end of `stream_child_survives_parent_group_sigint_and_finalizes_on_stdin_eof`,
once at the end of `stream_child_leads_its_own_process_group`). Replace **each** occurrence
verbatim with the single call line:

OLD (exact, both copies):
```rust
    let completed = process
        .finish()
        .expect("stdin EOF must finalize the isolated child");
    let CompletedOutput::Stream(stream_exit) = completed else {
        panic!("the isolated fake must complete as a Stream carrier");
    };
    assert_eq!(stream_exit.exit_code(), Some(0));
```
NEW (both copies):
```rust
    assert_finalizes_at_stdin_eof_with_success(process);
```

Use an `Edit` with `replaceAll` on the OLD block → NEW line (both occurrences are identical and
both become the same call). Do NOT reformat anything else; the preceding lines of each test
(survival loop / `assert_eq!(child_group, ...)` respectively) are unchanged.

### Why S1 is behavior-neutral (do not re-check these, they cannot change)

- `process` in test 1 is `let mut process` (the `mut` stays justified by the earlier
  `process.child.try_wait()`); passing it by value to the helper moves it — legal after the loop.
- `process` in test 2 is `let process` (non-`mut`); `FfmpegProcess::finish(self)` already consumed
  it there today, so the by-value helper param matches exactly. No `mut` change needed.
- `FfmpegProcess`, `CompletedOutput`, and the `FrameSink::finish` method all resolve via the file's
  existing `use super::*` — no import is added or removed, so **no new unused-import / dead-code
  warning** can appear.

## §2 Verification (junior must run, in the Compose `rust` service on the Alpine VM)

Exact prefix: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`.
One filter per test command. STOP-AND-REPORT if any count/warning differs from these expected.

1. `cargo fmt --check` → clean (if it fails, run `cargo fmt` then re-check; do not hand-wrap lines).
2. `cargo test --locked -- isolation` → `2 passed; 0 failed`. Build must show **only** the
   6 pre-existing warnings, all out-of-scope (`app/run/gate.rs` unused import, and the four
   `youtube/*` dead-code + `token.rs` field). **Any NEW warning = STOP.**
3. `cargo clippy --locked -- -D warnings` → clean (this is the binding plan's clippy form:
   no `--all-targets`, so the test-profile-only warnings are not linted here).
4. Authoritative full gate: `sh scripts/dev-checks.sh` → exit 0, `ALL CHECKS PASSED`, test total
   **389** (387 filtered in step 2 + the rest = unchanged). Record the new `logs/checks/<UTC>.log` path.
5. Hygiene: `git status --porcelain` → **only** `src/streaming/ffmpeg_isolation_tests.rs` modified.
   `git check-ignore -v logs/checks/*` still gitignored. No `.gitignore`/Cargo/other-file churn.

Expected arithmetic after S1: test total stays **389/0** (fold removes no test); isolation filter
still matches **2**; roster line count ~193 (≤ 200 cap intact).

## §3 Commits (this simplification cycle)

- C1 `refactor(test): fold the duplicated isolation-finalize epilogue into a helper`
  — S1 only (single file). Do NOT amend the prior `aedfaa8`/`0cc59df` commits.

## §4 Deferred / NOT-applied candidates (report-only; deliberately left untouched this cycle)

- **D-1 — Redundant `#[cfg(unix)]` on `use std::os::unix::process::CommandExt;` in
  `ffmpeg_real_tests.rs` and `ffmpeg_real_audio_tests.rs`.** Both modules are declared
  `#[cfg(all(test, unix))]` in `ffmpeg.rs`, so the inner `#[cfg(unix)]` gate is provably redundant
  here (it is only genuinely required in `console_tests.rs`, whose module is `#[cfg(test)]`-only and
  compiles on Windows). Kept anyway: harmless, self-documents the unix-only API, matches the
  binding plan's D10 "import gated per-file" shape, and is consistent across the four D10 sites.
  No behavior value; removing it re-triggers the whole gate for cosmetics → not applied.
- **D-2 — Chain-splits at the 3 streaming D10 sites** (`probe_recording`, `spawn_audio_smoke`,
  `probe_audio_smoke`): since those `command.process_group(0)` calls are NOT `#[cfg]`-gated and
  `CommandExt::process_group` returns `&mut Command`, they *could* be re-inlined into the original
  single expression (no `let mut`). NOT applied: (a) reverting breaks the uniform D10 shape shared
  with the cross-platform `console_tests` site, (b) `console_tests` genuinely requires the split
  (a `#[cfg(unix)]` statement cannot sit mid-chain), so unifying on the split is the deliberate
  choice, (c) it touches pinned collateral-immunity sites and needs a Windows cross-build re-verify.
- **D-3 — Duplicated setup across the two isolation tests** (`tempdir → write_fake_executable →
  stream_config → start_stream_fake`). Foldable into `spawn_isolated_fake_child(&directory)`,
  but weaker than S1: it moves `FIXTURE_URL` and the `.expect("the fake must spawn")` off the test
  line and the roster already expresses this as a clear 3-call trio. Not applied (marginal, and test
  bodies stay self-contained/readable). Candidate if a future edit already touches these tests.
- **D-4 — `spawn.rs` two `isolate_from_console_control` fns + two consts.** These are
  `#[cfg(target_family="windows")]` / `#[cfg(target_family="unix")]` **alternatives** (exactly one
  arm compiles per target) — the canonical Rust platform-shim pattern. Not duplicate code; cannot
  be merged into one runtime-dispatched helper. Kept.
- **D-5 — The 6 pre-existing test-profile warnings** (`app/run/gate.rs`, `youtube/*`, `youtube/token.rs`):
  out of Task-1 scope and on GUARDRAIL-protected surfaces (auth/YouTube) → not touched, not owned by
  this cycle.
- **Committed-doc wording (`interrupt.rs` item 6):** VERBATIM from binding-plan D11 and accurate;
  read-only review found no tightening worth making. Report-only per instructions — **not edited**.

## §5 Binding-contract reconfirmation (what S1 does NOT touch)

Test names, test count, argv vectors, error/exit mappings, demo exit codes, `RUN_STOP_MESSAGE`,
diagnostics constants, the §5 stability matrix, the roster's two constants/helper trio, the sender
redesign, and all four D10 collateral sites remain byte-stable. S1 changes only the *internal shape*
of the roster's shared epilogue.
