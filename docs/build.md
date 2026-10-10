# Build the Linux Release Executable

This guide explains how to build the distributable release executable `dist/rust-youtube-streamer-service` inside the pinned Linux Rust container.

## Table of Contents

- [Purpose](#purpose)
- [Prerequisites](#prerequisites)
- [Build Command](#build-command)
- [Expected Output](#expected-output)
- [Overwrite and Rebuild Behavior](#overwrite-and-rebuild-behavior)
- [Failure Semantics](#failure-semantics)
- [What This Proves and Does Not Prove](#what-this-proves-and-does-not-prove)
- [Dev Checks vs. Release Build](#dev-checks-vs-release-build)
- [See Also](#see-also)

## Purpose

- The repository ships two separate Docker-based workflows: fast validation checks and a release build. This guide covers only the release build.
- The build compiles the service in release mode (`cargo build --release --locked`) inside the pinned Linux Rust container and leaves exactly one final file behind.
- Success criterion from the user's perspective: the command exits `0` and prints, among its messages, `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (… bytes).`

## Prerequisites

- Same Docker prerequisites as the regular checks, documented in [Build Checks (Docker via Alpine VM)](../README.md#build-checks-docker-via-alpine-vm): Alpine VM running with the project shared at `/rust-youtube-streamer`, Docker with the Compose plugin (Compose v2.31.0 observed), and the container image built once with `docker compose -f /rust-youtube-streamer/docker-compose.yml build rust`.
- No Rust or Cargo toolchain is needed on the computer that runs the build command; everything happens inside the container.
- The build always compiles against the version-locked dependency set recorded in the tracked `Cargo.lock`; users never edit or regenerate it for this workflow.

## Build Command

The standard invocation (the single command users run):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

- It starts the Compose `rust` service from the repository working directory and runs the repository's tracked script `scripts/build-linux.sh` (POSIX `sh`, invoked the same way as `scripts/dev-checks.sh`). A first, cold full compile finishes on the order of half a minute; subsequent runs reuse cached intermediate output, so rebuilds are fast.
- Cargo's own progress/error output is shown as it happens: the script deliberately does not suppress Cargo diagnostics.

## Expected Output

- Output path on success: `dist/rust-youtube-streamer-service` in the repository root — a Linux executable produced inside the Linux container; it is not a Windows `.exe`. Whether the streaming pipeline actually works is a separate matter — see [What This Proves and Does Not Prove](#what-this-proves-and-does-not-prove).
- The script prints the final path and size; the exact byte count naturally changes as the codebase changes, so this guide never hardcodes it.
- `dist/` is gitignored: the executable is a build output, never committed.
- Cargo's intermediate build files never appear in the repository root; they live in the Docker named volume `rust-streamer-target`. The root stays clean except the tracked `Cargo.lock`.

## Overwrite and Rebuild Behavior

- Rerunning the standard command rebuilds from the current source and overwrites the previous `dist/rust-youtube-streamer-service` with a fresh copy.
- Intermediate build caches persist in the Docker volume, so routine rebuilds are incremental and fast; dependency resolution is never re-run — `--locked` fails fast if `Cargo.lock` does not match the manifest (see [Failure Semantics](#failure-semantics)).
- The artifact is disposable: deleting it has no repository effect; run the standard command again to regenerate it at any time. Nothing else in the workflow removes or prunes anything: Docker volumes are never pruned, and the script never deletes unrelated files under `dist/`.
- On a failed run nothing is placed in `dist/`; if a previous successful artifact exists, it stays in place untouched (the script only writes the artifact after a fully successful build-and-copy sequence).

## Failure Semantics

Every failure line begins with the literal prefix `build-linux: error:` and the command exits `1`; one or more lines appear on standard error, and Cargo's diagnostics, if any, are printed in full, never suppressed.

| # | If this happens | What you see |
|---|---|---|
| 1 | The command is not run through the documented Compose invocation (e.g. the script runs from some other working directory). | error: expected the current working directory to be /rust-youtube-streamer ... got: <directory>; exit 1 before any build starts. |
| 2 | The Compose environment variable that names the release-output folder (`CARGO_TARGET_DIR`) is missing or empty — in practice: the build was not started by the Compose service above. | error: CARGO_TARGET_DIR is unset or empty ... docker-compose.yml sets it to /rust-streamer-target; exit 1. |
| 3 | The Cargo release build itself fails (e.g. compile error, or `Cargo.lock` out of date with the manifest). | Cargo's full error output followed by error: cargo build --release --locked failed with exit code <code> ... diagnostics above are not suppressed; exit 1. |
| 4 | After a successful build the expected release executable is not found or is empty. | error: expected release binary not found or empty: <path>; exit 1. |
| 5 | The `dist/` directory cannot be created. | error: could not create the dist/ directory: <path>; exit 1. |
| 6 | Copying the build output to `dist/` fails. | error: failed to copy <source> to <destination>; exit 1. |
| 7 | After the copy, the final file is missing or zero bytes. | error: final artifact is missing or empty after the copy: <path>; exit 1. |

A non-zero exit means no new artifact was produced; fix the reported condition and run the standard command again.

## What This Proves and Does Not Prove

- What it proves: the current source compiles in release mode with the pinned Linux Rust toolchain against the tracked lockfile, inside the Linux container, and produces a non-empty Linux executable file.
- What it does NOT prove:
  - It is NOT native Windows or native Linux runtime validation of the produced executable's actual behavior (streaming, config load on a real host, logging on a real host).
  - It does NOT create a Windows `.exe` and involves no cross-compilation; the artifact runs on Linux only.
  - It does NOT mean the application's features are implemented: OAuth `auth` and `run` streaming are still placeholders that report not-implemented and exit code 3 after configuration validation.

## Dev Checks vs. Release Build

- The check script (`scripts/dev-checks.sh`, documented in the README's [Build Checks (Docker via Alpine VM)](../README.md#build-checks-docker-via-alpine-vm) section) runs formatting checks, type/build checks, unit tests and lint checks inside the same container, and produces NO artifact; its log goes to gitignored `logs/checks/`.
- The build script documented here produces the distributable release executable under `dist/`.
- The dev checks validate the code; the release build produces the distributable executable.
- Both scripts are plain POSIX `sh` invoked the same way through the Compose service, and neither modifies source files.

## See Also

- [`README`](../README.md) — project overview, and the Build Checks section covering the Docker environment shared with this workflow.
- [`scripts/dev-checks.sh`](../scripts/dev-checks.sh) — the fast validation workflow.
- [`docs/configuration.md`](configuration.md) — how the built executable consumes its configuration file.
