# Phase 01.1 Completion Report — Reproducible Linux Build Artifact

## Overview

Phase 01.1 added a documented, repeatable Linux release build for the current Rust executable: `scripts/build-linux.sh` runs `cargo build --release --locked` inside the pinned Compose `rust` service on the Alpine VM and copies the final Linux executable into the gitignored `dist/` directory, with the workflow documented in `docs/build.md` and the README and this tracked report under `.agent/reports/`.
State at report time: branch `feat/linux-release-build-artifact`, HEAD `ff447a4`, date 2026-10-10; source TODO `.agent/todos/20261009/20261009-todo-4.md`.
Verification is container-only (Linux checks and the Linux release build on the Alpine VM); no native Windows or Linux runtime validation was performed.
This report establishes the project's ongoing convention: future phase TODOs require a tracked completion report under `.agent/reports/` containing actual verification evidence.

## Files Created/Changed

| File | Task | Change | Commit |
| --- | --- | --- | --- |
| `scripts/build-linux.sh` | 1 | new, mode 100644 | `92880c7`, review fix `6118322` |
| `.gitignore` | 2 | no changes; existing rule confirmed | — |
| `docs/build.md` | 3 | new, 89 lines | `4cf8679` |
| `README.md` | 3 | modified, 2 hunks +19 lines | `7eb2583` |
| `.agent/project-structure.md` | 3 | updated | `417d6ba` |
| `.agent/project-info/context.md` | 3 | updated | `417d6ba` |
| `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` | 4 | new (this report) | this commit |

Row facts: per `20261009-phase-01-1-task1-adherence.md`, per `20261009-phase-01-1-task2-adherence.md`, per `20261009-phase-01-1-task3-adherence.md`; commit hashes observed in the `git log --oneline -25` output (Evidence Index).

`src/**, Cargo.toml, Cargo.lock, Dockerfile, docker-compose.yml: no changes in Phase 01.1 except the planned 0.3.0 version bump.`

## Exact Docker/MCP Commands & Exit Statuses

Each command is rendered byte-verbatim with its recorded `exit=<code>`; Task 1 run records are cited from committed evidence, and the dev-checks and `ls -l` rows are fresh observations for this task.

Release build success run (per `20261009-phase-01-1-task1-adherence.md`; message texts per `20261009-phase-01-1-task1-build-script.md`):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
exit=0
Building Linux release executable using the tracked lockfile: cargo build --release --locked
Release build succeeded.
Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (1715296 bytes).
The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe.
```

Build failure runs (E2 then E1; commands per `20261009-phase-01-1-task1-build-script.md` run matrix, exits and stderr per `20261009-phase-01-1-task1-adherence.md`):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e CARGO_TARGET_DIR= rust sh scripts/build-linux.sh
exit=1
build-linux: error: CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)
```

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cd /tmp && sh /rust-youtube-streamer/scripts/build-linux.sh'
exit=1
build-linux: error: expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: /tmp
```

Dev-checks run on the final tree (fresh observation):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
exit=0
--- Summary ---

PASS fmt-check exit=0 (1 s)
PASS check exit=0 (3 s)
PASS test exit=0 (11 s)
PASS clippy exit=0 (2 s)
ALL CHECKS PASSED
Log file: logs/checks/20261010T034008Z.log
```

MCP artifact re-check (fresh observation):

```text
ls -l /rust-youtube-streamer/dist
exit=0
-rwxrwx--- 1 root vboxsf 1715296 Oct 10 01:49 rust-youtube-streamer-service
```

Exit status is the authoritative MCP success evidence; README lines 146–148 document the caveat (quoted verbatim):

> - MCP-based agents: cargo's stderr (e.g., `Finished` lines) may not appear in captured MCP/VM
>   output. The command exit status is the authoritative success evidence; a missing `Finished` line
>   is not a failure.

## Release Build Result (observed path/size)

The Task 1 release build run recorded exit=0 (per `20261009-phase-01-1-task1-adherence.md`) and produced `dist/rust-youtube-streamer-service` — size 1715296 bytes, mode `-rwxrwx---`, owner `root:vboxsf`, re-confirmed by the fresh `ls -l` re-check. Compilation output stays in the named volume `rust-streamer-target`; the repository root shows no `target/` directory. The artifact is a Linux executable built inside the Linux container; it is not a Windows `.exe`.

## Standard Dev-Checks Result on Final Tree

This run is the acceptance gate for the final tree, covering all Task 1–3 deliverables; it completed with exit=0.

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
exit=0
--- Summary ---

PASS fmt-check exit=0 (1 s)
PASS check exit=0 (3 s)
PASS test exit=0 (11 s)
PASS clippy exit=0 (2 s)
ALL CHECKS PASSED
Log file: logs/checks/20261010T034008Z.log
```

The printed log path is under `logs/checks/`, a gitignored directory (README line 115 documents this behavior; `.gitignore` lines 17–18 match).

## dist/ Gitignore & Tracking Confirmation

```text
git check-ignore -v dist/rust-youtube-streamer-service
exit=0
.gitignore:34:dist/	dist/rust-youtube-streamer-service
```

```text
git ls-files dist/
exit=0
(empty stdout)
```

```text
git status --porcelain --ignored=matching -- dist/
exit=0
!! dist/
```

The artifact is gitignored by the matching rule `.gitignore:34` (`dist/` under the `# Build artifacts (generic)` section, lines 32–34 per `20261009-phase-01-1-task2-ignore-semantics.md`) and is not tracked or staged. The frozen full V1–V8 matrix — including empty `git diff --cached --name-only` (V5) and the single `!! dist/` line (V6) — is recorded in `.kilo/plans/20261009-phase-01-1-task2-adherence.md` Check 1 (per `20261009-phase-01-1-task2-adherence.md`).

## Known Limitations & Skipped Checks

- No native Windows or Linux runtime validation was performed (container-only; README/build-docs caveat text is authoritative).
- The build proves compilation and artifact production only; the application pipeline is not implemented: `auth`/`run` remain placeholders that validate config, report not-implemented, and exit 3. No OAuth, YouTube API, chat, renderer, or FFmpeg process behavior exists.
- No Windows cross-build/multi-platform matrix (out of scope per TODO Constraints).
- Skipped-by-design checks and reasons: no re-execution of Task 1 build failure paths in Task 4 (failing runs would delete/rebuild nothing but the evidence is durably recorded — freshness adds no proof); no artifact signature/permissions work beyond `ls -l` observation (out of scope); cargo stderr capture may be partial on the VM — exit codes used as authority (README lines 146–148 remain the sole policy source).
- Recording convention for unavailable elements: `not run / not observed: <element> — reason: <why>`; no planned evidence element was unavailable in this cycle.

## Evidence Index

Evidence files (repo-relative):

- [.kilo/plans/20261009-phase-01-1-build-artifact.md](../../.kilo/plans/20261009-phase-01-1-build-artifact.md) — global phase plan
- [.kilo/plans/20261009-phase-01-1-task1-build-script.md](../../.kilo/plans/20261009-phase-01-1-task1-build-script.md) — Task 1 plan
- [.kilo/plans/20261009-phase-01-1-task1-adherence.md](../../.kilo/plans/20261009-phase-01-1-task1-adherence.md) — Task 1 adherence
- [.kilo/plans/20261009-phase-01-1-task2-ignore-semantics.md](../../.kilo/plans/20261009-phase-01-1-task2-ignore-semantics.md) — Task 2 plan
- [.kilo/plans/20261009-phase-01-1-task2-adherence.md](../../.kilo/plans/20261009-phase-01-1-task2-adherence.md) — Task 2 adherence
- [.kilo/plans/20261009-phase-01-1-task3-build-docs.md](../../.kilo/plans/20261009-phase-01-1-task3-build-docs.md) — Task 3 plan
- [.kilo/plans/20261009-phase-01-1-task3-adherence.md](../../.kilo/plans/20261009-phase-01-1-task3-adherence.md) — Task 3 adherence
- [.kilo/plans/20261009-phase-01-1-task4-completion-report.md](../../.kilo/plans/20261009-phase-01-1-task4-completion-report.md) — this task's plan

Commits on `feat/linux-release-build-artifact` (observed), as of HEAD `ff447a4` at report time:

```text
ff447a4 docs: record task 4 completion report plan
080b534 docs: mark phase 01.1 task 3 complete
f7759e8 docs: record task 3 adherence confirmation
a808147 refactor: tighten build docs wording
75a36e9 docs: record task 3 simplification plan
417d6ba docs: record build workflow in structure and context
7eb2583 docs: link build guide from readme
4cf8679 docs: add linux release build guide
70f377e docs: record task 3 build docs plan
858e378 docs: mark phase 01.1 task 2 complete
5cb233d docs: record task 2 adherence confirmation
b1022f3 docs: record task 2 ignore semantics plan
8448f0a docs: mark phase 01.1 task 1 complete
53efb78 docs: record task 1 adherence confirmation
6118322 fix: address review feedback for build script
33fd265 docs: record task 1 simplification plan
92880c7 feat: add linux release build script
ddfd542 docs: record task 1 build script plan
f9a6b0e chore: bump version to 0.3.0
3b4922a docs: record phase 01.1 global plan
17c2177 docs: record Linux build artifact phase planning
```
