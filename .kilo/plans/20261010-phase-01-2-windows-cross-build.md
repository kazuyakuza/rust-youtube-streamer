# Global Plan — Phase 01.2: Reproducible Windows Executable Build

- Source TODO: `.agent/todos/20261010/20261010-todo-1.md` (Pattern C: 4 tasks under `## Tasks`).
- Date: 2026-10-10. Front-end related: **none** of the 4 tasks (build tooling + docs) → sub-steps 4.1a/4.5a are **not** executed for any task; every task runs 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6.
- TODO does not contain "Don't request me to approve plans" → present this global plan to the user for approval before step 4 cycles.

## Global Pre-Analysis (technical & architecture decisions)

### Baseline (live-verified)

- Repo on `main`, up to date with `origin/main`; `Cargo.toml` v0.3.0; deps serde/serde_json/tracing/tracing-subscriber (+dev tempfile) — all pure-Rust, no C deps, no CC-crate usage.
- `Dockerfile`: pinned `FROM rust:1.82` (Debian-based, apt available) + `rustup component add rustfmt clippy`.
- `docker-compose.yml`: service `rust`, `working_dir /rust-youtube-streamer`, bind-mount project, `CARGO_TARGET_DIR=/rust-streamer-target` named volume. **No Compose changes needed.**
- `scripts/build-linux.sh` (POSIX sh; precondition checks; `build-linux: error:` prefix; exit 1) is the template for the new Windows script.
- `.gitignore` already has generic `dist/` rule; `dist/` is the approved root-level artifact exception.
- Working tree has two **unstaged deletions** (`config/.gitkeep`, `logs/.gitkeep`) not caused by any task.

### Technical decisions (encoded; architector may refine details, not directions)

1. **Target**: `x86_64-pc-windows-gnu` via `rustup target add` inside the Dockerfile (image-level, reproducible). Rust stays pinned at `1.82`; no dependency changes; no application source changes unless the cross-build exposes a proven portability issue (stop + document first).
2. **Linker/toolchain route (decision order)**:
   a. Default: rustc's self-contained linking for cross `x86_64-pc-windows-gnu` (rust-mingw component installs bundled CRT objects + import libs; rust-lld drives the link — verified direction via rustc target spec `LinkSelfContainedDefault::InferredForMingw`).
   b. Dockerfile also installs Debian `gcc-mingw-w64-x86-64` (minimum: pulls `binutils-mingw-w64-x86-64` + `mingw-w64-x86-64-dev`) so `x86_64-w64-mingw32-gcc` and `x86_64-w64-mingw32-objdump` exist — fallback link route and import-audit tooling.
   c. If the plain `cargo build --target x86_64-pc-windows-gnu` link fails, fallback is a tracked `.cargo/config.toml` with `[target.x86_64-pc-windows-gnu] linker = "x86_64-w64-mingw32-gcc"`. Any config file added is documented. Bold decision made; implementer does not invent others.
3. **Runtime DLL handling**: expected outcome is a self-contained static GNU runtime link importing only standard Windows system DLLs (`msvcrt.dll`, `kernel32.dll`, `advapi32.dll`, `ntdll.dll`, `user32.dll`, `userenv.dll`, `ws2_32.dll`). Decision algorithm during implementation: enumerate imports with `x86_64-w64-mingw32-objdump -p <exe>`; if any non-system DLL appears (e.g. `libgcc_s_seh-1.dll`, `libwinpthread-1.dll`), prefer static remediation flags in the script; if remediation fails, copy only those DLLs next to the `.exe`. Final observed result is documented, never assumed claimed. A successful link alone is NOT runtime validation.
4. **Script**: `scripts/build-windows.sh` mirrors `build-linux.sh` conventions exactly (same check names adapted: `target x86_64-pc-windows-gnu` added to the cargo invocation; source path `$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/rust-youtube-streamer-service.exe`; artifact `dist/windows/rust-youtube-streamer-service.exe`; `build-windows: error:` prefix; no diagnostics suppression; never deletes/prunes anything).
5. **VM tooling constraint (alpine-vm MCP)**: vm command allowlist starts with `docker|sh|apk|ls|cat|ps|df|free|uname|pwd|whoami` — **`git` is not allowed on the VM**; all git checks run on the repo host. `vm_status` must be called before any `vm_run_command`. Build commands get generous `timeoutMs` (cold windows-gnu compile compiles all deps for the new target; allow ≥ 900 s).
6. **Git/versioning**: feature branch `feat/windows-gnu-cross-build`; version minor bump `0.3.0 → 0.4.0` (new build feature) with `Cargo.lock` package-entry refresh via a container `cargo check` then `--locked` re-verification (Phase 01 precedent). Push at step 5 to `origin/main` only.
7. **Placeholder deletions**: restore `config/.gitkeep` and `logs/.gitkeep` (`git restore`) in step 2 — they are tracked HEAD content and the project structure defines both; no task requests their removal. No commit needed for them after restore.
8. **Docs**: `docs/build-windows.md` (modelled on `docs/build.md`, TOC since >100 lines) + README new section + TOC entry; distinction of the three workflows (`dev-checks.sh` / `build-linux.sh` / `build-windows.sh`); "cross-compile ≠ native Windows runtime validation" stated everywhere claims are made; `.agent/project-structure.md` + `.agent/project-info/context.md` updated in Task 3.
9. **Reports**: completion report at `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` (tracked) with real evidence only — no invented outputs/sizes/statuses.

## Execution Steps (each entry = one `task` tool invocation)

1. **Step 2: Git Feature Branch Setup** → implementer
   - Restore the two placeholder deletions; verify clean status; ensure on `main`; create + switch to `feat/windows-gnu-cross-build`.
2. **Step 3: Version Update** → implementer
   - Bump `Cargo.toml` to 0.4.0; refresh `Cargo.lock` entry via container `cargo check` (VM), verify `cargo check --locked`; commit `chore: bump version to 0.4.0`.
3. **Task 1 — Add Windows GNU Cross-Compilation Support**
   - 4.1b plan (architector) → 4.2 implement (Dockerfile: minimal mingw-w64 x86_64 apt packages + `rustup target add x86_64-pc-windows-gnu`; rebuild image; smoke-verify target/linker/objdump present; optional early `--target` compile probe) → 4.3 reviewer+simplifier (+fixes by implementer) → 4.4 docs (Dockerfile comments only; skip-style "not required" allowed) → 4.5b adherence → 4.6 `[DONE]` marker + commit `feat: add windows gnu cross toolchain to docker build image`.
4. **Task 2 — Produce a Windows Artifact Safely**
   - 4.1b plan → 4.2 implement `scripts/build-windows.sh` + DLL-handling decision algorithm (objdump audit; static-preference; copy-DLLs only if required) + real container run of the script → 4.3 review+simplify → 4.4 docs (script comments only) → 4.5b adherence → 4.6 commit `feat: add windows release build script`.
5. **Task 3 — Document and Integrate the Workflow**
   - 4.1b plan → 4.2 implement docs: `docs/build-windows.md`, README section + TOC, `.agent/project-structure.md`, `.agent/project-info/context.md` (three-workflow distinction; limitations; platform model unchanged) → 4.3 review+simplify → 4.4 docs-specialist polish → 4.5b adherence → 4.6 commit `docs: document windows build workflow`.
6. **Task 4 — Verify Both Build Paths and Save a Completion Report**
   - 4.1b plan → 4.2 execute verifications via alpine-vm MCP (windows build cmd, linux build cmd, dev-checks cmd — record exact exit statuses/sizes) + host-side gitignore/tracking evidence + create tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` + flip acceptance-criteria checkboxes `[x]` in the TODO (only real evidence) → 4.3 review (report/docs only) → 4.5b adherence → 4.6 commit `docs: add phase 01.2 completion report and mark tasks done`.
7. **Step 5: TODO File Completion** → implementer
   - Rename TODO file with `-DONE` suffix (content untouched); review/remove any tmp files; ensure all committed; merge `feat/windows-gnu-cross-build` → `main`; delete branch on success; push `main` to `origin` only.

## Acceptance Mapping (from TODO)

- Windows GNU target + linker reproducibly installed by Docker build → Task 1 (+ Dockerfile evidence).
- `scripts/build-windows.sh` locked-build → `dist/windows/rust-youtube-streamer-service.exe` → Task 2 (+ Task 4 verification).
- Actionable errors, non-empty artifact check, no suppression → Tasks 2/4.
- GNU runtime DLL decision + evidence → Task 2 (objdump audit) documented in Tasks 3/4.
- Linux build unchanged success + dev-checks pass → Task 4.
- `docs/build-windows.md`, README, structure/context updates → Task 3.
- dist/ ignores; no root `target/`; report with real evidence; no Phase 02 scope → Tasks 2–4.
