# Task 2 Implementation Plan — `scripts/build-windows.sh` + Safe Windows Artifact Production

> **PLANNER ADDENDUM (2026-10-10, post-EOL incident — binding, read before any other step).**
> A working-tree incident was found and repaired before this task: the whole repo worktree currently carries CR characters (raster: every text file except `scripts/dev-checks.sh` and the `.gitkeep`s shows CR); `sh` inside the container rejects CRLF scripts (`sh scripts/build-linux.sh` probed exit 2). The repair normalized `scripts/build-linux.sh` + `Dockerfile` worktree bytes to LF (index blobs were already LF; no commit was needed). Root cause is the repo-host git environment: `core.autocrlf=true` re-materializes text files as CRLF on checkout.
> Therefore this task's implementation is amended with the following sub-steps (do NOT re-ask; encode as mandatory):
> 1. Create `scripts/build-windows.sh` with the LF content exactly as specified further below; after writing, verify no `\r` bytes (container `od -c`, e.g. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "od -c /rust-youtube-streamer/scripts/build-windows.sh | head -3"`) and normalize via container-side `tr -d '\r'` + `mv` if needed; only then run it.
> 2. Create a NEW tracked file `.gitattributes` with EXACTLY this content (LF endings):
>    ```
>    # Force LF line endings for container-executed files; core.autocrlf=true on the Windows host would re-materialize them as CRLF on any future checkout
>    *.sh text eol=lf
>    Dockerfile text eol=lf
>    ```
>    (No other attributes file content; do not add line-ending rules for other file types.)
> 3. Include `.gitattributes` in this task's commit alongside `scripts/build-windows.sh` (commit message unchanged: `feat: add windows release build script`).
> 4. After committing, verify: `git ls-files --eol scripts/build-windows.sh` shows `i/lf w/lf`; container `od` still shows zero `\r`.
> This addendum is recorded by the Planner; Markdown was updated per `.kilo/rules/markdown-generation-rule.md` (plans are planner-owned).

- Source TODO: `.agent/todos/20261010/20261010-todo-1.md`, section `### 2. Produce a Windows Artifact Safely` — read it first; all 8 bullets are binding.
- Global plan: `.kilo/plans/20261010-phase-01-2-windows-cross-build.md` (binding technical decisions #3, #4, #5, #6).
- Scope: **Task 2, sub-step 4.1b → this plan file only** (execution is sub-step 4.2, by the implementer). NOT Tasks 1/3/4, no doc/report/TODO-marker changes.
- Front-end: none → no sub-step 4.1a.

---

## Planning-time live pre-audit (executed read-only by the architector, 2026-10-10 — RESULTS ARE BINDING, do not re-derive)

### A. Import audit of the Task-1 Windows exe (Variant decision evidence)

1. First attempt on the VM host shell (`sh -c "x86_64-w64-mingw32-objdump ..."`) → exit 1, `sh: x86_64-w64-mingw32-objdump: not found` — the mingw tools exist only **inside the container image** (Task 1 installed them via apt in the image). Compliant invocation therefore starts with the `docker` allowlist prefix.
2. Working audit command (exit **0**):
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "x86_64-w64-mingw32-objdump -p /rust-streamer-target/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe | grep 'DLL Name'"
   ```
   Verbatim stdout (8 import lines):
   ```
   	DLL Name: kernel32.dll
   	DLL Name: KERNEL32.dll
   	DLL Name: msvcrt.dll
   	DLL Name: ntdll.dll
   	DLL Name: USERENV.dll
   	DLL Name: WS2_32.dll
   	DLL Name: api-ms-win-core-synch-l1-2-0.dll
   	DLL Name: bcryptprimitives.dll
   ```
3. Exe presence/size confirmed (exit 0): `/rust-streamer-target/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe` — `-rwxr-xr-x 2 root root 3191192` (≈ 3.19 MB; Nov-date Oct 10 16:40). The Task-1 probe artifact is real and is a valid PE (objdump parsed it).

### B. Import classification

| Observed import (case-normalized) | Class | Notes |
| --- | --- | --- |
| msvcrt.dll | Windows system DLL | shipped with every Windows release |
| kernel32.dll (observed twice: `kernel32.dll` + `KERNEL32.dll`) | Windows system DLL | case variants of the same import |
| ntdll.dll | Windows system DLL | loaded by every Windows process |
| userenv.dll (observed `USERENV.dll`) | Windows system DLL | user-profile/user-env API |
| ws2_32.dll (observed `WS2_32.dll`) | Windows system DLL | Winsock |
| api-ms-win-core-synch-l1-2-0.dll | Windows API Set (OS component) | loader-resolved virtual DLL; present on standard modern Windows (Win 8-era+, incl. all Win 10/11) |
| bcryptprimitives.dll | Windows system DLL (OS component) | CNG primitives; shipped on Windows 10/11 |

- `advapi32.dll` and `user32.dll` (named in the audit brief's example list) are **not** imported — absence is fine.
- **MinGW redistributable runtime DLLs: NONE observed** (`libgcc_s_seh-1.dll`, `libwinpthread-1.dll`, `libstdc++-6.dll` — all absent). The GCC runtime (exceptions unwinding, pthreads bits) is linked **statically** by rustc's self-contained windows-gnu linking (Task-1 route).

### C. Variant decision — **VARIANT A** (encoded; no runtime choice points)

All observed imports are standard Windows OS components; none is a MinGW/redistributable runtime DLL. Per the TODO policy (`prefer static-linking compatible GNU runtime components when supported and verified` — verified by the import audit) the script:

- copies **only** the `.exe` to `dist/windows/rust-youtube-streamer-service.exe` (no DLL copying, no FFmpeg bundling), and
- its success output explicitly states: GNU runtime statically linked → no MinGW runtime DLLs required, only standard Windows system DLLs (enumerates the observed list), **on the observed import-audit basis**, plus the mandatory caveat that cross-compilation does not prove native Windows runtime behavior and that FFmpeg stays an external prerequisite.

Variant B (copy DLLs, static remediation) is **rejected by evidence**: there is nothing to copy — no MinGW DLL is imported, so static remediation cannot change any observed import (nothing to remediate). **Do NOT re-run the audit decision at script runtime; the script contains no objdump/audit logic.** The audit result is a planning-time fact; Task 4 will re-cite it.

### D. Repo/VM baseline facts

- Branch `feat/windows-gnu-cross-build`, HEAD `d6e8f1b` (Task 1 committed: `e5505e1` + `d6e8f1b`); `git status --porcelain` → empty (clean).
- `Dockerfile` is the Task-1 7-line version (rust:1.82 + rustfmt/clippy + `gcc-mingw-w64-x86-64` + `rustup target add x86_64-pc-windows-gnu`).
- `dist/` currently contains exactly one entry: `rust-youtube-streamer-service` (Linux artifact, 1 715 296 bytes, `-rwxrwx--- root:vboxsf`). No `dist/windows/` yet, no other file under `dist/`.
- Repo root contains NO `target/` directory (named-volume redirection intact). Root has a `CHANGELOG.md` and `LICENSE` (tracked, unrelated — untouched by this task).
- No `.gitattributes` exists in the repo. `git config core.autocrlf` could not be read (host permission restriction) — **because of this the plan mandates hard LF verification of the new file from both host and VM side, never relying on git status alone.**

### E. CRITICAL pre-audit finding — CRLF breaks container `sh` (out-of-scope, reported to caller)

- Live evidence: `git ls-files --eol scripts/build-linux.sh` → `i/lf  w/crlf` (working copy has CRLF endings ON DISK); `scripts/dev-checks.sh` → `i/lf  w/lf`.
- The on-disk CRLF copy of `build-linux.sh` **fails inside the container** ("dash"): non-destructive probe `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cd /tmp && sh /rust-youtube-streamer/scripts/build-linux.sh"` → exit **2** with `7: 	: not found`, `12: 	: not found`, `15: }: not found`, `16: 	: not found`, `19: Syntax error: Bad fd number`. It dies before any cargo/copy side effect (no mutation happened; probe safe to repeat).
- This does NOT affect `dev-checks.sh` (LF on disk). The last successful `build-linux.sh` runs pre-date the CRLF corruption; `git status` shows the file **clean** because the diff normalizes EOLs — **git status cannot detect this class of fault**.
- **Caller decision required (NOT this task's action):** restore an LF working copy of `scripts/build-linux.sh` (e.g. rewrite it with LF content via a text tool that writes raw bytes) and consider a durable fix (`.gitattributes` with `*.sh text eol=lf`, or host git config change). Needed before Task 4 re-runs `sh scripts/build-linux.sh`. Do NOT fix it in this task.
- **Consequence encoded for the implementer:** the NEW file MUST be written with LF endings and MUST be EOL-verified from both the host (`git ls-files --eol`) and the VM (`cat -A`, no `^M$`), plus a container-side `sh -n` syntax check, BEFORE the real build run. CRLF is the #1 predicted failure mode of this step.

---

## Execution Environment Roles

- **HOST** = Windows implementation host, working directory `C:\repo\rust-youtube-streamer`. Only `git` (allowed patterns) and file creation/edit tools happen here. Never run cargo/rustup/docker on the host.
- **VM (MCP)** = Alpine VM via `alpine-vm_vm_status` (call before EVERY `alpine-vm_vm_run_command`) + `alpine-vm_vm_run_command` with allowlist prefixes `docker|sh|apk|ls|cat|ps|df|free|uname|pwd|whoami`. Compose commands always take the form `docker compose -f /rust-youtube-streamer/docker-compose.yml …`. Exit codes returned by `vm_run_command` are authoritative. Build-command `timeoutMs`: **900000**.

---

## Step 1 — HOST: create `scripts/build-windows.sh` (exact content, LF endings)

Create the file with EXACTLY this content (real LF newlines; no CR bytes anywhere; nothing else may be added):

```sh
#!/bin/sh
# Windows x86-64 GNU release build script (POSIX sh). Runs inside the Compose "rust"
# service with the working directory at /rust-youtube-streamer and
# CARGO_TARGET_DIR=/rust-streamer-target, as docker-compose.yml provides. Cross-compiles
# with the tracked lockfile and copies only the final .exe into the gitignored
# root-level dist/windows/ directory; intermediate Cargo output stays in the named
# Docker volume, Cargo diagnostics are passed through, and nothing is deleted.

expected_project_directory="/rust-youtube-streamer"
expected_target_triple="x86_64-pc-windows-gnu"
release_binary_name="rust-youtube-streamer-service.exe"
artifact_directory="dist/windows"
artifact_path="$artifact_directory/$release_binary_name"

print_message() {
    printf '%s\n' "$1"
}

fail_with_message() {
    printf '%s\n' "build-windows: error: $1" >&2
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

release_binary_path="$CARGO_TARGET_DIR/$expected_target_triple/release/$release_binary_name"

print_message "Building Windows x86-64 GNU release executable using the tracked lockfile: cargo build --release --locked --target $expected_target_triple"

cargo build --release --locked --target "$expected_target_triple"
build_exit_code=$?
if [ "$build_exit_code" -ne 0 ]; then
    fail_with_message "cargo build --release --locked --target $expected_target_triple failed with exit code $build_exit_code; the cargo diagnostics above are not suppressed"
fi

print_message "Release build succeeded."

if ! path_is_non_empty_file "$release_binary_path"; then
    fail_with_message "expected release executable not found or empty: $release_binary_path"
fi

if ! mkdir -p "$artifact_directory"; then
    fail_with_message "could not create the dist/windows/ directory: $artifact_directory"
fi

if ! cp "$release_binary_path" "$artifact_path"; then
    fail_with_message "failed to copy $release_binary_path to $artifact_path"
fi

if ! path_is_non_empty_file "$artifact_path"; then
    fail_with_message "final artifact is missing or empty after the copy: $artifact_path"
fi

print_message "Build succeeded: saved the Windows x86-64 GNU release executable to $artifact_path ($(wc -c < "$artifact_path") bytes)."
print_message "The GNU runtime is statically linked: the observed import audit (x86_64-w64-mingw32-objdump) found no MinGW runtime DLLs, and only standard Windows system DLLs are required (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll)."
print_message "Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled."
exit 0
```

Content rules the implementer must preserve:
- Function names, check order, and message wording mirror `scripts/build-linux.sh` 1:1, adapted only for the Windows target and the `.exe` file name (precondition wording identical; `build-windows: error:` prefix replaces `build-linux: error:`).
- The success output line 1 ends with `($(wc -c < "$artifact_path") bytes)` — the exact size printed by `wc -c` as in build-linux.sh.
- The DLL statement enumerates the audit-observed import list (lowercase-normalized; documented basis, see pre-audit §A–C). Never soften, never extend, never claim native-runtime proof.
- No objdump/audit logic, no rm/prune, no find/kill, no extra targets, no `2>/dev/null` or diagnostics suppression anywhere.
- File length ≈ 74 lines; functions are ≤ 3 lines; nesting ≤ 1 level (single-section conditions only).

## Step 2 — HOST: verify the file's EOL and git view

2.1 `git ls-files --eol scripts/build-windows.sh` → MUST print exactly `i/lf  w/lf	scripts/build-windows.sh`. If it prints `w/crlf` → rewrite the file with LF (same content) and re-check before anything else.
2.2 `git status --porcelain` → MUST be exactly `?? scripts/build-windows.sh` (gitignore-compliance rule satisfied: `.gitignore` read; nothing staged yet; nothing ignored staged).

## Step 3 — VM: hard EOL verification from the container's side

Call `alpine-vm_vm_status` first, then:

```
cat -A /rust-youtube-streamer/scripts/build-windows.sh
```

- Expected exit 0; output shows **no `^M$` line endings** (every line ends with plain `$`) and matches the content of Step 1 line-for-line.
- If any `^M` appears → the file on the shared folder is CRLF; fix on the host (rewrite with LF), re-verify Steps 2–3, and only then continue.
- STOP rule: never "fix" EOLs by editing inside the container; the host is the single source of the file.

## Step 4 — VM: container-side syntax check

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -n scripts/build-windows.sh
```

- Expected exit **0** with no output.
- Any non-zero/any output → STOP and report to the caller with the combined output; do not modify other files, do not "smart-fix" the script beyond matching Step 1's exact content.
- (`sh -n` parses only; it does not execute the build.)

## Step 5 — VM: the real build (documented standard command)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
```

- `timeoutMs`: `900000`.
- Expected exit **0** with success output (in this order):
  1. `Building Windows x86-64 GNU release executable using the tracked lockfile: cargo build --release --locked --target x86_64-pc-windows-gnu`
  2. (cargo's own progress lines on the warm cached second run these are short; first run compiles all deps for the new target — both outcomes are OK; `Finished release [optimized]` style lines may appear; exit code is authoritative)
  3. `Release build succeeded.`
  4. `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (<N> bytes).` — `<N>` is recorded verbatim for Tasks 3/4 (planning-time expectation ≈ 3 191 192; **record the actual number, never the expectation**).
  5. `The GNU runtime is statically linked: the observed import audit (x86_64-w64-mingw32-objdump) found no MinGW runtime DLLs, and only standard Windows system DLLs are required (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll).`
  6. `Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled.`
- On failure: capture `exitCode` + `combinedOutput` verbatim and STOP (report to caller). Preserve the failure message text; the failure paths are defined in the table below.

## Step 6 — VM: artifact + non-regression verification (read-only)

6.1 `ls -l /rust-youtube-streamer/dist/windows` → expected exit 0 and EXACTLY one entry: `rust-youtube-streamer-service.exe`, non-empty size equal to the printed `<N>`.
6.2 `ls -l /rust-youtube-streamer/dist` → expected exit 0 with EXACTLY two entries: the pre-existing `rust-youtube-streamer-service` (Linux artifact, unchanged size 1715296) and the directory `windows`. Nothing else. Anything else created under `dist/` is a regression → STOP and report.
6.3 `ls /rust-youtube-streamer` → expected: same listing as pre-audit §D — no `target` entry (Cargo intermediates stayed in the named volume).
6.4 Explanatory note (no action): the artifact lands mode `-rwxrwx---`/`root:vboxsf`-style on the shared mount, mirroring the Phase-01.1 Linux artifact — informational only.

## Step 7 — VM: failure-path validation probes (no edits, no side effects)

This step exercises the script's two failure preconditions exactly as encoded in Step 1 — both exit before any side effect (checks 1 and 2 run first; nothing is built, created, or copied):
- Doable ONLY via the container with cwd override in the same way the pre-audit probe ran:
  ```
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cd /tmp && sh /rust-youtube-streamer/scripts/build-windows.sh"
  ```
  Expected exit **1** with stderr `build-windows: error: expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: /tmp` (dash may render a trailing CR invisibly; text equality modulo that). No side effects (cwd check is first; nothing is built, created, or copied). Record as the Task-2 failure-path evidence.
- Empty `CARGO_TARGET_DIR` variant via compose `environment` override is NOT run in this step (it mutates compose invocation surface only temporarily):
  ```
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e CARGO_TARGET_DIR= rust sh -c "sh /rust-youtube-streamer/scripts/build-windows.sh"
  ```
  Expected exit **1** with stderr `build-windows: error: CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)`. Record as failure-path evidence. No side effects (check is second; the build hasn't started).
- Any other failure at this point → STOP and report; do not patch the script outside Step 1's content.

## Step 8 — HOST: gitignore/tracking evidence (read-only)

Run all three; record results:
1. `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` → expected output `.gitignore:34:dist/` (the generic `dist/` rule covers all depths).
2. `git ls-files dist/` → expected **empty** (nothing indexed under `dist/` even after windows/ was created).
3. `git status --porcelain --ignored=matching -- dist/` → expected exactly `!! dist/` (single ignored aggregate, no staged/untracked leakage).
4. `git status --porcelain` → expected exactly `?? scripts/build-windows.sh` (no other entries; if anything else appears → STOP, report).

## Step 9 — HOST: commit exactly one file

9.1 `git add scripts/build-windows.sh`
9.2 `git diff --cached --name-only` → expected exactly `scripts/build-windows.sh`. Nothing else may be staged.
9.3 `git commit -m "feat: add windows release build script"` → expected exit 0. Fixed message; no body, no scope suffix.
9.4 `git status --porcelain` → expected empty (clean tree). `.gitignore`-compliance recheck: nothing staged or committed matched `.gitignore` patterns.
9.5 **NEVER `git push`** in this step (push belongs to workflow step 5). Never edit TODO markers, docs reports, README, `docs/`, `.agent/project-structure.md`, `.agent/project-info/context.md` (Tasks 3/4 do that).

---

## Failure semantics table (from the script; all errors → stderr + exit 1)

| # | Trigger | Message (all prefixed `build-windows: error:`) | Exit |
| --- | --- | --- | --- |
| 1 | cwd ≠ `/rust-youtube-streamer` | `expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: <pwd>` | 1 |
| 2 | `CARGO_TARGET_DIR` unset/empty | `CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)` | 1 |
| 3 | `cargo build --release --locked --target x86_64-pc-windows-gnu` fails (compile error, link error, dlltool error, stale `Cargo.lock`, anything cargo-internal) | `cargo build --release --locked --target x86_64-pc-windows-gnu failed with exit code <N>; the cargo diagnostics above are not suppressed` | 1 |
| 4 | source exe missing/empty at `$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe` | `expected release executable not found or empty: <absolute path>` | 1 |
| 5 | `mkdir -p dist/windows` fails | `could not create the dist/windows/ directory: dist/windows` | 1 |
| 6 | `cp` fails | `failed to copy <source> to <dest>` | 1 |
| 7 | final artifact missing/empty post-copy | `final artifact is missing or empty after the copy: dist/windows/rust-youtube-streamer-service.exe` | 1 |
| 8 | success | four success lines above; exit 0 | 0 |

Implementation-level expectations (compose/dockers): exit codes are the script's; docker-level anomalies (125/126/127 → container start/exec issues) surface before the script runs and are reported verbatim, never silently improved.

---

## Explicitly NOT done in this step (belongs elsewhere)

- Task 3: `docs/build-windows.md`, README section, `.agent/project-structure.md`, `.agent/project-info/context.md` updates (three-workflow distinction, limitations text, toolchain/target choice explanation).
- Task 4: Task-2/4 verification runs + completion report + TODO marker/acceptance checkbox updates. This step records evidence in its execution notes only.
- CRLF fix for `scripts/build-linux.sh` (caller decision; see pre-audit §E).
- No Cargo.toml/Cargo.lock/`src/**` changes, no Docker/Compose changes, no `.gitattributes` creation here, no FFmpeg install/bundle, no MSVC, no extra Rust targets, no deletes/prunes anywhere, no version bump, no branch merge, no push.

## Verification checklist (mapped 1:1 to the TODO `### 2.` bullets)

- [ ] `scripts/build-windows.sh` exists as POSIX `sh` mirroring `build-linux.sh` validation/error conventions (Step 1 content; functional parity: same five helper functions, same precondition/verify/copy/report structure).
- [ ] Requires documented Compose cwd `/rust-youtube-streamer` and non-empty `CARGO_TARGET_DIR`; clear errors; non-zero on every precondition/Cargo/linker/copy/verify failure (Steps 1, 7; failure table).
- [ ] Builds with `--locked --target x86_64-pc-windows-gnu`, then copies only the expected exe from `$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe` to `dist/windows/rust-youtube-streamer-service.exe` (Steps 1, 5, 6).
- [ ] Source and final artifact verified existing and non-empty before success; exact artifact path + size printed (Steps 1, 5, 6).
- [ ] `dist/windows/` created when needed; nothing else written to repo root or `dist/` beyond the exe (Steps 6.2/6.3).
- [ ] No deletes/prunes; Linux artifact untouched (Steps 6.2 byte-size match 1715296 baseline; Steps 7 probe left nothing behind).
- [ ] GNU runtime DLL question answered with observed data, not claims: static GNU runtime, no MinGW DLLs, only standard Windows system DLLs ( Variant A decision §A–C; success line 5 in Step 5 ); statement anchored to the audit basis, never claiming native runtime validation (success line 6).
- [ ] Real exit-0 run of the documented container command produced the ready `dist/windows/rust-youtube-streamer-service.exe` (Step 5+6 evidence).
- [ ] New file committed alone: `git diff --cached` only `scripts/build-windows.sh`; commit message fixed; clean status after (Step 9).
