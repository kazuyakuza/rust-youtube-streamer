# Phase 03 Completion Report — FFmpeg Process and Local Video Pipeline

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
- [Environment Notes](#environment-notes)
- [Evidence Index](#evidence-index)
- [Known Limitations & Not-Run Checks](#known-limitations--not-run-checks)

## Overview

- Branch: `feat/phase03-ffmpeg-local-pipeline` (checked out; never pushed from this task). HEAD at report writing: `18caeb2`; report date 2026-10-12.
- TODO scope authority: `.agent/todos/20261011/20261011-todo-1.md` — Tasks 1–4 are marked `[DONE]`; Task 5 (tests, documentation, project metadata) is implemented on this branch, with its review/adherence/completion sub-steps (4.3/4.5b/4.6) owned by the remaining workflow steps.
- State: Phase 03 ships `src/streaming/` (FFmpeg process wrapper with the locked argument vector, no shell, drain threads, typed spawn/write/exit/finalize errors, collision-safe beside-executable `videos/` output), `src/app/pipeline/` (synchronous chat → renderer → FFmpeg frame loop with a `ctrlc` stop flag), `src/app/demo/` (the local-only `demo` command with a fixed 28-message timeline), and the clarified `run` boundary (still exit 3). Task 5 added 7 process-level tests (6 fake-FFmpeg + 1 real-FFmpeg probe smoke), ffmpeg dev tooling in the check image, housekeeping (restored `logs/.gitkeep`, 9 citation alignments), and the full documentation refresh. The suite is **147 tests green** in the container; both release artifacts were rebuilt fresh; a VM demo smoke recorded and probed a real MP4.
- Caveat (repeated everywhere): all verification is **container-only** (Linux checks on the Alpine VM). Native Windows/Linux runtime execution is NOT RUN; the Windows `.exe` evidence is cross-compilation only. Real-FFmpeg evidence is dev-image evidence against the OBSERVED `ffmpeg version 5.1.9-0+deb12u1`.

## Files Created/Changed

Tasks 1–5 window: `git log --oneline main..HEAD` (full list in [Evidence Index](#evidence-index)). Per-task file groups:

| File / group | Task | Change | Commit(s) |
| --- | --- | --- | --- |
| `src/streaming/{mod,error,args,ffmpeg,sink,utc_timestamp,video_output}.rs` + 4 test files | 1 | FFmpeg process wrapper: typed `StreamingError`, locked arg vector, drain threads, finalize contract, `Drop` kill+reap, beside-exe `videos/` output, UTC names | `3bde337`, `bb7c5ac`, `9c4514d`, `2bcadfc`, `0a51be6`, `57c2803` (fixes `df9b00a`, `21189b8`, `3bb24f1`; review `32ee507`; docs `087989f`) |
| `.gitignore` (`videos/` rule) | 1 | ignore local video output | `7b19901` |
| `Cargo.toml`, `Cargo.lock` | 2 | version `0.5.0 → 0.6.0`; new dep `ctrlc = "3"` (locked 3.5.2) | `35ee4ff`, `89c1f9a` |
| `src/app/pipeline/{mod,clock,pacing,runner,interrupt,error,source,test_helpers}.rs` + test files | 2 | frame pipeline: pacing, stop flag, typed pipeline errors, chat→renderer→sink wiring | `c4f40b6`, `1ebe80d`, `35db4d9`, `47d48e6`; `.gitkeep` removal `60102fc`; docs `5623940` |
| `src/app/{cli.rs,cli_tests.rs,modes.rs,mod.rs}`, `src/app/demo/*` (6 files) | 3 | `demo` command + fixed timeline + `DemoError → exit 1`; dead-code staging retired | `a0f27f4`, `eaa12dc`, `e82d5b1` |
| `src/app/modes.rs` run boundary | 4 | four-prerequisites message + no-local-fallback + demo pointer; exit 3 unchanged | `5b8c150` |
| `logs/.gitkeep` | 5 | restored versioned placeholder (negation `.gitignore:21` verified) | `801c08b` |
| `src/streaming/ffmpeg_tests.rs` (new), `src/streaming/ffmpeg.rs` (test-module declarations) | 5 | 6 fake-FFmpeg POSIX-script process tests + declaration | `1ccfc4c` (deviation fixes `ec107ec`, `8d8e739`) |
| `Dockerfile` | 5 | install `ffmpeg` (+`ffprobe`) as dev tooling in the check image | `1b1c5b2` |
| `src/streaming/ffmpeg_real_tests.rs` (new), `src/streaming/ffmpeg.rs` | 5 | 1 real-FFmpeg smoke test (record + ffprobe asserts) | `62349bb` |
| 9 doc-comment citation files: `src/chat/{mod,error,model,store}.rs`, `src/renderer/{mod,error}.rs`, `src/streaming/{mod,error}.rs`, `src/app/pipeline/mod.rs` | 5 | scope-authority citations → final `-DONE` TODO filenames | `75dc438` |
| `README.md`, `docs/demo.md` (new), `docs/configuration.md` | 5 | demo command/exit codes/FFmpeg-external corrections, new Local Demo Recording section, 8-section demo guide | `ba428fe` |
| `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md` | 5 | full structure refresh (demo/pipeline/streaming maps, v0.6.0, `.gitkeep` ledger, `videos/` note); sync-std + ctrlc + smoke pointer; Phase-03 context sections | `18caeb2` |
| `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (this file) | 5 | tracked completion report | `pending` (this commit) |
| `.kilo/plans/20261011-phase03-task5-verification-docs.md` | 5 | task-5 plan tracking | `pending` (plan-tracking commit) |

## Dependency Rationale

- **`ctrlc = "3"` (Task 2; locked 3.5.2)** — Task-2 plan §S7: "`ctrlc = \"3\"` (new dependency, D12-compliant rationale below). The handler flips an `Arc<AtomicBool>` shared through a `StopFlag` clone into the pacing loop, which polls it at the TOP of every frame iteration." Lockfile verification (task-2 adherence): platform deps `nix`/`windows-sys` enter the lockfile, no async runtime enters the tree (decision S2 holds); MIT OR Apache-2.0.
- **Task 5 added NO dependencies (T5-D6)** — no `Cargo.toml`/`Cargo.lock` edits anywhere in Task 5; fake and real tests use only `std` + the existing dev-dependency `tempfile` (locked 3.14.0).
- **Real-FFmpeg dev image (T5-D1, accepted)** — the pinned `rust:1.82` image lacks ffmpeg, but (i) Task-1's plan §0.2 explicitly deferred runtime verifications V1–V3 (natural EOF finalization, `-framerate` acceptance by the rawvideo demuxer, `libx264`+`veryfast`+`b:v`+`yuv420p` acceptance) to Task 5; (ii) brief §10.2 requires the FFmpeg argument vector to be "tested against the selected FFmpeg version"; (iii) the Task-3 plan pinned the demo success print to a Task-5 manual smoke; (iv) the TODO evidence rule requires probing the produced MP4's dimensions/fps/duration. Choosing NOT-RUN would have left the shipped locked argument vector never exercised by any real FFmpeg anywhere. The change is **dev tooling only**: `ffmpeg`/`ffprobe` are part of the dev-checks Rust image, NOT bundled with either release artifact, NOT shipped, and `build-linux.sh`/`build-windows.sh` behavior is unchanged (both re-ran fresh anyway).

## Test Coverage Audit (TODO Task-5 mapping)

Baseline at plan time: **140 tests** (gate log `logs/checks/20261012T030755Z.log`, RECORDED-CITED). Fresh after Task 5: **147 tests** (FRESH — both F1 and the final gate).

| TODO Task-5 bullet | Coverage | Verdict |
| --- | --- | --- |
| FFmpeg argument construction | `streaming::args_tests` (4): locked vector, defensive pixel format, output `yuv420p`, output path last | covered (Task 1) |
| Output-path resolution beside the executable | `video_output_tests` (10) | covered (Task 1) |
| Timestamp/collision handling | `utc_timestamp_tests` (7) + collision-suffix tests | covered (Task 1) |
| Process error mapping | `error_tests` (4) + NEW fake-process tests below | covered |
| Demo schedule | `demo::schedule_tests` (6) + `source_tests` (6) + `mod_tests` (2) | covered (Task 3) |
| CLI parsing/help | `cli_tests` (15, incl. `usage_text_advertises_the_demo_command`) | covered (Tasks 1/3) |
| Frame byte counts | `runner_tests` + `frame_tests` + NEW `fake_ffmpeg_success_writes_exactly_the_piped_bytes` (427 bytes through the pipe) | covered + NEW process test #1 |
| Continuous output with no messages | `runner_tests::run_emits_immediate_empty_first_frames_with_exact_byte_counts` | covered (Task 2) |
| Message appearance/eviction | `runner_tests::run_rotates_new_messages_into_visible_text` + demo tests (28 > 26) | covered (Tasks 2/3) |
| Orderly EOF/finalization | NEW `fake_ffmpeg_success_writes_exactly_the_piped_bytes` (positive) + NEW `fake_ffmpeg_exit_zero_without_output_file_reports_output_finalize` (negative) + real smoke + demo smoke print ordering | NEW (Task 5) |
| FFmpeg non-zero exit | NEW `fake_ffmpeg_nonzero_exit_surfaces_code_and_diagnostics_tail` + NEW `output_locked_by_existing_directory_surfaces_nonzero_exit` | NEW (Task 5) |
| Missing executable | NEW `missing_executable_reports_typed_spawn_error` (`ErrorKind::NotFound`) | NEW (Task 5) |
| Unwritable output directory | `create_directory_reports_existing_file` (dir-create class) + NEW locked-output surrogate `output_locked_by_existing_directory_surfaces_nonzero_exit` (chmod-based test rejected: non-deterministic in a root container, reason recorded) | NEW surrogate (Task 5) |
| Interruption behavior where feasible | `runner_tests` (stop before start / after Nth write), `interrupt_tests`, `demo::source_tests` (stop at final frame). NOT automated: ctrlc signal delivery itself and `Drop` kill/reap (no observable handle) — documented limitations | covered at the seam + documented limitation |
| Real-FFmpeg smoke (optional) | NEW `real_ffmpeg_records_and_probes_plausible_video` (64×36@5fps ×10 frames; ffprobe asserts `width=64`, `height=36`, `r_frame_rate=5/1`, duration ∈ [1.5, 2.5]; loud prerequisite asserts — never a silent skip) | NEW (Task 5) |

The single real smoke also validates Task-1's deferred **V1** (natural stdin-EOF finalization), **V2** (`-framerate` accepted by the rawvideo demuxer) and **V3** (`libx264`/`veryfast`/`b:v`/`yuv420p` acceptance) against the image's FFmpeg.

## Exact Docker/MCP Commands & Exit Statuses

All runs via the Alpine VM MCP (`vm_status` first; canonical pattern `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`). Exit status is the authoritative success evidence.

### FRESH — image rebuild (after Dockerfile commit `1b1c5b2`)

- `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust` → **exit 0** (`Service rust Built`; earlier layers CACHED, new ffmpeg layer installed 17 upgraded + 138 new packages).

### FRESH — spot run of the new suites (plan §8.2)

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming` → **exit 0**; `running 32 tests` … `32 passed; 0 failed; 0 ignored; 115 filtered out` (25 existing + 6 fake + 1 real).

### FRESH — F1: dev-checks gate on the code-complete tree (plan §8.3)

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → **exit 0**; log **`logs/checks/20261012T040733Z.log`**; `running 147 tests` → `147 passed; 0 failed; 0 ignored`; `PASS fmt-check exit=0`, `PASS check exit=0`, `PASS test exit=0`, `PASS clippy exit=0`; `ALL CHECKS PASSED`.

### FRESH — F5: observed FFmpeg/ffprobe versions (plan §4 version evidence)

- `… run --rm rust ffmpeg -version` → **exit 0**; first line: `ffmpeg version 5.1.9-0+deb12u1 Copyright (c) 2000-2026 the FFmpeg developers` (built with gcc 12, Debian bookworm).
- `… run --rm rust ffprobe -version` → **exit 0**; first line: `ffprobe version 5.1.9-0+deb12u1 Copyright (c) 2007-2026 the FFmpeg developers`.

### FRESH — F6: demo VM smoke (plan §8.4)

Command (verbatim, config.json created and removed inside the invocation):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cp config/config.example.json config/config.json && sed -i -e "s|\"font\": \"fonts/console.ttf\"|\"font\": \"fonts/Exo2-Light.otf\"|" -e "s|\"executable\": \"ffmpeg\"|\"executable\": \"/usr/bin/ffmpeg\"|" config/config.json && cargo build --locked && "$CARGO_TARGET_DIR/debug/rust-youtube-streamer-service" demo; status=$?; rm -f config/config.json; echo "demo_exit_code=$status"'
```

→ **exit 0**. stdout verbatim:

```
demo recording finalized: /rust-streamer-target/debug/videos/demo-20261012-040827.mp4
demo_exit_code=0
```

stderr (whitelisted tracing events, ANSI stripped): `mode selected command="demo" config_path=config/config.json` → `configuration loaded and validated` → `mode finished command="demo" exit_code=0`.

Follow-up probe (separate run, observed path substituted):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'ls -l "/rust-streamer-target/debug/videos/demo-20261012-040827.mp4" && ffprobe -v error -show_entries stream=width,height,r_frame_rate:format=duration -of default=noprint_wrappers=1 "/rust-streamer-target/debug/videos/demo-20261012-040827.mp4"'
```

→ **exit 0**:

```
-rw-r--r-- 1 root root 542416 Oct 12 04:09 /rust-streamer-target/debug/videos/demo-20261012-040827.mp4
width=1920
height=1080
r_frame_rate=30/1
duration=16.534000
```

The recording lives inside the Compose named volume (never the repo tree); duration ≈16.5 s matches the 496-frame/30-FPS timeline contract.

### FRESH — plan-§5 ruling diagnostics (fake body 1 without `exit 0`)

- Container run of body 1 minus `exit 0`: writable-file target → **status 0** (10-byte stdin written); directory target → **status 2** with stderr `cannot create …/locked.mp4: Is a directory`.
- Container run of body 1 WITH `exit 0` (the locked-text contradiction): directory target → **status 0** — proving test 6's non-zero-exit assertion could not hold until the planner ruling removed the line.

### FRESH — F2/F3/F4: builds and artifact cross-check (plan §8.5)

- `… run --rm rust sh scripts/build-linux.sh` → **exit 0**; `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (2045112 bytes).`
- `… run --rm rust sh scripts/build-windows.sh` → **exit 0**; `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (3675757 bytes).`
- `… run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` → **exit 0**; listing sizes `2045112` and `3675757` — cross-checks F2/F3 byte-for-byte; both artifacts non-empty.

### FRESH — host-side hygiene (plan §8.6)

- `git status --porcelain` → only `?? .kilo/plans/20261011-phase03-task5-verification-docs.md` (the task-5 plan file, staged by the final plan-tracking commit); no `config.json`, no videos, no dist, no logs staged.
- `git check-ignore -v logs/chat.log` → `.gitignore:18:logs/*`; chat log removed after the smoke (`logs/` pristine: `.gitkeep` + gitignored `checks/`).
- `git ls-files videos/` → empty; `git check-ignore -v videos/` → `.gitignore:55:videos/`; no `videos/` directory exists in the repository tree.

### FRESH — final gate on the docs-final tree (plan §8.7)

- `… run --rm rust sh scripts/dev-checks.sh` → **exit 0**; log **`logs/checks/20261012T041601Z.log`**; `running 147 tests` → `147 passed; 0 failed; 0 ignored`; four PASS lines; `ALL CHECKS PASSED`. (This report cites this final log; the report/plan-tracking commits are docs-only and land after it.)

### RECORDED-CITED (not re-executed this task)

- Task-4 run-boundary VM smoke — `.kilo/plans/20261011-phase03-task4-adherence.md` §4: stderr `error: runtime pipeline is not implemented yet: YouTube streaming needs OAuth authorization, broadcast lifecycle, RTMPS ingestion URL acquisition and YouTube live chat integration, none of which is built; the run mode never reads input, never runs the local demo pipeline, and will start the YouTube/FFmpeg pipeline only in a later phase. There is no local fallback: for a local demonstration recording use the demo command` (single line) and stdout `run_exit_code=3`; no demo/FFmpeg activity.
- Plan-time baseline gate (architector, §0 of the task-5 plan): `logs/checks/20261012T030755Z.log` — `140 passed; 0 failed`, `ALL CHECKS PASSED`.
- Prior-phase green gates and builds remain indexed in `.agent/reports/20261010-phase-02-renderer-chat-store.md` and the phase-01/01.2/01.1 reports.

## Linux Build Result

- Script: `sh scripts/build-linux.sh` → **exit 0**.
- Artifact: `dist/rust-youtube-streamer-service` — **2045112 bytes** (non-empty; mode `-rwxrwx---`, `root:vboxsf` on the shared mount).
- Cross-check: the `ls -l` listing shows `2045112` — matches the build script's printed size exactly.
- `dist/` remains gitignored (`dist/` rule); the artifact is never staged or committed.

## Windows Build Result

- Script: `sh scripts/build-windows.sh` → **exit 0** (`cargo build --release --locked --target x86_64-pc-windows-gnu`).
- Artifact: `dist/windows/rust-youtube-streamer-service.exe` — **3675757 bytes** (non-empty).
- Cross-check: the `ls -l` listing shows `3675757` — matches the build script's printed size exactly.
- This is **cross-compilation only** (GNU toolchain, static GNU runtime — script-reported objdump audit found no MinGW runtime DLLs). It is a build fact, NOT native Windows runtime validation.

## Manual Platform Tests Actually Performed

```text
Native Windows execution: NOT RUN
Native Linux execution: NOT RUN
```

Performed (container-only, via Alpine VM MCP):

- FRESH demo VM smoke (real pipeline end-to-end in the container): verbatim `demo recording finalized: /rust-streamer-target/debug/videos/demo-20261012-040827.mp4` + `demo_exit_code=0` + follow-up ffprobe fields (`width=1920`, `height=1080`, `r_frame_rate=30/1`, `duration=16.534000`).
- FRESH dev-checks gates (code-complete tree and docs-final tree), FRESH spot run, FRESH both release builds + artifact listing, FRESH image rebuild and version probes.
- RECORDED-CITED: Task-4 `run` boundary VM smoke (`run_exit_code=3` + verbatim single `error:` line).

## Acceptance Criteria Verification

Rows 1–12 are the TODO's 12 checkboxes; rows 13–14 mirror the task-5 plan's §10 matrix (standing native-claims constraint and the plan self-check). Authoritative `[x]` flips happen in sub-step 4.6.

| # | Criterion (short) | Evidence | Flip decision |
| --- | --- | --- | --- |
| 1 | `demo` accepted + documented in help/usage | `cli_tests::usage_text_advertises_the_demo_command` (green); README commands + Local Demo Recording section; `docs/demo.md`; `configuration.md` usage block `auth\|demo\|run` | ready |
| 2 | Predefined messages, deterministic schedule, no user input | `demo::schedule_tests`/`source_tests` (fixed timeline, clock-free, one-per-frame); `docs/demo.md` §What the Demo Does | ready |
| 3 | `demo` never contacts YouTube/OAuth/broadcast/chat | `src/app/demo` module docs + no-YouTube imports (Tasks 1–3); `docs/demo.md` §What the Demo Does Not Do; Documentation Accuracy claim-scan | ready |
| 4 | FFmpeg receives complete RGB24 frames at configured dims/FPS via stdin | `args_tests` (locked vector) + NEW `fake_ffmpeg_success_writes_exactly_the_piped_bytes` (427 exact bytes) + real smoke probe (`64`×`36`, `r_frame_rate=5/1`) | ready |
| 5 | Frames continue during empty intervals; ChatStore rendering; FIFO eviction | `runner_tests` + demo tests (28 > 26 capacity); `docs/demo.md` §What the Demo Does Not Do/Does | ready |
| 6 | Launched without shell; errors/early exits/broken pipes/non-zero exits surfaced clearly | Task-1 no-shell design + `error_tests` + NEW tests #2 (non-zero exit + tail) / #5 (broken pipe) / #6 (locked target) / #4 (missing exe) + real smoke | ready |
| 7 | MP4 finalized before completion reported | `finish` contract + NEW test #1 (positive) / #3 (negative) + demo smoke ordering (print only after FFmpeg exit 0, observed) | ready |
| 8 | Output in `videos/` beside executable, collision-safe, cwd-independent, no overwrite | `video_output_tests` (10) + demo smoke output path (beside debug exe in the named volume) + `docs/demo.md` §Output | ready |
| 9 | `run` is not interactive/local-demo; no false YouTube claims | Task-4 code + tests + RECORDED-CITED VM smoke (`run_exit_code=3`); README run bullet + configuration.md run bullet | ready |
| 10 | No screenshot/audio/silent-audio | negative scope across all phases; `docs/demo.md` §What the Demo Does Not Do; report limitations | ready |
| 11 | Tests cover CLI/scheduling/frame-process/output-path/failures; checks + both builds run | §Test Coverage Audit (147 tests) + F1/final gate + F2/F3/F4 (all fresh, exit 0) | ready |
| 12 | Docs, structure, context, completion report reflect reality | §Documentation Accuracy Audit + this report | ready |
| 13 | No unbacked native-Windows claims (standing constraint) | NOT-RUN wording in README, `docs/demo.md`, this report; cross-build caveat lines preserved | ready |
| 14 | Plan self-check: TODO §5 bullets all mapped | §Test Coverage Audit maps every bullet; zero unmapped | ready |

## Documentation Accuracy Audit

| Check | Result |
| --- | --- |
| §6.1 README: TOC entry `Local Demo Recording` between Logging and Build Checks | holds (anchor `#local-demo-recording`) |
| §6.1 README: Current Status = Phase 03 state, ACTUAL test count 147, `videos/` beside executable, run clarified boundary | holds |
| §6.1 README: Tokio bullet corrected (async reserved for later phases; sync `std::process` + threads + `ctrlc` 3) | holds (grep: no `async runtime for API requests` in live docs) |
| §6.1 README: `demo` bullet + exit-code line (`1 = … demo runtime failure (startup-failure class)`) + run cross-pointer | holds |
| §6.1 README: both stale FFmpeg sentences replaced (external prerequisite; demo launches configured path; never bundled) | holds (grep: no `neither installs nor launches` in live docs) |
| §6.2 `docs/demo.md`: all 8 planned sections + TOC + observed version/counts | holds (131 lines) |
| §6.3 configuration.md: usage `auth\|demo\|run` + demo line; demo bullet with `DemoError` delegation; exit summary exact (0/1/2/3); renderer-wired + FFmpeg-launched closing paragraph; `ffmpeg.executable` Spawn text | holds (grep: no `auth\|run`-only or `wired into the runtime in a later phase` in live docs) |
| §6.4 project-structure: demo/ + pipeline/ maps, real streaming map, `modes.rs` four-prerequisites line, v0.6.0, `logs/.gitkeep` ledger, deliberate `src/youtube/.gitkeep`, `videos/` note, `main.rs` no-allow line | holds (spot-verified against `ls src/...` + `git ls-files *.gitkeep`) |
| §6.5 tech.md: sync-std S2 bullet, ctrlc External-Processes + Dependencies rationale, dev-image smoke pointer | holds |
| §6.6 context.md: rewritten Current Work Focus; new Recent Changes (2026-10-12) + Known Deviations (Phase 03) sections; Immediate Next Steps rewritten; history blocks preserved | holds |
| Claim scan: no "begins streaming"/"OAuth implemented" claims anywhere | holds (hits are all negations: `never starts FFmpeg`, `not implemented yet`, `never contacts YouTube`) |
| FFmpeg-external statement present in README + configuration.md + build-windows.md | holds (build-windows.md lines 60/110 pre-existing, still accurate; build docs untouched per plan §11) |
| `docs/build.md` / `docs/build-windows.md` NOT touched (still accurate; rebuild prereq covered) | holds (no diff in any Task-5 commit) |

## Environment Notes

- Pinned dev-check image: `rust:1.82` + rustfmt/clippy + `gcc-mingw-w64-x86-64` + windows-gnu target + **new:** `ffmpeg`/`ffprobe` (Debian bookworm, `5.1.9-0+deb12u1`) — dev tooling only, not bundled with any artifact.
- Cargo intermediates stay in the Compose named volume `rust-streamer-target`; demo recording written beside the debug exe inside that volume; `logs/checks/<UTC>.log` gate logs are gitignored.
- Repo-host git `core.autocrlf=true`: staging printed the expected `LF will be replaced by CRLF` warnings for `.rs` files; tracked `.gitattributes` forces LF for `*.sh` and `Dockerfile`; no normalization incident this task.
- MCP stderr caveat: captured output may be truncated for long runs — exit status is authoritative; summary lines were re-read from the on-disk gate logs.
- Docker Compose v2.31.0 on the Alpine VM; image rebuild reused all cached layers except the new ffmpeg step.

## Evidence Index

- Plans: global `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md`; Task 1 `20261011-phase03-task1-ffmpeg-wrapper.md` (+ `task1-review-fixes`, `task1-simplification`, `task1-adherence`); Task 2 `20261011-phase03-task2-frame-pipeline.md` (+ `task2-adherence`); Task 3 `20261011-phase03-task3-demo-command.md` (+ `task3-simplification`, `task3-adherence`); Task 4 `20261011-phase03-task4-run-boundary.md` (+ `task4-adherence`); Task 5 `20261011-phase03-task5-verification-docs.md`.
- Prior reports: `.agent/reports/20261010-phase-02-renderer-chat-store.md`, `20261010-phase-01-2-windows-build-workflow.md`, `20261009-phase-01-1-build-artifact-workflow.md`.
- Fresh gate logs: **`logs/checks/20261012T040733Z.log`** (code-complete tree) and **`logs/checks/20261012T041601Z.log`** (docs-final tree) — both `147 passed; 0 failed`, `ALL CHECKS PASSED`; plus the post-mitigation final gate **`logs/checks/20261012T143115Z.log`** (`147 passed; 0 failed`, `ALL CHECKS PASSED`). Baseline (cited): `logs/checks/20261012T030755Z.log` (140 tests).
- Commit list, Tasks 1–5 window (`git log --oneline main..HEAD`; 47 commits after the ETXTBSY test-fix — the report-tracking and plan-tracking commits are now resolved to their actual hashes):

```text
bcbfae9 test: retry transient ETXTBSY when spawning fake ffmpeg
c249138 docs: track task 5 planning documents
82ab621 docs: add phase 03 completion report
18caeb2 docs: refresh project structure tech and context for phase 03
ba428fe docs: document the demo command and local ffmpeg recording
75dc438 docs: update scope authority citations to the completed todo filenames
8d8e739 style: rustfmt the fake ffmpeg process tests
62349bb test: probe a real ffmpeg encoded recording
ec107ec fix: compile fake ffmpeg tests without debug impls
1b1c5b2 build: install ffmpeg in the dev checks image
1ccfc4c feat: add fake ffmpeg process integration tests
801c08b chore: restore the versioned logs folder placeholder
fe165c3 docs: mark task 4 done in phase 03 todo
3e95be4 docs: track task 4 adherence report
97e384a docs: track task 4 planning documents
5b8c150 feat: clarify the run mode production boundary
e6c41f8 docs: mark task 3 done in phase 03 todo
6eae9cc docs: track task 3 simplification and adherence documents
e82d5b1 refactor: drop the demo served-count indirection and derive the test end frame
7c840b6 docs: track task 3 planning documents
eaa12dc refactor: retire the dead-code staging allows on the wired pipeline
a0f27f4 feat: add the demo command to the cli and local demo pipeline
7f940af docs: mark task 2 done in phase 03 todo
7910771 docs: track task 2 planning and adherence documents
5623940 docs: align frame pipeline documentation with task 2 implementation
60102fc chore: remove streaming folder placeholder marker
47d48e6 feat: wire chat renderer and ffmpeg sink into the frame pipeline
35db4d9 feat: add typed frame pipeline error model
1ebe80d feat: add ctrlc stop flag and interrupt install seam
c4f40b6 feat: add frame pacing schedule and monotonic clock seam
89c1f9a build: add ctrlc crate for cross-platform interrupt handling
228fe37 docs: mark task 1 done in phase 03 todo
cd69a3c docs: track task 1 adherence report
087989f docs: align streaming module comments with task 1 implementation
e007235 docs: track task 1 planning documents
32ee507 refactor: simplify streaming internals after task 1 review
3bb24f1 fix: kill and reap FFmpeg child on missing stdin and in Drop
21189b8 refactor: split oversized StreamingError Display impl and tests
df9b00a fix: correct UTC civil date algorithm across non-leap centuries
7b19901 chore: ignore local videos output directory
57c2803 feat: supervise ffmpeg process with drain threads and typed exits
0a51be6 feat: define frame sink seam for ffmpeg frame delivery
2bcadfc feat: resolve beside-exe video output with collision-safe names
9c4514d feat: add pure utc timestamp formatting for recording names
bb7c5ac feat: build ffmpeg raw-video arguments purely from config
3bde337 feat: add streaming module with typed process errors
35ee4ff chore: bump version to 0.6.0
```

## Known Limitations & Not-Run Checks

- **Native Windows/Linux execution NOT RUN** for every Phase-03 component (demo, pipeline, streaming, run boundary); the Windows artifact is a cross-compilation fact only. A future Windows-host validation session (ideally running `demo`) is the retirement path.
- **Ctrl+C signal delivery and `Drop` kill/reap are not asserted by automated tests** (code-reviewed only): the ctrlc handler is a process-global installation and the consumed child handle gives no observable kill signal; both are documented boundaries.
- **`finish()` wait is intentionally unbounded** this phase (no timeout configuration field; broken-wait hardening deferred per plan D10) — documented in README/`docs/demo.md`/`interrupt.rs` module docs, never claimed otherwise.
- **No audio** (neither generated nor silent) — explicit scope exclusion.
- **`DemoError` maps both variants to exit 1** — single startup-failure class, no demo-specific exit code (documented contract).
- **Fake body-1 deviation** from the locked plan text: no trailing `exit 0` (planner ruling; container-observed statuses recorded in §Commands and in the test-module docs); the two implementation deviation commits are `ec107ec` (compile fix: `expect_err` needs `Debug` on `CompletedVideo`/`FfmpegProcess`, avoided without production-code edits per T5-D7) and `8d8e739` (rustfmt normalization).
- **Fake-FFmpeg ETXTBSY flake — discovered, mitigated test-side (commit `bcbfae9`)**: an extra post-report gate run exposed an intermittent `Text file busy` (errno 26) spawn failure. Mechanism: a parallel libtest thread forks during another fake test's `fs::write` open window on the script, the child inherits the write fd (closed only at that child's own exec), and our own `execve` can hit ETXTBSY — reproduced pre-fix at ≈10% (5 failures in 50 sampled `cargo test --locked streaming` runs). Mitigation (test-only): the fake-suite spawn helper `start_fake` in `src/streaming/ffmpeg_tests.rs` bounds an 8-attempt immediate retry (`std::thread::yield_now()`, no sleeps) keyed on `raw_os_error() == Some(ETXTBSY_ERRNO /* 26 */)` — `ErrorKind::ExecutableFileBusy` is unstable before Rust 1.83 and the pinned image is 1.82. Post-fix stability: **30/30 green** streaming-suite runs + fresh full gate `logs/checks/20261012T143115Z.log` (exit 0, `147 passed; 0 failed`, `ALL CHECKS PASSED`). Production spawn behavior is unchanged — the fix commit touches only `src/streaming/ffmpeg_tests.rs` (26 insertions, 2 deletions).
- **Citation `-DONE` window**: the 3 × `20261011-todo-1-DONE.md` doc-comment citations become live only after workflow step 5 renames the TODO; if the rename is ever revoked the citations must revert (verified at 4.5b/step 5).
- Carried from earlier phases: renderer right/bottom-margin config-only deviation; chat-log epoch-format deviation; unclear-provenance `fonts/console.ttf`; `OutputKind::Session` reserved.
