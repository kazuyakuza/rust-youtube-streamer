# Global Plan — Phase 01.1: Reproducible Linux Build Artifact

- Source TODO: `.agent/todos/20261009/20261009-todo-4.md` (Pattern C: 4 tasks under `## Tasks`)
- Date: 2026-10-09
- Workflow: `.kilo/commands/critical-workflow.md`
- Front-end related: **NO** — none of the 4 tasks is front-end related. Sub-steps 4.1a (Front-end Technical Specification) and 4.5a (Front-end Implementation Verification) are **skipped for all tasks**; only 4.1b and 4.5b run.
- Branch name (step 2): `feat/linux-release-build-artifact`
- Version (step 3): `Cargo.toml` `0.2.0` → `0.3.0` (minor — new build-artifact workflow feature; matches Phase 01's pattern of one minor bump per shipped phase).

## Global Pre-Analysis (repository state)

Confirmed by reading the live repository:

- `main` is HEAD (`17c2177`), working tree clean, `origin` remote present and tracked (`origin/main`).
- Package: `rust-youtube-streamer-service` v0.2.0, edition 2021 (root `Cargo.toml`); no `[[bin]]` override → default binary name `rust-youtube-streamer-service`.
- Toolchain: pinned `rust:1.82` image (rustfmt+clippy baked in), Compose service `rust`, `working_dir: /rust-youtube-streamer`, bind mount `/rust-youtube-streamer:/rust-youtube-streamer`, named volume `rust-streamer-target:/rust-streamer-target`, env `CARGO_TARGET_DIR=/rust-streamer-target` → Cargo intermediates never land in the project root; `Cargo.lock` is the only root-level generated artifact and is tracked.
- Compose file lives at project root; on the Alpine VM the project is shared at `/rust-youtube-streamer`. Standard invocation pattern: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>` (Compose v2.31.0 observed).
- Established convention, referenced as the quality gate: `scripts/dev-checks.sh` (fmt --check / check --locked / test --locked / clippy -D warnings; aggregate exit 0/1; UTC log under gitignored `logs/checks/`).
- `.gitignore` already contains `dist/` (generic build-artifacts section, line 34) → the TODO's approved root-level exception directory is already ignored; expect no rule change, only verification (`git check-ignore`) and no staging of generated binaries.
- Shell-script conventions from `scripts/dev-checks.sh`: POSIX `#!/bin/sh`, header comment states runtime contract, clear `printf` messages, explicit exit codes.
- All Cargo commands must run via the Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`; allowlisted `docker`/`sh` prefixes cover the required invocations). Host has no Rust/Cargo; do not assume otherwise.
- MCP caveat (documented in README): cargo stderr may be partly missing in captured MCP output; **exit status is authoritative**.
- `[Project Info: Active]` confirmed: project-info files read; `context.md` states Phase 01.1 is the next phase and no implementation has started.

## Technical & Architecture Decisions (global)

1. **Artifact semantics**: the script produces a **Linux release executable** built inside the pinned Linux Rust container. This is a build-artifact workflow only; it is not native Windows/Linux runtime validation, does not create a `.exe`, and does not prove application runtime behavior (app `auth`/`run` are still placeholders).
2. **Path resolution in the script**: the binary lives at `$CARGO_TARGET_DIR/release/rust-youtube-streamer-service` (`CARGO_TARGET_DIR=/rust-streamer-target` from Compose). The script resolves the repo root from the documented Compose working directory `/rust-youtube-streamer` (validated, not assumed: refuse to run if cwd ≠ `/rust-youtube-streamer`) and **requires** `CARGO_TARGET_DIR` to be set non-empty — if unset, fail non-zero with a clear message rather than defaulting to `./target` (which would generate `target/` in the project root, violating the standing rule).
3. **`dist/` exception**: only `dist/rust-youtube-streamer-service` may be written under the root; `dist/` is gitignored, the artifact is never staged/committed. Intermediate Cargo output stays in the named volume. The script never creates a default `CARGO_TARGET_DIR` fallback and never cleans/prunes Docker resources or unrelated `dist/` files.
4. **Failure semantics**: script exits non-zero on (a) wrong cwd, (b) missing/empty `CARGO_TARGET_DIR`, (c) build failure, (d) missing/empty source binary, (e) failed copy, (f) missing/non-empty verification of the final artifact. It never reports success before the final file exists and is non-empty; Cargo diagnostics are not suppressed.
5. **Verification strategy**: acceptance requires the actual build to succeed via Docker/MCP on the real tree, plus `scripts/dev-checks.sh` passing on the **final tree** (after all docs/metadata changes) — dev-checks on the final tree is executed in Task 4's cycle so the durable report contains real evidence.
6. **Docs-first-facts**: `docs/build.md`, README link, `project-structure.md`, and the tracked completion report `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` must contain only observed results (exact commands, exit statuses, artifact path/size). No fabrication; not-run checks are recorded as such.
7. **Project-info upkeep**: `context.md` is updated at the end of the workflow with real results and the next phase (Phase 02 — Renderer and Chat Store), consistent with the TODO Task 3 requirement and the instructions.md "Critical Closing Step".
8. **No version/API risk**: no application source changes; `Cargo.toml`/`Cargo.lock` change only for the version bump; lockfile refresh via container `cargo check` then re-verified with `--locked`.

## Execution Mapping (steps 2–6, one 4.1–4.6 cycle per task)

| # | Step | Sub-agent |
| --- | --- | --- |
| 1 | Step 2: Git Feature Branch Setup (`git status`; confirm clean; branch `feat/linux-release-build-artifact` from `main`) | implementer |
| 2 | Step 3: Version Update (`Cargo.toml` 0.2.0→0.3.0; container `cargo check` to refresh `Cargo.lock`; verify `cargo check --locked`; commit `chore: bump version to 0.3.0`) | implementer |
| 3 | Task 1: 4.1b Analysis & Planning → per-task plan `.kilo/plans/20261009-phase-01-1-task1-build-script.md` | architector |
| 4 | Task 1: 4.2 Implementation (`scripts/build-linux.sh`; real container build + copy to `dist/`; commit) | implementer |
| 5 | Task 1: 4.3 Code Review & Simplification (concurrent) + fixes if required | code-reviewer & code-simplifier → implementer |
| 6 | Task 1: 4.4 Documentation (header/script comments only; README not yet — docs land in Task 3) | docs-specialist |
| 7 | Task 1: 4.5b Overall Plan Adherence | architector |
| 8 | Task 1: 4.6 Task Completion (`[DONE]` on `### 1.`; commit) | implementer |
| 9 | Task 2: 4.1b Analysis & Planning → `.kilo/plans/20261009-phase-01-1-task2-ignore-semantics.md` | architector |
| 10 | Task 2: 4.2 Implementation (`dist/` ignore verification via `git check-ignore`; no staging; semantics documented where required by this task; commit) | implementer |
| 11 | Task 2: 4.3 Code Review & Simplification (concurrent) + fixes if required | code-reviewer & code-simplifier → implementer |
| 12 | Task 2: 4.4 Documentation | docs-specialist |
| 13 | Task 2: 4.5b Overall Plan Adherence | architector |
| 14 | Task 2: 4.6 Task Completion (`[DONE]` on `### 2.`; commit) | implementer |
| 15 | Task 3: 4.1b Analysis & Planning → `.kilo/plans/20261009-phase-01-1-task3-build-docs.md` | architector |
| 16 | Task 3: 4.2 Implementation (`docs/build.md`, README link, `project-structure.md`, `context.md` update entries with actual results — not placeholders; commit) | implementer |
| 17 | Task 3: 4.3 Code Review & Simplification (concurrent) + fixes if required | code-reviewer & code-simplifier → implementer |
| 18 | Task 3: 4.4 Documentation | docs-specialist |
| 19 | Task 3: 4.5b Overall Plan Adherence | architector |
| 20 | Task 3: 4.6 Task Completion (`[DONE]` on `### 3.`; commit) | implementer |
| 21 | Task 4: 4.1b Analysis & Planning → `.kilo/plans/20261009-phase-01-1-task4-completion-report.md` | architector |
| 22 | Task 4: 4.2 Implementation (run `scripts/dev-checks.sh` on the final tree via Docker/MCP; create `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` with factual evidence; commit) | implementer |
| 23 | Task 4: 4.3 Code Review & Simplification (concurrent) + fixes if required | code-reviewer & code-simplifier → implementer |
| 24 | Task 4: 4.4 Documentation | docs-specialist |
| 25 | Task 4: 4.5b Overall Plan Adherence | architector |
| 26 | Task 4: 4.6 Task Completion (`[DONE]` on `### 4.`; commit) | implementer |
| 27 | Step 5: TODO File Completion (rename to `-DONE`; clean tmp files; verify all committed; merge to `main`; delete branch; push `origin` ONLY; notify on failures) | implementer |
| 28 | Step 6: Finish — planner summary + next-TODO handoff snippet | planner |

## Per-Task Pre-Analysis

### Task 1 — Implement a Release Build Script

- Create `scripts/build-linux.sh` (POSIX `sh`), contract:
  1. Validate cwd = `/rust-youtube-streamer` (fails fast otherwise with a clear message referencing the documented Compose working directory).
  2. Require non-empty `CARGO_TARGET_DIR`; no default (explicit TODO requirement to resolve from "current CARGO_TARGET_DIR" and fail if missing).
  3. Print a start message ("Building Linux release executable…"); do not suppress cargo output.
  4. `cargo build --release --locked` (tracked lockfile).
  5. Binary path `$CARGO_TARGET_DIR/release/rust-youtube-streamer-service`; fail non-zero if missing/empty.
  6. `mkdir -p dist`; copy to `dist/rust-youtube-streamer-service` via `cp`; fail non-zero on copy failure.
  7. Verify final file exists and is non-empty (`test -s`); print exact output path (+ size if available) and exit 0, else non-zero.
- Expected actual binary: `/rust-streamer-target/release/rust-youtube-streamer-service` (package name, no `[[bin]]` override).
- Write `dist/` inside the mounted project → lands in the repo root and stays gitignored (verified in Task 2). No `target/` creation; intermediates stay in the named volume.
- Style: mirror `scripts/dev-checks.sh` conventions (`#!/bin/sh`, short header comment stating runtime contract, plain printf, `set`-free explicit checks or per-command checks — architector picks and freezes one approach so it is POSIX-safe and testable).
- Verification inside 4.2: run the real build via `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` through the Alpine VM MCP; record exit status and observed artifact; also validate at least one failure path (e.g., wrong cwd or unset `CARGO_TARGET_DIR`) to prove non-zero exit, without deleting the built artifact.
- No source changes; no `cargo fmt` auto-apply; no unrelated `dist/` deletions.

### Task 2 — Define Artifact Semantics and Safe Ignore Rules

- `.gitignore`: confirm `dist/` matches `git check-ignore dist/rust-youtube-streamer-service`; expected outcome: no rule change needed (rule exists since Phase 00). Change only if a demonstrated gap appears; do not unignore anything else.
- Post-build `git status` must show the binary as **ignored**, never staged; `git ls-files` check that nothing under `dist/` is tracked.
- Semantics documentation for this task is delivered in Task 3's `docs/build.md`; Task 2's own scope: verification + minimal note where documentation lands (placeholder-free). Keep description out of `scripts/build-linux.sh` duplication of Task 1's comments.
- Out of scope: volume removal, Docker prune, deleting existing outputs, README/docs rewrites (Task 3 owns those).

### Task 3 — Document the Build Workflow

- `docs/build.md`: TOC + sections — Purpose; Prerequisites; Build command (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`, working dir contract, image built once via `docker compose ... build rust`); Expected output (`dist/rust-youtube-streamer-service` at repo root on the Windows/VM-host share); Overwrite/rebuild behavior (rerun the script; artifact is disposable/re-creatable); Failure semantics (non-zero exit conditions list); What it proves/does NOT prove (Linux container build ≠ native Windows/Linux runtime validation; no `.exe`; streaming pipeline still not implemented; `auth`/`run` are placeholders); Relation to `scripts/dev-checks.sh` (checks validate fmt/types/tests/lints; build script produces a distributable release artifact; run dev-checks first).
- README: add TOC entry + a short "Release Build (Linux Artifact)" section linking `docs/build.md`; keep all unrelated sections intact (minimal diff).
- `.agent/project-structure.md`: add `scripts/build-linux.sh`, `docs/build.md`, and root `dist/` (gitignored build-artifact output; exception to the no-root-generated-files rule).
- `.agent/project-info/context.md`: append Phase 01.1 entry with the **actual** implementation facts, exact verified command(s) + exit statuses, output path, platform caveat, and next phase (Phase 02). Only real, already-observed values (available because Task 1 completed its verification before this task runs).
- Markdown Generation Rule: these doc files are written by docs-specialist/implementer sub-agents under the workflow's authority; the planner reviews.

### Task 4 — Save a Durable Completion Report

- Run the standard gate on the **final tree**: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` (exit 0 expected; record real status; if it fails, stop and report exact failure — do not commit a failing tree as "done").
- Create `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` (tracked Markdown) with six mandatory factual sections: files created/changed; exact Docker/MCP commands executed with exit statuses; build success + observed artifact path/size (only if observed); standard dev-checks result on the final tree; confirmation `dist/` is gitignored and artifact untracked (`git check-ignore` + `git status`/`git ls-files` evidence); known limitations & skipped checks with reasons (no native runtime validation, MCP stderr caveat, app still placeholder).
- No fabricated outputs, sizes, commit IDs, or validations; unverifiable items recorded as "not run" + reason.
- Report establishes the ongoing convention: future phase TODOs require a tracked completion report under `.agent/reports/`.

## Out of Scope (global, from TODO "Constraints and Out of Scope" and "Acceptance Criteria")

- No application source changes; no OAuth/YouTube/chat/renderer/FFmpeg/game/config-schema work; no Windows cross-compilation or `.exe`; no installer/service/CI/deployment; no Dockerfile/Compose changes unless a demonstrated blocker (then stop and report); no volume cleanup; no host-side Rust install; no replacement/duplication of `dev-checks.sh`; never claim container compilation as native runtime validation.
- TODO must NOT be renamed `-DONE` until all acceptance criteria and workflow steps are complete (Step 5 handles the rename).
