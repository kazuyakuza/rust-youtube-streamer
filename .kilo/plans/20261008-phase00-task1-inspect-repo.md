# Task 1 Implementation Plan — Inspect the Current Repository

- Date: 2026-10-08
- Source TODO: `.agent/todos/20261008/20261008-todo-2.md` — task heading `### 1. Inspect the Current Repository` (lines 24–30), Pattern C sub-items.
- Global plan: `.kilo/plans/20261008-phase00-repo-foundation-baseline.md` (Step table entries 3–8: Task 1 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6).
- Workflow: `.kilo/commands/critical-workflow.md` (this plan is the 4.1b deliverable).
- Target implementer: JUNIOR developer under 50% restriction (do not deviate; ask the caller when blocked).
- Front-end: NOT related (no 4.1a / 4.5a).

---

## 0. Source Task (verbatim from the TODO)

`### 1. Inspect the Current Repository`

- Inspect the current branch and working tree before changes.
- Review the existing root files, `src/` directories, configuration/documentation directories, `.gitignore`, and project structure map.
- Confirm whether any files or changes have appeared since the project-info status was last updated.
- Reuse existing directories and conventions. Do not create duplicate structures or remove project/agent configuration.
- Before running build commands, inspect which MCP tools are available in the local implementation-agent environment and identify the supported way to access the user's Alpine VM. Do not assume that an MCP or command name exists without checking the available tools.

Task 1 is an **inspection-only** task. It produces a findings report in the sub-agent completion report; it does NOT create Cargo/Docker files and does NOT update documentation.

---

## 1. Encoded Decisions (structure / architecture / scope — binding)

| # | Decision |
|---|---|
| D1 | Task 1 steps 4.2 (work phases A–E) are strictly read-only. No repo file may be created, modified, or deleted by the implementer in 4.2. |
| D2 | Any delta (unexpected file, missing file, dirty git state, doc/reality mismatch) is RECORDED ONLY in the completion report. The implementer must NOT fix, remove, add, or normalize anything to resolve a delta. The caller decides next actions. |
| D3 | `.agent/project-info/context.md` is NOT updated by Task 1. The Phase 00 context update is deferred to Task 5 (global plan). |
| D4 | Git handling: **nothing to commit in 4.2**. Inspection generates no changes. The ONLY sanctioned repo modification in Task 1 occurs in 4.6 (mark the task `[DONE]`, flip the two mapped acceptance-criteria checkboxes, commit that single TODO-file edit). If a delta is found in 4.2, the same 4.6 commit applies unchanged (the delta is report content, not a file change). |
| D5 | Git branch creation/switching is NOT allowed in Task 1 (restricted to global Step 1, already executed). Git push is NOT allowed in Task 1 (restricted to global Step 5). |
| D6 | MCP route verification (4.2 Phase C) uses exactly the `alpine-vm` MCP tools `alpine-vm_vm_status` (called FIRST, always) and `alpine-vm_vm_run_command`, limited to two harmless single commands: `docker ps` and `ls /rust-youtube-streamer`. Single commands only — no `&&`/`;` chains, no pipes. Allowlist prefixes per the tool description: `docker`, `sh`, `apk`, `ls`, `cat`, `ps`, `df`, `free`, `uname`, `pwd`, `whoami`. VM project mount: `/rust-youtube-streamer`. |
| D7 | NO cargo/docker build commands run in Task 1. The Rust build checks (`cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`) belong to Task 3 and require Cargo scaffolding from Task 2, which does not exist yet. |
| D8 | No unit tests are run in Task 1: no code was introduced and no test suite exists in the repository (no Cargo project yet). Codified as "not required". |
| D9 | No build step is executed in Task 1 (see D7). The docker ps / ls commands of Phase C are route-reconnaissance, not builds. |
| D10 | Step 4.4 (Documentation) is **NOT REQUIRED** for Task 1. All documentation updates (README, `.agent/project-structure.md`, `.agent/project-info/context.md`) are Phase 00 Task 5 per the global plan. The docs-specialist must not edit any file and must return a clear "not required" message. Exception: none. |
| D11 | Step 4.3 (code-reviewer + code-simplifier) review this plan against the completion report. Expected outcome is "not required"/"no fix plan" because no code files are produced by Task 1. A fix plan is only produced if the reviewers find the implementer actually deviated (e.g., edited files it must not have touched). Max 3 review cycles per workflow. |
| D12 | Step 4.6 marks task `### 1. Inspect the Current Repository` as `[DONE]` in the TODO file and flips exactly the two acceptance-criteria checkboxes mapped in section 8. No other checkbox is touched. |
| D13 | Inspection findings live in the Task 1 sub-agent completion report (section 6 template below). They are not written to any repo file. |

---

## 2. Preconditions

1. Global plan Step 1 (Git Feature Branch Setup) has already been executed by the implementer: branch `feat/phase00-cargo-baseline` exists and the pending changes known to the planner (`.agent/todos/.gitkeep` deletion, `20261008-todo-2.md` and `CHANGELOG.md` modifications) were committed there.
2. Therefore the EXPECTED git state when Task 1 starts: current branch = `feat/phase00-cargo-baseline`; working tree clean; branch based on `main`.
3. If the observed state differs, record it as a delta (section 5 Table D rows G1–G3). Do NOT switch branches, do NOT commit, do NOT clean anything.

---

## 3. Work Phases for Step 4.2 (Implementer)

Start every Task 1 response with the acknowledgment required by `.agent/project-info/instructions.md`: `[Project Info: Active]` (followed by the one-line context acknowledgment format defined there).

Tool discipline (host): use `read`/`glob`/`grep` for all file inspection; use the `bash` tool only for the three git commands in Phase A, one command per call, no chaining (no `&&`, no `;`). No PowerShell cmdlets, no PowerShell manipulation.

### Phase 0 — Required reading (read-only, in this order)

1. `.agent/todos/20261008/20261008-todo-2.md` — full file; focus lines 9–30 (repository context, Alpine VM MCP note, Task 1 sub-items) and lines 121–133 (Required Completion Report).
2. This plan (check between every phase).
3. `AGENTS.md` (root).
4. `.agent/WORKFLOWS.md` and `.kilo/commands/critical-workflow.md` — confirm the 4.2–4.6 rules being executed.
5. AGENTS.md-referenced project-info, all six files in `.agent/project-info/`: `brief.md`, `product.md`, `context.md`, `architecture.md`, `tech.md`, `instructions.md`.
6. `.agent/project-structure.md` — the project structure map used as Phase C/D baseline.
7. `.kilo/rules/gitignore-compliance.md` — needed for the 4.6 commit discipline.

### Phase A — Git branch and working tree inspection (host)

Execute exactly three `bash` tool calls, one command each, recording the output verbatim:

| Step | Tool | Command | Record |
|---|---|---|---|
| A1 | bash | `git status` | full output (branch, staged/unstaged, untracked) |
| A2 | bash | `git branch --show-current` | branch name |
| A3 | bash | `git log --oneline -5` | last 5 commit subjects |

Expected values (from global plan pre-analysis):
- A2 → `feat/phase00-cargo-baseline`.
- A1 → `On branch feat/phase00-cargo-baseline`, `nothing to commit, working tree clean` (or equivalent clean message; localized wording may differ — record it verbatim).
- A3 → top commit message resembling a step-1-style pending-changes commit (record exact subject).

Create rows G1 (branch), G2 (clean/dirty), G3 (HEAD commit) in the delta table (section 5).

Guard rule: if `git status` shows staged or unstaged changes at this point, that is a DELTA (row G2). Record the file list verbatim. Do NOT stage, commit, stash, or reset anything.

### Phase B — Repository tree inspection (host, read-only tools)

| Step | Tool | Exact call | Expected result |
|---|---|---|---|
| B1 | read | repo root directory `C:\repo\rust-youtube-streamer` | listing containing: `.agent/`, `.git/`, `.gitignore`, `.kilo/`, `.opencode/`, `AGENTS.md`, `CHANGELOG.md`, `README.md`, `config/`, `credentials/`, `docs/`, `fonts/`, `logs/`, `src/` (exact set recorded; hidden dirs may or may not be listed). |
| B2 | glob | pattern `src/**/*` | exactly six matches: `.gitkeep` inside `src/app/`, `src/config/`, `src/youtube/`, `src/chat/`, `src/renderer/`, `src/streaming/`. Any other match is a delta. |
| B3 | glob | pattern `Cargo.toml` | no matches (absence is the expected baseline: no Cargo project exists). |
| B4 | glob | pattern `Cargo.lock` | no matches. |
| B5 | glob | pattern `src/main.rs` | no matches. |
| B6 | glob | pattern `Dockerfile` | no matches. |
| B7 | glob | pattern `docker-compose.*` | no matches. |
| B8 | read | `config` directory | contains only `.gitkeep` (or its documented placeholder); no `config.json`, no `config.example.json` yet. |
| B9 | read | `credentials` directory | empty of committed files besides placeholders/readme per project-structure map (structure map says "no files committed here"). |
| B10 | read | `docs` directory | empty of files (placeholder only), per project-structure map. |
| B11 | read | `fonts` directory | placeholder only, per project-structure map. |
| B12 | read | `logs` directory | `.gitkeep` only, per project-structure map. |
| B13 | read | `README.md` | record "Current Status" wording (states repo not yet scaffolded; no Cargo.toml/lockfile; src/ handles). |
| B14 | read | `.gitignore` | record section list; confirmed content has: OS files, temp, logs (+`!logs/.gitkeep`), `.env*`, IDE, generic `build/`/`dist/`, secrets, `.kilo/agent-manager.json`; NO Rust entries. |
| B15 | grep | pattern `(target|cargo|config\.json|credentials/.+|fonts/(?!README|\.gitkeep))` scan of `.gitignore` may be over-strict — use simple pattern `[Tt]arget|cargo|Cargo|config\.json` on file `.gitignore` | expected: no matches → proves `target/` and Rust-lock handling not yet added (that is Task 4's work). |
| B16 | read | `.agent/project-structure.md` | record all ten described entries (six `src/` modules + four "Other folders" groups). |
| B17 | read | `.agent/project-info/context.md` | record "Current Work Focus" and "Recent Changes (2026-10-08)" items as the documentation baseline for the delta analysis. |
| B18 | read | `CHANGELOG.md` | confirm exists (global plan pre-analysis lists it as modified); record last entry subject only. |
| B19 | read | `.agent/` and `.kilo/` and `.opencode/` top-level directories | confirm agent configuration intact (`.agent/todos/`, `.agent/project-info/`, `.agent/RULES.md` if present, `.kilo/plans/`, `.kilo/rules/`, `.kilo/commands/`, `opencode.json`). Do not open every file; confirm presence only. |

Notes:
- B15 is a verification aid; if the grep pattern yields matches, copy the matching lines verbatim into the delta table row I3.
- The six `src/` module directories are the "existing directories and conventions" the phase must reuse later (Task 2/3). Their presence must be confirmed, not altered.

### Phase C — MCP tools and Alpine VM route verification

Rules (binding, from D6 and the TODO sub-item "do not assume that an MCP or command name exists without checking the available tools"):

- C1 — Before ANY `alpine-vm_vm_run_command` call, verify the actual tool names available in this environment. Required tools: `alpine-vm_vm_status` and `alpine-vm_vm_run_command` (server: `alpine-vm`). If these exact tools are NOT available in the environment, STOP, do not attempt other MCP names, and return the question to the caller.
- C2 — Call `alpine-vm_vm_status`. Record the reported state verbatim (running / SSH reachable / error text).
- C3 — If `alpine-vm_vm_status` reports the VM running and reachable: call `alpine-vm_vm_run_command` with the single command `docker ps` (no chaining, no extra flags). Record the full output (docker daemon answers; list of containers; exit state of the call).
- C4 — Call `alpine-vm_vm_run_command` with the single command `ls /rust-youtube-streamer`. Record the full listing.
- C5 — Compare the C4 listing against the B1 host root listing: the mount mirrors the shared project directory. Expected: same top-level entries (`src/`, `config/`, `credentials/`, `docs/`, `fonts/`, `logs/`, `.agent/`, `.kilo/`, `.opencode/`, `README.md`, `.gitignore`, `CHANGELOG.md`; hidden dot-files are typically not shown by plain `ls` — expected difference, not a delta).
- C6 — If `alpine-vm_vm_status` reports the VM NOT running/reachable: record the exact message plus rows C1–C4 as "not executed", do NOT call `alpine-vm_vm_run_command` at all, do NOT attempt `docker` on the host, and report the unavailability to the caller in the completion report under "checks not run and why". This itself satisfies nothing further in Task 1 — the caller decides how to proceed.
- C7 — If `docker ps` fails (e.g., daemon down, permission error): record command + exact error, stop further VM commands, and report. Do NOT retry, do NOT repair.

Sequence enforcement: C2 STRICTLY precedes C3/C4. No build or cargo command may be issued in Task 1 (D6/D7).

### Phase D — Delta analysis (documentation vs. reality)

Produce a single comparison table with rows from ALL phases. Format per row:

`<RowID> | <what> | <documented expectation (source)> | <observed> | DELTA: yes/no`

Mandatory rows and their expectation sources:

| RowID | What | Expectation source | Expected |
|---|---|---|---|
| G1 | current branch | global plan Step 1 / pre-analysis | `feat/phase00-cargo-baseline` |
| G2 | working tree state | global plan pre-analysis | clean |
| G3 | HEAD commit | global plan Step 1 | pending-changes commit present / nothing untracked pending |
| S1 | `src/app/` | `.agent/project-structure.md` | exists, `.gitkeep` only |
| S2–S5 | `src/config/`, `src/youtube/`, `src/chat/`, `src/renderer/`, `src/streaming/` | `.agent/project-structure.md` | exists, `.gitkeep` only |
| S6 | no `src/main.rs` | `context.md` Recent Changes | absent |
| D1 | `config/` | `.agent/project-structure.md` | placeholder only |
| D2 | `credentials/` | `.agent/project-structure.md` | no committed files |
| D3 | `docs/` | `.agent/project-structure.md` | empty/placeholder |
| D4 | `fonts/` | `.agent/project-structure.md` | placeholder only |
| D5 | `logs/` | `.agent/project-structure.md` + `.gitignore` | `.gitkeep` present, other contents ignored |
| O1–O3 | `.agent/`, `.kilo/`, `.opencode/` | `.agent/project-structure.md` | present, intact |
| F1 | `README.md` | `context.md` recent change | replaced with initial MVP info (record "Current Status" text) |
| F2 | `.gitignore` | `context.md` recent change | log rule present; no Rust entries (row I3 via grep) |
| F3 | `CHANGELOG.md` | global plan pre-analysis | exists |
| I1 | `Cargo.toml` | `context.md` | absent (B3) |
| I2 | `Cargo.lock` | `context.md` | absent (B4) |
| I3 | Rust entries in `.gitignore` | pattern baseline for Task 4 | absent (`target/` not yet ignored) |
| C1 | MCP tool availability | TODO Task 1 item 5 | `alpine-vm_vm_status` + `alpine-vm_vm_run_command` present |
| C2 | VM status | TODO Alpine VM MCP section | running, SSH reachable |
| C3 | `docker ps` | TODO "Use Docker to run rust/cargo cmds." | docker engine answers |
| C4 | VM mount listing | TODO "project folder mounted in `/rust-youtube-streamer`" | matches B1 (excluding hidden dot-files) |

Interpretation rules (no junior judgment involved):
- Row marked `DELTA: no` → documentation and reality agree; nothing to do.
- Row marked `DELTA: yes` → this answers TODO Task 1 sub-item "Confirm whether any files or changes have appeared since the project-info status was last updated". Copy the row into the completion report section "Deltas". NO file operations of any kind are performed on any delta (D2).
- Every delta row becomes report content; `context.md` itself is NOT edited by Task 1 (D3).

---

## 4. Step 4.3 — Code Review & Simplification (applicability)

Assign code-reviewer and code-simplifier per workflow. Codified outcome rules:

- Task 1 produces no code files and no configuration files. Code-reviewer verifies the implementer followed the plan (phases executed, no forbidden edits, no forbidden commands) using the completion report. If the report shows zero deviations → return message: "not required — inspection-only task, implemented as planned".
- Code-simplifier: there are no source changes to simplify. Expected return: "not required — no source files produced by Task 1".
- A fix/simplification plan file (`.kilo/plans/<YYYYMMDD>-fix-name.md`) is produced ONLY if a reviewer identifies an actual deviation requiring a file correction (e.g., implementer wrongly edited a repo file in 4.2). Then the Planner assigns it to the implementer per workflow. Max 3 cycles.

---

## 5. Tests / Build applicability (codified)

- Unit tests: NOT REQUIRED. No test suite exists (no Cargo project), and Task 1 adds no executable code. Nothing to run.
- Build checks (`cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo clippy`): NOT RUN in Task 1. They belongs to Task 3 after Task 2 scaffolds Cargo. Running any cargo command in Task 1 would be an out-of-order violation.
- The two reconnaissance commands of Phase C (`docker ps`, `ls /rust-youtube-streamer`) are the terminal extent of remote command execution permitted by this plan.

---

## 6. Completion report content (Task 1 deliverable)

The implementer must return a report with exactly these sections (mirrors the TODO "Required Completion Report" fields):

1. **Files created or changed**: expected value `none` (plus the sanctioned 4.6 TODO-file edit only).
2. **Exact commands executed and results**: every bash/git command of Phase A, every glob/grep/read verification skipped or executed, and every MCP call of Phase C (tool name, command string, result/exit status verbatim).
3. **Alpine VM details**: tool names verified, VM status result, `docker ps` result summary, mount listing summary.
4. **Scope of validation**: state precisely that Task 1 validated documentation/reality agreement and MCP route readiness ONLY; no code was compiled, no platform claim is made (do not infer any Linux/Windows validation).
5. **Checks not run and why**: e.g., no cargo commands (Task 3 scope; `Cargo.toml` does not exist yet), no build, no unit tests, no docs updates (Task 5 scope). If VM unreachable, record here verbatim.
6. **Assumptions or blockers**: list every delta row from Phase D that needs caller attention; an empty list is an explicit statement.
7. **Confirmation**: no out-of-scope feature, file, or behaviour was introduced; directories/conventions were reviewed and reused as documented; no project/agent configuration was removed.

---

## 7. Step 4.4 — Documentation: codified "not required"

The docs-specialist must NOT edit any file for Task 1 and must return the message:

`Task 1 (repository inspection) required no documentation changes: documentation updates for the resulting repository state (README, project-structure, context.md) are Phase 00 Task 5 per the global plan. Task 1 findings live in the implementer completion report only.`

No exceptions. Any README/structure/context edit during Task 1 is a plan deviation.

---

## 8. Step 4.6 — Task Completion (exact edits + single commit)

After 4.5b confirms adherence (based on the completion report), the implementer runs 4.6. Permitted edits are ONLY the following two changes inside `.agent/todos/20261008/20261008-todo-2.md` (overwrite-todo-file-prevention rule honored: every other byte of the file stays untouched):

Edit 1 — mark the task heading done (line 24 of the TODO):

```
### 1. Inspect the Current Repository
```
becomes
```
### 1. Inspect the Current Repository [DONE]
```

Edit 2 — flip the two acceptance criteria that Task 1 fully satisfies (lines 105–106 of the TODO). Old:

```
- [ ] The existing repository and agent/project documentation have been inspected before changes.
- [ ] The available MCP tools and documented route to the shared Alpine VM have been inspected before attempting remote commands.
```
new:
```
- [x] The existing repository and agent/project documentation have been inspected before changes.
- [x] The available MCP tools and documented route to the shared Alpine VM have been inspected before attempting remote commands.
```

Guard: flip these two checkboxes only if Phases A–D completed and the report documents them. If Phase C failed (VM unreachable), go to section 9 (STOP + ask) instead of applying 4.6.

Git handling (D4/gitignore-compliance):

1. `git status` — single call; confirm only `.agent/todos/20261008/20261008-todo-2.md` is modified.
2. `git diff .agent/todos/20261008/20261008-todo-2.md` — verify only the two intended edit locations changed.
3. Read `.gitignore` (already known content) and re-check `git status` output: confirm no `.gitignore`-matching file (e.g., `.kilo/agent-manager.json`, logs, build artifacts) is staged. Stage only the TODO file:
   - `git add .agent/todos/20261008/20261008-todo-2.md` (single call)
4. Commit (single call): `git commit -m "docs(todos): mark Phase 00 task 1 (repository inspection) complete"`
5. No branch operations (D5), no push (restricted to Step 5 of the global plan), no merge.

---

## 9. Failure & ambiguity handling (binding)

The implementer must STOP and report to the caller instead of improvising whenever:
- the `alpine-vm` MCP tools are not available in its environment (Phase C guard C1);
- `alpine-vm_vm_status` reports the VM down/unreachable (Phase C guard C6);
- `docker ps` or `ls /rust-youtube-streamer` errors (Phase C guard C7);
- any delta is found (records only; caller decides — D2);
- the git state contradicts section 2 preconditions beyond a simple recordable delta;
- the `bash` tool reports "unknown command" — retry that same command up to 2 times, then STOP and question the caller.

No assumptions, no inventions, no auto-fixes, ever.

---

## 10. Explicitly out of scope for Task 1 (never do)

1. Creating `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `Dockerfile`, or any Compose file (Tasks 2–3).
2. Editing `.gitignore` (Task 4).
3. Editing `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` (Task 5).
4. Running any `cargo` command or any Rust container (Task 3; also impossible — no project yet).
5. Any code, comment, or test authoring.
6. Installing anything, globally or in-project.
7. Committing anything other than the single 4.6 TODO-file commit; any branch creation/switch/merge; any push.
8. Installing CI, services, or FFmpeg (out of scope for the whole phase).
9. Deleting, moving, renaming, or "cleaning" any existing file or folder, including deltas.

---

## 11. Completion criteria mapping (Task 1 → TODO acceptance criteria)

| TODO acceptance criterion | Satisfied by | Verification |
|---|---|---|
| "The existing repository and agent/project documentation have been inspected before changes." (TODO line 105) | Phases 0, A, B, D | Findings + delta table recorded in the completion report; no repo changes made before the inspection (D1) |
| "The available MCP tools and documented route to the shared Alpine VM have been inspected before attempting remote commands." (TODO line 106) | Phase C (with D6 sequence: `alpine-vm_vm_status` first, then the two harmless commands) | MCP call records + C5 mount comparison in the report; no cargo/docker build command attempted in Task 1 (D7) |
| Task sub-item: "Confirm whether any files or changes have appeared since the project-info status was last updated." | Phase D delta analysis vs `.agent/project-info/context.md` | "Deltas" report section (empty list counts as an explicit "none found") |
| Task sub-item: reuse existing directories/conventions; no duplicates; agent config intact | D1/D2 scope rules + Phase B19 | Report confirmation (field 7) |
| 4.6 marks the mapped checkboxes `[x]` (lines 105–106) and heading `[DONE]` | Section 8 edits + single commit `docs(todos): mark Phase 00 task 1 (repository inspection) complete` | `git diff` check in 4.6 shows exactly those two edit locations |

Verification for 4.5b: the architector re-runs this mapping against the implementer's completion report and confirms both acceptance criteria and all seven report fields are present and factually worded.

---

## 12. Plan self-check against the source task

- Inspect branch/working tree → Phase A. ✔
- Review root files, `src/` dirs, config/docs dirs, `.gitignore`, structure map → Phase B (B1–B19). ✔
- Confirm changes since project-info status updated → Phase D delta table with sourced expectations. ✔
- Reuse conventions; don't remove agent config → D1/D10/out-of-scope list; B19 verification. ✔
- Inspect MCP tools/route before any build commands → Phase C, D6/D7 sequencing rules. ✔
- Deliverable prevention: no repo file edits other than sanctioned 4.6 edits; findings in the completion report (D1–D4, D13). ✔
- Report mapping to the TODO "Required Completion Report" → section 6. ✔

End of plan.
