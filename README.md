# Rust YouTube Streamer Service

A Rust-based service that autonomously creates and manages a YouTube Live broadcast, generates video frames, and streams them to YouTube.
The initial MVP displays a black screen with incoming YouTube Live Chat messages rendered as white text; it validates the complete technical pipeline before any game logic, graphics, or interactive mechanics.
The repository is the architectural foundation for future YouTube chat-controlled games; the MVP is functionally simple but built as a modular, maintainable, testable, and extensible service.

## Table of Contents

- [MVP Summary](#mvp-summary)
- [Key Technologies](#key-technologies)
- [Current Status](#current-status)
- [Commands and Configuration](#commands-and-configuration)
- [Logging (RUST_LOG)](#logging-rust_log)
- [Local Demo Recording](#local-demo-recording)
- [Authentication (OAuth 2.0 — Key Features)](#authentication-oauth-20--key-features)
- [Production Run (YouTube Live)](#production-run-youtube-live)
- [Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm)
- [Release Build (Linux Artifact)](#release-build-linux-artifact)
- [Release Build (Windows .exe)](#release-build-windows-exe)
- [AI Agents](#ai-agents)
- [How to Start a Task](#how-to-start-a-task)

## MVP Summary

- Authenticate with YouTube via OAuth 2.0.
- Create a new unlisted YouTube Live broadcast and its stream resources via the YouTube Data API.
- Render continuous video frames in Rust: black background, incoming ordinary chat messages as white text (`username: message`), bounded in-memory visible-message queue.
- Encode and transmit via an external FFmpeg process (raw RGB24 frames piped to FFmpeg stdin over H.264 → RTMPS ingestion when supported).

The result is not yet a game — it is the reusable streaming foundation for future chat-controlled games.

## Key Technologies

- **Rust** — primary language; idiomatic modules, structs, traits, and explicit error types.
- **Tokio (`tokio` 1.53.1 — locked)** — the streaming chat client's runtime, CONFINED to the single dedicated chat worker thread as a current-thread runtime; the frame pipeline and FFmpeg supervision remain synchronous (`std::process` + std threads, with the `ctrlc` crate for Ctrl+C). The pin rationale (1.54.x needs Rust ≥ 1.85 > the pinned 1.82; crates.io registry metadata, accessed 2026-10-29) is recorded in the Phase 07 research record §4.
- **gRPC + Protocol Buffers (`tonic` 0.13.1 + `prost` 0.13.5 — locked)** — the official `liveChatMessages.streamList` server-streaming client (rustls `ring` provider + bundled `webpki-roots`, matching the existing ureq TLS family; Bearer-only OAuth; NO API key). Hand-authored prost message types transcribed from Google's officially published proto (no codegen step, no `protoc`); `tonic::async_trait` re-exports the seam traits. Registry access dates 2026-10-29; versions from the tracked lockfile pins.
- **OAuth 2.0 (`oauth2` 5.0 with the `ureq` feature + `ureq` 2.x)** — synchronous OAuth client for the installed-application authorization-code flow with PKCE and token refresh; the OAuth path itself pulls in no async runtime (`google-youtube3` remains rejected per brief §3.3; the only Tokio in the dependency tree is the Phase 07 chat stream client's runtime, confined to the chat worker thread).
- **FFmpeg (external process)** — Rust launches and supervises the executable and writes raw frames to its stdin; FFmpeg performs encoding and transmission. The executable path is configurable per environment; Rust never embeds FFmpeg.
- **YouTube Data API** — broadcast lifecycle and chat integration behind the project's own module interfaces (`google-youtube3` crate rejected per brief §3.3; later phases reuse the OAuth grant established by `auth`).
- **RTMPS ingestion** — TLS-protected delivery to YouTube Live.
- **Platforms** — Windows and Linux development/execution; Ubuntu Server production target.

## Current Status

Phase 00 (repository foundation and build baseline), Phase 01 (typed configuration, CLI, logging), Phase 02 (chat store + RGB24 renderer), Phase 03 (FFmpeg local recording pipeline) and Phase 04 (OAuth 2.0 credential layer) are complete. Phase 05 (production live runtime) is implemented — the `auth`, `demo`, and `run` commands are all real: `auth` performs the Google installed-application OAuth 2.0 flow, `demo` runs the local-only recording, and `run` creates and binds one new broadcast + live-stream resource pair from the validated `youtube.broadcast.*` metadata, readiness-checks with bounded, cancellable waits, acquires the RTMPS ingestion destination (never logged), and streams continuous rendered frames — with incoming YouTube Live Chat messages rendered as author/text when present, black frames while chat is idle — through the configured external FFmpeg executable over RTMPS. Shutdown is ordered (stop flag + wait cancellation + FFmpeg stdin EOF + wait, then the bounded chat-worker join at the gate finish), and the broadcast auto-completes on YouTube via `enableAutoStop` (this application never auto-deletes or auto-transitions broadcasts/streams). Phase 06 (Live Chat integration) landed, and Phase 07 (streamList migration, v0.13.0) swapped its retrieval to the official server-streaming `liveChatMessages.streamList` gRPC client: the connection is established once per session and re-established only after drops carrying the accumulated `nextPageToken` (never re-called on a schedule), pushed messages render on the next frame iteration after arrival, and the polling surfaces were retired. The operator-facing chat documentation and the Phase 06/07 completion reports are landed; the real-broadcast manual verification (now including the streaming build's push-latency check) remains OPERATOR-RUN and was not executed by the agent.

The containerized test suite is green (`ALL CHECKS PASSED`, container-only; 494 tests, re-recorded by the Phase 07 verification pass). Automated verification is container-only: native Windows/Linux runtime execution and the live YouTube end-to-end run were NOT performed by the agent — the manual production verification is operator-run and recorded in [`docs/production-run.md`](docs/production-run.md).

FFmpeg remains a separately provided external prerequisite — it is never installed or bundled by this project; the `demo` command launches the executable at the configured `ffmpeg.executable` path and surfaces launch or process failures as typed errors. The operator's FFmpeg build must include libmp3lame (the standard full builds do) for the streaming vector's MP3-CBR silent track; a build lacking it fails at stream start with the clear spawn-error path.

Build checks and both release builds (the Linux artifact and the cross-compiled Windows x86-64 `.exe`) run inside the Linux container only; there is still no native Windows/Linux runtime validation.

## Commands and Configuration

Usage (full text available via `--help`):

```
usage: rust-youtube-streamer-service [--config <path>] <command>
```

- `auth` — performs the OAuth authorization flow using the Google OAuth client file (`oauth.client_config_path`) and saves tokens to the configured credentials path (`oauth.credentials_path`); exits 0 on verified success; exits 1 on any failure (configuration errors included; never leaks secrets). Full guide: [`docs/authentication.md`](docs/authentication.md).
- `run` — streams the configured broadcast to YouTube Live: creates one new broadcast and live-stream resource from `youtube.broadcast.*`, waits for readiness, acquires the RTMPS ingestion destination (never logged), and sends continuous rendered frames through the configured external FFmpeg executable; incoming ordinary Live Chat messages render as author/text on the frames when present (see the [Live Chat section](docs/production-run.md#live-chat-behavior-diagnostics-recovery-privacy-quota) of the production-run guide for behavior, diagnostics, recovery, privacy, and quota details). Success stops cleanly with exit 0 on Ctrl+C; any startup or runtime failure exits 1 with one redacted `error:` line; never interactive. Full guide: [`docs/production-run.md`](docs/production-run.md).
- `demo` — runs a local-only FFmpeg demo recording (no YouTube, OAuth, broadcast, or chat connection): predefined messages at deterministic intervals are rendered into an MP4 under `videos/` beside the executable. **Success exits 0 after the MP4 is finalized**, printing `demo recording finalized: <path>` on stdout; any demo failure prints one `error:` line on stderr and exits 1; `demo` never exits 3. Full guide: [`docs/demo.md`](docs/demo.md).
- `--config <path>` — configuration file to load; accepted before or after the command; default `config/config.json` when absent.
- `--help` — prints usage to stdout; exit 0.
- Exit codes: 0 = `--help`, a successful demo recording, a verified `auth` success, or a clean run stop; 1 = configuration or logging startup failure, a demo runtime failure, any auth failure, or any run fatal failure; 2 = usage error.

Create the default `config/config.json` by copying `config/config.example.json` (the committed example, safe non-secret values) and editing it — `config/config.json` is gitignored and must never be committed. Full field-by-field reference: [`docs/configuration.md`](docs/configuration.md).

FFmpeg is an external prerequisite installed separately; the `demo` command launches the executable at the configured `ffmpeg.executable` path and surfaces launch/process failures as typed errors. FFmpeg is never bundled.

## Logging (RUST_LOG)

- Logs go to standard error as structured events via `tracing`/`tracing-subscriber`.
- The level filter comes from the environment variable `RUST_LOG` (exact, case-sensitive name).
- Default is `info` when `RUST_LOG` is unset, empty, or whitespace-only — applied silently, with no warning.
- An invalid non-empty `RUST_LOG` value additionally prints exactly one `warning:` line to stderr and then still falls back to `info`.
- A valid value (e.g. `debug`, `trace`) is honored verbatim after trimming surrounding whitespace.

Example:

```
RUST_LOG=debug rust-youtube-streamer-service run
```

## Local Demo Recording

The `demo` command produces a local demonstration MP4 without contacting YouTube:

1. Copy `config/config.example.json` to `config/config.json`.
2. Set `ffmpeg.executable` to the installed FFmpeg path (FFmpeg is an external prerequisite installed separately, never bundled) and `renderer.font` to a real, parseable font file (e.g. the OFL-licensed `fonts/Exo2-Light.otf`).
3. Run `rust-youtube-streamer-service demo`.

Results land in `videos/` beside the executable (never the launch working directory), created on demand, with collision-safe `demo-YYYYMMDD-HHMMSS.mp4` names — no existing recording is overwritten. Success prints `demo recording finalized: <path>` only after FFmpeg has exited 0 and the MP4 has been verified finalized; any failure prints one `error:` line on stderr and exits 1.

Ctrl+C acts as a shutdown request with documented limitations. Full guide (timeline, output details, Ctrl+C semantics, out-of-scope list, and verification evidence): [`docs/demo.md`](docs/demo.md).

## Authentication (OAuth 2.0 — Key Features)

The `auth` command performs the Google OAuth 2.0 installed-application authorization flow (authorization code + PKCE, loopback callback):

1. **Prerequisite**: a Google OAuth 2.0 "Desktop app" client, downloaded as JSON to `oauth.client_config_path` (default `credentials/client_secret.json`). This file is operator-supplied and gitignored; it is distinct from the issued-credentials file.
2. Run `rust-youtube-streamer-service auth`. The console prints the authorization URL — open it in a browser, sign in, and approve the authorization prompt.
3. The local callback listener is bound to `127.0.0.1` only, on an ephemeral port; it is closed deterministically on success and on every failure. No interactive input is ever read and no browser is auto-launched.
4. After approval the app validates the OAuth state (CSRF protection), exchanges the authorization code (PKCE S256), validates offline that the token response granted the scope `https://www.googleapis.com/auth/youtube.force-ssl` and carries a refresh token (the live `channels.list(mine=true)` probe is the credential service's verify-first path for unattended runs), and saves the credentials atomically to `oauth.credentials_path` (default `credentials/tokens.json`).
5. Success = one stdout line (`authorization succeeded: YouTube credentials saved at <path>; the required YouTube scope was granted`) and exit 0. Every failure is one redacted stderr `error:` line and exit 1 — secrets, tokens, and authorization codes never appear in logs or errors.
6. **Unverified-app screen**: the first authorization with a development client may show Google's `Google hasn't verified this app` dialog. This is expected for a local desktop client — choose **Advanced** and continue to authorize.

**Persistence and permissions**: credentials are written to a temporary file in the same directory and renamed into place (atomic on POSIX); the created file mode is `0600` (owner-only) on platforms that support POSIX permissions. Honest limitations: Windows uses inherited ACLs (the Rust standard library has no ACL management — there is no Windows-specific permission hardening), there is **no encryption** of the stored file, and the Linux container test suite cannot exercise the Windows rename/permission arms.

**Unattended refresh**: future sessions load the stored credentials, verify the effective authorization first (no clock-based refresh decision), refresh an expired access token from the stored refresh token when possible, and persist the refreshed credentials atomically. Long-unattended operation depends on the issued refresh credentials and Google's current OAuth policies.

**Failure and recovery**: missing, unreadable, malformed, revoked, expired/unrefreshable, and insufficient-scope credentials each produce a distinct redacted error. The recovery path for revoked or unrefreshable refresh credentials is to re-run `auth`.

**Security**: neither `credentials/client_secret.json` nor `credentials/tokens.json` is ever committed (`credentials/*` is gitignored; only `.gitkeep` is tracked); nothing secret is logged. Never copy real client secrets or tokens into the repository tree.

Full guide (schema, exit codes, failure classes, verification evidence and known limitations): [`docs/authentication.md`](docs/authentication.md). Standing limitation: no live Google authorization was performed during this phase — the flow is validated offline with fakes and loopback HTTP servers only, and native Windows/Linux runtimes were NOT RUN.

## Production Run (YouTube Live)

The `run` command is the production streaming mode: it consumes the Phase-04 credential service (load → verify-first → refresh-once → persist → re-verify), creates ONE new broadcast and ONE new live-stream resource from the validated `youtube.broadcast.*` metadata, binds them, waits for readiness (bounded, cancellable), acquires the RTMPS ingestion destination (never logged), and streams continuous rendered frames through the configured external FFmpeg executable over RTMPS. A bounded live-chat worker consumes the created broadcast's Live Chat over the server-streamed connection and feeds incoming ordinary messages into the frame pipeline: they render as author/text when present, and the frames stay black while chat is idle. The operator-facing chat documentation is landed (see the [Live Chat section](docs/production-run.md#live-chat-behavior-diagnostics-recovery-privacy-quota): prerequisites, behavior, diagnostics, recovery, privacy, quota); the real-broadcast manual verification stays operator-run.

Prerequisites in one line: authorized credentials at `oauth.credentials_path` (run `auth` first), a filled-in `config/config.json`, and a separately installed FFmpeg at `ffmpeg.executable` (external, never bundled).

Logs stay useful without secrets: the human-visible lines (`broadcast prepared for streaming` with the non-secret broadcast id, `FFmpeg started toward the configured streaming destination`, `stream confirmed active on YouTube`, then `public watch link for the live stream` carrying `https://www.youtube.com/watch?v=<broadcast_id>` — a non-secret URL built from the broadcast id; an unlisted broadcast is watchable by anyone holding the link) carry no tokens, stream keys, or RTMPS URLs — those are never printed. On Ctrl+C the shutdown is ordered (frame loop stop → FFmpeg stdin EOF → wait) and exits 0; the broadcast then auto-completes on YouTube via `enableAutoStop` — this application never deletes or transitions broadcasts/streams itself, and repeated failing runs can accumulate stale resources that operators delete in YouTube Studio.

Full operator guide (startup stages, log lines, shutdown semantics, broadcast fate, failure recovery, and the manual verification procedure): [`docs/production-run.md`](docs/production-run.md). Standing limitation: the manual unlisted-broadcast end-to-end procedure is operator-run and was NOT executed by the agent; native Windows/Linux runtime execution remains NOT RUN, and building the artifacts validates no runtime behavior.

## Build Checks (Docker via Alpine VM)

All Rust/Cargo commands execute inside a Linux container on the Alpine VM. The host machine that
runs the AI agents does NOT need Rust or Cargo installed.

### Prerequisites

- Alpine VM running, with the project shared at `/rust-youtube-streamer` on the VM.
- Docker with the Compose plugin available on the VM (observed: Docker Compose v2.31.0).
- AI agents access the VM through the `alpine-vm` MCP: call `vm_status` first, then run commands.

### Commands

Build the image once:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml build rust
```

Standard way — one command runs every check via the repository script `scripts/dev-checks.sh`
(POSIX sh; build the image once first, as shown above):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

The script runs `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` and
`cargo clippy --locked -- -D warnings` in that order and prints a per-check
`PASS|FAIL <name> exit=<code> (<duration> s)` line plus a final summary (`ALL CHECKS PASSED` or
`N CHECK(S) FAILED`); its exit code is `0` when all checks pass and `1` when any check fails.
Output is also written to a UTC-timestamped log under `logs/checks/` (a gitignored directory)
whose path is printed. The script never modifies source files and never applies `cargo fmt`
fixes.

Failure-path validation hook: `FORCE_FAIL` set to `fmt-check`, `check`, `test` or `clippy` makes
exactly that check fail with a synthetic exit code 7 while the other checks still run. Leave
`FORCE_FAIL` unset in normal use. Example:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh
```

Fallback/reference — run the individual checks manually via the `rust` Compose service:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings
```

Run checks sequentially when a later command needs output from an earlier one.

### Notes

- Build artifacts are redirected to a Docker named volume (`rust-streamer-target` via
  `CARGO_TARGET_DIR`); no `target/` directory is created in the project root. The only generated
  artifact in the root is `Cargo.lock`, which is tracked in git.
- Container image: official `rust:1.82` with the `rustfmt` and `clippy` components added.
- These checks validate the build inside the Linux container only. They are NOT native Windows or
  native Linux runtime validation; Windows/Linux runtime behavior is out of scope for this phase.
- MCP-based agents: cargo's stderr (e.g., `Finished` lines) may not appear in captured MCP/VM
  output. The command exit status is the authoritative success evidence; a missing `Finished` line
  is not a failure.

## Release Build (Linux Artifact)

To produce a distributable release executable (not just validate the code), run the repository's
build script through the same Compose `rust` service used for the checks above:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

On success the command exits `0` and leaves a Linux release executable at `dist/rust-youtube-streamer-service`
(a gitignored directory). It compiles inside the pinned Linux Rust container, so the artifact is a Linux
executable — not a Windows `.exe`, and building it does not validate native Windows/Linux runtime behavior.
Full prerequisites, output path, rebuild/overwrite behavior, failure semantics and limitations:
[`docs/build.md`](docs/build.md).

This is a different job from `scripts/dev-checks.sh`: that script validates formatting/types/tests/lints
and produces no artifact, while the build command produces the distributable executable.

## Release Build (Windows .exe)

To produce a distributable Windows x86-64 release executable, run the repository's Windows build
script through the same Compose `rust` service (build the image once first per the Build Checks
prerequisites above):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
```

On success the command exits `0` and leaves the Windows release executable at
`dist/windows/rust-youtube-streamer-service.exe` (a gitignored directory), cross-compiled for
`x86_64-pc-windows-gnu` inside the pinned Linux container using the MinGW-w64 GNU toolchain. The GNU
runtime is statically linked, so no MinGW runtime DLLs are needed on Windows. This is a
cross-compilation: it does NOT validate native Windows runtime behavior — the `.exe` must be tested
separately on Windows (reported as NOT RUN). FFmpeg remains an external prerequisite, not bundled.
This is a different job from `scripts/dev-checks.sh` (Linux-container checks, no artifact) and from
`scripts/build-linux.sh` (Linux artifact); `scripts/build-windows.sh` produces only the Windows
`.exe` above. Full prerequisites, output path, rebuild/overwrite behavior, failure semantics,
toolchain/target choice, runtime DLLs and limitations: [`docs/build-windows.md`](docs/build-windows.md).

## AI Agents

**All AI agents must read and follow [`AGENTS.md`](AGENTS.md) before any change.**

- [`brief.md`](.agent/project-info/brief.md) — core requirements and source of truth.
- [`product.md`](.agent/project-info/product.md) — problem, product goals, non-goals.
- [`context.md`](.agent/project-info/context.md) — living status log (current focus, recent changes, next steps).
- [`architecture.md`](.agent/project-info/architecture.md) — module map, data flow, boundary rules.
- [`tech.md`](.agent/project-info/tech.md) — stack, external processes, dependency policy.
- [`WORKFLOWS.md`](.agent/WORKFLOWS.md) and [`RULES.md`](.agent/RULES.md) — agent workflows and rules.

Both **Kilo Code** and **opencode** agent setups are supported.

## How to Start a Task

To initiate work with an AI agent, paste one of the following templates in the chat (works in both Kilo Code and opencode).

### Option 1: Using a TODO File (Recommended)

```text
full read @AGENTS.md & follow /critical-workflow
do @/.agent/todos/<YYYYMMDD>/<YYYYMMDD>-todo-<number>.md
```

### Option 2: Direct Chat Request

```text
full read @AGENTS.md & follow /critical-workflow
do [Your specific task or request here]
```

For the full Critical Workflow (dedicated sub-agents: planner, architector, implementer, code reviewer/simplifier, docs specialist), see [`AGENTS.md`](AGENTS.md) and [`WORKFLOWS.md`](.agent/WORKFLOWS.md).

*Initial README — detailed documentation (setup, configuration, streaming recovery, operations) will be added in future sessions.*
