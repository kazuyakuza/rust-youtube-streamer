# Adherence Report — Phase 03, TODO Task 2: Continuous Local Frame Pipeline (sub-step 4.5b)

- **Date**: 2026-10-12 (UTC evidence stamps; local 2026-10-11)
- **Verifier**: architector (task — adherence assessment + report only; no source files modified)
- **Scope**: TODO task heading `### 2. Continuous Local Frame Pipeline` of
  `.agent/todos/20261011/20261011-todo-1.md` (lines 43–51), the standing
  constraints (lines 9–28), the Constraints/Out-of-Scope block (lines 85–96),
  and the Task-2-relevant acceptance criteria (lines 98–111).
- **Primary plan**: `.kilo/plans/20261011-phase03-task2-frame-pipeline.md`
  including the end-of-file `## 8. Addendum — Planner conflict rulings
  (implementation-time)` — this check is ADDENDUM-COMPLIANCE; the pre-ruling
  plan sections R1–R4 supersede are treated accordingly.
- **Repo state verified**: branch `feat/phase03-ffmpeg-local-pipeline`, HEAD
  `5623940`, package v0.6.0, working tree clean except the untracked planning
  file `.kilo/plans/20261011-phase03-task2-frame-pipeline.md` (see Finding 6).
- **Commit chain audited**: `89c1f9a` → `c4f40b6` → `1ebe80d` → `35db4d9` →
  `47d48e6` → `60102fc` → `5623940` (each `git show --stat` + full diff read;
  chain cumulative diff `git diff 228fe37..5623940 --stat` read in full).

---

## Verdict summary

**ADHERENT.** All TODO Task 2 bullets, all Addendum rulings R1–R4 plus the
planner micro-decisions, and the Task-2-relevant acceptance criteria verify
against the code with no blocking deviation. Findings below are observational
or forward-flags; none requires a change to the Task 2 tree.

Evidence method: every claim below was verified by reading the live files (not
by trusting the gate), counting `#[test]` occurrences per file with ripgrep,
reading the commit stat/diffs, and a fresh standard dev-checks gate re-run on
the final committed tree via the Alpine VM MCP.

---

## 1. TODO Task 2 — bullet-by-bullet verdicts

| # | TODO bullet | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Wire Phase 02 renderer + ChatStore into app-level orchestration + FFmpeg frame writer | ADHERENT | `src/app/pipeline/runner.rs`: `FramePipeline<S: FrameSink>` (chat_store, `Box<dyn Renderer>`, `S` sink); `prepared_pipeline(config, PipelineConstruction)` builds `resolve_output_path` → `FfmpegProcess::start`, `ChatStore` via `ChatStoreSettings::from_config`, `TextRenderer` via `RendererSettings::from_config`; re-export blocks landed in `chat/mod.rs`, `renderer/mod.rs`, `streaming/mod.rs`. Orchestration lives at app level (S1); `runner.rs` **never names `FfmpegProcess`** except inside `prepared_pipeline`'s return type/construction (S9-compliant; the plan's own §2.5 sketches exactly this). |
| 2 | Frames emitted continuously at configured FPS incl. before first message and during idle intervals; empty initial frame = configured black background | ADHERENT | Loop in `FramePipeline::run` renders every paced iteration independent of message arrival. Test `run_emits_immediate_empty_first_frames_with_exact_byte_counts` asserts renderer inputs exactly `["", "", ""]` — the FIRST is empty, and `StubRenderer` fills via `Frame::filled(2, 2, Rgb::black())`; in production the background color comes from `RendererSettings` (config), matching "configured black background". |
| 3 | Per frame render the complete currently visible text incl. FIFO eviction | ADHERENT | `render_and_write_frame` builds `RenderInput { text: chat_store.visible_text() }` every iteration. Test `run_rotates_new_messages_into_visible_text` asserts `["alice: Hello!", "alice: Hello!\nbob: Hi!", "alice: Hello!\nbob: Hi!"]`. Eviction semantics stay owned by `ChatStore` (FIFO capacity tests in `chat/store_tests.rs`; fixture capacity 26 ≫ 2 messages). |
| 4 | Write exactly one full RGB24 frame; short writes/broken pipes; never silently drop or truncate bytes | ADHERENT | Defense-in-depth size gate `write_frame_bytes` → `frame_has_expected_size` helper (single-section boolean rule) → `FrameSizeMismatch { expected, actual }` before any write. Byte delivery reuses Task-1 semantics untouched: `FfmpegProcess::write_frame` loops `stdin.write_all(frame)` and maps failures (incl. broken pipe) to `StreamingError::FrameWrite` (`src/streaming/ffmpeg.rs` lines 109–119); the pipeline adds zero byte-level logic (S10). Broken-pipe surfacing proven by test `run_propagates_sink_write_failure_without_finalizing`, which asserts the Display contains "frame output failed" and the Task-1 broken-pipe phrase "exited while receiving frames" (verified present in `format_frame_write_error`, `error.rs` line 147–153). |
| 5 | Renderer respects width/height/font/margins/line-height/colors; no wrapping/scrolling/game logic/screenshot | ADHERENT | `build_renderer` constructs `TextRenderer` from `RendererSettings::from_config(&config.renderer, &config.video)`; the pipeline only forwards visible text and full-frame byte slices — no render logic added. No wrapping/scrolling/screenshot/game code exists anywhere in the Task-2 diff (cumulative diff reviewed file-by-file). |
| 6 | FPS-paced with no unbounded drift/queue growth | ADHERENT | `PacingSchedule` anchored-incremental math exactly per S6: `next_gap` = `target.saturating_add(interval)`, floored to now when behind (whole backlog dropped, at most ONE catch-up frame, wrap-proof). Tests 3–4 assert steady-state `[40,40,40]` sleep determinism and backlog drop `next_gap(200) == 0` → `next_gap(200) == 40`. Queue growth bounded: drain feeds the capacity-bounded `ChatStore`; per R2 at most ONE message consumed per frame iteration. |
| 7 | Normal completion stops production → closes stdin → waits → surfaces failures; finish-once + Drop kill/reap on error paths | ADHERENT | `run` breaks at the stop-flag check (before any further drain/render), then `sink.finish()` consumed EXACTLY ONCE (by-value move → single stdin EOF, drain-thread joins, `wait()`, exit-0 AND non-empty file verified — Task-1 `finish` reused verbatim). On ANY loop error the sink is returned without `finish`; `FfmpegProcess::Drop` (Task-1 hardening, `ffmpeg.rs` lines 182–190) kills AND reaps the child — the single safety net, exactly per S9. Tests 8/11/12/13 assert `finish_called == false` on every error/early-stop path. "Do not claim valid MP4 merely because file exists" holds: `is_output_finalized` = exists AND non-empty AND exit 0 (`OutputFinalize` otherwise). |
| 8 | Ctrl+C minimal cross-platform via ctrlc crate with stop flag and documented limitations | ADHERENT | `ctrlc = "3"` (locked 3.5.2, lockfile verified incl. `nix`/`windows-sys` platform deps; no async runtime enters the tree — S2 holds). `install_interrupt_handler` flips an `Arc<AtomicBool>`-backed `StopFlag` (`SeqCst`); the loop polls it at iteration top. The 5-point verified-scope + limitations block from S7 is present verbatim as module docs in `src/app/pipeline/interrupt.rs` (docs lines 1–21), including the stuck-op/OS-tools limitation, ≤1-frame-interval shutdown latency, at-most-one-extra-frame, detached-background caveat, and the intentionally unbounded `finish()` wait (D10 deferral, not silently claimed). |

## 2. Plan adherence incl. Addendum R1–R4

- **R1 (no self re-exports)** — VERIFIED: `src/app/pipeline/mod.rs` (30 lines)
  declares modules + test modules only; zero `pub(crate) use` self re-exports.
  `runner.rs` uses the submodule-path imports (`crate::app::pipeline::…`) as
  R1's governing text requires. Task 3 owns any re-exports if needed.
- **R2 (at most ONE message per frame iteration)** — VERIFIED:
  `accept_next_message` uses `if let` (not the S5 `while let` drain loop);
  `source.rs` module docs state "consumes at most one due message per frame";
  the stale plural "drains messages" phrasing in the `FramePipeline` struct doc
  was corrected by commit `5623940` to "accepts at most one due message per
  frame iteration". Plan test 10 is the authority and passes with the one-per-
  iteration ordering (`alice` accepted before frame 1 renders — verified).
- **R3 (split renderer re-export block)** — VERIFIED:
  `src/renderer/mod.rs` re-exports `RenderInput, Renderer, RenderError,
  Frame, RendererSettings, TextRenderer` plus `#[cfg(test)] pub(crate) use
  settings::Rgb;` with the test-only comment — the Q3-amended form exactly.
- **R4 (`PipelinePacing::from_config` staged in the error-model commit)** —
  VERIFIED: `35db4d9` diff touches `mod.rs`, `error.rs`, `error_tests.rs`,
  **`pacing.rs` (+20)**, `sink.rs` — the exact deviation (file set of Step 4 +
  pacing.rs only). Its implementation matches plan §2.2 (checked
  `rgb24_byte_count`, `FrameByteOverrun`, delegates to `raw`).
- **Micro-decisions** — VERIFIED: the trivially-true `assert!(first >= 0)` on
  `u64` is deleted (clock test 5 has the single `assert!(second >= first)`);
  `mod pipeline;` landed unstaged in `app/mod.rs` only at the wiring commit
  (`47d48e6` includes `src/app/mod.rs | 1 +`), then doc-aligned by `5623940`.
- **File layout (S1)** — VERIFIED: all 13 planned files exist under
  `src/app/pipeline/` (mod, clock, clock_tests, pacing, pacing_tests,
  interrupt, interrupt_tests, source, error, error_tests, runner, runner_tests,
  test_helpers) plus the one-line `mod pipeline;` in `src/app/mod.rs`.
  No top-level `src/pipeline`.
- **FramePipeline<S> / PipelineParts / PipelineConstruction** — VERIFIED
  field-for-field against plan §2.5 (7 parts; construction param object with
  source/stop_flag/output_kind; ≤2-args rule honored via parameter objects).
- **`prepared_pipeline` composition order** — VERIFIED: pacing from_config →
  `resolve_output_path` → `FfmpegProcess::start` → `build_chat_store` →
  `build_renderer` → parts with fixed `SystemFrameClock` → `FramePipeline::new`.
- **FrameClock/PacingSchedule APIs and math** — VERIFIED exactly per S6:
  `NANOS_PER_SECOND`, `interval_nanos` with the fps==0 guard, `started`,
  `next_gap` (saturating_add + floor), `SystemFrameClock` with
  `u64::try_from(...).unwrap_or(u64::MAX)` only in the real clock (plan risk
  item honored — no `as` casts on elapsed time anywhere in pacing).
- **StopFlag mechanics** — VERIFIED: `Arc<AtomicBool>`, `SeqCst`, clonable,
  starts cleared, `request_stop` through `&self`; tests 6–7 cover both.
- **FramePipelineError variants and strings** — VERIFIED exactly per S8:
  5 variants; Display strings match the plan verbatim
  ("chat message stage failed: …", "frame rendering failed: …",
  "…overflows usize; the pipeline refused to start",
  "rendered frame holds {actual} bytes but the pipeline requires exactly
  {expected} (width*height*3)", "frame output failed: …"); `Error::source` maps
  Chat/Render/Sink → Some, geometric variants → None. Tests 14–15 pin the two
  string contracts.
- **Re-export blocks (Q3-amended)** — VERIFIED: `streaming/mod.rs` carries the
  four `pub(crate) use` lines + the widen note; `sink.rs` has
  `CompletedVideo::new` widened `pub(super)` → `pub(crate)` with the expanded
  doc comment (Step 4.2); `chat/mod.rs` carries the three re-exports.
- **main.rs staging migration (S11)** — VERIFIED: `src/main.rs` (15 lines)
  carries NO allow attributes and no staging comment (7 lines removed in
  `47d48e6`); content matches plan §2.6's final form. The relocated
  `#![allow(dead_code)]` + the exact retire sentence are present in
  `chat/mod.rs`, `renderer/mod.rs`, `streaming/mod.rs`, and
  `app/pipeline/mod.rs` — the FOUR Task-3-obligated attributes are in place and
  face-documented ("retires in TODO Task 3 when the demo mode…").
- **Deferred .gitkeep ledger** — CONFIRMED: Task 2 removed ONLY
  `src/streaming/.gitkeep` (commit `60102fc`, single-file diff). Current
  `git ls-files "*.gitkeep"` → `.kilo/plans/.gitkeep`, `credentials/.gitkeep`,
  `src/youtube/.gitkeep`. `logs/.gitkeep` absence is NOT a Task-2 change — it
  was removed pre-Phase-03 in `dcb873d` (verified via `git log -- logs/.gitkeep`).
  **List handed to Task 5 planning**: (a) replay this ledger; (b) decide/review
  `src/youtube/.gitkeep` (still tracked; the folder holds no code and
  `main.rs` declares no `mod youtube;` in this phase); (c) note
  `logs/.gitkeep` is already absent from tracking while
  `.agent/project-structure.md` line 56 still claims it is versioned — a docs
  staleness item for the Task-5 docs step (project-structure.md as a whole is
  stale at Phase-02 v0.5.0 wording, which Task 5 owns); (d)
  `.kilo/plans/.gitkeep` remains tracked, explicitly left for Task 5.

## 3. Out-of-scope violations (plan §5, HARD)

NONE. The cumulative Task-2 diff (`228fe37..5623940`) touches only:
Cargo.toml/Cargo.lock, `src/app/**` (pipeline + mod.rs), `src/chat/mod.rs`,
`src/renderer/mod.rs`, `src/streaming/{mod,sink}.rs` + `.gitkeep`,
`src/main.rs`. No CLI/`modes.rs` change (`auth`/`run` placeholders remain exit
3 — verified live in `modes.rs` and unchanged in the diff), no docs
files touched, no config schema change, no FFmpeg timeout mechanism, no audio,
no screenshot, no YouTube/OAuth touches, no `streaming`-internal logic change
beyond the sink.rs visibility widen + re-exports. Doc-alignment commit
`5623940` adjusted in-source documentation only (allowed: "inline code-file
comments in changed files are fair game").

## 4. Test tally (verified by counting, not by trusting the gate)

`rg -c "#\[test\]" src` sums to **121** exactly: streaming 35 (args 4, error 4,
video_output 10, utc 7), config 16, chat 16, renderer 19, app 21 (cli 12,
logging 7, error 2, modes 3, mod 6 — 30), pipeline **15** (pacing 4, clock 1,
interrupt 2, error 2, runner 6; `test_helpers.rs` contains zero `#[test]` —
helpers only). Gate logs confirm both runs: `running 121 tests` →
`121 passed; 0 failed; 0 ignored`.

- All 15 test names match plan §2.8 verbatim (exe: none — all present).
- Assertions verified against plan substance: sleeps log `[40, 40, 40]`
  (TEST_INTERVAL_NANOS) in test 9; result-path
  `videos/test-recording.mp4` in test 8; `finish_called == false` on all three
  error-path tests; Display-fragment assertions in test 12 match the actual
  `StreamingError`/`FramePipelineError` wording.
- FFmpeg-free: every fake (`RecordingSink`, `FailingWriteSink`,
  `StubRenderer`, `ScriptedSource`, `FakeClock`) is harness-local; no test
  spawns a process, no binary required.
- No real sleeps: `FakeClock` advances elapsed by recorded sleeps only;
  the `SystemFrameClock` test performs reads only; `interrupt_tests` never
  installs the handler (single-global-handler boundary documented, matching
  the Phase-01 subscriber-install precedent).

## 5. Implementer compliance verification

No tracked Task-2 review-fix (4.3) or simplification fix plan exists — the
plan Addendum records that 4.3 review passed with no fix plan required and
simplification required no plan, with the doc-alignment commit `5623940`
absorbing the residual wording corrections. Against the caller's summary of
the forced-compliance items, the concrete outcomes verified in code:

1. R2 wording corrections are real: `runner.rs` struct doc now reads "accepts
   at most one due message per frame iteration"; module doc reads "drains one
   due message" (singular, accurate); `source.rs` doc updated to match.
2. `app/mod.rs` module doc distinguishes pipeline status precisely: "complete
   but exercised only by tests so far; the `demo` mode of TODO Task 3 makes it
   reachable from `main`", while the exit-code contract paragraph is unchanged.
3. `#![allow(dead_code)]` staging + retire notes present in all four dead
   subtree roots (S11), each with the mandated sentence; main.rs attributes
   fully retired.
4. `CompletedVideo::new` widen + doc comment present (`sink.rs`).
5. `#[cfg(test)] Rgb` re-export with rationale comment (`renderer/mod.rs`).
6. `PipelinePacing::from_config` lives with the error model (R4),
   implementation already used by `prepared_pipeline`.
7. Size-gate helper extraction (`frame_has_expected_size`) and
   `request_stop_when_due` keep boolean conditions single-section.
8. Test 12 asserts both the app-level wrapper wording ("frame output failed")
   and the Task-1 broken-pipe phrase actually present in `error.rs`.
9. No `pub` beyond `pub(crate)`/intended `pub` (Task-1 surface) anywhere
   new; `FramePipeline` fields private; only justified aggregate fields are
   `pub(crate)` with in-code justification. Harmless in every case.

## 6. Rule compliance snapshot (.kilo/rules)

- **≤200-line files**: all 56 `src` files verified; max =
  `src/app/pipeline/test_helpers.rs` = **exactly 200** (legal, at the limit —
  see Proposed changes, item 1). Largest remaining: `chat/store_tests.rs` 198,
  `runner_tests.rs` 188.
- **≤50-line methods**: longest observed body `FramePipeline::run` ≈ 13 lines;
  all pipeline methods are small helpers. OK.
- **Depth ≤2**: loop body depth 1; `if let` arms depth 1; no triple nesting
  anywhere in the new files. OK.
- **≤2 params**: verified on every new fn (parameter objects
  `PipelineParts`/`PipelineConstruction` used where more inputs exist;
  `stop_after_writes(self, threshold, stop_flag)` = 2 args). OK.
- **No commented-out code**: pipeline files inspected in full — only
  explanatory comments, no dead code blocks. OK.
- **Private members**: field-level privacy fine; only `pub(crate)` crosses the
  module boundary (justification comments present on aggregates); Task-1's
  pre-existing `pub` surface untouched. OK.
- **Real newlines**: all read files show actual line structure; no literal
  `\n` artifacts. OK (CRLF watch item: no whole-file churn in the audited
  diffs).
- **Self-documenting code**: names follow the plan verbatim. Gitignore
  compliance — `.gitignore` untouched (videos/ rule intact at line 55); no
  videos/dist/log files staged in any Task-2 commit (stat review).

## 7. Final gate evidence

- Original evidence: `logs/checks/20261012T010348Z.log` — exit 0,
  `PASS fmt-check/check/test/clippy`, `ALL CHECKS PASSED`, `running 121 tests`,
  `121 passed; 0 failed`.
- Timing nuance resolved: that log finished ~1 minute before the doc-alignment
  commit `5623940` (22:05:24 local −03:00). To prove the FINAL committed tree,
  the gate was re-run via the standard Alpine VM invocation after this audit's
  file reads: **exit 0**, log `logs/checks/20261012T011002Z.log`,
  `PASS fmt-check exit=0`, `PASS check exit=0`, `PASS test exit=0`
  (`running 121 tests` → `121 passed; 0 failed`), `PASS clippy exit=0`,
  `ALL CHECKS PASSED` — on HEAD `5623940`. Both logs are now available; the
  later one is the authoritative final-tree gate.

## Findings (diffs reported; none blocking)

1. **Flag for Task 3 planning** — `src/app/pipeline/test_helpers.rs` sits at
   exactly the 200-line cap. Any addition (e.g. new Task-3 demo fakes) exceeds
   the rule. The Task-3 plan should either split the helpers file or inline new
   fakes into that task's test files. No change required now.
2. `test_helpers.rs` fakes deviate microscopically from the plan §2.7 sketch:
   `RecordingSink::fresh()` takes no stop-flag argument (the flag is attached
   afterwards via `stop_after_writes`, whose failing-write twin instead relies
   on no threshold), `ScriptedSource::schedule` takes `Vec<ChatMessage>` by
   value, and `SinkEvents` derives `Default`. **Acceptable deviation** — the
   plan's binding notes explicitly state the sketch "is a SPEC, not a locked
   typing" for `TestSetup`-level structure, and all three LOCKED items
   ((a) `Rc<RefCell>` shared records, (b) `PipelinePacing::raw
   (TEST_INTERVAL_NANOS, TEST_FRAME_BYTES)` in runner tests, (c) no real
   sleeps / no FFmpeg binary) are honored.
3. Plan §2.5's `run` sketch used `drain_messages` with a `while let` loop;
   final code implements R2's `accept_next_message` with `if let`. **Accepted
   deviation by ruling R2** (test 10 remains the authority and passes).
4. Plan §2.4/§2.7 minor placement differences (limitations block as module
   docs; `TEST_INTERVAL_NANOS`/`TEST_FRAME_BYTES` as top-of-file consts in
   `test_helpers.rs` rather than trailing). **Cosmetically equivalent,
   acceptable.**
5. `.agent/project-structure.md` is stale (still Phase-02 wording: "chat/
   renderer implemented, not yet wired", v0.5.0, "logs/ .gitkeep versioned",
   no `app/pipeline` entry, streaming called "placeholder"). **Not a Task-2
   rule violation** — the plan's §5 hard-out-of-scope forbids Task-2 edits to
   the structure file; it is already owed to the Task-5 docs step and is now
   itemized above.
6. The planning file `.kilo/plans/20261011-phase03-task2-frame-pipeline.md`
   itself is currently untracked in the working tree (prior task planning
   files were tracked via explicit docs commits). Tracking it is the
   caller/Planner's later docs step, not part of this sub-step; recorded for
   the ledger.
7. Mild terminology note: `runner.rs` module-doc head still uses the verb
   "drains one due message" (singular — R2-accurate). Consistent with the
   struct doc and `source.rs`. No action required.

## Proposed changes (only where needed)

- **None for Task 2.** The tree as committed (`5623940`) is accepted as
  adherent. Forward-flagged to Task 3/Task 5 planning:
  1. Task 3: `test_helpers.rs` is at its 200-line cap — plan the helper file's
     fate (split or inline) before adding demo-mode tests; ALL FOUR inner
     `#![allow(dead_code)]` attributes must retire and clippy `-D warnings`
     re-prove zero-allow cleanliness once `demo` reaches the pipeline (S11
     obligation).
  2. Task 3: add `FramePipelineError` → exit-code mapping in the demo mode
     handler (plan already reserves this; no current consumer maps it —
     intentionally deferred, documented in `error.rs`).
  3. Task 5: project-structure.md/context refresh (v0.6.0, pipeline module
     map, staging-allow retire notes after Task 3); replay-and-decide the
     `.gitkeep` ledger (`src/youtube/.gitkeep` candidate for review +
     `logs/.gitkeep` claim fix in the docs).

[PLAN-CHECK-END]
