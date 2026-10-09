# Implementation Plan — Phase 00.1 Task 1: `scripts/dev-checks.sh` (Critical Workflow Steps 4.1b → handoff to 4.2)

- Date: 2026-10-09
- Source task: `.agent/todos/20261009/20261009-todo-2.md` — section `### 1. Create scripts/dev-checks.sh` ONLY.
- Global plan: `.kilo/plans/20261009-dev-checks-script.md` (Global pre-analysis + Task 1 pre-analysis).
- Workflow: `.kilo/commands/critical-workflow.md` — this file is the Task 1 step-4.1b output.
- Path-name note: caller instruction fixes this plan name as `20261009-phase00-1-task1-dev-checks-script.md` (phase = 00.1). It supersedes the tentative `20261009-phase01-task1-dev-checks-script.md` name recorded in the global plan.
- Role of implementer file consumer: JUNIOR developer under 50% restriction. All structural/scope decisions below are frozen; do not re-decide anything marked "FROZEN".
- Plan is plan-only: no code or commands are executed by the architect who authored it.

## 0. Scope Boundary (hard)

This plan covers Task 1 of the TODO only:

| In scope | Out of scope (other steps/sub-agents) |
|---|---|
| Create `scripts/dev-checks.sh` (content frozen in §2.1) | Task 2 documentation (README, `.agent/project-structure.md`, `.agent/project-info/context.md`) |
| Validation runs on the Alpine VM (§3) | Any change to `.gitignore`, Dockerfile, docker-compose.yml, `Cargo.*`, `src/**` |
| One commit on `feat/dev-checks-script` (§2.2) | Log rotation, CI, parallel checks, fmt auto-fix, semver bump |
| Reading `vm_status` before any VM command | Step 5 TODO-file completion / merge / push |
| 4.3 code review/simplification happens AFTER this task's 4.2 | 4.6 TODO `[DONE]` marking happens AFTER 4.3 |

Workflow applicability (Task 1, per global plan step table): `4.1b (this) → 4.2 → 4.3 → 4.6`.
4.3 (code review & simplification) is REQUIRED for Task 1 and is the next step after 4.2. 4.6 (mark `[DONE]` on `### 1. Create scripts/dev-checks.sh` + commit) is a LATER step. 4.4/4.5 are not scheduled for Task 1 (docs are Task 2). No `git push` and no branch creation in this task: `feat/dev-checks-script` already exists (created in global step 1); push is restricted to global step 5.

## 1. Preconditions (implementer must verify before writing code)

1. Branch: `git status` → expect on branch `feat/dev-checks-script`; working tree clean except this plan file(s) under `.kilo/plans/` (untracked/modified, tracked only if planner chose so) and `.agent/todos/20261009/20261009-todo-2.md` if it was already committed in global step 1. If the TODO file is still pending, commit it FIRST with message `docs: add phase 00.1 dev-checks script TODO` before any other work (gitignore compliance: read `.gitignore` and ensure `logs/*` and `target/` files are never staged).
2. Host must NOT have Rust; all execution is via the Alpine VM (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`, single command per call, no `&&` chains).
3. Compose service facts (FROZEN, verified 2026-10-09):
   - `docker-compose.yml` service `rust` pin `rust:1.82` + rustfmt/clippy; `working_dir: /rust-youtube-streamer`; bind mount `/rust-youtube-streamer:/rust-youtube-streamer`; env `CARGO_TARGET_DIR=/rust-streamer-target` with named volume `rust-streamer-target` (redirect is baked into compose, NOT passed at CLI — never pass volume/env flags at CLI).
   - Host repo is visible on the VM at path `/rust-youtube-streamer` (VirtualBox shared folder). When the implementer writes a file on the host, it appears at the same relative path on the VM.
4. `.gitignore` already ignores `logs/*` (subdirs included, `*.log` too) and keeps `!logs/.gitkeep`; `/target/` ignored; `!Cargo.lock` guard present. NO `.gitignore` edits in this task.
5. Image already built during Phase 00; `docker compose ... build` is not required for validation (add only if `run` errors with "no such image").

## 2. FROZEN Artifacts

### 2.1 `scripts/dev-checks.sh` — exact content

Design decisions (FROZEN by 4.1b encoder):

- D-1 POSIX sh only (container `/bin/sh` = dash). No bashisms (`[[`, `local` is avoided; dash supports `local` but plain globals are used instead), no `set -e` (checks must continue after failure), no `set -u`.
- D-2 Capture strategy: each check's output is captured via POSIX command substitution (`output=$(sh -c "$cmd" 2>&1)`); `$?` of the assignment is the command's exit code. Rationale: pipe-to-tee would need non-POSIX `pipefail`; temp files under `logs/checks/` would work but add clutter. Both channels merged by `2>&1`.
- D-3 Writes: ONLY `logs/checks/<UTC-timestamp>.log` created via `mkdir -p logs/checks` + append. No temp files, no writes outside `logs/checks/` (+ cargo artifacts in the named volume via cargo itself). No source mutation; fmt runs `--check` only (report diff, never apply).
- D-4 Failure-path hook (FROZEN OVER option): env var `FORCE_FAIL` (explicit name, no hidden magic). Values allowed: `fmt-check|check|test|clippy`. When `FORCE_FAIL` equals the current check name, `run_check` substitutes the real command with a failing command with known shape: `echo intentional failure: FORCE_FAIL=<F9> 1>&2; exit 7` (captured via `2>&1` lambda, exit 7). It writes nothing to source or root; other three checks run normally. `FORCE_FAIL` is unset in normal use, used only in an ad-hoc validation run. It stays in the committed script and is documented in README during Task 2 (handoff note §5).
- D-5 Duration: seconds via `date +%s` before/after each check; line format `PASS|FAIL <name> exit=<code> (<duration>s)`.
- D-6 Log name: `logs/checks/$(date -u +%Y%m%dT%H%M%SZ).log`. Files accumulate (no rotation). Duplicate-second collision is possible in theory (two runs starting in the same UTC second would append to the same file) — accepted; runs take minutes, risk negligible.
- D-7 Result aggregation: global counters `failed_count=0`, `passed_count=0`, accumulated multi-line `results`; summary block; exit 0 when `failed_count` 0, else 1 (never swallow failures).
- D-8 Every user-visible line goes to container stdout AND appends to the log file. Container stdout+stderr are relayed by compose and captured by the MCP (contract verified 2026-10-09: `exitCode`, `stdout`, `stderr`, `combined output`; stderr NOW captured).
- D-9 Checks ALWAYS run in the fixed order fmt-check → check → test → clippy, one after another; a fmt/check failure does not stop later checks; cascading failure after "check" is acceptable and reported.
- D-10 Condition hygiene: single condition per `if`/comparison; functions ≤ 50 lines body; ≤ 2 positional params per function; no commented-out code; comments only where non-obvious (header only); descriptive names with `run_check_`-prefixed locals and shared `emit`; all persistent globals declared at top (`log_file`, `results`, `passed_count`, `failed_count`); no hidden new globals; no `$(...)` nesting beyond what the file already has.

Exact content to write to `scripts/dev-checks.sh` (≈ 69 lines; implementer may adjust ONLY micro-formatting such as spacing inside `printf` strings, never the structure, names, messages formats, or semantics):

```sh
#!/bin/sh
# Container-side development checks runner (POSIX sh; cwd must be /rust-youtube-streamer).
# Writes ONLY under logs/checks/. Set FORCE_FAIL to fmt-check|check|test|clippy to make
# exactly that check fail with exit 7 (failure-path validation); leave unset in normal use.

log_file=""
results=""
passed_count=0
failed_count=0

emit() {
    printf '%s\n' "$1"
    printf '%s\n' "$1" >> "$log_file"
}

run_check() {
    run_check_name=$1
    run_check_command=$2
    run_check_start=$(date +%s)
    if [ "$FORCE_FAIL" = "$run_check_name" ]; then
        run_check_command="echo intentional failure: FORCE_FAIL=$FORCE_FAIL 1>&2; exit 7"
    fi
    run_check_output=$(sh -c "$run_check_command" 2>&1)
    run_check_exit=$?
    run_check_duration=$(( $(date +%s) - run_check_start ))
    emit "== $run_check_name == start"
    emit "$run_check_output"
    if [ "$run_check_exit" -eq 0 ]; then
        run_check_status=PASS
        passed_count=$((passed_count + 1))
    else
        run_check_status=FAIL
        failed_count=$((failed_count + 1))
    fi
    run_check_result="$run_check_status $run_check_name exit=$run_check_exit ($run_check_duration s)"
    results="${results}
$run_check_result"
}

print_header() {
    emit "UTC time: $(date -u)"
    emit "Cargo: $(cargo --version)"
    emit "Log file: $log_file"
}

print_summary() {
    emit "--- Summary ---"
    emit "$results"
    if [ "$failed_count" -eq 0 ]; then
        overall_status="ALL CHECKS PASSED"
    else
        overall_status="$failed_count CHECK(S) FAILED"
    fi
    emit "$overall_status"
    emit "Log file: $log_file"
}

run_fmt_check() { run_check fmt-check "cargo fmt --check"; }
run_cargo_check() { run_check check "cargo check --locked"; }
run_cargo_test() { run_check test "cargo test --locked"; }
run_cargo_clippy() { run_check clippy "cargo clippy --locked -- -D warnings"; }

mkdir -p logs/checks
log_file="logs/checks/$(date -u +%Y%m%dT%H%M%SZ).log"
print_header
run_fmt_check
run_cargo_check
run_cargo_test
run_cargo_clippy
print_summary
if [ "$failed_count" -eq 0 ]; then
    exit 0
fi
exit 1
```

Structural notes for the implementer (do not redesign):
- `passed_count` is tracked for symmetry/future summaries; currently only `failed_count` drives the exit code and overall message (single-condition rule).
- `FORCE_FAIL` is referenced without `${FORCE_FAIL:-}` because no `set -u` is used; unset expands empty and can never equal a check name.
- The per-check `== <name> == start` header precedes its output (TODO wording); the plan/global decision uses this header shape.
- Empty check output (e.g. `cargo fmt --check` when green) prints one blank line via `emit` — accepted, harmless.
- `sh -c "$run_check_command"` indirection is intentional (uniform exit-code propagation); do not replace with `eval`.

### 2.2 Commit plan (FROZEN)

1. Write the file on the host at `C:\repo\rust-youtube-streamer\scripts\dev-checks.sh` (new `scripts/` folder; verify it does not exist yet — confirmed 2026-10-09: no `scripts/**` paths).
2. Read `.gitignore`; run `git status`. Expected tracked changes: only `scripts/dev-checks.sh` untracked. `logs/checks/` outputs are ignored (verify nothing under `logs/` appears in status; if it does, DO NOT stage it and report).
3. Stage exactly: `git add scripts/dev-checks.sh` (if the shell rejects the single file, stage `scripts/`). Never stage anything under `logs/`, `target/`, or other root files; never force-add gitignored paths.
4. Commit (exact message): `feat: add dev-checks script for containerized Rust checks`
5. Do NOT commit: plan markdown files, log outputs, TODO-file edits, or any other repo file. Branch stays `feat/dev-checks-script`; NO push (restricted to global step 5).

## 3. Validation Runs on the Alpine VM (all FROZEN single commands)

Order and expectations. Every VM sequence starts with `vm_status` (no args). The `rust` image must exist; commands start with the allowlisted `docker ... compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ...` prefix.

V-1 (sync check) — after writing the file on the host:
`cat /rust-youtube-streamer/scripts/dev-checks.sh`
Expect: exact frozen content. If empty/missing, the shared-folder sync failed — stop and report to caller.

V-2 (POSIX syntax gate):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -n scripts/dev-checks.sh`
Expect: exit 0, empty output. On syntax error: fix ONLY trivial quoting/typos against §2.1, re-run V-1/V-2; if still failing, report.

V-3 (green full run — acceptance evidence #2/#3):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
Expect: exit 0; stdout shows header (UTC time + Cargo version + log path), four sequential check blocks each ending in `PASS <name> exit=0 (<n> s)` for `fmt-check`, `check`, `test`, `clippy`, then summary `ALL CHECKS PASSED` and the log path; MCP `stdout` AND `stderr` fields populated (cargo writes to stderr — `Finished` lines are now visible per new MCP contract; absence is not failure, exit code is authoritative).

V-4 (log artifact check):
`ls -l /rust-youtube-streamer/logs/checks`
Expect: at least one `*.log` file from the V-3 run (newer timestamp). If absent despite V-3 exit 0, report (root-cause before proceeding).

V-5 (failure-path evidence — acceptance #4, uses FROZEN hook D-4):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh`
Expect: exit 1; blocks for fmt-check and check end `PASS ...`, test block contains `intentional failure: FORCE_FAIL=test` and ends `FAIL test exit=7 (<n> s)`, clippy still runs and PASSES (order preserved, hook path writes nothing to source/root — the substituted command is an `echo`/`exit 7` only), summary `1 CHECK(S) FAILED`, a new log file recorded. If any expectation differs, stop and report.

V-6 (fresh-log confirmation for V-5):
`ls -l /rust-youtube-streamer/logs/checks`
Expect: a second `.log` file (newer timestamp) than the one from V-3.

V-7 (git hygiene — host):
`git status` → expect only `scripts/dev-checks.sh` as the single change for commit (plus pre-existing `.kilo/plans/`/TODO state from step 1); NO `logs/`, `target/`, or generated files listed. Cross-check against `.gitignore` compliance rule. Also verify no `target/` directory appeared in the repo root (host-side Glob for `target/*` → no files): acceptance #5.

Then the commit per §2.2. After the commit, leave the repo clean; do NOT delete or modify anything under `logs/checks/` (gitignored artifacts).

Rollback note: if V-3 unexpectedly exits 1 (tree was green per TODO facts), capture the failing block output, investigate ONLY within Task 1 scope (e.g., script bug vs. tree regression): script bugs → fix against §2.1 and revalidate; tree regressions outside Task 1 → stop and report to caller, do not touch unrelated files.

## 4. Acceptance Criteria Mapping (TODO Task 1 + section criteria)

| TODO criterion (Task 1-relevant lines) | Satisfied by |
|---|---|
| `scripts/dev-checks.sh` exists, POSIX-sh compatible, ≤ 100 lines | §2.1 (≈ 69 lines; measured after write, report real count); V-2 gate |
| No source mutation / no fmt auto-apply | D-3 (fmt `--check` only); D-4 hook writes nothing; no script writes outside `logs/checks/` |
| Writes only under `logs/checks/` (+ named volume via cargo) | D-3; verified V-4/V-6 + host `git status` V-7 |
| Single MCP command invokes the full check suite on pinned compose service | V-3 (and V-5) command shape |
| Full run exits 0 with four PASS lines and log under `logs/checks/` | V-3 (exit 0, 4× PASS), V-4 (log exists) |
| Failures produce `FAIL … exit=<n>` lines and script exit 1 (verified, else documented) | VERIFIED via D-4 hook: V-5 shows `FAIL test exit=7` + exit 1; no faked evidence, no broken tree |
| `logs/checks/` NOT tracked by git; `target/` never in mount root | `.gitignore` facts §1.4; V-7 |
| Helpers/small functions, clear names, minimal comments | D-10; code-reviewer re-checks in 4.3 |
| Commit message | `feat: add dev-checks script for containerized Rust checks` (§2.2) |
| Not in Task 1 scope: README/structure/context updates (acceptance #6), semver, CI, FFmpeg, app features | Task 2 steps and later phases |

Explicit "NOT done in this step" (implementer must state in the completion report): Task 2 documentation, 4.3 review, 4.6 `[DONE]` marking, branch merge/push.

## 5. Handoff Notes (to the Planner; do not act now)

1. Task 2 (docs) MUST document in README Build Checks: the one-command script invocation (V-3 shape), `logs/checks/` destination, and the `FORCE_FAIL` env hook (test-only; leave unset) — else D-4 would be an undocumented magic flag. `.agent/project-structure.md` gains a `scripts/` entry; `context.md` gains recent-changes bullets with V-3/V-5 evidence and the log path.
2. Step 4.3 reviewer must check: ≤ 100 lines, function bodies ≤ 50 lines, no bashisms, no commented-out code, single-condition `if`s, ≤ 2 params/function, no output outside `logs/checks/`, message formats preserved verbatim.
3. 4.5b plan-adherence check is NOT scheduled for Task 1 by the global plan; keep it that way unless the planner escalates.

## 6. Required Completion Report (implementer → planner)

1. Files created/changed (expect exactly `scripts/dev-checks.sh` + 1 commit hash + message).
2. Exact MCP commands + exitCodes for V-1…V-7 (include full V-3 stdout+stderr and V-5 combined output).
3. Log path(s) written (both V-3 and V-5 filenames).
4. Failure-path evidence result (V-5: `FAIL test exit=7`, script exit 1, other three PASS).
5. Checks/commands not run + why (e.g., `docker compose build` skipped because image pre-built).
6. Assumptions/blockers.
7. Out-of-scope confirmation (no Task 2 docs, no `.gitignore`/compose/Dockerfile/`Cargo.*`/`src/**` edits, no push).
