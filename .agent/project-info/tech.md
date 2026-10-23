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
* Decided during Phase 04 task 1 (OAuth): `oauth2 = { version = "5.0", default-features = false, features = ["ureq"] }` (locked 5.0.0) + `ureq = "2"` (locked 2.12.1) — the `oauth2` crate is an actively maintained OAuth 2.0 client covering the authorization-code flow, PKCE, and token refresh with a pluggable HTTP transport; its `ureq` feature keeps the whole OAuth path synchronous (no Tokio enters the dependency tree), compatible with Rust 1.82 / edition 2021, Windows+Linux, MIT OR Apache-2.0. `reqwest` was rejected because it pulls an async runtime; `google-youtube3` stays REJECTED per brief §3.3. Tasks 2–5 added NO further dependencies.
* The effective-authorization verifier probes `GET https://www.googleapis.com/youtube/v3/channels?part=id&mine=true` with the Bearer header and maps 200/401/403 (plus error bodies, classified status-first) into typed outcomes; loopback-test evidence pointer: `.kilo/plans/20261012-phase04-task4-adherence.md`.
* Decided during Phase 05 (production live runtime): NO new dependencies in this phase — no `Cargo.toml` dependency line changed and `Cargo.lock` is untouched (the phase's only `Cargo.toml` change is the version bump to 0.8.0); the live API transport is the concrete `GoogleLiveApiTransport` over the already-locked ureq 2: a redirects-disabled (`redirects(0)`) ureq agent, OAuth 2.0 Bearer authentication only (NEVER an API key), and NO request-level timeout (documented limitation — one stalled TCP connection can hang one call); JSON POST bodies are sent via `send_string(body.to_string())` with an explicit `Content-Type: application/json` because the locked ureq is pinned WITHOUT the `json` feature (enabling it would contradict the no-new-dependency ruling; wire bytes are identical, pinned by loopback body assertions). The bodyless bind POST (O3) transmits `Content-Length: 0` with zero body bytes via ureq 2 `send_bytes(&[])`, loopback-pinned after the HTTP 411 incident log of 2026-10-22 — no dependency change. Attempt-bounded wait constants: readiness 36 × 5 s (180 s) and post-start confirmation 24 × 5 s (120 s). Lifecycle ruling: broadcasts/streams are created with `enableAutoStart=true` / `enableAutoStop=true`, no monitor stream, and the binary contains NO `transition`/`delete` call ever. `RunError` exit mapping: clean stop and pre-streaming stop → 0; every fatal failure → 1 (one redacted `error:` line on stderr).

## Lifecycle (production run)

One `run` invocation creates one new broadcast + one live-stream pair from the validated config, binds them, waits for readiness, acquires the RTMPS destination (never logged), starts FFmpeg toward RTMPS, confirms `streamStatus == active` on a thread parallel with the chat-free frame loop, and streams until Ctrl+C; shutdown order is stop flag → wait cancellation → FFmpeg stdin EOF → wait. Resource fate: never deleted/transitioned by the app; `enableAutoStop` auto-completes the broadcast about a minute after ingestion ends; startup failures leave `ready` resources (documented accumulation limitation). Operator guide: `docs/production-run.md`.

Run shutdown diagnostics (v0.10.0, run-diagnostics Task 1): the run shutdown now surfaces FFmpeg's exit status and the destination-redacted stderr tail as `info` (clean exit) / `warn` (failure status) tracing lines — the redaction happens at capture time in the streaming `finish()`, and the evidence survives the gate-timeout shutdown path where the gate error outranks the pipeline outcome; on a confirmation-gate timeout the gate performs ONE tolerated final stream lookup and logs the last observed `streamStatus` label (or the lookup-failure fact) via `info`. The logging lives app-side (`src/app/run/`): the streaming boundary keeps its std+config-only dependency rule, and NO dependency changed (the task's only `Cargo.toml` line is the 0.10.0 version). This is evidence-gathering for the next run's diagnosis, not a claimed root-cause fix; the mid-loop broken-pipe frame-write path still surfaces only the single `error:` line (documented residual limitation).

## Configuration

JSON configuration is loaded at startup and validated before starting the broadcast; invalid configuration must result in a clear error and a non-zero exit code (brief §5). `config/config.example.json` is versioned; the real `config.json`, which carries secrets and OAuth credentials, must stay out of version control (brief §6.4).

Made-for-kids broadcast config (v0.10.0, made-for-kids Task 2): `youtube.broadcast.made_for_kids` (optional boolean) maps verbatim into the O2 `liveBroadcasts.insert` body as `status.selfDeclaredMadeForKids` — a documented insert-settable property per the official liveBroadcasts insert reference ("You can set values for these properties: … status.selfDeclaredMadeForKids"); when ABSENT the field is never sent (`skip_serializing_if`, explicit rename — not the read-only `madeForKids`) and YouTube applies its default handling, in practice the channel-level made-for-kids setting. NO dependency change (serde's typed `Option<bool>` is the whole contract; no validation rule added — the documented rule count stays 21); absent/true/false body shapes and the config→command flow are pinned offline (loopback/fake; no test reaches Google).

## Platforms

Windows and Linux development and execution; Ubuntu Server production target (brief §13). First manual validation may occur on Windows. Portable path handling, configurable executable paths, no hardcoded path separators. Docker is optional; the service must not require it to run natively.

## Authentication Modes

Two execution modes (brief §6):

* `auth` — interactive OAuth 2.0 authorization-code flow: load the OAuth client configuration, show the authorization URL, receive the callback, exchange the code for tokens, verify access to required YouTube resources, persist credentials, and exit.
* `run` — load persisted credentials, refresh expired access tokens when possible, verify scopes/permissions, fail clearly when authorization is missing, revoked, or insufficient; non-interactive (unattended) where Google's OAuth policies allow.
