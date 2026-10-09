# Global Plan — Phase 01: Configuration and Application Skeleton

- Date: 2026-10-09
- Status: PENDING USER APPROVAL
- TODO source: `.agent/todos/20261009/20261009-todo-3.md`
- Critical Workflow: `.kilo/commands/critical-workflow.md`
- Source of truth: `.agent/project-info/brief.md`

## Task Origin Analysis

- TODO format: Pattern C (`# Title` → `## Tasks` → `### Heading`). Each `###` in `## Tasks` is one task.
- Tasks (file order):
  1. Implement the Configuration Module
  2. Establish the Application and CLI Skeleton
  3. Establish Structured Logging and Error Boundaries
  4. Update Documentation and Project Metadata
- Front-end related: NONE (all tasks are backend/config/CLI/docs). Sub-steps 4.1a and 4.5a are omitted for every task.
- The TODO does NOT contain the string "Don't request me to approve plans" → user approval of the global plan is required before execution.

## Global Pre-Analysis (verified repository facts)

- Repo state: branch `main`, up to date with `origin/main` (HEAD `53e4846`).
- Pending unstaged/untracked changes (user renamed the Phase 01 TODO): deleted `.agent/todos/20261009/20261009-todo-2.md`, untracked `.agent/todos/20261009/20261009-todo-3.md`. Step 2 commits them (meaningful message, e.g. `docs: replace phase 01 todo with revised todo-3`).
- Baseline: Rust 2021 package `rust-youtube-streamer-service` v0.1.0, zero dependencies, `src/main.rs` Phase 00 placeholder, `config/` empty except `.gitkeep`, `scripts/dev-checks.sh` runner, pinned Docker `rust:1.82` compose service.
- Execution environment: no Rust/Cargo on the Windows host. ALL cargo commands run through the Alpine VM `alpine-vm` MCP (`vm_status` first) with the canonical single-command invocation `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cargo cmd>` or the standard `sh scripts/dev-checks.sh`. Exit code is authoritative.
- Containers must not generate files in the project root; `Cargo.lock` is the sole root exception. Any new runtime output paths belong under `logs/` (gitignored).
- Version policy: feature phase → semver minor bump `0.1.0` → `0.2.0` in `Cargo.toml` (Step 3, restricted to that step); `Cargo.lock` refreshed via container `cargo check --locked`... executed as plain `cargo check` first to regenerate, then `--locked` must pass. Commit `chore: bump version to 0.2.0`.
- Branch: `feat/phase01-config-app-skeleton` created from `main` (Step 2, restricted to that step). Merge back to `main` at Step 5; push `origin` only (Step 5).
- Dependency policy: maintained crates, suitable licenses, lockfile workflow. No Tokio this phase (no concrete async requirement). No `google-youtube3`. No empty placeholder feature modules. Justify every new dependency in the completion report.
- Coding rules: files ≤ 200 lines, functions ≤ 50 lines, ≤ 2 params (param objects in new files when needed), max nesting depth 2, single-section boolean conditions (extract named predicates), no commented-out code, self-documenting names, private members by default.
- Task 2/Task 3 interplay (handled in per-task plans): TODO order is preserved (1 → 4). Task 2 requires "initialize logging before handing off", but the tracing stack lands in Task 3. Task 2's plan will therefore include a minimal, replaceable logging-init seam in the application layer with a safe default filter; Task 3's plan will harden it (`RUST_LOG`, invalid-filter fallback, init-once guard, explicit error boundaries). No reordering, no scope expansion.
- Verification tool: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` (runs `fmt --check`, `check --locked`, `test --locked`, `clippy --locked -- -D warnings`; exit 0 = all pass).

## Steps 2–6 Overview

| Step | Work | Sub-agent |
|---|---|---|
| 2 | Git feature branch setup + commit pending TODO changes | implementer |
| 3 | Version bump 0.1.0 → 0.2.0 (+ Cargo.lock refresh) | implementer |
| Task 1 | 4.1b plan → 4.2 implement → 4.3 review+simplify (+fix) → 4.4 docs → 4.5b adherence → 4.6 completion | architector / implementer / code-reviewer + code-simplifier / docs-specialist / architector / implementer |
| Task 2 | same 4.1–4.6 cycle | same assignment pattern |
| Task 3 | same 4.1–4.6 cycle | same assignment pattern |
| Task 4 | same 4.1–4.6 cycle | same assignment pattern |
| 5 | TODO file completion (`-DONE` rename, tmp cleanup, merge to `main`, push `origin` only) | implementer |
| 6 | Finish: summary + handoff text | planner |

Before starting a new task: commit pending changes (4.0 Overall Process Management).

---

## Task 1 — Implement the Configuration Module

### Pre-analysis (technical & architecture decisions for 4.1b to encode)

- New files under `src/config/`: `mod.rs` (public API surface), `model.rs` (typed structs), `loader.rs` (read + deserialize), `validation.rs` (validation rules), `error.rs` (typed errors). Adjusted only if line limits (≤ 200/file) demand.
- Strongly typed structs mirroring brief §5.2: `youtube.broadcast` (title, description, privacy_status), `video` (width, height, fps, pixel_format), `renderer` (font path, font_size, line_height, four margins, text_color, background_color), `chat` (log_file path), `ffmpeg` (executable, video_codec, preset, bitrate).
- Dependencies: `serde` (derive) + `serde_json` are required by the TODO. Additional candidates the architector must justify or reject: `thiserror` (typed error reporting) and `tempfile` (test temp dirs). Standard library (`std::env::temp_dir` + unique suffix) is an acceptable alternative to `tempfile`; the plan must pick one.
- Loading API takes an explicit path (`load_from_path(&Path) -> Result<Config, ConfigError>`); no CWD assumptions; default path constant `config/config.json` lives in the app layer (Task 2).
- Validation (all rules from the TODO): positive width/height/fps; pixel format must be exactly `rgb24` (unsupported rejected, no silent substitution); positive font size and line height; margins ≥ 0 and vertical layout must allow ≥ 1 line (`height - top - bottom >= line_height`); required strings/paths non-empty and not whitespace-only; privacy ∈ {private, public, unlisted}; FFmpeg codec/preset/bitrate present and non-empty. No existence checks for font/FFmpeg files (belongs to consumers later).
- Errors: typed, actionable (file/field identified, invalid value explained); no panics; explicit error type (e.g. `ConfigError`) with variants for IO, JSON syntax/deser, validation.
- No `max_messages` field. Visible-line capacity derivation stays a pure function when introduced (renderer phase); config only guarantees it is ≥ 1.
- `config/config.example.json`: safe, non-secret values per brief §5.2, default privacy `unlisted`. `config/config.json` NOT created (already gitignored).
- Unit tests (in-module `#[cfg(test)]` or `tests/` per plan decision): valid config; missing/unreadable file; malformed JSON; missing required fields; invalid dimensions/FPS; unsupported pixel format; invalid margins/line geometry; invalid privacy; empty required values. Tests use temp files/dirs only; no developer's real config.
- Config module must not depend on youtube/renderer/ffmpeg/chat modules (modular-monolith boundary).

### 4.1–4.6 cycle

- 4.1b architector → plan `.kilo/plans/20261009-task1-config-module.md`
- 4.2 implementer → implement per plan; meaningful commits
- 4.3 code-reviewer + code-simplifier (concurrent) → fix plans if required; planner routes fixes to implementer; max 3 cycles
- 4.4 docs-specialist → module docs where they belong
- 4.5b architector → plan adherence report
- 4.6 implementer → add `[DONE]` to `### 1.` heading, `[x]` sub-items only where the workflow marks them; commit

## Task 2 — Establish the Application and CLI Skeleton

### Pre-analysis (technical & architecture decisions for 4.1b to encode)

- New files under `src/app/` (e.g. `mod.rs`, `cli.rs`, `commands.rs`/`application.rs` as needed; respect line limits); rewrite `src/main.rs` to delegate: parse → load config → init logging → dispatch mode handler → exit code.
- CLI: two commands `auth` and `run`; optional `--config <path>` (accepted before or after the command name — plan must pick one and encode it). CLI-parser choice: maintained `clap` OR small std parser; the plan must pick ONE and justify; interface stays minimal (no extra flags/commands).
- Default config path constant `config/config.json` (defined and unit-tested); `config.example.json` never loaded implicitly.
- Error behavior: invalid args, unknown command, missing/invalid config → concise actionable stderr message + non-zero exit; no panics.
- Mode handlers: `auth` → after config load+validation and logging init, report authentication not implemented yet, exit non-zero; `run` → after validation, report runtime pipeline not implemented yet, exit non-zero. Neither touches YouTube or FFmpeg. Messages must be accurate, not pretending success.
- Logging seam: Task 2 provides a minimal init call the mode handlers use; Task 3 replaces/hardens it (see Task 3 pre-analysis). Keep the seam one call, no parallel implementations later.
- Testability: command dispatch and config loading unit-testable without external processes or secrets (pure functions returning parsed CLI/exit decisions where possible).

### 4.1–4.6 cycle

- 4.1b architector → plan `.kilo/plans/20261009-task2-app-cli-skeleton.md`
- 4.2 implementer → implement per plan; meaningful commits
- 4.3 code-reviewer + code-simplifier → fixes via implementer; max 3 cycles
- 4.4 docs-specialist
- 4.5b architector → adherence report
- 4.6 implementer → `[DONE]` on `### 2.` heading; commit

## Task 3 — Establish Structured Logging and Error Boundaries

### Pre-analysis (technical & architecture decisions for 4.1b to encode)

- Logging stack: `tracing` + `tracing-subscriber` (with `EnvFilter`) — TODO explicitly names it as the candidate; plan must confirm suitability and pin versions via lockfile.
- Init once at startup, before command dispatch; `RUST_LOG` env var honored; unset/invalid filter → safe documented default (plan must name it, e.g. `info`). Init must not panic on invalid filter.
- Log selected command and startup/validation outcomes (info level); NEVER log secrets, tokens, stream keys, or full config file contents.
- Error boundaries: single fatal-error path — configuration/app setup errors propagate explicitly (typed errors, `?`) to the process entry point, which converts them to one logged diagnostic + non-zero exit; no duplicate logging of the same error across layers.
- No Tokio (no concrete async requirement this phase); no task supervision, no signal handling, no YouTube lifecycle states.
- Unit tests: logging init is side-effectful — plan must scope tests to what is testable without subscriber side effects (e.g., filter resolution from `RUST_LOG` values as pure function), keeping the rest validated via clippy/dev-checks and documented.

### 4.1–4.6 cycle

- 4.1b architector → plan `.kilo/plans/20261009-task3-structured-logging.md`
- 4.2 implementer → implement per plan; meaningful commits
- 4.3 code-reviewer + code-simplifier → fixes via implementer; max 3 cycles
- 4.4 docs-specialist
- 4.5b architector → adherence report
- 4.6 implementer → `[DONE]` on `### 3.` heading; commit

## Task 4 — Update Documentation and Project Metadata

### Pre-analysis (technical & architecture decisions for 4.1b to encode)

- `README.md`: add TOC; describe the new application foundation, `auth`/`run` commands, and that both modes are placeholders until later phases; do NOT overstate functionality; keep existing Docker/Alpine VM dev-check instructions accurate (update only if wording drifted); no full README rewrite.
- New docs file (e.g. `docs/configuration.md`): how to copy/edit `config/config.example.json` → gitignored `config/config.json`; the `--config` option; link from README. Doc file > 100 lines → include TOC.
- State explicitly: FFmpeg is a separately provided external prerequisite, not installed by this phase; this phase does not launch it.
- `.agent/project-structure.md`: reflect every new source module and `config/config.example.json`.
- `.agent/project-info/context.md`: implemented behavior, dependency choices + rationale, tests and actual results, remaining limitations, next phase.
- Do NOT create the deferred FFmpeg runtime-prerequisites guide in this phase.
- Final verification before completion: full dev-checks run via the standard invocation; all four checks must pass.

### 4.1–4.6 cycle

- 4.1b architector → plan `.kilo/plans/20261009-task4-docs-metadata.md`
- 4.2 implementer → implement per plan; meaningful commits
- 4.3 code-reviewer + code-simplifier (docs/markdown review; fixes via implementer)
- 4.4 docs-specialist → final polish of the docs deliverables
- 4.5b architector → adherence report
- 4.6 implementer → `[DONE]` on `### 4.` heading; commit

---

## Step 5 — TODO File Completion (implementer)

- Rename `.agent/todos/20261009/20261009-todo-3.md` → `20261009-todo-3-DONE.md` (content preserved; no edits beyond the `[DONE]` marks added during 4.6).
- Review and remove any tmp files/folders created during the process (none expected; container writes go to the named volume and `logs/checks/`).
- Ensure all files are committed in the feature branch; verify `git status` clean; verify no gitignored files staged.
- Merge: checkout `main` → merge `feat/phase01-config-app-skeleton` → on success delete the feature branch (verify first); on failure notify user.
- Push: `origin` remote exists → push `main` to `origin` ONLY. Notify user if push fails.

## Step 6 — Finish (planner)

- Short summary of realized work.
- Provide handoff text for the next TODO file in a new chat.

## Error Handling

- On sub-agent failure/empty response: resume the same task with the sub-agent, or split into smaller steps; escalate to the user on repeated failure.
- On check failures (dev-checks exit ≠ 0): stop, log details, commit safe changes if possible, notify user, pause for intervention.
