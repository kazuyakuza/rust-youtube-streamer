# Task 1 Implementation Plan — Add Windows GNU Cross-Compilation Support (Docker Image Toolchain)

- Source TODO: `.agent/todos/20261010/20261010-todo-1.md`, section `### 1. Add Windows GNU Cross-Compilation Support` (read it first).
- Global plan: `.kilo/plans/20261010-phase-01-2-windows-cross-build.md` (binding decisions #1, #2).
- Scope: **Task 1 only** (sub-step 4.1b plan → this file; execution happens in a later step). NOT Task 2/3/4.
- Front-end: none → no sub-step 4.1a.

## Planning-time evidence (verified live by the architector, 2026-10-10)

1. Current image tooling (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "rustup component list --installed && rustup target list --installed"`, exit 0): only host target `x86_64-unknown-linux-gnu`; components cargo/clippy/rust-std/rustc/rustfmt. No mingw bits. `rustc 1.82.0` (LLVM 19.1.1, host `x86_64-unknown-linux-gnu`).
2. **Hard requirement found**: a throwaway-container probe WITHOUT the mingw package
   (`docker run --rm -v /rust-youtube-streamer:/proj rust:1.82 sh -c "rustup target add x86_64-pc-windows-gnu && cd /proj && CARGO_TARGET_DIR=/tmp/t cargo build --release --locked --target x86_64-pc-windows-gnu"`, exit **101**) failed at COMPILE time:
   `error: Error calling dlltool 'x86_64-w64-mingw32-dlltool': No such file or directory` while compiling `windows-sys v0.61.2` (it uses raw-dylib imports, which make rustc invoke `x86_64-w64-mingw32-dlltool` to generate import libraries). So Debian `gcc-mingw-w64-x86-64` (pulls `binutils-mingw-w64-x86-64` + `mingw-w64-x86-64-dev`) is **mandatory**, not merely a linker fallback.
3. **Route validated end-to-end**: same probe WITH `apt-get update -qq && apt-get install -y -qq --no-install-recommends gcc-mingw-w64-x86-64` succeeded (exit 0): `Finished release profile [optimized]` in 33.38 s; `x86_64-w64-mingw32-gcc`, `x86_64-w64-mingw32-objdump`, `x86_64-w64-mingw32-dlltool` present under `/usr/bin/`; exe built at 3 191 192 bytes. Linking succeeded **without any `.cargo/config.toml`** (rustc's self-contained windows-gnu linking drove the link; the external mingw gcc is only the fallback route).
4. Note: on `rust:1.82`, `rustup target add x86_64-pc-windows-gnu` installs only `rust-std-x86_64-pc-windows-gnu` (no separate `rust-mingw` component is pulled in on this image) — irrelevant in practice because the CRT/import work is covered by rustc self-contained linking + the apt mingw package; verify by output, not by component name.
5. Repo state at planning: branch `feat/windows-gnu-cross-build`, working tree clean, HEAD `87159a9 chore: bump version to 0.4.0`; `Cargo.toml` v0.4.0; `Dockerfile` is 3 lines (`FROM rust:1.82` + one `RUN rustup component add rustfmt clippy`).

## Execution Environment Roles

- **HOST** = Windows implementation host, working directory `C:\repo\rust-youtube-streamer`. Only git and file edits happen here. Never run `cargo`/`rustup`/`docker` on the host.
- **VM (MCP)** = Alpine VM via `alpine-vm_vm_status` + `alpine-vm_vm_run_command`. Allowlist prefixes: `docker|sh|apk|ls|cat|ps|df|free|uname|pwd|whoami`. **Before EVERY `vm_run_command`, call `alpine-vm_vm_status`.** Compose commands always take the form `docker compose -f /rust-youtube-streamer/docker-compose.yml ...`. Exit codes from `vm_run_command` are authoritative.

---

## Step 1 — HOST: edit `Dockerfile` (exact content)

The current file is 3 lines. Replace its entire content with **exactly** these lines (7 lines; comment lines 1–3 explain the two new layers; nothing else added):

```dockerfile
# Local dev-check toolchain: pinned base rust:1.82 + baked-in rustfmt/clippy (the base image ships without them).
# gcc-mingw-w64-x86-64 provides the Windows GNU cross toolchain: dlltool is required at COMPILE time by
# windows-sys raw-dylib imports, gcc is the fallback linker route, objdump audits the exe imports.
FROM rust:1.82
RUN rustup component add rustfmt clippy
RUN apt-get update && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64
RUN rustup target add x86_64-pc-windows-gnu
```

Rules: keep `FROM rust:1.82` (Rust pin unchanged); keep the rustfmt/clippy line verbatim; no `-qq`, no `rm -rf /var/lib/apt/lists/*`, no other layers, no rearrangement. File stays ≤ 7 lines (max-lines rules satisfied).

Do NOT touch: `docker-compose.yml`, `Cargo.toml`, `Cargo.lock`, anything in `src/`, `scripts/`, `dist/`, docs (Tasks 3/4 own docs), TODO files.

---

## Step 2 — VM (MCP): rebuild the image

Call `alpine-vm_vm_status` first, then:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml build rust
```

- `timeoutMs`: `900000` (apt install of mingw-w64 takes several minutes on first run).
- Expected: exit code `0`; build log shows the apt layer installing `gcc-mingw-w64-x86-64` (with dependencies `binutils-mingw-w64-x86-64`, `mingw-w64-x86-64-dev`) and the `rustup target add` layer completing.
- On failure: capture `combinedOutput` verbatim and STOP — report to caller. Do not retry with altered/debloating variants.

---

## Step 3 — VM (MCP): smoke verification (3 commands, sequential, each after `vm_status`)

3.1 Target installed:
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust rustup target list --installed
```
Expected exit `0`; stdout lists BOTH `x86_64-pc-windows-gnu` AND `x86_64-unknown-linux-gnu` (Linux stay kept; the image must keep both).

3.2 MinGW tools present:
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "x86_64-w64-mingw32-gcc --version && x86_64-w64-mingw32-objdump --version && which x86_64-w64-mingw32-dlltool"
```
Expected exit `0`; gcc + objdump print version banners to stdout; `which` prints `/usr/bin/x86_64-w64-mingw32-dlltool`.

3.3 Rust pin preserved:
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust rustc --version --verbose
```
Expected exit `0`; `rustc 1.82.0 (…)` with host `x86_64-unknown-linux-gnu`.

Any non-zero exit here → capture output and STOP; report the exact command and code. Do not "fix" by downloading anything else.

---

## Step 4 — VM (MCP): compile probe (cold windows-gnu build through the Compose service)

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo build --release --locked --target x86_64-pc-windows-gnu
```

- `timeoutMs`: `900000` (the earlier real cold probe finished in ~34 s after downloads inside a warm network; but the compiler for the new target compiles the whole dep tree fresh in the named volume — keep the generous timeout).
- Expected: exit code `0`; output ends with compilation of `rust-youtube-streamer-service v0.4.0 (/rust-youtube-streamer)` and a `Finished` release line (cargo may print `Finished` on stderr; the MCP combined output shows it).
- The tracked `Cargo.lock` must NOT be modified (`--locked` guarantees the run fails instead of rewriting).

Then verify artifact placement (created inside the named volume, never the project root):

4.1
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe"
```
Wait — compose injects `CARGO_TARGET_DIR` into the container, not the VM shell; use the literal directory:
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-streamer-target/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe
```
Expected exit `0` and a non-empty `-rwxr-xr-x` (or similar executable) listing ~3.2 MB.

4.2 No root `target/` directory:
```
ls /rust-youtube-streamer
```
(`ls` is allowlisted on the VM host; the shared path is visible at the VM host level.) Expected: `target` ABSENT from the listing; everything else unchanged from the known structure. If `target` appears, STOP and report — volume redirection regressed, do not commit anything.

4.3 HOST: `git status --porcelain` — expected exactly `?? Dockerfile` is NOT yet applicable (tracked file, modified): expected exactly ` M Dockerfile` (plus nothing else). If `Cargo.lock` is modified → STOP, report (never expected at this step).

---

## Step 5 — Conditional fallback (ONLY if Step 4's link step failed)

Trigger condition: Step 4 exit code ≠ 0 AND cargo output shows a LINK/`dlltool`/`ld` error (not a Rust compile error or a lockfile error).

5.1 HOST: create `C:\repo\rust-youtube-streamer\.cargo\config.toml` with EXACTLY this content (3 lines):

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

5.2 VM (MCP): re-run the Step 4 probe command exactly, `timeoutMs` `900000`. Expected exit `0`.

5.3 If it STILL fails: STOP. Report the exact commands, exit codes and full `combinedOutput` to the caller. **Do not invent any other route** (no extra apt packages, no build-std, no toolchain bumps, no flags hacks).

5.4 Scope note: `.cargo/config.toml` is created ONLY when this fallback triggers. With the route validated in planning (evidence item 3), the expected outcome is NO config file; the plain Dockerfile route links successfully.

---

## Step 6 — HOST: commit

6.1 `git status` (full) — review; read `.gitignore` first per gitignore-compliance rule. Confirm staged candidates are ONLY `Dockerfile` (and `.cargo/config.toml` iff Step 5 ran). Nothing gitignore-matching may be staged.

6.2 Stage exactly:
```
git add Dockerfile
```
(and additionally `git add .cargo/config.toml` ONLY if the Step-5 fallback was used.)

6.3 Verify staging:
```
git diff --cached --name-only
```
Expected: `Dockerfile` alone (plus `.cargo/config.toml` iff fallback used).

6.4 Commit:
```
git commit -m "feat: add windows gnu cross toolchain to docker build image"
```
Expected exit `0`. Commit message fixed — no scope/so that suffix, no body.

6.5 Post-commit: `git status --porcelain` → clean (or only unrelated pre-existing entries if any surfaced; there were none at planning).

6.6 NEVER `git push` (push happens at workflow step 5, not here). NEVER edit TODO markers (Task 4 closes them). NEVER write to `dist/`.

---

## Regression guard (no commands, config edits, or additions — only non-actions)

- `scripts/build-linux.sh`, `scripts/dev-checks.sh`, `docker-compose.yml`, `Cargo.toml`, `Cargo.lock`, `src/**`, `docs/**`, `README.md` are UNTOUCHED by Task 1. Their unchanged status is the regression proof (verify with `git status --porcelain` and `git diff --stat` showing only `Dockerfile`).
- The image keeps both installed targets (Step 3.1), so Linux builds (run by `scripts/build-linux.sh` in the same rebuilt image) remain functional. Their end-to-end verification belongs to Task 4 — do not pre-run the full scripts here.
- Do NOT create `.cargo/config.toml` unless Step 5 triggered (evidence shows the default self-contained route links without it).
- Do NOT bundle FFmpeg or install any audio/encoder packages. Do NOT add MSVC tooling, 32-bit targets, or extra Rust targets.

---

## Verification checklist (against TODO Task 1 bullets)

- [ ] `x86_64-pc-windows-gnu` target reproducibly installed by the existing Docker build (Dockerfile layer; Step 3.1 lists it).
- [ ] Minimum required Windows GNU cross-linker/toolchain packages installed via the base image's package manager (apt; `gcc-mingw-w64-x86-64`; Step 3.2 passes).
- [ ] Rust stays pinned at `1.82` (Step 3.3; Dockerfile `FROM rust:1.82`).
- [ ] Existing Linux target still installed (Step 3.1 lists both); Compose service behavior unchanged (no Compose edits; `git status` shows only `Dockerfile`).
- [ ] Target-specific build with tracked lockfile proven working: Step 4 exit 0 with `--release --locked --target x86_64-pc-windows-gnu` (this is a probe, not a script; the scripted artifact step is Task 2).
- [ ] All intermediate Cargo artifacts stayed in the named volume (Steps 4.1/4.2: artifact inside `/rust-streamer-target/…`; no root `target/`).
- [ ] No new crate dependencies (no `Cargo.toml`/`Cargo.lock` change; `git status` must show them untouched).
- [ ] `Dockerfile` clean build documented inline only (comments within the file; no doc files).

## Explicitly NOT done in this step (belongs elsewhere)

- Task 2: `scripts/build-windows.sh`, DLL import audit (objdump-based), `dist/windows/` artifact handling.
- Task 3: `docs/build-windows.md`, README, `.agent/project-structure.md`, `.agent/project-info/context.md` updates.
- Task 4: full verification of both build paths + completion report + TODO marker changes.
- Any push, branch merge, or further Cargo/versioning changes (version bump already committed at planning-time HEAD `87159a9`).
