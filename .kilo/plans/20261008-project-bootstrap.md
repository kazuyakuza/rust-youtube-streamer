# Global Plan — Project Bootstrap (Rust YouTube Streamer Service)

- Date: 2026-10-08
- TODO file: `.agent/todos/20261008/20261008-todo-1.md`
- Request source: chat (direct user request)
- Workflow: `.kilo/commands/critical-workflow.md`

## TODO Tasks (Line Items format)

1. Initialize project info (`.agent/project-info` core files)
2. Update README.md with initial info about the project
3. Set up project initial structure folders (folders only, .gitkeep placeholders)

## Global Pre-Analysis

### Current repository state (verified)

- Git branch: `main` (upstream `origin/main` gone). Unstaged changes: `brief.md` (full project brief written), `.gitignore` (+1 line), `.opencode/*` (small updates), `TBD` deleted. These will be committed in Step 2.
- `.agent/project-info/`: contains `brief.md` (complete Rust YouTube Streamer brief), `instructions.md`, and the `.initialized` template marker. Missing core files: `product.md`, `context.md`, `architecture.md`, `tech.md`.
- `README.md`: still the generic "Base Project for AI Agent Driven Development" template text.
- `src/`: empty (only `.gitkeep`). `docs/`: template docs only (`how-to-set-up-git.md`, `how-to-write-todo-files.md`).
- `.gitignore`: generic template; notably ignores `logs/` (whole dir) and `.secrets/`, `.git-credentials`.
- No `Cargo.toml`, no `package.json` → no version file exists yet.

### Technical & architecture decisions (encoded here; implementer/junior must not re-decide)

- **Task 1 (project-info)**: Create the 4 missing core files (`product.md`, `context.md`, `architecture.md`, `tech.md`) under `.agent/project-info/` derived strictly from `brief.md` (source of truth) and repo facts. Do NOT modify `brief.md` or `instructions.md`. Remove `.agent/project-info/.initialized` marker file (per `.agent/project-info/instructions.md` initialization flow). `context.md` documents: brief defined, bootstrap in progress, next steps.
- **Task 2 (README)**: Replace README.md content with initial project info for the **Rust YouTube Streamer Service**: project purpose (one paragraph), MVP summary (black screen + chat text over FFmpeg→YouTube Live), key technologies (Rust, Tokio, FFmpeg external process, YouTube Data API/OAuth), status ("initial setup — detailed README coming later"), pointer to `AGENTS.md` for AI agents, and the Critical Workflow chat snippet usage. Keep it short (~60–90 lines). Do not invent features beyond the brief.
- **Task 3 (structure)**: Create empty folders with `.gitkeep` only, exactly per brief §4 (minus files explicitly excluded by the user's "no files" constraint):
  - Root: `config/`, `credentials/`, `fonts/`, `logs/` (each with `.gitkeep`; `docs/` and `src/` already exist)
  - `src/` subfolders: `app/`, `config/`, `youtube/`, `chat/`, `renderer/`, `streaming/` (each with `.gitkeep`)
  - Remove `src/.gitkeep` (redundant once subfolders exist — placeholder cleanup, in scope of this task)
  - `.gitignore` adjustment: add `!logs/.gitkeep` negation so `logs/.gitkeep` is tracked despite the `logs/` ignore rule (brief §4 intends logs/.gitkeep to be versioned; chat logs stay ignored via `*.log`).
  - Update `.agent/project-structure.md` per the Project Structure Maintenance Workflow (`# Folders in src/` + `# Other folders` sections).
- **Front-end related**: NO for all tasks → skip 4.1a / 4.5a everywhere.
- **Step 3 (Version Update)**: no version file exists (`Cargo.toml` comes later) → documented no-op.
- **Testing**: no build/test suite exists yet → 4.5b verification is plan-adherence only.

## Steps (each = separate `task` tool invocation)

- Step 2: Git Feature Branch Setup (commit unstaged on `main`, create `feat/project-bootstrap`) => implementer
- Step 3: Version Update (no-op, note in commit-less summary) => implementer
- Task 1: 4.1b Analysis & Planning => architector
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Code Review & Simplification => code-reviewer + code-simplifier
- Task 1: 4.4 Documentation => docs-specialist
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion ([DONE] mark + commit) => implementer
- Task 2: 4.1b => architector
- Task 2: 4.2 => implementer
- Task 2: 4.3 => code-reviewer + code-simplifier
- Task 2: 4.4 => docs-specialist
- Task 2: 4.5b => architector
- Task 2: 4.6 => implementer
- Task 3: 4.1b => architector
- Task 3: 4.2 => implementer
- Task 3: 4.3 => code-reviewer + code-simplifier
- Task 3: 4.4 => docs-specialist
- Task 3: 4.5b => architector
- Task 3: 4.6 => implementer
- Step 5: TODO File Completion (rename `-DONE`, cleanup, merge to `main`, push `origin`) => implementer

## Per-task plan files (to be produced in 4.1b)

- Task 1: `.kilo/plans/20261008-project-bootstrap-task1.md`
- Task 2: `.kilo/plans/20261008-project-bootstrap-task2.md`
- Task 3: `.kilo/plans/20261008-project-bootstrap-task3.md`

## Notes

- The global plan must never be overwritten.
- Git push restricted to Step 5; branch creation restricted to Step 2.
