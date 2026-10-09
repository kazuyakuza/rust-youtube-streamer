# Global Plan — Phase 00: Repository Foundation and Build Baseline

- Source TODO: `.agent/todos/20261008/20261008-todo-2.md` (Pattern C: 5 tasks under `## Scope and Tasks`)
- Workflow: `.kilo/commands/critical-workflow.md`
- Date: 2026-10-08
- Front-end related tasks: NONE (no 4.1a/4.5a steps for any task)

## Global Pre-Analysis

### Repository state (verified by Planner)
- Branch: `main`, up to date with `origin/main`.
- Unstaged working-tree changes (to be committed in Step 2):
  - deleted: `.agent/todos/.gitkeep`
  - modified: `.agent/todos/20261008/20261008-todo-2.md`
  - modified: `CHANGELOG.md`
- No `Cargo.toml`, no `Cargo.lock`, no `src/main.rs`. `src/{app,config,youtube,chat,renderer,streaming}/` contain only `.gitkeep` placeholders.
- `.gitignore`: has OS/tmp/logs/env/IDE/build-generic rules; NO Rust entries yet (`target/` not ignored); no `config/config.json`/credentials pattern yet.
- README exists ("Current Status" says repo not yet scaffolded — will be updated in Task 5).
- No version file exists yet (`package.json` absent). Version is established with `Cargo.toml` (0.1.0) in Task 2 → Step 3 is a documented no-op this phase.

### Technical & Architecture decisions (global)
1. **Build/validation route**: Rust/Cargo are NOT on the Windows implementation host. All cargo runs go through the `alpine-vm` MCP:
   - Tools: `alpine-vm_vm_status` (check first), `alpine-vm_vm_run_command`.
   - VM project mount: `/rust-youtube-streamer`.
   - Command allowlist prefixes: `docker`, `sh`, `apk`, `ls`, `cat`, `ps`, `df`, `free`, `uname`, `pwd`, `whoami`.
   - All Rust checks run as single `docker ...` commands (no `&&` chains), executed sequentially where output (e.g., `Cargo.lock`, `target/`) feeds later commands.
2. **Docker setup**: One Compose service (`docker-compose.yml`) pinned to an official maintained Rust image at a deliberate stable version (e.g., `rust:1.82` — architector verifies the current stable pin); no Dockerfile unless the pin needs custom layers. Project dir bind-mounted into the container; `working_dir` set to the mount; non-root user handling (UID/GID) via compose env so generated files don't get unexpected ownership; no privileged mode. No CI, no production image, no FFmpeg in the image.
3. **Cargo package**: single executable `rust-youtube-streamer-service`, edition 2021 (current stable verified by architector), version 0.1.0, zero dependencies. `src/main.rs` = minimal idiomatic `fn main()` proof-of-build entry point. No speculative deps, no empty module files.
4. **Git hygiene**: `Cargo.lock` tracked; `target/` ignored; secrets policy preserved (real `config/config.json`, OAuth tokens, stream keys, logs never committed).
5. **Validation honesty**: results reported only from actual MCP/VM output; Linux-container build checks are NOT claimed as Windows/Linux runtime validation.

## Step Table

| Step | Workflow Step | Sub-agent |
|---|---|---|
| 1 | Step 2: Git Feature Branch Setup (branch `feat/phase00-cargo-baseline`; commit pending changes) | implementer |
| 2 | Step 3: Version Update (no-op: no version file exists yet; documented) | implementer |
| 3 | Task 1: 4.1b Analysis & Planning | architector |
| 4 | Task 1: 4.2 Implementation (inspection + findings) | implementer |
| 5 | Task 1: 4.3 Code Review & Simplification (+fixes) | code-reviewer & code-simplifier → implementer |
| 6 | Task 1: 4.4 Documentation (if required) | docs-specialist |
| 7 | Task 1: 4.5b Overall Plan Adherence | architector |
| 8 | Task 1: 4.6 Task Completion (`[DONE]` + commit) | implementer |
| 9–14 | Task 2: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 (same sub-agent sequence) | per step |
| 15–20 | Task 3: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | per step |
| 21–26 | Task 4: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | per step |
| 27–32 | Task 5: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | per step |
| 33 | Step 5: TODO File Completion (rename `-DONE`, cleanup, merge to `main`, push `origin` only) | implementer |

## Per-Task Pre-Analysis

### Task 1 — Inspect the Current Repository (### 1)
- Non-front-end. Mostly read/verify work inside Task 1's 4.2.
- Inspect branch/working tree, root files, `src/` dirs, config/docs dirs, `.gitignore`, `.agent/project-structure.md`.
- Verify MCP route BEFORE any build command: `alpine-vm_vm_status`, then a harmless `docker ps` / `ls /rust-youtube-streamer` via `alpine-vm_vm_run_command`.
- Report deltas vs `.agent/project-info/context.md` (known unstaged changes listed above).
- Deliverable: findings recorded in the sub-agent completion report; context.md update deferred to Task 5. Likely no repo file changes.

### Task 2 — Establish the Minimal Cargo Project (### 2)
- Non-front-end. Files to create: `Cargo.toml` (name `rust-youtube-streamer-service`, `edition = "2021"`, `version = "0.1.0"`, no `[dependencies]`), `src/main.rs` (minimal `fn main()`), generated `Cargo.lock`.
- Lockfile generated inside the container on the VM (`docker compose run --rm <svc> cargo generate-lockfile` or via `cargo check`), then committed.
- No config parsing, logging, OAuth, YouTube calls, rendering, chat, or FFmpeg code.

### Task 3 — Reproducible Docker-Based Development Checks (### 3)
- Non-front-end. Files: `docker-compose.yml` (+ `Dockerfile` only if justified) with pinned Rust image; documented UID/GID handling.
- Sequential container checks (each a single `docker compose run --rm <svc> ...` via `alpine-vm_vm_run_command`):
  1. `cargo fmt` (if fmt --check fails) → 2. `cargo fmt --check` → 3. `cargo check --locked` → 4. `cargo test --locked` → 5. `cargo clippy --locked -- -D warnings`
- Report exact exit statuses from MCP output; never report unexecuted commands as passing; state validation scope precisely (Alpine VM + Linux container).
- Document exact commands in docs (Task 5) and verify `Cargo.lock` stays in the shared dir.

### Task 4 — Establish Repository Hygiene (### 4)
- Non-front-end. Add `.gitignore` entries: `/target/`; explicit ignores for real secrets/runtime artifacts (`config/config.json`, OAuth token/credential files, stream-key material) consistent with existing policy; keep `Cargo.lock` tracked (explicitly NOT ignored).
- Do not ignore FFmpeg binaries/bundled artifacts (none exist) — policy statements preserved.

### Task 5 — Update Documentation and Project Metadata (### 5)
- Non-front-end. Updates: `README.md` (minimal build baseline + Docker/MCP/VM build instructions, factual platform claims, no host-Rust requirement), `.agent/project-structure.md` (add `Cargo.toml`, `Cargo.lock`, `src/main.rs`, Docker/Compose files), `.agent/project-info/context.md` (resulting state, checks performed, MCP/VM/container setup without secrets, limitations, next step).
- Project Info closing step per `.agent/project-info/instructions.md`.

## Per-Task Plan Paths (4.1b)

- Task 1: `.kilo/plans/20261008-phase00-task1-inspect-repo.md`
- Task 2: `.kilo/plans/20261008-phase00-task2-minimal-cargo.md`
- Task 3: `.kilo/plans/20261008-phase00-task3-docker-checks.md`
- Task 4: `.kilo/plans/20261008-phase00-task4-repo-hygiene.md`
- Task 5: `.kilo/plans/20261008-phase00-task5-docs-metadata.md`

(Files to be generated by the architector during each task's 4.1b.)

## Approvals
- Global plan: pending user approval (question tool).
- Per-task plans: auto-approve if user picks "Approve Global and Tasks Plans".
