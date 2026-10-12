# Adherence Report — Phase 03, TODO Task 3: Dedicated `demo` Command (sub-step 4.5b)

- **Date**: 2026-10-12
- **Caller step**: TODO Task 3, sub-step 4.5b (Overall Plan Adherence; architector)
- **Branch / HEAD reviewed**: `feat/phase03-ffmpeg-local-pipeline` @ `e82d5b1`
- **Assessed against**: `### 3. Dedicated demo Command` of `.agent/todos/20261011/20261011-todo-1.md` + standing constraints + Task-3 acceptance criteria
- **References**: `.kilo/plans/20261011-phase03-task3-demo-command.md` + `.kilo/plans/20261011-phase03-task3-simplification.md` + the planner ruling ledger (D1–D5), which supersedes the plan where they differ
- **Verdict: ADHERENT** — every TODO Task-3 bullet and Task-3 acceptance criterion is satisfied by the live tree; all plan-vs-live differences are ledger-sanctioned (listed in §7). No unaccounted deviation found. No changes proposed.

---

## 1. TODO Task-3 bullets, one by one (live-code evidence)

| # | TODO bullet | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | `demo` added to the CLI parser and help/usage without breaking `auth`/`run`/`--config` ordering rules or exit-code contracts | PASS | `src/app/cli.rs`: `DEMO_COMMAND` (line 36), `Mode::Demo` (43), `USAGE` advertises `  demo                 run a local-only FFmpeg demo recording (no YouTube)` (25); both usage-error strings say `('auth', 'demo' or 'run')` (127, 146); parser structure otherwise untouched (`--config` before/after at most once, `--help` before command). `src/app/cli_tests.rs` pins all of it (15 tests incl. `parses_demo_alone_with_default_path`, `accepts_config_option_around_demo`, `usage_text_advertises_the_demo_command`; all 9 pre-existing tests still green in the gate log). |
| 2 | `demo` loads/validates config through the existing startup funnel, then runs a local-only demo; no YouTube auth / resources / chat API / transmission | PASS | `src/app/mod.rs` `start_mode`: logging init → mode-selected event → `load_configuration` → `dispatch` (79–101); `dispatch` arm `cli::Mode::Demo => demo::run_demo(app_config)` (123). Grep of `src/app/demo/` for `youtube|oauth|broadcast|rtmps|credential`: only the boundary sentence in the module doc (`mod.rs:2`). Demo imports are exactly pipeline/config/streaming/chat-model items. |
| 3 | Small fixed, documented predefined message set, deterministic timing, fed through ChatStore (logged + rendered by the same path) | PASS | `src/app/demo/schedule.rs`: `DEMO_MESSAGE_TOTAL = 28`, `LEAD_IN_FRAMES = 30`, `MESSAGE_GAP_FRAMES = 15`, `TAIL_FRAMES = 60` (12–15); `fixed_timeline()` is the only production constructor (39); messages flow `DemoMessageSource` → `MessageSource` seam → `ChatStore::accept` → `visible_text()` → renderer (runner.rs 83–107) — the same path real chat will use. Each message carries hidden `demo-NN` message_id (`schedule.rs:94`); `schedule_tests.rs::long_line_message_carries_the_pinned_text_and_hidden_metadata` proves metadata-never-visible. Module doc documents the deltas (schedule.rs 1–8). |
| 4 | Demo runs long enough; FIFO eviction of older messages happens naturally; finishes automatically and finalizes the MP4 | PASS | Final frame index 495, `rendered_frame_count` 496 (≈16.5 s at the example 30 FPS) — `final_frame_index_and_rendered_count_match_the_planning_formula` (495/496 pinned). 28 deliveries > 26 example-config visible capacity — `message_set_exceeds_the_default_visible_capacity` asserts `capacity == 26` and `total_deliveries > capacity`. Composition test asserts the last rendered input drops `demo message 01`/`02` and shows `demo message 28` (eviction observed). Auto-finish via `StopFlag` requested at the frame-495 drain call; sink `finish()` runs exactly once after the loop breaks. |
| 5 | Result under `videos/` beside the executable, `demo-` prefix; path printed/logged only after FFmpeg exits successfully | PASS | `OutputKind::Demo` → `resolve_output_path` inside `prepared_pipeline` (`runner.rs:139-140`; `video_output.rs`: `OUTPUT_DIRECTORY_NAME = "videos"`, prefix `demo`, collision-safe suffixes, executable-relative). `print_final_path` is called ONLY after `pipeline.run()` returned `Ok` (`demo/mod.rs:84-87`), so no success output on any error path; stdout `println!`, tracing whitelist untouched. `.gitignore:55` covers `videos/`. |
| 6 | Schedule testable without real-time sleeps (injectable seam / isolated schedule data) | PASS | Frame-indexed pure data (`schedule.rs` has no clock, no interval field — D3); source counts calls (one call = one frame, `source.rs` doc 1–8); `DemoTimeline::raw` is `#[cfg(test)]`; tests use `FakeClock` only as the `PipelineParts` clock. `schedule_tests.rs` + `source_tests.rs` (12 tests) are clock-free and sleep-free; the handler-level composition test completes with no wall-clock dependency. |
| 7 | `demo` is not a screenshot command / interactive-mode substitute | PASS | No screenshot code anywhere (`rg` for screenshot-style paths: none); no `stdin()` user-input read in `src/` (only FFmpeg stdin plumbing + `-nostdin` arg). |

## 2. Data-flow sanity (funnel → demo → pipeline)

- Dispatch: `app/mod.rs` `dispatch` → `demo::run_demo(app_config)` with the already-validated config. PASS.
- Interrupt-first order: `demo_outcome` installs the Ctrl+C handler BEFORE building the pipeline (`demo/mod.rs:78`). PASS.
- `DemoError` Display delegations verbatim: both arms `write!(formatter, "{error}")` — no wrapper text (56–63); mod_tests test 1 asserts the pinned phrases (`unable to install the Ctrl+C interrupt handler`, `frame output failed`). PASS.
- Exit mapping exactly `{usage 2, startup/config failure 1, demo runtime failure 1, success 0}`: `StartupError::exit_code` (error.rs 41–46), `DemoError::exit_code → STARTUP_FAILURE_EXIT` (1) for both variants, `DEMO_SUCCESS_EXIT = 0`; `demo` never returns 3; `auth`/`run` keep 3 (`modes.rs` untouched by Task 3 — absent from the 7f940af..HEAD diff). PASS.
- `modes.rs` untouched, `main.rs` clean (15 lines, no `#[allow]`): PASS (cumulative diff lists neither file).
- Tracing whitelist unchanged (3 events, `command`/`config_path`/`exit_code` only); the demo path print is stdout, not a log field. PASS.

## 3. Commit chain vs ledger

| Ledger item | Live state | Verdict |
| --- | --- | --- |
| Plan-commits 1–4 merged into `a0f27f4` `feat: add the demo command to the cli and local demo pipeline` | Commit exists; stat = cli.rs, cli_tests.rs, demo/{mod,mod_tests,schedule,schedule_tests,source,source_tests}.rs, app/mod.rs, pipeline/{interrupt,mod}.rs, chat/{mod,model}.rs — no unrelated files | PASS |
| Commit 5 = `eaa12dc` `refactor: retire the dead-code staging allows on the wired pipeline` — 3 item-level allowances with comments + 6 cfg(test) gates | Stat = the 4 former staging blocks (pipeline/mod, chat/mod, renderer/mod, streaming/mod) + allowances (chat/store.rs, chat/model.rs, streaming/video_output.rs) + cfg gates (renderer/frame.rs, renderer/settings.rs). Live greps: exactly 3 `#[allow(dead_code)]` in `src/` (store.rs:123 `is_empty`, model.rs:44 `with_timestamp`, video_output.rs:27 `Session`), each with its plan-§7 comment; ZERO module-inner `#![allow(dead_code)]` blocks remain; no other `#[allow(` anywhere | PASS |
| Commit 6 = `7c840b6` plan docs + `e82d5b1` simplification | `7c840b6` touches only `.kilo/plans/20261011-phase03-task3-demo-command.md`; `e82d5b1` touches only `demo/source.rs` + `demo/source_tests.rs` (the simplification plan's exact staging set) | PASS |
| D2 re-export blocks | `pipeline/mod.rs` production re-exports FramePipelineError, install_interrupt_handler/InterruptHandlerError/StopFlag, prepared_pipeline/PipelineConstruction, MessageSource; cfg(test) re-exports PipelinePacing, FramePipeline/PipelineParts, FakeClock/RecordingSink/StubRenderer/TEST_FRAME_BYTES/TEST_INTERVAL_NANOS; `chat/mod.rs` cfg(test)-gates `derive_visible_capacity`; NO `interval_nanos` re-export; `renderer/mod.rs` keeps the cfg(test) `Rgb` re-export with rationale; consumers (`demo/mod.rs`, `demo/source.rs`, test files) use the re-export block. The plan's "NO self re-exports" sentence is superseded by D2 | PASS |
| D3 | `DemoTimeline` has NO interval field; `raw(deliveries)` single-arg cfg(test) constructor; `for_config` renamed `fixed_timeline` (schedule.rs 39–49); `build_demo_source(stop_flag)` single-arg accordingly | PASS |
| D4 test-module placement | `schedule_tests` declared in `schedule.rs` (119–121), `source_tests` in `source.rs` (50–52), both via `#[path]`; `demo/mod.rs` keeps `mod_tests` via `#[path]` (14–16) | PASS |
| D5 suite = 138 | Counted from SOURCE files (not gate logs): 15 cli + 6 schedule + 6 source + 2 demo/mod + 109 unchanged = **138** `#[test]` items (`rg -c "^\|" -g "*.rs"` per-file sum). Matches ledger decomposition 121 + 3 + 6 + 6 + 2 | PASS |
| Mechanical enablers | `ChatMessage` derives `Debug, Clone` (model.rs:11); `Box::new(...)` wraps the source at `PipelineConstruction` (demo/mod.rs:80); §8.3 test 3 same-slot overdue fixture present (`at_most_one_message_is_delivered_per_call`, `due_frame_index: 5` ×3) | PASS |
| §4.3 locked stop semantics | `is_end` computed from the RAW pre-increment frame index; frame 495 drain call requests stop and frame 495 still renders; next iteration breaks at the top stop check → 496 total; `timeline_requests_the_stop_at_its_last_frame` + `timeline_ends_after_its_final_delivered_message` pin both edges | PASS |

No unrelated staging changes in any of the four commits (per-commit stats verified); cumulative `git diff 7f940af..HEAD` touches ONLY the Task-3 files above — no Cargo.toml/Cargo.lock, config schema, .gitignore, Dockerfile, scripts, docs, runner/pacing/clock/error/source pipeline files.

## 4. Tests

- Counting source files: **138** total = 121 pre-existing + 3 cli + 6 schedule + 6 source + 2 demo/mod. (The plan's §7.7 table row "3" for mod_tests was internally inconsistent with its own §8.1 note "folded into test 2"; the ledger D5 resolves it at 2 — live matches the ledger.)
- All demo tests are FFmpeg-free (`RecordingSink`/`StubRenderer`/`FakeClock` doubles; no `FfmpegProcess`) and sleep-free (frame-indexed drives; FakeClock advances only via recorded sleeps).
- cli suite: 15 tests; the 3 new ones + 3 updated literals exactly per plan §4.6; all other tests untouched and green.
- Gate log `logs/checks/20261012T024300Z.log` (newest; later than the 023810Z cited in the task prompt): `running 138 tests` → `138 passed; 0 failed; 0 ignored`; PASS fmt-check/check/test/clippy; `ALL CHECKS PASSED`.

## 5. Rule-compliance snapshot

- ≤200 lines/file: ALL `src/**.rs` measured (`rg -c "^"`); max = `pipeline/test_helpers.rs` at exactly 200 (pre-existing Task-2 watch item, untouched by Task 3 — not in the diff); next-near `chat/store_tests.rs` 198. No violation, none added by Task 3.
- ≤50-line method bodies: largest Task-3 bodies — `next_message` 13, `demo_outcome` 7, `run_demo` 5, `scheduled_message` 8. PASS.
- Depth ≤2: deepest nesting in Task-3 code is 2 (match arm → if). PASS.
- ≤2 params: `DemoMessageSource::new(timeline, stop_flag)`, `due_at(already_served, frame_index)`, `build_demo_source(stop_flag)`; >2-arg aggregates use parameter objects (`PipelineParts`, `PipelineConstruction`). PASS.
- Single-section boolean conditions: `scheduled.due_frame_index <= frame_index`, `frame_index >= final_frame_index()`, `is_long_line_slot(number)`, `frame_has_expected_size` helper. PASS.
- No commented-out code: `rg "^\s*//\s*(fn |let |pub |impl |mod |use |#\[|})"` over `src/` → zero hits. PASS.
- Prefer-private: `served`, `current_frame_index`, `timeline`, `stop_flag` private; `served_count()` redirect removed by the simplification (e82d5b1). PASS.
- Newline integrity: `git ls-files --eol src/app/demo` → all `i/lf w/lf`; working tree has no tracked modifications at all (git status clean except the untracked simplification plan file), so no CRLF conversion occurred. PASS.

## 6. Final-gate evidence

- Latest documented dev-checks run: `logs/checks/20261012T024300Z.log` — exit 0 (all four PASS lines), `ALL CHECKS PASSED`, `138 passed; 0 failed; 0 ignored`. This log post-dates the newest commit `e82d5b1`'s source state.
- Working tree: clean except ONE untracked file, `.kilo/plans/20261011-phase03-task3-simplification.md` — by design: the simplification plan's own commit (§3) staged only the two source files, and the plan file itself is left for the caller's docs tracking. No staged or modified source files.
- Working tree diff vs `7f940af` (pre-Task-3): 20 files, +1244/−27 — exclusively Task-3 scope + its plan doc.

## 7. Acceptable deviations (plan-document vs live; ALL ledger-sanctioned, none unauthorized)

1. **D1** — plan commits 1–4 merged into single `a0f27f4` (vs plan §6's four commits). Sanctioned; file content matches plan §4.1–4.6.
2. **D2** — re-export blocks adopted in `pipeline/mod.rs`/`chat/mod.rs` (plan §4.7 said "NO self re-exports" and plan §4.4 used full `crate::app::pipeline::…` paths). Sanctioned; live consumes the D2 block; also adds the cfg(test) re-export split.
3. **D3** — `DemoTimeline` carries no interval state; `raw(deliveries)` single-arg; `for_config` renamed `fixed_timeline`; `build_demo_source` loses its config parameter. Sanctioned.
4. **D4** — `schedule_tests`/`source_tests` declared in `schedule.rs`/`source.rs` instead of `demo/mod.rs` (plan §4.4). Sanctioned.
5. **D5** — suite is 138 with 2 `mod_tests` tests (plan §7.7 said 3 tests / 136 total; plan §8.1 contradicted itself; ledger resolves). Sanctioned.
6. **eaa12dc extra files** — plan §7 listed only 3 allowances; the commit also adds the 6 `#[cfg(test)]` gates (`DemoTimeline::rendered_frame_count`/`total_deliveries`, `Frame::width`/`height`, `Rgb::parse_hex`/`black`), required to keep clippy `-D warnings` green post-retirement. Ledger-sanctioned (explicitly enumerated in the task ruling).
7. **mod_tests test 2** asserts 25 `'\n'` characters (= 26 lines) where plan §8.1 phrased it "26 '\n'-separated lines" — same fact; simplification plan §4.11 pins the literal as intentional.
8. **`build_demo_source` doc/shape** and `Box::new` at `PipelineConstruction` — ledger mechanical enablers.

No other differences found. No changes proposed.

## 8. Per-item verdict summary

- CLI + help/usage: PASS
- Config through startup funnel: PASS
- No YouTube/auth/broadcast/chat/transmission: PASS
- Fixed message set + deterministic schedule, no user input: PASS
- Long-enough run + FIFO eviction (28 > 26): PASS
- `videos/` beside exe, `demo-` prefix, path printed only after FFmpeg success: PASS
- Sleep-free testable schedule: PASS
- No screenshot/interactive substitution: PASS
- Exit mapping {2, 1, 1, 0}; modes.rs untouched; main.rs clean: PASS
- Commit chain + retirement + allowances + cfg gates: PASS (per ledger)
- 138 tests, FFmpeg-free, sleep-free: PASS
- Rule compliance snapshot: PASS (test_helpers.rs at its pre-existing 200-line ceiling, untouched)
- Final gate green + working tree clean (one expected untracked plan file): PASS

**Overall: ADHERENT.**

[PLAN-CHECK-END]
