# Task 2 Plan (4.1b) — Documentation and Project Metadata for `scripts/dev-checks.sh`

- Source TODO: `.agent/todos/20261009/20261009-todo-2.md` → "### 2. Update Documentation and Project Metadata" (lines 42–48)
- Parent global plan: `.kilo/plans/20261009-dev-checks-script.md`
- Workflow: `.kilo/commands/critical-workflow.md` — this plan implements Task 2 step 4.2 (implementer). **4.3 code/docs review is REQUIRED after implementation; 4.6 task completion happens later (see "Workflow placement").**
- Branch: `feat/dev-checks-script` (current HEAD at planning time: `19567d0`)
- Date: 2026-10-09
- Non-front-end task. No container runs are required in this task (evidence already captured in Task 1).

---

## 0. Verified facts available for the docs (from Task 1, commits `12aaf37` / `e6b462f` — do NOT re-derive, do NOT invent others)

1. Script `scripts/dev-checks.sh`: 74 lines, POSIX sh (`#!/bin/sh`, dash-safe); helpers `emit`, `run_check`, `print_header`, `print_summary` + 4 wrappers (`run_fmt_check`, `run_cargo_check`, `run_cargo_test`, `run_cargo_clippy`).
2. Runs in order: `cargo fmt --check` → `cargo check --locked` → `cargo test --locked` → `cargo clippy --locked -- -D warnings`, each with merged output (`2>&1`).
3. Output shape: `== <name> == start` header per check; final per-check line `PASS|FAIL <name> exit=<code> (<duration>s)`; summary block `--- Summary ---` + per-check lines + overall `ALL CHECKS PASSED` or `N CHECK(S) FAILED` + log path.
4. Log file: `logs/checks/<UTC timestamp>.log` (e.g. `logs/checks/20261009T152555Z.log`); log path printed; `logs/*` is git-ignored (verified: `.gitignore:18:logs/*`).
5. Exit code: `0` all pass, `1` any fail. No source mutation, no fmt auto-apply.
6. Failure-path hook: `FORCE_FAIL=<name>` env var (validated against `fmt-check|check|test|clippy`) replaces exactly that check's command with a synthetic failing command exiting `7`; other checks still run.
7. Verified invocations/results (single MCP commands, `vm_status` first):
   - Standard: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit `0`, 4× PASS, summary `ALL CHECKS PASSED`, log `logs/checks/20261009T152555Z.log`.
   - Hook: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh` → exit `1`, `FAIL test exit=7`, 3× PASS, summary `1 CHECK(S) FAILED`, log `logs/checks/20261009T152623Z.log`.

**Rule: only the facts above may be written into the docs. Do NOT add parenthetical "verified/citation" claims of evidence not in this list; do NOT mention CI, MPS/PTY, or MCP contract details in the docs (README/context stay implementation-facing).**

---

## 1. Scope guard (HARD boundaries)

Files allowed to change (exactly three):

- `README.md`
- `.agent/project-structure.md`
- `.agent/project-info/context.md`

Files/actions explicitly FORBIDDEN in this task:

- `.agent/project-info/brief.md`, `architecture.md`, `tech.md`, `product.md` (touching requires a strict factual discrepancy — none is known; if one is spotted: STOP and report to caller, do not fix on your own).
- `.gitignore`, `Dockerfile`, `docker-compose.yml`, `Cargo.toml`, `Cargo.lock`, anything under `src/**`.
- `.agent/todos/20261009/20261009-todo-2.md` — do NOT tick checkboxes, do NOT rename. (Criterion `[ ] README describes script-first usage...` on line 66 is ticked later, during workflow Step 5 / 4.6, not here.)
- `.kilo/plans/**` — the plan file `.kilo/plans/20261009-phase00-1-task2-docs-metadata.md` stays UNTRACKED. Do not stage or commit it in either Task 2 commit (plan tracking is handled outside Task 2's two commits, like the earlier `chore: track ...` commits).
- No push to `origin` (merge/push happens later in workflow Step 5, not in this task). Exactly two local commits total.
- No CI wiring, no container runs, no re-running of checks.

Allowed tools: `read`/`grep` to inspect, `edit` for the two `.agent` files and README, `bash` (single commands only, no chains) for git. No PowerShell unless nothing else works.

---

## 2. Implementation steps

### Step S1 — Preflight (bash, one command at a time)

1. `git status` → expect `On branch feat/dev-checks-script`, clean tree (only possibly the untracked new plan file from `.kilo/plans/`).
2. `git log --oneline -3` → expect `19567d0` / `e6b462f` / `12aaf37` on top.
3. Read `.gitignore` once (required by the gitignore-compliance rule). Confirm rule `logs/*`.

Abort and report if branch or state differs.

### Step S2 — `README.md` edit (single edit, both insert + relabel)

In the `## Build Checks (Docker via Alpine VM)` section. NOTHING outside this section may change (Prerequisites subsection, Notes subsection, platform wording, and all other sections stay byte-identical).

Use the `edit` tool with:

- `old_string` (unique in README, verified):
  `Then run each check via the \`rust\` Compose service:`
- `new_string` (exact, preserving existing README fence style — plain ``` blocks, no language tag):

  ````text
  Standard way — one command runs every check via the repository script `scripts/dev-checks.sh`
  (POSIX sh; build the image once first, as shown above):

  ```
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
  ```

  The script runs `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` and
  `cargo clippy --locked -- -D warnings` in that order and prints a per-check
  `PASS|FAIL <name> exit=<code> (<duration>s)` line plus a final summary (`ALL CHECKS PASSED` or
  `N CHECK(S) FAILED`); its exit code is `0` when all checks pass and `1` when any check fails.
  Output is also written to a UTC-timestamped log under `logs/checks/` (a gitignored directory)
  whose path is printed. The script never modifies source files and never applies `cargo fmt`
  fixes.

  Failure-path validation hook: `FORCE_FAIL` set to `fmt-check`, `check`, `test` or `clippy` makes
  exactly that check fail with a synthetic exit code 7 while the other checks still run. Leave
  `FORCE_FAIL` unset in normal use. Example:

  ```
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh
  ```

  Fallback/reference — run the individual checks manually via the `rust` Compose service:
  ````

  The final line of `new_string` replaces the original `Then run each check via ...` sentence (adding the FALLBACK/REFERENCE label); everything above it is inserted BETWEEN the "Build the image once:" fenced block and the manual per-check fenced block. Line wrapping above is display-wrap only — content must use real newline characters, hard-wrapped nowhere else; the trailing sentence "Run checks sequentially when a later command needs output from an earlier one." remains untouched after the manual block.

Resulting `### Commands` order after the edit: build-image block → standard-way paragraph → script invocation block → behavior paragraph → FORCE_FAIL paragraph → copy of the behavior table: script fail-hook block → fallback label line → manual four-command block → sequential note. Verify with `git diff README.md`.

### Step S3 — commit README (bash, single commands)

1. `git add README.md`
2. `git status` → staged must be EXACTLY `README.md` and nothing else (no `scripts/`, no logs — verify none of the staged entries match `.gitignore` patterns; `logs/` under any path must show as untracked-ignored or ignored-untouched).
3. If clean: `git commit -m "docs: document dev-checks script usage"` (exact message, no Co-Authored-By, no scope suffix).

### Step S4 — `.agent/project-structure.md` edit

Use `edit` with:

- `old_string` (verified unique):
  `- fonts/ - font files used by the renderer`
- `new_string` (insert the `scripts/` entry directly above `logs/`, keeping alphabetical order of the "# Other folders" list):

  ````text
  - scripts/ - developer check utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory
  - fonts/ - font files used by the renderer
  ````

  (i.e. the new `scripts/` line is inserted BEFORE `- fonts/ ...`; the `logs/` line itself stays untouched — it already states contents are ignored).

### Step S5 — `.agent/project-info/context.md` edit

Use `edit` with:

- `old_string` (verified unique):

  ````text
  * `src/main.rs` gained a 2-line crate-level `//!` overview comment (comment-only); `cargo fmt --check` re-validated with exit 0 on the final tree.

  ## Immediate Next Steps
  ````

- `new_string` — same shown bullets PLUS a new dated section inserted between them:

  ````text
  * `src/main.rs` gained a 2-line crate-level `//!` overview comment (comment-only); `cargo fmt --check` re-validated with exit 0 on the final tree.

  ## Recent Changes (2026-10-09)

  * Phase 00.1 (reproducible dev checks, branch `feat/dev-checks-script`): added `scripts/dev-checks.sh` (commit `12aaf37`) — POSIX-sh runner that executes `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` and `cargo clippy --locked -- -D warnings` inside the Compose `rust` service; per-check `PASS|FAIL <name> exit=<code> (<duration>s)` lines with merged output, `--- Summary ---` block printing `ALL CHECKS PASSED`/`N CHECK(S) FAILED`, UTC-timestamped log under gitignored `logs/checks/`, aggregate exit code (0 when all pass, 1 when any fails), no source mutation, no fmt auto-apply; includes a `FORCE_FAIL=<check>` hook (synthetic exit 7) for failure-path validation.
  * First full green run via the standard invocation `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`: exit 0 with four PASS lines (`fmt-check`, `check`, `test`, `clippy`), summary `ALL CHECKS PASSED`, log `logs/checks/20261009T152555Z.log`.
  * Failure-path validation with `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh`: exit 1 with `FAIL test exit=7` plus three PASS lines, summary `1 CHECK(S) FAILED`, log `logs/checks/20261009T152623Z.log`; no source files touched in either run.
  * README 'Build Checks (Docker via Alpine VM)' updated: script invocation documented as the standard way; manual per-check commands kept as clearly-labeled fallback/reference; commit `docs: document dev-checks script usage`.
  * `project-structure.md` updated with the `scripts/` folder entry and the check-log note for gitignored `logs/checks/`; this context file updated; commit `docs: record dev-checks script in structure and context`.

  ## Immediate Next Steps
  ````

Boundaries: do NOT touch `## Current Work Focus`, `## Immediate Next Steps` (stays phase-level exactly as-is — this satisfies "Next steps kept at phase level"), `## Scope Decisions`, `## Notes`. Do NOT mark any checkbox anywhere. Do NOT append post-hoc commit hashes for the two new docs commits (subjects quoted above are the only allowed reference; adding hashes later would force a third commit).

### Step S6 — commit structure + context (bash, single commands)

1. `git add .agent/project-structure.md` — wait for completion, then:
2. `git add .agent/project-info/context.md` (run these as TWO separate commands, never chained)
3. `git status` → staged exactly those two files; nothing else.
4. If clean: `git commit -m "docs: record dev-checks script in structure and context"` (exact message).

### Step S7 — Post verification (bash, single commands; no push)

1. `git log --oneline -4` → top two commits, newest first:
   `docs: record dev-checks script in structure and context`, then `docs: document dev-checks script usage`.
2. `git show --stat HEAD~1` → touches ONLY `README.md`.
3. `git show --stat HEAD` → touches EXACTLY `.agent/project-structure.md` and `.agent/project-info/context.md`.
4. `git status` → clean tree (optionally only the untracked plan file remains; that is acceptable and must be left alone).
5. `git check-ignore -v logs/checks/20261009T152555Z.log` → must return `.gitignore:18:logs/*` (log files gitignored and never staged).
6. Content greps (grep tool): README contains `scripts/dev-checks.sh` (twice: standard block + fail-hook block), the line `Fallback/reference — run the individual checks manually via the \`rust\` Compose service:`; structure file contains `scripts/ - developer check utilities`; context file contains `## Recent Changes (2026-10-09)`, `12aaf37`, `20261009T152555Z.log`, `20261009T152623Z.log`, `FAIL test exit=7`.
7. Anchor link integrity: confirm README still contains one `## Build Checks (Docker via Alpine VM)` H2 heading, so the `#build-checks-docker-via-alpine-vm` anchor from Current Status remains valid.

---

## 3. Encoded prohibitions (repetition, they are binding)

- No scope creep in README: do NOT rewrite the Build Checks section, no new top-level sections, no CI mention, no Prerequisites/Notes changes, no platform-wording changes.
- No invented facts: every statement in the docs must trace to Section 0 of this plan; no parenthetical evidence claims beyond those facts.
- No push, no merge, no third commit, no TODO-file edits, no `-DONE` rename, no fmt/brief/architecture/tech modifications.

## 4. Workflow placement and follow-ups

- After this plan's S7, context.md's own closing rule from `.agent/project-info/instructions.md` ("Critical Closing Step") is satisfied by Step S5.
- 4.3 Code Review & Simplification is REQUIRED (docs-reviewed pass: verify the docs contain no inconsistencies with the actual script behavior, then have fixes applied by the implementer if the reviewer flags any).
- 4.6 Task Completion and workflow Step 5 (TODO `-DONE` rename incl. ticking line 66, merge, push to origin only) happen AFTER 4.3 and are executed by other steps, not by this plan.

## 5. Required completion report for the implementer (signal)

1. The exact two commits (`git log --oneline -2`). 2. `git show --stat HEAD ~/HEAD~1` file lists. 3. `git status` result (clean except untracked plan file). 4. Result of `git check-ignore -v logs/checks/20261009T152555Z.log`. 5. The three content-edit verification results from S7.6–7.7. 6. Explicit statement: brief/architecture/tech, .gitignore, Dockerfile, docker-compose.yml, Cargo files and src/** untouched, no container runs, no push. 7. Any blockers. Step 4.3 follow-up (docs review) must be scheduled before 4.6.
