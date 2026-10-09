# Implementation Plan — Task 3: Establish Reproducible Docker-Based Development Checks

- Source TODO: `.agent/todos/20261008/20261008-todo-2.md` → "### 3. Establish Reproducible Docker-Based Development Checks"
- Global plan: `.kilo/plans/20261008-phase00-repo-foundation-baseline.md` (Task 3 pre-analysis)
- Workflow: `.kilo/commands/critical-workflow.md` — this plan covers **only step 4.2 (Implementation) for Task 3**, plus codified 4.3 / 4.4 / 4.5b applicability.
- Implementer profile: JUNIOR developer under 50% restriction. Follow every step exactly. If anything is ambiguous or fails in a way not covered here, STOP and return the question/error to the caller. Never guess.
- Date: 2026-10-08
- **AMENDED 2026-10-08 (user-approved caller decision):** the compose-only setup FAILED Step 4.2 because the pinned base image ships WITHOUT rustfmt/clippy. Plan amended IN PLACE — new step order: Steps 3.4 / 3.5 / 3.5b / 3.5c (create `Dockerfile` → amend compose → ONE fix commit → explicit image build) precede the four-check sequence. The four checks keep their numbering (3.6–3.9); Steps 3.10–3.12 are kept; no other renumbering.

---

## 0. Context Snapshot (verified state — do not re-verify beyond the checks listed)

- Working directory: `C:\repo\rust-youtube-streamer` (Windows host). **Cargo/Rust are NOT installed on the host — NEVER run `cargo` on the host.**
- All cargo commands run on the Alpine VM via `alpine-vm` MCP tools, through Docker (Docker 27.3.1 verified). Project is mounted on the VM at `/rust-youtube-streamer`.
- Tasks 1–2 are DONE: `Cargo.toml`, `Cargo.lock`, `src/main.rs` exist at the repo root (verified by glob); the package is `rust-youtube-streamer-service` v0.1.0 edition 2021 with zero dependencies.
- Image `rust:1.82` pull was verified working on the VM earlier (caller-provided environment fact).
- **Blocked 4.2 state (recorded 2026-10-08):** `docker-compose.yml` was created per the ORIGINAL §2/§3.4 and COMMITTED as `bf5cf7f` (message `chore: add rust:1.82 compose service for Docker-based dev checks`). First-pass check attempts under the bare base image failed ONLY because `rust:1.82` ships WITHOUT the `rustfmt`/`clippy` components. `src/main.rs` formatting style is therefore UNVERIFIED; no `target/` appeared on the mount; `Cargo.lock` is intact and clean.
- **AMENDMENT decision (2026-10-08, user-approved, BINDING):** add a minimal root `Dockerfile` derived FROM the pinned `rust:1.82` base, baking in rustfmt + clippy via `rustup component add rustfmt clippy`; the compose service switches from `image:` to `build: context: .` building from that Dockerfile. The pin REMAINS THROUGH the Dockerfile `FROM rust:1.82` line — `rust:1.82` stays the pinned base. The original Steps 3.4/3.5 (compose creation + first commit) are SUPERSEDED/completed (`bf5cf7f`); Steps 3.4–3.5c below encode the amendment.
- The Docker **named volume** `rust-streamer-target` already exists on the VM (auto-created by Task 2's docker runs).
- **Caller decision (2026-10-08, user-approved, BINDING from Task 2 plan §8):** the compose service MUST replicate `-v rust-streamer-target:/rust-streamer-target` (named volume) AND `CARGO_TARGET_DIR=/rust-streamer-target`, so NO build artifacts are generated in the shared project root (VirtualBox mount mmap issue caused silent exit 101; the named volume solved it). `Cargo.lock` must remain the ONLY generated root file.
- User preference recorded (2026-10-08): the project folder MAY be shared into the container, but the container must NOT generate any files in the project root directly (other than `Cargo.lock`).
- `.gitignore` currently has NO Rust entries (`target/` not ignored) — that is **Task 4's** work. Do NOT touch `.gitignore` in this task.
- README "Current Status" section is **Task 5's** scope. Do NOT touch README in this task.
- MCP tool constraints (caller-verified): call `alpine-vm_vm_status` FIRST before any `vm_run_command`; commands are SINGLE (no `&&`, no pipes); allowed prefixes: `docker`, `sh`, `apk`, `ls`, `cat`, `ps`, `df`, `free`, `uname`, `pwd`, `whoami`.
- Note on `docker compose` syntax: every compose invocation in this plan uses an explicit `-f /rust-youtube-streamer/docker-compose.yml` flag so the command works regardless of the VM shell's current working directory (the MCP tool does not document its cwd, and `cd` is not an allowlisted prefix). Compose v2 uses the compose-file directory as the project directory, so relative/absolute mounts resolve as written.
- Git: work happens on the feature branch `feat/phase00-cargo-baseline` created in Step 2 of the Critical Workflow. Branch creation/merge/push are OUTSIDE this plan. This plan contains ONLY: this task's commit(s), gitignore-compliance pre-commit checks, and no push.
- Expected git state of `Cargo.lock`: tracked and clean. It must remain byte-identical after this task (all cargo runs use `--locked`; fmt does not touch the lockfile).

---

## 1. Final Decisions (architector-verified; implementer must NOT change these)

1. **Derived toolchain image (AMENDED — supersedes and REPEALS the earlier compose-only decision).** A minimal root `Dockerfile` (§2a) derives FROM the pinned base and bakes in the two missing toolchain components (evidence for the amendment: the blocked 4.2 state in §0). Exactly ONE build mechanism exists after this task: the compose service's `build: context: .` pointing at that root `Dockerfile`. No second Dockerfile, no `docker-compose.override.yml`, no `.dockerignore`, no service duplication.
2. **Image pin: `rust:1.82`, enforced THROUGH the Dockerfile `FROM rust:1.82` line** (AMENDED — the compose file no longer declares `image:`; the pin travels with the Dockerfile in the build context). Deliberate stable version — not `:latest`, not floating. **No tag switching and no base-image replacement: on a base-pull failure OR a `rustup component add rustfmt clippy` failure (network/registry), STOP and report the full build output.**
3. **Service name: `rust`.** Every check invocation reads: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo <subcommand> ...`.
4. **Mount path: SAME path inside the container.** The VM's project directory `/rust-youtube-streamer` is bind-mounted at `/rust-youtube-streamer` in the container, and the service's `working_dir` is `/rust-youtube-streamer`. (Decision over the alternative `/workspace` — same path avoids any divergence between VM paths, compose mounts, and container `working_dir`; one path everywhere = fewer mistakes for a junior implementer.)
5. **Named-volume redirect (BINDING, caller decision):** the service declares `rust-streamer-target:/rust-streamer-target` and `CARGO_TARGET_DIR: /rust-streamer-target`. The top-level `volumes:` entry uses NO `external:` flag (compose will use the existing volume, or auto-create it if absent — both fine). Every cargo invocation goes through compose, so the env var and volume travel with the service definition — NO per-invocation `-e` or extra `-v` duplication at CLI level. Encode: implementer must NOT add `-e CARGO_TARGET_DIR=...` or target-volume flags to the `docker compose run` lines.
6. **No `user:` override, no `--privileged`.** The container runs as its default user (root in the Debian-based rust image). Files written to the bind mount (`Cargo.lock` updates under `--locked` should be none anyway) appear via the VirtualBox shared-folder mapping, which is how Task 2's identically-configured runs already produced a committable `Cargo.lock`. This satisfies "ownership addressed" by documentation + verification, not by manipulation.
7. **Ownership handling = verify-then-report, NOT chown.** After the first cargo run, ONE `ls -l /rust-youtube-streamer/Cargo.lock` (and `ls -l /rust-youtube-streamer`) records actual ownership/permissions for the completion report. If any container write FAILS with a permission/ownership error: capture full output, change nothing, STOP, and report to caller. **No silent or scheduled chown sweeps, no uid/gid overrides mid-task.**
8. **Check sequence order & lockfile semantics (AMENDED):**
   0. **Build the derived toolchain image explicitly ONCE** (Step 3.5c) — a prerequisite gate that runs BEFORE any check; first-run behavior encodes ONE explicit sequence, NOT reliance on `docker compose run` auto-build.
   1. `cargo fmt --check` (formatting gate first — it may trigger the one-time `cargo fmt` apply; see Step 3.6)
   2. `cargo check --locked`
   3. `cargo test --locked`
   4. `cargo clippy --locked -- -D warnings`
   Sequential execution is REQUIRED (later commands reuse the build cache in the named volume and all rely on the already-generated `Cargo.lock`). `--locked` guarantees cargo refuses to modify `Cargo.lock`; if any command reports the lockfile is dirty/needs update, STOP and report — do NOT run lockfile-updating commands.
   First-pass attempt outcomes recorded in §0 are HISTORY only: every check in Steps 3.6–3.9 must produce a FRESH exit status from the derived image before Step 4.2 may complete.
9. **No `check_lock`/healthcheck/ports/networks/custom `entrypoint` in the compose file.** Minimal service definition only (§2).
10. **Docs: one short comment block inside `docker-compose.yml` (existing since `bf5cf7f`) plus ONE comment line inside the new `Dockerfile` (§2a) — nothing else.** README/docs/project-info/context.md updates are Task 5. The executed-command log lives in the completion report, not in repo files.
11. **No `.gitignore` edit.** `docker-compose.yml` and the new `Dockerfile` must match no current ignore rule — VERIFY both against the read `.gitignore` before the Step 3.5b commit (do not assume), so gitignore compliance passes without changes.

---

## 2. Toolchain image (Dockerfile) + compose amendment — exact content [AMENDED]

### 2a. NEW file `C:\repo\rust-youtube-streamer\Dockerfile` (host, file-write tool, real newlines — never literal `\n` sequences)

EXACTLY 3 lines — one comment line plus the two functional lines. NOTHING else: no labels, no `curl`/`apt` installs, no FFmpeg, no `WORKDIR`/`ENV`/`ENTRYPOINT`/`CMD`, no CI steps:

```dockerfile
# Local dev-check toolchain: pinned base rust:1.82 + baked-in rustfmt/clippy (the base image ships without them).
FROM rust:1.82
RUN rustup component add rustfmt clippy
```

Notes (encoded, do not deviate):
- Final byte is a newline after the `RUN` line; no trailing blank lines, no trailing whitespace.
- The ONE comment line is FROZEN (sanctioned by the amendment; mirrors the committed compose comment style) and joins the Step 4.4 in-file-comment exception. The `FROM rust:1.82` line IS the image pin's new home.

### 2b. AMENDED file `docker-compose.yml` (currently committed as `bf5cf7f`)

After Step 3.5 the file MUST byte-match this amended target. The comment block, `working_dir`, both volume entries, `CARGO_TARGET_DIR`, and the top-level `volumes:` are UNTOUCHED — only the single `image:` key becomes the two `build:` keys:

```yaml
# Local development checks only. Runs Rust/Cargo commands against the shared
# project directory from the Alpine VM's Docker engine. Build artifacts are
# redirected to the named volume "rust-streamer-target" via CARGO_TARGET_DIR
# so no artifact files are generated inside the mounted project directory
# (only Cargo.lock is intentionally generated there).
services:
  rust:
    build:
      context: .
    working_dir: /rust-youtube-streamer
    volumes:
      - /rust-youtube-streamer:/rust-youtube-streamer
      - rust-streamer-target:/rust-streamer-target
    environment:
      CARGO_TARGET_DIR: /rust-streamer-target

volumes:
  rust-streamer-target:
```

Notes (encoded, do not deviate):
- The EXACT edit: the service's single line `    image: rust:1.82` becomes the two lines `    build:` and `      context: .` (indentation exactly as shown — two-space levels). Everything else is UNCHANGED; `git diff` after Step 3.5 must show exactly 1 removed line and 2 added lines in `docker-compose.yml`.
- Build-only form chosen by the planner (encode exactly one form): NO `image:` key next to `build:` — compose does not require one; it auto-names the built image from the project/service names. Do NOT add `image:` back.
- No `user:` key, no `privileged:`, no `ports:`, no `networks:`, no `entrypoint:`, no `pull_policy:`, no `restart:`, no `healthcheck:`.
- Line-level style: two-space indentation, keys in the order shown. Cosmetic but frozen for adherence checking.

---

## 3. Step-by-step implementation (Step 4.2)

### Preamble (AMENDED resume point)

- All VM commands are SINGLE commands through `alpine-vm_vm_run_command`. No `&&`, no pipes, no `sh -c` wrappers. Host-side git commands are also single commands (no chains).
- **Resume point: execution starts at Step 3.4.** The ORIGINAL Steps 3.1–3.5 already executed in the first pass (`docker-compose.yml` created and committed as `bf5cf7f`; first-pass fmt/attempt outcomes recorded in §0). Steps 3.1–3.2 need not re-run (harmless if re-run); Step 3.3's mount listing is re-run with the amended expectation below.
- First-pass outcomes (including any check that may have passed under the bare base image) are HISTORY only: every check in Steps 3.6–3.9 must produce a FRESH exit status from the derived image built in Step 3.5c before Step 4.2 may complete.

### Step 3.1 — Pre-flight: VM status

1. Call `alpine-vm_vm_status`. If the VM is not running/SSH-unreachable, STOP and report to caller.

### Step 3.2 — Verify compose plugin availability

1. Run ONE command:
   ```
   docker compose version
   ```
2. If that fails (plugin missing), run ONE fallback command:
   ```
   docker-compose --version
   ```
3. **If NEITHER works: STOP and report both outputs to the caller.** Do not attempt to install compose (no global installs rule; `apk add docker-compose` is out of scope). Do not fall back to plain `docker run` silently — plain docker run was the Task 2 mechanism; this task exists to make Compose the permanent one. Report and wait for caller direction.

### Step 3.3 — Inspect the mount before changes

1. Run ONE command:
   ```
   ls /rust-youtube-streamer
   ```
   Record the listing. Resume expectation (AMENDED): `Cargo.toml`, `Cargo.lock`, `docker-compose.yml` PRESENT (committed `bf5cf7f`, still holding `image: rust:1.82` until Step 3.5), `Dockerfile` ABSENT until Step 3.4, plus `src/`, `config/`, `credentials/`, `fonts/`, `logs/`, `README.md`, `.gitignore`, etc. If a `target/` directory appears, STOP and report (it contradicts Task 2's completion state; the caller decides cleanup — do NOT run `cargo clean` on your own).

### Step 3.4 — Create `Dockerfile` (host, file-write tool) [AMENDED — replaces the original compose-creation step; the compose file already exists as `bf5cf7f`]

1. Create `C:\repo\rust-youtube-streamer\Dockerfile` with the EXACT 3-line content of §2a. Do not reformat, do not add directives, do not translate the comment.
2. NOTHING else is created (no second Dockerfile, no `.dockerignore`, no `docker-compose.override.yml`).

### Step 3.5 — Amend `docker-compose.yml` (host, structured edit tool) [AMENDED — replaces the original compose-commit step]

1. Apply the EXACT one-key change encoded in §2b: replace the service's `image: rust:1.82` line with `build:` + `context: .` (indentation exactly as shown in §2b). Nothing else changes — the file must byte-match §2b.
2. Verify on host with ONE command:
   ```
   git diff -- docker-compose.yml
   ```
   The diff must show EXACTLY 1 removed line (`image: rust:1.82`) and 2 added lines (`build:`, `context: .`). If the diff shows anything else, change nothing further and STOP — report to the caller.

### Step 3.5b — Commit `Dockerfile` + compose amendment as ONE commit (host) [AMENDED — supersedes the original compose-first commit; `bf5cf7f` already landed]

1. Run `git status` (host). Verify ONLY untracked `Dockerfile` and modified `docker-compose.yml` appear (plus pre-existing known items); nothing unexpected.
2. Read `.gitignore` (host) and confirm NEITHER `Dockerfile` NOR `docker-compose.yml` matches any ignore rule — verify, do not assume.
3. Stage ONLY these two paths:
   ```
   git add Dockerfile docker-compose.yml
   ```
4. Commit (ONE commit for both files):
   ```
   git commit -m "fix: bake rustfmt and clippy into pinned toolchain image"
   ```
5. Record for §5: this is amendment commit #1 (historical #0 = `bf5cf7f`). Committing happens BEFORE the image build (Step 3.5c) so a build failure leaves a committed, reviewable state for the caller — matching the `bf5cf7f` precedent.

### Step 3.5c — Build the derived toolchain image EXPLICITLY ONCE (VM, first run) [NEW]

1. Verify the Dockerfile is visible through the shared mount and byte-matches §2a. Run ONE command:
   ```
   cat /rust-youtube-streamer/Dockerfile
   ```
   Must show exactly 3 lines (comment + `FROM rust:1.82` + `RUN rustup component add rustfmt clippy`). If absent or different: change nothing, STOP, and report (host edit not visible on the mount).
2. Build ONCE with ONE explicit command:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml build rust
   ```
   This explicit build is the ONLY build invocation in this plan — first-run behavior does NOT rely on `docker compose run` auto-build. After a successful build, later `run` invocations must NOT trigger a new build (the image exists); if any later output unexpectedly shows a build occurring, record it in the completion report.
3. Expected: exit 0 with the `RUN rustup component add rustfmt clippy` step completing. Record the ACTUAL exit status and the final output lines.
4. Record the built image identity for the completion report. Run ONE command:
   ```
   docker images
   ```
   Note the image name/tag compose created for the service.
5. **STOP condition: if the build fails at ANY step (base pull, `rustup component add rustfmt clippy` network/registry failure, any other error), capture the FULL output, change nothing, and STOP — report to the caller. No tag switching, no alternative base image, no manual `docker run ... rustup ...` repair sessions, no retries.**

### Step 3.6 — Check 1/4: `cargo fmt --check` [AMENDED context — flow UNCHANGED]

First-pass context: this check FAILED under the bare base image (§0 — no rustfmt component). `src/main.rs` formatting style is UNVERIFIED as of this amendment; there is NO "already applied" state to skip from. Skipping the apply is permitted ONLY when `--check` EXITS 0 inside the derived image built in Step 3.5c — the conditional apply branch below is LIVE.

1. Run ONE command on the VM:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check
   ```
2. **If it EXITS 0:** record success. Skip to Step 3.7. The apply-run (3.6b) must NOT execute when --check passes.
3. **If it FAILS** (non-zero exit, diff output): run `cargo fmt` ONCE:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt
   ```
4. Then re-run the check ONCE:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check
   ```
5. Report ALL exit statuses involved (the failing check, the apply run's exit status, the re-check's exit status — at minimum BOTH the failing-check and the re-check exit codes) in the completion report. If the re-check still fails: capture full output, change nothing further, STOP, and report.
6. If `cargo fmt` modified any file, `git status` on the host will show it — record what was modified in the report; the modification is expected and acceptable ONLY for formatting (typically none, since `src/main.rs` is 3 lines). If any file was modified, commit it now:
   ```
   git add src/main.rs
   git commit -m "style: apply rustfmt per cargo fmt"
   ```
   (If nothing was modified, skip this staging/commit entirely.)

### Step 3.7 — Check 2/4: `cargo check --locked` + mount verification

1. Run ONE command:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked
   ```
2. Expected: exit 0, `Finished ...` for `rust-youtube-streamer-service (bin)`. All artifacts land inside the named volume — never on the mount.
3. Verify the mount invariant with ONE command:
   ```
   ls /rust-youtube-streamer
   ```
   `target/` must NOT appear. If it does, capture full `ls` + check output, change nothing, STOP, and report.
4. If the check itself fails: capture full output, change nothing, STOP, and report.

### Step 3.8 — Check 3/4: `cargo test --locked` + mount verification

1. Run ONE command:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked
   ```
2. Expected: exit 0, one test executed (`0 filtered out` with 0 tests, or the doctest warning for a binary crate — either accepted; record actual output verbatim).
3. Verify mount invariant: `ls /rust-youtube-streamer` — no `target/`.
4. On failure: capture full output, change nothing, STOP, and report.

### Step 3.9 — Check 4/4: `cargo clippy --locked -- -D warnings` + mount verification

1. Run ONE command:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings
   ```
2. Expected: exit 0 (possible `Finished` with linter golf at this code size — accept exit 0 as pass regardless of warnings-vs-errors semantics; only `-D warnings` errors make it fail).
3. Verify mount invariant: `ls /rust-youtube-streamer` — no `target/`.
4. On failure: capture FULL output, change nothing, STOP, and report. Do not "fix" warnings on your own (e.g., do not rewrite `src/main.rs` to satisfy a lint) — that would change files outside this plan's specified content.

### Step 3.10 — Ownership verification (documentation duty, no changes)

1. Run ONE command:
   ```
   ls -l /rust-youtube-streamer/Cargo.lock
   ```
2. Run ONE command:
   ```
   ls -l /rust-youtube-streamer
   ```
3. Record the actual owner/group/permissions of `Cargo.lock` and the project root as shown by the mount. This is the "UID/GID or permission handling required" evidence demanded by the TODO. If permissions prevent container writes (evidenced by a concrete failure earlier), the task already STOPPED upstream — this step only completes the record for the success case.
4. If `ls -l` is somehow not supported in the VM shell (unlikely — it is standard busybox/coreutils), run `ls -l` through the container instead:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer
   ```
   and record that output instead (it shows container-side ownership, which is the relevant view).

### Step 3.11 — Cargo.lock integrity verification (host)

1. Run on host: `git status` — `Cargo.lock` must show NO modification. (`--locked` on every cargo run guarantees this; fmt never touches it.)
2. Run on host: `git diff -- Cargo.lock` — empty output.
3. If `Cargo.lock` is modified or `target/` exists anywhere in the worktree: capture output, change nothing, STOP, and report (this contradicts the binding decision record).

### Step 3.12 — Final verification snapshot (host)

- `git status` — clean working tree except pre-existing unrelated items; NO `target/` anywhere.
- `git log --oneline -5` — shows this task's commit stack: `bf5cf7f` (already landed), the Step 3.5b single fix commit on top, plus the conditional `style:` commit if Step 3.6.6 triggered — in §5 order.

No further commands. Do NOT run `git push` (restricted to workflow step 5), do not merge (step 5), do not add `[DONE]` to the TODO (Task 3's 4.6 is a separate sub-task).

---

## 4. Workflow sub-step applicability (codified)

- **Step 4.3 (code review & simplification) — REQUIRED [AMENDED].** New file `Dockerfile` + amended `docker-compose.yml` exist. Assign `code-reviewer` and `code-simplifier` concurrently, confined to these two files. Reviewers may not propose: `FROM` base-tag changes, Dockerfile directives beyond §2a (labels/`curl`/FFmpeg/`WORKDIR`/`ENV`/`ENTRYPOINT`), a second Docker/Compose file or `.dockerignore`, `user:`/uid/gid overrides, removal of the named volume/env entries (Task 2 §8 binding decision), CI additions, or README edits. Any such proposal is out of scope — reject/refer to caller.
- **Step 4.4 (documentation) — NOT REQUIRED [AMENDED wording]** (codified, mirror of Task 2's decision): all documentation for this phase belongs to Task 5 (README build instructions, `project-structure.md`, `context.md`). The only permitted exceptions are the in-file comments created during implementation (the compose comment block from the first pass — UNCHANGED — and the new one-line `Dockerfile` comment, §2a) — they are NOT doc-specialist work. The docs-specialist must return a clear "not required" message. Executed-command documentation goes in the completion report, not in repo files.
- **Step 4.5a — N/A** (no front-end tasks in this phase per global plan).
- **Step 4.5b (overall plan adherence) — REQUIRED [AMENDED].** Assign `architector` to verify: `Dockerfile` byte-matches §2a; compose file byte-matches §2b (vs `bf5cf7f`: exactly the one-key image→build change); commits match §5 messages (`bf5cf7f` historical; the single fix commit; conditional fmt commit); the explicit Step 3.5c build recorded exit 0 (and `docker images` output recorded); all four checks executed with real FRESH exit statuses from the derived image in the completion report; `Cargo.lock` unchanged; no `target/` on the mount; no `.gitignore`/README/docs edits; no tag switching; no extra Docker files.

## 5. Git commit summary (exact messages) [AMENDED]

| Order | What | Message |
|---|---|---|
| 0 (HISTORICAL — already landed as `bf5cf7f`; do NOT re-run, amend, or force-push) | `docker-compose.yml` (initial, image-based) | `chore: add rust:1.82 compose service for Docker-based dev checks` |
| 1 (Step 3.5b, output of Steps 3.4–3.5) | `Dockerfile` (new) + `docker-compose.yml` (single-key amendment) staged as ONE commit | `fix: bake rustfmt and clippy into pinned toolchain image` |
| 2 (Step 3.6.6, CONDITIONAL — only if cargo fmt modified files) | formatted source file(s) | `style: apply rustfmt per cargo fmt` |

Gitignore compliance before each commit: read `.gitignore`, run `git status`, stage only the listed paths, never stage `target/` or container artifacts. No push, no merge in this task. Single-commit grouping for the amendment is deliberate (caller-approved): `bf5cf7f` already landed a compose commit before its checks exposed the missing-components problem, so the amendment is ONE follow-up commit containing exactly the two Docker-related files.

---

## 6. Explicit prohibitions (restate — junior implementer)

- Dockerfile scope: EXACTLY the 3-line §2a file. Do NOT add labels, `curl`/`apt` installs, FFmpeg, `WORKDIR`/`ENV`/`ENTRYPOINT`/`CMD`, CI steps, or `.dockerignore`; do NOT create a second Dockerfile or an override compose file. ONE build mechanism: compose `build: context: .` → root `Dockerfile`.
- Do NOT switch base-image tags on pull/build failure (no `:latest`, no other version) — STOP and report the full `docker compose build`/pull error output.
- Do NOT attempt to repair a failed image build (e.g., component-add network/registry failure) via manual `docker run ... rustup ...` sessions, retries, or alternate registries — capture output, STOP, report to caller.
- Do NOT edit `.gitignore` (Task 4), README, `.agent/**` docs, project-info files, `.kilo/**` (except this plan's file), or any `src/` file not touched by `cargo fmt` itself.
- Do NOT run cargo on the Windows host; only host-side git, the `Dockerfile` creation, and the single-key compose edit.
- Do NOT use multi-command chains (`&&`, `&`, pipes) or privileged containers (`--privileged`, `--user` elevation changes) via the VM MCP.
- Do NOT set up CI, a production/deployment image, or install/bundle FFmpeg into the image.
- Do NOT run `chown`/`chmod` sweeps on the mount; ownership handling = verify + report (Step 3.10) + stop-and-report on write failures.
- Do NOT run `cargo fmt` unless `cargo fmt --check` first failed in Step 3.6 (and then only ONCE).
- Do NOT run lockfile-modifying commands (no `cargo update`, no `generate-lockfile`); all checks use `--locked`.
- Do NOT omit the `CARGO_TARGET_DIR` env or the named volume from the compose service (Task 2 §8 binding decision); do NOT duplicate them as CLI flags either — compose carries them.
- Do NOT stage or commit anything other than the files/messages in §5.
- Do NOT claim Windows or Linux runtime validation anywhere — the checks run in a Linux Debian-based container DERIVED FROM `rust:1.82` (§2a) on the Alpine VM against the shared mount only.

---

## 7. Mapping to TODO Task 3 sub-items and Phase-00 acceptance criteria

| TODO Task 3 sub-item (20261008-todo-2.md) | Covered by |
|---|---|
| Minimal Docker config to run Rust/Cargo reproducibly from the Alpine VM against the shared dir; official maintained image pinned to a deliberate stable version | §1 decisions 1–2 (AMENDED); §2a `FROM rust:1.82`; Steps 3.4–3.5b |
| Dockerfile+Compose OR equally clear minimal setup; prefer Compose for consistent standard commands; avoid overlapping mechanisms | §1 decision 1 (AMENDED) — Dockerfile + Compose, ONE derived-image build mechanism (`build: context: .` → root `Dockerfile`); no overlapping second mechanism |
| Project dir mounted; Cargo operates on the shared source tree; generated files like `Cargo.lock` remain available in the shared dir afterwards | §2 volumes + `working_dir`; Steps 3.6–3.9; Step 3.11 (lockfile intact) |
| Container writes without unexpected ownership where feasible; document UID/GID/permission handling; no privileged containers | §1 decisions 6–7; Step 3.10 verification record; write-failure stop-and-report |
| Document exact Docker commands for each check adapted to the actual MCP/VM environment | §3 Steps 3.5c–3.9 exact single commands (`docker compose -f ... build rust`; `docker compose -f ... run --rm rust cargo ...`); carried to README in Task 5; recorded in completion report |
| Run `cargo fmt --check` (apply fmt once + re-run if needed) | Step 3.6 |
| Run `cargo check --locked` after lockfile generated | Step 3.7 |
| Run `cargo test --locked` | Step 3.8 |
| Run `cargo clippy --locked -- -D warnings` | Step 3.9 |
| Sequential execution where lockfile/depends-on earlier output | §1 decision 8 (AMENDED): explicit image build → fmt → check → test → clippy; named-volume cache reuse |
| Confirm exact exit statuses from MCP/VM output; never report unexecuted as passing; report inability + reason | §3 stop conditions throughout; §6 Completion report items 2–5 |
| Do not claim Windows/Linux runtime validation; report actual VM/container scope precisely | §1 decision 11 report wording; §6 prohibition; completion report item 4 |
| No CI / no deployment image in this phase | §6 prohibition |

Acceptance criteria from the TODO touched by Task 3:
- "[ ] A reproducible Docker-based Rust toolchain setup exists and operates on the shared project directory from the Alpine VM." → Steps 3.4–3.9 (derived toolchain image + compose service + checks).
- "[ ] The selected Rust container image/version is explicit and documented." → `rust:1.82` pinned through the `Dockerfile` `FROM` line (§2a); documentation completed by Task 5 (factual recording here belongs in this task's report).
- "[ ] Container-generated project files remain accessible in the shared directory and ownership/permissions are addressed." → Steps 3.10, 3.11.
- "[ ] `cargo fmt --check` succeeds in the container, or the inability is explicitly documented." → Step 3.6.
- "[ ] `cargo check --locked` succeeds..." → Step 3.7.
- "[ ] `cargo test --locked` succeeds..." → Step 3.8.
- "[ ] `cargo clippy --locked -- -D warnings` is run in the container where feasible and its actual result is reported." → Step 3.9.

Checkbox completion belongs to Task 3's 4.6 sub-task, not here. `.gitignore` criterion (`target/` ignored), README criterion, and docs criterion belong to Tasks 4 and 5 — explicitly out of this plan.

---

## 8. Completion report requirement for the implementer

At the end of Step 4.2, the implementer must report:
1. Files created/changed: `Dockerfile` (NEW) and `docker-compose.yml` (single-key image→build amendment), staged TOGETHER as the ONE §5-order commit (+ `src/main.rs` ONLY if Step 3.6's conditional fmt apply modified it) — with `git log --oneline` evidence of the commit stack: `bf5cf7f` (historical), the fix commit, and the conditional fmt commit if triggered.
2. Every exact MCP/VM command executed (Step 3.5c: `cat`, `docker compose ... build rust`, `docker images`; Steps 3.6.x, 3.7, 3.8, 3.9, 3.10) plus the host git commands of Step 3.5b, each with its ACTUAL exit status and key output lines — quoting real output, not expectations. State the build outcome explicitly (component-add success or failure).
3. Alpine VM details and Rust container/toolchain actually used (docker 27.3.1; derived image built FROM `rust:1.82` WITH `rustfmt` + `clippy` baked in — record the built image name/tag from Step 3.5c's `docker images`; cargo/rustc version if captured incidentally from check output).
4. Scope of validation stated precisely: checks ran inside a Linux Debian-based container DERIVED FROM `rust:1.82` (rustfmt/clippy baked in) on the Alpine VM against the shared project mount; NO Windows or Linux native runtime validation is claimed.
5. Any check not run, and why (e.g., fmt apply skipped because `--check` passed; compose fallback path not used; no fmt apply has EVER succeeded yet — the first-pass apply never ran, which is why the conditional stays live).
6. Any assumptions or blockers (compose plugin status; ownership/permission observations from Step 3.10).
7. Confirmation that no out-of-scope feature was implemented.

---

## 9. Plan verification against the original task

- Every sub-item of TODO "### 3. Establish Reproducible Docker-Based Development Checks" is encoded (mapping table §7). ✓
- Compose-only decision REVOKED by amendment; AMENDED decision encoded with full `Dockerfile` + amended compose content (§2a/§2b), pin-through-`FROM rust:1.82`, explicit ONE-command build (Step 3.5c), stop-on-component-add-failure; mount-at-same-path decision retained. ✓
- Task 2 §8 binding decision carried forward verbatim (named volume + `CARGO_TARGET_DIR` on the service; CLI duplication forbidden). ✓
- Exact single-command sequence with order and stop conditions encoded (Steps 3.1–3.12 incl. 3.5b/3.5c); compose availability check with fallback and stop; no tag switching. ✓
- Mount invariant (`ls` no `target/`) + `Cargo.lock` integrity verified after the cargo runs. ✓
- Ownership/UID-GID handling = verify-then-report, no chown sweeps, no privileged mode; stop-and-report on write failures. ✓
- README untouched (Task 5); command record lives in the completion report; in-file documentation = the compose comment block (UNCHANGED since `bf5cf7f`) + the ONE `Dockerfile` comment line (§2a), nothing else. ✓
- 4.3 required / 4.4 not required (with composed-comment exception) / 4.5b required — codified. ✓
- Prohibitions: no CI, no production image, no FFmpeg, no host cargo, no `.gitignore` edits, no tag switching, no manual image-build repair sessions, no silent fallbacks; Dockerfile limited to the exact §2a content. ✓
- Git handling: one amendment commit (`Dockerfile` + compose amendment) on top of the historical `bf5cf7f` (+conditional fmt commit), gitignore compliance, no push/merge. ✓
- Amendment scope audit (2026-10-08): only Steps 3.4/3.5 rewritten and new Steps 3.5b/3.5c inserted; four-check numbering 3.6–3.9 and Steps 3.10–3.12 kept (Step 3.10 as-is); §5 commit grouping updated per caller; no unrelated content altered. ✓
