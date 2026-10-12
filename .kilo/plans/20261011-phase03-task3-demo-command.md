# Implementation Plan — Phase 03, TODO Task 3: Dedicated `demo` Command (sub-step 4.1b)

- **Date**: 2026-10-11
- **Caller step**: TODO Task 3, sub-step 4.1b (Analysis & Planning; architector)
- **Branch**: `feat/phase03-ffmpeg-local-pipeline` (continues Tasks 1–2; HEAD `7f940af` at planning time)
- **Version**: `0.6.0` (already bumped in phase step 3 — NO further bump inside Task 3)
- **Global plan**: `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md` — D9 "static documented message set with deterministic schedule … schedule isolated for unit testing via an injectable seam … final path printed only after FFmpeg exits successfully" governs; this plan refines D1–D12, never reverses them.
- **Front-end (steps 4.1a / 4.5a)**: none (Rust backend).
- **Implementer target**: junior, 50% restriction — every choice below is locked; do not redesign, do not improvise additional allowances or refactor outside listed files.

---

## 1. Verified live baseline (read from the tree at planning time, not assumed)

- `src/app/cli.rs` (149 lines): commands `auth`/`run` only; `USAGE` const; `Mode { Auth, Run }`; usage messages `unknown command '{token}' (expected 'auth' or 'run')` and `expected a command ('auth' or 'run')`; `--config` before/after the command at most once; `--help` only before the command (stdout, exit 0).
- `src/app/cli_tests.rs` (129 lines, 12 tests): pins both usage messages verbatim — both change in this task.
- `src/app/mod.rs` (173 lines): funnel `parse → init logging → load+validate config → dispatch`; exactly three `info` events with whitelisted fields only (`command`, `config_path`, `exit_code`); `report_failure` = one stderr `error:` line; `SUCCESS_EXIT = 0`; `command_name`/`dispatch` match `Mode` exhaustively. Module doc states the pipeline is "complete but exercised only by tests so far; the `demo` mode of TODO Task 3 makes it reachable from `main`".
- `src/app/modes.rs`: `auth`/`run` placeholders exit 3 (`NOT_IMPLEMENTED_EXIT = 3`); UNTOUCHED by Task 3 (Task 4 owns `run`; `auth` untouched per TODO Task 4).
- `src/app/pipeline/runner.rs`: `FramePipeline<S: FrameSink>` with `PipelineParts { chat_store, renderer, sink, source, clock, pacing, stop_flag }` (fields `pub(crate)`); production constructor `prepared_pipeline(config, PipelineConstruction { source, stop_flag, output_kind })` → `PipelinePacing::from_config` → `resolve_output_path(kind)` → `FfmpegProcess::start` → chat store (`ChatStoreSettings::from_config`) → `TextRenderer` (`RendererSettings::from_config`) → fixed `SystemFrameClock` → `FramePipeline::new`. The loop: stop check at the top → `accept_next_message()` UNCONDITIONALLY (exactly one `next_message()` call per frame iteration, ruling R2) → render full `visible_text()` → size gate → one `sink.write_frame` → `PacingSchedule::next_gap` sleep → `sink.finish()` exactly once after breaking on the stop flag.
- `src/app/pipeline/source.rs`: `MessageSource { fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> }` — the demo schedule feeds through this seam; the error channel is reserved for TODO Task 4 (documented evolution point in the file).
- `src/app/pipeline/clock.rs`: `FrameClock` trait + `SystemFrameClock` (monotonic `Instant` anchor, real thread sleeps).
- `src/app/pipeline/pacing.rs`: pure `interval_nanos(fps: u32) -> u64` (`NANOS_PER_SECOND / fps`, `fps == 0` degrades to 1 s), `PacingSchedule`, `PipelinePacing`.
- `src/app/pipeline/interrupt.rs`: `install_interrupt_handler() -> Result<StopFlag, InterruptHandlerError>` (ctrlc crate); `StopFlag` is `Clone` (shared `Arc<AtomicBool>`); `InterruptHandlerError { reason: String }` has a private field and NO test constructor yet (Task 3 adds one, §7.4).
- `src/app/pipeline/error.rs`: `FramePipelineError { Chat, Render, FrameByteOverrun, FrameSizeMismatch, Sink }` with pinned Display strings ("chat message stage failed: …", "frame rendering failed: …", "frame output failed: …", etc.); module doc says the exit-code mapping belongs to "the mode command in TODO Task 3" — this task fulfills that promise.
- `src/app/pipeline/test_helpers.rs`: EXACTLY 200 lines (`FakeClock`, `StubRenderer`, `RecordingSink`, `FailingWriteSink`, `ScriptedSource`, `Rc<RefCell<SinkEvents>>`, `TEST_INTERVAL_NANOS = 40`, `TEST_FRAME_BYTES = 12`). MUST NOT be extended (Task-2 watch item); demo fakes go in NEW `#[cfg(test)]` files; demo test files MAY consume the existing helpers via `crate::app::pipeline::test_helpers::*` (items are `pub(crate)`).
- `src/streaming/video_output.rs`: `OutputKind::{Demo, Session}` (`Demo` prefix `demo-`, `Session` prefix `session-` reserved for the future production pipeline); `resolve_output_path(kind)` = executable dir → `videos/` (created on demand) → collision-safe `demo-YYYYMMDD-HHMMSS.mp4` beside the executable. `OutputKind::Session` is never constructed at runtime.
- `src/streaming/sink.rs`: `FrameSink { write_frame(&[u8]), finish(self) -> Result<CompletedVideo, StreamingError> }`; `CompletedVideo::path()`.
- `src/chat/`: `ChatMessage::new(&author, &text)` (normalized), `with_message_id(&str)`, `with_timestamp(Option<u64>)`; `ChatStore::new(ChatStoreSettings::from_config(config))`, `accept`, `visible_text` (joined `'\n'`), `is_empty`; log record `<epoch_seconds> <author>: <text>\n` appended FIRST; capacity = floor((height − top − bottom)/line_height) ≥ 1.
- `src/renderer/`: `Renderer` trait (`render(&mut self, &RenderInput) -> Result<Frame, RenderError>`), `TextRenderer::new(RendererSettings)`, `RendererSettings::from_config`; horizontal clipping at `x == width` is renderer-owned (brief §9.5).
- `src/config/`: full schema, no new fields allowed in this task (D10).
- `.gitignore` line 55 already carries `videos/` (Task 1 landed it) — nothing to add.
- `src/main.rs` (15 lines): `mod app; mod chat; mod config; mod renderer; mod streaming;` — no allow attributes; `main` → `app::run`. Untouched by Task 3.
- FOUR module-inner `#![allow(dead_code)]` blocks each carrying the "Staging allow: retires in TODO Task 3 when the demo mode …" sentence: `src/chat/mod.rs:15`, `src/renderer/mod.rs:14`, `src/streaming/mod.rs:24`, `src/app/pipeline/mod.rs:10`.
- Test baseline: 121 passing. `cargo clippy --locked -- -D warnings` (dev-checks, no `--all-targets`) evaluates dead_code on the NON-test build, so once the four allows retire every runtime-unreachable item becomes a hard error. Full exposure list pre-analyzed in §7 (exactly three items need dispositions: `ChatStore::is_empty`, `ChatMessage::with_timestamp`, `OutputKind::Session`).

## 2. Task-level decisions (T1–T9; refine D9, never reverse)

- **T1 — CLI extension**: `Mode::Demo` variant + third command word `demo` + one `USAGE` line, listed in the order `auth`, `demo`, `run`; that same order defines the tuple order in both usage-error messages. `--config`/`--help` rules are structurally untouched (command-agnostic in `parse`). §4.1–4.2.
- **T2 — Demo handler**: new module `src/app/demo/` (`mod.rs` + `schedule.rs` + `source.rs` plus test files). Public surface exactly `pub(super) fn run_demo(config: &AppConfig) -> u8`, wired from `dispatch`. Success returns 0 and prints the finalized output path via a user-facing `println!` strictly AFTER `finish()` returned `Ok` — deliberately NOT a tracing event: the tracing whitelist (`command`, `config_path`, `exit_code`) stays untouched; the path is user output, not a log field. Any failure reports exactly one stderr `error:` line and returns 1. §4.5.
- **T3 — Exit-code contract for demo**: minimal — NO new exit-code value. Mapping table:

  | Failure class | Reporter | Exit code |
  | --- | --- | --- |
  | Command-line usage errors | startup funnel (unchanged) | 2 |
  | Config / logging-init failures | startup funnel (unchanged) | 1 |
  | Demo: interrupt-handler install failure | `run_demo` → `DemoError` → stderr | 1 |
  | Demo: ANY `FramePipelineError` (chat/message stage, render, frame-size guards, FFmpeg spawn/write/wait/exit-status/finalization) | `run_demo` → `DemoError` → stderr | 1 |
  | Demo success | `run_demo` → `println!` final path | 0 |
  | `auth` / `run` placeholders (unchanged) | `modes.rs` | 3 |

  Rationale: today's stable classes are 1 = fatal failure after process start (config/log), 2 = usage, 3 = placeholder-not-implemented. A demo runtime failure is a fatal failure after start, so it belongs in class 1, reported identically (single stderr `error:` line naming the cause — the sub-error `Display` strings are already actionable; `DemoError` delegates verbatim, adding no wrapper). `demo` is never a placeholder, so it NEVER exits 3; `run`/`auth` keep exit 3.
- **T4 — Schedule seam (frame-indexed, clock-free)**: the schedule is expressed in FRAME indices, not timestamps. Because the pipeline loop guarantees exactly one `MessageSource::next_message` call per frame iteration (verified in `runner.rs`), `DemoMessageSource` counts its calls to know the current frame index — no second clock, no anchor divergence between pacing and schedule (a second `SystemFrameClock` instance inside the source would anchor later and drift; explicitly rejected). Determinism is verified by call sequences; `FakeClock` is needed only for the `clock` field of `PipelineParts` in composition tests. `ChatError` is returned `Ok` always (static schedule has no failure class). §4.2–4.3.
- **T5 — Demo timeline (fixed, documented math)**: 28 scheduled messages (exceeds the example-config derived capacity of 26, so FIFO eviction IS exercised in the shipped demo), 30-frame black lead-in, one message due at most every 15 frames (each stays visible ≥ 15 frames), 60-frame tail, automatic finish via the existing `StopFlag` seam (the source requests stop at its final drain call; the pipeline then breaks, finalizes the sink exactly once). Final rendered frame count 496 (≈ 16.5 s at the example 30 FPS). Off-by-one conventions are encoded in named helpers, never hardcoded in tests. §4.2.
- **T6 — Retirement commit**: delete ALL FOUR module-inner `#![allow(dead_code)]` blocks + their "Staging allow …" doc sentences in a dedicated commit AFTER the wiring commit; refresh those module docs to state reachability from `main` via the demo mode; keep clippy `-D warnings` green with exactly THREE targeted item-level allowances (§7). `src/youtube/` and its `.gitkeep` untouched.
- **T7 — Demo message metadata**: each demo message carries a deterministic message_id `demo-NN` (e.g. `demo-07`) through the existing `with_message_id` builder — giving the Phase-02 "future API layer" builder a real production consumer and proving the metadata-never-visible contract end-to-end (`Display` stays exactly `author: text`). NO timestamps attached (`timestamp: None`; the store stamps generation time as today).
- **T8 — Testable composition point**: NO new seam, NO refactor of Task-2 code. `FramePipeline::new(PipelineParts { … })` is already the sink-generic seam; handler-level integration tests assemble `PipelineParts` with `RecordingSink`/`StubRenderer`/`FakeClock` from `pipeline::test_helpers`, the REAL `DemoTimeline` (via the test-only `raw` constructor), and the real `DemoMessageSource` — a full demo run with no FFmpeg and no real sleeps. `prepared_pipeline` is exercised only in production; it is not unit-tested here (its parts are individually covered).
- **T9 — Out of scope**: no config field (D10), no docs files (Task 5), no release builds (Task 5), no real-FFmpeg smoke (Task 5), no dependency additions (§5), `modes.rs` untouched (Task 4).

## 3. Git actions

1. Work continues on `feat/phase03-ffmpeg-local-pipeline`. Verify `git status` before starting; ignore-untracked planning file(s) are committed in the final docs commit.
2. No rebase/merge/push inside this task (phase step 5 owns that). No `.gitignore` change.
3. Commit sequence in §6; every commit keeps the tree compile-clean (`commit 4` and the final tree MUST pass the standard gate).

## 4. File-by-file changes

### 4.1 `src/app/cli.rs` (~149 → ~165 lines)

Add below `const RUN_COMMAND: &str = "run";`

```rust
const DEMO_COMMAND: &str = "demo";
```

Extend the enum:

```rust
pub(super) enum Mode {
    Auth,
    Run,
    Demo,
}
```

Replace `USAGE` (alignment preserved: two leading spaces; description column unchanged from today's offset):

```rust
pub(super) const USAGE: &str = "\
usage: rust-youtube-streamer-service [--config <path>] <command>

commands:
  auth                 run the OAuth authorization flow (not implemented yet)
  demo                 run a local-only FFmpeg demo recording (no YouTube)
  run                  start the streaming runtime (not implemented yet)

options:
  --config <path>      configuration file to load (default: config/config.json)
  --help               print this help text";
```

Extend `resolved_mode` (match arms auth → run → demo, then the error) and BOTH command-list strings:

```rust
match token {
    AUTH_COMMAND => Ok(Mode::Auth),
    RUN_COMMAND => Ok(Mode::Run),
    DEMO_COMMAND => Ok(Mode::Demo),
    _ => Err(StartupError::Usage(format!(
        "unknown command '{token}' (expected 'auth', 'demo' or 'run')"
    ))),
}
```

```rust
None => Err(StartupError::Usage(
    "expected a command ('auth', 'demo' or 'run')".to_string(),
)),
```

Module doc: change "exactly two commands (`auth`, `run`)" to "exactly three commands (`auth`, `demo`, `run`)". No other change.

### 4.2 `src/app/demo/schedule.rs` (NEW; ≤110 lines)

```rust
//! The fixed, documented demo schedule: a static ordered set of chat
//! messages, each due at an exact frame index of the demo timeline. Pure
//! data + pure frame-index arithmetic: unit-testable without any clock,
//! sleep, or pipeline (TODO Task 3; plan §4.2). Deltas: the timeline leads
//! with LEAD_IN_FRAMES black frames, admits at most one message every
//! MESSAGE_GAP_FRAMES frames, holds a TAIL_FRAMES tail after the last
//! delivery, and carries more messages than the example config's derived
//! visible capacity so FIFO eviction is exercised in the shipped demo.

use crate::app::pipeline::pacing::interval_nanos;
use crate::chat::ChatMessage;
use crate::config::model::AppConfig;

pub(crate) const DEMO_MESSAGE_TOTAL: usize = 28;
pub(crate) const LEAD_IN_FRAMES: u64 = 30;
pub(crate) const MESSAGE_GAP_FRAMES: u64 = 15;
pub(crate) const TAIL_FRAMES: u64 = 60;
/// 1-based message index whose text is the long line that classes through
/// the renderer's horizontal clipping path (pixel-level assertions for
/// clipping live in the renderer test suite, brief §9.5).
pub(crate) const LONG_LINE_MESSAGE_INDEX: usize = 15;

const MESSAGE_AUTHORS: [&str; 4] = ["alice", "bob", "charlie", "dana"];
const TEXT_PREFIX: &str = "demo message";
const LONG_LINE_PIECE: &str = "0123456789";
const LONG_LINE_PIECES: usize = 25;
const MESSAGE_ID_PREFIX: &str = "demo-";

pub(crate) struct ScheduledMessage {
    pub(crate) due_frame_index: u64,
    pub(crate) message: ChatMessage,
}

pub(crate) struct DemoTimeline {
    interval_nanos: u64,
    deliveries: Vec<ScheduledMessage>,
}
```

`impl DemoTimeline` (locked):

- `pub(crate) fn for_config(app_config: &AppConfig) -> Self` — `interval_nanos(app_config.video.fps)` + `build_deliveries()`; the ONLY production constructor.
- `pub(crate) fn raw(interval_nanos: u64, deliveries: Vec<ScheduledMessage>) -> Self` — test-only constructor (cfg(test) gated), used by `mod_tests` to inject `TEST_INTERVAL_NANOS`.
- `pub(crate) fn due_at(&self, already_served: usize, frame_index: u64) -> Option<ChatMessage>` — `self.deliveries.get(already_served)` kept only when its `due_frame_index <= frame_index`; clones the message. Returns at most ONE message per call (the pacing loop's one-per-frame rule; no loop here).
- `pub(crate) fn final_frame_index(&self) -> u64` — `LEAD_IN_FRAMES + ((DEMO_MESSAGE_TOTAL as u64) - 1) * MESSAGE_GAP_FRAMES + TAIL_FRAMES` = 30 + 27·15 + 60 = **495**: the index of the LAST rendered frame (the last delivery's frame is 435; the tail renders frames 436..=495, keeping the last 26 visible messages on screen 60 further frames).
- `pub(crate) fn rendered_frame_count(&self) -> u64` — `final_frame_index() + 1` = **496**.
- `pub(crate) fn total_deliveries(&self) -> usize` — `self.deliveries.len()`.

Private builder:

- `fn build_deliveries() -> Vec<ScheduledMessage>` — for `index in 0..DEMO_MESSAGE_TOTAL`: `number = index + 1`; `author = MESSAGE_AUTHORS[index % 4]`; text = `"demo message {number:02}"` except when `number == LONG_LINE_MESSAGE_INDEX` → text = `LONG_LINE_PIECE.repeat(LONG_LINE_PIECES)` (250 chars — "0123456789" × 25; far past any realistic text budget of a 1920-px frame at the example font size); every message gets `.with_message_id(&format!("{MESSAGE_ID_PREFIX}{number:02}"))` (T7); `due_frame_index = LEAD_IN_FRAMES + (index as u64) * MESSAGE_GAP_FRAMES` (first due = frame 30; long-line message #15 due = frame 240; last = frame 435). Single-section guards via small named helpers (`fn is_long_line_slot(number: usize) -> bool`).

Documented derived math for the example config (fps 30, interval ≈ 33 333 333 ns): ≈1.0 s of empty black frames (frames 0–29); message 1 visible from ≈1.0 s; successive messages ≈0.5 s apart; message 26 evicted when message 27 is accepted; the last 26 messages stay on screen through the tail; total ≈496 frames ≈16.5 s.

### 4.3 `src/app/demo/source.rs` (NEW; ≤75 lines)

```rust
//! Delivers the fixed demo timeline through the existing MessageSource
//! seam. The pipeline loop calls next_message exactly once per frame
//! iteration, so counting calls IS frame counting — no clock of any kind
//! lives here, and the schedule can never drift from the pacing loop. The
//! cloned StopFlag is the timeline's END REQUEST through the pipeline's
//! existing shutdown seam: requesting it during the final drain call makes
//! the loop render the last frame, break at the next iteration top, and
//! finalize the sink exactly once.

use crate::app::demo::schedule::DemoTimeline;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::source::MessageSource;
use crate::chat::{ChatError, ChatMessage};

pub(crate) struct DemoMessageSource {
    timeline: DemoTimeline,
    served: usize,
    current_frame_index: u64,
    stop_flag: StopFlag,
}

impl DemoMessageSource {
    /// ≤2-arguments rule; every counter starts empty/cleared.
    pub(crate) fn new(timeline: DemoTimeline, stop_flag: StopFlag) -> Self {
        Self {
            timeline,
            served: 0,
            current_frame_index: 0,
            stop_flag,
        }
    }

    /// Consumer-side accounting for `due_at`.
    fn served_count(&self) -> usize {
        self.served
    }
}

impl MessageSource for DemoMessageSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> {
        let frame_index = self.current_frame_index;
        let is_end = frame_index >= self.timeline.final_frame_index();
        self.current_frame_index += 1;
        if is_end {
            self.stop_flag.request_stop();
            return Ok(None);
        }
        let due = self.timeline.due_at(self.served_count(), frame_index);
        if due.is_some() {
            self.served += 1;
        }
        Ok(due)
    }
}
```

LOCKED semantics (the exact committed form; no schematic residue):

- Stop condition (locked): `is_end` is computed from the RAW (pre-increment) `frame_index` — the index of the frame THIS drain call feeds. Fire condition `frame_index >= final_frame_index()` (495): the drain call belonging to frame 495 requests the stop; frame 495 still renders (the render happens later in the same iteration), and the NEXT iteration breaks at the loop's top stop check → total rendered = `495 + 1 = 496` = `rendered_frame_count()` ✓ (frames 0..=495).
- Delivery condition (locked): `due_at(served, frame_index)` uses the RAW frame index; on non-stop paths it runs 0..=494; the LAST due slot falls at frame 435 (< 495, so delivery is never crowded out by the end request).
- Declaration detail: `DemoMessageSource` does NOT keep a separate `served`-style isolated accessor beyond the shown `served_count()` private helper (single-source accounting; `served` stays private).

The source can never fail: the static schedule has no failure class, so `Ok` is always returned; the `MessageSource` error channel is reserved for TODO Task 4 (documented evolution point in `source.rs`). Depth ≤2 everywhere, ≤2-arg constructor, all helpers ≤ the 50-line method cap, single-section boolean conditions only.

### 4.4 `src/app/demo/mod.rs` (NEW; ≤125 lines incl. test-module decls)

```rust
//! The `demo` mode handler (TODO Task 3): a local-only demonstration that
//! never contacts YouTube, never attempts OAuth, never creates a broadcast,
//! and never connects to a chat API. It installs the app's single Ctrl+C
//! interrupt handler, builds the demo message source, and runs the shared
//! chat → renderer → FFmpeg pipeline against a `demo-`-prefixed MP4 beside
//! the executable. Exit contract: success 0; ANY demo-origin failure —
//! interrupt-install failure or any FramePipelineError — is reported by
//! exactly one stderr `error:` line and exits 1 (the startup-failure class;
//! see app::error). `demo` never exits 3.

mod schedule;
mod source;

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
#[cfg(test)]
#[path = "schedule_tests.rs"]
mod schedule_tests;
#[cfg(test)]
#[path = "source_tests.rs"]
mod source_tests;

use std::fmt;

use crate::app::demo::schedule::DemoTimeline;
use crate::app::demo::source::DemoMessageSource;
use crate::app::error::STARTUP_FAILURE_EXIT;
use crate::app::pipeline::error::FramePipelineError;
use crate::app::pipeline::interrupt::{install_interrupt_handler, InterruptHandlerError, StopFlag};
use crate::app::pipeline::runner::{prepared_pipeline, PipelineConstruction};
use crate::config::AppConfig;
use crate::streaming::{CompletedVideo, OutputKind};

/// Exit code returned by the demo mode on success.
const DEMO_SUCCESS_EXIT: u8 = 0;

/// All fatal demo-origin failures; every variant exits code 1.
#[derive(Debug)]
pub(super) enum DemoError {
    /// The Ctrl+C interrupt handler could not be installed.
    Interrupt(InterruptHandlerError),
    /// Building or running the frame pipeline failed (chat message stage,
    /// rendering, setup guards, or the FFmpeg sink lifecycle).
    Pipeline(FramePipelineError),
}

impl DemoError {
    /// Scheduled exit-code mapping: both variants are fatal failures after
    /// process start → the startup-failure class (code 1); `demo` never
    /// returns a placeholder code.
    fn exit_code(&self) -> u8 {
        STARTUP_FAILURE_EXIT
    }
}

/// Verbatim delegation keeps the single stderr line naming the cause: the
/// inner Display strings are already actionable ("unable to install the
/// Ctrl+C interrupt handler: …", "chat message stage failed: …", "frame
/// rendering failed: …", "frame output failed: …").
impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Interrupt(error) => write!(formatter, "{error}"),
            Self::Pipeline(error) => write!(formatter, "{error}"),
        }
    }
}

/// Runs the demo to completion: 0 on a verified finalized recording, 1 with
/// one stderr `error:` line on any failure.
pub(super) fn run_demo(app_config: &AppConfig) -> u8 {
    match demo_outcome(app_config) {
        Ok(()) => DEMO_SUCCESS_EXIT,
        Err(error) => report_demo_failure(&error),
    }
}

/// Ordered demo lifecycle: interrupt handler FIRST (a Ctrl+C during the
/// remaining setup is already observed), then the shared pipeline against
/// the `OutputKind::Demo` output, then the success-only print.
fn demo_outcome(app_config: &AppConfig) -> Result<(), DemoError> {
    let stop_flag = install_interrupt_handler().map_err(DemoError::Interrupt)?;
    let construction = PipelineConstruction {
        source: build_demo_source(app_config, stop_flag.clone()),
        stop_flag,
        output_kind: OutputKind::Demo,
    };
    let pipeline = prepared_pipeline(app_config, construction).map_err(DemoError::Pipeline)?;
    let completed = pipeline.run().map_err(DemoError::Pipeline)?;
    print_final_path(&completed);
    Ok(())
}

/// User-facing completion output, printed ONLY after
/// `FramePipeline::run` returned a verified `CompletedVideo` (FFmpeg exited
/// 0 and the MP4 was confirmed present and non-empty on disk). Standard
/// println!, deliberately NOT a tracing event: the tracing whitelist
/// (command, config_path, exit_code) stays untouched and no configuration
/// or path value ever enters a log field.
fn print_final_path(completed: &CompletedVideo) {
    println!("demo recording finalized: {}", completed.path().display());
}

fn report_demo_failure(error: &DemoError) -> u8 {
    eprintln!("error: {error}");
    error.exit_code()
}
```

Where `build_demo_source(app_config: &AppConfig, stop_flag: StopFlag) -> DemoMessageSource` = `DemoMessageSource::new(DemoTimeline::for_config(app_config), stop_flag)`.

### 4.5 `src/app/mod.rs` (wiring; minimal diff)

- Insert `mod demo;` between `mod cli;` and `mod error;`.
- `command_name`: add arm `cli::Mode::Demo => "demo",`.
- `dispatch`: add arm `cli::Mode::Demo => demo::run_demo(app_config),`.
- Module doc: extend the exit-code sentence to "…, 2 = command-line usage error, 3 = selected mode not implemented yet (`auth` stays a placeholder); demo-origin fatal failures — interrupt-install failure or any `FramePipelineError` — report through the demo handler and also exit 1", and replace the "exercised only by tests so far" tail with the reached state ("the `demo` mode makes the pipeline reachable from `main`").
- No changes to `string_tokens`/`evaluate`/`print_help`/`report_failure`/whitelisted events. File stays ≤185 lines.

### 4.6 `src/app/cli_tests.rs` (net +3 tests; 2 expectation updates)

- UPDATE the literal in `rejects_unknown_command` to `"unknown command 'status' (expected 'auth', 'demo' or 'run')"`.
- UPDATE the literals in `rejects_empty_arguments` AND `rejects_config_option_without_command` to `"expected a command ('auth', 'demo' or 'run')"`.
- ADD `parses_demo_alone_with_default_path` (mode `Mode::Demo`, config path `DEFAULT_CONFIG_PATH`).
- ADD `accepts_config_option_around_demo` (before/after: paths `a.json`, `b.json`).
- ADD `usage_text_advertises_the_demo_command` (`USAGE` contains the exact demo command line string from §4.1).
- All other 9 tests untouched and still green; `parses_auth_and_run_alone_with_default_path` unchanged.
- Result 12 → 15 tests.

### 4.7 Retirement changes (commit 5; exact)

- `src/chat/mod.rs` — delete line 15 `#![allow(dead_code)];` and the doc sentence "Staging allow: retires in TODO Task 3 …; clippy `-D warnings` guards everything else." Replace with "Reachable from `main` through the `demo` mode (TODO Task 3)."
- `src/renderer/mod.rs` — delete line 14 allow + its doc sentence, same replacement; KEEP the `#[path = "renderer.rs"] mod contract;` mapping (A9) and the `#[cfg(test)] pub(crate) use settings::Rgb;` re-export with its rationale comment, unchanged.
- `src/streaming/mod.rs` — delete line 24 allow + the staging sentence in the module docs ("Staging allow: retires in TODO Task 3 …"); replace with the demo-reachability sentence; KEEP the `CompletedVideo::new` widen note verbatim.
- `src/app/pipeline/mod.rs` — delete lines 10–11 (the `#![allow(dead_code)]` + its trailing comment `// Deliberate staging: …`), reword; NO self re-exports (R1 holds — consumers use full `crate::app::pipeline::…` paths).
- `src/chat/store.rs` — targeted allow above `pub fn is_empty` (§7.2).
- `src/chat/model.rs` — targeted allow above `pub fn with_timestamp` (§7.3).
- `src/streaming/video_output.rs` — targeted allow above the `Session` variant (§7.4).

## 5. Commands (Alpine VM MCP; call `alpine-vm_vm_status` FIRST, then `alpine-vm_vm_run_command`)

- Standard gate (after commit 4 and on the final tree after commit 5):
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  EXPECTED: exit 0; PASS lines `fmt-check`, `check`, `test`, `clippy`; `running 136 tests` → `136 passed; 0 failed; 0 ignored`; summary `ALL CHECKS PASSED`.
- Optional intermediate spot checks (after commits 1 and 2) using the standard compose pattern:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked`
- NOT run in this task (Task 5): `sh scripts/build-linux.sh`, `sh scripts/build-windows.sh`, any real-FFmpeg probe, any native execution. NO source edits via MCP beyond the repo (MCP is read-only for the implementer's env checks).

## 6. Commit plan (locked messages; one logical change each)

1. `feat: accept the demo command in the command line parser` — `src/app/cli.rs` + `src/app/cli_tests.rs`.
2. `feat: add the fixed demo message timeline` — `src/app/demo/schedule.rs` + `src/app/demo/schedule_tests.rs` (+ `src/app/demo/mod.rs` with ONLY the `mod schedule;`/test-module declarations needed to compile).
3. `feat: deliver demo messages through the message source seam` — `src/app/demo/source.rs` + `src/app/demo/source_tests.rs`.
4. `feat: run the local demo pipeline from the demo mode` — `src/app/demo/mod.rs` (handler), `src/app/demo/mod_tests.rs`, `src/app/mod.rs` (wiring + doc), `src/app/pipeline/interrupt.rs` (test constructor only).
5. `refactor: retire the dead-code staging allows on the wired pipeline` — four module docs + three targeted allowances + doc refreshes (§4.7).
6. `docs: track task 3 planning documents` — this plan file (and later the adherence report) under `.kilo/plans/`.

Gate runs per §5 (after commit 4 and after the final source commit 5).

## 7. Retirement, allowances, and test tally (hard requirements)

### 7.1 What the retirement commit removes

The FOUR module-inner staging attributes (T6) — the exact promise recorded in their face comments. Nothing else is allowed to change in that commit except §4.7's listed files.

### 7.2 `ChatStore::is_empty` — KEEP with targeted allow

`src/chat/store.rs`, directly above `pub fn is_empty`:

```rust
// Retains the Phase-02 public contract surface; no consumer exists until a
// later phase (deliberate, single-item allowance).
#[allow(dead_code)]
pub fn is_empty(&self) -> bool {
```

Rationale: deleting it would break the documented Phase-02 API and churn `store_tests.rs` (lines 85, 189) for no benefit. This is NOT one of the four staging blocks being retired.

### 7.3 `ChatMessage::with_timestamp` — KEEP with targeted allow

`src/chat/model.rs`, directly above `pub fn with_timestamp`:

```rust
// Reserved for the YouTube chat API layer (later phase); the demo attaches
// deterministic message IDs but no timestamps. Deleting it would delete the
// store's own-timestamp log-record coverage.
#[allow(dead_code)]
pub fn with_timestamp(mut self, seconds: Option<u64>) -> Self {
```

### 7.4 `OutputKind::Session` — KEEP with targeted allow

`src/streaming/video_output.rs`, directly above the `Session` variant:

```rust
// `session-` is reserved for the future production pipeline (Task-1
// contract); the demo is the only runtime OutputKind in this phase.
#[allow(dead_code)]
Session,
```

Clippy evaluates the non-test build, so the existing `video_output_tests.rs` usage does NOT suppress the lint — the allowance is required.

### 7.5 `ChatMessage::with_message_id` — NO attribute

It gains a real production consumer: `build_deliveries` attaches `demo-NN` IDs (T7). Verifying this is part of the post-retirement clippy run.

### 7.6 Test-constructor addition (required for `mod_tests`; commit 4)

`src/app/pipeline/interrupt.rs` (82 → ~87 lines; production behavior untouched):

```rust
#[cfg(test)]
impl InterruptHandlerError {
    /// Test-only construction for Display/exit-code mapping tests; real
    /// failures always arrive through install_interrupt_handler.
    pub(crate) fn test_failure(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}
```

`interrupt_tests` still never install a real handler (single-global-handler boundary, unchanged).

### 7.7 Final test tally (locked)

| Area | Before | After |
| --- | --- | --- |
| `src/app/cli_tests.rs` | 12 | 15 |
| `src/app/demo/schedule_tests.rs` (new) | 0 | 6 |
| `src/app/demo/source_tests.rs` (new) | 0 | 6 |
| `src/app/demo/mod_tests.rs` (new) | 0 | 3 |
| All other suites (streaming 35, config 16, chat 16, renderer 19, app 21−12→9, pipeline 15 — unchanged) | 103 | 103 |
| **Total** | **121** | **136** |

`cargo test` expectation at every gate in this task: `running 136 tests` → `136 passed; 0 failed` (task starts from 121; the ±0 diffs above are cumulative). NO additions to interrupt/pacing/clock/error/runner test suites.

## 8. Demo test files (exact content contracts)

### 8.1 `src/app/demo/mod_tests.rs` (NEW; 3 tests)

Test-local helper (NOT in `pipeline/test_helpers.rs`): `fn temp_store()` copying the documented pattern from `runner_tests.rs` (tempdir + `valid_config()` + `chat.log_file` overridden + `ChatStoreSettings::from_config` + `ChatStore::new`).

1. `demo_origin_failures_map_to_the_startup_failure_code` — `DemoError::Interrupt(InterruptHandlerError::test_failure("already taken")).exit_code() == 1` and `DemoError::Pipeline(FramePipelineError::Sink(<StreamingError::WaitFailed…>)).exit_code() == 1` (`StreamingError` variants are constructible per `streaming/error.rs`); also assert both `Display` delegations produce exactly the inner text (substring check on the pinned interrupt phrase "unable to install the Ctrl+C interrupt handler" and pipeline phrase "frame output failed").
2. `demo_timeline_drives_the_frame_pipeline_to_full_completion` — THE handler-level integration test: the REAL production schedule via the `#[cfg(test)]`-only helper `scheduled_messages_for_tests()` in `schedule.rs` (exposes the production `build_deliveries()` output) wrapped in `DemoTimeline::raw(TEST_INTERVAL_NANOS, scheduled_messages_for_tests())`; composition: `PipelineParts { chat_store: temp_store, renderer: Box::new(StubRenderer::fresh()), sink: RecordingSink::fresh(), source: Box::new(DemoMessageSource::new(DemoTimeline::raw(TEST_INTERVAL_NANOS, scheduled_messages_for_tests()), stop_flag)), clock: Box::new(FakeClock::fresh()), pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES), stop_flag }`; `pipeline.run()` succeeds; assertions: `writes.len() == timeline.rendered_frame_count()` (496 via the helper — never a literal 496 in the assertion); `finish_called == true` with the fake sink's documented path `videos/test-recording.mp4`; the FIRST renderer input is `""` (black lead-in, empty chat); the LAST renderer input has exactly 26 `'\n'`-separated lines and contains `charlie: demo message 03` and `demo message 28`, and does NOT contain `demo message 01` or `demo message 02` (FIFO eviction: 28 deliveries vs the 26-line capacity evict the two oldest); at least two consecutive visible inputs are identical (messages persist across frames).
3. `demo_timeline_finishing_needs_no_real_time` — additional composition with `stop_before` variant NOT needed; folded into test 2's evidence (FakeClock advances only via recorded sleeps; the run completes without any wall-clock dependency). NOT a separate test — the tally's 3 stands.

Note: `run_demo` itself (interrupt handler install, `prepared_pipeline`, println) is NOT unit-tested — it touches the global handler, the process environment, and real stdout; composition test 2 IS the handler-level behavioral test. Recording exact stdout of `print_final_path` is a Task-5 manual verification item (documented there, not claimed here).

### 8.2 `src/app/demo/schedule_tests.rs` (NEW; 6 tests — ALL pure, no clock)

1. `fixed_message_set_is_completely_ordered` — build via the `#[cfg(test)]`-exposed `build_deliveries()`: `len == 28`; due frames exactly `30, 45, …, 435` (sorted, strictly increasing); author cycle `alice, bob, charlie, dana` repeating; text `demo message 01..28` in order.
2. `long_line_message_carries_the_pinned_text_and_hidden_metadata` — delivery #15 (`LONG_LINE_MESSAGE_INDEX`): author `charlie` (index (15−1)%4 = 2), text `"0123456789"` × 25, message_id `demo-15`, and `format!("{message}")` == `"charlie: 0123456789…" .chars 250` — i.e. `Display` contains ONLY "author: text" (no `demo-15`).
3. `due_frames_respect_the_one_per_frame_budget` — consecutive due frames differ by exactly `MESSAGE_GAP_FRAMES` ⇒ at most one due per frame by construction; every message stays visible ≥ `MESSAGE_GAP_FRAMES` frames (the next delivery arrives `MESSAGE_GAP_FRAMES` later) and the last stays visible through the 60-frame tail.
4. `final_frame_index_and_rendered_count_match_the_planning_formula` — `final_frame_index() == 495` and `rendered_frame_count() == 496` on the `for_config`-derived timeline built from `valid_config()` (fps 30 — index math, NOT nanoseconds; the interval only enters `interval_nanos(fps)`).
5. `every_due_frame_sits_at_or_before_the_final_frame` — last due (435) ≤ final_frame_index (495); strict slack of 60 tail frames asserted via `final_frame_index() - last_due == TAIL_FRAMES`.
6. `message_set_exceeds_the_default_visible_capacity` — `total_deliveries() (28) > crate::chat::derive_visible_capacity(config.video, config.renderer)?` for the `valid_config()` fixture → 26 ⇒ the shipped demo exercises FIFO eviction.

### 8.3 `src/app/demo/source_tests.rs` (NEW; 6 tests — clock-free call sequences)

1. `no_message_is_due_during_the_black_lead_in` — frames 0..29: 30 calls all `Ok(None)`; `served` stays 0.
2. `first_message_is_due_at_its_slot` — call at frame 30 (index 30): `Ok(Some("alice: demo message 01"))`; the NEXT call (frame 31): `Ok(None)` (slot consumed).
3. `at_most_one_message_is_delivered_per_call` — a call at a slot TWO frames past due STILL returns the NEXT message singly (delivery order never bunches; two calls in the same slot return distinct messages in order).
4. `timeline_delivers_all_messages_in_order_without_any_clock` — step frame indices one per call through 28 deliveries: exact order `demo message 01` … `demo message 28`; no sleeps, no wall clock anywhere in the module.
5. `timeline_requests_the_stop_at_its_last_frame` — drive to the drain call for frame index `final_frame_index()` + 1: `Ok(None)` immediately followed by a `StopFlag::is_requested()` check on the clone that reads `true`.
6. `timeline_ends_after_its_final_delivered_message` — at frame indices between the LAST due (435) and `final_frame_index()` − 1 (494) the source returns `Ok(None)` WITHOUT requesting the stop (the tail frames render the still-visible messages; the store keeps them visible).

## 9. Out of scope (HARD — do not touch)

- `run`/`auth` behaviors — `modes.rs` completely untouched (Task 4 owns `run`; `auth` untouched per TODO).
- Config schema/validation/example config (D10) — no new field.
- Docs files (README, docs/, `.agent/project-structure.md`, `.agent/project-info/context.md`, completion report) — Task 5.
- Release builds (build-linux/build-windows) and real-FFmpeg smoke — Task 5.
- Audio, screenshots, stdin/interactive input, wrapping/scrolling — standing TODO constraints.
- `pipeline/test_helpers.rs` (at its 200-line ceiling) — no additions.
- `src/main.rs` — no change.
- New dependencies — none (D12; the schedule reuses `pipeline::pacing::interval_nanos`, everything else is std + existing crates).
- Tracing whitelist — unchanged; no new field, no new event.

## 10. Task-3 acceptance-criteria mapping

| TODO acceptance criterion | Mechanism |
| --- | --- |
| `demo` accepted by the CLI and documented in help/usage | §4.1 USAGE line; §4.6 tests |
| predefined messages, deterministic schedule, no user input | `DemoTimeline` constants + pure builder; no stdin anywhere in `src/` |
| never contacts YouTube / OAuth / broadcast / chat | no YouTube import anywhere; `MessageSource` is a local trait; `demo/mod.rs` module doc states the boundary |
| FFmpeg receives complete RGB24 frames at the configured dimensions/FPS | unchanged `FramePipeline` / Task-1 writer path (covered by existing tests) |
| frames continue during empty intervals; messages rendered from ChatStore; eviction by bounded FIFO capacity | unchanged loop behavior + 28 > 26 schedule; §8.1 test 2, §8.2 test 6 |
| MP4 finalized successfully before the command reports completion | success print strictly after `run()` returned `Ok(CompletedVideo)`; no success output on any error path |
| output under `videos/` beside the executable, `demo-` prefix, collision-safe, cwd-independent | `resolve_output_path(OutputKind::Demo)` inside `prepared_pipeline` (Task-1 code unchanged) |
| schedule testable without real-time sleeps | frame-indexed pure data + clock-free call-counting source + `#[cfg(test)] raw` constructor + FakeClock only in composition tests |
| exit-code contract preserved; demo never exits 3 | §2 T3 mapping + `DemoError::exit_code` pinned in tests |
| `--config`/`--help` ordering rules unchanged | parser structure untouched except the third command word; full existing cli regression suite stays green |

## 11. Risks / watch items

- **dead_code exposure after retirement** — pre-analyzed (§7.1–7.5); the three targeted allowances are declared here, not left to implementer judgment. If clippy surfaces an UNLISTED dead item at commit 5, STOP and return to the caller (do not improvise a fourth allowance).
- **Second-clock divergence** — rejected by design (T4): the source carries NO clock; call counting equals frame counting because `accept_next_message` is unconditionally called once per iteration.
- **Stop timing off-by-one** — governed by named helpers (`final_frame_index`, `rendered_frame_count`) and pinned in tests via builders, not literals.
- **Stop-flag double request** — harmless (`AtomicBool::store` idempotent).
- **Ctrl+C during demo setup/teardown** — the existing interrupt limitations doc in `interrupt.rs` applies unchanged (shutdown latency ≤ one frame interval; stuck-op limitation documented in Task 2; nothing new claimed).
- **`println!` evidence** — the success-path print is verified by manual run in Task 5 (recorded there); unit tests verify the code ORDER (print is inside the success arm only) and the exit mapping, never stdout capture.
- **Newline integrity** — real newlines in every written file (repo-host CRLF incident rule); `.gitattributes` already covers `*.sh`/`Dockerfile`; Rust sources unaffected.

[PLAN-CHECK-END]
