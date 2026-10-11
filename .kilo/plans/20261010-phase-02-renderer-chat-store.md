# Global Plan — Phase 02: Renderer and Chat Store

- Date: 2026-10-10
- TODO source: `.agent/todos/20261010/20261010-todo-2.md` (Pattern C: `## Tasks` section, 4 `###` tasks → 4 tasks)
- Workflow: `.kilo/commands/critical-workflow.md` (each task gets its own 4.1–4.6 cycle)
- TODO is the TOP PRIORITY source; this plan never overrides it.

## Task Origin and Baseline

- Effective plan: `.agent/todos/20261010/20261010-todo-2.md`. Mandatory reading for every sub-agent.
- Repository baseline: Phases 00, 00.1, 01, 01.1, 01.2 complete; package v0.4.0; `git` clean on `main`, up to date with `origin/main`.
- `src/chat/` and `src/renderer/` contain only `.gitkeep` — both components are greenfield.
- Config already exists and validates: geometry `video.width/height/fps`, `pixel_format == "rgb24"`, `renderer.font/font_size/line_height/{left,top,right,bottom}_margin/text_color/background_color`, `chat.log_file`, vertical-layout check guarantees `top_margin + bottom_margin + line_height <= height` (≥1 line). `deny_unknown_fields` is active; `max_messages` must NOT be added.
- Version bump is restricted to Step 2/3 implementer tasks; git push is restricted to Step 5.

## Global Pre-Analysis (technical & architecture decisions to be encoded per task)

1. ChatStore capacity derives overflow-safe: `floor((height − top_margin − bottom_margin) / line_height)` in u64 arithmetic; capacity ≥ 1 already guaranteed by config validation, but the store repeats the derivation defensively (no `max_messages` field).
2. Chat model stays application-level: display identity `username: message`; optional message ID + timestamp carried internally, never in visible text; log lines timestamped per brief §11.2 record format.
3. Log/queue I/O-failure semantics: log write failure must not silently lose the record nor produce ambiguous state — the architector decides acceptance semantics (e.g., message rejected → not enqueued → acceptance returns a typed error) and encodes it in the plan with tests.
4. Font rasterization crate: must be maintained, pure Rust, cross-platform (no system graphics libs), MSRV ≤ 1.82. Candidates: `ab_glyph` / `fontdue` / `rusttype` (maintenance to be checked). Architector selects and justifies; new deps are pinned/locked; no silent font fallback (missing/invalid font → typed, testable error).
5. Colors: config contract (`renderer.text_color`/`background_color`) must not be silently dropped. Brief §9.4 says white-on-black; example config is `#FFFFFF`/`#000000`. Decision (encoded by architector): parse hex values and use configured colors; defaults in the example already match the brief contract; deviation documented if colors are restricted to black/white. Schema changes only if genuinely required (all-or-nothing: model + tests + example JSON + docs).
6. Renderer independence: no YouTube/chat-store history ownership, no FFmpeg/process code. Frame = exactly `width × height × 3` bytes, RGB24, checked arithmetic, actionable typed errors, no panics; clip (not wrap) at frame bounds; margins and line height respected.
7. Sub-agent tools: unit tests run in the Docker/Compose toolchain via Alpine VM MCP commands (`docker compose -f /rust-youtube-streamer/docker-compose.yml ...`); no global installs; artifacts never staged (Gitignore Compliance).

## Front-end Determination

- Task 1 (chat model/store): not front-end related → 4.1b + 4.5b only.
- Task 2 (RGB24 text renderer): not front-end related → 4.1b + 4.5b only.
- Task 3 (documentation/metadata): not front-end related → 4.1b + 4.5b only.
- Task 4 (tests/builds/completion report): not front-end related → 4.1b + 4.5b only.
- Sub-steps 4.1a (frontend spec) and 4.5a (frontend verification) are OMITTED for all tasks.

## Execution Order (each bullet = separate `task` invocation)

1. Step 2: Git Feature Branch Setup → implementer
   - `git status` clean-check; on `main` already → create+switch `feat/phase02-renderer-chat-store` (default feature naming; fix branch not applicable).
2. Step 3: Version Update → implementer
   - Cargo.toml `0.4.0 → 0.5.0` (minor: new runtime components); commit `chore: bump version to 0.5.0`; refresh/verify Cargo.lock (`--locked` green).
3. Task 1 — Chat Message Model and ChatStore
   - 4.1b Analysis & Planning → architector (plan → `.kilo/plans/20261010-p02-task1-chat-model-store.md`; present to user only if user chose "Approve Global Plan")
   - 4.2 Implementation → implementer
   - 4.3 Code Review → code-reviewer (concurrent) + Code Simplification → code-simplifier (concurrent); fix plans → implementer (max 3 cycles)
   - 4.4 Documentation → docs-specialist
   - 4.5b Overall Plan Adherence → architector
   - 4.6 Task Completion → implementer (add `[DONE]` to `### 1.` header; commit)
4. Task 2 — RGB24 Text Renderer
   - 4.1b → architector (`.kilo/plans/20261010-p02-task2-text-renderer.md`)
   - 4.2 → implementer; 4.3 → code-reviewer + code-simplifier, fixes → implementer; 4.4 → docs-specialist; 4.5b → architector; 4.6 → implementer (`### 2.` `[DONE]`)
5. Task 3 — Documentation and Project Metadata
   - 4.1b → architector (`.kilo/plans/20261010-p02-task3-docs-metadata.md`)
   - 4.2 → implementer; 4.3 → code-reviewer + code-simplifier → fixes; 4.4 → docs-specialist; 4.5b → architector; 4.6 → implementer (`### 3.` `[DONE]`)
6. Task 4 — Tests, Builds and Completion Report
   - 4.1b → architector (`.kilo/plans/20261010-p02-task4-tests-builds-report.md`)
   - 4.2 → implementer; 4.3 → code-reviewer + code-simplifier → fixes; 4.4 → docs-specialist; 4.5b → architector; 4.6 → implementer (`### 4.` `[DONE]`)
   - Includes final dev-checks gate + both release builds (Alpine VM MCP) and the tracked report `.agent/reports/20261010-phase-02-renderer-chat-store.md`.
   - Note: acceptance-criteria checkboxes may be flipped to `[x]` only for verified items (mirrors Phase 01/01.1/01.2 precedent; TODO must not be renamed yet).
7. Step 5: TODO File Completion → implementer
   - Rename TODO → `20261010-todo-2-DONE.md`; remove tmp files; verify all commits on feature branch; checkout `main`, merge feature branch; delete branch on success; push to `origin` ONLY.

## Error Handling

- Sub-agent failures: resume/retry with smaller scope; max 3 review cycles; pause and escalate to the user on repeated failure or ambiguity. Planner presents scope questions to the user when TODO/architector leave no single encoded choice.
