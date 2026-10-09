# Implementation Plan — Phase 00, Task 5: Update Documentation and Project Metadata

- TODO: `.agent/todos/20261008/20261008-todo-2.md` — task heading `### 5. Update Documentation and Project Metadata`
- Workflow step: 4.2 (implementation of this docs-only task). This task IS a documentation task; per caller decision, the docs-specialist role is subsumed into 4.2 here. Steps 4.3, 4.4 (verification-only pass), 4.5b are codified at the end of this plan for the caller.
- Branch: `feat/phase00-cargo-baseline` (already exists — do NOT create/switch branches; branch setup was step 2 and is closed).
- Executor: JUNIOR developer under 50% restriction. All structural decisions are encoded below; do not invent unspecified content.

[Project Info: Active]

---

## 0. Fixed Facts (authoritative — do not re-derive)

These are verified facts from Tasks 1–4. The implementer must copy them verbatim into the docs; do not research, verify, or embellish them.

- Package: `rust-youtube-streamer-service`, version `0.1.0`, Rust edition 2021, zero dependencies.
- `src/main.rs`: minimal `fn main() {}` entry point only.
- `Cargo.lock`: tracked in git.
- Container image: `rust:1.82` (pinned, official Docker image); Dockerfile also runs `rustup component add rustfmt clippy`.
- Docker Compose service name: `rust`; file `docker-compose.yml`; build context `.`; working_dir `/rust-youtube-streamer`; bind mount `/rust-youtube-streamer:/rust-youtube-streamer`; named volume `rust-streamer-target` mounted at `/rust-streamer-target`; env `CARGO_TARGET_DIR=/rust-streamer-target`.
- Docker Compose version observed: v2.31.0 on the Alpine VM.
- Image built explicitly: `rust-youtube-streamer-rust` (exit 0).
- All four checks ran in the container with exit 0:
  - `cargo fmt --check` (no apply needed)
  - `cargo check --locked`
  - `cargo test --locked` (0 tests)
  - `cargo clippy --locked -- -D warnings`
- Command invocation pattern: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo <subcmd> ...`
- Ownership observed on the shared mount: `root:vboxsf`.
- MCP/VM captured output may omit cargo's stderr `Finished` lines; the exit status is the authoritative success evidence. Do not interpret missing output as failure.
- Host (Windows implementation-agent environment) needs NO Rust/Cargo installed.
- FFmpeg: external prerequisite only — a minimal mention, no guide (deferred to a later docs phase).
- Not part of this phase: OAuth authorization ("first authorization" etc.), runtime features, CI, services, installers.

## 1. Files to Change (and ONLY these)

1. `README.md` (edit in place)
2. `.agent/project-structure.md` (edit in place)
3. `.agent/project-info/context.md` (edit in place)

Do NOT touch: `.agent/project-info/brief.md`, `product.md`, `architecture.md`, `tech.md`, `.gitignore`, `src/**`, `Dockerfile`, `docker-compose.yml`, `Cargo.toml`, `Cargo.lock`, any `docs/*` file, any plan/TODO file, `.agent/CONTEXT.md` (uppercase copy is a system-generated duplicate — do not edit it).

## 2. Step-by-step

### Step A — README.md edit

Current `## Current Status` section (lines 25–30) must be replaced in place. All other sections (`MVP Summary`, `Key Technologies`, `AI Agents`, `How to Start a Task`, intro paragraph, closing italic line) stay byte-identical. Do not rewrite the README.

**A1. Replace the `## Current Status` section content** (keep the `## Current Status` heading) with exactly:

```markdown
## Current Status

Phase 00 (repository foundation and build baseline) is complete:
a minimal Cargo package exists (`rust-youtube-streamer-service`, Rust edition 2021, no dependencies)
with `src/main.rs` as the smallest buildable entry point. Build checks run inside Docker on the
Alpine VM — see [Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm) below.
No application feature (OAuth, chat, rendering, streaming) exists yet; each feature module will be
added by its own phase.
```

**A2. Insert a new section immediately after the `## Current Status` section** (i.e., between it and `## AI Agents`), with exactly this heading and content:

```markdown
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
```

Formatting: use fenced code blocks as shown; the markdown heading anchor in the Current Status link matches the heading above. Keep the final italic footer line (`*Initial README — detailed documentation ...*) as the last line of the file — the new section sits above `## AI Agents`, so the footer remains last.

**A3. FFmpeg mention check**: with the sections above, no FFmpeg statement is added at all — this satisfies "minimal mention" (the brief's existing `Key Technologies` FFmpeg bullets remain untouched). Do not add any FFmpeg prerequisite section.

### Step B — .agent/project-structure.md edit

Existing content must be preserved. Additions only.

**B1. Add a new section `# Root files` between `# Folders in src/` and `# Other folders`** with these five entries:

```markdown
# Root files

- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.1.0, edition 2021, no dependencies
- Cargo.lock - tracked lockfile for the executable package (sole root-level generated artifact)
- src/main.rs - minimal executable entry point proving the package builds (no features yet)
- Dockerfile - pinned official `rust:1.82` image; installs rustfmt and clippy components for build checks
- docker-compose.yml - `rust` service for Docker-based cargo checks; bind-mounts the project and redirects build artifacts to the `rust-streamer-target` named volume
```

**B2. In `# Folders in src/`, append one new first-listed-style line at the TOP of that folder list** (before `app/`), describing the placeholder state:

```markdown
- module folders contain only `.gitkeep` placeholders; feature modules are added by their respective phases
```

All six existing folder descriptions (`app/` through `streaming/`) remain untouched and in current order.

**B3. `# Other folders` section: add nothing.** (Named-volume artifact note already covered by the `docker-compose.yml` root-file entry; scope stays file/structure entries only.)

### Step C — .agent/project-info/context.md edit

Update ONLY the sections `Current Work Focus`, `Recent Changes`, `Immediate Next Steps`, and `Notes`. Keep `Scope Decisions` unchanged. Keep the header blockquote unchanged.

**C1. `## Current Work Focus`** — replace its paragraph with:

```markdown
Phase 00 (repository foundation and build baseline) is complete: the minimal Cargo package,
Docker-based build checks via the Alpine VM, and repository hygiene are in place. The next phase
will begin the first real application module.
```

**C2. `## Recent Changes (2026-10-08)`** — keep the six existing bullets and append these new bullets to the end of the list (exact commit-free facts):

```markdown
* Task 2 of Phase 00: added `Cargo.toml` (package `rust-youtube-streamer-service` v0.1.0, edition 2021, no dependencies), `src/main.rs` minimal entry point, and generated `Cargo.lock`; `Cargo.lock` is tracked.
* Task 3 of Phase 00: added `Dockerfile` (pinned official `rust:1.82`, plus rustfmt/clippy components) and `docker-compose.yml` (`rust` service; project bind-mounted at `/rust-youtube-streamer`; `CARGO_TARGET_DIR` redirected to named volume `rust-streamer-target`).
* Build checks executed with Docker Compose v2.31.0 on the Alpine VM via the `alpine-vm` MCP, commands pattern `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo <subcmd>`: `fmt --check` exit 0, `check --locked` exit 0, `test --locked` exit 0 (0 tests), `clippy --locked -- -D warnings` exit 0. MCP/VM captured output may omit cargo stderr `Finished` lines; exit status is authoritative.
* Ownership on shared mount entries: `root:vboxsf`.
* Task 4 of Phase 00: `.gitignore` extended — `/target/`, `/config/config.json`, `credentials/*` with `!credentials/.gitkeep`, and a `!Cargo.lock` guard. Build artifacts never written to project root (named-volume redirect; `Cargo.lock` is the sole root exception, tracked).
* Task 5 of Phase 00: README 'Current Status' rewritten to the build baseline; new 'Build Checks (Docker via Alpine VM)' section added; `project-structure.md` updated with root files and placeholder note; this context file updated.
* README documents that the implementation-agent host needs no Rust/Cargo install, and distinguishes container-only build checks from native Windows/Linux runtime validation.
```

**C3. `## Immediate Next Steps`** — keep numbered items as a new list; replace item 1 only, and drop nothing else that still applies to later phases (old items 2 and 3 remain, renumbered):

```markdown
1. Plan next phase TODO (first application module) in a new chat session referencing updated project info.
2. In later phases, re-read the current repository and project info before drafting each new TODO, as requested.
3. Add the separate runtime prerequisites/permissions document and link it from README in the appropriate later documentation phase; it must state FFmpeg is a preinstalled external prerequisite, the executable runs as a normal user from a writable/readable location, and the app is not installed or registered as a Windows service.
```

Do NOT invent detailed future plans (no specific module name beyond "first application module"; user policies note: first authorization / OAuth is NOT part of any announced phase — say nothing about it here).

**C4. `## Notes`** — append these lines below the existing `brief.md` line:

```markdown
* Validation scope of Phase 00 checks: build/type/test/clippy checks inside the Linux container on the Alpine VM only; no native Windows or Linux runtime validation was performed, and no application runtime exists yet.
* `context.md` contains no secrets, credentials, tokens, or machine-specific paths.
```

## 3. Git handling

Branch is `feat/phase00-cargo-baseline`; no branch creation/switch. Two commits, in this exact order:

1. After Step A only: `git add README.md` → commit message:
   `docs: document Docker build workflow and current baseline`
2. After Steps B and C: `git add .agent/project-structure.md .agent/project-info/context.md` → commit message:
   `docs: update project structure and context for Phase 00 results`

Gitignore compliance (mandatory before each commit):
- `git status` — must show ONLY the intended file(s) staged; never `git add -A` or `git add .`.
- Confirm no `target/`, no `config/config.json`, no `credentials/*` (except `.gitkeep`), no `.agent/CONTEXT.md` duplicate is staged.

No push (push is restricted to step 5 of the workflow).

## 4. Verification codified for the caller (4.3 / 4.4 / 4.5b applicability)

- **4.3 REQUIRED** (docs files reviewed): code-reviewer/code-simplifier review README.md, `.agent/project-structure.md`, `.agent/project-info/context.md` for factual errors vs the Fixed Facts in section 0 and fidelity to the plan snippets. Markdown-only; simplification plans go to `.kilo/plans/`.
- **4.4 REQUIRED-lite**: the docs-specialist role was fulfilled within 4.2 of this task (this task is itself documentation). The 4.4 pass reduces to: verify no remaining doc gaps against TODO Task 5 sub-items (README build instructions; structure file; context file) — checklist below, in the junior implementer's completion checklist.
- **4.5b REQUIRED**: architector checks adherence of the resulting docs to this plan section by section (A1/A2 headings and bullets verbatim; B1 five entries; C2 seven bullets; C4 two notes), and that no non-listed file was modified (`git diff --name-only` shows exactly the three files).

## 5. Acceptance criteria mapping (TODO section 5)

| TODO Task 5 sub-item | Covered by | Covered where in this plan |
|---|---|---|
| README minimal update, no full rewrite | Step A | A1, A2, A3 |
| Build instructions: prerequisites, how to invoke, where commands execute; no host Rust/Cargo | Step A | A2 Prerequisites/Commands/Notes |
| Factual platform claims (Alpine VM + container image; distinguish from native runtime validation) | Step A + C | A2 Notes bullets 3–4; C2 last bullet; C4 note 1 |
| project-structure.md reflects Cargo.toml/lock, main.rs, Docker files; existing module descriptions retained | Step B | B1, B2, B3 |
| context.md updated: resulting state, checks run, MCP/VM setup without secrets, limitations, next step | Step C | C1–C4 |
| Tests and docs part of acceptance criteria | — | (checks already run in Task 3; this task is docs-only) |

Remaining TODO acceptance boxes this task closes (Task 6/step 4.6/TODO completion step marks them, not here): "README build instructions describe the Docker/MCP/VM workflow..." and "`.agent/project-structure.md` and `.agent/project-info/context.md` reflect the resulting repository state."

## 6. Completion checklist + report skeleton (items 1–7 of TODO "Required Completion Report")

The implementer returns exactly this skeleton, filled:

```markdown
## Task 5 Completion Report
1. Files created or changed: README.md, .agent/project-structure.md, .agent/project-info/context.md (nothing else)
2. Exact Docker/MCP commands executed and results: none in Task 5 (docs-only; checks were executed in TODO task 3 with exit 0 — see context.md)
3. Alpine VM details and Rust container image/toolchain used: Alpine VM, Docker Compose v2.31.0, official rust:1.82 image with rustfmt/clippy components (from Task 3, documented only)
4. OS/toolchains actually tested and precise validation scope: build checks in Linux container only (rust:1.82 on Alpine VM); no native Windows/Linux runtime validation
5. Checks not run and why: no Docker commands re-run in Task 5 — documentation-only task
6. Assumptions or blockers: <none expected; report if any>
7. No out-of-scope feature confirmed: yes — no app feature, no FFmpeg install, no service registration, no installer, no CI, no elevation
```

Checklist before signaling completion:
- [ ] README: all non-`Current Status` sections byte-identical to before.
- [ ] README: new section heading exactly `## Build Checks (Docker via Alpine VM)`, placed before `## AI Agents`.
- [ ] project-structure.md: `# Root files` has exactly the five entries; six module descriptions unchanged.
- [ ] context.md: `Scope Decisions` unchanged; header blockquote unchanged.
- [ ] context.md: stderr-capture note present (C2 bullet 3).
- [ ] Two commits with the exact messages above; `git status` clean after; no gitignored files staged.
- [ ] `git diff --name-only` (vs previous commit baseline) shows ONLY the three files.

## 7. Verification of this plan against the task

- TODO Task 5 sub-items → all mapped in section 5. ✔
- Caller STEP 4.1b items 1–5 → covered (files read; junior plan; 4.3/4.4/4.5b codified §4; criteria mapping §5; completion skeleton §6). ✔
- Out-of-scope guardrails (brief.md untouched, no README rewrite, no FFmpeg guide, no invented future plans) encoded. ✔
