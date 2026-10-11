# Global Plan — Phase 03: FFmpeg Process and Local Video Pipeline

- **TODO source**: `.agent/todos/20261011/20261011-todo-1.md` (Pattern C — 5 tasks, `###` headings under `## Tasks`)
- **Date**: 2026-10-11
- **Branch (step 2)**: `feat/phase03-ffmpeg-local-pipeline` (from `main`)
- **Version (step 3)**: `0.5.0` → `0.6.0` (minor bump; feature phase)
- **Front-end tasks**: **none** (Rust backend service; steps 4.1a / 4.5a skipped for every task)

## Baseline (verified from live repository)

- `main` is the current branch state after Phase 02 (v0.5.0, 81 tests green).
- `src/app/cli.rs` parser: exactly two commands (`auth`, `run`), `--config` before/after command at most once, `--help` only before command. Exit codes: 1 startup failure, 2 usage, 3 not-implemented placeholder.
- `src/app/modes.rs`: `auth`/`run` placeholders exit 3 after config validation.
- `src/chat/`: `ChatStore` (bounded FIFO, append-first log, `ChatStoreSettings::from_config`, `visible_text`, `accept`, `is_empty`), `ChatMessage { author, text, message_id, timestamp }`, `ChatError { Capacity, LogOpen, LogAppend }`.
- `src/renderer/`: `Renderer` trait (`render(&mut self, &RenderInput) -> Result<Frame, RenderError>`), `TextRenderer::new`, `Frame` RGB24 checked buffer, `RendererSettings::from_config`, `RenderError` typed variants.
- `src/config/`: full `AppConfig` schema (`video.{width,height,fps,pixel_format}`, `renderer.{font,font_size,line_height,margins,text_color,background_color}`, `chat.log_file`, `ffmpeg.{executable,video_codec,preset,bitrate}`), 18 validation checks, `deny_unknown_fields`.
- `src/streaming/`: placeholder folder entry only — no implementation yet.
- `.gitignore`: no `videos/` rule yet (must be added; keep scope narrow).
- Docs to update: README (Current Status, Commands), `docs/configuration.md`, `.agent/project-structure.md`, `.agent/project-info/context.md`, new completion report `.agent/reports/`.

## Canonical environment

- All build/check/gui-free verification runs inside the Compose `rust` service via the Alpine VM MCP, standard pattern: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cargo-command-or-script>`.
- Standing rules: Rust 1.82 / edition 2021; `--locked` builds; artifacts in named volume (never repo-root `target/`); no `google-youtube3`; justify any new dependency; no root-generated files (code output under gitignored `videos/` and `dist/` only).
- FFmpeg is an external prerequisite and is **not** in the pinned `rust:1.82` image: fake-process tests are mandatory; a real-FFmpeg smoke test is optional and requires either making the test image carry FFmpeg (toolchain-image change, must be a documented decision in the per-task plan) or recording an explicit NOT-RUN limitation. No invented evidence.

## Global technical & architectural decisions (encoded; per-task plans may refine details, never reverse them)

- **D1 — New `src/streaming` module** with clear sub-files (respect 200-line / 50-line rules): FFmpeg argument construction (pure, testable), process supervisor (spawn/drain/exit), frame writer (exact full-frame writes with short-write and broken-pipe handling), and output-path/filename logic beside the executable. `streaming` never depends on YouTube API types or chat semantics; it receives opaque frame bytes only.
- **D2 — No shell**: arguments are produced as a vector of OS-safe strings and passed to a direct spawn (`std::process::Command`); stdin piped; stdout and stderr drained concurrently so pipes cannot deadlock; stderr diagnostics captured and surfaced in event of failure without logging secrets (there are none this phase).
- **D3 — MP4 output settings for local files**: encoder settings from config (`video_codec`, `preset`, `bitrate`), input `-f rawvideo -pix_fmt rgb24 -s <width>x<height> -r <fps> -i pipe:0`; output pixel format and container flags (e.g., `yuv420p`, `+faststart`) chosen by the Task-1 architector plan with verified authority (FFmpeg official documentation), documented in that plan — not invented here.
- **D4 — Output directory**: resolved **beside the executable** via `std::env::current_exe()` parent → `videos/`, created if missing; typed error if creation/write fails.
- **D5 — Collision-safe filenames**: pure/testable timestamped generator `demo-YYYYMMDD-HHMMSS.mp4` / `session-YYYYMMDD-HHMMSS.mp4` with a deterministic collision suffix tested by unit tests; never overwrites an existing file.
- **D6 — Frame pipeline (Task 2) is synchronous first**: pacing loop at the configured FPS using a minimal, testable timing seam; per frame `ChatStore::visible_text()` → `RenderInput` → `TextRenderer::render` → exact `width*height*3` byte write to FFmpeg stdin. Threads (std) preferred over introducing Tokio now — unless the Task-2 architector plan demonstrates Tokio is required; the decision must be explicit and justified in the plan.
- **D7 — Interrupt (Ctrl+C)**: minimal cross-platform approach via a maintained, justified crate (e.g., `ctrlc`/`signal-hook`) so shutdown: stop frame production → close stdin → wait FFmpeg → report. Sub-task plans must state the verified scope of Ctrl+C on both platforms. If reliable on only one platform, the other is documented as a limitation (per TODO), never silently claimed.
- **D8 — `run` stays a placeholder (TO-DO Task 4)**: it must FAIL CLEARLY before any streaming (exit 3 contract retained, message updated to point at the not-yet-built YouTube/OAuth path). `run` does **not** fall back to local demo behavior; `auth` untouched.
- **D9 — `demo` (Task 3)**: static documented message set with deterministic schedule; messages flow through `ChatStore::accept` (log + FIFO) and the same render path as future real chat; schedule is isolated for unit testing via an injectable seam; no interactive input; final path printed only after FFmpeg exits successfully and the MP4 is finalized.
- **D10 — Config contract**: no new schema fields expected; if the per-task plan discovers a necessary field, it must add + document it with example config updated and validation preserved (and the plan approved before implementation).
- **D11 — `.gitignore`**: add a narrow `videos/` rule only.
- **D12 — Dependency policy**: any new crate (D7, possible timing/clock helpers) enters via `Cargo.toml` + refreshed `Cargo.lock` (commit `--locked` verified) with rationale in the changing task plan; standard library preferred when sufficient.

## Execution order (each image = separate `task` invocations with the listed sub-agent; all steps share the standing Sub-Task Prompt Requirements, TODO path, plan paths and constraints)

1. Step 2 — Git Feature Branch Setup ⇒ implementer
   - `git status` cleanup on current branch; ensure `main`; create/switch `feat/phase03-ffmpeg-local-pipeline`.
2. Step 3 — Version Update ⇒ implementer
   - `Cargo.toml` 0.5.0 → 0.6.0; `Cargo.lock` refreshed via container `cargo check`, verified `--locked`; commit `chore: bump version to 0.6.0`.
3. TODO Task 1 — FFmpeg Process Wrapper and Local MP4 Output
   - 4.1b architector → 4.2 implementer → 4.3 code-reviewer + code-simplifier (fixes ⇒ implementer) → 4.4 docs-specialist → 4.5b architector adherence → 4.6 implementer (`[DONE]` + commit).
   - Focus: pure argument builder; process supervisor with drain threads; frame-writer trait/seam; output dir beside exe; collision-safe names; typed process errors; `.gitignore` `videos/`.
4. TODO Task 2 — Continuous Local Frame Pipeline (depends on Task 1)
   - 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6.
   - Focus: pacing seam; ChatStore→RenderInput→renderer→stdin; EOF/abort sequencing; Ctrl+C mechanism (D7).
5. TODO Task 3 — Dedicated `demo` Command
   - 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6.
   - Focus: CLI extension (`demo` command + usage/help without breaking ordering/exit contracts); demo schedule data + injectable seam; demo messages via ChatStore; automatic finish + finalization; path printed only after success.
6. TODO Task 4 — Production `run` Mode Boundary
   - 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6.
   - Scope-small: keep exit 3, clarify `run` messages/structure so it cannot stream to YouTube and cannot be mistaken for the local demo; `auth` untouched.
7. TODO Task 5 — Tests, Documentation and Project Metadata (final gate task)
   - 4.1b → 4.2 → 4.3 → 4.4 (main documentation load) → 4.5b → 4.6.
   - Focus: gap-filling tests (CLI/help, schedule, arguments, paths/collision, error mapping, fake-process smoke, optional real-FFmpeg probe); full `sh scripts/dev-checks.sh` gate; both release builds `build-linux.sh` + `build-windows.sh`; README/docs/configuration updates; new `demo` docs; `.agent/project-structure.md` refresh; completion report in `.agent/reports/`.
8. Step 5 — TODO File Completion ⇒ implementer
   - Rename TODO with `-DONE` suffix; remove tmp artifacts; verify all committed; merge to `main`; delete branch after verified merge; push `origin` only.
9. Step 6 — Finish ⇒ Planner
   - `context.md` update; summary + next-TODO chat opener.

## Per-task content pointers (from the TODO file; sub-agents must treat the TODO text itself as authoritative)

- Task 1 (`### 1. FFmpeg Process Wrapper and Local MP4 Output`): all bullets on arguments/stdin/stderr/output-dir/naming/typed errors/interface.
- Task 2 (`### 2. Continuous Local Frame Pipeline`): pacing, empty-frame behavior, exact byte writes, short-write/broken-pipe, finalization sequence, Ctrl+C minimal handling with documented limitations.
- Task 3 (`### 3. Dedicated demo Command`): CLI extension, deterministic schedule, testability seam, FIFO eviction during demo, finish/finalize, no interactive controls.
- Task 4 (`### 4. Production run Mode Boundary`): fail-clearly before streaming, no interactive input, no silent substitution by demo, `auth` untouched.
- Task 5 (`### 5. Tests, Documentation and Project Metadata`): tests listed in TODO (incl. real/fake FFmpeg separation), dev-check gate, both release builds, docs/structure/context/report updates, no unbacked native-Windows claims.

## Risks / watch items

- FFmpeg argument ordering and local-MP4 flags must be verified against official FFmpeg docs in Task-1 planning (authoritative source recorded), not from memory.
- Pipe deadlock: stdout/stderr must never block the writer loop (dedicated drain threads; supervised exit code).
- Clock/file-codec edge cases: invalid config (e.g., font path) must surface typed startup errors through the existing funnel; renderer font is loaded at `TextRenderer::new` — demo failure semantics must match the app error contract (exit codes).
- Container lacks a real FFmpeg binary; scope of any real-FFmpeg smoke test is a per-task-plan decision requiring the factor documented in "Canonical environment" above.
- `#[allow(dead_code)]` on `chat`/`renderer` in `src/main.rs` retires when wiring lands in Task 2 — verify clippy `-D warnings` stays green.
- Newline integrity: source files keep real newline characters (repo-host CRLF incident rule is known; `.gitattributes` already forces LF for scripts).
