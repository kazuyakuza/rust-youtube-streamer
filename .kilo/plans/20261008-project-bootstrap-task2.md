# Task 2 Plan — Update README.md with Initial Project Info (Rust YouTube Streamer Service)

- Date: 2026-10-08
- TODO: `.agent/todos/20261008/20261008-todo-1.md` → line item 2
- Global plan (binding): `.kilo/plans/20261008-project-bootstrap.md` — sections "Global Pre-Analysis" and "Task 2" (encoded decisions there are NOT re-decided here)
- Workflow: Critical Workflow, step 4.1b (Analysis & Planning). This plan is AUTO-APPROVED per user approval; no user interaction required at 4.2.
- Front-end: NO → skip steps 4.1a / 4.5a.
- Scope guard: This plan covers ONLY the second TODO line item. Task 1 (project-info files) is DONE and committed (`568b4ed`); Task 3 (folder structure) is out of scope.
- Single file constraint: only `C:\repo\rust-youtube-streamer\README.md` may be modified in step 4.2. Do NOT touch `.agent/`, `.kilo/`, `docs/`, or create any other file.

## 1. Goal

Replace the current generic template `README.md` with an initial project README for the **Rust YouTube Streamer Service**, using only facts from `.agent/project-info/brief.md` (source of truth), `.agent/project-info/product.md` (short product summary), and verified repository state. Target length ~60–90 lines. No TOC (file stays under 100 lines). No install/build/run documentation yet (the project is not scaffolded — no `Cargo.toml` exists).

## 2. Current State (verified)

- `README.md` is still the "Base Project for AI Agent Driven Development" template (171 lines): Compatibility, Prerequisites, About this Project, template Project Structure, Critical Workflow mermaid diagram, Agent Models, Getting Started (New Project Setup), How to Start a Task, AI Agent Plans, Troubleshooting.
- Branch `feat/project-bootstrap`; working tree clean; Task 1 project-info files committed.
- `.agent/project-info/` now contains: `brief.md`, `instructions.md`, `product.md`, `context.md`, `architecture.md`, `tech.md` (all exist; the README can link to them).
- No `Cargo.toml`/`Cargo.lock`; `src/` has only `.gitkeep` → README must NOT contain build/install/usage instructions.

## 3. Steps

### Step 1 — Rewrite `C:\repo\rust-youtube-streamer\README.md`

Use the `write` tool to overwrite the whole file (the current file was already read). UTF-8, real newlines (never literal `\n` escape sequences), no BOM concerns. The new file consists of exactly the 7 sections below, in this order. Do not add any other section; do not add a Table of Contents.

**Section 1 — Title + Purpose (H1)**

- Heading: `# Rust YouTube Streamer Service`
- One paragraph (3–5 prose lines, sourced strictly from brief §1.1):
  - Rust-based service that autonomously creates and manages a YouTube Live broadcast, generates video frames, and streams them to YouTube.
  - Initial MVP displays a black screen with incoming YouTube Live Chat messages rendered as white text; it validates the complete technical pipeline before any game logic, graphics, or interactive mechanics.
  - The repository is the architectural foundation for future YouTube chat-controlled games; the MVP is functionally simple but built as a modular, maintainable, testable, extensible service.

**Section 2 — `## MVP Summary`**

- Short bullets (from brief §1.1, §7.4, §10.1, §17; keep them factual):
  - Authenticate with YouTube via OAuth 2.0.
  - Create a new unlisted YouTube Live broadcast and its stream resources via the YouTube Data API.
  - Render continuous video frames in Rust: black background, incoming ordinary chat messages as white text (`username: message`), bounded in-memory visible-message queue.
  - Encode and transmit via an external FFmpeg process (raw RGB24 frames piped to FFmpeg stdin over H.264 → RTMPS ingestion when supported).
  - Close with one line (from brief §17): the result is not yet a game — it is the reusable streaming foundation for future chat-controlled games.

**Section 3 — `## Key Technologies`**

- Bullet list (from brief §3, §10.3, §13; no crate names beyond what the brief names):
  - **Rust** — primary language, idiomatic modules/structs/traits/error types.
  - **Tokio** — async runtime for API requests, chat streaming, task coordination, FFmpeg process management, graceful shutdown.
  - **FFmpeg (external process)** — Rust launches/supervises the executable and writes raw frames to stdin; FFmpeg does encoding and transmission. Executable path configurable per environment; Rust never embeds FFmpeg.
  - **YouTube Data API + OAuth 2.0** — broadcast lifecycle and chat integration behind the project's own module interfaces (`google-youtube3` crate rejected per brief §3.3; crate selection happens later during implementation).
  - **RTMPS ingestion** — TLS-protected delivery to YouTube Live.
  - **Platforms** — Windows and Linux development/execution; Ubuntu Server production target (brief §13.1).

**Section 4 — `## Current Status`**

- 2–4 lines, factual: initial setup — detailed README to come in future sessions. The repository is not yet scaffolded: no `Cargo.toml`/lockfile, `src/` empty; Cargo scaffolding, module skeleton, and the MVP pipeline are next steps. Point generically to the project-info files below for the current state (`context.md` is the living status log — mention that).

**Section 5 — `## AI Agents` (direct adaptation of the old README's "Attention AI Agents" pointer; keep it compact)**

- One bold/quoted directive line: all AI agents must read and follow [`AGENTS.md`](AGENTS.md) before any change.
- Bullet list of project-info pointers (relative links from repo root):
  - [`brief.md`](.agent/project-info/brief.md) — core requirements and source of truth.
  - [`product.md`](.agent/project-info/product.md) — problem, product goals, non-goals.
  - [`context.md`](.agent/project-info/context.md) — living status log (current focus, recent changes, next steps).
  - [`architecture.md`](.agent/project-info/architecture.md) — module map, data flow, boundary rules.
  - [`tech.md`](.agent/project-info/tech.md) — stack, external processes, dependency policy.
  - [`WORKFLOWS.md`](.agent/WORKFLOWS.md) and [`RULES.md`](.agent/RULES.md) — agent workflows and rules.
- One closing line noting both **Kilo Code** and **opencode** agent setups are supported (as stated in `AGENTS.md`); the tasks in the README of the template about `.kilo/`, `.opencode/` config directories are dropped — do not reproduce them.

**Section 6 — `## How to Start a Task` (preserved from old README § "How to Start a Task", trimmed)**

- Intro line: to initiate work with an AI agent, paste one of the following templates in the chat (works in both Kilo Code and opencode).
- Keep the two snippet patterns verbatim as fenced `text` blocks:
  - `### Option 1: Using a TODO File (Recommended)` — with the block:

    ```text
    full read @AGENTS.md & follow /critical-workflow
    do @/.agent/todos/<YYYYMMDD>/<YYYYMMDD>-todo-<number>.md
    ```

  - `### Option 2: Direct Chat Request` — with the block:

    ```text
    full read @AGENTS.md & follow /critical-workflow
    do [Your specific task or request here]
    ```

- One closing line pointing to [`AGENTS.md`](AGENTS.md) and [`WORKFLOWS.md`](.agent/WORKFLOWS.md) for the full Critical Workflow (dedicated sub-agents: planner, architector, implementer, code reviewer/simplifier, docs specialist). Do NOT copy the mermaid diagram, Agent Models, or AI Agent Plans sections from the old README.

**Section 7 — Footer line**

- Single italic line: `*Initial README — detailed documentation (setup, configuration, streaming recovery, operations) will be added in future sessions.*`

### Step 2 — Verify (no build/test exists; markdown-only verification)

Re-read the new `README.md` and check:

1. Line count is between 60 and 90 total lines; no Table of Contents present.
2. Real newlines used (no literal `\n` in file content); UTF-8.
3. All relative links resolve to existing files: `AGENTS.md`, `.agent/WORKFLOWS.md`, `.agent/RULES.md`, and the five `.agent/project-info/*.md` targets.
4. Zero template remnants: no occurrences of "Base Project", "template", "kilocodeignore", "Agent Models", "Troubleshooting", or the mermaid diagram.
5. Zero install/build/run instructions (no `cargo build`, no FFmpeg install steps — those are explicitly deferred; brief §13.3 only requires FFmpeg docs later).
6. No features beyond the brief (no invented capabilities, no roadmap items not in `context.md` next steps).

### Step 3 — `.agent/project-info/context.md` upkeep: NOT part of step 4.2 (assign to step 4.4 docs-specialist)

- Step 4.2 (implementer) must NOT create or modify any file other than `README.md` — this is a binding constraint for this task.
- The mandated context upkeep from `.agent/project-info/instructions.md` § "Critical Closing Step" is instead executed at step 4.4 (Documentation, docs-specialist), which owns documentation updates per the Critical Workflow. Exact instruction for the docs-specialist: append one dated bullet under `## Recent Changes (2026-10-08)` in `.agent/project-info/context.md`: `README.md replaced with initial project info for the Rust YouTube Streamer Service (detailed README deferred to future sessions).`
- Rules for that bullet: do not reorder existing bullets; do not remove existing content (overwrite-todo prevention rule applies to `context.md` content); do not change `## Immediate Next Steps` (item 2 of the TODO is marked `[DONE]` only at step 4.6; the remaining item — folder structure — stays).
- If the docs-specialist considers this out of its allowed scope, it must report the pending `context.md` bullet back to the Planner Agent instead of editing outside scope — never edit beyond assigned scope.

### Step 4 — No commit in this step

- Git staging/commit happens at step 4.6 (Task Completion) by the implementer. Before that commit, obey `.kilo/rules/gitignore-compliance.md`: run `git status`, ensure only `README.md` (plus the `context.md` bullet from step 4.4) is staged.

## 4. Out of Scope (must NOT be done)

- Any edits to `.agent/` files at step 4.2 (in particular: NOT `brief.md`, NOT `context.md`, NOT `architecture.md`/`tech.md`/`instructions.md`; the single `context.md` bullet is a step 4.4 docs-specialist action only).
- Anything under `.kilo/`, `docs/`, or root files other than `README.md` (`.gitignore`, folder creation are Task 3).
- Any Cargo scaffolding, source code, or front-end work.
- Detailed README sections (setup, configuration, OAuth procedure, FFmpeg install, docs/ links) — explicitly deferred to future sessions.

## 5. Review checklist (4.3 / 4.5b)

- Only `README.md` (step 4.2) plus the single `context.md` bullet (step 4.4) differ from HEAD at 4.6 staging; step 4.2 alone must have touched ONLY `README.md` (`git status` / `git diff --stat` check).
- README contains exactly the 7 sections specified above, in order, within 60–90 lines; no TOC.
- Every claim in the README is traceable to brief.md §1.1/§3/§7.4/§10/§13/§17 or `product.md`; no invented features.
- All relative links valid; AGENTS.md pointer and both Critical Workflow snippets present verbatim.
- Status line present: initial setup — detailed README to come in future sessions.
- `brief.md`, `product.md`, `architecture.md`, `tech.md`, `instructions.md` untouched.
