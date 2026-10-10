# Implementation Plan — Phase 01.1, TODO Task 3: Document the Build Workflow

Step 4.1b (architector). Input: `C:\repo\rust-youtube-streamer\.agent\todos\20261009\20261009-todo-4.md` (Task 3, section "### 3. Document the Build Workflow"), its "Constraints and Out of Scope", and the Task-3-relevant acceptance criteria rows.

## 1. Task Identity and Live State (verified 2026-10-10)

- Repo: `C:\repo\rust-youtube-streamer`; branch `feat/linux-release-build-artifact`; HEAD `858e378`; tree clean (`git status --short --branch` ⇒ `## feat/linux-release-build-artifact`).
- `Cargo.toml` version is `0.3.0` (bumped in step 3, commit `f9a6b0e`).
- Tasks 1–2 are complete and marked `[DONE]` in the TODO file. Their completion commits (`8448f0a`, `858e378`) changed ONLY the TODO file — verified with `git show --stat`. Therefore `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` are still in their pre-Phase-01.1 state; Task 3 performs all edits on them.
- Non-front-end task → 4.1a skipped; 4.1b only. Docs-only change.

## 2. Facts Inventory (observed; may be cited as verified in the docs)

- [Task 1] Standard invocation `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` → exit 0.
- Success messages printed by the script:
  - M1 `Building Linux release executable using the tracked lockfile: cargo build --release --locked`
  - M2 `Release build succeeded.`
  - M3 `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (1715296 bytes).`
  - M4 `The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe.`
- Failure paths observed (both exit 1, single `build-linux: error:` line on stderr):
  - E1-equivalent (script run from `/tmp`): `build-linux: error: expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: /tmp`
  - E2 (empty `CARGO_TARGET_DIR`): `build-linux: error: CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)`
- Artifact: `dist/rust-youtube-streamer-service`, 1715296 bytes, mode `-rwxrwx---` `root:vboxsf` (shared-mount ownership), ignored via `.gitignore` line 34 (`dist/`). No `target/` directory in the project root.
- `scripts/build-linux.sh` mode is `100644` (no exec bit); invocation convention is `sh scripts/build-linux.sh`, same as `scripts/dev-checks.sh`.
- `docker-compose.yml`: service `rust`, `working_dir: /rust-youtube-streamer`, bind mount `/rust-youtube-streamer:/rust-youtube-streamer`, named volume `rust-streamer-target:/rust-streamer-target`, env `CARGO_TARGET_DIR: /rust-streamer-target`. Docker Compose observed v2.31.0 on the Alpine VM.
- Cargo build `Finished ... in 33.04s` observed once (cold run); diagnostics are never suppressed by the script.
- `cargo build --release --locked` is the only Cargo command the script runs; the tracked `Cargo.lock` is authoritative.
- README current sections (in order): MVP Summary; Key Technologies; Current Status; Commands and Configuration; Logging (RUST_LOG); Build Checks (Docker via Alpine VM); AI Agents; How to Start a Task. TOC lines 9–16; `## AI Agents` at line 149; the Build Checks `### Notes` block ends at line 147.
- `docs/` currently holds `configuration.md` (117 lines: `# Title` → intro → `## Table of Contents` → sections → `## See Also`) plus `how-to-set-up-git.md` and `how-to-write-todo-files.md`. `docs/build.md` must follow the `configuration.md` conventions.
- `.gitignore` line 33–34: generic build artifact ignore, `dist/` present.
- Commit style on this branch: lowercase conventional (`docs: ...`, `feat: ...`, `fix: ...`, `chore: ...`).

## 3. Frozen Decisions

1. **No Alpine-VM commands in this task.** The docs are static text; Task 1's verified facts suffice. NO `vm_status`, no `docker` re-runs, NO `scripts/dev-checks.sh` run here (the final-tree dev-checks run belongs to Task 4 verification). Docs validation is pure markdown self-check (Step 6 of §4).
2. **`docs/build.md` section list and order** (9 content sections + TOC; all headings chosen to produce clean GitHub anchor slugs — no `/`, `(` or `·` in headings):
   1. `# Build the Linux Release Executable`
   2. `## Table of Contents`
   3. `## Purpose`
   4. `## Prerequisites`
   5. `## Build Command`
   6. `## Expected Output`
   7. `## Overwrite and Rebuild Behavior` — includes artifact disposal/re-creatability (caller item folded here; TODO covers both in one sentence)
   8. `## Failure Semantics` — E1–E7 list in user terms
   9. `## What This Proves and Does Not Prove` — Linux-not-.exe caveat lives here (per handoff note 1)
   10. `## Dev Checks vs. Release Build`
   11. `## See Also`
   TOC anchor list (exact GitHub slugs; TOC must link all nine content sections):
   - `#purpose`
   - `#prerequisites`
   - `#build-command`
   - `#expected-output`
   - `#overwrite-and-rebuild-behavior`
   - `#failure-semantics`
   - `#what-this-proves-and-does-not-prove`
   - `#dev-checks-vs-release-build`
   - `#see-also`
3. **End-user phrasing contract (handoff note 1):** `docs/build.md` describes what the build produces and where it lives. FORBIDDEN: restating the `scripts/build-linux.sh` header contract verbatim (a third copy of the script-header text). FORBIDDEN raw shell-variable terminology as prose (`$CARGO_TARGET_DIR`, `pwd`-style phrasing) — express E1/E2 as "started from a different working directory" / "not started by the documented Compose service".
4. **Artifact size number is NOT copied into `docs/build.md`** (byte count would go stale with code; the script itself prints the size in M3). The size 1715296 and mode/ownership `-rwxrwx--- root:vboxsf` go ONLY into the `context.md` factual entry (frozen facts, never placeholders).
5. **README edit points frozen (handoff note 2):** exactly two edits — (a) one new TOC entry line after the `Build Checks (Docker via Alpine VM)` TOC line; (b) one new short section `## Release Build (Linux Artifact)` inserted immediately before `## AI Agents` (i.e., right after the Build Checks section). Anchor: `#release-build-linux-artifact`. NO other README changes (do not touch MVP Summary, Key Technologies, Current Status, Logging, Build Checks body, AI Agents, How to Start a Task, the TOC intro line, or the trailing italic line).
6. **`.agent/project-structure.md` edits frozen (4 points; handoff note 3 + staleness flag):**
   - (a) `scripts/` entry gains `build-linux.sh` description.
   - (b) `docs/` entry gains `build.md`.
   - (c) New `dist/` bullet under `# Other folders` (gitignored output; explicit exception to the no-root-generated-files rule).
   - (d) `Cargo.toml` entry refreshed `v0.2.0` → `v0.3.0`. Justification: the acceptance-criteria row "`.agent/project-structure.md` ... reflect the final state" plus Task 3 must produce a structure file that reflects the final state; a knowingly-stale version contradicting the final state would fail 4.5b adherence review. Bounded to the version number only.
7. **`.agent/project-info/context.md` edits frozen (3 points; handoff note 4):** Current Work Focus rewrite; new `## Recent Changes (2026-10-09, Phase 01.1)` section; Immediate Next Steps item-1 update. Existing entries/sections stay byte-identical; voice matches existing entries (terse factual bullets, `*` bullets, no first person).
8. **Commit strategy frozen — 3 logical commits** (messages exact, lowercase conventional, consistent with branch history):
   1. `docs: add linux release build guide` — files: `docs/build.md`
   2. `docs: link build guide from readme` — files: `README.md`
   3. `docs: record build workflow in structure and context` — files: `.agent/project-structure.md`, `.agent/project-info/context.md`
9. **TODO file / step-5 mechanics untouched here:** no `[DONE]` flips, no acceptance-criteria checkbox edits, no `.agent/reports/` creation (that file belongs to Task 4), no TODO renaming/commit for 4.6 within this sub-task; the Planner handles 4.6.

## 4. Step-by-step implementation (Junior implementer, follow exactly)

### Step 0 — Preconditions (do all, in order; stop and ask if any fails)

1. `git status --short --branch` ⇒ expect `## feat/linux-release-build-artifact` and NO file entries (tree clean). If NOT clean, stop and ask the caller — never commit or stash unknown files.
2. Read the current versions of the four target files (README.md; docs/; .agent/...; context.md) before editing (tool contract requires Read before Edit).
3. Confirm the TODO file's Task 3 section is still the input as described here (it is; it was already read). No further TODO-file edits.

### Step 1 — Create `docs/build.md` (commit 1)

File location: `C:\repo\rust-youtube-streamer\docs\build.md`. Target length ≈ 100–150 lines (docs are exempt from the 200-line source rule, but stay concise like `configuration.md`, which is 117 lines). The file content must use real line breaks (never literal `\n` escape sequences). No emoji. No HTML except fenced code blocks.

Skeleton with frozen headings. For each section: "mandatory content" bullets are the required statements; keep them in the order given, each as its own short paragraph or bullet; the wording itself may be polished slightly but the meaning must stay verbatim-faithful.

````markdown
# Build the Linux Release Executable

One-sentence intro: this guide explains how to build the distributable release
executable `dist/rust-youtube-streamer-service` inside the pinned Linux Rust container.

## Table of Contents
(exactly the nine frozen anchor links from Frozen Decision 2, same order)

## Purpose

Mandatory content:
- The repository ships two separate Docker-based workflows: fast validation checks and a release build. This guide covers only the release build.
- The build compiles the service in release mode (`cargo build --release --locked`) inside the pinned Linux Rust container and leaves exactly one final file behind.
- Success criterion from the user's perspective: the command exits `0` and prints, among its messages, `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (<size> bytes).`

## Prerequisites

Mandatory content:
- Same Docker prerequisites as the regular checks (link the README section: `[Build Checks (Docker via Alpine VM)](../README.md#build-checks-docker-via-alpine-vm)`): Alpine VM running with the project shared at `/rust-youtube-streamer`, Docker with the Compose plugin (Compose v2.31.0 observed), and the container image built once with `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust`.
- No Rust or Cargo toolchain is needed on the computer that runs the build command; everything happens inside the container.
- The build always compiles against the version-locked dependency set recorded in the tracked `Cargo.lock`; users never edit or regenerate it for this workflow.

## Build Command

Mandatory content:
- The standard invocation (single command users run):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

- Explain it starts the Compose `rust` service from the repository working directory and runs the repository's tracked script `scripts/build-linux.sh` (POSIX `sh`, invoked the same way as `scripts/dev-checks.sh`). A first, cold full compile finishes on the order of half a minute; subsequent runs reuse cached intermediate output, so rebuilds are fast.
- Cargo's own progress/error output is shown as it happens: the script deliberately does not suppress Cargo diagnostics.

## Expected Output

Mandatory content:
- Output path on success: `dist/rust-youtube-streamer-service` in the repository root — a Linux executable produced inside the Linux container; it is not a Windows `.exe`. Whether the streaming pipeline actually works is a separate matter — see What This Proves and Does Not Prove.
- The script prints the final path and size; the exact byte count naturally changes as the codebase changes (never hardcode the size in this guide).
- `dist/` is gitignored: the executable is a build output, never committed.
- Cargo's intermediate build files never appear in the repository root; they live in the Docker named volume `rust-streamer-target`. The root stays clean except the tracked `Cargo.lock`.

## Overwrite and Rebuild Behavior

Mandatory content:
- Rerunning the standard command rebuilds from the current source and overwrites the previous `dist/rust-youtube-streamer-service` with a fresh copy.
- Intermediate build caches persist in the Docker volume, so routine rebuilds are incremental and fast; dependency resolution is never re-run — `--locked` fails fast if `Cargo.lock` does not match the manifest (see Failure Semantics).
- The artifact is disposable: deleting it has no repository effect; run the standard command again to regenerate it at any time. Nothing else in the workflow removes or prunes anything: Docker volumes are never pruned, and the script never deletes unrelated files under `dist/`.
- On a failed run nothing is placed in `dist/`; if a previous successful artifact exists, it stays in place untouched (the script only writes the artifact after a fully successful build-and-copy sequence).

## Failure Semantics

Mandatory content (user-facing framing; every error line begins with the literal prefix `build-linux: error:` and the command exits `1`; one or more lines appear on standard error; Cargo's diagnostics, if any, are printed in full, never suppressed). Freeze this 7-row list — condition + consequence in user terms (E-id labels may carry over from internal records but the doc must present them as plain condition rows):

| # | If this happens | What you see |
|---|---|---|
| 1 | The command is not run through the documented Compose invocation (e.g. the script runs from some other working directory). | error: expected the current working directory to be /rust-youtube-streamer ... got: <directory>; exit 1 before any build starts. |
| 2 | The Compose environment variable that names the release-output folder (`CARGO_TARGET_DIR`) is missing or empty — in practice: the build was not started by the Compose service above. | error: CARGO_TARGET_DIR is unset or empty ... docker-compose.yml sets it to /rust-streamer-target; exit 1. |
| 3 | The Cargo release build itself fails (e.g. compile error, or `Cargo.lock` out of date with the manifest). | Cargo's full error output followed by error: cargo build --release --locked failed with exit code <code> ... diagnostics above are not suppressed; exit 1. |
| 4 | After a successful build the expected release executable is not found or is empty. | error: expected release binary not found or empty: <path>; exit 1. |
| 5 | The `dist/` directory cannot be created. | error: could not create the dist/ directory: <path>; exit 1. |
| 6 | Copying the build output to `dist/` fails. | error: failed to copy <source> to <destination>; exit 1. |
| 7 | After the copy, the final file is missing or zero bytes. | error: final artifact is missing or empty after the copy: <path>; exit 1. |

Plus one closing sentence: a non-zero exit means no new artifact was produced; fix the reported condition and run the standard command again.

## What This Proves and Does Not Prove

Mandatory content (frame the limitation as user-facing; from the TODO: "A successful Linux container build does not prove native Windows or native Linux runtime behavior, does not create a Windows executable, and does not mean the application streaming pipeline is implemented"):
- What it proves: the current source compiles in release mode with the pinned Linux Rust toolchain against the tracked lockfile, inside the Linux container, and produces a non-empty Linux executable file.
- What it does NOT prove (required bullets):
  - It is NOT native Windows or native Linux runtime validation of the produced executable's actual behavior (streaming, config load on a real host, logging on a real host).
  - It does NOT create a Windows `.exe` and involves no cross-compilation; the artifact runs on Linux only.
  - It does NOT mean the application's features are implemented: OAuth `auth` and `run` streaming are still placeholders that report not-implemented and exit code 3 after configuration validation.

## Dev Checks vs. Release Build

Mandatory content:
- The check script (`scripts/dev-checks.sh`, documented in README Build Checks) runs formatting checks, type/build checks, unit tests and lint checks inside the same container, and produces NO artifact; its log goes to gitignored `logs/checks/`.
- The build script documented here produces the distributable release executable under `dist/`.
- One-line distinction to include verbatim: "The dev checks validate the code; the release build produces the distributable executable."
- Both scripts run `sh` invoked the same way (`sh scripts/<script>.sh` via the Compose service) and neither modifies source files.

## See Also

Mandatory content (bullet list):
- [`README`](../README.md) — project overview, and the Build Checks section covering the Docker environment shared with this workflow.
- [`scripts/dev-checks.sh`](../scripts/dev-checks.sh) — the fast validation workflow (linking the file is fine; plain backtick text is also acceptable).
- [`docs/configuration.md`](configuration.md) — how the built executable consumes its configuration file.
````

Forbidden content in docs/build.md: any placeholder tokens (`TODO`, `TBD`, `<...>` outside the error-line tables where the script literally prints angle brackets — keep only those), byte counts, alternative build commands (no `cargo build` run directly, no `docker build`), no description of editing the script, no Windows/macOS instructions, no CI/installer/service wording.

### Step 2 — README edits (commit 2)

Exactly two edits to `C:\repo\rust-youtube-streamer\README.md`:

1. TOC: after the current line `- [Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm)` insert one line:
   ```
   - [Release Build (Linux Artifact)](#release-build-linux-artifact)
   ```
2. New section inserted immediately after the Build Checks section's `### Notes` closing bullet (the MCP exit-status note, currently the last bullet before `## AI Agents`) and immediately before the `## AI Agents` heading:

````markdown
## Release Build (Linux Artifact)

To produce a distributable release executable (not just validate the code), run the repository's
build script through the same Compose `rust` service used for the checks above:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

On success the command exits `0` and leaves a Linux release executable at `dist/rust-youtube-streamer-service`
(a gitignored directory). It compiles inside the pinned Linux Rust container, so the artifact is a Linux
executable — not a Windows `.exe`, and building it does not validate native Windows/Linux runtime behavior.
Full prerequisites, output path, rebuild/overwrite behavior, failure semantics and limitations:
[`docs/build.md`](docs/build.md).

This is a different job from `scripts/dev-checks.sh`: that script validates formatting/types/tests/lints
and produces no artifact, while the build command produces the distributable executable.
````

Rules for this step: the fenced command block must match the standard invocation byte-for-byte. Keep the section to ~3 short paragraphs exactly like the existing minimal section style. Do NOT renumber or touch any other heading, bullet, or the TOC's existing entries.

### Step 3 — `.agent/project-structure.md` edits (commit 3, with Step 4)

Four exact edit points (keep the file's existing bullet style `- name - purpose`):

1. `scripts/` entry (currently only `dev-checks.sh`) → extend to mention both scripts, e.g.:
   ```
   - scripts/ - developer utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to gitignored `logs/checks/`; `build-linux.sh` builds the release with the tracked lockfile in the same service and copies the final Linux executable to gitignored `dist/`
   ```
   Keep the existing `dev-checks.sh` sentence intact and append the `build-linux.sh` sentence.
2. `docs/` entry → append `build.md (Linux release build guide: prerequisites, standard invocation, output, failure semantics, limitations)` to the existing description, matching the current `configuration.md` mention style.
3. New bullet under `# Other folders`, appended after the `logs/` entry:
   ```
   - dist/ - gitignored release-build output; `build-linux.sh` writes only the final Linux executable `rust-youtube-streamer-service` here; the explicitly approved exception to the no-root-generated-files rule (contents never committed)
   ```
4. `Cargo.toml` root-file entry: `v0.2.0` → `v0.3.0` (single whole-version-string change in that line; touch nothing else in the entry).

No other lines in the file change.

### Step 4 — `.agent/project-info/context.md` edits (same commit 3)

Three edits, all inside this file's own living-log voice (terse factual bullets; no first person; no machine-specific paths; never placeholders):

1. **Current Work Focus** — replace the single paragraph at `## Current Work Focus` with an equivalent that states:
   - Phase 01 is complete (unchanged fact).
   - Phase 01.1 implementation is underway: Tasks 1–2 are done and verified (release script + artifact/ignore semantics), Task 3 (this change) documents the workflow in `docs/build.md`, README, project-structure.md and this file, and Task 4 (tracked completion report at `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md`) remains.
   - Still no Phase 02.
2. **New section `## Recent Changes (2026-10-09, Phase 01.1)`** — appended after the existing `## Recent Changes (2026-10-09, Phase 01 execution)` section (i.e., immediately before `## Known Deviations and Limitations (Phase 01)`), containing (all facts below are already verified; copy them in as plain stated facts; do not invent extra numbers):
   - Branch `feat/linux-release-build-artifact`; version bumped to `0.3.0` (`f9a6b0e`).
   - Task 1 — `scripts/build-linux.sh` (commit `92880c7`, review fix `6118322`): POSIX-sh release builder; asserts the documented Compose working directory and a non-empty `CARGO_TARGET_DIR`; runs `cargo build --release --locked`; verifies the built binary exists non-empty; creates `dist/` if needed; copies only `dist/rust-youtube-streamer-service` and re-verifies non-empty; prints start/result messages with the exact output path and size; never suppresses Cargo diagnostics; never touches files elsewhere under `dist/`.
   - Standard invocation verified exit 0; observed success messages (quote M1–M4 exactly as in §2, including the path and size 1715296); a cold full compile showed Cargo `Finished` in 33.04 s; no `target/` directory appears in the project root; Cargo intermediates remain in the named volume `rust-streamer-target`.
   - Artifact observed as mode `-rwxrwx---` `root:vboxsf` on the shared mount; `dist/` is ignored via the existing generic `dist/` `.gitignore` rule; nothing was unignored; the artifact is never staged or committed.
   - Failure paths verified: wrong working directory → exit 1 with the exact E1 stderr string from §2; empty `CARGO_TARGET_DIR` → exit 1 with the exact E2 stderr string from §2.
   - Task 2 — `.gitignore` reviewed: the existing `dist/` rule already covers the artifact; nothing was unignored; no build artifacts were staged or committed; any Windows/macOS cross-build or multi-platform matrix remains out of scope per the TODO constraints.
   - Task 3 (this change) — `docs/build.md` (purpose, prerequisites, standard command, expected output, overwrite/rebuild behavior, failure semantics table, what-the-build-does-not-prove, dev-checks comparison); README gained the `Release Build (Linux Artifact)` section + TOC entry linking `docs/build.md`; project-structure.md gained build-linux.sh/build.md/dist entries and the v0.3.0 refresh.
   - Task 3 verification is a docs-only self-check (TOC anchors, relative links); the final-tree `dev-checks.sh` run and the completion report belong to Task 4.
3. **Immediate Next Steps** — replace step 1 ("Execute `.agent/todos/20261009/20261009-todo-4.md` through the Critical Workflow.") with:
   - "Finish Phase 01.1 by recording the tracked completion report (TODO Task 4) at `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md`, then mark and rename the TODO."
   - Keep numbered items 2–4 byte-identical in content and order.

No other sections/bullets in context.md change. Do not touch `## Scope Decisions`, `## Notes`, the `brief.md` blockquote line 3, older Recent Changes sections, or the Phase 01 execution section.

### Step 5 — Commits (one per grouping as frozen in Frozen Decision 8)

1. `git add docs/build.md`; message `docs: add linux release build guide`.
2. `git add README.md`; message `docs: link build guide from readme`.
3. `git add .agent/project-structure.md .agent/project-info/context.md`; message `docs: record build workflow in structure and context`.

For each commit: run `git status` first, verify only intended files staged, verify no gitignored files (`dist/`, `logs/`, `.kilo/agent-manager.json`) are involved, commit, then `git status` to confirm clean. NEVER `git push` (push is restricted to workflow step 5; do not push). TODO `[DONE]`/checkbox flips and the 4.6 commit are out of scope here.

### Step 6 — Docs verification (no VM, no Cargo)

1. **TOC/anchor self-check** (per file): re-read `docs/build.md` and `README.md`; for every TOC link, derive the GitHub slug from the corresponding heading (`#` content lowercased, spaces → `-`, removing `/`, `(`, `)`, `.`, `,`, `:`) and confirm the link target matches exactly. The frozen headers were chosen precisely so slugs are the ones listed in Frozen Decision 2 and Step 2's `#release-build-linux-artifact`.
2. **Relative-link self-check**: from `README.md`, `docs/build.md` must exist (link `docs/build.md`); from `docs/build.md`, `../README.md`, `configuration.md` and `../scripts/dev-checks.sh` (if linked) must exist. Use `read`/`grep` on the repo, not the shell.
3. **Content self-checks**: `grep` for forbidden tokens (`TODO`, `TBD`, accidental `<...>` outside the error tables); confirm the standard invocation string appears byte-identically in `docs/build.md` and in the new README section (`run --rm rust sh scripts/build-linux.sh` — each file contains it once), and that the README Build Checks section is untouched (its `dev-checks.sh` invocation line still present, unmodified).
4. **Scope self-check**: `git diff HEAD~3 --stat` (or `git show --stat` per commit) — expect exactly 4 files: `docs/build.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` and nothing else.
5. Explicitly do NOT run `scripts/dev-checks.sh` here even though the tree changed — that final-tree run belongs to Task 4 and the TODO's acceptance row for dev-checks is validated at Task 4/4.5b, not in this doc task. Also do NOT run any `alpine-vm` MCP commands (Frozen Decision 1).
6. Return the list of commit hashes and the frozen-decision confirmation to the caller.

### Step 7 — Contingency: git commit permission-blocked

If local `git add/commit` is denied on the Windows host, use the Alpine VM fallback (already authorized): `docker exec`-into-the-shared-repo pattern via `git -c safe.directory='*' -c core.fileMode=false add <paths>` and `git -c safe.directory='*' -c core.fileMode=false commit --author="JEB <jebarrocal@gmail.com>" -m "<frozen message>"` inside the VM's shared repo location; after the VM-side commit, re-verify Windows-side (`git log --oneline -3` shows the frozen messages; `git status` clean). Escalate to caller before improvising anything more.

## 5. Do-Not List (hard blocked)

- No changes to `scripts/build-linux.sh`, `scripts/dev-checks.sh`, `src/**`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `docker-compose.yml`, `.gitignore`, `config/**`.
- No acceptance-criteria checkbox flips; no `[DONE]` marks; no TODO rename; no 4.6 commit in this sub-task.
- No `.agent/reports/**` creation (Task 4 owns the report; this task only references it internally).
- No OS-level file moves, no `.gitkeep` edits, no `.kilo/plans/` writes other than this plan file itself.
- No VM/docker commands; no dev-checks run; no network fetches of external content.
- Never stage anything matching `.gitignore` (in particular nothing under `dist/` or `logs/checks/`), never force-add, never push.

## 6. Acceptance Criteria Rows Covered by This Task (from the TODO)

Covered by this task:

- `docs/build.md` explains prerequisites, exact invocation, artifact path, overwrite behavior, and Linux-only artifact semantics (Step 1).
- README links to the build guide without unrelated rewrites (Step 2).
- `.agent/project-structure.md` and `.agent/project-info/context.md` reflect the final state (Steps 3–4, including the stale-version refresh as part of "final state").

Explicitly NOT covered here (deferred to Task 4 per its own TODO bullets): the standard `scripts/dev-checks.sh` run on the final tree, and the tracked completion report under `.agent/reports/`. This plan only notes their ownership; it must not execute them.

No out-of-scope feature rows are addressed (the Do-Not List in §5 enforces the TODO's Constraints and Out of Scope).

## 7. Task-3 TODO bullet → plan mapping (self-check)

| TODO Task 3 bullet | Covered by |
|---|---|
| Create `docs/build.md` with a concise table of contents and clear prerequisites, build command, expected output path, behavior/failure semantics, and artifact platform limitations | Step 1 (complete skeleton in §4 Step 1; section list in Frozen Decision 2) |
| Documented invocation uses the established Compose service, e.g. the given `docker compose ...` command | §4 Step 1 Build Command (byte-identical string) |
| Distinguish the release artifact workflow from `scripts/dev-checks.sh` | §4 Step 1 Dev Checks vs. Release Build; Step 2 one-liner |
| Explain how to rebuild/overwrite the generated artifact by rerunning the script and that the artifact is disposable/re-creatable | §4 Step 1 Overwrite and Rebuild Behavior |
| Explicitly state that a successful Linux container build does not prove native Windows/Linux runtime behavior, does not create a Windows `exe`, and does not mean the streaming pipeline is implemented | §4 Step 1 What This Proves and Does Not Prove |
| Link `docs/build.md` from the README with a short summary; keep unrelated README sections intact | Step 2 (two frozen edit points) |
| Update `.agent/project-structure.md` with the script and documentation paths and the ignored `dist/` output | Step 3 (4 frozen edits) |
| Update `.agent/project-info/context.md` with the actual implementation, exact build/check results, output path, platform caveat, and next phase | Step 4 (3 frozen edits, exact-fact bullets) |
| Constraints + acceptance criteria respected | §5 Do-Not List; §6 rows |

All nine TODO Task 3 bullets map to a plan step. Plan is complete.
