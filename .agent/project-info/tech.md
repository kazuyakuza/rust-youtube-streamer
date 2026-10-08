# Tech

> Source of truth: [`brief.md`](brief.md) — if this summary conflicts with the brief, the brief wins; see [`instructions.md`](instructions.md) for project-info rules.

## Stack

* Primary language: Rust — idiomatic use of modules, structs, enums, traits, ownership, and explicit error types (brief §3.1).
* Async runtime: Tokio for YouTube API requests, chat streaming, coordination between application tasks, process management and standard-input writes, and graceful shutdown/task supervision (brief §3.2). Synchronous components stay synchronous where simpler (e.g., rendering a single frame).

## External Processes

FFmpeg runs as an external, centrally configurable process (brief §3.4). The application launches and supervises the executable, configures its arguments, and writes raw RGB24 frames to its standard input; FFmpeg owns video encoding and transmission. The executable path is configurable per environment — `C:\ffmpeg\bin\ffmpeg.exe` and `/usr/bin/ffmpeg` are examples, not hardcoded paths. Encoding defaults (brief §10.3): codec `libx264`, preset `veryfast`, bitrate `6000k`, 30 FPS, 1920×1080 — all configurable.

## Dependencies (policy)

* The `google-youtube3` crate is REJECTED as a foundation — its maintenance status is unsuitable (brief §3.3).
* Evaluate maintained Rust libraries for HTTP requests/response handling, JSON serialization/deserialization, OAuth 2.0 authorization and token refresh, and gRPC/Protocol Buffers if required for the selected chat API. Use a direct YouTube API client implementation where necessary.
* YouTube integration stays isolated behind the application's own module interfaces so the underlying HTTP, OAuth, or gRPC implementation can be replaced without changing the renderer or application logic.
* Config and logging (brief §3.5): JSON configuration loading and validation, structured logging, error context, OAuth token persistence, graceful shutdown — favor actively maintained, widely used crates with appropriate licenses; pin dependencies via the Cargo lockfile; avoid redundant dependencies.
* Exact crate selection happens during implementation, not now.

## Configuration

JSON configuration is loaded at startup and validated before starting the broadcast; invalid configuration must result in a clear error and a non-zero exit code (brief §5). `config/config.example.json` is versioned; the real `config.json`, which carries secrets and OAuth credentials, must stay out of version control (brief §6.4).

## Platforms

Windows and Linux development and execution; Ubuntu Server production target (brief §13). First manual validation may occur on Windows. Portable path handling, configurable executable paths, no hardcoded path separators. Docker is optional; the service must not require it to run natively.

## Authentication Modes

Two execution modes (brief §6):

* `auth` — interactive OAuth 2.0 authorization-code flow: load the OAuth client configuration, show the authorization URL, receive the callback, exchange the code for tokens, verify access to required YouTube resources, persist credentials, and exit.
* `run` — load persisted credentials, refresh expired access tokens when possible, verify scopes/permissions, fail clearly when authorization is missing, revoked, or insufficient; non-interactive (unattended) where Google's OAuth policies allow.
