# Simplification Plan — Phase 01.1, Task 1: `scripts/build-linux.sh`

- Critical Workflow step: 4.3 (Code Simplifier review output) for TODO Task 1
- Source TODO: `.agent/todos/20261009/20261009-todo-4.md` → `### 1. Implement a Release Build Script`
- Frozen contract (authoritative): `.kilo/plans/20261009-phase-01-1-task1-build-script.md` (R1–R12, E1–E7, M1–M4, §5 git handling)
- Reviewed file: `scripts/build-linux.sh` (72 lines, created at commit `92880c7`; content verified byte-faithful to the frozen §2 script)
- Review date: 2026-10-09 — branch `feat/linux-release-build-artifact`, clean tree
- Plan consumer: implementer sub-agent (JUNIOR, under 50% restriction) in sub-step 4.3-fix. Execute exactly the single edit below. NO judgment calls, NO other changes.
- Front-end related: **NO** — 4.5a skipped.

## 1. Review verdict

The script is already minimal and mirrors the frozen architecture. Exactly ONE behavior-preserving, clearly beneficial simplification was found (S1 below): it removes two duplicated string literals that are a latent divergence bug. Everything else touches the frozen contract and is explicitly OUT of simplification scope (§4).

## 2. The single approved edit (S1)

**Problem:** `artifact_path` hardcodes the literals already defined by the constants above it:

```sh
expected_project_directory="/rust-youtube-streamer"
release_binary_name="rust-youtube-streamer-service"
artifact_directory="dist"
artifact_path="dist/rust-youtube-streamer-service"
```

If `artifact_directory` or `release_binary_name` is ever changed, `artifact_path` silently diverges from them (copy target, E6/E7 messages, and M3 would point elsewhere). Derive it instead — single source of truth.

**Exact edit** — in `scripts/build-linux.sh`, replace the single line

```sh
artifact_path="dist/rust-youtube-streamer-service"
```

with

```sh
artifact_path="$artifact_directory/$release_binary_name"
```

- Location: the 4th line of the constants block (currently line 11), immediately after `artifact_directory="dist"` — both referenced variables are already assigned on the lines above, so the expansion is valid POSIX `sh` evaluation order.
- Touch nothing else in the file: not the header comment, not the helpers, not any flow line, not spacing elsewhere. Whitespace/indentation of the replacement line identical to the original (no leading spaces).
- Ensure LF line endings and the file's trailing newline are preserved.
- Keep git mode `100644` (no exec-bit change; do not chmod).

## 3. Why behavior is provably unchanged

- The expanded string is byte-identical: `dist` + `/` + `rust-youtube-streamer-service` = `dist/rust-youtube-streamer-service`. Every message that interpolates `$artifact_path` (E6, E7, M3) and the actual `cp` destination are unchanged, verbatim per frozen R6.
- Exit codes, exit ordering, on-disk effects (E1–E7 failure matrix, R8/R9/R3): untouched — the edit is confined to a constant definition evaluated before any branch runs.
- Messages M1–M4 and E1–E7 texts: no change of any kind.
- POSIX-only constructs maintained (double-quoted variable expansion).

## 4. Rejected candidates (do NOT implement)

Each deviation from the frozen contract is out of scope for this cycle; recorded so the implementer and 4.5b know they were considered:

1. **Removing/inlining helpers** (`cargo_target_directory_is_set`, `path_is_non_empty_file`, etc. are each used 1–2 times): the five-helper list is frozen in plan §2 (R7 and the single-section boolean rule justify them). Rejected.
2. **Dropping the double `test -s` (E4 + E7)**: explicitly frozen (R7, "double test -s verification"). Rejected.
3. **Renaming to `dev-checks.sh` idioms** (`emit`, `run_check_*`): `build-linux.sh`'s helper names are frozen verbatim in §2; `dev-checks.sh`'s `emit` also writes to a log file (different semantics). Renaming would be a contract deviation without visible consistency gain. Rejected.
4. **Caching `pwd -P`** (E1 calls it twice): the frozen §2 requires stateless helpers and verbatim E1 text; behavior is deterministic within one process. Rejected.
5. **Any structural change** (`set -e`, frameworks, reordering, added files): forbidden by R1 and plan §7/§6. Rejected.

## 5. Implementer procedure (4.3-fix, in order, atomic)

1. Precheck: `git status --porcelain` at repo root → expected empty; `git log --oneline -1` → expected HEAD is the Task 1 implementation commit chain on `feat/linux-release-build-artifact` (contains `92880c7 feat: add linux release build script`). If either differs from expectations — STOP and report to caller.
2. Apply the §2 edit exactly.
3. Self-check the file: constants block still 4 lines; only difference vs. the pre-edit file is line 11; no other line modified (verify with `git diff` — the diff must show exactly one removed and one added line in the constants block).
4. Verification via Alpine VM MCP (this is the minimum re-verification set for this simplification cycle; MCP sequence rule: `alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`; exit codes are authoritative):
   - **Success run** — `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` (timeoutMs 900000). Expected: `exitCode` 0; stdout contains M1, M2, M3, M4; the M3 artifact path renders exactly `dist/rust-youtube-streamer-service` with a byte count > 0. Then `ls -l /rust-youtube-streamer/dist` (timeoutMs 30000) — expected: `rust-youtube-streamer-service` listed, size matching M3.
   - **Failure run (E2)** — `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e CARGO_TARGET_DIR= rust sh scripts/build-linux.sh` (timeoutMs 60000). Expected: `exitCode` 1; stderr contains the verbatim E2 message prefixed `build-linux: error:`; stdout has no M2/M3/M4; the prior artifact untouched. (The E2 guard runs after the edited constant line, so this proves the script still parses and guards intact; per the caller minimum of one success + one failure run this suffices — no need to re-run the full A–F matrix.)
   - Record exact commands, `exitCode`s, and observed message lines in the returned completion summary. Nothing may be invented; unobserved output is recorded as "not observed (MCP capture) — exit status authoritative".
   - If either run deviates from expectations: STOP, report exact evidence to caller. Do not improvise a second fix.
5. Git: `git status --porcelain` → expected exactly ` M scripts/build-linux.sh` (nothing under `dist/`, `logs/`, `target/`). Stage only that file. Commit with message exactly:

   `fix: address review feedback for build script`

   Note: this message is verbatim from the frozen Task 1 plan §5 item 2 (the designated 4.3-fix commit for this file). The caller's draft `refactor: simplify linux build script` was NOT used because §5 is FROZEN; flag to caller if they insist on the alternative.
6. Post-commit: `git status --porcelain` → expected empty. No push, no branch/checkout operations, no TODO marking (owned by 4.6).

## 6. Hard boundaries for this cycle

- Only `scripts/build-linux.sh` may change; only the single line of §2.
- Forbidden: `.gitignore`, `Cargo.toml`, `Dockerfile`, `docker-compose.yml`, `docs/`, `README.md`, `.agent/**`, `src/**`, `dev-checks.sh`, new files, deleted files.
- No `cargo fmt`/`clippy`/`test`, no Docker prune/volume actions, no deletion in `dist/`.

## 7. Return to caller (simplifier summary)

- One edit approved (S1: derive `artifact_path` from `artifact_directory` + `release_binary_name`; kills duplicated literals, latent-divergence fix; byte-identical behavior).
- Five candidate classes rejected as frozen-contract deviations (documented in §4).
- Re-verification contract for 4.3-fix: one success run + one E2 failure run via MCP (§5 step 4), single commit `fix: address review feedback for build script` on the current branch.
