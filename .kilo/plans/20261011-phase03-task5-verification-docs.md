# Implementation Plan — Phase 03, TODO Task 5: Tests, Documentation and Project Metadata (sub-step 4.1b)

- **Date**: 2026-10-12
- **Caller step**: TODO Task 5, sub-step 4.1b (Analysis & Planning; architector). This plan governs sub-steps 4.2 (implementer), 4.3 (review), 4.4 (docs-specialist), 4.5b (adherence), 4.6 (completion).
- **TODO source**: `.agent/todos/20261011/20261011-todo-1.md`, section `### 5. Tests, Documentation and Project Metadata` (lines 70–83), the standing constraints (lines 15–28), `## Constraints and Out of Scope` (lines 85–96), and ALL `## Acceptance Criteria` (lines 98–111 — this task closes them with evidence).
- **Global plan**: `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md` (D1–D12; refined here, never reversed).
- **Prior-phase precedent**: `.agent/reports/20261010-phase-02-renderer-chat-store.md` (report structure + `pending` row convention) and `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` (FRESH vs RECORDED-CITED evidence convention).
- **Branch**: `feat/phase03-ffmpeg-local-pipeline` — ALREADY CHECKED OUT; HEAD `fe165c3` at planning time; working tree clean. NO branch create/switch, NO push (step 5 of the Critical Workflow owns merging/push).
- **Version**: `0.6.0` (bumped at phase step 3) — NO bump in this task.
- **Canonical verification environment**: Alpine VM MCP (`alpine-vm_vm_status` FIRST before every `alpine-vm_vm_run_command`), standard pattern:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`; exit status is the authoritative success evidence (README policy).
- **Implementer profile**: junior under 50% restriction. Every structural/architectural/scope decision below is already made; do not redesign, rename APIs, or touch unlisted files.

---

## 0. Verified live baseline (read from the tree/plans at planning time, not assumed)

- Suite: **140 tests green** (per grep of `#[test]`: chat 16, config 16, streaming 25, app 37 (error 2, logging 7, cli 15, mod 6, modes 5), pipeline 13, demo 14, renderer 19); gate log `logs/checks/20261012T030755Z.log` shows `140 passed; 0 failed`, `ALL CHECKS PASSED`.
- Branch top chain: `fe165c3` (chore: mark task 4 done) → `3e95be4` (task-4 adherence) → `97e384a` (task-4 plan) → `5b8c150` (run boundary).
- `src/streaming/` = 10 real files (args, error, ffmpeg, sink, utc_timestamp, video_output + 4 test files); `.gitkeep` already removed (Task 2). `src/app/pipeline/` = 12 files, `src/app/demo/` = 6 files — all wired; zero `#[allow(dead_code)]` staging remains (retired in Task 3 commit `eaa12dc`).
- `src/app/demo/mod.rs`: `run_demo(config) -> u8` — exit 0 success with `println!("demo recording finalized: {}", path)`, exit 1 on `DemoError { Interrupt, Pipeline }` (one stderr `error:` line); demo never exits 3. Demo timeline (Task 3 contract, authority `src/app/demo/schedule.rs`): 28 messages (`DEMO_MESSAGE_TOTAL`), lead-in 30 frames, gap 15 frames, tail 60 frames, `final_frame_index() == 495`, `rendered_frame_count() == 496` (≈16.5 s at the example 30 FPS); message ids `demo-NN`; long-line message index 15; FIFO eviction exercised (28 > 26 example capacity).
- `src/streaming/ffmpeg.rs`: `FfmpegProcess::start(config, output_path)` (no shell, piped stdin/stdout/stderr, drain threads joined before `wait`), `write_frame` → `write_all`, `finish()` = single stdin EOF → drain joins → wait → exit-status check (`ExitFailure { exit_code: Option<i32>, diagnostics }`) → non-empty-file check (`OutputFinalize`), `Drop` kills+reaps. `DIAGNOSTIC_CAPTURE_LIMIT_BYTES = 4096` (trailing bytes). No process-level tests exist yet (Task 1 plan §3 explicitly deferred them to Task 5).
- `src/streaming/video_output.rs`: `resolve_output_path(kind)` = beside-exe `videos/`, created on demand, collision-safe `demo-YYYYMMDD-HHMMSS.mp4` (suffix `_2.._1000`); conservative existence probe (`unwrap_or(true)`).
- `src/config/test_fixtures.rs`: `pub(crate) fn valid_config() -> AppConfig` (1920×1080@30; ffmpeg.executable `"ffmpeg"`); test-reachable by all modules.
- `Dockerfile` (7 lines): `FROM rust:1.82`, rustfmt+clippy, `gcc-mingw-w64-x86-64`, windows-gnu target. **No ffmpeg in the image** — verified.
- `docs/`: configuration.md (117 lines), build.md, build-windows.md, two agent how-tos. `docs/configuration.md` line 94 usage block still says `auth|run` (stale — demo CLI landed in Task 3 with docs deferred to Task 5 per its plan §1 scope list); line 76 says "the renderer is wired into the runtime in a later phase" (stale); line 100/99 describe `auth`/`run` only (no demo row); line 103 exit-summary has no demo mapping. README lines 52/72 say "this phase neither installs nor launches it" — stale after Task 2/3 wiring (the demo launches FFmpeg).
- README `Key Technologies` still lists Tokio as the current async runtime for FFmpeg process management — STALE: the shipped pipeline is synchronous with std threads (Task-2 plan S2). Docs must be corrected in Task 4.4 (D5 below).
- `.agent/project-structure.md`: `streaming/` still says "(placeholder; future phase)" (line 35 — STALE), `app/` has no `demo/` or `pipeline/` entries, root Cargo.toml says v0.5.0 (stale, actually 0.6.0 with ctrlc), `logs/` says "only .gitkeep placeholder versioned" (currently FALSE: `logs/.gitkeep` is untracked), `youtube/` placeholder row lacks the explicit deliberate `.gitkeep` statement.
- `.gitignore` line 21: `!logs/.gitkeep` negation exists; **the file itself is absent and untracked** (verified: `ls logs` → only `checks/`; `git ls-files "*.gitkeep"` → only `.kilo/plans/.gitkeep`, `credentials/.gitkeep`, `src/youtube/.gitkeep`).
- Stale doc-comment citations in `src/` (grep-verified, 9 hits in 9 files): 6 × `.agent/todos/20261010/20261010-todo-2.md` (that file was renamed `-DONE.md` — citation dead): `src/chat/mod.rs:8`, `src/chat/error.rs:5`, `src/chat/model.rs:5`, `src/chat/store.rs:7`, `src/renderer/mod.rs:4`, `src/renderer/error.rs:6`; 3 × `.agent/todos/20261011/20261011-todo-1.md` (still correctly named NOW, but gets renamed `-DONE.md` at workflow step 5 later this phase): `src/streaming/mod.rs:2`, `src/streaming/error.rs:4`, `src/app/pipeline/mod.rs:8`.
- `features/limits` facts for docs: Ctrl+C limitations live in `src/app/pipeline/interrupt.rs` module docs (S7 5-point list); demo `Error` mapping in `src/app/demo/mod.rs` (Task-3 plan T3).
- Reported evidence from Task 4 (CITED, not re-run): run-mode boundary VM smoke (adherence report `20261011-phase03-task4-adherence.md` §4: verbatim `run` error line + `run_exit_code=3`).

---

## 1. Task-level decisions (locked; implementer/4.4 do not re-decide)

- **T5-D1 — Real-FFmpeg smoke: EXTEND the dev-check image (option (a)), accepted.**
  Rationale (recorded verbatim in the docs + completion report): the pinned `rust:1.82` image lacks ffmpeg, but (i) Task-1's plan §0.2 explicitly deferred runtime verifications V1–V3 (natural EOF finalization, `-framerate` acceptance by the rawvideo demuxer, `libx264`+`veryfast`+`b:v`+`yuv420p` acceptance) to Task 5; (ii) brief §10.2 requires the FFmpeg argument vector to be "tested against the selected FFmpeg version"; (iii) the Task-3 plan §11 pinned the demo success print to a Task-5 manual smoke; and (iv) the TODO's evidence rule says test the produced MP4 IS run → must probe dimensions/fps/duration. Choosing option (b) (NOT-RUN) would leave the shipped locked argument vector never exercised by any real FFmpeg anywhere.
  The change is **dev tooling only**: `ffmpeg`+`ffprobe` become part of the dev-checks Rust image (Dockerfile); they are NOT bundled with either release artifact, NOT shipped, and don't change `build-linux.sh`/`build-windows.sh` behavior (both re-run as fresh evidence anyway). The term "bundle" in the constraints means bundling with the application — image tooling is not a bundle.
- **T5-D2 — Fake-FFmpeg tests: POSIX `#!/bin/sh` scripts created at test time in a `tempfile::TempDir`**, gated `#[cfg(all(test, unix))]`. No compiled helper binary, no `sh -c` in production code, no PATH lookup: the fake path is passed as `config.ffmpeg.executable` verbatim. The fake scripts never consult a real ffmpeg (no `ffmpeg` token in any script body); they read the LAST argument as the output path — matching the locked vector (output path is appended last, asserted by `args_tests`).
- **T5-D3 — Housekeeping (three ledger items + one discovered extension)**: restore `logs/.gitkeep`; document `src/youtube/.gitkeep` as deliberate; update ALL 9 stale TODO-citation doc comments in `src/` to the final `-DONE` filenames (the 6 dead `20261010-todo-2.md` refs MUST be fixed; the 3 `20261011-todo-1.md` refs are updated to `20261011-todo-1-DONE.md` because workflow step 5 renames that file right after Task 5 — accepted 1-commit dead-link window, verified at 4.5b).
- **T5-D4 — Docs touch list (4.4 executes)**: README (Current Status, Commands, exit codes, FFmpeg paragraphs, Key Technologies Tokio fix, demo run section + TOC), **new `docs/demo.md`**, `docs/configuration.md` (demo CLI/exit codes/derivative fixes), `.agent/project-structure.md` (full refresh), `.agent/project-info/tech.md` (sync-std decision + ctrlc + dev-image ffmpeg note), `.agent/project-info/context.md` (planner-owned; drafted in 4.4 with facts from the task adherence reports + fresh evidence). `docs/build.md` + `docs/build-windows.md`: NOT touched (verified still accurate — the Dockerfile change does not alter their instructions; the existing "build the image first" prerequisite covers the rebuild).
- **T5-D5 — Completion report**: new tracked file `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (naming precedent: report date = the phase's todo folder date `20261011`, like `20261010-*` reports for todos under `todos/20261010/` — the caller's suggested date 20261010 is corrected to match the actual phase folder). Written in 4.4 with REAL values from the implementer's summaries and fresh runs; `pending` rows follow the phase-02 convention.
- **T5-D6 — No new dependencies**: fake/real tests use std + existing dev-dep `tempfile`. `ctrlc` is already justified (Task 2). No `Cargo.toml`/`Cargo.lock` edits anywhere in this task.
- **T5-D7 — No source features**: zero production-behavior changes. The only `src/` edits are (a) the two new test modules (+ `ffmpeg.rs` test-module declaration line) and (b) the 9 citation doc-comment lines (H3).
- **T5-D8 — Evidence hygiene**: container-only verification; NO native Windows/Linux claims anywhere; real-FFmpeg evidence is recorded as container evidence (observed version; image tooling). MP4 artifacts live in gitignored `videos/` (repo runs) or the Compose named volume (demo smoke beside the debug exe in `/rust-streamer-target`); never committed.

---

## 2. Test-gap audit (TODO §5 bullet list → existing vs NEW coverage)

| TODO Task-5 bullet | Existing coverage (exact tests) | Verdict |
| --- | --- | --- |
| FFmpeg argument construction | `streaming::args_tests` (4): locked vector, defensive pixel format, output `yuv420p`, output path last | covered (Task 1) |
| Output-path resolution beside the executable | `video_output_tests` (10): beside-exe mapping, `create_directory` make/exists-as-file, `next_available` ×4, prefix mapping, format check | covered (Task 1) |
| Timestamp/collision handling | `utc_timestamp_tests` (7 anchors incl. leap) + collision-suffix tests above | covered (Task 1) |
| Process error mapping | `error_tests` (4) pin every Display contract + `source()`; process-level mapping arrives via the NEW fake-process tests | covered — NO new pure-error tests |
| Demo schedule | `demo::schedule_tests` (6) + `source_tests` (6) + `mod_tests` (2) | covered (Task 3) |
| CLI parsing/help | `cli_tests` (15, incl. `usage_text_advertises_the_demo_command`) | covered (Task 1/3) |
| Frame byte counts | `runner_tests` asserts `TEST_FRAME_BYTES` writes + `frame_tests::frame_filled_has_exact_byte_count`; NEW fake test counts actual bytes through the pipe | covered — NEW process-level test #1 |
| Continuous output with no messages | `runner_tests::run_emits_immediate_empty_first_frames_with_exact_byte_counts` | covered (Task 2) |
| Message appearance/eviction | `runner_tests::run_rotates_new_messages_into_visible_text` + `demo::mod_tests` (26-line composition) + `schedule_tests` (28 > 26) | covered (Tasks 2–3) |
| Orderly EOF/finalization | pipeline fakes assert `finish_called`; process-level natural-EOF finalization is NEW test #1 (+ real smoke) | NEW |
| FFmpeg non-zero exit | NEW tests #2/#6 (+ real smoke encodes args) | NEW |
| Missing executable | NEW test #4 | NEW |
| Unwritable output directory | `create_directory_reports_existing_file` covers the dir-create error class; NEW test #6 is the locked-output-target surrogate (a chmod-based test is non-deterministic in a root container — REJECTED, reason recorded) | NEW surrogate |
| Interruption behavior where feasible | `runner_tests` (stop before start / stop after Nth write), `interrupt_tests` (flag clone contract), `demo::source_tests` (stop at final frame). NOT automated: signal delivery itself (ctrlc global handler — documented boundary since Phase 02/01) and `Drop` kill/reap (no observable without the consumed child handle) — record as limitations | covered at the seam + documented limitation |
| Real-FFmpeg smoke (optional) | NEW real test | NEW (T5-D1) |

---

## 3. Fake-FFmpeg process tests (file spec; exact)

### 3.1 Declaration

Append to the bottom of `src/streaming/ffmpeg.rs` (file is 190 lines; +3 lines ⇒ 193 ≤ 200):

```rust
#[cfg(test)]
#[cfg(unix)]
#[path = "ffmpeg_tests.rs"]
mod ffmpeg_tests;
```

Keep the existing `utc_timestamp`-style comment style; no other `ffmpeg.rs` change.

### 3.2 `src/streaming/ffmpeg_tests.rs` (NEW; ≤200 lines; 6 tests)

Module docs (3–5 lines): fake FFmpeg as a `#!/bin/sh` script written into a `tempfile::TempDir` at test time; scripts consume stdin and/or fail deliberately; they never consult a real ffmpeg; the last argument received is the MP4 target (locked vector). Gated `#[cfg(all(test, unix))]` via the declaration above — tests run only in the Linux dev-checks container per project policy (the Windows cross-build never compiles `cfg(test)` code).

Shared private helpers (self-documenting names, ≤2 args):

```rust
fn write_fake_executable(directory: &Path, body: &str) -> PathBuf {
    // file "fake-ffmpeg.sh" in `directory`, content = "#!/bin/sh\n" + body + "\n",
    // permissions 0o755 via std::os::unix::fs::PermissionsExt::from_mode.
}

fn pipeline_config(executable_path: &Path) -> AppConfig {
    // mutable clone of crate::config::test_fixtures::valid_config() with
    // config.ffmpeg.executable = executable_path.to_string_lossy().into_owned();
}

fn started_fake(directory: &Path, body: &str, output_path: PathBuf) -> FfmpegProcess {
    // write_fake_executable + pipeline_config + FfmpegProcess::start — the three
    // shared assertions-unfree setup lines each test would repeat.
}
```

Script bodies (LOCKED; every one starts `#!/bin/sh` and stays under ~6):

1. Success writer (tests 1):
   ```sh
   #!/bin/sh
   while [ "$#" -gt 1 ]; do shift; done
   cat > "$1"
   exit 0
   ```
   (`while [ "$#" -gt 1 ]; do shift; done` is portable to dash: `$1` ends up as the LAST argument — the MP4 output path; `cat > "$1"` consumes the whole stdin to EOF = orderly EOF.)
2. Non-zero-exitter (test 2):
   ```sh
   #!/bin/sh
   awk 'BEGIN{for(i=0;i<200;i++)printf "diagnostic filler line %04d\n",i;print "FFMPEG_FAKE_FATAL_TAIL_MARKER"}' 1>&2
   exit 2
   ```
   (200 numbered lines ≈ 6 KB > `DIAGNOSTIC_CAPTURE_LIMIT_BYTES` — proves the drain thread + tail truncation.)
3. Silent-success-no-output (test 3): `cat > /dev/null; exit 0` — consumes stdin, writes nothing → finalization failure.
4. Immediate exitter (test 5): `exit 0` — exits without reading stdin.
5. Success-writer reuse for test 6 (same body as 1).

Tests (names verbatim; every test uses a fresh `TempDir`):

1. `fake_ffmpeg_success_writes_exactly_the_piped_bytes` — body 1; output path `<TMP>/out.mp4`; `write_frame` three slices of lengths 100, 250 and 77 (arbitrary-ish content, unequal lengths prove concatenation order); `finish()` → `Ok(completed)`; assert `completed.path()` == the passed output path; assert `fs::metadata(path).len() == 427`. This is the orderly-EOF + exact-byte-plumbing + verified-finalization evidence.
2. `fake_ffmpeg_nonzero_exit_surfaces_code_and_diagnostics_tail` — body 2 → `finish()` → `Err(StreamingError::ExitFailure { exit_code: Some(2), diagnostics })`; assert Display contains `"FFmpeg exited with failure status 2"`; assert diagnostics contains `"FFMPEG_FAKE_FATAL_TAIL_MARKER"`; assert diagnostics does NOT contain `"diagnostic filler line 0000"` (tail truncation cut the head); assert `diagnostics.chars().count() <= 4096` (the capture limit; the marker line sits INSIDE the trailing window, so head filler is gone while the marker survives).
3. `fake_ffmpeg_exit_zero_without_output_file_reports_output_finalize` — body 3 → `Err(StreamingError::OutputFinalize { path })` with `path` equal to the passed output path; Display contains `"is missing or empty; the video was not finalized"`.
4. `missing_executable_reports_typed_spawn_error` — `pipeline_config(<TMP>/definitely-missing-ffmpeg)`; `FfmpegProcess::start` → `Err(StreamingError::Spawn { .. })`; Display contains `"check that the configured path is correct and FFmpeg is installed and runnable"`; `source().kind() == ErrorKind::NotFound` (downcast via `source()` → `io::Error` — build a tiny local helper or match on the variant's `source.kind()`).
5. `exited_child_surfaces_a_broken_pipe_frame_write` — body 4; frame = `vec![0u8; 76800]` (256×100×3; larger than the 64 KiB pipe buffer ⇒ write cannot park in the buffer and must fail once the child exits; Rust ignores SIGPIPE so `write_all` returns `ErrorKind::BrokenPipe`); `write_frame(&frame)` → `Err(StreamingError::FrameWrite)`, Display contains `"the FFmpeg process exited while receiving frames"`.
6. `output_locked_by_existing_directory_surfaces_nonzero_exit` — create `<TMP>/locked.mp4` as a DIRECTORY first; body 1 (its `cat > "$1"` fails: "Is a directory" ⇒ sh exits nonzero with the error on its stderr); `finish()` → `Err(StreamingError::ExitFailure { exit_code: Some(_), .. })` (assert `exit_code.is_some()` — do not pin the shell's own code); Display contains `"FFmpeg exited with failure status"`. Surrogate evidence for the unwritable-output-target failure class (dir-create failure class is separately covered by `create_directory_reports_existing_file`).

Rules: no sleeps, no real ffmpeg, no PATH dependence, no `assert!` on wall-clock timing; every leftover child (case 5 and 2) is reaped by `finish()`/`Drop` — the struct goes out of scope normally in tests.

---

## 4. Real-FFmpeg smoke test (file spec; exact)

### 4.1 Dockerfile edit (exact; commit before its test compiles in CI order — see commit plan)

```dockerfile
# FFmpeg is dev-tooling for the container-only checks (fake/real smoke tests and
# manual demo smoke): it is NOT bundled with either release artifact.
RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg
```

Append as a new `RUN` line after line 7 (`rustup target add ...`). Compose header comment (lines 1–3) ordering: insert this `RUN` last. Rebuild once: `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust` (expect exit 0; if apt fails, STOP and report to the caller — no workarounds).

### 4.2 `src/streaming/ffmpeg_real_tests.rs` (NEW; ≤120 lines; 1 test)

Declaration at the bottom of `src/streaming/ffmpeg.rs`:

```rust
#[cfg(test)]
#[cfg(unix)]
#[path = "ffmpeg_real_tests.rs"]
mod ffmpeg_real_tests;
```

Test `real_ffmpeg_records_and_probes_plausible_video` (name verbatim):

- Prerequisite assertions FIRST (evidence rule — fail loudly, never silently skip):
  ```rust
  // The real-FFmpeg smoke is part of the dev-checks suite: the pinned image
  // installs ffmpeg (Dockerfile). An absent ffmpeg binary is a suite
  // failure, never a silent skip (no invented / absent-as-green evidence).
  ```
  `require_image_tool("ffmpeg", Path::new("/usr/bin/ffmpeg"))` and `require_image_tool("ffprobe", Path::new("/usr/bin/ffprobe"))` — ONE tiny private helper (2 args): `fn require_image_tool(label: &str, path: &Path) -> PathBuf` returning the `PathBuf` and panicking with `"{label} is missing at {path}: the pinned dev-checks image must install ffmpeg (Dockerfile decision T5-D1)"`.
- Config: `valid_config()` clone with `video.width = 64`, `video.height = 36`, `video.fps = 5` (tiny, fast; fps 5 → 10 frames ≈ 2.0 s duration).
- Run: `FfmpegProcess::start(config, tempdir.path().join("real-smoke.mp4"))` → 10 × `write_frame(&[0u8; 6912])` (64×36×3 zero/black frames) → `finish()`; assert `Ok(completed)` and `completed.path()` is non-empty on disk (`metadata.len() > 0`).
- Probe: `Command::new("/usr/bin/ffprobe")` with args `["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,r_frame_rate:format=duration", "-of", "default=noprint_wrappers=1", <path>]`, `.output()` (no pipe deadlock — ffprobe prints tiny output; still use `.output()` to collect), assert `status.success()`.
- Assertions on stdout (string contains, no fragile parsing): `width=64`, `height=36`, `r_frame_rate=5/1`, and a parsed `duration=` float in `[1.5, 2.5]` (10 frames at 5 fps = 2.0 s; container timing tolerance). Parse duration by finding the `duration=` line and `str::trim`/parse::<f64> — implement a ≤20-line private helper `fn probe_value(probe_stdout: &str, key: &str) -> Option<&str>` (2 args).
- This single test validates Task-1's deferred V1 (natural stdin-EOF finalization), V2 (`-framerate` accepted by the rawvideo demuxer) and V3 (`libx264`/`veryfast`/`b:v`/`yuv420p` output OK) — record that fact in the report.

**Version evidence (NOT a test; goes in 4.2 evidence + report):** VM command
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ffprobe -version`
and `... run --rm rust ffmpeg -version` — record the observed version line (expected Debian bookworm ffmpeg 5.x/6.x — record ACTUAL, never guess) in the completion report under "Environment Notes" + the test coverage audit row.

---

## 5. Housekeeping steps (exact)

### H1 — Restore `logs/.gitkeep` (ledger item 1)

1. Create the empty file `logs/.gitkeep` (0 bytes; the Write tool with empty content or container `touch logs/.gitkeep` — NOTE: `touch` via container is fine; the file must end up in the repo working tree so git can stage it).
2. Host verify: `git check-ignore -v logs/.gitkeep` → expect `.gitignore:21:!logs/.gitkeep` is matched as a NEGATION (the command exits 0 with the negation line; if it says ignored instead, STOP and report — the negation must win).
3. `git add logs/.gitkeep` → `git status --porcelain` shows exactly `A  logs/.gitkeep`.
4. Commit: `chore: restore the versioned logs folder placeholder`.
5. Post-verify `git ls-files "*.gitkeep"` → now includes `logs/.gitkeep` (4 tracked `.gitkeep` files total: `.kilo/plans/`, `credentials/`, `logs/`, `src/youtube/`).

### H2 — `src/youtube/.gitkeep` stays (ledger item 2) — documentation only

No git action. The deliberate-placeholder statement is encoded in the project-structure refresh (D5 §7.3).

### H3 — TODO-citation doc-comment alignment (ledger item 3 + discovery extension)

Doc-comment ONLY changes; grep-verified exact sites (update the path text, keep the surrounding sentence bytes otherwise):

- `20261010/20261010-todo-2.md` → `20261010/20261010-todo-2-DONE.md`, in:
  `src/chat/mod.rs` (line 8), `src/chat/error.rs` (line 5), `src/chat/model.rs` (line 5), `src/chat/store.rs` (line 7), `src/renderer/mod.rs` (line 4), `src/renderer/error.rs` (line 6).
- `20261011/20261011-todo-1.md` → `20261011/20261011-todo-1-DONE.md`, in:
  `src/streaming/mod.rs` (line 2), `src/streaming/error.rs` (line 4), `src/app/pipeline/mod.rs` (line 8).

Rationale (record in the commit message body): scope-authority citations must point at the final tracked filenames; the `20261011` TODO is renamed with the `-DONE` suffix by workflow step 5 immediately after Task 5, so the citations are written to the final names now. 4.5b MUST verify `.agent/todos/20261011/20261011-todo-1-DONE.md` exists at the end of the phase (step 5 complete); if the rename is ever revoked the citations must revert.

Commit: `docs: update scope authority citations to the completed todo filenames`.

---

## 6. Docs touch list for sub-step 4.4 (docs-specialist; exact)

> All doc file changes in ONE logical area per commit (§8). Read the current files first; preserve every still-true statement; keep README slim (details live in `docs/demo.md`); TOC required for any doc >100 lines; real LF newlines; no invented native-runtime claims.

### 6.1 README.md

| Section | Edit |
| --- | --- |
| Table of Contents | Insert `[Local Demo Recording](#local-demo-recording)` between `Logging (RUST_LOG)` and `Build Checks (Docker via Alpine VM)`. |
| Current Status | Replace stale paragraph(s) with the Phase 03 state: streaming module + app frame pipeline wired via the new `demo` command; `run` clarified boundary (still a placeholder, exit 3); test suite green in the container — write the ACTUAL final-gate test count the gate printed (never a guessed number); directory `videos/` beside the executable for demo recordings. |
| Key Technologies | Fix the Tokio bullet: async runtime is reserved for the future YouTube API/chat streaming phases; the current frame pipeline and FFmpeg supervision are synchronous (`std::process` + std threads + `ctrlc` 3 for Ctrl+C). |
| Commands and Configuration | Add `- demo` bullet: runs a local-only FFmpeg demo recording (no YouTube/OAuth/broadcast/chat); **success 0 after the MP4 is finalized**, printing `demo recording finalized: <path>` on stdout; any demo failure = one `error:` line on stderr + exit 1; demo never exits 3. Update the exit-codes line: add "demo runtime failure = 1 (startup-failure class)". Update the `run` bullet's cross-pointer to the demo ("for a local demonstration use the demo command"). |
| FFmpeg paragraphs (two stale sentences) | Replace with: FFmpeg is an external prerequisite installed separately; the `demo` command launches the executable at the configured `ffmpeg.executable` path and surfaces launch/process failures as typed errors; never bundled. |
| NEW `## Local Demo Recording` | Change to: copy `config.example.json` → `config/config.json`; set `ffmpeg.executable` to the installed FFmpeg path and `renderer.font` to a real font file; run `rust-youtube-streamer-service demo`; results under `videos/` beside the executable with collision-safe `demo-YYYYMMDD-HHMMSS.mp4` names; Ctrl+C semantics + documented limitations (point at `docs/demo.md`); full guide link namely. |

### 6.2 `docs/demo.md` (NEW; target ~120–160 lines; TOC required)

Sections (exact list):
1. `What the demo does` — local-only demonstration recording; the fixed message timeline (28 messages, 30-frame black lead-in, one message at most every 15 frames, 60-frame tail; ≈496 rendered frames ≈16.5 s at the example 30 FPS config; visible capacity 26 at the example geometry so the oldest message is evicted at message 27); message ids `demo-NN` hidden from the screen (`Display` = `author: text`); the long demo-message index 15 exercises horizontal clipping.
2. `Prerequisites` — a working configuration file; **FFmpeg is an external prerequisite**: install separately, set `ffmpeg.executable` to an absolute path (e.g. `/usr/bin/ffmpeg` on Linux; `ffmpeg` on PATH also works); a parseable font file at `renderer.font` (the committed example points at `fonts/console.ttf` whose provenance is indeterminate — prefer a documented font such as the OFL-licensed `fonts/Exo2-Light.otf` used by the test suite).
3. `Running the demo` — exact invocation + example success output (`demo recording finalized: <path>` + stderr tracing events whitelist) + exit codes recap (0 success; 1 any demo failure — one `error:` stderr line; 3 never used by demo).
4. `Output` — `videos/` beside the executable (never the working directory), created on demand, `demo-YYYYMMDD-HHMMSS.mp4` + `_2`-collision suffixes; nothing overwritten; generated MP4s are gitignored and never committed.
5. `Ctrl+C and shutdown` — the current behavior + the 5 documented limitations from `src/app/pipeline/interrupt.rs` (replacing default termination; handler flips the stop flag; shutdown latency ≤ one frame interval; blocked-op limitation with no timeout in this phase — the operator terminates with OS tools; container/foreground vs detached-background caveat).
6. `What the demo does not do` — no YouTube contact/OAuth/broadcast/chat, no stdin messages/keyboard/hotkeys (planned later phase), no screenshots, no audio, no `run`-substitution.
7. `Verification evidence and limits` — verified via the standard container gate with fake-FFmpeg and real-FFmpeg smoke tests + the once-per-phase VM demo smoke; native Windows/Linux runtime runs NOT RUN; FFmpeg arguments verified against the image's FFmpeg version (drop in the observed version after the gate).
8. `See Also` — README / configuration.md links.

### 6.3 `docs/configuration.md`

| Line(s) | Edit |
| --- | --- |
| 92–96 usage block | Add `rust-youtube-streamer-service demo [--config <path>]`; update the alternation line to `auth|demo|run`. |
| 98 bullets | New demo bullet: config validated first; success exits 0 and prints `demo recording finalized: <path>`; failure exits 1 with one `error:` line (`DemoError` delegates the inner cause verbatim: interrupt-install, chat message stage, rendering, frame-size guards, FFmpeg spawn/write/wait/exit/finalization); demo never exits 3 and never contacts YouTube. |
| 99–100 | Keep `auth`/`run` bullets (verify the `run` pointer sentence matches the shipped message). |
| 103 exit-codes summary | Extent: `0 = --help and success(?)` — replace with exact current contract: 0 = help OR successful demo recording; 1 = config/log startup failure AND demo-origin failures; 2 = usage; 3 = `auth`/`run` placeholders only. |
| 76 closing paragraph | Replace "the renderer is wired into the runtime in a later phase" note with the shipped state (wired via demo); replace "this phase neither installs nor launches it" — the demo launches the configured executable. |
| Field reference `ffmpeg.executable` bullet | Add: used at demo start; launch failures surface as `unable to start the FFmpeg executable '<path>': …; check that the configured path is correct and FFmpeg is installed and runnable` (Spawn). |

### 6.4 `.agent/project-structure.md` (full refresh)

- `app/`: add `demo/` (6-file map incl. test files; schedule/source/handler, DemoError → exit-1) and `pipeline/` (12-file map: `clock`/`error`/`interrupt`/`pacing`/`runner`/`source` + `test_helpers` + test files; FramePipeline + prepared_pipeline + StopFlag); update `modes.rs` line (run boundary message now names the four missing YouTube prerequisites + pointer to demo).
- `streaming/`: REPLACE the stale `(placeholder; future phase)` line with the real per-file map of all 10 files + note the sealed boundary rules (std + config only; opaque `&[u8]` frame seam).
- `youtube/`: keep the row; state explicitly that `src/youtube/.gitkeep` is a DELIBERATE tracked placeholder for the future YouTube module (do not delete).
- `logs/`: correct to — versioned placeholder `logs/.gitkeep` (restored this task); all runtime logs (incl. gitignored `logs/checks/<UTC>.log` gate logs, the chat log `logs/chat.log`) are untracked.
- Root files: Cargo.toml v0.6.0 + ctrlc bullet; `src/main.rs` description updated (plain `mod app; mod chat; mod config; mod renderer; mod streaming;` + no allow attributes).
- `dist/`/`videos/`: add a `videos/` note — runtime MP4 output beside the executable (gitignored by rule `videos/`; never in the repo tree beside the repo binary).

### 6.5 `.agent/project-info/tech.md`

- Stack: one bullet — "The frame pipeline and FFmpeg supervision are synchronous (std::process + std threads); Tokio remains reserved for later YouTube API/chat phases (task-2 decision S2)."
- External Processes: add — Ctrl+C via the `ctrlc = "3"` crate (cross-platform flag-flip handler, MIT OR Apache-2.0), the shared StopFlag seam; FFmpeg arg rationale pointer (Task-1 plan §0.2) + "the exact arguments and finalization are verified against the dev-image FFmpeg version by the container smoke tests (recorded version in the phase-03 report)".
- Dependencies: add the ctrlc rationale sentence (`nix`/`windows-sys` platform deps, no async runtime).

### 6.6 `.agent/project-info/context.md` (planner-owned; 4.4 drafts the content blocks)

- `## Current Work Focus`: bounded rewrite of the stale paragraph → Phase 03 status (Tasks 1–5 complete on the branch; verification container-only; leftover NOT-RUN items named).
- New `## Recent Changes (2026-10-12, Phase 03 Tasks 1–4 + Task 5)` bullets: Task 1 streaming module summary; Task 2 pipeline + ctrlc; Task 3 demo command + retirement; Task 4 run boundary + smoke; Task 5 tests (fake ffmpeg + real smoke + counts), dev-image ffmpeg decision (T5-D1), docs/README/structure/tech refresh, `logs/.gitkeep` restore, citation alignment, completion report path + commit references (fill commit hashes from the implementer summaries in 4.6 — `pending` rows like the phase-02 precedent).
- New `## Known Deviations and Limitations (Phase 03)` section, expected bullets: DemoError maps both variants to exit 1 (single startup-failure class; no demo-specific code); Ctrl+C 5-point limitations (S7) + unbounded `finish()` wait with no timeout config field (broken-wait hardening deferred to a later phase — D10 process); unclear-provenance `fonts/console.ttf` note; verification container-only incl. native Windows/Linux NOT RUN; `OutputKind::Session` reserved; audio absent by scope.
- `## Immediate Next Steps`: rewrite (phase 03 closed; next = later phases — OAuth/YouTube module, timestamp logs, streaming-recovery doc per project plan; native Windows validation session to retire that NOT-RUN item).
- Section may grow past ~200 lines — keep entries compact; the file is not a source-code file (limit rule doesn't apply), but trim stale Phase-00 narrative no more than the precedent pattern allows (never delete history blocks; only REWRITE Current Work Focus + ADD new sections).

---

## 7. Completion report spec (`20261011-phase-03-ffmpeg-local-pipeline.md`)

Structure mirrors `.agent/reports/20261010-phase-02-renderer-chat-store.md` (TOC + these sections). Every value comes from captured output or the implementer's recorded facts; `pending` only for commit hashes created after the report text (phase-02 convention — hashes recorded in the caller's/4.6 execution notes, never back-patched):

| Section | Sourced from |
| --- | --- |
| Overview | branch/HEAD/date/todo path; one-paragraph state; container-only caveat |
| Files Created/Changed | `git show --stat` per commit over the whole phase (Tasks 1–5 table: file → task → change → commit) |
| Dependency Rationale | `ctrlc = "3"` (Task 2 plan §S7 evidence verbatim) + "Task 5 added NO dependencies"; real-ffmpeg dev-image rationale (T5-D1 wording) |
| Test Coverage Audit (TODO Task-5 mapping) | The §2 audit table (updated with the NEW test names real names + the fresh suite count from the final gate) |
| Exact Docker/MCP Commands & Exit Statuses | F1 fresh dev-checks gate (log path + `running N tests` + `ALL CHECKS PASSED`); F2 `build-linux.sh`; F3 `build-windows.sh`; F4 artifact `ls -l` cross-check; F5 `ffprobe -version`+`ffmpeg -version`; F6 the demo VM smoke (§9.4 verbatim block incl. observed print + exit code); image-rebuild run; CITED: Task-4 run-boundary smoke (adherence §4) + prior gate logs; FRESH/RECORDED-CITED markers per the phase-01.2 convention |
| Linux Build Result / Windows Build Result | F2/F3/F4 observed paths + byte sizes (report ACTUAL numbers; cross-check F2/F3 sizes against the F4 listing) |
| Manual Platform Tests Actually Performed | Literalstyle blocks: `Native Windows execution: NOT RUN` / `Native Linux execution: NOT RUN`; performed = container-only VM smoke records (run-boundary cited; demo fresh) |
| Acceptance Criteria Verification | ALL 14 criteria of the TODO; per-row evidence pointer + flip decision (the authoritative flips happen in 4.6) |
| Documentation Accuracy Audit | §6.1–§6.6 checklist results (claim-scan greps: no "begins streaming"/"OAuth implemented" claims; FFmpeg-external statement present in README+configuration+build-windows) |
| Environment Notes | image with ffmpeg + pinned rust:1.82; named-volume intermediates; autocrlf/.gitattributes note; MCP stderr caveat |
| Evidence Index | plan files (global + tasks 1–5 + adherence/review/simplification files), prior reports, fresh gate logs (exact filenames), the commit list `git log --oneline` (Tasks 1–5 window) |
| Known Limitations & Not-Run Checks | native Windows/Linux execution NOT RUN; Ctrl+C signal delivery + `Drop` kill not asserted by automated tests (code-reviewed only); unbounded `finish()` wait by design this phase; audio absent; renderer right/bottom-margin config-only deviation (carried from phase 02); chat-log epoch-format deviation (carried); citation `-DONE` update note (H3) |

---

## 8. Verification command sequence (sub-step 4.2 executes; order matters)

Every VM command preceded by `alpine-vm_vm_status`. The VM timeoutMs hints: checks 600000; builds 900000; smokes 600000.

1. **Image rebuild** (after the Dockerfile commit):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust` → exit 0.
   Probe: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ffprobe -version` → exit 0; record the version line.
2. **Spot test run of new suites**:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming` → exit 0 (25 existing + 7 new = 32 streaming tests).
3. **Full standard gate (code-complete tree)**:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   EXPECT: exit 0; PASS fmt-check/check/test/clippy; `running 147 tests` → `147 passed; 0 failed; 0 ignored`; summary `ALL CHECKS PASSED`. Record the printed `logs/checks/<UTC>.log` path. Report the ACTUAL count even when it differs (e.g. if a review adds/removes a test).
4. **Demo VM smoke (real pipeline; once this task)**:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cp config/config.example.json config/config.json && sed -i -e "s|\"font\": \"fonts/console.ttf\"|\"font\": \"fonts/Exo2-Light.otf\"|" -e "s|\"executable\": \"ffmpeg\"|\"executable\": \"/usr/bin/ffmpeg\"|" config/config.json && cargo build --locked && "$CARGO_TARGET_DIR/debug/rust-youtube-streamer-service" demo; status=$?; rm -f config/config.json; echo "demo_exit_code=$status"'`
   EXPECT (record verbatim): stderr tracing events (`mode selected command="demo"`, `configuration loaded and validated`, `mode finished exit_code=0`); stdout exactly one line `demo recording finalized: /rust-streamer-target/debug/videos/demo-<UTCSTAMP>.mp4`; `demo_exit_code=0`; the recording lives inside the named volume (never in the repo tree).
   Follow-up probe (separate VM command, substituting the OBSERVED printed path):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'ls -l "<print>" && ffprobe -v error -show_entries stream=width,height,r_frame_rate:format=duration -of default=noprint_wrappers=1 "<path>"'` — expect 1920×1080, `r_frame_rate=30/1`, duration ≈ 16–17 s.
5. **Both release builds (after all source commits)**:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` → exit 0, record size.
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` → exit 0, record size.
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` → cross-check both sizes; both artifacts non-empty.
6. **Host-side hygiene after the smokes**: `git status --porcelain` → empty (config.json removed inside the invocation; `logs/chat.log` is gitignored — VERIFY `git check-ignore -v logs/chat.log` matches, and optionally remove it with one container `rm -f logs/chat.log` to leave logs/ pristine); `git ls-files videos/` → empty; `git check-ignore -v videos/` → the `videos/` rule.
7. **Final fresh gate on the docs-final tree** (after ALL commits incl. docs/report):
   repeat step 3 (cheap; precedent = phase-02 F1). The report cites this final log.

---

## 9. Commit plan (locked order; each commit leaves `cargo check --locked` green)

1. `chore: restore the versioned logs folder placeholder` — `logs/.gitkeep` (H1).
2. `test: add fake ffmpeg process smoke tests` — `src/streaming/ffmpeg_tests.rs` (new) + `src/streaming/ffmpeg.rs` (test-module declaration only).
3. `build: install ffmpeg in the dev checks image` — `Dockerfile` only.
4. `test: probe a real ffmpeg encoded recording` — `src/streaming/ffmpeg_real_tests.rs` (new) + `src/streaming/ffmpeg.rs` (second declaration line).
5. `docs: update scope authority citations to the completed todo filenames` — the 9 `src/` doc-comment files (H3).
6. `docs: document the demo command and local ffmpeg recording` — `README.md`, `docs/demo.md` (new), `docs/configuration.md` (6.1–6.3).
7. `docs: refresh project structure tech and context for phase 03` — `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md` (6.4–6.6; context rows may still carry `pending` commit references).
8. `docs: add phase 03 completion report` — `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (new; written AFTER the §8 runs so all evidence values are real) + this plan file's final updates if the 4.5b feedback changed anything (planner-approval only).
9. Task-5 plan tracking precedes/parallels the above: this plan file itself is the caller's 4.1b artifact; stage it with the report commit if still untracked (message: `docs: track phase 03 task 5 planning documents` — separate small commit when the adherence report file also lands).
10. **Sub-steps 4.3 (review/simplify fix commits, if any), 4.5b (adherence report file under `.kilo/plans/`), and 4.6 (`chore: mark phase 03 task 5 and acceptance criteria done` — TODO heading `[DONE]` + all 14 checkbox flips `[x]`)** are owned by their workflow steps, NOT by this plan's 4.2 commit list above.

Verification runs: gate after commit 4 (step 8.3), builds after (8.5), demo smoke (8.4), FINAL gate after commit 7 (step 8.7). Gitignore compliance before EVERY commit: `git status --porcelain` + confirm no `videos/`, `logs/*` (except the restored `.gitkeep`), `dist/`, `checks/` entries staged.

---

## 10. Acceptance-criteria matrix (every criterion has an owner; this task closes them)

| # | Acceptance criterion (short) | Owner / evidence |
| --- | --- | --- |
| 1 | `demo` accepted + documented in help/usage | Task 3 code + `cli_tests::usage_text_advertises_the_demo_command`; docs: README commands + docs/demo.md + configuration.md usage block (6.1/6.2/6.3) |
| 2 | Predefined messages, deterministic schedule, no user input | Task 3 code/tests; docs/demo.md §1; report audit table |
| 3 | Demo never contacts YouTube/OAuth/broadcast/chat | Module docs + no-YouTube-import state (Tasks 1–3); docs/demo.md §6; report Documentation Accuracy Audit claim-scan |
| 4 | FFmpeg receives complete RGB24 frames at configured dimensions/FPS via stdin | args_tests + NEW `fake_ffmpeg_success_writes_exactly_the_piped_bytes` + real smoke (dims/r_frame_rate probe) |
| 5 | Frames continue during empty intervals; messages rendered from ChatStore; FIFO eviction | Task-2 runner tests + demo tests (28 > 26); docs/demo.md §1 |
| 6 | Launched without shell; errors/early exits/broken pipes/non-zero exits surfaced clearly | Task-1 no-shell design + error tests + NEW tests #2/#5/#6 + real smoke |
| 7 | MP4 finalized before completion reported | Task-1 `finish` contract + NEW test #3 (negative) + #1 (positive) + demo smoke observed print ordering |
| 8 | Output in `videos/` beside the executable, collision-safe, cwd-independent, no overwrite | Task-1 video_output tests + demo smoke output path; docs/demo.md §4 |
| 9 | `run` is not interactive/local-demo; no false YouTube claims | Task-4 (code + tests + cited VM smoke); docs: README run bullet + configuration.md run bullet |
| 10 | No screenshot/audio/silent-audio | All phases' negative scope; docs/demo.md §6 + report limitations |
| 11 | Tests cover CLI, scheduling, frame/process boundaries, output path, failures; checks and both builds run (or limitations recorded) | This plan §2 audit + NEW tests; §8.3 gate + §8.5 builds (ACTUAL results recorded) |
| 12 | Docs, structure, context and completion report reflect reality | §6 touch list + §7 report; Documentation Accuracy Audit |
| 13 | No unbacked native-Windows claims | NOT-RUN wording everywhere (README, demo.md, report §Manual/§Limitations); cross-compilation caveat lines preserved |
| 14 | (Self-check) plan covers TODO §5 with zero unmapped bullets | §2 table — every bullet mapped |

---

## 11. Explicitly out of scope for Task 5 (HARD — any appearance = plan violation)

- NO new production source features/logic (the only `src/` edits are the test modules + 9 citation lines).
- NO dependency additions/removals; NO `Cargo.toml`/`Cargo.lock` edits; NO config schema changes.
- NO real-FFmpeg test beyond §4 (no demo-time automated re-run of the full demo; no YouTube/network tests — YouTube stays unimplemented).
- NO commit of MP4/videos/dist artifacts/logs (all ignored targets); no force-adds.
- NO native Windows/Linux execution claims; NO silent cliff: if any evidence value cannot be observed, write the NOT-RUN limitation instead (never invent).
- NO renaming the TODO file, NO push/merge (workflow steps 5 belong to the implementer's later step, not 4.2).
- NO edits to `docs/build.md`, `docs/build-windows.md` (verified still accurate; rebuild prereq already covered).

---

## 12. Review & adherence checklist (steps 4.3 / 4.5b)

1. Suite count ACTUAL matches the final gate line (`running N tests` — expected 147); the fake-process tests are `#[cfg(all(test, unix))]`-gated; real test asserts image-tool presence (never silent-skip).
2. Fake scripts: shebang only, 0o755, inside `TempDir`, never call `ffmpeg`, last-arg = output path; no sleeps; cleanup by TempDir drop.
3. `ffmpeg.rs` ≤ 200 lines; `ffmpeg_tests.rs` ≤ 200; `ffmpeg_real_tests.rs` ≤ 120; test bodies ≤ 50 lines; ≤2 args/test helper; no commented-out code; no `unwrap`/`panic` outside test assertions.
4. Housekeeping: `logs/.gitkeep` tracked; the 9 citation files point at `-DONE` filenames; `src/youtube/.gitkeep` untouched; project-structure rows match the live tree (spot-verify each claim with `git ls-files` / `ls`).
5. Docs: grep audit over README/docs/configuration/build docs — no stale "not launched"/`auth|run`-only statements remain; Tokio claim fixed; demo documented; TOC valid (anchor check); ≤ sensible lengths; real newlines.
6. Report: all 14 sections; no invented values; FRESH/CITED markers correct; ACTUAL build sizes from listing cross-check; commit list matches §9 + fix commits.
7. Commits: exact messages/order; nothing staged beyond each commit's file list; tree clean at the end; no push.

---

## 13. Risks / watch items

- **Image rebuild prerequisite**: the real smoke fails without the rebuilt image — run the rebuild FIRST and verify `ffprobe -version` before the gate; if the apt install fails (network/registry), STOP and report (no workaround).
- **dash portability**: `while [ "$#" -gt 1 ]; do shift; done` + `cat > "$1"` verified under POSIX sh semantics; if the container's sh rejects any construct, use the same loop with `if [ "$#" -gt 1 ]`-free bodies as long as the CONTRACT (last arg = output; stdin exhausted to EOF) holds — record any deviation in the test-module docs, never change the app code.
- **Broken-pipe test margin**: 76800 bytes > 64 KiB pipe buffer guarantees the failure path; if a write unexpectedly succeeds (pipe buffer larger than expected on some kernel/policy), treat the finding as STOP-and-report, not "fix by sleeping".
- **ffprobe tolerance**: assert `r_frame_rate=5/1` exactly (constant input rate ⇒ exact), duration within [1.5, 2.5]; if the observed duration is outside, record ACTUAL and stop (possible container stall — escalate).
- **Citation-to-future-rename window**: the H3 `20261011-todo-1-DONE.md` references are live only after workflow step 5; 4.5b/step-5 implementer verify the rename happened.
- **Report `pending` rows**: hashes created after the report text are listed as pending; they are recorded in execution notes (phase-02 precedent), never back-patched silently.
- **MCP stderr capture may be partial** — exit status authoritative; `ls -l`/`ffprobe` outputs give the byte-level cross-checks.

---

## 14. Sub-agent hand-off notes

- 4.2 implementer: executes §5 (H1–H3 after test commits), §3–§4 (tests), §8 runs, §9 commits 1–5; reports the recorded evidence values back to the caller (gate log paths, test count ACTUAL, build sizes, ffprobe version, demo smoke outputs).
- 4.3 reviewer/simplifier: verify against §3–§5 + §12 checklist; fix plans go to `.kilo/plans/20261011-phase03-task5-*-fix.md` naming per precedent.
- 4.4 docs-specialist: executes §6 (+§7 draft) with the implementer's recorded values; commits 6–8.
- 4.5b architector: adherence check vs this plan (diffs recorded; verdict + deviations like Task-4's report).
- 4.6 implementer: `[DONE]` marker + 14 checkbox flips verified against observed evidence; commit.
- Planner: final accept → workflow step 5 (TODO rename, merge, push `origin` only).

[PLAN-CHECK-END]
