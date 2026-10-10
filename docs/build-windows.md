# Build the Windows Release Executable (x86-64 GNU Cross-Compile)

This guide explains how to cross-compile the distributable Windows x86-64 release executable `dist/windows/rust-youtube-streamer-service.exe` inside the pinned Linux Rust container using the MinGW-w64 GNU toolchain.

## Table of Contents

- [Purpose](#purpose)
- [Prerequisites](#prerequisites)
- [Build Command](#build-command)
- [Expected Output](#expected-output)
- [Overwrite and Rebuild Behavior](#overwrite-and-rebuild-behavior)
- [Failure Semantics](#failure-semantics)
- [Toolchain and Target Choice](#toolchain-and-target-choice)
- [Runtime DLLs](#runtime-dlls)
- [What This Proves and Does Not Prove](#what-this-proves-and-does-not-prove)
- [Dev Checks vs. Release Build](#dev-checks-vs-release-build)
- [See Also](#see-also)

## Purpose

- The repository ships three separate Docker-based workflows: fast validation checks, a Linux release build and this Windows release build. This guide covers only the Windows cross-build.
- The build cross-compiles in release mode (`cargo build --release --locked --target x86_64-pc-windows-gnu`) inside the pinned Linux `rust:1.82` container using the MinGW-w64 GNU toolchain and leaves exactly one final `.exe` behind.
- Success criterion from the user's perspective: the command exits `0` and prints, among its messages, `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (<N> bytes).` — the byte count `<N>` is printed by the script at run time and is never hardcoded in this guide.

## Prerequisites

- Same Docker prerequisites as the regular checks and the Linux build, documented in [Build Checks (Docker via Alpine VM)](../README.md#build-checks-docker-via-alpine-vm): Alpine VM running with the project shared at `/rust-youtube-streamer`, Docker with the Compose plugin (Compose v2.31.0 observed).
- No Rust or Cargo toolchain is needed on the computer that runs the build command; everything happens inside the container.
- The build always compiles against the version-locked dependency set recorded in the tracked `Cargo.lock`; `--locked` fails fast if it does not match the manifest.

The container image must contain the Windows GNU cross toolchain, so build it (or rebuild it after a `Dockerfile` change) once first:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml build rust
```

- The tracked Dockerfile starts from the pinned `rust:1.82` image and installs `gcc-mingw-w64-x86-64` plus the Rust target `x86_64-pc-windows-gnu` on top of it.

## Build Command

The standard invocation (the single command users run):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
```

- It runs the repository's tracked POSIX script `scripts/build-windows.sh` through the Compose `rust` service from the documented working directory.
- The first cold run compiles the whole dependency tree for the windows-gnu target once and takes noticeably longer than the Linux build — allow generous time; later runs are incremental because the Docker named volume `rust-streamer-target` keeps the intermediate output.
- Cargo's own progress/error output is shown as it happens: the script deliberately does not suppress Cargo diagnostics.

## Expected Output

- Output path on success: `dist/windows/rust-youtube-streamer-service.exe` — one Windows executable in the gitignored `dist/` directory (never committed).
- The script prints the exact path and byte size at run time; the byte count naturally changes as the codebase changes, so this guide never hardcodes it.
- Cargo's intermediate build files never appear in the repository root; they live in the Docker named volume `rust-streamer-target` (shared with the Linux workflows), so no root `target/` directory ever appears.
- On success the script prints the success line shown under [Purpose](#purpose), then these two lines:

```
The GNU runtime is statically linked: the observed import audit (x86_64-w64-mingw32-objdump) found no MinGW runtime DLLs, and only standard Windows system DLLs are required (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll).
Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled.
```

- The workflow does not touch the Linux artifact: `dist/rust-youtube-streamer-service` remains produced solely by `scripts/build-linux.sh`.

## Overwrite and Rebuild Behavior

- Rerunning the standard command rebuilds from the current source and overwrites the previous `dist/windows/rust-youtube-streamer-service.exe` with a fresh copy.
- Intermediate build caches persist in the Docker named volume, so routine rebuilds are incremental; dependency resolution is never re-run — `--locked` fails fast if `Cargo.lock` does not match the manifest (see [Failure Semantics](#failure-semantics)).
- The artifact is disposable: deleting it has no repository effect; run the standard command again to regenerate it at any time.
- On a failed run nothing is placed in `dist/windows/`; if a previous successful artifact exists, it stays in place untouched (the script only writes after a fully successful build-and-copy sequence).
- Nothing else in the workflow removes or prunes anything: Docker volumes are never pruned, and the script never deletes unrelated files under `dist/`.

## Failure Semantics

Every failure line begins with the literal prefix `build-windows: error:` and the command exits `1`; one or more error lines appear on standard error, and Cargo's diagnostics, if any, are printed in full, never suppressed.

| # | If this happens | What you see |
|---|---|---|
| 1 | The command is not run through the documented Compose invocation (e.g. the script runs from some other working directory). | `build-windows: error: expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: <dir>`; exit 1 before any build starts. |
| 2 | The Compose environment variable that names the Cargo output directory (`CARGO_TARGET_DIR`) is unset or empty — in practice: the build was not started by the Compose service above. | `build-windows: error: CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)`; exit 1. |
| 3 | The Cargo release build itself fails (e.g. compile error, or `Cargo.lock` out of date with the manifest). | Cargo's full diagnostics followed by `build-windows: error: cargo build --release --locked --target x86_64-pc-windows-gnu failed with exit code <code>; the cargo diagnostics above are not suppressed`; exit 1. |
| 4 | After a successful build the expected release executable is not found or is empty. | `build-windows: error: expected release executable not found or empty: <path>`; exit 1. |
| 5 | The `dist/windows/` directory cannot be created. | `build-windows: error: could not create the dist/windows/ directory: dist/windows`; exit 1. |
| 6 | Copying the build output to `dist/windows/` fails. | `build-windows: error: failed to copy <source> to <destination>`; exit 1. |
| 7 | After the copy, the final file is missing or zero bytes. | `build-windows: error: final artifact is missing or empty after the copy: dist/windows/rust-youtube-streamer-service.exe`; exit 1. |

A non-zero exit means no new artifact was produced; fix the reported condition and run the standard command again.

## Toolchain and Target Choice

- The target is `x86_64-pc-windows-gnu` because the build runs inside a Linux container, where the MinGW-w64 GNU route is the supported one.
- The MSVC cross toolchain, 32-bit Windows targets and other platforms are explicitly out of scope for this phase; only 64-bit Windows is covered.
- Rust stays pinned at `1.82`.
- The Debian packages come from apt (`gcc-mingw-w64-x86-64`, installed by the tracked Dockerfile) and the Rust target comes from `rustup target add x86_64-pc-windows-gnu` in the same Dockerfile.
- The image is rebuilt reproducibly from the tracked Dockerfile; no Rust crate dependencies were added for cross-compilation.

## Runtime DLLs

- The GNU runtime is statically linked — the decision basis is the script's own success output quoted in [Expected Output](#expected-output).
- The `x86_64-w64-mingw32-objdump` import audit observed NO MinGW runtime DLLs; only ordinary Windows system DLLs are imported: `msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll`.
- Therefore a user installing the `.exe` on a standard 64-bit Windows machine needs to install no MinGW runtime DLLs and no redistributables.
- The audit's scope is import-table inspection, not execution: it does not prove the program runs on Windows (see [What This Proves and Does Not Prove](#what-this-proves-and-does-not-prove)).

## What This Proves and Does Not Prove

- What it proves: the current source cross-compiles in release mode for `x86_64-pc-windows-gnu` against the tracked lockfile, inside the Linux container, and yields a non-empty Windows `.exe`.
- What it does NOT prove:
  - Cross-compilation is NOT native Windows runtime validation: the built `.exe` must be tested separately on a real Windows host, and this documentation reports such validation as NOT RUN unless evidence from a Windows environment exists.
  - It does NOT validate OAuth/`run` streaming behavior — both remain placeholders that report not-implemented and exit with code 3 after configuration validation.
  - FFmpeg remains an external prerequisite: the application does not install it and this workflow does not bundle it.
- Platform model (unchanged): the executable is a normal user executable — no installer, no Windows service registration, no elevation.

## Dev Checks vs. Release Build

Three distinct workflows share the same Compose `rust` service:

- `scripts/dev-checks.sh` — Linux-container formatting, check, tests and clippy; produces NO artifact; its log goes to gitignored `logs/checks/`.
- `scripts/build-linux.sh` — Linux release executable at `dist/rust-youtube-streamer-service`.
- `scripts/build-windows.sh` — Windows x86-64 GNU release executable at `dist/windows/rust-youtube-streamer-service.exe`.

All three are plain POSIX `sh` scripts invoked the same way through the Compose service, none of them modifies source files, and the Windows and Linux builds produce different artifacts from the same container.

## See Also

- [`../README.md`](../README.md) — project overview, its `Release Build (Windows .exe)` section, and the Build Checks section covering the Docker environment shared with this workflow.
- [`docs/build.md`](build.md) — the Linux counterpart guide.
- [`scripts/dev-checks.sh`](../scripts/dev-checks.sh) and [`scripts/build-windows.sh`](../scripts/build-windows.sh) — the repository scripts behind these workflows.
- [`docs/configuration.md`](configuration.md) — how the built executable consumes its configuration file.
