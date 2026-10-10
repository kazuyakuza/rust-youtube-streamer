# Task 4 Implementation Plan — Verify Both Build Paths and Save a Completion Report (Phase 01.2)

- Source TODO: `.agent/todos/20261010/20261010-todo-1.md`, section `### 4. Verify Both Build Paths and Save a Completion Report` — all 6 bullets binding; read first.
  1. VM MCP Windows build command → recorded exit status + observed output/artifact size.
  2. Re-run Linux release build (documented command) to prove no toolchain regression.
  3. Standard dev-checks gate on the final tree.
  4. `dist/` stays ignored; neither output tracked or staged; logs only under gitignored `logs/checks/` (if the script writes them there).
  5. Tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` with real evidence only — **never invent verification, outputs, sizes, or test results**.
  6. Update TODO task markers / acceptance checkboxes only AFTER actual steps + verification complete; do NOT rename TODO with `-DONE` (Critical Workflow step 5 owns that).
- Global plan: `.kilo/plans/20261010-phase-01-2-windows-cross-build.md` — binding decisions #5 (VM allowlist; git host-only; timeoutMs ≥ 900000), #6 (branch/commit conventions), #9 (report with real evidence only).
- Prior evidence trail (REAL, recorded — cited, not re-derived, not re-produced unless mandated below):
  - Task 1: `.kilo/plans/20261010-phase-01-2-task1-docker-cross-toolchain.md` — image route `rust:1.82` + `gcc-mingw-w64-x86-64` (dlltool needed at COMPILE time by `windows-sys` raw-dylib) + `rustup target add x86_64-pc-windows-gnu`; self-contained rust-lld link verified, no `.cargo/config.toml`; probe exe 3 191 192 bytes; commit `e5505e1`.
  - Task 2: `.kilo/plans/20261010-phase-01-2-task2-build-windows-script.md` — script content; objdump import audit (8 lines verbatim; system-DLL-only; NO MinGW runtime DLLs); CRLF incident + repair + `.gitattributes` guard; commits `e775a7b`, `8c4fb43`.
  - Task 3: `.kilo/plans/20261010-phase-01-2-task3-windows-build-docs.md` — docs facts; commit `f72ec2e`; marker commit `783eceb`.
  - Implementer sub-step evidence: first windows run exit 0, 3 191 192 bytes; linux repair-run exit 0, 1 715 240 bytes; failure probes exit 1 (cwd /tmp; empty `CARGO_TARGET_DIR`); ignore evidence recorded.
  - Marker/plan-tracking commits: `9e74b31` (global plan), `87159a9` (version 0.4.0), `d6e8f1b` (task-1 marker), `abbbad3`, `1dc18e0` (pre-phase planning).
- Report format precedent (mirror): `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md`.
- MCP/VM constraints (binding for every VM command below):
  - Call `alpine-vm_vm_status` BEFORE every `alpine-vm_vm_run_command`.
  - Allowlist prefixes: `docker|sh|apk|ls|cat|ps|df|free|uname|pwd|whoami` — **`git` is NOT allowed on the VM**; all git evidence runs on the HOST.
  - Cargo stderr visibility may be partial — **exit status is authoritative** (documented policy in README).
  - Long builds: `timeoutMs` ≥ `900000`.
- Scope: Task 4 only. Compose commands always `docker compose -f /rust-youtube-streamer/docker-compose.yml …`.

---

## Step A — VM: Fresh final-tree verification sequence (in this exact order)

All runs are FRESH observations of the final tree; record per run: exit code, full observed stdout/stderr tokens (verbatim, no paraphrase), and artifact size as printed/observed.

### A.1 Windows build (fresh, run FIRST)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
```

- `timeoutMs`: `900000`.
- Expected outcome (warm incremental): exit `0`; script's documented output lines (build banner, `Release build succeeded.`, `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (<N> bytes).`, static-linking/import-audit line enumerating the 7 system DLLs, cross-compile caveat line, FFmpeg line). Record `<N>` verbatim — never the expectation.
- On non-zero: capture exitCode + combinedOutput verbatim → STOP entire step, report failure to caller (do not fix anything; fixes are a caller decision per the no-assumption rule).

### A.2 Windows artifact verification (read-only)

```
ls -l /rust-youtube-streamer/dist/windows/
```

- Expected exit 0; exactly one entry `rust-youtube-streamer-service.exe` with a non-empty size; the listed size must match the size printed by A.1. Record both.

### A.3 Linux build (fresh, proves no regression from the toolchain change)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

- `timeoutMs`: `900000`.
- Expected exit `0`; expected directory/file `dist/rust-youtube-streamer-service`; record printed size (prior repair-run observation was exit 0, 1 715 240 bytes — cite as prior run in the report; the fresh run's OWN number is the report's fresh value if it differs).
- A.3 runs AFTER A.1: after linux build, verify the Windows artifact still exists and is non-empty (`ls -l /rust-youtube-streamer/dist/windows/` again) — the Linux run must not have disturbed it. Record.

### A.4 Dev-checks gate (fresh, on the final tree)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

- `timeoutMs`: `900000`.
- Record: exit code; the four `PASS|FAIL` lines (`fmt-check`, `check`, `test`, `clippy`); the summary line (`ALL CHECKS PASSED` or the actual failure summary); the printed `Log file:` path. The log path must be under gitignored `logs/checks/` only — verify the printed path starts with `logs/checks/`; nothing else may write outside it.
- **CRITICAL**: if any check is `FAIL` or exit ≠ 0, record the EXACT failure text and summary verbatim, and STOP the step — do not claim green anywhere, do not "fix" source. The stop and its handling belong to the caller.
- Note: no failure probes/re-runs of A.1–A.4 beyond the above; earlier failure-probe evidence (exit 1 runs) is cited from the sub-step report, flagged as earlier runs.

### A.5 — Conditional re-audit (ONLY if A.1's output is ambiguous)

Trigger condition: A.1's printed size line or artifact listing is missing/contradictory (ambiguous), not a fresh-audit on a whim. If triggered:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "x86_64-w64-mingw32-objdump -p /rust-streamer-target/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe | grep 'DLL Name'"
```

- Mirrors Task 2's audit command; expected exit 0; the 8 observed import lines should match the Task 2 verbatim list (kernel32, KERNEL32, msvcrt, ntdll, USERENV, WS2_32, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives). Any difference → record verbatim and report to caller (do not reclassify silently).

## Step B — HOST: git evidence (post-hoc, all read-only)

Run on the host repo (`C:\repo\rust-youtube-streamer`), after Step A succeeds. Record each command verbatim with its output:

1. `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` → expect `.gitignore:34:dist/` (path + match exactly as observed; record actual).
2. `git ls-files dist/` → expect empty stdout (neither artifact tracked).
3. `git status --porcelain --ignored=matching -- dist/` → expect exactly `!! dist/` (no untracked/staged leakage under dist).
4. `git status --porcelain` → expect clean (no modified/untracked tracked-tree entries besides potentially nothing). Any unexpected entry → record verbatim and report to caller.
5. `git status` (full) → confirm tracked tree clean; confirm branch is `feat/windows-gnu-cross-build`.
6. Confirm neither artifact is staged: covered by 2–4 (empty `ls-files`, `!!` only, clean porcelain). If any contradicting evidence appears → STOP and report.

Also confirm (from A.4's printed log path) no log file landed anywhere except `logs/checks/` (gitignored); if the script wrote no log, state that in the report ("no log written" style record — no omission).

## Step C — HOST: create the completion report (NEW tracked file)

Path: `.agent/reports/20261010-phase-01-2-windows-build-workflow.md`. Mirror `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` structure. Full skeleton (implementer replaces the verbatim-run placeholders `<observed:…>` with the ACTUAL measured values from Steps A/B; sections with nothing observed are NOT allowed — every section must be filled with observed data or an explicit `not run / not observed: <element> — reason: <why>` recording):

```markdown
# Phase 01.2 Completion Report — Reproducible Windows Executable Build

## Overview
<phase summary; state at report time: branch feat/windows-gnu-cross-build, HEAD=<observed git rev-parse --short HEAD>, date 2026-10-10; source TODO .agent/todos/20261010/20261010-todo-1.md; container-only verification statement: no native Windows runtime validation was performed — no Windows-host evidence exists>

## Files Created/Changed
<table mirroring 01.1: File | Task | Change | Commit — rows:
  Dockerfile | 1 | extended (mingw+w64-target) | e5505e1
  scripts/build-windows.sh (+ .gitattributes) | 2 (+CRLF guard) | new | e775a7b
  docs/build-windows.md, README.md, docs/build.md, .agent/project-structure.md, .agent/project-info/context.md | 3 | created/updated | f72ec2e
  markers/plan-tracking: 9e74b31, d6e8f1b, 8c4fb43, 783eceb, abbbad3, 1dc18e0; version bump 87159a9 (0.3.0→0.4.0)
  this report + task-4 marker commits: "this commit" values observed at commit time>

## Exact Docker/MCP Commands & Exit Statuses
<fenced verbatim blocks, one per run; clearly flag which runs are fresh (Step A of this task) and which are recorded earlier runs (first windows run exit 0 3191192 bytes; linux repair-run exit 0 1715240 bytes; failure probes exit 1)>

## Target/Toolchain Chosen
<x86_64-pc-windows-gnu; rust:1.82 pinned + gcc-mingw-w64-x86-64 (dlltool required at compile time by windows-sys raw-dylib); rustc self-contained rust-lld link route; no .cargo/config.toml fallback needed; rustup target add in the Dockerfile; binaries observed>

## Windows Build Result (observed path/size)
<dist/windows/rust-youtube-streamer-service.exe; sizes observed in Step A + earlier recorded run; ls -l evidence>

## Linux Build Result (observed path/size)
<dist/rust-youtube-streamer-service; fresh Step A.3 size + prior 1715240 bytes repair-run citation; non-regression statement and a note that the Windows artifact was re-checked non-empty after the Linux run (Step A.3 second ls check)>

## Runtime DLL Handling
<static GNU runtime; system-DLL-only audit basis; enumerate exactly the 7 observed names: msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll; audit via x86_64-w64-mingw32-objdump; cross-compile ≠ native runtime validation caveat; FFmpeg external, not bundled>

## Dev-Checks Result (final tree)
<fenced block with the four PASS|FAIL lines + summary line + Log file: path; exit code; the log path is under gitignored logs/checks/>
<CRITICAL: fill ONLY from the ACTUAL dev-checks output. Never write "ALL CHECKS PASSED" unless the gate actually printed it with exit 0. If any check failed: paste exact failure text, and stop short of any green claim — record "dev-checks gate: FAILED as observed; see failure text above".>

## dist/ Gitignore & Tracking Verification
<fenced blocks from Step B commands 1–3 verbatim; statement that neither artifact is tracked or staged; clean-tree statement from command 5>

## Environment Notes
<core.autocrlf=true on the repo host; the CRLF incident: working tree-wide CRLF conversion broke container sh; repair normalized build-linux.sh + Dockerfile to LF; durable guard .gitattributes (*.sh text eol=lf, Dockerfile text eol=lf) added in commit e775a7b; EOL verified from host (git ls-files --eol i/lf w/lf) and VM (cat -A, no ^M)>

## Evidence Index
<linked list: .kilo/plans/20261010-phase-01-2-windows-cross-build.md, task1-docker-cross-toolchain.md, task2-build-windows-script.md, task3-windows-build-docs.md, this task's plan 20261010-phase-01-2-task4-verification-report.md; 01.1 precedent report>
<commits on feat/windows-gnu-cross-build observed via git log --oneline, listed verbatim>

## Known Limitations & Not-Run Checks
<native Windows execution/runtime validation NOT performed — no Windows-host evidence exists; no Windows environment was used; container Linux checks and cross-build only; FFmpeg external; no installer/service/elevation; OAuth/run placeholders exit 3; any checkbox failing verification (from Step E) is listed here with the exact reason>
```

Report writing rules (binding):
- Only observed data. Every `<observed:…>` token above is replaced with the real measured value; empty sections forbidden; a check that could not be completed is documented with the reason in Known Limitations.
- **Never invent lines like `ALL CHECKS PASSED`**; the summary table is filled from the ACTUAL dev-checks output; if any check fails → record exact failure text and stop short of claiming green in the whole report.
- Garbage/expectation values (e.g. sizes from the plan prose) are never written as observed facts — only the run's own printed output.

## Step D — VM/HOST cross-check (before markers)

- Re-read the report; confirm every number in it matches Step A/B outputs byte-for-byte (spot-check: exe size printed vs `ls -l` bytes; dev-check summary text vs gate output; log path vs `logs/checks/`).
- Ambiguity → re-run only the single ambiguous observation once; persistent ambiguity → STOP and report.

## Step E — HOST: TODO-marker plan (AFTER Steps A–D all succeed)

Edit `.agent/todos/20261010/20261010-todo-1.md` ONLY:
1. Heading `### 1.…[DONE]` style already present for tasks 1–3; set `### 4. Verify Both Build Paths and Save a Completion Report` → `### 4. Verify Both Build Paths and Save a Completion Report [DONE]`.
2. Acceptance criteria checklist (`## Acceptance Criteria`, lines 75–84): flip `[ ]` → `[x]` ONLY for the checkboxes whose verification succeeded per actual evidence from Steps A/B/C — all 10 expected to pass on a green run, but each flip individually conditioned:
   - L75 (target/toolchain reproducibly installed): evidence = Task 1 image build + smoke checks (cited).
   - L76 (build-windows.sh locked-build produces the exe): evidence = Step A.1 exit 0 + A.2 listing.
   - L77 (actionable errors, non-empty check, no suppression): evidence = script content (Task 2) + failure-probe runs exit 1 (cited as earlier runs).
   - L78 (runtime DLLs decided + documented): evidence = Task 2 audit + docs Task 3 + report runtime section.
   - L79 (Linux build still succeeds): evidence = Step A.3 exit 0.
   - L80 (dev-checks passes): evidence = Step A.4 output; if it did not pass, leave `[ ]` unchecked and record the exact failure in the report's Known Limitations.
   - L81 (docs accurate): evidence = Task 3 files at HEAD (`docs/build-windows.md`, README, `docs/build.md`, `.agent/project-structure.md`, `.agent/project-info/context.md`) cross-checked against the observed facts recorded in this task's report (sizes never hardcoded; DLL list verbatim; caveat text present).
   - L82 (both artifacts under gitignored dist/, no root target/): evidence = Step A.2/A.3 listings + Step B commands 2–3 + absence of a root `target/` directory in `ls /rust-youtube-streamer` (record the listing evidence).
   - L83 (tracked completion report with real evidence): evidence = Step C file existence + `git ls-files .agent/reports/` showing it tracked after the report commit.
   - L84 (no Phase 02 features / unrelated scope): evidence = Step B git status clean + no source/dependency changes observed in this task's commit set.
3. Guard: any checkbox failing verification stays `[ ]` and the reason goes ONLY into the report's Known Limitations section; do NOT edit checkbox text.
4. Revise any ctx/notes files ONLY if a fact in them is now demonstrably wrong; otherwise untouched.
5. The earlier `[DONE]` rule: do NOT rename the TODO file (Critical Workflow step 5 owns renaming); do not create new TODO files.

(Procedure numbered to keep mapping explicit):

```text
E.1 flip ### 4. heading to [DONE]
E.2 flip acceptance checkboxes per evidence (each conditioned on real output)
E.3 leave any failing checkbox unchecked + reason in report limitations
E.4 no TODO rename, no new TODO files, no other markdown edits beyond 1–2
```

## Step F — HOST: commit sequencing (two commits, phase-01.1 precedent)

1. Report commit first:
   - `git add .agent/reports/20261010-phase-01-2-windows-build-workflow.md .kilo/plans/20261010-phase-01-2-task4-verification-report.md`
   - `git diff --cached --name-only` → exactly those two files. Nothing else staged (gitignore-compliance: nothing tracked matching ignore patterns staged).
   - `git commit -m "docs: add phase 01.2 completion report"` → expect exit 0.
2. TODO-marker commit second:
   - `git add .agent/todos/20261010/20261010-todo-1.md`
   - `git diff --cached --name-only` → exactly the TODO.
   - `git commit -m "chore: mark phase 01.2 task 4 and acceptance criteria done"` → expect exit 0.
3. `git status --porcelain` → empty (clean). `git log --oneline -5` → confirm the two new commits on top of `783eceb`. Record hashes for the Evidence Index (append to report is NOT done post-commit; hashes are recorded in execution notes if the caller wants them).
4. **NEVER `git push`** in this step (push belongs to Critical Workflow step 5, to `origin` only, per git-remote-safety).

## Explicitly NOT done in this step

- No fixes to code, scripts, Dockerfile, or docs unless a verification failure FORCES it — and even then: STOP and report to the caller instead of fixing (no-assumption rule).
- No `dist/` deletes/prunes; no Docker pruning; no image rebuild (image already built by Task 1).
- No push; no branch merge; no TODO `-DONE` file rename; no new TODO files; no new plan files beyond this one; no changes to `Cargo.toml`/`Cargo.lock`/`src/**`/`.gitignore`.
- No re-run of Task 2's failure probes as fresh evidence (cited, not re-executed).
- No claims anywhere that native Windows runtime was validated.

## Failure/STOP semantics (encoded)

| Condition | Action |
| --- | --- |
| Any VM command non-zero (A.1/A.3/A.4 or re-audit) | Capture exitCode + combinedOutput verbatim; STOP; report to caller; no fix attempt, no green claim, no checkbox flip for the failing item |
| Dev-checks FAIL line or blocked summary | Record exact text; report failed-branch; leave L80 unchecked; Commit only the report mirroring reality (limitations section), and STOP before the marker commit (caller decides) |
| gitignore/tracking evidence contradicts expectations (Step B) | Record verbatim; STOP; report to caller |
| Ambiguous A.1 output | One re-run of the single ambiguous observation (A.5 audit if import-related); persistent ambiguity → STOP + report |

## Verification checklist (mapped 1:1 to TODO Task 4 bullets + acceptance criteria)

- [ ] Fresh Windows build command run via VM MCP; exit status + observed output tokens + artifact size recorded (Step A.1 + A.2).
- [ ] Fresh Linux build run with the unchanged documented command; exit status, artifact dir/file and size recorded; Windows artifact still non-empty after (Step A.3).
- [ ] Fresh dev-checks gate run; exit code, four PASS|FAIL lines, summary line, log path recorded; logs under `logs/checks/` only (Step A.4).
- [ ] Conditional A.5 re-audit only if ambiguous; mirrors Task 2 audit command if triggered.
- [ ] Gitignore/tracking evidence collected host-side (Step B, all 6 observations) with expected/actual recorded.
- [ ] Completion report created at the exact path with the full skeleton, real values everywhere, no invented lines, no empty sections, limitations section complete (Step C).
- [ ] Re-read/double-check pass executed (Step D).
- [ ] Task-4 heading `[DONE]` + acceptance-criteria checkboxes flipped only per real evidence (Step E).
- [ ] Two commits in sequence with the exact messages and staged-file sets; clean tree after; no push (Step F).
