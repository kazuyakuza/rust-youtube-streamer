# Implementation Plan — Phase 01.1, Task 2: Define Artifact Semantics and Safe Ignore Rules

- Critical Workflow step: 4.1b (Implementation Plan) for TODO Task 2
- Source TODO: `.agent/todos/20261009/20261009-todo-4.md` → `### 2. Define Artifact Semantics and Safe Ignore Rules`
- Global plan: `.kilo/plans/20261009-phase-01-1-build-artifact.md` (Task 2 pre-analysis, section "Per-Task Pre-Analysis")
- Plan date: 2026-10-09 — repo: `C:\repo\rust-youtube-streamer`, branch `feat/linux-release-build-artifact`, HEAD `8448f0a`, tree clean
- Front-end related: **NO** — 4.1a/4.5a skipped
- Plan consumer: implementer sub-agent (JUNIOR, under 50% restriction), executing sub-step 4.2 (and the Task-2-scoped parts of 4.3–4.6), exactly ONE cycle, no other TODO tasks.

---

## 0. Verified repository facts (observed 2026-10-09, HEAD `8448f0a`)

These are confirmed against the live tree on this host; the implementer must treat them as given:

- Branch `feat/linux-release-build-artifact`; HEAD `8448f0a` (`docs: mark phase 01.1 task 1 complete`); `git status --porcelain` empty. `main` remains at `17c2177` (push happens only at workflow step 5).
- `.gitignore` contains, under the comment header `# Build artifacts (generic)` (line 32): line 33 `build/`, line 34 `dist/`. **`dist/` is present and correct — expected outcome of this task is NO change to `.gitignore`.**
- The artifact exists at `dist/rust-youtube-streamer-service` (repo-root `dist/`), created by Task 1's build run (1 715 296 bytes, per Task 1's recorded run evidence). It is untracked and uncommitted.
- Verified on this host (planning-time pre-run, to be re-executed by the implementer in §3):
  - `git check-ignore -v dist/rust-youtube-streamer-service` → stdout `.gitignore:34:dist/` + `dist/rust-youtube-streamer-service`, exit 0.
  - `git ls-files dist/` → empty stdout, exit 0 (nothing tracked under `dist/`).
  - `git status --porcelain --ignored=matching -- dist/` → single line `!! dist/` (ignored, nothing else).
- `scripts/build-linux.sh` (72 lines, mode `100644`, commit `92880c7` + S1 fix `6118322`) already documents the freeze/ignore contract in its header: binary goes to "the gitignored root-level dist/ directory; intermediate Cargo output stays in the named Docker volume". Task 2 must NOT touch this script (Task 1's scope; header does not duplicate Task 2's semantics bullets).
- `.agent/reports/` does not exist yet — it is created by Task 4 only. Creating it in Task 2 is forbidden.
- `.agent/project-structure.md` does not list `dist/` yet — Task 3 owns that update. Do not touch it in Task 2.

## 1. Residual ambiguities — resolved and FROZEN

Each row resolves a point the TODO/caller leaves open. The implementer gets NO choice; these are decisions:

| # | Question | FROZEN decision | Rationale |
| --- | --- | --- | --- |
| R1 | Do the git verifications run on the host or via the Alpine VM MCP? | **Host-only. All Task 2 verification commands are `git` commands executed on the Windows host (repo root). No VM/MCP call, no Docker, no Cargo, no Compose. `alpine-vm_*` tools are NOT called in Task 2 at all.** | Staging/ignore status is a property of the host repository, not of the VM. The artifact already exists (Task 1); no build or container work belongs to Task 2. Host-only was pre-validated during planning (§0 commands all ran on the host with expected results). |
| R2 | WHERE is the verification evidence durably recorded for Task 4? | **Task 2's 4.5b adherence confirmation file: `.kilo/plans/20261009-phase-01-1-task2-adherence.md`** — the same durable, tracked pattern Task 1 established (commit `53efb78`). It contains a verbatim evidence section (each §3 run: exact command, exit code, exact stdout line(s)), committed as `docs: record task 2 adherence confirmation`. Task 4 consumes that file verbatim for its report row "confirmation that `dist/` is gitignored and the artifact is not tracked". Until 4.5b runs, the implementer's returned summary carries the record (Task 1 precedent). | Established project convention; a tracked `.kilo/plans/` file survives the session and is available to Task 4's sub-agent without transcript dependence. The Task 2 plan file itself is NOT appended to after its own planning commit (plan files are frozen deliverables; the implementer must not modify them). `.agent/reports/` is Task 4's file and must not be created here. |
| R3 | Expected `.gitignore` outcome? | **NO change to `.gitignore`.** Verification-only. `dist/` exists at line 34 under the `# Build artifacts (generic)` section and is sufficient. | Confirmed live: `git check-ignore -v` resolves the artifact to `.gitignore:34:dist/`. Any edit would be pointless churn; TODO wording is "adjust only if needed". |
| R4 | Contingency if `.gitignore` verification unexpectedly fails (V1 or V2 non-zero / no `dist/` rule match)? | **Contingency path only (never expected). Steps: (1) STOP normal flow; read `.gitignore` live. (2) If the failure is explainable ONLY by the absence/loss of the `dist/` rule inside the `# Build artifacts (generic)` section: perform a single-line APPEND of `dist/` into that section (after `build/`, keeping the `# Build artifacts (generic)` comment header intact above it). Nothing else in the file may change. (3) Re-run the FULL §3 matrix and commit `.gitignore` alone as `chore: restore dist/ ignore rule`. (4) If the failure is NOT fixed by that single-line append, or would require editing/removing any other rule: STOP and report to the caller — do not redesign, reorder, or "clean up" rules.** | TODO: "adjust the rule only if needed, without unignoring any other build artifacts". Append-only, section-placement-preserving, no `!` negation entries (adding any `!dist/...` unignore, `!**/dist/`, or touching other sections is forbidden). The `# Rust build output` section (`/target/`) and every other rule stay untouched. |
| R5 | Precise guard against staging a generated binary? | **(a) Never stage with `git add -A`, `git add .`, `git commit -a`, or `git add *` — only explicit per-path `git add <file>`. (b) Before EVERY commit in this task (plan file, contingency, fixes, adherence, `[DONE]`), run and record `git status --porcelain` and `git diff --cached --name-only`; both must contain no `dist/` entry. (c) If any generated binary under `dist/` appears staged: STOP immediately and report to the caller. Do NOT run `git reset --hard`, do NOT delete or move the artifact, do NOT amend — leave the index as-is for caller intervention.** | Gitignore-Compliance Rule + workflow error handling ("pause and ask user intervention"). `dist/` cannot be staged without force-add because it is ignored, but the guard is mandatory and explicit regardless. |
| R6 | Are any rebuild/Cargo/VM actions needed to prove semantics? | **No. The artifact is NOT rebuilt, re-copied, modified, measured (again), or deleted by Task 2. Its existing on-disk state is input, not output. The recorded size `1 715 296 bytes` may be cited in the adherence file only as "observed by Task 1" (attributed), never re-measured.** | TODO Task 2 is verification + semantics-mapping; "Do not … delete existing build outputs as part of the workflow". Re-measurement would add a non-reproducible host-shell dependency for nothing. |
| R7 | Where come the two semantic explanations (Linux-executable-not-.exe; named-volume vs `dist/` persistence) live? | **Task 3's `docs/build.md` (and supporting metadata updates in `project-structure.md` / `context.md`) — recorded by Task 2 as a mapping table (§6 of this plan), NOT duplicated. Task 2 creates no documentation file containing these explanations.** TODO Task 3's own bullets are unchanged by this mapping. | Caller's frozen decision: "Task 2 records this mapping rather than duplicating it"; global-plan Task 2 row: "Semantics documentation … delivered in Task 3's docs/build.md; Task 2's own scope: verification + minimal note where documentation lands (placeholder-free)". |
| R8 | 4.3/4.4 outcomes for a verification-only task? | **4.3 (review): reviewers verify the §3 execution records against §3's expected outputs and §5's guards; fixes → conditional `fix:` commit only if a defect is proven; simplification of a task with zero code files is "not required" unless a defect is found. 4.4 (documentation): expected outcome "not required" — Task 2 has no code/comments of its own (script header from Task 1 already documents the runtime contract; semantics prose is Task 3's docs). No new documentation file is created.** | Scope boundary: the only Task-2-scoped artifacts are the plan file (already committed) and the adherence file (4.5b). |
| R9 | Does Task 2 ever mark the TODO `[DONE]` or create `.agent/reports/`? | **No `[DONE]` until 4.6 (single heading edit at `### 2.`, nothing else in the TODO file); no `.agent/reports/` creation ever in Task 2 (Task 4's file).** | Workflow sub-steps; Overwrite-TODO-File-Prevention rule. |

## 2. High-level approach

Task 2 is a **verification-only** task (expected: zero file changes):

1. Implementer runs the frozen verification command matrix (§3) on the host — proves `dist/` is ignored by the existing `.gitignore:34` rule and nothing under `dist/` is tracked or staged.
2. Contingency branch (R4) fires only if verification fails — a single-line `.gitignore` append is the only implementer-approved repair.
3. The semantic explanations demanded by TODO bullets 3–4 are NOT written by Task 2; §6 freezes the mapping to Task 3's `docs/build.md`. TODO bullet 5's prohibitions become §7's do-not list.
4. Evidence flows: implementer summary (§3 record template) → Task 2 adherence confirmation file (4.5b) → Task 4's report. No other recording channel.

## 3. Verification command matrix (4.2 — host git commands, in this exact order)

Run from repo root `C:\repo\rust-youtube-streamer`, on branch `feat/linux-release-build-artifact`. Git-only; all commands are single commands (no `&&` chains). Record every run per the template at the end of this section.

| Run | Command (exact) | Expected exit | Expected stdout | Proves |
| --- | --- | --- | --- | --- |
| P0 | `git status --porcelain` | 0 | empty | tree clean before verification |
| P0b | `git log --oneline -1` | 0 | one line whose hash = the plan-file commit recorded in §8 of the caller handoff (first line after this plan's commit), subject `docs: record task 2 ignore semantics plan` | right branch state |
| V1 | `git check-ignore -v dist/rust-youtube-streamer-service` | 0 | `.gitignore:34:dist/` TAB `dist/rust-youtube-streamer-service` | the artifact is ignored, by the exact expected rule |
| V2 | `git check-ignore -v dist/` | 0 | `.gitignore:34:dist/` TAB `dist/` | the directory rule itself is the matcher |
| V3 | `git ls-files dist/` | 0 | empty | nothing under `dist/` is tracked |
| V4 | `git status --porcelain` | 0 | empty | the ignored artifact does not appear as pending change |
| V5 | `git diff --cached --name-only` | 0 | empty | index contains nothing (nothing staged) |
| V6 | `git status --porcelain --ignored=matching -- dist/` | 0 | exactly one line: `!! dist/` | `dist/` is in ignored state at git level |
| V7 | `git ls-files --others --exclude-standard dist/` | 0 | empty | nothing under `dist/` is even an untracked-not-ignored candidate (no path can sneak into a normal staging call) |
| V8 | Read `.gitignore` lines 32–34 (Read tool) | — | line content exactly: `# Build artifacts (generic)` / `build/` / `dist/` | rule lives in the intended section (placement intact) |

- If any of V1, V2, V8 shows a mismatch → contingency path R4. If V3, V4, V5, V6 or V7 shows a non-expected output (including any `dist/` entry) → STOP and report (R5(c)); never self-repair an index.
- After the matrix, if a contingency `.gitignore` edit was applied: re-run the FULL matrix; all rows must pass before any commit.

### Record template (mandatory output for the returned completion summary; Task 4 consumes this verbatim)

For each run P0/P0b/V1–V8: exact command, exact exit code, exact stdout (TAB separators noted as `→TAB←`), and the byte size line of `.gitignore:34` (only from V8's read; no sizes of the binary). Nothing may be invented; a check not run is recorded as "not run" with the reason.

## 4. Implementation steps for the implementer (sub-step 4.2)

Execute in order. Check the plan between steps.

**Git/branch precheck (no branch/checkout allowed — step 2 is already complete):**

1. Run `git status --porcelain` (repo root). Expected: empty. If not empty — STOP and report to caller; do not commit or stash anything.
2. Run `git log --oneline -1` and `git branch --show-current`. Expected HEAD = the plan-file commit from §8 of the caller's handoff (message `docs: record task 2 ignore semantics plan`) and branch `feat/linux-release-build-artifact`. If either differs — STOP and report.
3. Confirm the plan file `.kilo/plans/20261009-phase-01-1-task2-ignore-semantics.md` exists on disk (Read tool). If missing — STOP and report.

**Verification:**

4. Execute the §3 matrix (P0/P0b first, then V1–V8) on the HOST; follow the STOP rules above; compile the record template results into the returned summary.
5. Expected net effect of the whole task: **zero repository file changes** (all runs pass; no contingency edit needed). If exactly that holds, do NOT create any commit in 4.2 (nothing to commit). If the contingency fired and `.gitignore` was appended (single line), commit ONLY `.gitignore` with the message from R4(3).

**Completion-summary content (returned to caller):** the §3 record-template table verbatim; explicit statement "no files changed" or the single-contingency-edit statement; explicit confirmation that no `dist/` path was ever staged.

## 5. Git handling (FROZEN)

- Branch: current `feat/linux-release-build-artifact` only. No branch creation, no checkout, no merge, no push (steps 2/5 of the workflow own those; push restricted to step 5).
- Pre-commit guard (R5) before EVERY commit: `git status --porcelain` + `git diff --cached --name-only` recorded and free of `dist/` entries. Explicit per-path staging only.
- Commits in the Task 2 cycle (in order):
  1. (This planning step, already done by architector at plan save) `docs: record task 2 ignore semantics plan` — the plan file only.
  2. (4.2, conditional — only if contingency landed) `chore: restore dist/ ignore rule` — `.gitignore` only.
  3. (4.3 fix cycle, only if a proven defect requires a fix) `fix: address task 2 verification feedback` — Task-2-scoped files only.
  4. (4.5b adherence) `docs: record task 2 adherence confirmation` — `.kilo/plans/20261009-phase-01-1-task2-adherence.md` only.
  5. (4.6 completion) `docs: mark phase 01.1 task 2 complete` — the `[DONE]` mark on `### 2.` in `.agent/todos/20261009/20261009-todo-4.md` only.
- If 4.2 produced no commit (expected) and 4.6's `[DONE]` edit is the first Task-2-cycle commit after the plan file: that ordering is correct and requires no adjustment.

## 6. Semantic-explanations mapping (Task 2 record → Task 3 delivery; FROZEN)

TODO Task 2 bullets 3–4 are delivered by Task 3. The implementer records nothing beyond this table (which is already a fact of the approved global plan); no prose is duplicated anywhere by Task 2.

| TODO Task 2 bullet | Verbatim requirement | Delivery home (Task 3 scope) | Frozen section target in `docs/build.md` |
| --- | --- | --- | --- |
| Bullet 3 | "Document that this script produces a **Linux release executable** using the pinned Linux Rust container, not a Windows `.exe`." | Task 3's `docs/build.md` | "What it proves / does NOT prove" section (Linux container build ≠ native runtime validation; no `.exe`; `auth`/`run` placeholders) — plus "Expected output" noting the artifact path | 
| Bullet 4 | "Explain that the named Docker volume stores Cargo's intermediate/release output, while the final copied artifact persists in the shared repository's `dist/` directory." | Task 3's `docs/build.md` | "Expected output" + a short "Where build output lives" statement (named volume `rust-streamer-target` for intermediates; repo-root `dist/` persists the final artifact; `dist/` gitignored; artifact disposable/re-creatable). Also slated for `.agent/project-structure.md` dist/ entry (Task 3) |

Task 3's own TODO bullets (lines 38–48 of the TODO file) remain untouched by this plan — this table only records attribution, it does not broaden or rewrite them.

## 7. Hard scope boundaries ("explicitly do not do" list)

- Do NOT remove, rename, or modify the named Docker volume (`rust-streamer-target`) in any way; no `docker volume rm`, no `docker system prune`, no `docker builder prune`, no any-prune verb — no Docker commands at all (R1).
- Do NOT delete or modify any existing build output: not `dist/rust-youtube-streamer-service`, not anything inside the named volume, not `Cargo.lock`, not old logs.
- Do NOT touch `scripts/build-linux.sh` or `scripts/dev-checks.sh`.
- Do NOT create/modify `docs/build.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md`, or any file under `docs/`, `src/**`, `config/`, `credentials/`, `fonts/` (Task 3 scope).
- Do NOT create `.agent/reports/` or any file under it (Task 4 scope).
- Do NOT run any `cargo` command (no build/check/fmt/clippy/test) — Task 2 needs no toolchain work at all; `scripts/dev-checks.sh` on the final tree belongs to Task 4.
- Do NOT edit `Cargo.toml`/`Cargo.lock`/`Dockerfile`/`docker-compose.yml`.
- Do NOT mark other TODO tasks or acceptance checkboxes; the only Task-2 TODO edit is 4.6's ` [DONE]` on `### 2.`.
- Do NOT unignore anything (no `!dist/...` or similar negation anywhere in `.gitignore`), even in contingency.
- Do NOT push, merge, checkout, rename the TODO file, or force-add any ignored path (`git add -f` is forbidden outright since staged-binaries must never exist).
- Do NOT run `git reset --hard`, `git restore --staged`, `git clean` on any state (if something is staged that must not be: STOP and report per R5(c)).

## 8. Workflow-cycle notes for Task 2's remaining sub-steps

- **4.3 (review/simplification)**: reviewers validate the §3 execution record (commands/outputs/exit codes) against the expected-output column, confirm zero unexpected file changes, and confirm §5's guards were executed per commit attempted. Fix plans only for proven defects; otherwise "not required". Simplification must not invent code for a task with none.
- **4.4 (documentation)**: expected outcome "not required" (R8). Nothing to comment (no code), nothing to document (Task 3 owns docs). If the docs-specialist believes a change is needed, it returns a question instead of editing.
- **4.5b (plan adherence)**: architector re-runs the §3 matrix at current HEAD, compares implementation behavior against this plan (§3–§5, §7), and writes the durable adherence confirmation file `.kilo/plans/20261009-phase-01-1-task2-adherence.md` containing: header (same pattern as Task 1's adherence file), the re-verified evidence table verbatim (R2's durable record), Task-2 TODO-row cross-check, commit-history sanity, deviations (none expected), verdict. Commit per §5 item 4.
- **4.6 (completion)**: append only ` [DONE]` to the `### 2. Define Artifact Semantics and Safe Ignore Rules` heading in `.agent/todos/20261009/20261009-todo-4.md`; preserve every other byte of the file. Commit per §5 item 5.

## 9. Cross-check against TODO Task 2 (point-by-point)

| TODO Task 2 requirement | Plan coverage |
| --- | --- |
| "Confirm `dist/` is ignored by the existing `.gitignore` rule; adjust the rule only if needed, without unignoring any other build artifacts." | §3 V1/V2/V8 (positive confirmation, exact rule + section placement); R4 contingency (append-only repair semantics, no unignoring, single-line limit, STOP on complexity) |
| "Ensure generated binaries under `dist/` are not staged or committed." | §3 V3–V7 (`ls-files` empty, index empty, ignored-state proof, no sneaking candidates); §1 R5 (staging guard + STOP policy); §5 pre-commit guard on every commit; §7 force-add ban |
| "Document that this script produces a Linux release executable … not a Windows `.exe`." | §1 R7 + §6 mapping (delivered in Task 3's `docs/build.md`; recorded, not duplicated) |
| "Explain that the named Docker volume stores Cargo's intermediate/release output, while the final copied artifact persists in the shared repository's `dist/`." | §1 R7 + §6 mapping (Task 3's `docs/build.md`; recorded, not duplicated) |
| "Do not remove the named volume, prune Docker resources, or delete existing build outputs as part of the workflow." | §1 R1 (no Docker/VM at all) + §7 first three bullets |
| Caller deliverable: evidence location stable for Task 4 | §1 R2 → `.kilo/plans/20261009-phase-01-1-task2-adherence.md` (traced to Task 4 report row 5) |
| Caller deliverable: VM/MCP need decision | §1 R1 — host-only, frozen |

Acceptance criteria touched by Task 2 (checked later, not now): "The final artifact is generated under root-level `dist/`, which remains gitignored; no artifact is tracked or staged" (evidence set = §3 matrix via the adherence file), and Task 4's report row 5 consumes the same file. All other AC rows belong to Tasks 1/3/4 and are deliberately out of Task 2's reach.

## 10. Return to caller (architector summary)

- Decision set: R1–R9 (host-only verification; evidence durably in Task 2's adherence file; `.gitignore` expected untouched with a frozen single-line-append contingency; explicit staging-guard policy; zero Cargo/docker work; semantic mapping to Task 3 frozen; 4.3/4.4 outcomes pre-decided).
- Verification: matrix P0/P0b/V1–V8 on the host with the frozen expected outputs (pre-validated during planning) and the verbatim record template.
- Git handling: only frozen-message commits (§5), explicit per-path staging, pre-commit guard on every commit, no push.
- Explicit do-not list: §7 (volume/prune/delete prohibition, script/docs/metadata/report non-touching, no unignore, no force-add, no reset --hard).
