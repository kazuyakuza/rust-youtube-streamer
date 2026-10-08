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

Initial setup — detailed README to come in future sessions.
The repository is not yet scaffolded: no `Cargo.toml`/lockfile exists and `src/` is empty; Cargo scaffolding, the module skeleton, and the MVP pipeline are the next steps.
For the current state, see the project-info files below — `context.md` is the living status log.

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
