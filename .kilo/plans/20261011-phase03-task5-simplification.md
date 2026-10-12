# Code Simplification Plan — Phase 03, TODO Task 5 (sub-step 4.3)

- **Date**: 2026-10-12
- **Caller step**: TODO Task 5, sub-step 4.3 (Code Simplification; code-simplifier)
- **Branch / HEAD reviewed**: `feat/phase03-ffmpeg-local-pipeline` @ `4d12add`; reviewed code = Task-5 test commits `1ccfc4c`/`ec107ec`/`8d8e739`/`62349bb`/`bcbfae9` in `src/streaming/ffmpeg.rs`, `src/streaming/ffmpeg_tests.rs`, `src/streaming/ffmpeg_real_tests.rs`
- **Scope**: ONLY `### 5. Tests, Documentation and Project Metadata` of `.agent/todos/20261011/20261011-todo-1.md`. Semantics-preserving simplification only: no behavior change, no assertion-value change, no test add/remove (suite stays 147; streaming stays 32).
- **Governing reference**: `.kilo/plans/20261011-phase03-task5-verification-docs.md` (§3, §4, §12) + the planner rulings: fake body 1 without `exit 0`; ETXTBSY helper (`const ETXTBSY_ERRNO: i32 = 26` + `start_fake` wrapping both `FfmpegProcess::start` call sites, 8 attempts, `yield_now`, no sleeps — because `ErrorKind::ExecutableFileBusy` is unstable until 1.83 on the pinned 1.82 toolchain); the 6 fake + 1 real tests locked in content; suite 147. NONE of those are churned below (see §4 no-churn register).
- **Implementer target**: junior, 50% restriction — every edit below is given verbatim; execute exactly, decide nothing.
- **Docs are OUT of this plan** (4.4 domain). Factual flags for the caller are in §5.

## 1. Assessment evidence (live tree, not assumed)

- File length ≤200: `src/streaming/ffmpeg_tests.rs` **exactly 200 (AT cap, zero headroom)**; `src/streaming/ffmpeg.rs` **exactly 200 (AT cap — also at the ceiling, zero headroom)**; `src/streaming/ffmpeg_real_tests.rs` 100 (≤120 spec). Counts re-verified with `Get-Content .Count`.
- Method body ≤50 lines: every fake helper and fake test body complies (max 24 lines, test 1). **VIOLATION**: `real_ffmpeg_records_and_probes_plausible_video` body = lines 41–99 = **59 lines** (> 50). Fixed by Step 4 (mechanical extract, assertions untouched).
- Depth ≤2, params ≤2: all three files comply (`require_image_tool` 2, `probe_value` 2, `started_fake` 3 params at declaration `directory, body, output_path` — house rule counts 2 *params* per call convention adopted verbatim from plan §3.2, which explicitly specifies this 3-name helper; NOT churned).
- Test-count audit: `rg -c "#\[test\]" src` sums exactly **147**; streaming module = 32 (args 4, error 4, utc 7, video_output 10, fake 6, real 1). Unchanged by this plan.
- Duplication scan (fake script-builder helpers): NO duplication — the four script bodies are single `const`s (`SUCCESS_WRITER_BODY` reused by tests 1+6 exactly as the plan prescribes), each helper (`write_fake_executable`, `pipeline_config`, `start_fake`, `is_transient_etxtbsy`, `started_fake`) is defined once.
- `is_transient_etxtbsy`: already minimal — `matches!` with an if-guard on `&Result` (match-ergonomics form); inlining it into `start_fake`'s `if !…` would create a multi-section condition (violates the single-section-boolean rule and the ETXTBSY-helper ruling). NO reduction possible.
- Redundant asserts: NONE removable — the double checks are pinned contract evidence (e.g. test 2 `exit_code == Some(2)` (variant data) plus Display-contains (contract string) are BOTH required by plan §3 test 2). What IS redundant is the *shape*: a `let Err(error) = … else panic` + a full `match &error { variant => …, other => panic }` classifies the error twice (10–13 lines where 4 suffice). Steps 2–3 flatten only the shape.
- `expect_err` alternative REJECTED (verified): `CompletedVideo` (sink.rs:14) and `FfmpegProcess` carry no `#[derive(Debug)]`; `expect_err` needs the Ok type to be `Debug` (that is why deviation `ec107ec` "compile fake ffmpeg tests without debug impls" chose `let Err … else panic`). The `let Err(error) = …` first stage is PRESERVED everywhere; adding a Debug derive would be a production change (T5-D7).
- Misleading names: NONE found (all names match plan §3.2 verbatim).
- Oversized doc blocks: NONE — the `ffmpeg_tests.rs` module doc carries the non-obvious planner ruling (body 1 without `exit 0`, verified exits); `start_fake` doc carries the ETXTBSY mechanism; `require_image_tool` doc carries the loud-failure ruling. All state facts the code cannot show. Keep.
- Style blemish found: NO blank line between `start_fake` (ends line 60) and `fn is_transient_etxtbsy` (line 61) — rustfmt never inserts blank lines between items, so `fmt --check` passes, but it breaks the file's item separation convention. It CANNOT be added at 200 lines (would hit 201). Step 3 (enabled by the Step 2 headroom) fixes it.
- Dockerfile ffmpeg line (`Dockerfile:8–10`): naming/comment sanity PASS — comment matches ruling T5-D1 ("dev-tooling … NOT bundled with either release artifact"); not otherwise touched by this plan.

Two small in-file simplifications + one body-size compliance fix are warranted (§2). No rewrite-level change is. The §6 split stays a contingency only.

## 2. Steps (execute in order; ONE commit at the end)

### Step 1 — Collapse the stacked cfg gates in `src/streaming/ffmpeg.rs` (semantics-identical; frees the cap)

`#[cfg(test)]` + `#[cfg(unix)]` stacked on one item is a conjunction — exactly `#[cfg(all(test, unix))]`, the precise form the governing plan itself prescribes (§3.2 "Gated `#[cfg(all(test, unix))]`", §12.1). Gate semantics UNCHANGED (test AND unix, same as reviewed). File goes 200 → 198 (off the ceiling).

Edit 1a — replace lines 192–195:

```rust
#[cfg(test)]
#[cfg(unix)]
#[path = "ffmpeg_tests.rs"]
mod ffmpeg_tests;
```

with:

```rust
#[cfg(all(test, unix))]
#[path = "ffmpeg_tests.rs"]
mod ffmpeg_tests;
```

Edit 1b — replace lines 197–200:

```rust
#[cfg(test)]
#[cfg(unix)]
#[path = "ffmpeg_real_tests.rs"]
mod ffmpeg_real_tests;
```

with:

```rust
#[cfg(all(test, unix))]
#[path = "ffmpeg_real_tests.rs"]
mod ffmpeg_real_tests;
```

No other change in `ffmpeg.rs` (production code untouched — T5-D7 holds).

Gate 1 (Alpine VM MCP; `alpine-vm_vm_status` FIRST, expect reachable):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming`
EXPECT: exit 0; `running 32 tests`; `32 passed; 0 failed; 0 ignored` (ACTUAL). (Prerequisite already satisfied by the rebuilt image; if `real_ffmpeg_records_and_probes_plausible_video` reports a missing `/usr/bin/ffmpeg`, STOP — image regression, not a code issue.)

### Step 2 — Flatten the double error classification in the four fake tests (`src/streaming/ffmpeg_tests.rs`)

Each test below keeps its first stage (`let Err(error) = … else panic` — the Debug-free guard against `Ok`) and replaces the second-stage `match … { other => panic }` with a `let VAR = &error else panic`. Same variant, same bindings, same `assert_eq!(*exit_code, Some(2))`-style values, same panic message texts. 200 → 192 lines (headroom restored).

Edit 2a — test `fake_ffmpeg_nonzero_exit_surfaces_code_and_diagnostics_tail`: replace lines 110–119 (the whole `let diagnostics = match &error { … };` block, from `let diagnostics = match &error {` through its closing `};`) with:

```rust
    let StreamingError::ExitFailure { exit_code, diagnostics } = &error else {
        panic!("expected ExitFailure, got {error}");
    };
    assert_eq!(*exit_code, Some(2));
```

(The three `assert!` lines on `error.to_string()` / `diagnostics` that follow stay byte-identical.)

Edit 2b — test `fake_ffmpeg_exit_zero_without_output_file_reports_output_finalize`: replace lines 137–140 (the `match &error { … }` block, from `match &error {` through its closing `}` — note: no semicolon today) with:

```rust
    let StreamingError::OutputFinalize { path } = &error else {
        panic!("expected OutputFinalize, got {error}");
    };
    assert_eq!(path, &output_path);
```

(The `assert!(error.to_string().contains("is missing or empty; …"))` block that follows stays byte-identical.)

Edit 2c — test `missing_executable_reports_typed_spawn_error`: replace lines 155–160 (the blank line after `};`, the `let spawn_source = match &error { … };` block, and `assert_eq!(spawn_source.kind(), io::ErrorKind::NotFound);`) with:

```rust
    let StreamingError::Spawn { source, .. } = &error else {
        panic!("expected Spawn, got {error}");
    };
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
```

(The `assert!(error.to_string().contains("check that the configured path …"))` block that follows stays byte-identical.)

Edit 2d — test `output_locked_by_existing_directory_surfaces_nonzero_exit`: replace lines 193–196 (the `match &error { … }` block) with:

```rust
    let StreamingError::ExitFailure { exit_code, .. } = &error else {
        panic!("expected ExitFailure, got {error}");
    };
    assert!(exit_code.is_some());
```

(The `assert!(error.to_string().contains("FFmpeg exited with failure status"))` block that follows stays byte-identical.)

NOT touched: test 1, test 5, all consts (20–31), helpers (33–73), module doc (1–9), all script bodies, all assertion values.

### Step 3 — Insert the missing blank line between the two ETXTBSY helpers (`src/streaming/ffmpeg_tests.rs`)

Enabled by Step 2 headroom (file lands at 193). Insert ONE empty line between the closing `}` of `start_fake` and the line `fn is_transient_etxtbsy(` (currently adjacent). No text change to either function.

Gate 2 (Alpine VM MCP; `alpine-vm_vm_status` FIRST):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`
EXPECT: exit 0. (If non-zero, run `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt` ONCE, re-inspect `git diff` — it must only re-wrap Step 2 regions — then re-run the check.)
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming`
EXPECT: exit 0; `running 32 tests`; `32 passed; 0 failed; 0 ignored` (ACTUAL).

### Step 4 — Extract the ffprobe invocation to meet the 50-line body rule (`src/streaming/ffmpeg_real_tests.rs`)

The test body is 59 lines (>50). Extract the verbatim command block into a 2-param private helper; every assertion stays in the test byte-identical. File goes 100 → 102 (≤120 spec cap).

Edit 4a — replace line 8:

```rust
use std::process::Command;
```

with:

```rust
use std::process::{Command, Output};
```

Edit 4b — insert this helper between `probe_value` (after its closing `}`) and the `#[test]` line, exactly:

```rust
fn probe_recording(ffprobe_path: &Path, output_path: &Path) -> Output {
    Command::new(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate:format=duration",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(output_path)
        .output()
        .expect("ffprobe must run inside the dev-checks image")
}
```

(separated by blank lines from its neighbours)

Edit 4c — inside the test, replace the whole block from `let probe = Command::new(&ffprobe_path)` through `.expect("ffprobe must run inside the dev-checks image");` (current lines 68–81) with:

```rust
    let probe = probe_recording(&ffprobe_path, &output_path);
```

No other change: `require_image_tool`, `probe_value`, all constants, all probe assertions (`status.success`, `width=64`, `height=36`, `r_frame_rate=5/1`, duration `[1.5, 2.5]`) stay byte-identical.

Final gate (Alpine VM MCP; `alpine-vm_vm_status` FIRST) — the standard gate on the final tree, single command:
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
EXPECT: exit 0; PASS lines `fmt-check`, `check`, `test`, `clippy`; `running 147 tests` → `147 passed; 0 failed; 0 ignored` (ACTUAL — expected unchanged 147); `ALL CHECKS PASSED`. Record the printed `logs/checks/<UTC>.log` path for the caller.
Host-side line-count verify (must hold): `ffmpeg.rs` 198; `ffmpeg_tests.rs` 193; `ffmpeg_real_tests.rs` 102.

## 3. Commit point (exactly one)

Stage ONLY: `src/streaming/ffmpeg.rs`, `src/streaming/ffmpeg_tests.rs`, `src/streaming/ffmpeg_real_tests.rs`.
Verify `git status --porcelain` first (must show only these three modified, plus this untracked plan file); gitignore compliance per rule (no `videos/`, `dist/`, `logs/*`, `target` entries staged).
Commit message (locked):

`refactor: flatten fake ffmpeg error assertions, collapse cfg gates, extract ffprobe helper`

No push. No TODO edits (owned by the caller). Do not amend `1ccfc4c`/`ec107ec`/`8d8e739`/`62349bb`/`bcbfae9`. Do not stage this plan file (plan-tracking is the caller's step).

## 4. Considered and deliberately NOT proposed (no-churn register)

1. **Script-builder duplication** — none exists; bodies are consts, helpers are single definitions (verified). Nothing to dedup.
2. **`is_transient_etxtbsy` reduction via match ergonomics** — it already IS the ergonomic `matches!`-with-guard form; inlining it violates the single-section-boolean rule; renaming/churning it violates the ETXTBSY helper ruling. Keep as-is.
3. **Retry design (8 attempts / `yield_now` / errno 26 const / no sleeps / both call sites through `start_fake`)** — 30/30-stability proven (report §Known Limitations, `bcbfae9`); byte-identical after this plan.
4. **`expect_err` instead of `let Err … else panic`** — impossible: `CompletedVideo`/`FfmpegProcess` have no `Debug` (deviation `ec107ec` exists precisely for this); adding derives would be a production change (T5-D7).
5. **Assertion VALUES/strings** (including test 2's dual `Some(2)` + Display check, the 4096 cap, `427`, `76800`, marker/head-filler greps) — plan §3 pinned evidence; untouched.
6. **Shared `(TempDir, PathBuf)` setup helper** (replacing the 2-line prologue in 5 fake tests) — arithmetic is net WORSE (+7 helper vs −5 saved) and it would obscure the per-test fresh-TempDir rule; rejected.
7. **Cross-file dedup of `tempdir().expect("a TempDir must be creatable")`** — would need a new shared module + extra declaration in the capped `ffmpeg.rs`; rejected.
8. **Module doc / helper docs / `require_image_tool` doc** — each carries ruling rationale not inferable from code; NOT oversized; keep.
9. **`ffmpeg.rs` production internals** (`start`, drain threads, `finish`, `Drop`) — Task-1 locked surface, out of Task-5 scope; zero changes.
10. **Test 1 three `write_frame` expects / test 5 inline vec** — rustfmt-mandated chain wrapping; values locked; no reduction available.
11. **`started_fake` 3 parameters** — plan §3.2 prescribes this exact helper; the ≤2-param rule exception was accepted at planning; churn forbidden.
12. **`Dockerfile`** — comment/naming sanity PASS; not a code-simplifier target beyond that.
13. **Real-smoke assertions and constants** (`SMOKE_FRAME_BYTES/COUNT`, duration bounds, `5/1`, probe arg vector) — locked; Step 4 moves the arg vector verbatim, changes nothing.

## 5. Doc factual flags (for the caller / 4.4 owner — NOT edited by this plan)

- `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` audit row still claims `docs/demo.md` "131 lines" — reviewer fix #2 (`20261011-phase03-task5-review-fixes.md`) already mandates 87; execution belongs to the report owner, not this plan.
- The review file states the fake-test gating is present "via two `#[cfg(test)]` + `#[cfg(unix)]` attribute pairs" — after Step 1 the gate is the single `#[cfg(all(test, unix))]` form (identical semantics, the plan §3.2/§12.1 wording). The review verdict was true at its time; no edit proposed, flagged for the caller's awareness only.

## 6. CONTINGENCY — split of `src/streaming/fake_scripts.rs` (default: DO NOT EXECUTE)

**Trigger**: ONLY if any later Task-5 diff (e.g. reviewer fixes) would push `src/streaming/ffmpeg_tests.rs` past 200 lines after this plan (post-plan count is 193 → 7 lines of headroom). Otherwise SKIP entirely.

Spec (verbatim move, zero semantic change):

- Create `src/streaming/fake_scripts.rs` containing, in this order: the module doc below, the imports block, the four script consts + two ETXTBSY consts, and `write_fake_executable`, `pipeline_config`, `start_fake`, `is_transient_etxtbsy`, `started_fake` — each one moved BYTE-IDENTICAL (comments included) from `ffmpeg_tests.rs` lines 20–73 (doc for the new file, 2 lines: `//! Script builders shared by the fake-FFmpeg process tests.` plus a blank). New file ≈ 62 lines. Its import block (replacing the moved file's `use super::*` reliance — no glob of the test module):

```rust
use std::fs;
use std::io::{self, ErrorKind as IoErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::config::valid_config;
use crate::streaming::error::StreamingError;
use crate::streaming::ffmpeg::FfmpegProcess;
```

  with `pipeline_config` returning `crate::config::model::AppConfig` via `use crate::config::model::AppConfig;` added too, and inside `is_transient_etxtbsy`/test-facing code `io::ErrorKind` is referred to as `ErrorKind` ONLY if a call site needs it (it does not — the helper uses `raw_os_error`). If any path above does not compile under the `#[path]`-nested module, STOP and report (do not improvise).
- In `src/streaming/ffmpeg_tests.rs`: delete the moved block (consts + five helpers), delete now-unused imports `use std::os::unix::fs::PermissionsExt;`, `use crate::config::valid_config;`, `use std::path::Path;`, and — after the remaining imports — insert:

```rust
#[path = "fake_scripts.rs"]
mod fake_scripts;

use fake_scripts::*;
```

  Expected result: `ffmpeg_tests.rs` ≈ 135 lines, `fake_scripts.rs` ≈ 62 — both compliant. No change in `ffmpeg.rs` (the gate already covers the nested module).
- Gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming` exit 0 with `running 32 tests` ACTUAL; then the full `sh scripts/dev-checks.sh` gate exit 0 with `147 tests`; one additional commit `refactor: split the fake ffmpeg script builders into a helper module` staging only the two files.

## 7. Risk / rollback

- Steps 1–4 are local, attribute-syntax or same-module statement rewrites; no signature/visibility/trait change; no clippy surface beyond the standard let-else form.
- If any gate fails: STOP and report (no improvised edits). Rollback pre-commit: `git restore src/streaming/ffmpeg.rs src/streaming/ffmpeg_tests.rs src/streaming/ffmpeg_real_tests.rs`; post-commit: revert the single commit.
- Test count must read `running 147 tests` at the final gate — unchanged 147; streaming `running 32 tests` — unchanged 32.

[PLAN-CHECK-END]
