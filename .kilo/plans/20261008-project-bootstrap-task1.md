# Task 1 Plan — Initialize Project Info Core Files (Rust YouTube Streamer Service)

- Date: 2026-10-08
- TODO: `.agent/todos/20261008/20261008-todo-1.md` → line item 1
- Global plan (binding): `.kilo/plans/20261008-project-bootstrap.md` — sections "Global Pre-Analysis" and "Task 1" (encoded decisions there are NOT re-decided here)
- Workflow: Critical Workflow, step 4.1b (Analysis & Planning). This plan is AUTO-APPROVED per user approval; no user interaction required at 4.2.
- Front-end: NO → skip steps 4.1a / 4.5a.
- Scope guard: This plan covers ONLY the first TODO line item. Do not touch README.md (Task 2) or folder structure (Task 3).

## 1. Goal

Create the 4 missing core project-info files required by `.agent/project-info/instructions.md` § "Core Files (Required)", derived strictly from `brief.md` (source of truth) and verified repository facts, and remove the template initialization marker. Markdown only — no code, no `Cargo.toml`, no Cargo scaffolding.

## 2. Current State (verified)

- `.agent/project-info/` contains exactly: `brief.md`, `instructions.md`, `.initialized` (marker file, no content).
- Missing: `product.md`, `context.md`, `architecture.md`, `tech.md`.
- `src/` is empty (only `.gitkeep`); no `Cargo.toml`/`Cargo.lock` exists anywhere — the project is NOT yet scaffolded. `tech.md` must state this factually.
- Branch: `feat/project-bootstrap` (created in global Step 2).
- Do NOT modify `brief.md` or `instructions.md`.

## 3. Steps

### Step 1 — Create `C:\repo\rust-youtube-streamer\.agent\project-info\product.md`

Title: `# Product — Rust YouTube Streamer Service`. Sections (all content sourced from brief §1; do not invent features beyond the brief):

1. `## Problem` — From brief §1.1: there is no validated pipeline for autonomously creating/managing a YouTube Live broadcast from Rust and streaming generated frames to it; existing `google-youtube3` crate is unuitable (brief §3.3). The MVP must validate the complete technical pipeline.
2. `## Product Description` — One paragraph from brief §1.1 and §17: Rust service that authorizes via OAuth 2.0, creates a new unlisted YouTube Live broadcast, generates continuous video frames from Rust (MVP: black screen with incoming chat messages as white text), encodes via external FFmpeg process, transmits over RTMPS.
3. `## Core Objectives` — Bullet list of the 10 numbered objectives in brief §1.2 (quote/paraphrase each, keep all 10: OAuth auth, broadcast resource management, valid stream from startup, render chat onto black background, continuous FFmpeg encode/transmit, bounded visible-message queue, separate chat log file, Windows+Linux parity, predictable error/shutdown handling, clean interfaces for future games).
4. `## Non-Goals (MVP)` — Bullet list of the 9 exclusions in brief §1.3 (game mechanics, chat commands/moderation/authorization, sprites/animations/GUI, audio, persistent storage, multiple concurrent broadcasts, web dashboard, distributed/microservices, automatic crash recovery). Close with the brief's line: the goal is to prove the streaming architecture end to end, not to build the game.
5. `## Future Direction` — 2–3 sentences from brief §2.3/§17: architecture is the foundation for future chat-controlled games (Chat Processing → Command Processing → Game State → RenderInput pipeline); streaming infrastructure must not need replacement.

Guideline: keep under ~120 lines.

### Step 2 — Create `C:\repo\rust-youtube-streamer\.agent\project-info\context.md`

Title: `# Context`. This is a factual log (per instructions.md § "Core Files": current work focus, recent changes, immediate next steps). Only verified facts:

1. `## Current Work Focus` — Project bootstrap: initializing project info core files from the newly written brief; repository not yet scaffolded (no `Cargo.toml`, `src/` empty).
2. `## Recent Changes` (dated 2026-10-08, brief factual bullets):
   - `brief.md` defined as complete technical brief and source of truth for the Rust YouTube Streamer Service.
   - Project-info template customized to this project (`instructions.md` core-file structure in place).
   - `.initialized` template marker present (list as present; the implementer removes it in the same task — see Step 6 of this plan; write it as "being removed during this initialization").
   - Git branch `feat/project-bootstrap` created for bootstrap work.
3. `## Immediate Next Steps` (exact order, matching remaining TODO items):
   - Update `README.md` with initial project info (TODO item 2).
   - Create initial structure folders with `.gitkeep` placeholders per brief §4 (TODO item 3).
   - After bootstrap: Cargo scaffolding, module skeleton under `src/`, then MVP pipeline implementation.
4. `## Notes` — Point to `brief.md` as source of truth; note status = "initial setup — detailed README and docs coming later".

Guideline: keep short (<80 lines).

### Step 3 — Create `C:\repo\rust-youtube-streamer\.agent\project-info\architecture.md`

Title: `# Architecture`. Source: brief §2 and §4 (binding layout). Sections:

1. `## Style` — Modular monolith (brief §2.1): single Rust executable, well-defined internal modules, explicit responsibilities, communication via types/interfaces/controlled dependencies. Principles list (concise): separation of concerns, dependency inversion at boundaries, explicit configuration, strong typing, testable logic, structured error handling, minimal shared mutable state, async only where beneficial, no unnecessary abstractions/frameworks, no microservices.
2. `## Module Map` — Table or list, exactly these 6 modules with responsibilities from brief §4 "Module responsibilities":
   - `src/app/` — orchestrates startup, task execution, error handling, shutdown.
   - `src/config/` — loads, validates, exposes JSON configuration.
   - `src/youtube/` — owns authentication (OAuth), broadcast lifecycle, stream configuration, chat integration.
   - `src/chat/` — application-level chat message types; bounded FIFO queue (`VecDeque<ChatMessage>`) of visible messages.
   - `src/renderer/` — converts RenderInput into raw RGB24 video frames; trait contract (`Renderer` / `TextRenderer`, brief §9.2). Must NOT depend on YouTube API or broadcast lifecycle.
   - `src/streaming/` — starts and supervises FFmpeg; manages raw frame input pipeline.
3. `## Data Flow` — State that the authoritative diagram lives in brief §2.2, reproduce it (copy the ASCII diagram block verbatim from brief §2.2), and summarize: YouTube → Chat → ChatStore → RenderInput → Renderer → raw frames → FFmpeg stdin → YouTube. Note briefly §2.3 future evolution (Command Processing / Game State may be inserted between chat store and renderer; not implemented in MVP).
4. `## Boundary Rules` (critical paths/configs from brief): 
   - Renderer must not depend on YouTube-specific types or API behavior (brief §2.2, §9.1).
   - FFmpeg knows nothing about chat/users/game mechanics; receives raw frames via stdin only (brief §2.2, §10.1–10.2). Rust never implements H.264/RTMPS itself.
   - Chat store is transient in-memory state, never persisted/restored; chat log is independent of the queue (brief §8.4, §11.2).
   - Secrets (tokens, stream keys, client secrets) never logged or committed.
5. `## Application Lifecycle` — Brief §14.1 state chain: Starting → ConfigurationValidated → Authenticated → BroadcastPrepared → Streaming → Stopping → Stopped; transitions logged. Concurrent tasks: render loop, chat listener, FFmpeg process, shutdown mechanism (brief §14.2).
6. `## Status` — Not yet scaffolded; folder layout will be created during bootstrap; brief §4 tree is the target.

Guideline: keep under ~150 lines.

### Step 4 — Create `C:\repo\rust-youtube-streamer\.agent\project-info\tech.md`

Title: `# Tech`. Source: brief §3, §5, §13. Sections:

1. `## Stack` — Primary language: Rust (idiomatic: modules, structs, enums, traits, ownership, explicit error types — brief §3.1). Async runtime: Tokio (API requests, chat streaming, task coordination, process management/stdin writes, graceful shutdown — brief §3.2); keep synchronous components synchronous (e.g., single-frame rendering).
2. `## External Processes` — FFmpeg as an external, centrally configurable process (path configurable per environment; examples `C:\ffmpeg\bin\ffmpeg.exe` / `/usr/bin/ffmpeg` are examples, not hardcoded — brief §3.4). App launches, supervises, writes raw RGB24 frames to stdin; defaults: libx264 / veryfast / 6000k / 30 FPS / 1920×1080 (brief §10.3), all configurable.
3. `## Dependencies (policy)` — brief §3.3: `google-youtube3` crate REJECTED (maintenance status unsuitable). Evaluate maintained HTTP, JSON, OAuth 2.0, and gRPC/Protobuf crates; direct YouTube API client implementation where necessary; YouTube integration isolated behind module interfaces. Config/logging: JSON config loading + validation, structured logging, error context, token persistence, graceful shutdown — favor actively maintained, widely used crates; pin via Cargo lockfile; no redundant dependencies (brief §3.5). Note: exact crate selection happens during implementation, not now.
4. `## Configuration` — JSON config at startup, validated before broadcast; invalid config ⇒ clear error + non-zero exit; `config/config.example.json` versioned, real `config.json` git-ignored; secrets/credentials never committed (brief §5).
5. `## Platforms` — Windows and Linux development/execution; Ubuntu Server production; first manual validation on Windows; portable path handling, configurable executable paths, no hardcoded separators (brief §13). Docker optional, not required.
6. `## Authentication Modes` — Two modes: `auth` (interactive OAuth authorization-code flow, token persistence) and `run` (load/refresh persisted credentials, non-friendly-unattended where possible) (brief §6).
7. `## Current Status` — Factual: project NOT yet scaffolded — no `Cargo.toml`, no lockfile, `src/` empty. Scaffolding occurs after bootstrap (TODO items 2–3 and later sessions).

Guideline: keep under ~120 lines.

### Step 5 — Verify files (no terminal needed)

Re-read the 4 created files; check each:
- Content is traceable to brief.md sections / verified repo facts; no invented features.
- Not modified: `brief.md`, `instructions.md`.
- Real newlines in files (no literal `\n`), UTF-8.
- Line counts within guidelines.

### Step 6 — Remove marker file

- Delete `C:\repo\rust-youtube-streamer\.agent\project-info\.initialized` (per instructions.md initialization flow: "Remove `.agent/project-info/.initialized` file").
- Confirm deletion (directory listing shows only the 5 files: brief.md, instructions.md, product.md, context.md, architecture.md, tech.md).

### Step 7 — No commit in this step

- Git staging/commit happens in step 4.6 (Task Completion) by the implementer per the workflow. Do not commit here. `.initialized` deletion will be included in the 4.6 commit.

## 4. Out of Scope (must NOT be done)

- README.md changes (Task 2), folder structure creation (Task 3), `.gitignore` edits, `.agent/project-structure.md` updates (Task 3), any Cargo scaffolding, any source code.

## 5. Review checklist (4.3 / 4.5b)

- `brief.md` and `instructions.md` untouched (git diff empty for them at 4.6).
- `.initialized` deleted; 4 new files present with the exact section outlines above.
- No contradictions between the 4 files and brief.md (if conflict, brief.md wins).
- No code files created.
