# Implementation Plan — Phase 03, TODO Task 1: FFmpeg Process Wrapper and Local MP4 Output

- **TODO source**: `.agent/todos/20261011/20261011-todo-1.md`, section `### 1. FFmpeg Process Wrapper and Local MP4 Output`
- **Global plan**: `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md` (decisions D1–D12 apply; this plan refines D1–D5, D10–D12, never reverses them)
- **Branch**: `feat/phase03-ffmpeg-local-pipeline` — ALREADY CREATED (step 2 done). Do NOT create it again; verify with `git branch --show-current` and stop if otherwise.
- **Version**: `0.6.0` — ALREADY BUMPED (step 3 done, `Cargo.toml` shows 0.6.0). Do NOT bump again.
- **Canonical verification environment**: the Alpine VM MCP (`alpine-vm_vm_status` before every `alpine-vm_vm_run_command`), standard command pattern:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  Everything is Linux-container-first; every example path/executable in this plan is written for the Linux container.
- **Implementer profile**: junior, under 50% restriction. Follow steps exactly; do not restructure files, rename APIs, or change scope.

---

## 0. Verified FFmpeg research (authoritative; do NOT re-derive from memory)

### 0.1 Doc-verified facts and sources

| # | Fact (used in this design) | Source URL |
| --- | --- | --- |
| R1 | `rawvideo` demuxer options: `framerate` (default 25), `pixel_format` (default `yuv420p`), `stride`, `video_size` (must be specified explicitly). Official example: `ffplay -f rawvideo -pixel_format rgb24 -video_size 320x240 -framerate 10 input.raw` | https://ffmpeg.org/ffmpeg-formats.html (§3.23 rawvideo) |
| R2 | `-s <w>x<h>` as an **input** option is a shortcut for the `video_size` private option, "recognized by some demuxers … e.g. raw video". | https://ffmpeg.org/ffmpeg.html (Video Options) |
| R3 | `-r <fps>` as an **input** option generates timestamps assuming constant frame rate (valid for raw formats only). The doc's generic advice for demuxer-native rates is "If in doubt use `-framerate` instead of the input option `-r`". | https://ffmpeg.org/ffmpeg.html (Video Options) |
| R4 | Input options come **before** `-i`; output options come after the output file name; `-f` is "guessed from the file extension for output files"; `-i url` names the input, anything uninterpretable is an output url. | https://ffmpeg.org/ffmpeg.html (Synopsis / Description) |
| R5 | `-pix_fmt` sets the pixel format (input and output sides are separate per-stream options); if the requested output pixel format cannot be selected, ffmpeg warns and picks the encoder's best. | https://ffmpeg.org/ffmpeg.html (Video Options) |
| R6 | `movflags +faststart`: "Run a second pass moving the index (moov atom) to the beginning of the file… not enabled by default." Also: a normal, non-fragmented MOV/MP4 "is undecodable if it is not properly finished" — which is exactly why we must not report success before FFmpeg exits cleanly. | https://ffmpeg.org/ffmpeg-formats.html (§4.4 MOV/MPEG-4/ISOMBFF muxers) |
| R7 | `-n` (global): "Do not overwrite output files, and exit immediately if a specified output file already exists." `-y` is the overwrite counterpart. | https://ffmpeg.org/ffmpeg.html (Main options) |
| R8 | `-nostdin` disables interaction on standard input ("interaction on stdin is on by default unless standard input is used as an input"). Since our stdin IS the frame-feed input, interaction is off automatically, but the explicit flag is the documented way to guarantee frame bytes are never consumed as interactive commands. | https://ffmpeg.org/ffmpeg.html (Main options) |
| R9 | `pipe:0` is the documented way to read from stdin (`0 for stdin`); example: `cat test.wav \| ffmpeg -i pipe:0`. | https://ffmpeg.org/ffmpeg-protocols.html (§3.20 pipe) |
| R10 | Log levels: default is `info`; `error` (16) includes recoverable errors, `fatal` (8) shows only fatal ones; ffmpeg writes all diagnostics to stderr. | https://ffmpeg.org/ffmpeg.html (Advanced `-loglevel`) |

Distro mirrors of the same `ffmpeg(1)` man document (manpages.ubuntu.com, manpages.opensuse.org) corroborate R2–R5, R7–R10.

### 0.2 What this design defers to runtime verification (Task 5 fake/real-FFmpeg tests)

- V1 — Natural stdin EOF finalizes the MP4 and FFmpeg exits 0 (with `+faststart` second pass). Doc support is R6; actual behavior must be proven with a real FFmpeg run (fake processes can only mimic exit codes).
- V2 — The ffmpeg CLI accepts the demuxer-private `-framerate <fps>` for `-f rawvideo` (docs name the option `framerate` on the demuxer, R1; ffplay uses it verbatim in the official example). Runtime check required; if the ffmpeg tool rejected it, the fallback is the input-side `-r <fps>` from R3 — that substitution decision may only be made with recorded evidence, by a later architector review, never silently.
- V3 — `libx264` accepts `-preset veryfast` and `-b:v <bitrate>` (`-b:v` is the per-stream bitrate alias of `-b`) and encodes `-pix_fmt yuv420p` output.
- V4 — `ffmpeg <args>` shell substitutability: the exact argument vector can be replayed manually to diagnose Task 5 failures.

**Locked argument vector (pure, deterministic; built by `args.rs`):**

```text
-nostdin
-n
-f           rawvideo
-pix_fmt     rgb24
-s           <width>x<height>
-framerate   <fps>
-i           pipe:0
-c:v         <ffmpeg.video_codec>
-preset      <ffmpeg.preset>
-b:v         <ffmpeg.bitrate>
-pix_fmt     yuv420p
-movflags    +faststart
<output_path>
```

Rationale record (write this into `src/streaming/args.rs` module docs):
- R1+R2+R3: rawvideo input needs explicit `pixel_format` (`rgb24`, which equals `config.video.pixel_format` — already validated by the config loader to be exactly `rgb24`), explicit `video_size` via the `-s` input shortcut, and an explicit rate via the demuxer's documented `framerate` option.
- R4: all input-side options before `-i pipe:0`; all output-side options after; output format is driven by the `.mp4` extension, so no `-f mp4` is passed (fewer args, doc-supported).
- R5: output `-pix_fmt yuv420p` is chosen for maximum player/browser/device compatibility (8-bit 4:2:0 H.264 is the most broadly decodable configuration). It is passed explicitly because R5 says ffmpeg would otherwise silently pick the encoder's best.
- R6: `+faststart` rewrites the container with the `moov` index at the front after encoding; it runs during FFmpeg's normal completion on our seekable file output, so success is reported only after process exit.
- R7: `-n` converts any accidental file-exists race into a clean non-zero FFmpeg exit instead of an interactive overwrite prompt reading our frame pipe. The filename generator already guarantees non-existent paths; `-n` is defense in depth.
- R8: `-nostdin` guarantees interactive command handling can never read a single byte of the frame pipe.
- R7/R8 position: first in the vector (global options).
- Program arguments are passed directly (`std::process::Command` + `.args`, no shell anywhere, per D2); the executable path comes from `config.ffmpeg.executable` (already validated non-blank).

---

## 1. Architecture (encodes decisions; implementer MUST follow these names)

### 1.1 New module layout — `src/streaming/` (all files < 200 lines)

```text
src/streaming/
  mod.rs            module docs + private submodule declarations (no re-exports yet;
                    first external consumer — Task 2 frame pipeline — adds them)
  error.rs          StreamingError, all variants, hand-written Display + std::error::Error
  args.rs           pure FFmpeg argument builder (+ args_tests.rs)
  utc_timestamp.rs  pure UTC civil-date/time formatting (+ utc_timestamp_tests.rs)
  video_output.rs   videos/ directory beside the executable + collision-safe filename
                    (+ video_output_tests.rs)
  ffmpeg.rs         FfmpegProcess: spawn (no shell), drain threads, finish/kill
  sink.rs           FrameSink trait + CompletedVideo struct (the "small interface")
```

Module-independence rules (write into `mod.rs` docs):
- `streaming` depends only on `std` and `crate::config::model` (typed `AppConfig`). It never depends on `crate::chat`, `crate::renderer`, or any YouTube type.
- The seam receives opaque frame bytes (`&[u8]`) only — it cannot know what a chat message is.
- `streaming` is imported by `src/main.rs` with a deliberate `#[allow(dead_code)]` that retires when Task 2 wires the frame pipeline (same pattern as `mod chat;` / `mod renderer;` today).

Exact `src/main.rs` edit (only change to main.rs in Task 1):

```rust
#[allow(dead_code)]
mod streaming;
```

placed with the other `mod` declarations (after `mod renderer;`). Keep the existing `#[allow(dead_code)]` on `mod chat;` and `mod renderer;` untouched.

### 1.2 `src/streaming/mod.rs`

```rust
//! FFmpeg process supervision and the raw-frame input pipeline (brief
//! §10.1–10.5; scope authority `.agent/todos/20261011/20261011-todo-1.md`
//! §1). The module launches FFmpeg as an external process with arguments
//! built purely from validated configuration, feeds raw RGB24 frames to its
//! standard input, waits for clean stdin-EOF finalization, and reports
//! process failures as typed errors. It depends only on the standard library
//! and the typed configuration; it knows nothing about chat messages,
//! rendering, or YouTube. Frame bytes arrive through the `FrameSink` seam as
//! opaque byte slices.
```

Declarations below the docs (private submodules, matching `chat`/`renderer` precedent):

```rust
mod args;
mod error;
mod ffmpeg;
mod sink;
mod utc_timestamp;
mod video_output;
```

Test-only module re-declarations follow the `#[path]` pattern used by `chat`/`renderer` (e.g. inside each source file: `#[cfg(test)] #[path = "args_tests.rs"] mod args_tests;`).

### 1.3 `src/streaming/error.rs` — `StreamingError`

Hand-written `Display` + `impl std::error::Error` with `source()`, mirroring `src/chat/error.rs` and `src/renderer/error.rs` (no thiserror/anyhow — dependency policy D12). All fields private-capable (enum variants are fine; chat precedent uses struct variants).

```rust
pub enum StreamingError {
    /// The pixel format in the loaded configuration is not the pipeline's
    /// `rgb24`. Defensive check; the config validator makes this
    /// unreachable through `load_from_path`, but the argument builder still
    /// refuses to produce a wrong-format command.
    UnsupportedPixelFormat { value: String },
    /// The application's own executable location could not be determined,
    /// so the `videos/` output directory cannot be placed beside it.
    ExecutableLocation { reason: String },
    /// The `videos/` output directory could not be created (or exists as a
    /// non-directory). Carries the final target path and the I/O cause.
    OutputDirectoryCreate { path: PathBuf, source: io::Error },
    /// All collision-suffix attempts for a recording filename were already
    /// taken — this proves an overwrite would have happened, so nothing is
    /// written.
    OutputNameExhausted { directory: PathBuf, attempts: u32 },
    /// FFmpeg could not be started at the configured executable path
    /// (missing executable, permissions, etc.). The `executable` field is
    /// the configured path text (needed for actionable output) — this is
    /// not a secret; there are no secrets in this phase.
    Spawn { executable: String, source: io::Error },
    /// A frame write to FFmpeg's stdin failed (including broken pipe).
    /// Task 2's frame loop surfaces this to the user.
    FrameWrite { source: io::Error },
    /// Waiting for the child process raised an I/O error.
    WaitFailed { source: io::Error },
    /// FFmpeg exited with a non-zero status (or was killed by a signal).
    /// Carries the extracted exit code (`None` for signal death) and the
    /// captured stderr diagnostics tail.
    ExitFailure { exit_code: Option<i32>, diagnostics: String },
    /// FFmpeg exited successfully but the output file is missing or empty —
    /// the MP4 was not finalized.
    OutputFinalize { path: PathBuf },
}
```

Display contracts (each variant one `write!` arm; message style follows existing modules):
- UnsupportedPixelFormat → `streaming pipeline requires pixel format 'rgb24' in configuration (got '{value}')`
- ExecutableLocation → `unable to resolve the application executable location: {reason}`
- OutputDirectoryCreate → exactly `unable to create video output directory '{}': {}` (path, source)
- OutputNameExhausted → `video file naming gave up in directory '{}' after {attempts} collision suffixes; the recording was not started`
- Spawn → `unable to start the FFmpeg executable '{}': {}; check that the configured path is correct and FFmpeg is installed and runnable` (append the hint only when the kind is NotFound — implement as two write arms on `is_file_not_found(source)` helper)
- FrameWrite → message text: `writing a frame to the FFmpeg process failed: {source}` plus a second line when `kind == io::ErrorKind::BrokenPipe` (single-section boolean rule: put the branch on the helper `is_broken_pipe(&io::Error) -> bool`, then the arm uses one `if is_broken_pipe(...)` that appends `; the FFmpeg process exited while receiving frames`)
- WaitFailed → `waiting for the FFmpeg process failed: {source}`
- ExitFailure → when `exit_code` is `Some(c)`: `FFmpeg exited with failure status {c}; captured diagnostics follow: {diagnostics}`; when `None`: `FFmpeg terminated by a signal; captured diagnostics follow: {diagnostics}` (branch via helper `exit_failure_description(exit_code) -> String` to honor single-section boolean conditions)
- OutputFinalize → `FFmpeg reported success but the output file '{}' is missing or empty; the video was not finalized`

`source()` returns the `io::Error` for `OutputDirectoryCreate`, `Spawn`, `FrameWrite`, `WaitFailed`; `None` for the rest. No panics. Every constructor needed by the other files is implemented as small private-generic helpers only if shared (e.g. `FrameWrite`).

### 1.4 `src/streaming/args.rs` — pure argument builder

```rust
//! Builds the full FFmpeg command-line argument vector purely from typed,
//! already-validated configuration plus the resolved output path. No
//! environment access, no file system, no spawning — pure and unit-testable.
//! The locked vector and its documented rationale (official FFmpeg doc
//! sources FFmpeg-formats §3.23/§4.4, FFmpeg main options, FFmpeg-protocols
//! §3.20) are in .kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md §0.2;
//! the vector order is asserted by tests and must not be reordered.

pub(crate) fn build_arguments(
    config: &AppConfig,
    output_path: &Path,
) -> Result<Vec<String>, StreamingError> {
    // 1. Defensive pixel-format check (single-section boolean):
    //    if is_non_rgb24_pixel_format(&config.video.pixel_format) {
    //        return Err(StreamingError::UnsupportedPixelFormat { value: ... });
    //    }
    // 2. Return Vec<String> in the exact locked order from §0.2.
    //    Width/height/fps formatting: format!("{}x{}"), format!("{}", fps)
    //    (config already guarantees positive values; no re-validation).
}
```

Constants (file-private): `const PIPE_SOURCE: &str = "pipe:0";`, `const OUTPUT_PIXEL_FORMAT: &str = "yuv420p";`, `const MOVFLAGS_FASTSTART: &str = "+faststart";`, plus the literal flag names inline in order.

Notes for the implementer:
- `config.video.pixel_format` is the string to emit as the **input** pixel format — but only after the defensive check above; never pass any other value.
- The output path is appended last, verbatim (`output_path.to_string_lossy().into_owned()` — test paths are always valid UTF-8 in tests; lossy conversion keeps non-UTF-8 operator paths from panicking).
- Max-2-args rule: `build_arguments` takes exactly `&AppConfig` and `&Path`.

### 1.5 `src/streaming/utc_timestamp.rs` — pure UTC timestamp formatting

Needed because the standard library has no calendar formatting and D12 forbids adding a crate for this. All functions pure; documented as UTC (a deliberate, tested deviation from wall-clock local time; note it in the file docs).

```rust
pub(crate) fn format_utc_timestamp(epoch_seconds: u64) -> String
// -> "YYYYMMDD-HHMMSS", zero-padded, proleptic Gregorian, e.g. epoch 0 ->
// "19700101-000000".

// Internal helpers (private):
//  fn civil_date_from_epoch_days(days: u64) -> (u64, u32, u32) // (year, month, day)
//  fn is_leap_year(year: u64) -> bool
```

Algorithm (standard civil-from-days; implement exactly or equivalently, tests are the gate):
- `seconds_of_day = epoch_seconds % 86_400`; `days = epoch_seconds / 86_400`.
- `hour = seconds_of_day / 3600`; `minute = (seconds_of_day % 3600) / 60`; `second = seconds_of_day % 60`.
- Date from days: derive year by stepping through era arithmetic — find the largest year `Y >= 1970` such that the day offset of Jan 1 of Y is <= `days`, using the fixed-length block trick (`365`-day years + leap correction), then walk months with a `const MONTH_LENGTHS: [u32; 12]` array adjusted for leap February. Prefer the simplest correct loop that passes the anchor tests below; do NOT add external math crates.

Unit tests (`utc_timestamp_tests.rs`) — required exact anchors:
- `format_utc_timestamp(0)` == `"19700101-000000"`
- `format_utc_timestamp(1_000_000_000)` == `"20010909-014640"`
- `format_utc_timestamp(1_600_000_000)` == `"20200913-122640"`
- `format_utc_timestamp(59)` == `"19700101-000059"` and `format_utc_timestamp(60)` == `"19700101-000100"` (minute rollover)
- leap-year sanity: `format_utc_timestamp(951_868_800)` == `"20000229-000000"` (2000-02-29T00:00:00Z; verify this epoch constant once online before writing the test and record it in the test comment)

### 1.6 `src/streaming/video_output.rs` — output directory + collision-safe names

```rust
/// Which pipeline names the recording: the demo command uses `demo-`;
/// `session-` is reserved for the future production/local pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputKind { Demo, Session }

impl OutputKind {
    pub(crate) fn filename_prefix(self) -> &'static str  // "demo" / "session"
}

/// All inputs for generating one recording filename. Parameter object keeps
/// the generator within the max-2-arguments rule.
pub(crate) struct RecordingNameSpec {
    output_directory: PathBuf,
    kind: OutputKind,
    epoch_seconds: u64,
}

impl RecordingNameSpec {
    pub(crate) fn new(output_directory: PathBuf, kind: OutputKind, epoch_seconds: u64) -> Self
    // 3-parameter constructor is a constructor, not a call-site method call —
    // allowed; callers then invoke only `next_available`.

    /// Returns the first non-existing candidate filename (name only, not a
    /// full path) in the deterministic order:
    ///   "{prefix}-YYYYMMDD-HHMMSS.mp4", then "_2", "_3", ... appended
    /// before the extension ("...-090000_2.mp4"), up to MAX_SUFFIX_ATTEMPTS.
    /// `exists` is an injected predicate so this stays pure and testable.
    /// Exceeding the attempts budget fails with
    /// StreamingError::OutputNameExhausted; names are never truncated.
    pub(crate) fn next_available(
        &self,
        exists: &mut dyn FnMut(&Path) -> bool,
    ) -> Result<String, StreamingError>
}

/// Resolves `videos/` beside the executable's directory and creates it if
/// missing. Two-step split so the mapping stays testable:
///   1. pub(crate) fn resolve_videos_directory(
///          executable_path: &Path,
///      ) -> Result<PathBuf, StreamingError>            // pure mapping
///   2. pub(crate) fn create_directory(
///          directory: &Path,
///      ) -> Result<(), StreamingError>                // fs side effect
/// i. `resolve_videos_directory` walks `executable_path.parent()`;
///    a missing parent is StreamingError::ExecutableLocation with a reason.
/// ii. `create_directory` uses `std::fs::create_dir_all`; an existing plain
///    file at the path surfaces the create error via
///    OutputDirectoryCreate.
///
/// Composition used by future Task 2/3 wiring (side-effectful, integration
/// territory; provide it now so the Task 3 demo wiring can call exactly this):
///   pub(crate) fn resolve_output_path(kind: OutputKind) -> Result<PathBuf, StreamingError>
/// which:
///   1. std::env::current_exe() -> ExecutableLocation error on failure.
///   2. resolve_videos_directory(...) then create_directory(...).
///   3. epoch seconds from SystemTime::now().
///   4. RecordingNameSpec::next_available with the real existence probe
///      `fn path_exists(path: &Path) -> bool` that treats every probe error
///      (permissions, etc.) as "exists" so a mistaken probe can never let
///      an overwrite through.
```

Constants: `const OUTPUT_DIRECTORY_NAME: &str = "videos";`, `const FILENAME_EXTENSION: &str = "mp4";`, `const FILE_EXTENSION_SEPARATOR: &str = ".";`, `const COLLISION_SUFFIX_SEPARATOR: &str = "_";`, `const FIRST_COLLISION_SUFFIX: u32 = 2;`, `const MAX_SUFFIX_ATTEMPTS: u32 = 1000;`.

Unit tests (`video_output_tests.rs`):
- prefix mapping: `OutputKind::Demo.filename_prefix()` == `"demo"`, `Session` == `"session"`.
- `resolve_videos_directory` mapping: executable `/opt/app/service` → `/opt/app/videos`; executable `/opt/app/nested/service.exe` → `/opt/app/nested/videos`; executable without a parent (`/`) → `ExecutableLocation`.
- `create_directory`: with `tempfile::TempDir` (dev-dep already in `Cargo.lock`): nonexistent → Ok, persists the directory; an existing file at the target path (create a plain file first) → `OutputDirectoryCreate`.
- `next_available` with injected predicate over a simulated existing-name set: first call with nothing existing → base name; base pre-existing → `_2` suffix; `_2` also existing → `_3` (loop until a free slot); a set where every suffix is taken proves `OutputNameExhausted { attempts: 1000 }` with no truncation and no overwrite.
- generated-format check: for kind `Demo` and epoch seconds `1_600_000_000`, base candidate == `"demo-20200913-122640.mp4"`.

### 1.7 `src/streaming/sink.rs` — the small frame-sending interface (the seam)

```rust
//! The minimal contract for sending complete frames and finishing the
//! process: the frame pipeline (Task 2) sees only this trait, never
//! `std::process` types, keeping FFmpeg process management independent of
//! chat/render details (deliverable bullet: "Define a small interface for
//! sending complete frames and finishing the process").

pub struct CompletedVideo {
    output_path: PathBuf,
}

impl CompletedVideo {
    pub fn path(&self) -> &Path
    // Only accessor; field stays private (prefer-private-members rule).
}

/// Sends complete RGB24 frames to the FFmpeg process, then finalizes it.
pub trait FrameSink {
    /// Writes exactly one complete frame. Implementations loop short writes
    /// (`write_all`) and surface pipe failures as `StreamingError::FrameWrite`.
    /// Frame length is simply forwarded; capacity-vs-size validation is the
    /// renderer's job, not the sink's.
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError>;

    /// Ends the frame stream, waits for FFmpeg to exit after stdin-EOF,
    /// verifies the MP4 was finalized, and consumes the sink. Success means
    /// exit status 0 AND an existing, non-empty output file — never less.
    fn finish(self) -> Result<CompletedVideo, StreamingError>;
}
```

### 1.8 `src/streaming/ffmpeg.rs` — process supervision (D2 drain-thread design)

```rust
//! Starts FFmpeg without a shell, keeps refusing pipe deadlocks by draining
//! stdout and stderr on dedicated threads, waits for clean stdin-EOF
//! finalization, and maps exit codes/diagnostic tails to typed errors.
//! Module docs record: drain threads are joined BEFORE `child.wait()` so a
//! fast-exiting FFmpeg can never fill its pipe while we wait; there is no
//! timeout in Task 1 (hang handling belongs to Task 2's pacing/shutdown
//! design); the child is killed on Drop so an abandoned process cannot leak
//! into the operator's machine.

pub struct FfmpegProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    output_path: PathBuf,
    stdout_drain: JoinHandle<CapturedPipe>,
    stderr_drain: JoinHandle<CapturedPipe>,
}

/// What a drain thread captured from one output pipe.
struct CapturedPipe {
    collected_bytes: Vec<u8>,
    final_error: Option<io::Error>,  // a read failure ends reading; bytes stay
}

impl FrameSink for FfmpegProcess { /* write_frame + finish */ }

impl FfmpegProcess {
    /// Builds arguments (validated config in, locked vector out), spawns the
    /// child process with piped stdin/stdout/stderr and no shell, then hands
    /// both output pipes to dedicated drain threads.
    pub fn start(config: &AppConfig, output_path: PathBuf) -> Result<Self, StreamingError>
}

impl Drop for FfmpegProcess {
    /// Kills a still-running child so an unrevisited FfmpegProcess cannot
    /// leak; kill errors on an already-exited process are ignored.
    fn drop(&mut self)
}
```

`start` sequence (each step small; helpers keep every function ≤ 50 lines and ≤ 2 levels deep):
1. `let arguments = args::build_arguments(config, &output_path)?;`
2. `let mut command = Command::new(&config.ffmpeg.executable);` then `.arg(...)` per argument — **no** `Command::new("sh").arg("-c", ...)`, no `current_dir`, no env mutation, `Stdio::piped()` for stdin, stdout, stderr.
3. `Child::spawn` → map to `StreamingError::Spawn { executable: <configured path text>, source }`.
4. `child.stdin.take()` — the piped handle is expected, but panic is not acceptable: if it is `None`, map to `StreamingError::FrameWrite { source: io::Error::other("stdin unexpectedly unavailable") }` in a single defensive `ok_or_else` site, documented inline.
5. Spawn drain threads: `stdout_drain = spawn_drain(child.stdout.take())`, same for stderr. Inside the thread: `read_to_end` into a `Vec<u8>`, remember any read error as `final_error`, return `CapturedPipe`. Constant `const DIAGNOSTIC_CAPTURE_LIMIT_BYTES: usize = 4096;` — applied NOT at capture time but when rendering `ExitFailure.diagnostics` (take the trailing 4096 bytes so the most recent — usually the error — lines are kept).
6. return the assembled struct.

`write_frame(&mut self, frame)` body:
1. `let Some(stdin) = self.stdin.as_mut() else { /* already finished/failed once */ return Err(StreamingError::FrameWrite { source: io::Error::other("the FFmpeg process has already been finalized") }) }` — (Rust 1.82 supports let-else; the exact construction is the implementer's, but the meaning is fixed: writes after `finish`/`Drop` consumption must type-error out, never misbehave).
2. `stdin.write_all(frame)` → `map_err` to `FrameWrite`. `write_all` already loops on short writes; a broken pipe is just an `io::Error` here — the typed Display adds the hint via the `is_broken_pipe` helper.

`finish(self)` body (consuming; sequence is fixed):
1. `self.stdin = None;` — dropping the handle closes FFmpeg's frame stream (this is the "close stdin to signal end-of-input" deliverable; FFmpeg then finalizes on natural EOF per V1).
2. `let stdout = self.stdout_drain.join()` and `stderr = self.stderr_drain.join()` — **before** `wait`, per the deadlock-order note above. Never `unwrap` a `JoinError`: route it through one helper `fn join_drain(handle: JoinHandle<CapturedPipe>) -> Result<CapturedPipe, StreamingError>` that maps a panicked thread to `WaitFailed` with `io::Error::other("a drain thread panicked")` (the helper takes exactly one argument).
3. `child.wait()` → `map_err(WaitFailed)`; on `status.success() == false` build `ExitFailure { exit_code: status.code(), diagnostics: trailing_bytes(&stderr, DIAGNOSTIC_CAPTURE_LIMIT_BYTES) }` — after steps 2/3, `final_error` info is folded into the diagnostics text when present (deterministic format: append `;\n(read failure: {error})`) — no secrets anywhere, and the argument vector is deliberately never echoed (D2: "no reckless-arg logging").
4. Finalization validation (single-section boolean via helper `is_output_finalized(&Path) -> bool`): `path.try_exists()` AND `metadata().len() > 0` → on failure `OutputFinalize { path }`; a probe ERROR (permissions) must not crash: treat probe error as NOT finalized (`Ok` unwrap_or(false) semantics through `try_exists` + metadata with `map_or(false, ...)`) — document the conservative direction.
5. Return `Ok(CompletedVideo { output_path })`.

Cap map: `FfmpegProcess` is the only `FrameSink` in this phase; trait object usage not needed (Task 2 passes it by value/concrete type).

### 1.9 Error-mapping matrix (malformed situations → exact variant)

| Situation | Variant |
| --- | --- |
| `config.video.pixel_format != "rgb24"` (defensive; unreachable via loader) | `UnsupportedPixelFormat { value }` |
| `std::env::current_exe()` fails | `ExecutableLocation { reason: <io error text> }` |
| `parent()` of the exe resolved nowhere (missing parent) | `ExecutableLocation { reason: "the executable path has no parent directory" }` |
| `create_dir_all(videos/…)` fails (including path exists as a file) | `OutputDirectoryCreate { path, source }` |
| All 1000 collision candidates taken | `OutputNameExhausted { directory, attempts: 1000 }` |
| `Command::spawn` fails (missing binary, permissions) | `Spawn { executable, source }` |
| stdin became unavailable after a prior finish/consumption (defensive) | `FrameWrite` with `io::Error::other` (see 1.8 step 4) |
| `write_all` fails (incl. broken pipe) | `FrameWrite { source }` |
| Drain `join` fails (thread panicked) | `WaitFailed` (see 1.8 step 2) |
| `child.wait()` fails | `WaitFailed { source }` |
| Non-zero exit / signal death | `ExitFailure { exit_code, diagnostics }` |
| Exit 0 but output missing/empty | `OutputFinalize { path }` |

---

## 2. Ordered steps (each is one atomic verifiable action)

Environment note: all `cargo` commands run through the Alpine VM MCP with `vm_status` first; the Compose command pattern is exactly `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`; exit status from the MCP is authoritative. Work happens on `feat/phase03-ffmpeg-local-pipeline` (already checked out).

### Step 0 — Baseline verification (no changes yet)

- [ ] VM: `alpine-vm_vm_status` → confirm running/SSH-reachable.
- [ ] VM: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0 (`ALL CHECKS PASSED`; 81 tests as of v0.6.0 baseline).
- [ ] Repo host (bash tool, single commands): `git status --porcelain` → expect clean; `git branch --show-current` → `feat/phase03-ffmpeg-local-pipeline`. `git log --oneline -3` → expect the version bump as the latest commit. Stop and report if any check differs.

### Step 1 — Error type skeleton

- [ ] Create `src/streaming/error.rs` exactly per §1.3 (all 10 variants; hand-written `Display`; `impl Error` with `source()`; helper fns `is_broken_pipe`, `is_file_not_found`, `exit_failure_description`, `trailing_bytes` can wait for their consumers in later steps — create only what compiles with zero dead-code warnings now, or gate unused private helpers behind their later users).
- [ ] Create `src/streaming/mod.rs` per §1.2 — for this step only declare `mod error;` (add other `mod` lines in their own steps so nothing is dead).
- [ ] Edit `src/main.rs`: add `#[allow(dead_code)] mod streaming;`.
- [ ] Deploy check: `docker compose … run --rm rust cargo check --locked` → exit 0.
- [ ] Commit: `feat: add streaming module with typed process errors` (stage only `src/streaming/error.rs`, `src/streaming/mod.rs`, `src/main.rs`; run `git status` first per gitignore-compliance rule — nothing under `videos/`, `logs/`, `dist/` may be staged).

### Step 2 — Pure argument builder (+ pixel-format guard)

- [ ] Add `mod args;` to `mod.rs`; write `src/streaming/args.rs` per §1.4 (module docs must include the R-number rationale references and the exact locked vector from §0.2).
- [ ] Create `src/streaming/args_tests.rs`; wire with `#[cfg(test)] #[path = "args_tests.rs"] mod args_tests;` at the bottom of `args.rs`.
- [ ] Use `crate::config::valid_config()` (already `pub(crate)` under cfg(test)) to build fixtures; then patch only the fields under test (e.g. set `video.width`/`height`/`fps` to distinctive values like 1280/720/25) so the exact vector is asserted verbatim per test.
- [ ] Deploy check: `docker compose … run --rm rust cargo test --locked streaming` (or `cargo test --locked` — either), exit 0 with the new tests.
- [ ] Commit: `feat: build ffmpeg raw-video arguments purely from config`.

### Step 3 — UTC timestamp formatter

- [ ] Add `mod utc_timestamp;`; implement `src/streaming/utc_timestamp.rs` per §1.5.
- [ ] Verify the leap-year anchor epoch **before writing the test** (search once: UNIX epoch of 2000-02-29T00:00:00Z) and put the source URL in the test's comment; keep the three hard anchors from §1.5 as-is.
- [ ] Deploy check: `cargo test --locked` partial run green.
- [ ] Commit: `feat: add pure utc timestamp formatting for recording names`.

### Step 4 — Output directory and collision-safe filenames

- [ ] Add `mod video_output;`; implement `src/streaming/video_output.rs` per §1.6 (`OutputKind`, `RecordingNameSpec`, directory resolve/create pair, `resolve_output_path` composition, `path_exists` conservative probe).
- [ ] Create `src/streaming/video_output_tests.rs` per §1.6 test list (use `tempfile::TempDir` for the fs pieces — dev-dep already pinned).
- [ ] Deploy check: `cargo test --locked` green (video_output tests included).
- [ ] Commit: `feat: resolve beside-exe video output with collision-safe names`.

### Step 5 — The frame sink seam

- [ ] Add `mod sink;`; write `src/streaming/sink.rs` per §1.7 (`FrameSink` trait, `CompletedVideo` with private field + `path()` accessor). Keep it small; no tests needed beyond doc-level (trait implemented in Step 6; behavioral tests are Task 5's fake-process scope).
- [ ] Deploy check: `cargo check --locked` green.
- [ ] Commit: `feat: define frame sink seam for ffmpeg frame delivery`.

### Step 6 — Process supervision

- [ ] Add `mod ffmpeg;`; implement `src/streaming/ffmpeg.rs` per §1.8 (`start`, drain threads + `CapturedPipe`, `write_frame`, `finish`, `Drop`, `DIAGNOSTIC_CAPTURE_LIMIT_BYTES`).
- [ ] Ensure `FrameSink`/`CompletedVideo`/`StreamingError` visibility: within-module use is fine since `mod.rs` keeps submodules private; no re-exports yet.
- [ ] Deploy checks (sequence, each exit-0): `cargo fmt --check`, `cargo check --locked`, `cargo clippy --locked -- -D warnings`. Any clippy finding on the process code must be fixed in the file, not by silencing globally.
- [ ] Commit: `feat: supervise ffmpeg process with drain threads and typed exits`.

### Step 7 — Ignore rule for generated videos (the only non-code file this task)

- [ ] Append to `.gitignore` (narrow rule only; no other sections touched):
  ```text
  # Local video recordings (ffmpeg output; never committed)
  videos/
  ```
- [ ] Repo host verify (bash tool, single commands): `git check-ignore -v videos/` → shows the new rule; `git status --porcelain` → `.gitignore` only as modified.
- [ ] Commit: `chore: ignore local videos output directory`.

### Step 8 — Full gate (Task 1's verification evidence)

- [ ] VM: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, `ALL CHECKS PASSED` (baseline 81 tests + all new Task-1 unit tests). Save the printed log path for the Task-5 report.
- [ ] Repo host: `git check-ignore -v videos/` (rule present), `git ls-files videos/` (empty), `git status --porcelain` (clean after all commits).
- [ ] explicit NOT-RUN record (goes in the Task 5 completion report): no real FFmpeg run happened in Task 1 (the pinned `rust:1.82` image has no ffmpeg and D-global says embedding/installing is forbidden); the process layer is compile/lint-verified only until Task 5's fake-process suite.

---

## 3. Unit test list (Task-1 deliverable, all passing before Step 8)

`args_tests.rs`
1. `valid_config_builds_exact_locked_vector` — asserts the full `Vec<String>` equality against §0.2 with a distinctive 1280x720@25 fixture.
2. `non_rgb24_pixel_format_is_rejected_defensively`— patch fixture to `yuv420p` (or any other string), expect `UnsupportedPixelFormat { value }` and the exact Display text.
3. `output_pixel_format_is_always_yuv420p` — the emitted output-side `-pix_fmt` arg is `yuv420p` even when the input side is `rgb24`.
4. `output_path_is_appended_last` — a path with an underscore-bearing collision-suffix name lands as the final element.

`utc_timestamp_tests.rs`
5. the four §1.5 anchor tests + `leap_day_tests_the_civil_conversion` (the verified 2000-02-29 epoch).

`video_output_tests.rs`
6. `filename_prefixes_match_outputs` (`demo` / `session`).
7. `videos_directory_is_placed_beside_the_executable` (pure mapping cases incl. missing-parent error).
8. `create_directory_makes_missing_directory` and `create_directory_reports_existing_file` (tempdir).
9. `next_available_returns_unsuffixed_name_when_free`.
10. `next_available_walks_collision_suffixes_deterministically` (`_2`, then `_3`).
11. `next_available_never_overwrites_when_every_candidate_exists` (asserts `OutputNameExhausted { attempts: 1000 }`).
12. `generated_base_name_matches_timestamped_format` (`demo-20200913-122640.mp4`).
13. `path_exists_treats_probe_errors_as_existing` (inject an `FnMut` that returns always-true; plus a doc comment stating the conservative direction).

`error_tests.rs` (new small module; wired like the others)
14. `each_variant_renders_its_display_contract` — one assertion per variant keyed on the exact strings from §1.3.
15. `source_returns_io_error_where_defined` — OutputDirectoryCreate/Spawn/FrameWrite/WaitFailed return `Some`, others `None`.

Process-running parts (spawn, stdin EOF, drain, exit status, fake-FFmpeg executable) are **Task 5 scope** — not built in Task 1 (global plan D1/Task order; the "fake FFmpeg" test doubles and optional real-FFmpeg smoke test belong to TODO §5).

---

## 4. Code-review/critical checks for steps 4.3 reviewer prompts

- No `sh -c`/`shell=true` anywhere; argument vector exactly §0.2 (order asserted by tests).
- `use std::process::{Child, ChildStdin, Command, Stdio};` only — no runtime crates.
- Files ≤ 200 lines, functions ≤ 50 lines, ≤ 2 nesting levels, ≤ 2 args (exception documented: `RecordingNameSpec::new` constructor and `next_available`'s `exists` closure with receiver).
- No commented-out code; secrets/args never logged (the argument vector must never appear in tracing/error text — review `ExitFailure.diagnostics` construction owns stderr text only).
- `#[allow(dead_code)]` only on `mod streaming;` in main.rs, with a note that it retires at Task 2 wiring.
- `.gitignore` diff is exactly the 3 appended lines.

---

## 5. Explicitly out of scope for Task 1 (must NOT appear in the diff)

- Frame pacing / render loop / ChatStore wiring / renderer invocation (Task 2).
- `demo` command, CLI parser changes, help/usage text (Task 3).
- `run`/`auth` handler changes (Task 4; they keep exiting 3 after config validation).
- README/docs/configuration/context/structure updates and the tracked completion report (Tasks 2–5; Task 1's only non-code file is `.gitignore`).
- Real-FFmpeg smoke test, fake-FFmpeg spawn tests, dev-checks-completion-report updates (Task 5).
- Audio, screenshots, stdin messaging, Youtube/RTMPS output.

---

## 6. Acceptance mapping (TODO §1 bullets → where they are defined)

| TODO Task 1 bullet | Plan section |
| --- | --- |
| streaming module: spawn/pipeline/typed errors | §1.7, §1.8, §1.3 |
| arguments w/o the shell w/ rgb24/video size/fps/codec/preset/bitrate | §0.2, §1.4 |
| finalized-on-exit reporting (no false success) | §0.2 (R6), §1.8 step 3–4, `OutputFinalize` variant |
| no pipe deadlock, no secret/arg leakage, useful diagnostics | §1.8 drain design, `DIAGNOSTIC_CAPTURE_LIMIT_BYTES` |
| videos/ beside executable, created on demand, typed errors | §1.6 |
| demo-/session- prefix naming, collision-safe, no overwrite | §1.6 (RecordingNameSpec + suffix budget) + `-n` defense (R7) |
| MP4 outputs never committed; .gitignore narrow rule | Step 7 |
| independence from YouTube/chat/render; small interface | §1.2 docs + `FrameSink` (§1.7) |

All Task-1 TODO bullets map present tense — the plan covers Task 1 completely. Tasks 2–5 remain untouched.
