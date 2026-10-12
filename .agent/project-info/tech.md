# Tech

> Source of truth: [`brief.md`](brief.md) — if this summary conflicts with the brief, the brief wins; see [`instructions.md`](instructions.md) for project-info rules.

## Stack

* Primary language: Rust — idiomatic use of modules, structs, enums, traits, ownership, and explicit error types (brief §3.1).
* The frame pipeline and FFmpeg supervision are synchronous (std::process + std threads); Tokio remains reserved for later YouTube API/chat streaming phases (task-2 decision S2).

## External Processes

FFmpeg runs as an external, centrally configurable process (brief §3.4). The application launches and supervises the executable, configures its arguments, and writes raw RGB24 frames to its standard input; FFmpeg owns video encoding and transmission. The executable path is configurable per environment — `C:\ffmpeg\bin\ffmpeg.exe` and `/usr/bin/ffmpeg` are examples, not hardcoded paths. Encoding defaults (brief §10.3): codec `libx264`, preset `veryfast`, bitrate `6000k`, 30 FPS, 1920×1080 — all configurable.

Ctrl+C is a shutdown request handled by the `ctrlc = "3"` crate (cross-platform flag-flip handler, MIT OR Apache-2.0, pulls platform deps `nix` on Unix / `windows-sys` on Windows) through the shared `StopFlag` seam (`Arc<AtomicBool>` polled at the top of every frame iteration); documented shutdown limitations live in `src/app/pipeline/interrupt.rs` module docs. The locked FFmpeg argument vector and its rationale are recorded in `.kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md` §0.2; the exact arguments and the natural-EOF finalization are verified against the dev-image FFmpeg version by the container smoke tests (OBSERVED version and results recorded in the phase-03 completion report, `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`).

## Dependencies (policy)

* The `google-youtube3` crate is REJECTED as a foundation — its maintenance status is unsuitable (brief §3.3).
* Evaluate maintained Rust libraries for HTTP requests/response handling, JSON serialization/deserialization, OAuth 2.0 authorization and token refresh, and gRPC/Protocol Buffers if required for the selected chat API. Use a direct YouTube API client implementation where necessary.
* YouTube integration stays isolated behind the application's own module interfaces so the underlying HTTP, OAuth, or gRPC implementation can be replaced without changing the renderer or application logic.
* Config and logging (brief §3.5): JSON configuration loading and validation, structured logging, error context, OAuth token persistence, graceful shutdown — favor actively maintained, widely used crates with appropriate licenses; pin dependencies via the Cargo lockfile; avoid redundant dependencies.
* Exact crate selection happens during implementation, not now.
* Decided during Phase 02 (renderer): `ab_glyph = "0.2"` for font rasterization — pure Rust, no platform-installed graphics libraries, Rust 1.82/edition-2021 compatible, Windows+Linux, pinned via the tracked lockfile; replaces nothing and is independent of YouTube/streaming crates.
* Decided during Phase 03 task 2 (pipeline): `ctrlc = "3"` for the Ctrl+C shutdown request — cross-platform, no async runtime (its platform dependencies are `nix` on Unix and `windows-sys` on Windows, pulled transitively), MIT OR Apache-2.0; justifies the minimal cross-platform interrupt approach required by the TODO.

## Configuration

JSON configuration is loaded at startup and validated before starting the broadcast; invalid configuration must result in a clear error and a non-zero exit code (brief §5). `config/config.example.json` is versioned; the real `config.json`, which carries secrets and OAuth credentials, must stay out of version control (brief §6.4).

## Platforms

Windows and Linux development and execution; Ubuntu Server production target (brief §13). First manual validation may occur on Windows. Portable path handling, configurable executable paths, no hardcoded path separators. Docker is optional; the service must not require it to run natively.

## Authentication Modes

Two execution modes (brief §6):

* `auth` — interactive OAuth 2.0 authorization-code flow: load the OAuth client configuration, show the authorization URL, receive the callback, exchange the code for tokens, verify access to required YouTube resources, persist credentials, and exit.
* `run` — load persisted credentials, refresh expired access tokens when possible, verify scopes/permissions, fail clearly when authorization is missing, revoked, or insufficient; non-interactive (unattended) where Google's OAuth policies allow.
