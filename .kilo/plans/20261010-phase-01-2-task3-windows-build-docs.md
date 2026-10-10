# Plan — Phase 01.2, Task 3, sub-step 4.1b: Document and Integrate the Windows Build Workflow

- Source TODO: `.agent/todos/20261010/20261010-todo-1.md`, section `### 3. Document and Integrate the Workflow` (all bullets).
- Global plan: `.kilo/plans/20261010-phase-01-2-windows-cross-build.md` (decisions #2/#3/#5/#8/#9).
- This is the 4.1b (architector) step only. Implementation happens in the later 4.2 step by the implementer. Non-front-end: no 4.1a step.
- Scope: exactly five files are created/edited (`docs/build-windows.md`, `README.md`, `docs/build.md`, `.agent/project-structure.md`, `.agent/project-info/context.md`) plus this plan file. Everything else is explicitly OFF-limits (see §8).

## 0. Facts the Docs Must Encode (VERIFIED — never weaken into vague claims)

These facts are binding for the documentation content. Do not relax, soften, or re-derive them:

1. Standard command (verbatim, appears exactly in README section and docs/build-windows.md):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`
   observed exit 0.
2. Image prerequisite: build the image once first with
   `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust`
   (the Dockerfile at HEAD is `FROM rust:1.82` + `rustup component add rustfmt clippy` + `apt-get install gcc-mingw-w64-x86-64` + `rustup target add x86_64-pc-windows-gnu` on the pinned Debian-based image).
3. Artifact on success: `dist/windows/rust-youtube-streamer-service.exe`, non-empty. The byte size VARIES with codebase changes and MUST NOT be hardcoded anywhere in the docs; sizes are printed by the script at run time (rule mirrors `docs/build.md` line 43).
4. Cold first windows-target run compiles the whole dependency tree for the new target once; later runs are incremental because `CARGO_TARGET_DIR=/rust-streamer-target` (Docker named volume) keeps intermediates. No root `target/` ever appears. Linux intermediates share the same volume.
5. Runtime DLL decision (blanket statement from `scripts/build-windows.sh` line 73, verbatim basis): the GNU runtime is statically linked; the observed import audit via `x86_64-w64-mingw32-objdump` found no MinGW runtime DLLs and only standard Windows system DLLs are required: `msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll`. These names were verified by the objdump audit — quote them exactly; do not add or invent others.
6. Linux artifact path is untouched by the Windows workflow (`dist/rust-youtube-streamer-service` remains produced solely by `scripts/build-linux.sh`).
7. Failure paths (probe-tested; exact `build-windows: error:` texts from the script, path/code placeholders angle-bracketed):
   | # | Condition | Message (behind the `build-windows: error: ` prefix) | Exit |
   |---|---|---|---|
   | 1 | wrong cwd | `expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: <dir>` | 1 |
   | 2 | empty/unset `CARGO_TARGET_DIR` | `CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)` | 1 |
   | 3 | cargo build fails | `cargo build --release --locked --target x86_64-pc-windows-gnu failed with exit code <code>; the cargo diagnostics above are not suppressed` | 1 |
   | 4 | built exe missing/empty | `expected release executable not found or empty: <path>` | 1 |
   | 5 | `dist/windows/` not creatable | `could not create the dist/windows/ directory: dist/windows` | 1 |
   | 6 | copy fails | `failed to copy <source> to <destination>` | 1 |
   | 7 | final artifact missing/empty after copy | `final artifact is missing or empty after the copy: dist/windows/rust-youtube-streamer-service.exe` | 1 |
   Nothing is suppressed: Cargo diagnostics print in full before the error line in case #3.
8. Success lines (script lines 72–74 — quote in Expected Output): `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (<N> bytes).` then the static-linking/audit line (fact 5) then `Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled.`
9. Environment guidance (goes into context.md per the caller): repo-host git has `core.autocrlf=true` — text files materialize CRLF on the working tree; only `*.sh` and `Dockerfile` are LF-forced via `.gitattributes`. Phase-incident note: a stray working-tree-wide CRLF conversion earlier today broke `sh scripts/build-linux.sh` until the files were re-normalized; the guard `.gitattributes` now prevents recurrence.
10. Linux artifact current observed size as of the v0.4.0 tree: 1715240 bytes (fresh observation in the context log only — sizes change with code; never hardcoded in user docs).
11. Platform model unchanged: the `.exe` is a normal user executable — no installer, no Windows service registration, no elevation; FFmpeg remains external and is not installed by this project.

## 1. File 1 (NEW): `docs/build-windows.md`

Full doc, mirroring `docs/build.md`'s section style (title sentence, `##` sections, tables). Expected >100 lines → TOC is MANDATORY. Create at exact path `docs/build-windows.md`.

Title (line 1): `# Build the Windows Release Executable (x86-64 GNU Cross-Compile)`

Opening sentence mirrors build.md style: the guide explains how to cross-compile the distributable Windows x86-64 release executable `dist/windows/rust-youtube-streamer-service.exe` inside the pinned Linux Rust container via MinGW-w64 GNU tooling.

Sections in EXACTLY this order, with these exact headings/anchors:

1. `## Table of Contents` — one bullet per section below using anchors: `#purpose`, `#prerequisites`, `#build-command`, `#expected-output`, `#overwrite-and-rebuild-behavior`, `#failure-semantics`, `#toolchain-and-target-choice`, `#runtime-dlls`, `#what-this-proves-and-does-not-prove`, `#dev-checks-vs-release-build`, `#see-also`.
2. `## Purpose` — three bullets, mirroring build.md: (a) the repository now ships three Docker-based workflows; this guide covers only the Windows cross-build; (b) the build cross-compiles in release mode (`cargo build --release --locked --target x86_64-pc-windows-gnu`) inside the pinned Linux `rust:1.82` container using the MinGW-w64 GNU toolchain and leaves exactly one final `.exe` behind; (c) success criterion: the command exits `0` and prints, among its messages, the success line of fact 8 (size `<N>` explained as run-time-printed).
3. `## Prerequisites` — mirrors build.md: same Docker prerequisites as the checks/Linux build, linked to `../README.md#build-checks-docker-via-alpine-vm` (Alpine VM with project at `/rust-youtube-streamer`, Docker + Compose plugin v2.31.0 observed); **image-build note (fact 2)**: the container image must be (re)built once with `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust` so it contains the Windows GNU cross toolchain — the current Dockerfile installs `gcc-mingw-w64-x86-64` and the Rust target `x86_64-pc-windows-gnu` on top of pinned `rust:1.82`; no Rust/Cargo needed on the host; `--locked` always compiles against the tracked `Cargo.lock`.
4. `## Build Command` — the standard invocation verbatim (fact 1) in a fenced code block, exactly as facts 1 states it; bullets: it runs the tracked POSIX script `scripts/build-windows.sh` through the Compose `rust` service; first cold run compiles the whole tree for the windows-gnu target once and takes noticeably longer than the Linux build (allow generous time); later runs are incremental via the named volume; Cargo diagnostics are shown as they happen, never suppressed.
5. `## Expected Output` — bullets: output path `dist/windows/rust-youtube-streamer-service.exe` (fact 3); the script prints the exact path and byte size at run time and this guide never hardcodes the number (fact 3 rule); `dist/` is gitignored (never committed); Cargo intermediates live only in the named volume `rust-streamer-target` — no root `target/`; on success the script additionally prints the static-linking/DLL line and the cross-compile caveat line (fact 8). Mention that the workflow does not touch the Linux artifact path (fact 6).
6. `## Overwrite and Rebuild Behavior` — mirror build.md wording adapted to `dist/windows/`: rerunning overwrites the previous `.exe` with a fresh copy; incremental rebuilds via the named volume; `--locked` fails fast if `Cargo.lock` mismatches; the artifact is disposable and regenerable; on a failed run nothing is placed in `dist/windows/` and a previous artifact stays untouched (script only writes after a fully successful build-and-copy sequence); nothing is ever pruned or deleted.
7. `## Failure Semantics` — lead sentence mirrors build.md line 56 adapted: every failure line begins with the literal prefix `build-windows: error:` and the command exits `1`; one or more error lines appear on standard error, and Cargo diagnostics are printed in full, never suppressed. Then a 7-row table using the EXACT message texts from §0 fact 7 (columns: # | If this happens | What you see). Closing sentence mirrors build.md line 68: a non-zero exit means no new artifact was produced; fix the reported condition and run the standard command again.
8. `## Toolchain and Target Choice` — bullets: the target is `x86_64-pc-windows-gnu` because the build runs inside a Linux container and the MSVC cross toolchain is explicitly out of scope for this phase (targeting 32-bit Windows and other platforms touches only MSVC out-of-scope; this phase covers 64-bit Windows only); Rust stays pinned at `1.82`; the Debian packages come from apt (`gcc-mingw-w64-x86-64`, installed by the Dockerfile) and the Rust target comes from `rustup target add`; the image is rebuilt reproducibly by the tracked Dockerfile; no Rust crate dependencies were added for cross-compilation.
9. `## Runtime DLLs` — bullets: the GNU runtime is statically linked (decision basis = the script's own success output, fact 5); the `x86_64-w64-mingw32-objdump` import audit observed NO MinGW runtime DLLs, only ordinary Windows system DLLs (list them verbatim from fact 5); therefore a user installing the `.exe` on a standard 64-bit Windows machine needs to install no MinGW runtime DLLs and no redistributables; the audit's scope is import-table inspection, not execution.
10. `## What This Proves and Does Not Prove` — CRITICAL section. Proves: the current source cross-compiles in release mode for `x86_64-pc-windows-gnu` with the tracked lockfile inside the Linux container and yields a non-empty Windows `.exe`. Does NOT prove: cross-compilation does NOT equal native Windows runtime validation — the built `.exe` must be tested separately on a real Windows host, and this documentation reports such validation as NOT RUN unless evidence from a Windows environment exists; it does not validate OAuth/`run` streaming behavior (placeholders exit with code 3 after config validation); FFmpeg remains external — the app does not install it (fact 11 + fact 8 caveat). Also a platform-model bullet: the executable is a normal user executable — no installer, no Windows service registration, no elevation.
11. `## Dev Checks vs. Release Build` — heading spelled exactly as `docs/build.md` (anchor `#dev-checks-vs-release-build`), but the body distinguishes THREE workflows (fact list from TODO Task 3): `scripts/dev-checks.sh` — Linux-container formatting/check/tests/clippy, no artifact, log to gitignored `logs/checks/`; `scripts/build-linux.sh` — Linux release executable at `dist/rust-youtube-streamer-service`; `scripts/build-windows.sh` — Windows x86-64 GNU release `.exe` at `dist/windows/rust-youtube-streamer-service.exe`. Note: all three are plain POSIX `sh` invoked the same way through the Compose service; neither modifies source files; the Windows and Linux builds produce different artifacts from the same container.
12. `## See Also` — bullets: `../README.md` (`Release Build (Windows .exe)` section and Build Checks section); `./build.md` (the Linux counterpart guide); `scripts/dev-checks.sh` and `scripts/build-windows.sh` path references back to the repository scripts (relative `../scripts/…` links); `./configuration.md` (how the built executable consumes its configuration).

Rules for this file: no hardcoded byte sizes anywhere (rule specific: the failure-table paths may name files but never sizes); real newline characters only (gitignore-compliance & newline rules); docs file is tracked git content (no ignore-matching content); keep the compact factual tone of `docs/build.md`; no commented-out content.

## 2. File 2 (EDIT): `README.md`

Three EXACT edits:

1. **TOC entry** (line 15 area): after the existing line `- [Release Build (Linux Artifact)](#release-build-linux-artifact)` insert exactly:
   `- [Release Build (Windows .exe)](#release-build-windows-exe)`
2. **New section** after the `## Release Build (Linux Artifact)` section (i.e., after its last paragraph, currently line 166, before `## AI Agents`): add `## Release Build (Windows .exe)` containing THREE concise elements:
   - a short opening sentence: to produce a distributable Windows x86-64 release executable, run the repository's Windows build script through the same Compose `rust` service (build the image once first per the Build Checks prerequisites);
   - the standard command verbatim (fact 1) in a fenced code block;
   - a compact paragraph: on success the command exits `0` and leaves the Windows release executable at `dist/windows/rust-youtube-streamer-service.exe` (gitignored directory), cross-compiled for `x86_64-pc-windows-gnu` inside the pinned Linux container using the MinGW-w64 GNU toolchain; the GNU runtime is statically linked so no MinGW runtime DLLs are needed on Windows; this is a cross-compilation and does NOT validate native Windows runtime behavior — the `.exe` must be tested separately on Windows; FFmpeg remains an external prerequisite, not bundled; this is a different job from `scripts/dev-checks.sh` (no artifact) and from `scripts/build-linux.sh` (Linux artifact). Link: `Full prerequisites, output path, rebuild/overwrite behavior, failure semantics, toolchain/target choice, runtime DLLs and limitations: [`docs/build-windows.md`](docs/build-windows.md).`
   - Keep it concise (roughly `docs/build.md`-README-section size; ~15 lines).
3. **Current Status paragraph** (line 51): the existing sentence `Build checks validate the build inside the Linux container only; there is no native Windows/Linux runtime validation yet.` — adjust minimally so it stays true AND mentions the new workflow exists: replace with wording equivalent to: release builds for both the Linux artifact and the Windows x86-64 `.exe` run inside the Linux container (the Windows artifact is cross-compiled); there is still no native Windows/Linux runtime validation. Do not rewrite the rest of the paragraph.

## 3. File 3 (EDIT): `.agent/project-structure.md`

Five EXACT in-place updates, keeping the doc's compact bullet style (single line per item):

1. `scripts/` entry (line 41): append to the same bullet: `; build-windows.sh cross-compiles the Windows x86-64 GNU release in the same service (cargo build --release --locked --target x86_64-pc-windows-gnu) and copies the final .exe to gitignored dist/windows/`.
2. `docs/` entry (line 40): append: `; build-windows.md (Windows x86-64 GNU cross-build guide: prerequisites incl. image rebuild, invocation, output, failure semantics, toolchain/target choice, runtime DLLs, limitations)`.
3. `dist/` entry (line 44): append: `; build-windows.sh writes only the final Windows executable rust-youtube-streamer-service.exe under dist/windows/`.
4. `Dockerfile` line (line 30): replace with: `- Dockerfile - pinned official `rust:1.82` base; installs rustfmt/clippy components for checks, `gcc-mingw-w64-x86-64` (Windows GNU cross toolchain: fallback linker route + objdump import audit) and the `x86_64-pc-windows-gnu` Rust target for the Windows cross-build`.
5. Root files list: add one bullet: `- .gitattributes - forces LF line endings for `*.sh` and `Dockerfile` so container-executed files survive the repo host's `core.autocrlf=true` checkout behavior`.

## 4. File 4 (EDIT): `.agent/project-info/context.md`

Append ONE new section after the existing `## Recent Changes (2026-10-09, Phase 01.1)` section (before `## Known Deviations and Limitations (Phase 01)`), titled exactly `## Recent Changes (2026-10-10, Phase 01.2 implementation)`, plus update `## Immediate Next Steps`.

New section content (factual-log style, bullet list, no secrets):
- Branch `feat/windows-gnu-cross-build`; version bumped `0.3.0 → 0.4.0` (commit `87159a9`).
- Task 1 (commit `e5505e1`): the Dockerfile now installs `gcc-mingw-w64-x86-64` (Windows GNU cross toolchain: fallback linker route + `x86_64-w64-mingw32-objdump` audit tooling) and adds the Rust target `x86_64-pc-windows-gnu` on the pinned `rust:1.82` Debian base; image rebuilt; probe-verified target/linker/objdump presence inside the container.
- Environment note + incident: repo-host git `core.autocrlf=true`; a stray working-tree-wide CRLF conversion earlier today broke `sh scripts/build-linux.sh` execution until the files were re-normalized; new tracked `.gitattributes` forces LF for `*.sh` and `Dockerfile` to prevent recurrence. Root `target/` never appears (named-volume redirect holds).
- Linux artifact fresh observation on the v0.4.0 tree: 1715240 bytes (sizes change with code; no doc hardcoding).
- Task 2 (commit `e775a7b`): `scripts/build-windows.sh` added — mirrors `build-linux.sh` conventions (`build-windows: error:` prefix, exit 1, no diagnostics suppression, never deletes/prunes); first standard-command run exit 0; artifact `dist/windows/rust-youtube-streamer-service.exe` produced non-empty; DLL decision basis: static GNU runtime link — objdump import audit found no MinGW runtime DLLs, only standard Windows system DLLs (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll); probe-tested failure paths: wrong cwd → exit 1 cwd message; empty `CARGO_TARGET_DIR` → exit 1 compose-reference message; cargo build failure → diagnostics shown + exit 1.
- Task 3 (this change): `docs/build-windows.md` created; README gained the `Release Build (Windows .exe)` section + TOC entry; `docs/build.md` See Also links the Windows guide; `project-structure.md` and this context file updated. Cross-compilation ≠ native Windows runtime validation is stated wherever claims are made; native Windows validation remains NOT RUN.
- Task 4 verification is NOT part of this change: the final-tree Linux/Windows/dev-check re-runs and the tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` belong to Task 4.

`## Immediate Next Steps` — replace item 1 (`Execute ... todo-1.md ...`) with wording reflecting the actual remaining state:
1. Task 4 of Phase 01.2 remains: re-run the documented Windows build and Linux release build commands and the `dev-checks.sh` gate on the final tree via the Alpine VM MCP, record exact exit statuses/artifact sizes, verify `dist/` ignore/tracking state, and create the tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md`; then mark TODO tasks done.
2. Steps 5 of the Critical Workflow remain for Phase 01.2: post-completion TODO rename with `-DONE`, review tmp files, merge the feature branch into `main`, and push to `origin/main` only.
Keep items about Phase 02 and the later runtime-prerequisites document unchanged (as existing items 2–4).

## 5. File 5 (EDIT): `docs/build.md`

ONE change only. In `## See Also` (lines 85–89), add a single new bullet linking the Windows counterpart, e.g.:
`- [`docs/build-windows.md`](build-windows.md) — the Windows counterpart guide: x86-64 GNU cross-compilation producing a Windows `.exe` (three different workflows: dev checks, Linux release build, Windows release build).`
No other modification anywhere in `docs/build.md`.

## 6. Files that MUST NOT be touched

`CHANGELOG.md`, `src/**`, `Cargo.toml`, `Cargo.lock`, `scripts/**`, `Dockerfile`, `docker-compose.yml`, `.gitignore`, `.gitattributes`, `.agent/reports/**` (Task 4 creates the new report), `.agent/todos/**`, anything under `.kilo/` other than this plan file.

## 7. Git Actions

1. Before staging: read `.gitignore` and run `git status` (gitignore-compliance rule). Expected pre-state: branch `feat/windows-gnu-cross-build`, clean tree.
2. Stage EXACTLY five paths: `git add docs/build-windows.md README.md docs/build.md .agent/project-structure.md .agent/project-info/context.md`.
3. Verify staged set: `git status` shows exactly those five staged files and nothing else; `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` confirms the artifact stays ignored (`.gitignore` `dist/` rule) — docs never stage ignore-matching files; docs themselves are tracked content.
4. Commit: `git commit -m "docs: document windows build workflow"`.
5. Do NOT push (push happens at global step 5). Do NOT amend, do NOT rename the TODO file (Task 4/global step 5 handle markers/rename).

## 8. Order of Implementation for the Implementer (4.2 step)

1. Create `docs/build-windows.md` (§1). 2. Edit `README.md` (§2). 3. Edit `docs/build.md` (§5). 4. Edit `.agent/project-structure.md` (§3). 5. Edit `.agent/project-info/context.md` (§4). 6. Verification (§9). 7. Git (§7). Regressions on `docs/build.md` must not occur — it receives only the one See Also bullet.

## 9. Docs-Only Verification Self-Check checklist for the Implementer

1. `docs/build-windows.md` contains a Table of Contents and every bullet above (tables, code blocks, and placement missing equals FAIL).
2. TOC anchor resolution: each TOC entry's anchor equals GitHub-style slug of the exact heading spelling chosen in §1 (e.g. `#toolchain-and-target-choice` ↔ `## Toolchain and Target Choice`; `#dev-checks-vs-release-build` ↔ `## Dev Checks vs. Release Build`); fix spelling mismatches.
3. Relative links exist: `../README.md`, `../scripts/dev-checks.sh`, `../scripts/build-windows.sh`, `configuration.md`, `../.agent/...` targets all resolve on disk.
4. The standard command string from §0 fact 1 appears VERBATIM in `docs/build-windows.md` AND in the README's new section (exact bytes; check whitespace).
5. The image-build command from §0 fact 2 appears in `docs/build-windows.md` Prerequisites.
6. NO hardcoded byte sizes anywhere in the new/edited user docs (`grep -n "[0-9] bytes"`-style scan over `docs/build-windows.md`, `README.md`, `docs/build.md` must show only explanatory rule text, not observed sizes).
7. The three-workflow distinction (`dev-checks.sh` / `build-linux.sh` / `build-windows.sh`) appears in both `docs/build-windows.md` (`Dev Checks vs. Release Build` section) and the README section's closing line.
8. The "cross-compilation ≠ native Windows runtime validation / validation NOT RUN on this workflow" statement appears in `docs/build-windows.md` (`What This Proves...`) and the README section.
9. The platform-model statement (normal user executable; no installer; no Windows service; no elevation; FFmpeg external) appears in `docs/build-windows.md`.
10. `docs/build.md` diff is exactly one added bullet; `git diff -- docs/build.md` confirms.
11. If Current Status was edited: the sentence still truthfully asserts no native runtime validation.
12. After commit: `git status` clean; the five files and this plan file are the only Task-3-related commits.
