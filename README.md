# Rust YouTube Streamer Service

A Rust-based service that autonomously creates and manages a YouTube Live broadcast, generates video frames, and streams them to YouTube.
The initial MVP displays a black screen with incoming YouTube Live Chat messages rendered as white text; it validates the complete technical pipeline before any game logic, graphics, or interactive mechanics.
The repository is the architectural foundation for future YouTube chat-controlled games; the MVP is functionally simple but built as a modular, maintainable, testable, and extensible service.

## MVP Summary

- Authenticate with YouTube via OAuth 2.0.
- Create a new unlisted YouTube Live broadcast and its stream resources via the YouTube Data API.
- Render continuous video frames in Rust: black background, incoming ordinary chat messages as white text (`username: message`), bounded in-memory visible-message queue.
- Encode and transmit via an external FFmpeg process (raw RGB24 frames piped to FFmpeg stdin over H.264 → RTMPS ingestion when supported).

The result is not yet a game — it is the reusable streaming foundation for future chat-controlled games.

## Key Technologies

- **Rust** — primary language; idiomatic modules, structs, traits, and explicit error types.
- **Tokio** — async runtime for API requests, chat streaming, task coordination, FFmpeg process management, and graceful shutdown.
- **FFmpeg (external process)** — Rust launches and supervises the executable and writes raw frames to its stdin; FFmpeg performs encoding and transmission. The executable path is configurable per environment; Rust never embeds FFmpeg.
- **YouTube Data API + OAuth 2.0** — broadcast lifecycle and chat integration behind the project's own module interfaces (`google-youtube3` crate rejected per brief §3.3; crate selection happens later during implementation).
- **RTMPS ingestion** — TLS-protected delivery to YouTube Live.
- **Platforms** — Windows and Linux development/execution; Ubuntu Server production target.

## Current Status

Phase 00 (repository foundation and build baseline) is complete:
a minimal Cargo package exists (`rust-youtube-streamer-service`, Rust edition 2021, no dependencies)
with `src/main.rs` as the smallest buildable entry point. Build checks run inside Docker on the
Alpine VM — see [Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm) below.
No application feature (OAuth, chat, rendering, streaming) exists yet; each feature module will be
added by its own phase.

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

Then run each check via the `rust` Compose service:

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
