# Phase 01.2 Completion Report — Reproducible Windows Executable Build

## Table of Contents

- [Overview](#overview)
- [Files Created/Changed](#files-createdchanged)
- [Exact Docker/MCP Commands & Exit Statuses](#exact-dockermcp-commands--exit-statuses)
- [Target/Toolchain Chosen](#targettoolchain-chosen)
- [Windows Build Result (observed path/size)](#windows-build-result-observed-pathsize)
- [Linux Build Result (observed path/size)](#linux-build-result-observed-pathsize)
- [Runtime DLL Handling](#runtime-dll-handling)
- [Dev-Checks Result (final tree)](#dev-checks-result-final-tree)
- [dist/ Gitignore & Tracking Verification](#dist-gitignore--tracking-verification)
- [Environment Notes](#environment-notes)
- [Evidence Index](#evidence-index)
- [Known Limitations & Not-Run Checks](#known-limitations--not-run-checks)

## Overview

Phase 01.2 added a Docker-based Windows x86-64 GNU cross-compilation workflow: `scripts/build-windows.sh` runs `cargo build --release --locked --target x86_64-pc-windows-gnu` inside the pinned Compose `rust` service on the Alpine VM and copies the resulting Windows `.exe` into the gitignored `dist/windows/` directory, documented in `docs/build-windows.md`, the README, `docs/build.md`, and this tracked report under `.agent/reports/`.
State at report time: branch `feat/windows-gnu-cross-build`, HEAD `783eceb`, date 2026-10-10; source TODO `.agent/todos/20261010/20261010-todo-1.md`.
Verification is container-only: no native Windows runtime validation was performed — no Windows-host evidence exists. The fresh verification sequence (Windows build → artifact listing → Linux build → re-check → dev-checks gate) all completed with exit 0 on the final tree during this task; earlier recorded implementation evidence (failure probes, Task 2 objdump audit) is cited, not re-executed.

## Files Created/Changed

| File | Task | Change | Commit |
| --- | --- | --- | --- |
| `Dockerfile` | 1 | extended (gcc-mingw-w64-x86-64 + `rustup target add x86_64-pc-windows-gnu`), +4 lines | `e5505e1` |
| `scripts/build-windows.sh` (+ `.gitattributes` CRLF guard) | 2 | new, +75 lines (+3 lines) | `e775a7b` |
| `docs/build-windows.md`, `README.md`, `docs/build.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` | 3 | created/updated | `f72ec2e` |
| markers/plan-tracking: `1dc18e0`, `abbbad3`, `9e74b31`, `d6e8f1b`, `8c4fb43`, `783eceb`; version bump `87159a9` (0.3.0→0.4.0) | — | marker/plan-tracking commits as listed | as listed |
| `.kilo/plans/20261010-phase-01-2-task4-verification-report.md` | 4 | this task's plan; untracked in working tree at report time, staged by the plan's Step F report commit | pending (Step F commit `docs: add phase 01.2 completion report`, hash recorded in execution notes after commit) |
| `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` | 4 | new (this report); untracked in working tree at report time, staged by the plan's Step F report commit | pending (Step F commit `docs: add phase 01.2 completion report`, hash recorded in execution notes after commit) |

Row facts: commit file stats observed via `git show --stat` — `e5505e1` touches `Dockerfile` only; `e775a7b` touches `.gitattributes` and `scripts/build-windows.sh`; `f72ec2e` touches the five listed docs files (172 insertions, 9 deletions).

`src/**`, `Cargo.toml`, `Cargo.lock`, `docker-compose.yml`, `.gitignore`: no changes in Phase 01.2 (`Cargo.lock`/`Cargo.toml` version bumps belong to commit `87159a9`'s planned version-bump scope; no dependency or source changes).

## Exact Docker/MCP Commands & Exit Statuses

Each command is rendered byte-verbatim with its recorded exit code. Rows marked FRESH are new observations made during this task on the final tree (Step A of the task-4 plan); rows marked RECORDED-CITED are earlier implementation evidence cited from committed task evidence, not re-executed.

Fresh Windows build run (run first, Step A.1):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
exit=0
Building Windows x86-64 GNU release executable using the tracked lockfile: cargo build --release --locked --target x86_64-pc-windows-gnu
    Updating crates.io index
    Finished `release` profile [optimized] target(s) in 3.02s
Release build succeeded.
Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (3191192 bytes).
The GNU runtime is statically linked: the observed import audit (x86_64-w64-mingw32-objdump) found no MinGW runtime DLLs, and only standard Windows system DLLs are required (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll).
Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled.
```

The captured output shows the crates.io index update and the `Finished \`release\` profile [optimized] target(s) in 3.02s` line; cargo stderr capture may be partial on the VM, so exit status remains the authoritative success evidence (README policy).

Fresh Windows artifact listing (Step A.2):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows
exit=0
/rust-youtube-streamer/dist:
total 1676
-rwxrwx--- 1 root 103 1715240 Oct 10 17:10 rust-youtube-streamer-service
drwxrwx--- 1 root 103       0 Oct 10 17:19 windows

/rust-youtube-streamer/dist/windows:
total 3120
-rwxrwx--- 1 root 103 3191192 Oct 10  2026 rust-youtube-streamer-service.exe
```

Fresh Linux build run (Step A.3):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
exit=0
Building Linux release executable using the tracked lockfile: cargo build --release --locked
    Updating crates.io index
    Finished `release` profile [optimized] target(s) in 2.50s
Release build succeeded.
Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (1715240 bytes).
The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe.
```

Fresh Windows artifact re-check after the Linux run (Step A.3 second listing):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist/windows/
exit=0
total 3120
-rwxrwx--- 1 root 103 3191192 Oct 10  2026 rust-youtube-streamer-service.exe
```

Fresh dev-checks gate run (Step A.4) — full summary block in the [Dev-Checks Result](#dev-checks-result-final-tree) section:

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
exit=0
```

Fresh project-root listing (Step A root listing, evidence for acceptance criterion "no generated target output in the repository root"):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer
exit=0
total 42
-rwxrwx--- 1 root 103   845 Oct  8 21:24 AGENTS.md
-rwxrwx--- 1 root 103   156 Oct  9 00:35 CHANGELOG.md
-rwxrwx--- 1 root 103 11661 Oct 10 16:27 Cargo.lock
-rwxrwx--- 1 root 103   371 Oct 10 16:26 Cargo.toml
-rwxrwx--- 1 root 103   502 Oct 10 17:09 Dockerfile
-rwxrwx--- 1 root 103  1233 Oct  8 21:24 LICENSE
-rwxrwx--- 1 root 103 12809 Oct 10 17:40 README.md
drwxrwx--- 1 root 103     0 Oct 10 16:24 config
drwxrwx--- 1 root 103     0 Oct 10 16:12 credentials
drwxrwx--- 1 root 103     0 Oct 10 17:19 dist
-rwxrwx--- 1 root 103   677 Oct  9 03:14 docker-compose.yml
drwxrwx--- 1 root 103     0 Oct 10 17:39 docs
drwxrwx--- 1 root 103     0 Oct  8 22:40 fonts
drwxrwx--- 1 root 103     0 Oct 10 16:24 logs
drwxrwx--- 1 root 103     0 Oct 10 17:17 scripts
drwxrwx--- 1 root 103     0 Oct  9 23:22 src
```

No `target/` directory appears at the project root; Cargo intermediates stay in the named volume `rust-streamer-target` via `CARGO_TARGET_DIR`.

RECORDED-CITED earlier implementation runs (cited from committed task evidence and the implementer sub-step record; not re-executed in this task per the task-4 plan's no-re-probe rule):

```text
# first windows run during implementation (implementer sub-step)
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
exit=0, artifact 3191192 bytes

# linux repair-run during implementation (implementer sub-step)
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
exit=0, artifact 1715240 bytes

# failure probe E1: empty CARGO_TARGET_DIR
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e CARGO_TARGET_DIR= rust sh scripts/build-windows.sh
exit=1 (build-windows precondition error; verbatim text recorded in Task 2 evidence)

# failure probe E2: wrong cwd
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cd /tmp && sh /rust-youtube-streamer/scripts/build-windows.sh'
exit=1 (build-windows precondition error; verbatim text recorded in Task 2 evidence)

# Task 1 toolchain probe
probe executable size 3191192 bytes (commit e5505e1 evidence)
```

MCP/VM constraints applied to all runs above: `alpine-vm_vm_status` was consulted before command execution; `docker compose` prefixes via the allowlisted compose route; `timeoutMs` 900000 for build/check commands and 600000 for the `ls` listings; exit status is the authoritative success evidence.

## Target/Toolchain Chosen

- Target: `x86_64-pc-windows-gnu` (64-bit Windows GNU route) — selected because builds run in a Linux container; MSVC was not attempted this phase.
- Image route: pinned `rust:1.82` base extended with `gcc-mingw-w64-x86-64` (dlltool required at compile time by `windows-sys` raw-dylib imports) plus `rustup target add x86_64-pc-windows-gnu`, added in the Dockerfile in commit `e5505e1`; Rust stays pinned to the existing 1.82 baseline (fresh run confirms `Cargo: cargo 1.82.0 (8f40fc59f 2024-08-21)` in the dev-checks output).
- Link route: rustc's self-contained `rust-lld` link verified in Task 1; no `.cargo/config.toml` linker override was needed.
- Build invocation: `cargo build --release --locked --target x86_64-pc-windows-gnu` with the tracked lockfile, intermediates kept in the named volume `rust-streamer-target` via `CARGO_TARGET_DIR`; no new Rust crate dependencies were added.
- Binaries observed: `dist/windows/rust-youtube-streamer-service.exe` (3191192 bytes) and `dist/rust-youtube-streamer-service` (1715240 bytes).

## Windows Build Result (observed path/size)

The fresh Step A.1 run completed with exit 0 and printed `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (3191192 bytes).` The Step A.2 listing above independently confirms exactly one entry in `dist/windows/` at 3191192 bytes, matching the size printed by A.1 byte-for-byte. The same size (3191192 bytes) was also observed in the Task 1 probe and the first implementation run — the number is stable across runs.

## Linux Build Result (observed path/size)

The fresh Step A.3 run completed with exit 0 and printed `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (1715240 bytes).`, and the Step A.2 listing above shows the artifact at 1715240 bytes. The prior implementation repair-run also observed exit 0 with 1715240 bytes (cited, not re-derived); the fresh run's own value agrees. This proves the Phase 01.2 toolchain change did not regress the Linux build path. (The Phase 01.1 report recorded 1715296 bytes on the older tree; the fresh Phase 01.2 runs — implementation repair-run and this task's run — both produce 1715240 bytes.) The Windows artifact was re-checked non-empty (3191192 bytes) immediately after the Linux run, per Step A.3's second listing; the Linux run did not disturb it.

## Runtime DLL Handling

- Decision: static GNU runtime. The build script's fresh output (Step A.1) states: the observed import audit (`x86_64-w64-mingw32-objdump`) found no MinGW runtime DLLs, and only standard Windows system DLLs are required — exactly these 7 names: `msvcrt.dll`, `kernel32.dll`, `ntdll.dll`, `userenv.dll`, `ws2_32.dll`, `api-ms-win-core-synch-l1-2-0.dll`, `bcryptprimitives.dll`.
- Audit basis: Task 2 ran `x86_64-w64-mingw32-objdump -p` against the built `.exe` and recorded an 8-line import list verbatim (`kernel32`, `KERNEL32`, `msvcrt`, `ntdll`, `USERENV`, `WS2_32`, `api-ms-win-core-synch-l1-2-0.dll`, `bcryptprimitives`) — 7 unique system DLLs after case-fold dedupe of `kernel32`/`KERNEL32`; system DLLs only, no MinGW runtime DLLs. That audit is cited from Task 2 evidence; it was not re-run in this task because the conditional re-audit trigger (ambiguous A.1 output) was not met.
- Consequence: no redistributable MinGW DLLs need to be copied next to the `.exe`; nothing is bundled. FFmpeg remains an independently installed external prerequisite and is not bundled.
- Caveat: cross-compilation and a clean import audit do not prove native Windows runtime behavior — the `.exe` must be tested separately on Windows. No such test exists yet (see Known Limitations).

## Dev-Checks Result (final tree)

Fresh Step A.4 run on the final tree (acceptance gate); filled ONLY from the actual output:

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
exit=0
UTC time: Sat Oct 10 18:32:06 UTC 2026
Cargo: cargo 1.82.0 (8f40fc59f 2024-08-21)
Log file: logs/checks/20261010T183206Z.log
== fmt-check == start
== check == start
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.03s
== test == start
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.53s
running 46 tests
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
== clippy == start
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.42s
--- Summary ---

PASS fmt-check exit=0 (1 s)
PASS check exit=0 (2 s)
PASS test exit=0 (14 s)
PASS clippy exit=0 (2 s)
ALL CHECKS PASSED
Log file: logs/checks/20261010T183206Z.log
```

The printed log path starts with `logs/checks/` — a gitignored location (`logs/*` at `.gitignore:18`). Host-side confirmation that nothing landed outside it: `git status --porcelain --ignored=matching -- logs/` printed exactly `!! logs/checks/`.

## dist/ Gitignore & Tracking Verification

All commands run on the repo host after Step A succeeded; each rendered with its actual output.

Command 1 — ignore rule match:

```text
git check-ignore -v dist/windows/rust-youtube-streamer-service.exe
exit=0
.gitignore:34:dist/	dist/windows/rust-youtube-streamer-service.exe
```

Command 2 — no tracked artifacts:

```text
git ls-files dist/
exit=0
(empty stdout)
```

Command 3 — dist/ is ignored-only, no untracked/staged leakage:

```text
git status --porcelain --ignored=matching -- dist/
exit=0
!! dist/
```

Command 4 — tracked-tree status (recorded verbatim; one expected untracked entry):

```text
git status --porcelain
exit=0
?? .kilo/plans/20261010-phase-01-2-task4-verification-report.md
```

Command 5 — full status:

```text
git status
On branch feat/windows-gnu-cross-build
Untracked files:
  (use "git add <file>..." to include in what will be committed)
	.kilo/plans/20261010-phase-01-2-task4-verification-report.md

nothing added to commit but untracked files present (use "git add" to track)
```

Command 6 — staged-artifact check: covered by commands 2–4 (empty `git ls-files dist/`, single `!! dist/` line, no staged entries in porcelain output). Neither `dist/windows/rust-youtube-streamer-service.exe` nor `dist/rust-youtube-streamer-service` is tracked or staged.

Interpretation: the sole untracked entry at Step B time is this task's own plan file (`.kilo/plans/20261010-phase-01-2-task4-verification-report.md`), which the plan's Step F report commit stages together with this report; it is not a tracked-tree modification or a dist/ leak. The tracked tree is clean; branch is `feat/windows-gnu-cross-build`. At report-writing time the report file itself is additionally untracked (left in the working tree per this step's scope); after Step F both files are committed and the tree returns to clean.

## Environment Notes

- The repo host runs `core.autocrlf=true` (fresh observation: `git config --get core.autocrlf` → `true`).
- CRLF incident (Phase 01.2 Task 2, recorded): the working-tree-wide CRLF conversion under `core.autocrlf=true` broke the container's `sh` execution of build scripts; the repair normalized `scripts/build-linux.sh` and `Dockerfile` back to LF, and a durable guard `.gitattributes` (`*.sh text eol=lf`, `Dockerfile text eol=lf`) was added in commit `e775a7b`.
- EOL verification: fresh host observation `git ls-files --eol scripts/build-windows.sh scripts/build-linux.sh scripts/dev-checks.sh Dockerfile .gitattributes` shows `i/lf w/lf` with `attr/text eol=lf` for the four guarded files and `i/lf w/lf` for `.gitattributes` itself; VM-side `cat -A` verification (no `^M` in scripts) was recorded during Task 2 implementation (cited, not re-run — EOL state is durably enforced by the `.gitattributes` guard and the fresh host check).
- MCP/VM policy applied throughout: cargo stderr capture may be partial on the VM; exit status is the authoritative success evidence (README lines 146–148 remain the sole policy source). In this task's runs the stderr channel was fully captured, but the policy was still applied.

## Evidence Index

Re-verification procedure (for future agents): the FRESH commands in the [Exact Docker/MCP Commands & Exit Statuses](#exact-dockermcp-commands--exit-statuses) section are the re-run sequence (Windows build → artifact listing → Linux build → artifact re-check → dev-checks gate); the exit status is the authoritative success evidence. Do not re-execute the RECORDED-CITED Task 2 failure probes or the conditional objdump re-audit — see the no-re-probe note in that section and Known Limitations.

Evidence files (repo-relative):

- [.kilo/plans/20261010-phase-01-2-windows-cross-build.md](../../.kilo/plans/20261010-phase-01-2-windows-cross-build.md) — global phase plan
- [.kilo/plans/20261010-phase-01-2-task1-docker-cross-toolchain.md](../../.kilo/plans/20261010-phase-01-2-task1-docker-cross-toolchain.md) — Task 1 plan
- [.kilo/plans/20261010-phase-01-2-task2-build-windows-script.md](../../.kilo/plans/20261010-phase-01-2-task2-build-windows-script.md) — Task 2 plan
- [.kilo/plans/20261010-phase-01-2-task3-windows-build-docs.md](../../.kilo/plans/20261010-phase-01-2-task3-windows-build-docs.md) — Task 3 plan
- [.kilo/plans/20261010-phase-01-2-task4-verification-report.md](../../.kilo/plans/20261010-phase-01-2-task4-verification-report.md) — this task's plan
- [.agent/reports/20261009-phase-01-1-build-artifact-workflow.md](20261009-phase-01-1-build-artifact-workflow.md) — Phase 01.1 precedent report

Commits on `feat/windows-gnu-cross-build` (observed via `git log --oneline`), as of HEAD `783eceb` at report time, Phase 01.2 set:

```text
783eceb chore: mark phase 01.2 task 3 done and track its plan
f72ec2e docs: document windows build workflow
8c4fb43 chore: mark phase 01.2 task 2 done and track its plan
e775a7b feat: add windows release build script
d6e8f1b chore: mark phase 01.2 task 1 done and track its plan
e5505e1 feat: add windows gnu cross toolchain to docker build image
87159a9 chore: bump version to 0.4.0
9e74b31 docs: add phase 01.2 windows cross-build global plan
abbbad3 docs: record Windows build phase as next work
1dc18e0 docs: plan Windows cross-compilation workflow
```

Task-4 commits (created after this report was written, by the plan's Step F; hashes pending at report-writing time and recorded in the caller's execution notes):

```text
pending — docs: add phase 01.2 completion report   (report + task-4 plan file)
pending — chore: mark phase 01.2 task 4 and acceptance criteria done   (TODO markers, Step E/F)
```

## Known Limitations & Not-Run Checks

- Native Windows execution/runtime validation NOT performed — no Windows-host evidence exists; no Windows environment was used at any point. This phase provides container Linux checks and a cross-build only; the printed caveat line (`Cross-compilation does not prove native Windows runtime behavior…`) is authoritative.
- FFmpeg is an external prerequisite, not bundled: the `.exe` expects a separately installed FFmpeg at runtime; no FFmpeg behavior was exercised.
- No installer, no Windows service registration, no elevation: the app's platform model is unchanged (normal user executable).
- `auth`/`run` remain placeholders that validate config, report not-implemented, and exit 3; the application pipeline (OAuth, YouTube API, chat, renderer, FFmpeg process, streaming) is not implemented — out of scope per TODO Constraints.
- Conditional objdump re-audit (plan Step A.5) not run in this task — trigger condition (ambiguous A.1 output) not met; the DLL audit evidence is Task 2's recorded objdump run, cited rather than re-executed.
- Task 2 failure probes (exit 1: wrong cwd, empty `CARGO_TARGET_DIR`) were not re-run as fresh evidence per the plan; their exit-1 records are cited from earlier runs.
- not run / not observed: the two task-4 commit hashes inside this report — reason: this step's scope excludes committing; the report is left untracked in the working tree and the commits are created afterwards by the plan's Step F (hashes recorded in execution notes, not back-patched into this file).
- Step E TODO checkbox flips are outside this step's scope; all verification prerequisites observed here are green (Windows build exit 0, Linux build exit 0, dev-checks `ALL CHECKS PASSED` with exit 0, dist/ ignored and untracked, docs/tree evidence cited), and no observed failure requires leaving any checkbox unchecked.
