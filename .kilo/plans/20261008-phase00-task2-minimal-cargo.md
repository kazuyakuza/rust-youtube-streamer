# Implementation Plan — Task 2: Establish the Minimal Cargo Project

- Source TODO: `.agent/todos/20261008/20261008-todo-2.md` → "### 2. Establish the Minimal Cargo Project"
- Global plan: `.kilo/plans/20261008-phase00-repo-foundation-baseline.md` (Task 2 pre-analysis)
- Workflow: `.kilo/commands/critical-workflow.md` — this plan covers **only step 4.2 (Implementation) for Task 2**, plus codified 4.3 / 4.4 / 4.5b applicability.
- Implementer profile: JUNIOR developer under 50% restriction. Follow every step exactly. If anything is ambiguous or fails in a way not covered here, STOP and return the question/error to the caller. Never guess.
- Date: 2026-10-08

---

## 0. Context Snapshot (verified state — do not re-verify beyond the checks listed)

- Working directory: `C:\repo\rust-youtube-streamer` (Windows host). **Cargo/Rust are NOT installed on the host — NEVER run `cargo` on the host.**
- All cargo commands run on the Alpine VM via `alpine-vm` MCP tools, through Docker in the VM. Project is mounted on the VM at `/rust-youtube-streamer`.
- Task 1 is DONE: repo inspected, docker engine confirmed working on VM, mount `/rust-youtube-streamer` mirrors this host directory.
- Caller decision (2026-10-08, user-approved): all Cargo build artifacts are redirected to the Docker **named volume** `rust-streamer-target` (`CARGO_TARGET_DIR=/rust-streamer-target`) so that nothing but source files and the required `Cargo.lock` live in the project root, and to avoid the silent rmeta mmap failure observed on the VirtualBox shared-folder mount (see §1 decision 8; Task-3 decision record §8). A stale `target/` possibly left in the mount by an earlier crashed run must be removed FIRST (Step 2.1a).
- Current repo files (verified): no `Cargo.toml`, no `Cargo.lock`, no `src/main.rs`. `config/`, `credentials/`, `fonts/`, `logs/`, and `src/{app,config,youtube,chat,renderer,streaming}/` contain only `.gitkeep` placeholders.
- `.gitignore` has NO Rust entries yet — that is Task 4's work. Do NOT touch `.gitignore` in this task.
- Git: work happens on the feature branch created in Step 2 of the Critical Workflow (`feat/phase00-cargo-baseline`). Branch/merge/push actions are outside this plan (restricted to other steps). This plan contains ONLY: this task's commits, gitignore-compliance pre-commit checks, and no push.

---

## 1. Final Decisions (architector-verified; implementer must NOT change these)

1. **Container image pin: `rust:1.82`** — official maintained image, long-supported stable series.
2. **Edition: 2021** — the verified-stable pairing usable with broad rustc versions, including 1.82.
3. **Package: single binary** `rust-youtube-streamer-service`, version `0.1.0`, zero dependencies.
4. **`Cargo.lock` IS committed** (executable application policy per TODO; not ignored).
5. **No `[dependencies]`/`[dev-dependencies]` section in `Cargo.toml` at all** (an absent section, not an empty one).
6. **No Dockerfile/docker-compose.yml in this task** — Task 2 precedes Task 3, so lockfile generation uses explicit `docker run` with the explicit project bind mount PLUS the named-volume target redirect (decision 8). Do not create any Docker/Compose files here.
7. If the image tag `rust:1.82` is not available locally in the VM and `docker run` fails to pull it, **STOP and return the exact docker error output to the caller**. Do NOT switch tags, do NOT use `:latest`, do NOT invent network diagnostics.
8. **Cargo target-dir redirect to a Docker named volume — caller decision (2026-10-08), user-approved.**
   - All Cargo build artifacts MUST go to the named volume `rust-streamer-target`, mounted at `/rust-streamer-target`, selected via `-e CARGO_TARGET_DIR=/rust-streamer-target`.
   - EVERY docker run in this task that invokes cargo — Steps 2.5, 2.6, and the optional `cargo --version` capture (§6) — must include BOTH `-v rust-streamer-target:/rust-streamer-target` AND `-e CARGO_TARGET_DIR=/rust-streamer-target`, in addition to the project bind mount, exactly as written in those steps. The ONLY exception is the Step 2.1a stale-target cleanup, which deliberately omits the named volume and env so `cargo clean` acts on the stale `target/` inside the mount.
   - `Cargo.lock` STAYS the ONLY generated file written into the project root — it is the required dependency lockfile for the executable application and remains generated on the mount and committed exactly as planned (Steps 2.5/2.7 and their commit message/staging group are unchanged). NO other generated files may be written into the project root; `target/` must NOT reappear in the mount (verified with `ls /rust-youtube-streamer` after every cargo command, and explicitly after `cargo check` in Step 2.6).
   - Rationale (caller): keeps the project root free of generated files other than `Cargo.lock`, and avoids the silent rmeta mmap failure observed on the VirtualBox shared-folder mount.

---

## 2. Step-by-step implementation (Step 4.2)

### Step 2.1 — Pre-flight checks

1. Call `alpine-vm_vm_status`. If the VM is not running/SSH-unreachable, STOP and report to caller.
2. Run ONE command via `alpine-vm_vm_run_command`:
   ```
   docker --version
   ```
   If it fails, STOP and report the output to caller.

Do not run any other exploratory commands.

### Step 2.1a — Remove stale `target/` from the mount (conditional cleanup, run BEFORE the first cargo command)

1. Run ONE command via `alpine-vm_vm_run_command` to inspect the mount:
   ```
   ls /rust-youtube-streamer
   ```
2. If the listing contains a `target/` directory, run ONE single command, EXACTLY as written (note: it DELIBERATELY has no named volume and no `CARGO_TARGET_DIR` so that `cargo clean` removes the stale in-mount `target/`; do NOT "fix" it by adding the redirect — this is the codified sole exception per §1 decision 8):
   ```
   docker run --rm -v /rust-youtube-streamer:/rust-youtube-streamer -w /rust-youtube-streamer rust:1.82 cargo clean
   ```
   If the listing does NOT contain `target/`, skip this command entirely — the cleanup is conditional and must not run otherwise.
3. Verify with ONE command:
   ```
   ls /rust-youtube-streamer
   ```
   `target/` must be gone. If `cargo clean` failed, or `target/` still appears, capture the FULL output, change nothing, STOP, and return it to the caller.
4. Reason (do not act beyond this): an earlier crashed run left broken rmeta artifacts inside the mounted `target/`; they must not be present when the named-volume workflow starts (§1 decision 8, §8).

### Step 2.2 — Create `Cargo.toml` (host, file-write tool, NOT bash)

Create `C:\repo\rust-youtube-streamer\Cargo.toml` with EXACTLY this content (use real newlines; no literal `\n` sequences):

```toml
[package]
name = "rust-youtube-streamer-service"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rust-youtube-streamer-service"
path = "src/main.rs"
```

Notes:
- No `[dependencies]` section. No `[profile.*]` sections. No metadata keys (authors/description/readme) — those are documentation-phase decisions (Task 5) and adding them now is out of scope.
- The explicit `[[bin]]` section pins the single-binary layout; it is intentional. Keep it.
- Do NOT create any other src files (see prohibitions in §4).

### Step 2.3 — Create `src/main.rs` (host, file-write tool)

Create `C:\repo\rust-youtube-streamer\src\main.rs` with EXACTLY this content — a single line, no comments, no other code:

```rust
fn main() {
    println!("rust-youtube-streamer-service");
}
```

Notes:
- The println exists to make the binary's proof-of-build observable without any dependencies or logic. This is the final wording (task prompt's `e.g.` wording was an example; this exact form is the decision).
- Do NOT add `use` statements, modules, arguments, or return-type changes.

### Step 2.4 — Commit `Cargo.toml` FIRST (host git, single staged group)

1. Run `git status` (host). Verify only `Cargo.toml` (untracked) plus the known pre-existing state appear; nothing else unexpected.
2. Read `.gitignore` (host) and confirm `Cargo.toml` matches no ignore rule (it does not — verified). Gitignore compliance is satisfied.
3. Stage ONLY `Cargo.toml`:
   ```
   git add Cargo.toml
   ```
4. Commit:
   ```
   git commit -m "feat: add minimal Cargo package baseline"
   ```

### Step 2.5 — Generate `Cargo.lock` on the VM (single docker command, named-volume pattern)

1. (Already verified in Step 2.1 that VM is up.) Run ONE single command via `alpine-vm_vm_run_command` — note BOTH the named-volume mount and the `CARGO_TARGET_DIR` env var, per §1 decision 8:
   ```
   docker run --rm -v /rust-youtube-streamer:/rust-youtube-streamer -v rust-streamer-target:/rust-streamer-target -e CARGO_TARGET_DIR=/rust-streamer-target -w /rust-youtube-streamer rust:1.82 cargo generate-lockfile
   ```
2. Expected outcome: `Cargo.lock` appears at the repo root on the shared mount (visible on the host, matching VM path `/rust-youtube-streamer/Cargo.lock`); Docker auto-creates the named volume `rust-streamer-target` on first use. `Cargo.lock` is the ONLY generated file allowed in the project root — verify after the command with ONE command:
   ```
   ls /rust-youtube-streamer
   ```
   The listing must show `Cargo.lock` and must NOT show `target/`. If `target/` appears, capture the full `ls` and docker output, change nothing, STOP, and report to caller.
3. If the docker run fails (image not pullable, mount error, etc.), capture the FULL docker error output, change nothing, STOP, and return the error text to the caller.

### Step 2.6 — Validate with `cargo check --locked` (single VM command, named-volume pattern)

1. Run ONE single command via `alpine-vm_vm_run_command` — the exact caller-approved redirect form; do NOT omit either mount or the env var:
   ```
   docker run --rm -v /rust-youtube-streamer:/rust-youtube-streamer -v rust-streamer-target:/rust-streamer-target -e CARGO_TARGET_DIR=/rust-streamer-target -w /rust-youtube-streamer rust:1.82 cargo check --locked
   ```
2. Expected: exit success, `Finished ...` output for checking `rust-youtube-streamer-service (bin)`. All build artifacts (rmeta/deps output) are written INSIDE the named volume `rust-streamer-target` — never onto the shared mount (§1 decision 8, §8).
3. Verify with ONE command `ls /rust-youtube-streamer` that `target/` has NOT reappeared in the mount; the only generated root-level change remains `Cargo.lock`. If `target/` appears, capture the full `ls` and check outputs, change nothing, STOP, and report to caller.
4. Scope boundary: this validates ONLY that the package compiles with its lockfile. It is NOT the full Task 3 check suite (`cargo fmt --check`, `cargo test --locked`, `cargo clippy`) — do NOT run those here and do NOT report them as passing.
5. If the check fails, capture full output, change nothing about the file contents without narrating to the caller first, and STOP to report.

### Step 2.7 — Commit `Cargo.lock` + `src/main.rs` (one commit)

1. Read `.gitignore` and confirm neither `Cargo.lock` nor `src/main.rs` matches an ignore rule (verified: they do not; `.gitignore` has no Rust/src rules).
2. Run `git status`, confirm exactly two items to stage: untracked `Cargo.lock` and untracked `src/main.rs`; confirm there is no `target/` in the worktree at all (guaranteed by Steps 2.1a/2.6) and that no generated file other than `Cargo.lock` is present or staged (§1 decision 8).
3. Stage both:
   ```
   git add Cargo.lock src/main.rs
   ```
4. Commit:
   ```
   git commit -m "chore: generate Cargo.lock and add minimal main entry point"
   ```

### Step 2.8 — Final verification snapshot

Run on the HOST (single commands, no chains):
- `git status` — clean working tree except pre-existing unrelated items; NO `target/` entry may appear in the output (it must not exist in the mount per Steps 2.1a/2.6 and §1 decision 8).
- `git log --oneline -5` — shows the two new commits on top of the Task-1 state.

No further commands. Do not run `git push` (restricted to workflow step 5), do not merge (step 5), do not add `[DONE]` to the TODO (that is Task 2's 4.6, a separate sub-task).

---

## 3. Step 4.3 applicability (code review & simplification) — REQUIRED

- Code now exists (`Cargo.toml`, `src/main.rs`), so run 4.3: assign `code-reviewer` and `code-simplifier` concurrently per the critical workflow.
- Expected minimum findings given the tiny scope; reviewers must confine themselves to the two committed files. Reviewer may propose a fix plan only if it does not add dependencies, comments, modules, or Cargo sections beyond those in §1/§2 — if a proposal touches those boundaries, it is out of scope and must be rejected/referred to the caller.

## Step 4.4 applicability (documentation) — NOT REQUIRED (codified decision)

- All documentation for this phase (README build instructions, `project-structure.md`, `context.md`) is explicitly allocated to Task 5. Task 2 produces no doc work. Running 4.4 here would duplicate/conflict with Task 5's mandate, so 4.4 is skipped with a clear "not required" message from the docs-specialist (per workflow: return clear msg if not required).
- Corollary: this plan changes NO documentation files and NO `.gitignore` (Task 4 owns it). The only allowed repo changes in this task are `Cargo.toml` + `Cargo.lock` + `src/main.rs` and the two commits.

## Step 4.5a — N/A (no front-end tasks in this phase per global plan).

## Step 4.5b applicability (plan adherence) — REQUIRED

- Assign `architector` to verify 4.2 output against this plan: file contents byte-match §2.2/§2.3, exactly two commits with the specified messages in the specified order, `Cargo.lock` present and tracked, no other files created, no `.gitignore`/docs edits. Report adherence or diffs.

---

## 4. Explicit prohibitions (restate — junior implementer)

- Do NOT create module files or placeholders under `src/{app,config,youtube,chat,renderer,streaming}`. Do not touch the existing `.gitkeep` placeholders there.
- Do NOT add any dependency entries, `[features]`, build scripts (`build.rs`), tests, benches, or examples.
- Do NOT implement config parsing, logging, OAuth, YouTube API calls, rendering, chat handling, or FFmpeg anything.
- Do NOT modify `.gitignore` (Task 4), README/docs/project-info/metadata (Task 5), or `.agent/**`, `.kilo/**`.
- Do NOT create Dockerfile/docker-compose files (Task 3).
- Do NOT run cargo on the Windows host; only host-side git and file creation.
- Do NOT run multi-command chains (`&&`, `&`) or privileged containers (`--privileged`) via the VM MCP.
- Do NOT stage `target/` or any container artifacts beyond the three files listed. `target/` must NOT exist at all in the mount (Steps 2.1a/2.5/2.6); no generated file other than `Cargo.lock` may be written into the project root (§1 decision 8).
- Do NOT omit `-v rust-streamer-target:/rust-streamer-target` or `-e CARGO_TARGET_DIR=/rust-streamer-target` from any cargo docker run in this task (sole exception: the Step 2.1a cleanup command, which must keep the plain form exactly as written there).
- Do NOT push, merge, or mark the TODO task `[DONE]` here.

---

## 5. Mapping to TODO Task 2 sub-items and Phase-00 acceptance criteria

| TODO Task 2 sub-item (20261008-todo-2.md) | Covered by |
|---|---|
| Root `Cargo.toml`, single executable, name consistent with project, valid crate name | Step 2.2 — `rust-youtube-streamer-service` |
| Current stable toolchain in container; document chosen edition | Step 2.2 pins `edition = "2021"`; container pin `rust:1.82` recorded in §1 and delegated check (and to Task 3/5 docs). Sub-agent completion REPORT must repeat the pin + edition explicitly (report item 3 of the Required Completion Report). |
| `src/main.rs` smallest idiomatic entry point proving build | Steps 2.3, 2.5, 2.6 |
| No empty module files for future features | §4 prohibition |
| Generate and commit `Cargo.lock` (executable policy) | Steps 2.5, 2.7 |
| No config/logging/OAuth/YouTube/render/chat/FFmpeg code | §4 prohibition |

Acceptance criteria touched by Task 2:
- "[ ] A minimal Rust executable package exists with `Cargo.toml` and `src/main.rs`." → Steps 2.2/2.3 (checkbox completion belongs to Task 2's 4.6 sub-task, not here).
- "[x→] `Cargo.lock` is present and tracked where appropriate." → Steps 2.5/2.7.

Other acceptance criteria (Docker setup, gitignore, fmt/test/clippy results, README/structure docs) are owned by Tasks 3/4/5 — explicitly out of this plan.

---

## 6. Completion report requirement for the implementer

At the end of Step 4.2 (and again merge-aware at 4.6), the implementer must report:
1. Files created/changed: `Cargo.toml`, `src/main.rs`, `Cargo.lock` (+ two commits).
2. Exact MCP/VM commands executed (Steps 2.1, 2.1a, 2.5, 2.6) with their actual results/exit statuses — including the Step 2.1a conditional clean + its verification `ls`, and the post-check `ls` proving no `target/` in the mount; every cargo docker run reported must show both named-volume mount/env verbatim (§1 decision 8).
3. Container image + toolchain actually used (rust:1.82 edition 2021; cargo version printed by `cargo --version` may be captured as minor local detail via one extra single command — permitted, optional; report actual output, not expectations): `docker run --rm -v /rust-youtube-streamer:/rust-youtube-streamer -v rust-streamer-target:/rust-streamer-target -e CARGO_TARGET_DIR=/rust-streamer-target -w /rust-youtube-streamer rust:1.82 cargo --version`.
4. Scope of validation: compile check inside a Linux (Debian-based rust image) container on the Alpine VM via the shared mount, with Cargo artifacts redirected to the named volume `rust-streamer-target` (`CARGO_TARGET_DIR=/rust-streamer-target`) — NOT runtime validation on Windows/Linux.
5. Checks not run and why (fmt/test/clippy deferred to Task 3 by plan).
6. Assumptions or blockers (e.g., image pull failures — full error text, no tag swaps).
7. Confirmation no out-of-scope feature was implemented.

---

## 7. Plan verification against the original task

- All sub-items of TODO "### 2. Establish the Minimal Cargo Project" are encoded (mapping table §5). ✓
- Exact full file contents specified, not left as judgment. ✓
- Lockfile generation order correct (Task 2 before Task 3 → explicit `docker run` + mount, not compose). ✓
- Stop-and-ask codified for image pull failure and all blockers. ✓
- Single validation command boxed in (`cargo check --locked`), suite explicitly deferred to Task 3. ✓
- Git staging groups and messages exactly codified; gitignore compliance enforced pre-commit; Cargo.lock committed; no tag swaps; host never runs cargo. ✓
- 4.3 required / 4.4 not required / 4.5b required — codified. ✓
- Caller amendment (2026-10-08) encoded: named-volume target redirect on every cargo docker run (§1 decision 8), stale-target cleanup step 2.1a before the first cargo command, `Cargo.lock`-only-at-root guarantee + no-`target/`-reappear `ls` verification, Task-3 compose decision record (§8). ✓

---

## 8. Decision record — Cargo target-dir named volume (caller decision, 2026-10-08)

- All cargo build artifacts are redirected to the Docker **named volume** `rust-streamer-target` (container path `/rust-streamer-target`) via `CARGO_TARGET_DIR=/rust-streamer-target`, so that nothing but source files and the required `Cargo.lock` live in the project root, and to avoid the silent rmeta mmap failure observed on the VirtualBox shared-folder mount.
- The volume name is FIXED: `rust-streamer-target` (Docker auto-creates the named volume on first use; no manual `docker volume create` step is in scope).
- **Binding note for downstream Task 3 planning:** the docker-compose service defined by Task 3 MUST replicate BOTH `-v rust-streamer-target:/rust-streamer-target` AND `CARGO_TARGET_DIR=/rust-streamer-target` for every cargo-invoking workload (fmt/check/test/clippy). Any Task 3 plan must carry these two elements forward verbatim, and inherits the rule that `target/` must not reappear under `/rust-youtube-streamer` (verified via `ls` after cargo commands).
- The Step 2.1a cleanup command is the ONLY deliberate docker run in this task WITHOUT the named volume/env — its sole purpose is to remove the stale in-mount `target/`, conditionally and only if it exists.
