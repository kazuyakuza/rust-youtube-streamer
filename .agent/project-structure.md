# Project Structure

# Folders in src/

- app/ - application orchestration: CLI parsing and dispatch, startup pipeline, logging initialization, error boundaries, the `demo` mode and the frame pipeline
  - `cli.rs` - std-library CLI parser: `auth` | `demo` | `run` | `--config <path>` (before/after the command) | `--help`; `DEFAULT_CONFIG_PATH` constant
  - `cli_tests.rs` - parser unit tests (15 cases: valid/invalid forms, default path, demo usage text)
  - `error.rs` - `StartupError { Usage, Configuration, LogInit }` with exit-code mapping (2/1/1) and Display contracts
  - `logging.rs` - tracing-subscriber init seam: `RUST_LOG` honored, default filter `info`, invalid value → one stderr warning + default
  - `mod.rs` - startup funnel: parse → init logging → load+validate config → dispatch (incl. `demo`); 3 whitelisted-field info events; `report_failure` single stderr line
  - `modes.rs` - `auth`/`run` placeholder handlers (config validated first; report not-implemented; exit 3); the `run` message names the four missing YouTube prerequisites (OAuth authorization, broadcast lifecycle, RTMPS ingestion URL acquisition, YouTube live chat integration), denies interactive input and any local fallback, and points to the `demo` command
  - `demo/` - local-only demonstration mode (6 files): predefined messages at deterministic intervals through the shared pipeline (never contacts YouTube)
    - `mod.rs` - `run_demo(config) -> u8`: installs the single Ctrl+C handler first, runs the shared pipeline against `OutputKind::Demo`, prints `demo recording finalized: <path>` only after verified finalization; `DemoError { Interrupt, Pipeline }` both map to exit 1 (the startup-failure class; `demo` never exits 3)
    - `schedule.rs` - fixed timeline: 28 messages (`DEMO_MESSAGE_TOTAL`), 30-frame black lead-in, at most one message per 15 frames, 60-frame tail (final frame index 495 → 496 rendered frames ≈ 16.5 s at 30 FPS); ids `demo-NN` hidden from screen; long-line message index 15; exceeds the example 26-line visible capacity so FIFO eviction is exercised
    - `source.rs` - `DemoMessageSource`: at most one scheduled message per call, stop requested at the final frame
    - `mod_tests.rs` / `schedule_tests.rs` / `source_tests.rs` - 14 unit tests (exit mapping, capacity/eviction schedule facts, clock-free source behaviour)
  - `pipeline/` - application-level frame pipeline: chat → renderer → streaming (13 files: 7 sources + 5 test files + `test_helpers.rs`)
    - `mod.rs` - `FramePipeline`, `prepared_pipeline` construction seam, `PipelineConstruction`, re-exports (`StopFlag`, error types)
    - `clock.rs` / `pacing.rs` - monotonic frame clock and FPS-paced schedule (backlog dropped to one immediate frame; `interval_nanos` exact division)
    - `runner.rs` - the frame loop: continuous frames at configured FPS, at most one new message per frame, complete-frame writes with short-write/broken-pipe propagation, ordered sink finalization
    - `interrupt.rs` - `StopFlag` (`Arc<AtomicBool>` clone-shared) + the app's single `ctrlc` handler install; the 5 documented shutdown limitations live in its module docs
    - `error.rs` - frame-size guards and typed `FramePipelineError` (chat stage, rendering, output)
    - `source.rs` / `test_helpers.rs` - chat-source seam and shared pipeline test doubles
    - `*_tests.rs` - unit tests at every seam (exact byte counts, empty first frames, rotation/eviction, stop-before-start/after-Nth-write, error mapping)
- config/ - loading, validation and exposure of the application configuration
  - `mod.rs` - public API: `load_from_path(&Path) -> Result<AppConfig, ConfigError>`; module docs
  - `model.rs` - strongly typed structs mirroring `config/config.example.json` (serde derive, `deny_unknown_fields`)
  - `loader.rs` - read → parse → validate pipeline
  - `validation.rs` - 18 validation checks grouped as named predicates (5 positive dimensions/FPS/size values, exact `rgb24`, 1 vertical layout ≥ 1 line, 10 non-blank strings/paths incl. FFmpeg fields, 1 privacy enum)
  - `error.rs` - `ConfigError { Io, Deserialization, Validation }` + aggregated `ConfigIssue` (typed, actionable, no panics)
  - `loader_tests.rs` / `validation_tests.rs` - 16 unit tests (temp files via `tempfile`; no real developer config)
  - `test_fixtures.rs` - shared test fixture builder (`#[cfg(test)]`)
- youtube/ - YouTube API integration: OAuth auth, broadcast lifecycle, streams, chat transport (placeholder; future phase). `src/youtube/.gitkeep` is a DELIBERATE tracked placeholder for the future YouTube module — do not delete it.
- chat/ - application-level chat message model and bounded visible-message queue (ChatStore); independent of YouTube API types, network, streaming and FFmpeg (implemented and wired into the runtime through the app frame pipeline and the `demo` mode)
  - `mod.rs` - module docs: log-append-first contract and scope authority (TODO §1); private submodule declarations (re-exports return at the first external consumer)
  - `model.rs` - `ChatMessage { author, text, message_id: Option, timestamp: Option<u64> }`; builder attachers for optional API metadata; `Display` is exactly `author: text` (metadata never visible); newline normalization flattens line breaks to one space
  - `store.rs` - `ChatStore`: `VecDeque` FIFO of visible messages, log file opened eagerly in append/create mode; `accept` appends the log record FIRST and rejects the message on I/O failure (queue and log never diverge); evicts oldest beyond capacity; `visible_text` joins `author: text` lines with `'\n'`; new stores start empty, the log is never read back
  - `error.rs` - `ChatError { Capacity, LogOpen, LogAppend }`; hand-written `Display` contracts, `std::error::Error::source`, no panics
  - `model_tests.rs` / `store_tests.rs` - 16 unit tests (display/normalization/metadata; capacity, FIFO eviction, append-only log ordering and I/O-failure semantics; temp files via `tempfile`)
- renderer/ - converts RenderInput to complete raw RGB24 frames (width * height * 3 bytes, row-major, R/G/B); independent of YouTube API types, chat state, network and FFmpeg (implemented and wired into the runtime through the app frame pipeline and the `demo` mode)
  - `mod.rs` - module docs; `#[path = "renderer.rs"] mod contract;` mapping keeps the trait out of `clippy::module_inception`
  - `renderer.rs` - `RenderInput { text }` (complete visible multiline text) and the `Renderer` trait (`render -> Result<Frame, RenderError>`); validation happens at construction, `render` fails only on frame-allocation overflow
  - `settings.rs` - `RendererSettings::from_config` (re-checks positive dimensions, vertical layout ≥ 1 line, parses `#RRGGBB` `Rgb` case-insensitively) and `Rgb` with uppercase `Display`; no file-existence check (Phase 01 precedent)
  - `frame.rs` - `Frame` with checked `width * height * 3` construction (`filled`), `blend_at` no-op outside the frame, deterministic round-half-up coverage blending
  - `text.rs` - `TextRenderer` (ab_glyph `FontVec`, font parsed once at construction, missing/unparseable fonts are typed errors, no silent fallback); line-box placement baseline = `top_margin + ascent + i * line_height`; per-pixel clipping at the physical frame boundary, horizontal cut at `x == width`, no wrapping/font resizing; unknown codepoints render as notdef, skipped with advance kept
  - `error.rs` - `RenderError { Geometry, FrameSize, Layout, InvalidColor, FontOpen, FontParse }`; hand-written `Display`, `source()`, no panics
  - `*_tests.rs` / `text_test_helpers.rs` - 19 unit tests (`#[cfg(test)]` in-tree test modules; font fixture `fonts/Exo2-Light.otf`)
- streaming/ - FFmpeg process supervision and raw-frame input pipeline (13 files: 7 sources + 6 test files), sealed boundary rules: depends only on `std` + `crate::config::model`, never on chat/renderer/YouTube types, and receives frames as opaque `&[u8]` through the `FrameSink` seam
  - `mod.rs` - module docs + boundary rules; `pub(crate)` re-exports (`StreamingError`, `FfmpegProcess`, `FrameSink`/`CompletedVideo`, `resolve_output_path`/`OutputKind`)
  - `args.rs` - pure locked argument vector built from validated config (`-nostdin -n -f rawvideo … -movflags +faststart <output>`); no shell, no env, no filesystem
  - `error.rs` - typed `StreamingError` (pixel-format guard, executable location, output directory/name, Spawn, FrameWrite, WaitFailed, ExitFailure with diagnostics tail, OutputFinalize); hand-written `Display`/`source()`; 4096-byte trailing diagnostics capture
  - `ffmpeg.rs` - `FfmpegProcess`: spawn without a shell with piped stdin/stdout/stderr, dedicated drain threads joined before `wait`, `finish()` = stdin EOF → drains → exit-status check → non-empty-file check; `Drop` kills and reaps
  - `sink.rs` - `FrameSink` trait (`write_frame`/`finish`) and `CompletedVideo` — the only types the pipeline sees
  - `video_output.rs` - `resolve_output_path`: beside-executable `videos/` directory created on demand; collision-safe `demo-`/`session-` names (`_2.._1000`); conservative existence probe
  - `utc_timestamp.rs` - `YYYYMMDD-HHMMSS` UTC timestamp (civil-from-days, leap handling)
  - `*_tests.rs` - unit tests incl. `ffmpeg_tests.rs` (6 fake-FFmpeg POSIX-script process tests, `#[cfg(all(test, unix))]`) and `ffmpeg_real_tests.rs` (1 real-FFmpeg smoke: 64×36@5fps record + ffprobe assertions, loud image-tool prerequisite)

# Root files

- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.6.0, edition 2021; deps: ab_glyph (renderer font rasterization), ctrlc (cross-platform Ctrl+C flag-flip handler for the pipeline stop seam), serde (derive), serde_json, tracing, tracing-subscriber (env-filter); dev-dep: tempfile
- Cargo.lock - tracked lockfile for the executable package (sole root-level generated artifact)
- src/main.rs - entry point delegating to the `app` layer (`mod app; mod chat; mod config; mod renderer; mod streaming;` — no `#[allow(dead_code)]` attributes; they were retired when Phase 03 wired chat/renderer into the runtime)
- Dockerfile - pinned official `rust:1.82` base; installs rustfmt/clippy components for checks, `gcc-mingw-w64-x86-64` (Windows GNU cross toolchain: fallback linker route + objdump import audit) and the `x86_64-pc-windows-gnu` Rust target for the Windows cross-build
- docker-compose.yml - `rust` service for Docker-based cargo checks; bind-mounts the project and redirects build artifacts to the `rust-streamer-target` named volume
- .gitattributes - forces LF line endings for `*.sh` and `Dockerfile` so container-executed files survive the repo host's `core.autocrlf=true` checkout behavior

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/ (per-task implementation plans tracked)
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- config/ - runtime JSON configuration files; `config.example.json` is tracked; real `config.json` is gitignored
- credentials/ - OAuth credentials and token storage location (no files committed here)
- docs/ - Documentation files: `configuration.md` (configuration setup + CLI usage guide incl. the `demo` command), `demo.md` (local demo recording guide: timeline, prerequisites, running, output, Ctrl+C, out-of-scope, verification evidence), `build.md` (Linux release build guide: prerequisites, standard invocation, output, failure semantics, limitations), `build-windows.md` (Windows x86-64 GNU cross-build guide: prerequisites incl. image rebuild, invocation, output, failure semantics, toolchain/target choice, runtime DLLs, limitations), plus agent how-to guides
- scripts/ - developer utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory; `build-linux.sh` builds the release with the tracked lockfile in the same service and copies the final Linux executable to gitignored `dist/`; `build-windows.sh` cross-compiles the Windows x86-64 GNU release in the same service (`cargo build --release --locked --target x86_64-pc-windows-gnu`) and copies the final .exe to gitignored `dist/windows/`
- fonts/ - font assets: `Exo2-Light.otf` + `OFL.txt` (SIL Open Font License 1.1; vendored deterministic renderer-test fixture with provenance and SHA-256 recorded in `fonts/README.md`) and the pre-existing tracked `console.ttf` (provenance indeterminate, referenced by no test; the configured runtime font path remains operator-supplied)
- logs/ - versioned folder placeholder `logs/.gitkeep` only (restored in Phase 03 Task 5); ALL runtime logs are untracked — the gitignored `logs/checks/<UTC>.log` dev-check gate logs and the chat log `logs/chat.log` are never committed
- dist/ - gitignored release-build output; `build-linux.sh` writes only the final Linux executable `rust-youtube-streamer-service` here; `build-windows.sh` writes only the final Windows executable `rust-youtube-streamer-service.exe` under `dist/windows/`; the explicitly approved exception to the no-root-generated-files rule (contents never committed)
- videos/ - gitignored runtime MP4 output directory (rule `videos/`): recordings are created BESIDE THE EXECUTABLE at run time (never beside the repo binary in this tree), so no MP4 ever appears in the repository tree; contents never committed
