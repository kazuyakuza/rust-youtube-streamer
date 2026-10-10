# Implementation Plan — Phase 01.1, Task 1: Implement a Release Build Script

- Critical Workflow step: 4.1b (Implementation Plan) for TODO Task 1
- Source TODO: `.agent/todos/20261009/20261009-todo-4.md` → `### 1. Implement a Release Build Script`
- Global plan: `.kilo/plans/20261009-phase-01-1-build-artifact.md` (Task 1 pre-analysis, section "Per-Task Pre-Analysis")
- Plan date: 2026-10-09 — repo: `C:\repo\rust-youtube-streamer`, branch `feat/linux-release-build-artifact`, HEAD `f9a6b0e`
- Front-end related: **NO** — 4.1a/4.5a skipped
- Plan consumer: implementer sub-agent (JUNIOR, under 50% restriction), executing sub-step 4.2 (and the Task-1-scoped parts of 4.3–4.6), exactly ONE cycle, no other TODO tasks.

---

## 0. Verified repository facts (observed 2026-10-09, HEAD `f9a6b0e`)

These are confirmed against the live tree; the implementer must treat them as given:

- `Cargo.toml` v0.3.0 edition 2021; a `[[bin]]` section exists with `name = "rust-youtube-streamer-service"`, `path = "src/main.rs"` → the compiled binary name is `rust-youtube-streamer-service` in both cases. **Do not modify `Cargo.toml`.** (Note: the caller/global-plan context said "no `[[bin]]` override"; the live file now has one, but with the identical name — artifact name is unaffected. This is recorded here so nobody is surprised.)
- `scripts/dev-checks.sh` is tracked at git mode `100644` (no exec bit); the standard invocation runs it as `sh scripts/dev-checks.sh`. `build-linux.sh` must match this style and mode (plain `100644`, invoked via `sh`).
- `docker-compose.yml`: service `rust`, `working_dir: /rust-youtube-streamer`, bind mount `/rust-youtube-streamer:/rust-youtube-streamer`, named volume `rust-streamer-target:/rust-streamer-target`, env `CARGO_TARGET_DIR=/rust-streamer-target`.
- Expected release artifact inside the container: `$CARGO_TARGET_DIR/release/rust-youtube-streamer-service` → concretely `/rust-streamer-target/release/rust-youtube-streamer-service`.
- Compose image already built on the VM (Phase 00/01 runs used it). Documented fallback if missing: `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust`.
- `.gitignore` line 34 contains `dist/` (generic build-artifacts section). **Task 2 owns ignore-rule confirmation; this plan and the implementer must NOT change `.gitignore`.**
- `.agent/project-structure.md` still shows Cargo v0.2.0 — pre-existing staleness; Task 3 owns the structure-doc update. Do not touch it in Task 1.
- MCP: `alpine-vm` tools are allowlist-prefixed (`docker`, `sh`, `cat`, `ls`, …). Cargo stderr may be partially missing in captured output — **exit status is authoritative** (also documented in README).
- Standard Compose invocation precedent: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <command>`.

## 1. Residual ambiguities — resolved and FROZEN

Each row resolves a point the TODO leaves open. The implementer gets NO choice; these are decisions:

| # | Question | FROZEN decision | Rationale |
| --- | --- | --- | --- |
| R1 | Guards style: `set -e` vs explicit per-command checks? | **Explicit per-command checks only. No `set -e`, no `set -u`, no `set -o pipefail`.** Every failure branch is an explicit `if ! <predicate>; then fail_with_message ...` block ending in `exit 1`. | `set -e` semantics vary across POSIX shells and would make individual failure branches and their specific messages untestable. `scripts/dev-checks.sh` also uses explicit checks; this maximizes consistency. |
| R2 | How is the cwd requirement validated and what is the contract? | **Exact physical-path equality**: `[ "$(pwd -P)" = "/rust-youtube-streamer" ]`. No `Cargo.toml`-existence check, no prefix/substring match, no subdirectory allowance. | Compose sets `working_dir: /rust-youtube-streamer`, so the documented contract is one exact path. Prefix checks would permit subdirectories where relative `dist/` and cargo behavior silently change. Matches the TODO wording "the documented Compose working directory" and the `dev-checks.sh` header contract. |
| R3 | When is `dist/` created? | **Only after a successful `cargo build --release --locked`**, immediately before the copy: `mkdir -p dist` sits between the binary-locating check and the copy. | TODO: "After a successful build, copy … Create `dist/` if needed." Creating it after build success guarantees no empty `dist/` appears when the build fails (failing runs must leave nothing). |
| R4 | `CARGO_TARGET_DIR` handling | **Read from the environment, require set AND non-empty; fail otherwise. No default, no fallback, no value validation beyond non-empty, no normalization.** | TODO: "Resolve paths from … current `CARGO_TARGET_DIR` … fail with a clear error if required environment/paths … missing", plus the standing rule that no `target/` may appear in the project root (a default would create one). |
| R5 | Message channel split | **Progress and success result messages → stdout. All error messages → stderr**, formatted `build-linux: error: <message>`. Cargo output passes through untouched (no suppression, no redirection of its channels). | Conventional split; makes the success picture unambiguous, and per MCP caveat success is anyway proven by `exitCode` = 0 plus the stdout result line. |
| R6 | Exact message wording | **Frozen verbatim in §2** (start, per-branch errors, two-line result). Implementer must not paraphrase. | Junior-implementer plan; determinism required for Task 4's factual report. |
| R7 | "Missing or empty" predicates | **Use a single named helper `path_is_non_empty_file` (`[ -s "$1" ]`) for (a) the expected release binary before copy and (b) the final artifact after copy.** | `test -s` covers both existence and non-emptiness in one POSIX test and keeps the single-section-boolean rule intact. |
| R8 | Exit codes | **`exit 1` for every failure branch; explicit `exit 0` at the end.** (No per-branch distinct codes.) | Mirrors `dev-checks.sh` aggregate style (0/1); keeps verification matrix and docs simple. |
| R9 | Does the script write any other file (logs, markers)? | **No. It writes exactly two filesystem effects: `mkdir -p dist` and the copy into `dist/rust-youtube-streamer-service` (overwriting `dist/rust-youtube-streamer-service` only, on rerun). No log files, no temp files.** | TODO: "write logs elsewhere" forbidden; only the requested artifact may be written under `dist/`; rerun = overwrite semantics (what Task 3 will document as "rebuild/overwrite behavior"). |
| R10 | Size display in the result message | **Print byte count via `wc -c < "$artifact_path"` at result time; numbers are informational, not a correctness criterion.** | Task 4's report wants the actual artifact size; the observed count feeds it without extra tooling. |
| R11 | Executable bit / chmod | **No chmod, no exec bit (tracked at `100644`, like `dev-checks.sh`); documented invocation is `sh scripts/build-linux.sh`.** | Matches the existing script convention exactly; avoids Windows/`core.fileMode` churn. |
| R12 | Where do verification facts live? | **The implementer records exact commands + exit statuses + observed lines ONLY in its returned completion summary** (and later in Task 4's report). Task 1 must NOT edit `context.md`, `project-structure.md`, README, `docs/`, or `.agent/reports/`. | Scope boundary per global plan mapping and TODO Task 3/4 ownership. |

## 2. Script architecture (FROZEN)

File: `scripts/build-linux.sh` — a single small POSIX `sh` file, structure fixed as follows:

1. **Shebang + header comment** (4 lines, stating the runtime contract — mirrors `dev-checks.sh`).
2. **Constants block**: `expected_project_directory="/rust-youtube-streamer"`, `release_binary_name="rust-youtube-streamer-service"`, `artifact_path="dist/rust-youtube-streamer-service"`. (The copy target directory `dist` is derived once as `artifact_directory="dist"`.)
3. **Helper functions (all ≤ 2 params, ≤ a few lines, self-documenting names, no state)**:
   - `print_message` — `printf '%s\n' "$1"` (stdout).
   - `fail_with_message` — `printf '%s\n' "build-linux: error: $1" >&2; exit 1`.
   - `current_directory_matches_project_root` — the frozen cwd predicate (R2).
   - `cargo_target_directory_is_set` — `[ -n "$CARGO_TARGET_DIR" ]` (R4).
   - `path_is_non_empty_file` — `[ -s "$1" ]` (R7).
4. **Linear top-level flow** (mirrors `dev-checks.sh`'s linear style), in this exact order:
   1. cwd check → fail branch E1.
   2. `CARGO_TARGET_DIR` check → fail branch E2.
   3. Compute `release_binary_path="$CARGO_TARGET_DIR/release/$release_binary_name"` (quoted expansion).
   4. Start message (M1) → `cargo build --release --locked` → capture `$?` immediately → branch E3 on non-zero.
   5. Build-success message (M2).
   6. `path_is_non_empty_file "$release_binary_path"` check → fail branch E4.
   7. `mkdir -p "$artifact_directory"` → fail branch E5 (R3: `dist/` is created only here).
   8. `cp "$release_binary_path" "$artifact_path"` → fail branch E6.
   9. `path_is_non_empty_file "$artifact_path"` check → fail branch E7.
   10. Two-line result message (M3 + M4) → explicit `exit 0`.
5. Rules applied: single-section booleans (each multi-part condition is a named helper), max-depth 2, functions ≤ 50 lines (each is ≤ 6), file ≤ 200 lines (expect ≈ 60), no commented-out code, no magic numbers, POSIX-only constructs (`[ ]`, `$()`, `printf` — no `echo -e`, no arrays, no `local` at top level needed).

### Frozen message texts (verbatim)

- M1 (start, stdout): `Building Linux release executable using the tracked lockfile: cargo build --release --locked`
- M2 (after build, stdout): `Release build succeeded.`
- M3 (result, stdout): `Build succeeded: saved the Linux release executable to $artifact_path (<N> bytes).`
- M4 (result, stdout): `The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe.`
- E1 (stderr): `expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: <pwd -P output>`
- E2 (stderr): `CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)`
- E3 (stderr): `cargo build --release --locked failed with exit code <n>; cargo diagnostics above are not suppressed`
- E4 (stderr): `expected release binary not found or empty: <release_binary_path>`
- E5 (stderr): `could not create the dist/ directory: dist`
- E6 (stderr): `failed to copy <release_binary_path> to <artifact_path>`
- E7 (stderr): `final artifact is missing or empty after the copy: <artifact_path>`

### Full final script content (FROZEN — implementer reproduces this file; layout may adjust whitespace only)

```sh
#!/bin/sh
# Linux release build script (POSIX sh). Runs inside the Compose "rust" service with
# the working directory at /rust-youtube-streamer and CARGO_TARGET_DIR=/rust-streamer-target,
# as docker-compose.yml provides. Builds with the tracked lockfile and copies only the
# final executable into the gitignored root-level dist/ directory; intermediate Cargo
# output stays in the named Docker volume and Cargo diagnostics are passed through.

expected_project_directory="/rust-youtube-streamer"
release_binary_name="rust-youtube-streamer-service"
artifact_directory="dist"
artifact_path="dist/rust-youtube-streamer-service"

print_message() {
    printf '%s\n' "$1"
}

fail_with_message() {
    printf '%s\n' "build-linux: error: $1" >&2
    exit 1
}

current_directory_matches_project_root() {
    [ "$(pwd -P)" = "$expected_project_directory" ]
}

cargo_target_directory_is_set() {
    [ -n "$CARGO_TARGET_DIR" ]
}

path_is_non_empty_file() {
    [ -s "$1" ]
}

if ! current_directory_matches_project_root; then
    fail_with_message "expected the current working directory to be $expected_project_directory (the documented Compose working directory); got: $(pwd -P)"
fi

if ! cargo_target_directory_is_set; then
    fail_with_message "CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)"
fi

release_binary_path="$CARGO_TARGET_DIR/release/$release_binary_name"

print_message "Building Linux release executable using the tracked lockfile: cargo build --release --locked"

cargo build --release --locked
build_exit_code=$?
if [ "$build_exit_code" -ne 0 ]; then
    fail_with_message "cargo build --release --locked failed with exit code $build_exit_code; cargo diagnostics above are not suppressed"
fi

print_message "Release build succeeded."

if ! path_is_non_empty_file "$release_binary_path"; then
    fail_with_message "expected release binary not found or empty: $release_binary_path"
fi

if ! mkdir -p "$artifact_directory"; then
    fail_with_message "could not create the dist/ directory: $artifact_directory"
fi

if ! cp "$release_binary_path" "$artifact_path"; then
    fail_with_message "failed to copy $release_binary_path to $artifact_path"
fi

if ! path_is_non_empty_file "$artifact_path"; then
    fail_with_message "final artifact is missing or empty after the copy: $artifact_path"
fi

print_message "Build succeeded: saved the Linux release executable to $artifact_path ($(wc -c < "$artifact_path") bytes)."
print_message "The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe."
exit 0
```

### Failure-branch matrix (authoritative)

| Branch | Trigger (observed when) | Effect | Exit |
| --- | --- | --- | --- |
| E1 | `pwd -P` ≠ `/rust-youtube-streamer` | Nothing written (no build, no `dist/`) | 1 |
| E2 | `CARGO_TARGET_DIR` unset or empty | Nothing written | 1 |
| E3 | `cargo build --release --locked` non-zero | Cargo output shown as-is; nothing written | 1 |
| E4 | expected release binary missing or empty (build "succeeded" but no binary — unexpected toolchain change) | `dist/` NOT created (M2 already printed; E4 follows) | 1 |
| E5 | `mkdir -p dist` fails | Nothing copied | 1 |
| E6 | `cp` fails | `dist/` may exist (empty) — acceptable; no success message was printed | 1 |
| E7 | destination missing/empty after `cp` (copy partially failed) | No success message | 1 |
| OK | all checks pass | `dist/rust-youtube-streamer-service` non-empty; M3+M4 on stdout | 0 |

Note on Run E: the script is invoked by absolute path after `cd /tmp`, so the E1 cwd check is what rejects the run; nothing in the mount is touched.

## 3. Implementation steps for the implementer (sub-step 4.2)

Execute in order. Check the plan between steps.

**Git/branch precheck (no branch/checkout allowed — step 2 is already complete):**

1. Run `git status --porcelain` (repo root). Expected: empty. If not empty — STOP and report to caller (never stage unrelated files) — do not commit or stash anything.
2. Run `git log --oneline -1`. Expected `f9a6b0e` on top. If different — STOP and report.

**Code writing:**

3. Create `scripts/build-linux.sh` with exactly the frozen content of §2 (mode `100644`, like `dev-checks.sh`; on Windows no chmod needed — plain new-file creation is enough; ensure LF line endings and a trailing newline).
4. Self-review the file against §2's architecture bullet list (shebang present; no `set` calls; constants block; five helpers; linear flow; all seven fail branches precede the two result lines; final `exit 0`; no commented-out code; no comments other than the header).

**Verification (Alpine VM via MCP — run A first, then runs B–F per §4):** record every result per §4's record template before committing.

**Git commit:**

5. Run `git status --porcelain`. Expected: exactly one entry, `?? scripts/build-linux.sh`. (`dist/` must NOT appear — it is gitignored; if it appears, STOP and report.) Optionally `git check-ignore -v dist/rust-youtube-streamer-service` → expect a `.gitignore` line matching `dist/` (informational only; do not edit `.gitignore`).
6. Stage only the script: `git add scripts/build-linux.sh`. Then `git status` — verify nothing under `dist/`, `logs/`, or `target/` is staged (Gitignore Compliance Rule).
7. Commit with exactly: `feat: add linux release build script`

**Post-verification recheck:**

8. After §4's runs, `git status --porcelain` must again be empty (untracked ignored artifacts never show). If anything unexpected appears — STOP and report.

## 4. Verification matrix (run in sub-step 4.2 by the implementer)

MCP sequence rule: call `alpine-vm_vm_status` first (every session), then `alpine-vm_vm_run_command` for each command. All commands below start with the allowlisted `docker` prefix. Success evidence = recorded `exitCode`; stdout/stderr text is supporting evidence only (cargo stderr may be partially missing in capture).

### Run A — success path (the acceptance run)

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`
- `timeoutMs`: `900000` (first release-profile compile of dependencies happens inside the named volume; generous ceiling).
- Expected: `exitCode` = 0; stdout contains M1, M2, M3 (with a byte count > 0), M4; cargo output interspersed or partially missing (does not judge).
- If exit ≠ 0: STOP. Do not silently modify the script or invent a fix (workflow error-handling; pause for caller intervention). If the commit (§3 step 7) has not happened yet, leave the file as-is for caller review; if it has already happened, report the exact output and exit status and pause.
- If `docker compose` reports the image missing, first run `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust` (record it), then retry Run A once.

### Run B — artifact presence (host-visible)

- Command: `ls -l /rust-youtube-streamer/dist`
- `timeoutMs`: `30000`
- Expected: `exitCode` = 0; output lists `rust-youtube-streamer-service` with a byte size matching M3's count (the file is untouched between Runs A and B, so the two sizes must agree). This proves the artifact landed at the repo-root `dist/` visible on the Windows-host share, not only inside the container.

### Run C — no stray root `target/`

- Command: `ls /rust-youtube-streamer`
- `timeoutMs`: `30000`
- Expected: `exitCode` = 0; **no `target` entry** in the listing (and no new generated build files beyond the pre-existing set). Acceptance criterion "Intermediate Cargo output remains under the named volume; no `target/` in the project root."

### Run D — failure path: empty `CARGO_TARGET_DIR` (proves E2 → exit 1, artifact untouched)

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e CARGO_TARGET_DIR= rust sh scripts/build-linux.sh`
- `timeoutMs`: `60000`
- Expected: `exitCode` = 1; stderr contains the E2 message prefixed `build-linux: error:`; stdout contains **no** M2/M3/M4; no `dist/` is created (verify optional: `ls /rust-youtube-streamer/dist` still lists only the existing artifact).
- If compose rejects the empty `-e VAR=` syntax, retry once with quoted `--rm -e "CARGO_TARGET_DIR="`; do not use any other mechanism.

### Run E — failure path: wrong cwd (proves E1 → exit 1, mount untouched)

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cd /tmp && sh /rust-youtube-streamer/scripts/build-linux.sh'`
- `timeoutMs`: `60000`
- Expected: `exitCode` = 1; stderr contains the E1 message (with `got: /tmp`); nothing is built or copied (cwd `/tmp` is outside the bind mount).

### Run F — artifact intactness after failure paths

- Command: `ls -l /rust-youtube-streamer/dist`
- `timeoutMs`: `30000`
- Expected: `exitCode` = 0; the Run B artifact is still listed with its original size (runs D/E must not delete or modify it).
  - If the listed byte count differs, report the diff (D must not even touch dist/; E doesn't reach the copy stage).

### Record template (mandatory output for the returned completion summary; Task 4 consumes this verbatim)

For each run A–F: `command` (exact), `exitCode` (exact), `observed stdout lines` (verbatim, incl. M1–M4 / `build-linux: error:` lines), `observed stderr` (verbatim, may be partial — note partial capture as partial — do not fabricate missing cargo lines), bytecode size from Run B (only if observed). Nothing may be invented; anything not observed is recorded as "not observed (MCP capture) — exit status authoritative".

## 5. Git handling (FROZEN)

- Branch: current `feat/linux-release-build-artifact` only. No branch creation, no checkout, no merge (steps 2/5 of the workflow own those).
- Commits in Task 1 (in order):
  1. `feat: add linux release build script` — the single implementation commit (only `scripts/build-linux.sh`).
  2. (Conditional 4.3 fix cycle) If code-reviewer/code-simplifier fixes follow, one commit `fix: address review feedback for build script` (adjust wording only if a distinct concern needs a separate commit; same file only).
  3. (4.6 completion) `docs: mark phase 01.1 task 1 complete` — the `[DONE]` mark on `### 1.` in the TODO file only.
- No push (git-remote-safety: push is restricted to step 5 of the Critical Workflow).
- `.gitignore` untouched; artifact never staged.

## 6. Workflow-cycle notes for Task 1's remaining sub-steps

- **4.3 (review/simplification)**: reviewers check this file against §2's frozen architecture, POSIX-only constructs, message list, and failure-matrix completeness. Reviewers must not introduce `set -e`, frameworks, or extra files. Simplification outcomes go to a `.kilo/plans/<date>-*.md` fix plan only if fixes are actually needed; otherwise "not required" (existing-file churn is not a simplification goal).
- **4.4 (documentation)**: the script's header comment is already part of the frozen content (§2) and satisfies this step for Task 1. README / `docs/build.md` / `.agent/project-structure.md` / `context.md` updates are **Task 3 scope** — do not touch them here. Expected outcome of 4.4 for Task 1: confirmation that the header comment documents the runtime contract; no file changes needed beyond the script itself unless review feedback requires a comment correction (script file only).
- **4.5b (plan adherence)**: compare the implementation against §2 (architecture + full content), §4 (runs A–F executed, recorded correctly), §5 (commit history exactly as listed), and §8 (acceptance cross-check).
- **4.6 (completion)**: append only ` [DONE]` to the `### 1. Implement a Release Build Script` heading in `.agent/todos/20261009/20261009-todo-4.md`; preserve every other byte of the file (Overwrite-TODO-File-Prevention rule). Commit per §5 item 3.

## 7. Hard scope boundaries ("do not do" list)

- Do NOT create or modify anything outside `scripts/build-linux.sh` and the TODO file's single heading (4.6). Specifically forbidden: `.gitignore`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `docker-compose.yml`, `README.md`, `docs/`, `.agent/project-structure.md`, `.agent/project-info/*`, `.agent/reports/*`, `src/**`.
- Do NOT run `cargo fmt` (any mode), `cargo clippy`, or `cargo test` from within this task; `scripts/dev-checks.sh` on the final tree belongs to Task 4.
- Do NOT delete or prune anything: no `dist/` cleanup, no Docker volume/prune/registry actions, no removal of unrelated files.
- Do NOT write files outside the approved artifact path: no logs, no temp dirs, nothing under the repo root besides `dist/rust-youtube-streamer-service`.
- Do NOT push, merge, rename the TODO file, or mark other tasks/acceptance boxes.
- Do NOT invent verification numbers; unobserved = "not observed (MCP capture), exit status authoritative".
- Do NOT add dependencies, frameworks (shellcheck, bats), aliases, or NPM-script wrappers.

## 8. Cross-check against TODO Task 1 (point-by-point)

| TODO Task 1 requirement | Plan coverage |
| --- | --- |
| `scripts/build-linux.sh`, POSIX `sh`, runs in Compose `rust` service, cwd `/rust-youtube-streamer` | §0 (facts), §2 (shebang + cwd contract), §5 (mode/invocation) |
| `cargo build --release --locked` (tracked lockfile) | §2 flow item 4 (exact command) |
| Copy from `$CARGO_TARGET_DIR/release/rust-youtube-streamer-service` into `dist/rust-youtube-streamer-service`; resolve paths from documented cwd + current `CARGO_TARGET_DIR`; fail clearly when env/paths/exe missing | §2 flow items 2–3, 6–10 (R2, R4, R7; branches E2, E4) |
| `mkdir -p dist`; only `dist/rust-youtube-streamer-service` written under root; intermediates stay in named volume | R3 (mkdir-after-build-only), R9 (write exactly one path), Run C |
| Fail non-zero on build or copy failure; no success before final file exists and is non-empty | R8; R7 helper; branches E3–E7; §2 flow item 9 (final `path_is_non_empty_file` before M3/M4) |
| Start/result messages + exact output path; no cargo diagnostics suppressed | R5, R6 (M1–M4 verbatim; cargo untouched) |
| No source modification, no auto-`cargo fmt`, no logs elsewhere, no `dist/` deletion | §7 |
| Maintainable, consistent with shell-script conventions, no framework/dependency | §2 (dev-checks conventions: POSIX, plain `printf`, explicit exits, mode `100644`) |

Acceptance criteria relevant to Task 1 (checked later, not now): AC1 (script + flags), AC2 (single-copy + non-zero failures), AC3 (no `target/` in root), AC5 (Docker/MCP success on real tree — Runs A–F provide the evidence), AC7–8 (Task 3), AC9–10 (Tasks 3–4). All Task-1-scoped evidence columns map to §4's record template.

## 9. Return to caller (architector summary)

- Decision set: R1–R12 (R1–R9, R10, R11, R12 as tabled above).
- Approach: explicit per-command checks, no `set -e`; exact-path cwd match; env-var must be set; mkdir-after-build; single `cp`; double `test -s` (source + destination); `exit 1` per branch / `exit 0` explicit; stdout for progress, stderr for `build-linux: error:`.
- Verification: runs A (success) + B (host-visible artifact) + C (no root `target/`) + D (empty-var failure) + E (wrong-cwd failure) + F (artifact intactness) with the record template in §4.
- Commits: `feat: add linux release build script`; conditional `fix:` per review; completion `docs: mark phase 01.1 task 1 complete`; no push.
