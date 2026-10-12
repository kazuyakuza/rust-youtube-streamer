# Implementation Plan — Phase 03, TODO Task 2: Continuous Local Frame Pipeline

- **TODO source**: `.agent/todos/20261011/20261011-todo-1.md`, task heading
  `### 2. Continuous Local Frame Pipeline` (line 43), the `## Current
  Repository Baseline` standing constraints (lines 9–28), `## Constraints and
  Out of Scope` (lines 85–96), and the Task‑2‑relevant `## Acceptance Criteria`
  (lines 98–111).
- **Global plan**: `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md`
  (decisions D1–D12; this plan refines, never reverses them).
- **Task 1 adherence report** (verified facts used here):
  `.kilo/plans/20261011-phase03-task1-adherence.md`.
- **Branch**: `feat/phase03-ffmpeg-local-pipeline` (already checked out, working
  tree clean — verified live; no further branch setup in this task).
- **Version**: `0.6.0` already (set at Phase 03 step 3). No version change in
  Task 2; only a lockfile refresh for the new dependency.
- **Front-end task**: no (steps 4.1a / 4.5a skipped).
- **Implementer profile**: JUNIOR developer under 50% restriction — every
  structural choice below is already made; do not redesign anything.

---

## 0. Verified baseline (read from the live repository, not assumed)

Branch state at planning time (`git status`: clean; branch:
`feat/phase03-ffmpeg-local-pipeline`; suite: 106 tests green per gate log
`logs/checks/20261011T232229Z.log`).

### 0.1 `src/streaming` — exact public API from Task 1 (all verified live)

| Item | Signature (as written) | File | Notes |
| --- | --- | --- | --- |
| `FrameSink` trait | `fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError>` | `sink.rs` | Loops `write_all`; maps pipe failures to `StreamingError::FrameWrite`. |
| `FrameSink` trait | `fn finish(self) -> Result<CompletedVideo, StreamingError>` | `sink.rs` | **By-value `self` receiver — the trait is NOT object-safe (E0038).** This shapes decision S4. |
| `CompletedVideo` | `pub fn path(&self) -> &Path`; constructor `pub(super) fn new(output_path: PathBuf)` | `sink.rs` | Constructor is `pub(super)` — widened in Step 5 for the app's test double. |
| `FfmpegProcess` | `pub fn start(config: &AppConfig, output_path: PathBuf) -> Result<Self, StreamingError>` | `ffmpeg.rs` | `impl FrameSink for FfmpegProcess` exists; `Drop` kills **and** reaps the child. `finish` sets `stdin = None` (single stdin EOF), joins drain threads, `wait()`s, requires exit 0 AND non-empty output file, else `ExitFailure`/`OutputFinalize`. |
| `StreamingError` | 9 variants: `UnsupportedPixelFormat`, `ExecutableLocation`, `OutputDirectoryCreate`, `OutputNameExhausted`, `Spawn`, `FrameWrite`, `WaitFailed`, `ExitFailure`, `OutputFinalize` | `error.rs` | Hand-written `Display` + `source()`. No new variants are needed by Task 2. |
| `OutputKind` | `pub(crate) enum `{ `Demo`, `Session` }`, `filename_prefix()` | `video_output.rs` | Passed as value into `resolve_output_path`. |
| `resolve_output_path` | `pub(crate) fn resolve_output_path(kind: OutputKind) -> Result<PathBuf, StreamingError>` | `video_output.rs` | Beside-exe `videos/`, created on demand, collision-safe name. |
| `RecordingNameSpec` | pure naming generator | `video_output.rs` | NOT re-exported; the pipeline never needs it. |
| `build_arguments` | `pub(crate)` in `args.rs` | — | NOT re-exported; `FfmpegProcess::start` is the only caller. |

Module visibility fact: `src/streaming/mod.rs` declares private submodules
(`mod args; mod error; …`) and **no re-exports yet**; the doc comment says the
cross-module `pub use` re-exports "return when the first external consumer
lands". Task 2 IS that first external consumer → re-exports are added in this
task (Step 5, exact list in §2.6).

### 0.2 `src/chat` — exact API

- `ChatStoreSettings::from_config(config: &AppConfig) -> Result<Self, ChatError>`
  (derives visible capacity from video geometry).
- `ChatStore::new(settings) -> Result<Self, ChatError>` (opens append/create log
  eagerly; fresh queue), `accept(&mut self, ChatMessage) -> Result<(), ChatError>`
  (log-first, rejects on append failure, FIFO eviction beyond capacity),
  `visible_text(&self) -> String` (`author: text` lines joined by `'\n'`),
  `is_empty(&self) -> bool`.
- `ChatMessage::new(author: &str, text: &str)` (normalizes one line), optional
  `.with_message_id` / `.with_timestamp`; `Display` = `author: text`.
- `ChatError` is a `pub` enum with `pub`-visible variant fields —
  `ChatError::LogAppend { path: PathBuf, source: io::Error }` and
  `ChatError::Capacity { .. }` are **constructible from any module** (verified —
  enables the pipeline's source-failure test without touching `ChatStore`
  internals). Do NOT modify the chat module's logic in this task.

### 0.3 `src/renderer` — exact API

- `Renderer` trait: `fn render(&mut self, input: &RenderInput) -> Result<Frame,
  RenderError>` — lives in `src/renderer/renderer.rs`, exposed only as
  `contract` (`mod.rs`: `#[path = "renderer.rs"] mod contract;`). Not
  object-safety blocking: the only receiver is `&mut self` → `Box<dyn Renderer>`
  is legal.
- `TextRenderer::new(settings: RendererSettings) -> Result<Self, RenderError>`
  (font parsed once at construction; `render` draws complete visible text,
  background fill otherwise).
- `RendererSettings::from_config(config: &RendererConfig, video: &VideoConfig)
  -> Result<Self, RenderError>`; its fields are `pub(super)` — deliberately NOT
  widened; the pipeline derives byte counts from `config.video` itself.
- `Frame`: `pixels(&self) -> &[u8]` (exactly `width*height*3` bytes),
  `width()`, `height()`; `blend_at` is `pub(super)`.
- `RenderError` variants: `Geometry { width, height }`, `FrameSize { width,
  height }`, `Layout`, `InvalidColor`, `FontOpen`, `FontParse` — all with
  pub-visible fields (`Geometry` constructible in tests).

### 0.4 `src/app` — exact state

- `app::run(args) -> ExitCode` → `evaluate` → `cli::parse` → `start_mode`
  (logging → config load → `dispatch`) → `modes::{auth,run}` placeholders exit 3.
- `StartupError { Usage, Configuration, LogInit }`, exit codes 2 / 1 / 1.
- `src/main.rs` (verified live, 22 lines): `mod app; #[allow(dead_code)] mod
  chat; mod config; #[allow(dead_code)] mod renderer; #[allow(dead_code)]
  mod streaming;` with a 2-line comment above `mod streaming;`. These three
  attributes are the Task‑1 staging, retired per decision S11.
- CLI surface is unchanged in this task (`auth` | `run` | `--config` | `--help`;
  exit codes 0/1/2/3 fixed). **Task 2 adds NO CLI changes** (Task 3 does `demo`).

### 0.5 Other verified facts

- `src/streaming/.gitkeep` EXISTS (tracked since the Phase 00 scaffold; the
  folder now has real code) → deleted in Step 7 (decision S12).
- `.gitignore` already carries the narrow `videos/` rule (Task 1, line 55). No
  ignore changes in Task 2.
- `config::model` is `pub(crate) mod` — the pipeline uses
  `crate::config::model::{AppConfig, VideoConfig}` directly; `AppConfig` is
  already re-exported by `config::mod`.
- `config::valid_config` test fixture (verified):
  1920×1080, `fps = 30`, `line_height 40`, top/bottom margins 20 → derived
  visible capacity = 26; `fps = 30` → interval = `33_333_333 ns`. Test-only alias
  `#[cfg(test)] pub(crate) use test_fixtures::valid_config;` is reachable from
  app-level tests (precedent: `app/modes.rs` tests).
- Container toolchain: pinned `rust:1.82`, edition 2021; artifacts in the named
  volume; all checks through the standard Compose invocation.

---

## 1. Locked design decisions (S1–S12; refine D1–D12, never reverse)

### S1 — Wiring location: `src/app/pipeline/` (new subdirectory module of `app`)

The frame-pipeline orchestration lives in the application layer because it is
exactly the place where `chat`, `renderer`, and `streaming` meet (brief §2.2 and
§4: `app` "orchestrates startup, task execution, error handling, and shutdown";
`streaming` must stay free of chat-message and renderer semantics per D1 and
TODO Task‑1 bullet "Define a small interface for sending complete frames").
`streaming` receives opaque `&[u8]` frame bytes only; `chat`/`renderer` are
touched only by the app-level pipeline.

Boundary rules encoded:
- `src/app/pipeline/**` may `use crate::chat::*`, `crate::renderer::*`,
  `crate::streaming::*`, `crate::config::model::*`, and `std`.
- `src/streaming/**` is NOT touched except (a) adding re-exports in
  `streaming/mod.rs`, (b) widening `CompletedVideo::new` to `pub(crate)`. Zero
  logic changes inside `streaming`.
- `src/chat/**` and `src/renderer/**` are NOT touched except (a) re-exports in
  their `mod.rs`, (b) the dead-code staging attribute move (S11). Zero logic
  changes.
- No new module under `streaming` may import chat/renderer ever.

New files (all ≤200 lines; `src/app/` is the only home — `pipeline` must not be
a top-level `src/pipeline` because the project-structure rule keeps source under
`src/` and app is the orchestration layer):

```
src/app/pipeline/mod.rs          — module docs, staging allow (S11), declarations, re-exports
src/app/pipeline/pacing.rs       — PacingSchedule (pure), PipelinePacing, interval derivation
src/app/pipeline/pacing_tests.rs — 4 tests
src/app/pipeline/clock.rs        — FrameClock trait + SystemFrameClock
src/app/pipeline/clock_tests.rs  — 1 test
src/app/pipeline/interrupt.rs    — StopFlag, InterruptHandlerError, install_interrupt_handler
src/app/pipeline/interrupt_tests.rs — 2 tests
src/app/pipeline/source.rs       — MessageSource trait
src/app/pipeline/error.rs        — FramePipelineError
src/app/pipeline/error_tests.rs  — 2 tests
src/app/pipeline/runner.rs       — FramePipeline<S>, PipelineParts<S>, PipelineConstruction, prepared_pipeline
src/app/pipeline/runner_tests.rs — 6 tests
src/app/pipeline/test_helpers.rs — #[cfg(test)] fakes (single file shared by runner_tests)
```

New submodule declaration in `src/app/mod.rs`: `mod pipeline;` (one line, no
attribute — the staging attribute is INSIDE `pipeline/mod.rs`, see S11).

### S2 — Synchronous std-thread design; NO Tokio (refines D6)

The pipeline is a single synchronous thread paced by sleeps; Ctrl+C is served by
`ctrlc`'s own dedicated handler thread. Tokio would add an async dependency for
zero benefit in a synchronous local-file loop; brief §3.2 explicitly forbids
introducing async complexity where components can stay simple and synchronous.
**Decision: std only.** Justification recorded here satisfies D6's requirement
that the choice be explicit.

### S3 — Renderer form: `Box<dyn Renderer>` (chosen over `TextRenderer` directly)

Justification: brief §1.2 goal 10 demands clean interfaces so future renderers
can be introduced without rewriting the streaming/orchestration infrastructure;
the `Renderer` trait (`&mut self` receiver only) is object-safe; a stub renderer
is the natural test seam (runner tests use `StubRenderer`). Per-frame vtable cost
is negligible next to rasterization. `TextRenderer` remains the production
implementation and is constructed by the convenience constructor
(`prepared_pipeline`, §2.5) and boxed; tests supply their own boxed fake.

### S4 — Sink form: one struct-level generic parameter `S: FrameSink`
(chosen over `Box<dyn FrameSink>`)

`FrameSink::finish(self)` takes `self` **by value**, which makes the trait
non-object-safe (E0038) — `Box<dyn FrameSink>` will not compile. Two options
existed:

- (a) Amend the Task‑1 trait to `finish(self: Box<Self>)` — REJECTED: avoids
  editing reviewed Task‑1 code; also forces `Box` ownership shuffling in
  `FfmpegProcess::finish` call semantics review for no benefit.
- (b) **CHOSEN**: `FramePipeline<S: FrameSink>` owns the sink concretely
  (static dispatch). Call sites never name `S` (type inference resolves it from
  the parts struct); Task 3's demo passes a concrete `FfmpegProcess`; tests pass
  fakes; a future RTMPS sink is a compile-time type swap (TODO Task 4 bullet:
  "Clearly separate local-file output from future RTMPS output").

`Renderer`, `MessageSource`, `FrameClock` ARE object-safe (no by-value self
methods) and are held as `Box<dyn …>` — those two seams survive test
substitution and future live listeners cleanly.

### S5 — Message source seam: trait `MessageSource` (exact type + call point)

```rust
// src/app/pipeline/source.rs
use crate::chat::{ChatError, ChatMessage};

/// Supplies chat messages into the pipeline. The pacing loop calls
/// `next_message` exactly at the TOP of each frame iteration (after the stop
/// check, before rendering): the demo command (TODO Task 3) feeds a
/// deterministic schedule; a future production run mode feeds live chat
/// messages without changing this loop (TODO Task 4 bullet). A source may
/// return multiple messages across successive calls within one iteration;
/// `ChatStore`'s bounded visible queue keeps any unbounded arrival from
/// accumulating unbounded state.
pub(crate) trait MessageSource {
    /// Returns the next message due now, or `None` when nothing is due.
    /// The only failure class available in this phase is the chat-domain
    /// error; TODO Task 4 may widen this error type when a real network
    /// listener exists (documented evolution point).
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError>;
}
```

A closure seam was considered and rejected: a named trait gives Task 3's
schedule and Task 4's listener stable, self-documenting impls and keeps the
runner generic-free on this seam. Call point (exact, in `FramePipeline::run`):
`self.drain_messages()?` as the first action of every iteration after the
stop-flag check — `drain_messages` loops `while let Some(message) =
self.source.next_message()?` and passes EACH message through
`ChatStore::accept` (the same path as future real chat; log-first contract
preserved).

### S6 — Pacing: pure anchored incremental schedule + injectable clock

Drift model: **anchored incremental target time** relative to a monotonic
anchor. The target advances by exactly one interval per emitted frame and is
floored to "now" whenever the loop has fallen a whole interval behind — the
entire backlog is dropped, catch-up is bounded to at most ONE immediate
(gap‑0) frame, and the loop can never burst or accumulate unbounded drift.
Frame production is never "as fast as the CPU allows": every iteration after
the first is followed by a `sleep` of ≥ 0 ns computed by the schedule (and in
the steady state exactly one interval).

```rust
// src/app/pipeline/pacing.rs
pub(crate) const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// Frame interval for a validated config FPS: fixed-point truncation to whole
/// nanoseconds. `fps == 0` cannot come from the validated config loader; the
/// guard keeps the helper total (no panics) and degrades to one frame per
/// second instead of dividing by zero.
pub(crate) fn interval_nanos(fps: u32) -> u64 {
    if fps == 0 {
        return NANOS_PER_SECOND;
    }
    NANOS_PER_SECOND / u64::from(fps)
}

/// Anchored incremental schedule (pure; unit-testable without a clock).
pub(crate) struct PacingSchedule {
    interval_nanos: u64,
    target_nanos: u64,
}

impl PacingSchedule {
    /// Anchors the schedule: the first frame is emitted immediately, the
    /// first sleep target sits one interval ahead of `start_nanos`.
    pub(crate) fn started(interval_nanos: u64, start_nanos: u64) -> Self {
        Self { interval_nanos, target_nanos: start_nanos }
    }

    /// Called once AFTER each emitted frame. Advances the target by one
    /// interval, floors it to `now_nanos` when the loop is behind (dropping
    /// the whole backlog), and returns the gap the clock must sleep: at
    /// most one immediate catch-up frame per wake, no burst, no drift
    /// accumulation. Wrap-around-proof via `saturating_add` (u64 ns ≈ 584 y).
    pub(crate) fn next_gap(&mut self, now_nanos: u64) -> u64 {
        self.target_nanos = self.target_nanos.saturating_add(self.interval_nanos);
        if self.target_nanos <= now_nanos {
            self.target_nanos = now_nanos;
        }
        self.target_nanos - now_nanos
    }
}
```

```rust
// src/app/pipeline/clock.rs
use std::thread;
use std::time::{Duration, Instant};

/// Injectable time seam for pacing: production sleeps a real thread; tests
/// manipulate deterministic time. Monotonic reads only (no wall clock) so a
/// system clock jump can never distort pacing.
pub(crate) trait FrameClock {
    /// Monotonic nanoseconds since the clock's own anchor.
    fn now_nanos(&self) -> u64;
    /// Sleeps `nanos`; a value of 0 must return promptly.
    fn sleep_nanos(&self, nanos: u64);
}

/// Real implementation: one process-local monotonic anchor.
pub(crate) struct SystemFrameClock {
    start: Instant,
}

impl FrameClock for SystemFrameClock {
    fn now_nanos(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
    fn sleep_nanos(&self, nanos: u64) {
        thread::sleep(Duration::from_nanos(nanos));
    }
}
```

`SystemFrameClock { start: Instant::now() }` with a plain `new()` constructor
(`Instant` is monotonic, so no wall-clock dependency).

```rust
// also in pacing.rs — derived setup parameters (pipeline pacing + geometry)
pub(crate) struct PipelinePacing {
    interval_nanos: u64,
    frame_byte_count: usize,
}

impl PipelinePacing {
    /// Derives the frame interval from the validated FPS and the expected
    /// RGB24 byte count `width * height * 3` from validated video geometry.
    /// Overflow (checked u32 math) is a typed pipeline error; the config
    /// validator makes it unreachable through `load_from_path`.
    pub(crate) fn from_config(video: &VideoConfig) -> Result<Self, FramePipelineError>;
    /// Direct constructor for tests and thin pipelines: no validation.
    pub(crate) fn raw(interval_nanos: u64, frame_byte_count: usize) -> Self;
    pub(crate) fn interval_nanos(&self) -> u64;
    pub(crate) fn frame_byte_count(&self) -> usize;
}
```

### S7 — Interrupt (Ctrl+C): the `ctrlc` crate, D7 satisfied

**Mechanism**: `ctrlc = "3"` (new dependency, D12-compliant rationale below).
The handler flips an `Arc<AtomicBool>` shared through a `StopFlag` clone into
the pacing loop, which polls it at the TOP of every frame iteration. Shutdown
then executes exactly: stop accepting/rendering → `sink.finish()` (single stdin
EOF + bounded waits on drain threads + child wait) → report. Build/install
sits in `src/app/pipeline/interrupt.rs`; the mode command (Task 3) is the
caller.

```rust
// src/app/pipeline/interrupt.rs
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared shutdown request. `Clone` shares ONE `Arc<AtomicBool>` between the
/// Ctrl+C handler and the pipeline owner. Starts cleared.
#[derive(Clone, Debug)]
pub(crate) struct StopFlag {
    stopped: Arc<AtomicBool>,
}

impl StopFlag {
    pub(crate) fn new() -> Self {
        Self { stopped: Arc::new(AtomicBool::new(false)) }
    }
    /// Handler-side mutation; `AtomicBool` allows mutation through `&self`.
    pub(crate) fn request_stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
    pub(crate) fn is_requested(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}

/// Fatal failure of the ctrlc installation, typed (no panics); carries only
/// a display reason, never the crate error type.
#[derive(Debug)]
pub(crate) struct InterruptHandlerError {
    reason: String,
}
impl fmt::Display for InterruptHandlerError { … }
impl std::error::Error for InterruptHandlerError { … }

/// Installs the application's single Ctrl+C handler (app-level, exactly
/// once) and returns the flag the handler flips. Called by the mode command
/// before the pipeline runs.
pub(crate) fn install_interrupt_handler() -> Result<StopFlag, InterruptHandlerError> {
    let stop_flag = StopFlag::new();
    let handler_flag = stop_flag.clone();
    ctrlc::set_handler(move || handler_flag.request_stop())
        .map_err(|error| InterruptHandlerError { reason: error.to_string() })?;
    Ok(stop_flag)
}
```

**Dependency rationale (D7/D12) with verified evidence** (fetched during
planning; implementer must re-verify the version at execution time):
- docs.rs — https://docs.rs/ctrlc/latest/ctrlc/ — ctrlc **3.5.2** (2026‑09‑13),
  MIT OR Apache‑2.0: "Cross platform handling of Ctrl-C signals… On Unix,
  this corresponds to a `SIGINT` signal. On windows, `Ctrl+C` corresponds to
  `CTRL_C_EVENT` or `CTRL_BREAK_EVENT`." → Windows + Linux support with one
  crate, exactly the TODO's "minimal approach compatible with Windows and
  Linux".
- Same page: "Setting a handler will start a new dedicated signal handling
  thread where we execute the handler each time we receive a `Ctrl+C`
  signal. There can only be one handler, you would typically set one at the
  start of your program." → matches our single install + async flag flip.
  `try_set_handler` exists if a second registration must fail loudly — NOT
  used (we install once, app-level).
- Maintenance: actively maintained (release present in the current year);
  dependencies are platform-native crates (`nix` on Unix, `windows-sys` on
  Windows) and **no async runtime** — no Tokio enters the tree.
- Project policy: crates.io-pinned by the tracked lockfile; MSRV compatibility
  with rust:1.82 verified by the container `cargo check` in Step 1; if the
  latest ctrlc needs a newer rustc, pin the newest compatible version via
  `cargo update -p ctrlc --precise <version>` (tempfile 3.14 precedent from
  Phase 01) and record it.

**Verified cross-platform scope + documented limitations (encode as comments in
`interrupt.rs` and repeat in the Task-5 docs):**
1. Installing the handler REPLACES the default Ctrl+C termination on both
   platforms: pressing Ctrl+C while the process runs never force-kills it;
   the handler only sets the flag. Repeated Ctrl+C is equally inert. If the
   pipeline is stuck inside a long blocking op (a stdin write to a hung
   FFmpeg, or the unbounded `finish()` wait), the operator must terminate the
   process with OS tools. **Documented limitation — never claimed otherwise.**
2. Ctrl+C while the loop is blocked in a pacing sleep: the dedicated handler
   thread sets the flag asynchronously; the main thread observes it after the
   current sleep returns — shutdown latency is at most one frame interval.
3. Ctrl+C while rendering/writing a frame: observed at the next iteration top
   → the current frame completes, then shut down. At most one extra frame.
4. In Docker/Linux the container's foreground process receives SIGINT
   normally (interactive runs); in detached background processes no console
   signal exists — documented, no claims made about daemonized execution.
5. Unbounded `finish()` wait: FFmpeg's exit wait is intentionally NOT given a
   timeout in this phase (Task‑1 contract: no timeout; brief §14.3's
   bounded-wait + configured-timeout hardening needs a config field under
   D10 — DEFERRED with a TODO/plan note, not silently claimed).

### S8 — Error model: one new app-level type reusing Task‑1 semantics

New file `src/app/pipeline/error.rs`:

```rust
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::chat::ChatError;
use crate::renderer::RenderError;
use crate::streaming::StreamingError;

/// All fatal failures of the application-level frame pipeline. Streaming
/// failures reuse Task 1's typed `StreamingError` set verbatim (no new
/// streaming semantics); one variant per pipeline stage keeps reporting
/// actionable. Mapped to exit codes by the mode command in TODO Task 3.
#[derive(Debug)]
pub(crate) enum FramePipelineError {
    /// A message stage failure (`ChatStore::accept` rejection or source
    /// failure).
    Chat(ChatError),
    /// The renderer refused a frame.
    Render(RenderError),
    /// Setup-time defense: `width*height*3` overflows usize. Unreachable
    /// through the validated config loader.
    FrameByteOverrun { width: u32, height: u32 },
    /// Per-frame defense-in-depth: the rendered frame is not exactly
    /// `width*height*3` bytes (renderer contract violation never reported
    /// silently).
    FrameSizeMismatch { expected: usize, actual: usize },
    /// Streaming failures (spawn, frame write incl. broken pipe, wait,
    /// exit status, output finalization) reuse `StreamingError` as-is.
    Sink(StreamingError),
}
```

Display contracts (exact strings; `source()` maps `Chat`/`Render`/`Sink` to
`Some`):
- `Chat(e)` → `"chat message stage failed: {e}"`
- `Render(e)` → `"frame rendering failed: {e}"`
- `FrameByteOverrun` → `"frame byte count for {width}x{height} overflows usize; the pipeline refused to start"`
- `FrameSizeMismatch` → `"rendered frame holds {actual} bytes but the pipeline requires exactly {expected} (width*height*3)"`
- `Sink(e)` → `"frame output failed: {e}"`

Guideline #13 justification for the two new variants: the pipeline is the
process boundary between independent modules; assert-style typed checks here
prevent a malformed frame from reaching FFmpeg. NOTE: exit-code mapping does
NOT happen in Task 2 (no CLI change allowed) — `FramePipelineError` is
`Display`-outfitted so Task 3 maps it (expected: `error:` line + exit 1).

### S9 — Completion sequence and sink/FFmpeg-process ownership

- The pipeline NEVER names `FfmpegProcess`. It owns one `S: FrameSink`
  (constructed by the caller — Task 3 demo or tests — via
  `FfmpegProcess::start(config, resolve_output_path(kind)?)`). This keeps
  `streaming`'s process management independent of chat/render details (D1).
- Normal completion (`run` loop exit — stop requested):
  1. loop breaks BEFORE any further drain/render (stop observed at iteration top),
  2. `self.sink.finish()` consumed EXACTLY ONCE (moves the sink; stdin dropped
     → one EOF; drains joined; child awaited; exit + non-empty file verified),
  3. `Ok(CompletedVideo)` returned. Interruption via Ctrl+C after at least one
     frame is a SUCCESS (`CompletedVideo`): a finalized, valid (truncated)
     MP4 — FFmpeg exited 0. Never report success on anything less (Task‑1
     `finish` guarantees verified finalization).
- Failure on ANY loop error (`FramePipelineError`): return early WITHOUT
  calling `finish`. The boxed/concrete sink (an `FfmpegProcess` in production)
  then drops → `FfmpegProcess::Drop` (Task‑1 hardening) kills and reaps the
  child — the single, already-defined safety net. No second code path.
- This satisfies TODO bullet: "normal completion stops frame production,
  closes FFmpeg stdin, waits for process completion, and surfaces any FFmpeg
  failure" — `finish` returns `ExitFailure`/`OutputFinalize`/`WaitFailed`
  typed errors from Task 1 unchanged (reused via `FramePipelineError::Sink`).
- Unbounded wait limitation (see S7 item 5) documented, not solved.

### S10 — Exact byte writes: full reuse of Task‑1's `write_all` semantics

Per frame: `ChatStore::visible_text()` → `RenderInput { text }` →
`renderer.render(&input)` → defensive size check → `sink.write_frame(
frame.pixels())`. The pipeline adds ZERO byte-level logic:
`FfmpegProcess::write_frame` already loops short writes (`write_all`) and maps
broken pipes to `StreamingError::FrameWrite` — reused, not duplicated. The
expected byte count is derived at setup from `config.video` (`u32` checked math
→ `PipelinePacing::from_config`), NOT from `RendererSettings` fields (which
stay `pub(super)`).

### S11 — `#[allow(dead_code)]` retirement (main.rs) — REACHABILITY ANALYSIS ENCODED

Fact (verified semantics of rustc's `dead_code` lint in a BINARY crate): items
unreachable from `main` are dead, transitively — an item used only by dead code
is itself dead, and allow attributes only silence lints for items defined in
their own module subtree. Between this task and TODO Task 3 (the `demo`
command = first CLI consumer of the pipeline), the ENTIRE chain `pipeline →
chat → renderer → streaming` is unreachable from `main`, so clippy
`-D warnings` cannot pass with zero attributes even with fully wired code.

**Encoded retirement therefore moves the staging exactly one step deeper,
keeping the letter of the constraint:**

1. DELETE the three attributes and the staging comment from `src/main.rs`
   (the file becomes `mod app; mod chat; mod config; mod renderer; mod
   streaming;` + entry point — the attrs in main.rs are gone).
2. The retired coverage is RELOCATED into the dead subtree roots, each with a
   retire note the SAME three modules previously carried in main.rs:
   - `src/chat/mod.rs`, `src/renderer/mod.rs`, `src/streaming/mod.rs`: add
     one line `#![allow(dead_code)]` (module-inner attribute; covers the
     whole subtree the old main.rs attribute covered) plus one in-block doc
     sentence: "Staging allow: retires in TODO Task 3 when the demo mode
     (the pipeline's CLI consumer) makes this module reachable from `main`;
     clippy `-D warnings` guards everything else."
   - `src/app/pipeline/mod.rs`: the same inner attribute + the same retire
     note (the pipeline itself awaits its CLI consumer).
3. Update the statements in those module docs that currently say "…the
   `#[allow(dead_code)]` on `mod chat;` in `main.rs` retires at that Phase 03
   app-wiring step" to the new exact sentence (see §3 Step 5 file edits).
4. **Task 3 obligation** (forward-pinned, executed in that task's plan):
   when `demo` in `modes.rs` calls the pipeline from a live path, ALL FOUR
   inner attributes are removed and clippy `-D warnings` re-proves zero-allow
   cleanliness.

Alternatives explicitly rejected (recorded for adherence): wiring the local
pipeline into `run` mode (violates D8/TODO Task 4: `run` must fail clearly
before streaming), restructuring to lib+bin (scope explosion; would nullify
dead_code protection), keeping the attributes in main.rs until Task 3
(contradicts the mandated retirement).

### S12 — `src/streaming/.gitkeep`

The folder now contains real code; the placeholder marker is obsolete. Execute
`git rm src/streaming/.gitkeep` (tracked file) and commit
`chore: remove streaming folder placeholder marker`. Verify no other `src/**`
`.gitkeep` remains (`git ls-files "*.gitkeep"` → expect only `logs/.gitkeep`
and `credentials/.gitkeep`). Do NOT touch `logs/`, `credentials/`, `fonts/`,
`config/` markers.

---

## 2. Target file specifications (exact names, signatures, boundaries)

### 2.1 `src/app/pipeline/mod.rs` (≤40 lines)

```
//! module docs (6–10 lines): purpose = application-level frame pipeline
//! wiring (chat → renderer → streaming at app level, brief §2.2); boundary
//! rules (S1); D6 synchronous pacing; S7 Ctrl+C; S11 retire note; scope
//! authority = TODO Task 2 + this plan.
#![allow(dead_code)] // Deliberate staging: retires in TODO Task 3 when the
                     // demo mode makes this module reachable from `main`.
mod clock;
mod error;
mod interrupt;
mod pacing;
mod runner;
mod source;

#[cfg(test)]
mod clock_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod interrupt_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod test_helpers;

pub(crate) use clock::{FrameClock, SystemFrameClock};
pub(crate) use error::FramePipelineError;
pub(crate) use interrupt::{InterruptHandlerError, StopFlag, install_interrupt_handler};
pub(crate) use pacing::{NANOS_PER_SECOND, PipelinePacing, PacingSchedule, interval_nanos};
pub(crate) use runner::{FramePipeline, PipelineConstruction, PipelineParts, prepared_pipeline};
pub(crate) use source::MessageSource;
```

### 2.2 `src/app/pipeline/pacing.rs`

Contents exactly as S6 (constants, `interval_nanos`, `PacingSchedule`,
`PipelinePacing`). `PipelinePacing::from_config` implementation:

```rust
impl PipelinePacing {
    pub(crate) fn from_config(video: &VideoConfig) -> Result<Self, FramePipelineError> {
        let width = video.width;
        let height = video.height;
        let byte_count = rgb24_byte_count(width, height)
            .ok_or(FramePipelineError::FrameByteOverrun { width, height })?;
        Ok(Self::raw(interval_nanos(video.fps), byte_count))
    }
}

/// Checked `width * height * 3` for validated u32 geometry.
fn rgb24_byte_count(width: u32, height: u32) -> Option<usize> {
    usize::checked_mul(width as usize, height as usize).and_then(|pixels| pixels.checked_mul(3))
}
```

Accessors plain `pub(crate) fn interval_nanos(&self) -> u64 { self.interval_nanos }`
and `frame_byte_count(&self) -> usize { self.frame_byte_count }`.

### 2.3 `src/app/pipeline/clock.rs` — exactly S6's trait + `SystemFrameClock`.

### 2.4 `src/app/pipeline/interrupt.rs` — exactly S7 (with the 5-point
limitations block as module docs).

### 2.5 `src/app/pipeline/runner.rs` (≤200 lines; kept small by helper fns)

```rust
use crate::app::pipeline::clock::FrameClock;
use crate::app::pipeline::interrupt::StopFlag;
use crate::app::pipeline::pacing::{PipelinePacing, PacingSchedule};
use crate::app::pipeline::source::MessageSource;
use crate::chat::{ChatStore, ChatStoreSettings};
use crate::config::model::{AppConfig, VideoConfig};
use crate::renderer::{Frame, RenderInput, Renderer, RendererSettings, TextRenderer};
use crate::streaming::{CompletedVideo, FfmpegProcess, FrameSink, OutputKind, StreamingError, resolve_output_path};

/// Assembled parts for one frame pipeline. Callers (TODO Task 3 demo mode,
/// tests) populate the struct literal directly; fields are `pub(crate)`
/// because this crate-internal aggregate's single job is field collection
/// (keeps `FramePipeline::new` within the max-2-arguments rule).
pub(crate) struct PipelineParts<S> {
    pub(crate) chat_store: ChatStore,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) sink: S,
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) clock: Box<dyn FrameClock>,
    pub(crate) pacing: PipelinePacing,
    pub(crate) stop_flag: StopFlag,
}

/// Cell types for `prepared_pipeline` (production convenience constructor;
/// ≤2-arguments rule via parameter object).
pub(crate) struct PipelineConstruction {
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) stop_flag: StopFlag,
    pub(crate) output_kind: OutputKind,
}

/// The continuous frame pipeline: drains messages, renders the complete
/// visible text, writes one exact RGB24 frame per paced iteration, and
/// finalizes the sink on stop. Never names `FfmpegProcess` (S9).
pub(crate) struct FramePipeline<S> {
    chat_store: ChatStore,
    renderer: Box<dyn Renderer>,
    sink: S,
    source: Box<dyn MessageSource>,
    clock: Box<dyn FrameClock>,
    pacing: PipelinePacing,
    stop_flag: StopFlag,
}

impl<S: FrameSink> FramePipeline<S> {
    pub(crate) fn new(parts: PipelineParts<S>) -> Self {
        Self {
            chat_store: parts.chat_store,
            renderer: parts.renderer,
            sink: parts.sink,
            source: parts.source,
            clock: parts.clock,
            pacing: parts.pacing,
            stop_flag: parts.stop_flag,
        }
    }

    /// The paced loop. STOP LATENCY: at most one frame interval.
    pub(crate) fn run(mut self) -> Result<CompletedVideo, FramePipelineError> {
        let mut schedule =
            PacingSchedule::started(self.pacing.interval_nanos(), self.clock.now_nanos());
        loop {
            if self.stop_flag.is_requested() {
                break;
            }
            self.drain_messages()?;
            self.render_and_write_frame()?;
            let gap_nanos = schedule.next_gap(self.clock.now_nanos());
            self.clock.sleep_nanos(gap_nanos);
        }
        let sink = self.sink;
        sink.finish().map_err(FramePipelineError::Sink)
    }

    /// Message seam call point (S5): top of every iteration, before render.
    fn drain_messages(&mut self) -> Result<(), FramePipelineError> {
        while let Some(message) = self.source.next_message()? {
            self.chat_store.accept(message).map_err(FramePipelineError::Chat)?;
        }
        Ok(())
    }

    fn render_and_write_frame(&mut self) -> Result<(), FramePipelineError> {
        let input = RenderInput { text: self.chat_store.visible_text() };
        let frame = self.renderer.render(&input).map_err(FramePipelineError::Render)?;
        self.write_frame_bytes(frame)
    }

    /// Defense-in-depth size gate (S8), then Task‑1's exact-byte `write_all`
    /// semantics via the sink (S10).
    fn write_frame_bytes(&mut self, frame: Frame) -> Result<(), FramePipelineError> {
        let expected = self.pacing.frame_byte_count();
        if !frame_has_expected_size(&frame, expected) {
            return Err(FramePipelineError::FrameSizeMismatch {
                expected,
                actual: frame.pixels().len(),
            });
        }
        self.sink.write_frame(frame.pixels()).map_err(FramePipelineError::Sink)
    }
}

/// Single-section boolean rule: the comparison lives behind a named helper.
fn frame_has_expected_size(frame: &Frame, expected: usize) -> bool {
    frame.pixels().len() == expected
}
```

`prepared_pipeline` (same file, keep each method ≤50 lines via one-line helpers):

```rust
/// Production convenience constructor for TODO Task 3's demo command: builds
/// output path, FFmpeg process, chat store, and text renderer from validated
/// config; a SystemFrameClock is fixed inside. Tests do NOT use this (they
/// assemble `PipelineParts` directly).
pub(crate) fn prepared_pipeline(
    config: &AppConfig,
    construction: PipelineConstruction,
) -> Result<FramePipeline<FfmpegProcess>, FramePipelineError> {
    let pacing = PipelinePacing::from_config(&config.video)?;
    let output_path = resolve_output_path(construction.output_kind)
        .map_err(FramePipelineError::Sink)?;
    let process = FfmpegProcess::start(config, output_path).map_err(FramePipelineError::Sink)?;
    let parts = PipelineParts {
        chat_store: build_chat_store(config)?,
        renderer: build_renderer(config)?,
        sink: process,
        source: construction.source,
        clock: Box::new(SystemFrameClock::new()),
        pacing,
        stop_flag: construction.stop_flag,
    };
    Ok(FramePipeline::new(parts))
}

fn build_chat_store(config: &AppConfig) -> Result<ChatStore, FramePipelineError> {
    let settings = ChatStoreSettings::from_config(config)?;
    ChatStore::new(settings).map_err(FramePipelineError::Chat)
}

fn build_renderer(config: &AppConfig) -> Result<Box<dyn Renderer>, FramePipelineError> {
    let settings = RendererSettings::from_config(&config.renderer, &config.video)?;
    let renderer = TextRenderer::new(settings).map_err(FramePipelineError::Render)?;
    Ok(Box::new(renderer))
}
```

(`resolve_output_path`, `FfmpegProcess`, `OutputKind`, `StreamingError`,
`CompletedVideo`, `FrameSink` are consumed via the new `streaming`
re-exports of §2.6.)

### 2.6 Re-export additions — EXACT lists (first external consumers land)

`src/streaming/mod.rs` — append before the trailing test-module wiring… (there
is none; simply append after the `mod` declarations):

```rust
pub(crate) use error::StreamingError;
pub(crate) use ffmpeg::FfmpegProcess;
pub(crate) use sink::{CompletedVideo, FrameSink};
pub(crate) use video_output::{OutputKind, resolve_output_path};

// Widened for the app's test double (sink.rs): `CompletedVideo::new`
// changed `pub(super)` → `pub(crate)`; the test fake must fabricate a
// successful `finish` value without a real FFmpeg process.
```

`src/chat/mod.rs` — append:

```rust
pub(crate) use error::ChatError;
pub(crate) use model::ChatMessage;
pub(crate) use store::{ChatStore, ChatStoreSettings};
```

`src/renderer/mod.rs` — append:

```rust
pub(crate) use contract::{RenderInput, Renderer};
pub(crate) use error::RenderError;
pub(crate) use frame::Frame;
pub(crate) use settings::RendererSettings;
pub(crate) use text::TextRenderer;
```

`src/main.rs` — final content (retired attrs + removed staging comment):

```rust
//! `rust-youtube-streamer-service` process entry point. Delegates to the
//! application layer for command-line parsing, logging initialization,
//! configuration loading, and mode dispatch.

mod app;
mod chat;
mod config;
mod renderer;
mod streaming;

use std::process::ExitCode;

fn main() -> ExitCode {
    app::run(&std::env::args_os().skip(1).collect::<Vec<_>>())
}
```

(`Cargo.toml` untouched in Step 5; no `main.rs` changes beyond the above.)

### 2.7 `src/app/pipeline/test_helpers.rs` (all `#[cfg(test)]`, ≤150 lines)

BINDING design constraint for every fake: `FramePipeline::run(mut self)`
CONSUMES the pipeline (and each fake inside it), so a test can no longer read
recordings off the consumed fake afterwards. Therefore every recording fake
shares its records with the test through a `Rc<RefCell<…>>` (single-threaded
tests only; no `Mutex`/atomics): each fake's constructor returns the pair
`(fake, shared_record)` and the test keeps the shared handle.

```rust
// Deterministic clock: elapsed advances exactly by each recorded sleep.
pub(crate) struct FakeClock {
    elapsed: Cell<u64>,
    sleeps: Rc<RefCell<Vec<u64>>>,
}
impl FakeClock {
    /// Returns the fake plus the shared sleep log the test keeps.
    pub(crate) fn fresh() -> (Self, Rc<RefCell<Vec<u64>>>) {
        let sleeps = Rc::new(RefCell::new(Vec::new()));
        (Self { elapsed: Cell::new(0), sleeps: Rc::clone(&sleeps) }, sleeps)
    }
}
impl FrameClock for FakeClock {
    fn now_nanos(&self) -> u64 { self.elapsed.get() }
    fn sleep_nanos(&self, nanos: u64) {
        self.sleeps.borrow_mut().push(nanos);
        self.elapsed.set(self.elapsed.get() + nanos);
    }
}

// Everything the runner tests observe about a sink after a run.
pub(crate) struct SinkEvents {
    pub(crate) writes: Vec<Vec<u8>>,
    pub(crate) finish_called: bool,
    pub(crate) finish_path: Option<PathBuf>,
}

// Renderer stub: records every `RenderInput` text into a shared log; optional
// one-shot failure (the failure does NOT need to be observed afterwards).
pub(crate) struct StubRenderer {
    inputs: Rc<RefCell<Vec<String>>>,
    failure: Option<RenderError>,
}
impl StubRenderer {
    /// Returns the stub plus the shared input log the test keeps.
    pub(crate) fn fresh() -> (Self, Rc<RefCell<Vec<String>>>) {
        let inputs = Rc::new(RefCell::new(Vec::new()));
        (Self { inputs: Rc::clone(&inputs), failure: None }, inputs)
    }
    pub(crate) fn failing(failure: RenderError) -> Self {
        Self { inputs: Rc::new(RefCell::new(Vec::new())), failure: Some(failure) }
    }
}
impl Renderer for StubRenderer {
    fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        self.inputs.borrow_mut().push(input.text.clone());
        // 2x2 black frame => exactly TEST_FRAME_BYTES bytes.
        Frame::filled(2, 2, Rgb::black())
    }
}

// Frame-sink fake: records every frame byte slice into a shared event log;
// requests the stop flag after the Nth write; fabricates a `CompletedVideo`
// (path = its own output path) on finish and marks the event log.
pub(crate) struct RecordingSink {
    events: Rc<RefCell<SinkEvents>>,
    output_path: PathBuf,
    stop_after_writes: Option<(usize, StopFlag)>,
}
impl RecordingSink {
    /// Returns the sink plus the shared event log the test keeps.
    pub(crate) fn fresh(stop_flag: StopFlag) -> (Self, Rc<RefCell<SinkEvents>>) {
        let events = Rc::new(RefCell::new(SinkEvents {
            writes: Vec::new(),
            finish_called: false,
            finish_path: None,
        }));
        let sink = Self {
            events,
            output_path: PathBuf::from("videos/test-recording.mp4"),
            stop_after_writes: None,
        };
        (sink, events)
    }
    /// Halts the loop by requesting the shared stop flag right after the
    /// Nth recorded write (the N-th frame completes; the loop then breaks).
    pub(crate) fn stop_after_writes(mut self, threshold: usize, stop_flag: StopFlag) -> Self {
        self.stop_after_writes = Some((threshold, stop_flag));
        self
    }
    fn request_stop_when_due(&self) {
        if let Some((threshold, flag)) = &self.stop_after_writes {
            let write_count = self.events.borrow().writes.len();
            if write_count == *threshold {
                flag.request_stop();
            }
        }
    }
}
impl FrameSink for RecordingSink {
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError> {
        self.events.borrow_mut().writes.push(frame.to_vec());
        self.request_stop_when_due();
        Ok(())
    }
    fn finish(self) -> Result<CompletedVideo, StreamingError> {
        let mut log = self.events.borrow_mut();
        log.finish_called = true;
        log.finish_path = Some(self.output_path.clone());
        Ok(CompletedVideo::new(self.output_path))
    }
}
```

// Sink fake that always fails its first write (broken-pipe reuse path).
// `finish` is never reached on the error path; the trait still requires an
// implementation.
// Sink fake that always fails its first write (broken-pipe reuse path).
// Shares the SAME `SinkEvents` shape so the test can still observe that
// `finish` was never reached on the error path.
pub(crate) struct FailingWriteSink {
    events: Rc<RefCell<SinkEvents>>,
}
impl FailingWriteSink {
    /// Returns the fake plus the shared event log the test keeps.
    pub(crate) fn fresh() -> (Self, Rc<RefCell<SinkEvents>>) {
        let events = Rc::new(RefCell::new(SinkEvents {
            writes: Vec::new(),
            finish_called: false,
            finish_path: None,
        }));
        (Self { events }, events)
    }
}

impl FrameSink for FailingWriteSink {
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError> {
        self.events.borrow_mut().writes.push(frame.to_vec());
        Err(StreamingError::FrameWrite {
            source: io::Error::new(io::ErrorKind::BrokenPipe, "pipe closed by the child"),
        })
    }
    fn finish(self) -> Result<CompletedVideo, StreamingError> {
        self.events.borrow_mut().finish_called = true;
        Ok(CompletedVideo::new(PathBuf::from("videos/not-reached.mp4")))
    }
}

// Message-source fakes.
pub(crate) struct ScriptedSource {
    pending: VecDeque<ChatMessage>,
    failure: Option<ChatError>,
}
impl ScriptedSource {
    pub(crate) fn schedule(messages: &[ChatMessage]) -> Self { … }   // no failure
    pub(crate) fn failing(failure: ChatError) -> Self { … }          // Errs on first poll
}
impl MessageSource for ScriptedSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        Ok(self.pending.pop_front())
    }
}

pub(crate) const TEST_INTERVAL_NANOS: u64 = 40;
pub(crate) const TEST_FRAME_BYTES: usize = 12; // 2 * 2 * 3

// Chat store for pipeline tests: rides the crate's `valid_config` test
// fixture (capacity 26) with `chat.log_file` pointed at `<tempdir>/chat.log`,
// so two scripted messages stay far below the visible capacity.
pub(crate) fn temp_store() -> (ChatStore, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("create temp dir");
    let mut config = valid_config();
    config.chat.log_file = temp.path().join("chat.log").to_string_lossy().into_owned();
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let store = ChatStore::new(settings).expect("open store");
    (store, temp)
}

// Parts assembly helper re-used by every runner test so each test stays
// short: one parameter object carries the concrete fake-typed fields.
pub(crate) struct TestSetup<S> {
    pub(crate) chat_store: ChatStore,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) sink: S,
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) clock: Box<dyn FrameClock>,
    pub(crate) stop_flag: StopFlag,
}
impl<S: FrameSink> TestSetup<S> {
    /// `pacing` is always `PipelinePacing::raw(TEST_INTERVAL_NANOS,
    /// TEST_FRAME_BYTES)`; the generic sink type is resolved by inference
    /// from the field at the call site.
    pub(crate) fn into_pipeline(self) -> FramePipeline<S> {
        FramePipeline::new(PipelineParts {
            chat_store: self.chat_store,
            renderer: self.renderer,
            sink: self.sink,
            source: self.source,
            clock: self.clock,
            pacing: PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES),
            stop_flag: self.stop_flag,
        })
    }
}
```

Notes for the implementer (binding):
- The helper sketch above is a SPEC, not a locked typing: `TestSetup` may be
  restructured, and the generic passthrough (`sink` field) may be simplified
  by inlining the parts literal directly in each test if the helper turns out
  harder than the tests themselves. What is
  LOCKED: (a) every recording fake shares its records via `Rc<RefCell>`
  because `run(mut self)` consumes the fakes; (b) pacing in runner tests is
  `PipelinePacing::raw(TEST_INTERVAL_NANOS, TEST_FRAME_BYTES)`; (c) no test
  sleeps for real and no test needs an FFmpeg binary.
- Interior mutability is LOCKED: `Cell<u64>`, `RefCell<…>`, `Rc<…>` only
  (single thread); `finish_called` and the recorded writes live in the shared
  `SinkEvents` — `finish(self)` must record BEFORE it returns, which the
  sketch does via `borrow_mut()` on the events. REQUIRED observable: after a
  failed run, `finish_called == false`.
- `ScriptedSource::failing` builds
  `ChatError::LogAppend { path: PathBuf::from("chat.log"), source:
  io::Error::other("log write failed") }` (variant fields are pub — verified).
- Stub failing uses `RenderError::Geometry { width: 2, height: 2 }`.
- `FailingWriteSink` constructs a fresh `io::Error` per call (no `Clone`
  needed); it may be a unit struct.

### 2.8 Test files — exact test list (ALL must be implemented, names verbatim)

`src/app/pipeline/pacing_tests.rs`

1. `interval_nanos_divides_one_second_across_fps_values` — asserts
   `interval_nanos(25) == 40_000_000`, `interval_nanos(30) == 33_333_333`,
   `interval_nanos(60) == 16_666_666`.
2. `interval_nanos_survives_zero_fps_without_panic` — `interval_nanos(0) ==
   NANOS_PER_SECOND` (documented defensive guard).
3. `pacing_schedule_sleeps_full_interval_when_punctual` —
   `started(40, 0)`; `next_gap(0) == 40`; then `next_gap(40) == 40`;
   `next_gap(80) == 40` (steady state).
4. `pacing_schedule_drops_backlog_to_one_immediate_frame` — `started(40, 0)`;
   stalled clock: `next_gap(200) == 0` (backlog dropped, ONE immediate
   catch-up frame queued); immediately after: `next_gap(200) == 40` (no
   burst); also `next_gap(1_000) == 0` again (every resync behaves the same,
   no unbounded drift).

`src/app/pipeline/clock_tests.rs`

5. `system_frame_clock_reads_are_monotonic_and_non_negative` — construct
   `SystemFrameClock`, read twice in a row: both `>= 0`, second `>=` first.
   (No sleeps in tests; `sleep_nanos` itself is exercised by the runner tests
   through `FakeClock` recording and by production use; a real-sleep test is
   intentionally omitted — container determinism.)

`src/app/pipeline/interrupt_tests.rs`

6. `stop_flag_starts_clear_and_flips_on_request` — `StopFlag::new()`:
   `is_requested() == false`; `request_stop()`; `is_requested() == true`.
7. `stop_flag_clone_shares_one_request_across_holders` — clone the flag,
   request on the CLONE, read `true` on the ORIGINAL (the `Arc<AtomicBool>`
   sharing contract the Ctrl+C handler depends on).
   (`install_interrupt_handler` itself has NO unit test — it mutates process
   global signal state and ctrlc allows a single handler globally; documented
   boundary, precedent: Phase 01 subscriber-install.)

`src/app/pipeline/runner_tests.rs`

(All post-run observations come from the SHARED handles returned by the
`fresh()` constructors per §2.7 — the fakes themselves are consumed by
`run(mut self)`.)

8. `run_returns_completed_video_without_writing_when_stopped_before_start` —
   stop flag pre-set (`flag.request_stop()` before `run`); empty
   `ScriptedSource::schedule(&[])`; `RecordingSink::fresh(flag)`; `FakeClock`.
   Asserts: result `Ok(CompletedVideo)` with the fake's recorded
   `finish_path`; shared events: `writes` empty, `finish_called == true`;
   shared sleep log EMPTY (no sleeps ran).
9. `run_emits_immediate_empty_first_frames_with_exact_byte_counts` —
   `RecordingSink::fresh(flag)` chained with
   `.stop_after_writes(3, flag)` so the loop stops after the third
   frame; `ScriptedSource::schedule(&[])`; `StubRenderer::fresh()`;
   `FakeClock::fresh()`. Asserts: exactly 3 writes, every write
   `len() == TEST_FRAME_BYTES`, renderer input logs are exactly
   `["", "", ""]` (FIRST is empty — the empty initial frame bullet),
   shared sleep log `== [40, 40, 40]` (pacing seam determinism; the loop
   never free-runs), `finish_called == true`, run returns `Ok`.
10. `run_rotates_new_messages_into_visible_text` — `ScriptedSource::schedule`
    with `ChatMessage::new("alice", "Hello!")` then `ChatMessage::new("bob",
    "Hi!")` (then `None` per `next_message`); `stop_after_writes(3, ..)`;
    asserts the shared renderer input log is exactly
    `["alice: Hello!", "alice: Hello!\nbob: Hi!",
    "alice: Hello!\nbob: Hi!"]` — message arrival affects the visible text
    from the second frame on, old messages persist until `ChatStore`
    evicts (eviction itself stays covered by chat-module tests; fixture
    capacity 26 >> 2 messages).
11. `run_propagates_render_failure_without_finalizing_sink` —
    `StubRenderer::failing(RenderError::Geometry { width: 2, height: 2 })` +
    `RecordingSink::fresh(flag)` (no writes can happen) → run returns
    `Err(FramePipelineError::Render(..))`; shared sink events:
    `writes` empty and `finish_called == false` (the sink never reached
    `finish`; Drop is the Task‑1 kill/reap safety net — S9).
12. `run_propagates_sink_write_failure_without_finalizing` —
    `FailingWriteSink::fresh()` → run returns
    `Err(FramePipelineError::Sink(..))`; shared events:
    `finish_called == false`; asserts the error's Display contains
    "frame output failed" and (via `FrameWrite`'s own Task‑1 Display) the
    broken-pipe wording — documented REUSE of Task‑1 error semantics.
13. `run_propagates_message_source_failure_without_rendering` —
    `ScriptedSource::failing(ChatError::LogAppend {...})` +
    `RecordingSink::fresh(flag)` → run returns
    `Err(FramePipelineError::Chat(..))`; shared renderer input log EMPTY;
    shared sink events `finish_called == false`. (ChatStore::accept's own
    LogAppend rejection is already covered by `chat::store_tests` — the
    pipeline wraps it through the same `FramePipelineError::Chat` arm as the
    source failure, so this arm is exercised without touching ChatStore
    internals.)

`src/app/pipeline/error_tests.rs`

14. `frame_size_mismatch_display_states_expected_and_actual_byte_counts`.
15. `frame_byte_overrun_display_states_offending_dimensions`.

**Test tally: 15 new tests** (4 pacing + 1 clock + 2 interrupt + 6 runner +
2 error) → expected suite total **106 + 15 = 121** at the Step 6 gate. All
runnable in the container with NO FFmpeg binary and NO real sleeps (fakes
only). No real-FFmpeg smoke claims anywhere — that stays Task 5.

---

## 3. Ordered implementation steps (execute exactly; commit after each)

Preface (workflow): you are on
`feat/phase03-ffmpeg-local-pipeline` (verified clean). NEVER switch branches.
Never push. Run git stages with only the files a step lists. Every commit must
leave `cargo check --locked` green in the container (gate G2/G3 run the full
script).

All cargo commands use the standard invocation on the Alpine VM
(`alpine-vm_vm_status` FIRST, then `alpine-vm_vm_run_command`, command prefix
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust …`;
exit codes are authoritative even when captured stderr is clipped).

### Step 1 — Dependency: `ctrlc` (D7/D12)

1.1 Edit `Cargo.toml`: add `ctrlc = "3"` to `[dependencies]`
    (alphabetical order → list becomes ab_glyph, ctrlc, serde, serde_json,
    tracing, tracing-subscriber).
1.2 Refresh lockfile in the container:
    `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check`
    (WITHOUT `--locked` — required so Cargo may update `Cargo.lock`).
1.3 Verify `Cargo.lock` now contains `name = "ctrlc"` (with its platform deps
    `nix`/`windows-sys` for the respective targets) — inspect with
    `docker compose … run --rm rust sh -c "grep -n 'ctrlc' Cargo.lock"`.
1.4 Prove the locked build: `docker compose … run --rm rust cargo check --locked`
    → exit 0. If the latest ctrlc cannot compile on rust:1.82 (MSRV error),
    pin: `docker compose … run --rm rust cargo update -p ctrlc --precise
    <last-1.82-compatible-version>` (tempfile 3.14 precedent), then re-run
    `cargo check --locked` and record the pin in the task report.
1.5 Commit: `build: add ctrlc crate for cross-platform interrupt handling`
    — files: `Cargo.toml`, `Cargo.lock` (nothing else).
    ([Gitignore check: neither file is ignored; `Cargo.lock` stays tracked.]

### Step 2 — Pacing + clock seams (pure, no consumers yet)

2.1 Create `src/app/pipeline/` with `mod.rs` EXACTLY as §2.1 except: comment
    out (do NOT declare yet) the lines `mod runner;`, `mod source;`, the
    `#[cfg(test)] mod runner_tests;`, `#[cfg(test)] mod test_helpers;`, and
    all `pub(crate) use` lines except the pacing/clock/error/interrupt ones
    (they reference not-yet-existing files). NOTE: `error.rs` and
    `interrupt.rs` don't exist yet — this step declares only:
    `mod clock; mod error; mod pacing;` — WAIT: keep it simple and compile
    safe: this step's `mod.rs` declares `mod clock; mod pacing;` ONLY,
    plus `#[cfg(test)] mod clock_tests; #[cfg(test)] mod pacing_tests;`,
    plus `pub(crate) use clock::{FrameClock, SystemFrameClock}; pub(crate)
    use pacing::{NANOS_PER_SECOND, PipelinePacing, PacingSchedule,
    interval_nanos};`. (Succeeding steps add their declarations.)
2.2 Write `pacing.rs` (S6) and `clock.rs` (S6).
2.3 Write `pacing_tests.rs` (tests 1–4) and `clock_tests.rs` (test 5).
2.4 Verify in container:
    `docker compose … run --rm rust cargo test --locked pipeline` → new tests
    green; then `docker compose … run --rm rust cargo clippy --locked -- -D warnings`
    → exit 0 (the warnings produced by temporarily-unused pacing items MUST be
    silenced by the pipeline module's inner `#![allow(dead_code)]` staging
    attribute added NOW in `pipeline/mod.rs` per S11 — include the attr line +
    comment from step 5's final shape in this first version; runner/source
    simply don't exist yet).
2.5 Commit: `feat: add frame pacing schedule and monotonic clock seam`
    — files: `src/app/pipeline/mod.rs`, `pacing.rs`, `pacing_tests.rs`,
    `clock.rs`, `clock_tests.rs`.

### Step 3 — Interrupt seam

3.1 Create `interrupt.rs` (S7, including the 5-point limitations doc block).
3.2 Add declarations: `mod interrupt;`, `#[cfg(test)] mod interrupt_tests;`
    and `pub(crate) use interrupt::{InterruptHandlerError, StopFlag,
    install_interrupt_handler};` to `pipeline/mod.rs`.
3.3 Write `interrupt_tests.rs` (tests 6–7).
3.4 Container verify: `cargo test --locked pipeline`; `clippy --locked -- -D warnings`.
3.5 Commit: `feat: add ctrlc stop flag and interrupt install seam`
    — files: `src/app/pipeline/{mod,interrupt,interrupt_tests}.rs`.

### Step 4 — Error model + `CompletedVideo` visibility

4.1 Create `error.rs` (S8: variant set, Display strings, `Error::source` map).
4.2 Edit `src/streaming/sink.rs`: change `pub(super) fn new` →
    `pub(crate) fn new` on `CompletedVideo`, and extend its doc comment
    (`/// Test doubles in the app crate fabricate a success value without a
    real process; construction stays crate-internal.`). NO other sink change.
4.3 Declarations into `pipeline/mod.rs`: `mod error;`, `#[cfg(test)] mod
    error_tests;`, `pub(crate) use error::FramePipelineError;`.
4.4 Write `error_tests.rs` (tests 14–15).
4.5 Container verify: `cargo test --locked pipeline`,
    `clippy --locked -- -D warnings`, `cargo check --locked`.
4.6 Commit: `feat: add typed frame pipeline error model`
    — files: `src/app/pipeline/{mod,error,error_tests}.rs`,
    `src/streaming/sink.rs` (visibility widen only).

### Step 5 — Wiring: runner + source + re-exports + main.rs retirement

5.1 Create `source.rs` (S5 trait).
5.2 Create `runner.rs` (S9/§2.5: PipelineParts, PipelineConstruction,
    FramePipeline + impl, frame_has_expected_size, prepared_pipeline +
    build_chat_store + build_renderer).
5.3 Add the EXACT re-export blocks of §2.6 to `src/chat/mod.rs`,
    `src/renderer/mod.rs`, `src/streaming/mod.rs` (streaming gets its four
    `pub(crate) use` lines + the `CompletedVideo` widen if not already done —
    it was done in 4.2; the streaming/mod.rs re-exports are added here).
5.4 Apply the §2.6 final `src/main.rs` content (delete the three
    attributes + the two comment lines — main.rs staging retirement, S11).
5.5 WHERE the inner allow attributes land (S11): add one
    `#![allow(dead_code)]` line + retire-note sentence to
    `src/chat/mod.rs`, `src/renderer/mod.rs`, `src/streaming/mod.rs`, and
    keep the one already present in `src/app/pipeline/mod.rs`. Also rewrite
    the explicit doc sentences that refer to the old main.rs attributes:
    - `chat/mod.rs` & `renderer/mod.rs`: replace "the `#[allow(dead_code)]`
      on `mod chat;` in `main.rs` retires at that Phase 03 app-wiring step"
      (and the `renderer` equivalent paragraph) with: "Staging allow: retires
      in TODO Task 3 when the demo mode (the pipeline's CLI consumer) makes
      this module reachable from `main`; clippy `-D warnings` guards
      everything else."
    - `streaming/mod.rs`: replace the doc lines that say the allow retires
      when Task 2 wires the frame pipeline with the same retire sentence.
5.6 Create `test_helpers.rs` (§2.7) and `runner_tests.rs` (tests 8–13).
5.7 Final `pipeline/mod.rs` shape = §2.1 verbatim (all declarations + all
    re-exports).
5.8 Add `mod pipeline;` to `src/app/mod.rs` (after `mod modes;`, keeping
    alphabetical order of the block: cli, error, logging, modes, pipeline).
5.9 Container verify: `cargo fmt --check`, `cargo check --locked`,
    `cargo test --locked`, `cargo clippy --locked -- -D warnings` — all exit 0.
    (fmt first, fix formatting as needed via `cargo fmt` — run it BEFORE the
    fmt --check.)
5.10 Commit: `feat: wire chat renderer and ffmpeg sink into the frame pipeline`
     — files: `src/main.rs`, `src/app/mod.rs`, new
     `src/app/pipeline/{source,runner,test_helpers,runner_tests}.rs`,
     `src/app/pipeline/mod.rs`, `src/chat/mod.rs`, `src/renderer/mod.rs`,
     `src/streaming/mod.rs` (re-exports + staging attrs).

### Step 6 — `.gitkeep` housekeeping

6.1 `git rm src/streaming/.gitkeep`.
6.2 Verify: `git ls-files "*.gitkeep"` → only `logs/.gitkeep` and
    `credentials/.gitkeep` remain.
6.3 Commit: `chore: remove streaming folder placeholder marker`.

### Step 7 — Gate G2 (the standard full gate on the final tree)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```
Expected: exit 0; `PASS fmt-check exit=0`, `PASS check exit=0`, `PASS test
exit=0`, `PASS clippy exit=0`, summary `ALL CHECKS PASSED`, and the test count
line `running 121 tests` → `121 passed; 0 failed`. Record the produced
`logs/checks/<timestamp>.log` filename. If any check fails: fix ONLY this
task's code, re-run the failing script; do not touch unrelated code.

### Step 8 — NOT executed by this task (hand-off)

- Code review (4.3) by code-reviewer + code-simplifier against this plan;
  adherence (4.5b) separately: verify every S1–S12 decision; verify suite
  total is 121; verify no `demo`/CLI/exit-code/docs files changed.
- Real-FFmpeg execution does NOT exist (the pinned container has no ffmpeg
  binary): it is a Task‑5 scope with fake-process tests and optional smoke
  test. State the NOT-RUN limitation verbatim; never invent evidence.

---

## 4. Cross-check table: TODO Task 2 bullets → plan coverage

| TODO Task 2 bullet | Where covered |
| --- | --- |
| Wire Phase 02 renderer + ChatStore into app orchestration + FFmpeg frame writer | S1/§2.5 (`runner.rs` + `prepared_pipeline`), Step 5 re-exports |
| Emit frames continuously at configured FPS incl. before first message / empty intervals; empty initial frame shows configured black background | S6 pacing loop; test 9 (first input `""`, background filled by `Frame::filled` inside the renderer; color comes from renderer settings) |
| Per frame render complete visible text and write exactly one full RGB24 frame; short writes + broken pipes; never drop/truncate bytes | S10 (reuse `write_all` semantics), size gate `write_frame_bytes`; broken-pipe surface via `StreamingError::FrameWrite`; tests 9/12 |
| Respect renderer width/height/font/margins/line-height/colors; no wrapping/scrolling/game/screenshot | pipeline only passes `visible_text`; all rendering owned by `TextRenderer` (construction via `RendererSettings::from_config`); nothing added |
| Paced against configured FPS; no unbounded drift; no unbounded frame/message queue | S6 (anchored incremental + backlog drop + 0-catch-up bound); drain loop feeds the bounded ChatStore only; tests 3–4, 8–9 |
| Normal completion stops production, closes stdin once, waits, surfaces failures | S9 + `FfmpegProcess::finish` (single `stdin = None` EOF; verified exit + non-empty file); test 8/12 |
| Ctrl+C as shutdown request cross-platform; document limits | S7 (ctrlc, StopFlag, install fn, 5 verified points + limitations) |
| Standing constraints: Rust 1.82/edition 2021; no installed FFmpeg; no google-youtube3; justify deps; bounded wait defer | S2, S7, S8, S9, Step 1 |

Acceptance criteria interlock (phase-level, so only the wiring-relevant set):
- "FFmpeg receives complete RGB24 frames at the configured dimensions and FPS
  through stdin" → frame bytes exactly `PipelinePacing::frame_byte_count`
  (`width*height*3`), fps → interval + FFmpeg `-framerate` from Task 1 args.
- "Frames continue during empty-chat intervals; messages rendered from
  ChatStore; eviction by FIFO capacity" → loop + ChatStore (eviction in chat
  module, already tested).
- "MP4 finalized before completion reported" → `finish()` semantics reused.
- CLI/demo/path/`run` criteria belong to Tasks 3–5 and are NOT implemented here.

## 5. Out of scope (HARD — any appearance = plan violation)

- NO `demo` command, NO CLI surface/help/usage changes, NO exit-code changes,
  NO new `Mode` variants (Task 3).
- NO `run`-mode behavior change (`auth`/`run` placeholders stay exit 3) (Task 4).
- NO README/docs/configuration.md/project-structure.md/context.md edits
  (Task 5; inline code-file comments in changed files are fair game).
- NO real FFmpeg runs/smoke claims (Task 5; the pinned rust:1.82 container has
  no ffmpeg binary).
- NO FFmpeg timeout mechanism, NO config schema field additions (D10 process),
  NO audio, NO screenshots, NO interactive controls, NO YouTube/OAuth touches.
- NO source-module logic changes inside `streaming` beyond sink.rs visibility
  widen + re-exports; NO changes inside `chat`/`renderer` beyond re-exports +
  staging attributes.

## 6. Review & adherence checklist (for steps 4.3 / 4.5b)

1. Files ≤200 lines, methods ≤50 lines, depth ≤2, ≤2 constructor/loop args
   (param objects used: `PipelineParts`, `PipelineConstruction`,
   `RecordingNameSpec` untouched).
2. Boolean conditions single-section (`frame_has_expected_size`,
   `request_stop_when_due` helpers extracted).
3. Self-documenting names; no commented-out code; private fields everywhere
   (only `pub(crate)` items cross the module boundary; `PipelineParts` fields
   `pub(crate)` justified in-code).
4. All 15 tests present with the EXACT names of §2.8; suite = 121; no test
   performs a real sleep (`FakeClock` everywhere; the `SystemFrameClock` test
   performs reads only); no test requires an FFmpeg binary.
5. `git log --stat` for Task 2 shows exactly the commits of §3 in order;
   each commit's file set matches; no unrelated files staged; `.gitignore`
   compliance checked (no `videos/`, `dist/`, log files staged).
6. `dev-checks.sh` gate output archived at the step-7 point (log filename
   reported back to the Planner).
7. Every decision S1–S12 demonstrable in code with file references.

## 7. Risks / watch items for the implementer

- `PacingSchedule::next_gap` arithmetic: never use `as` casts on elapsed time
  (keep `u64::try_from(..).unwrap_or(u64::MAX)` only where sanctioned, in the
  real clock).
- `Frame::filled` inside `StubRenderer` returns a `Result` — propagate with
  `?`/expect inside test code ONLY (`#[cfg(test)]`).
- `while let Some(message) = self.source.next_message()?` — the `?` inside
  `while let` requires the closure of `drain_messages` returning
  `Result<_, FramePipelineError>`; keep the `map_err` on `accept`.
- Do NOT add `pub` beyond `pub(crate)` anywhere in this task.
- CRLF: repo-host `core.autocrlf=true` — write/edit files with LF-normalized
  content and confirm `git diff` has no whole-file churn after staging.
- Multiple Ctrl+C presses, delayed flag observation, and stuck-pipe
  termination are DOCUMENTED LIMITATIONS (S7): they must appear in code docs
  and flow to Task 5 docs; do not "fix" them in this task by claiming they
  work.

## 8. Addendum — Planner conflict rulings (implementation-time)

Four plan-internal contradictions were escalated by the implementer and
resolved by the Planner BEFORE any further implementation. These rulings are
part of the adopted plan; sections that conflict with them are superseded:

- **R1 (Q1→B)**: the §2.1 `pipeline/mod.rs` self re-export lines are DEFERRED
  — no re-exports in Task 2 (house rule: "re-exports return at the first
  external consumer"; Task 3 adds them if needed). §2.5's submodule-path
  imports (`crate::app::pipeline::clock::…` etc.) governed.
- **R2 (Q2→A)**: the runner consumes AT MOST ONE message per frame iteration
  (`accept_next_message`, `if let`), not S5's drain-all loop. §2.5 prose and
  test 10 stand together only under R2; test 10 itself is the authority.
- **R3 (Q3→amended A)**: the §2.6 renderer re-export block is split —
  `pub(crate) use settings::RendererSettings;` plus a `#[cfg(test)]`
  gated `pub(crate) use settings::Rgb;` (test-only construction access),
  avoiding an unused-import in the non-test build under `clippy -D warnings`.
- **R4 (Q4→A)**: `PipelinePacing::from_config` (returns `FramePipelineError`)
  is staged in the error-model commit (`feat: add typed frame pipeline error
  model`), whose file set deviates by exactly `pacing.rs`.
- Additional planner-ordered micro-decisions: the trivially-true
  `assert!(first >= 0)` on `u64` in clock test 5 was deleted;
  `mod pipeline;` was added to `src/app/mod.rs` unstaged until the wiring
  commit (FYI-1 approved).

These rulings were applied by commits `89c1f9a`..`60102fc` and verified by
the 4.3 review (no fix plan required) and simplification (no plan required).

[PLAN-END]
