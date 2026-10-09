# Context

> Source of truth: [`brief.md`](brief.md). Living status log — read at the start of every task and update it before finishing, per [`instructions.md`](instructions.md).

## Current Work Focus

Phase 00 (repository foundation and build baseline) is complete: the minimal Cargo package,
Docker-based build checks via the Alpine VM, and repository hygiene are in place. The next phase
will begin the first real application module.

## Recent Changes (2026-10-08)

* `brief.md` defined as the complete technical brief and source of truth for the Rust YouTube Streamer Service.
* Project-info template customized to this project (`instructions.md` core-file structure in place).
* `README.md` replaced with initial project info for the Rust YouTube Streamer Service (detailed README deferred to future sessions).
* Initial structure scaffolded: `config/`, `credentials/`, `fonts/`, `logs/` and `src/{app,config,youtube,chat,renderer,streaming}` folders with `.gitkeep` placeholders; `.gitignore` logs rule adjusted; `project-structure.md` updated.
* Verified current repository contents and agent workflow conventions before creating the next TODO. Existing completed TODO is `.agent/todos/20261008/20261008-todo-1-DONE.md`; the new Phase 00 task is numbered 2.
* Created `.agent/todos/20261008/20261008-todo-2.md` to define Phase 00 — Repository Foundation and Build Baseline.
* Task 2 of Phase 00: added `Cargo.toml` (package `rust-youtube-streamer-service` v0.1.0, edition 2021, no dependencies), `src/main.rs` minimal entry point, and generated `Cargo.lock`; `Cargo.lock` is tracked.
* Task 3 of Phase 00: added `Dockerfile` (pinned official `rust:1.82`, plus rustfmt/clippy components) and `docker-compose.yml` (`rust` service; project bind-mounted at `/rust-youtube-streamer`; `CARGO_TARGET_DIR` redirected to named volume `rust-streamer-target`).
* Build checks executed with Docker Compose v2.31.0 on the Alpine VM via the `alpine-vm` MCP, commands pattern `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo <subcmd>`: `fmt --check` exit 0, `check --locked` exit 0, `test --locked` exit 0 (0 tests), `clippy --locked -- -D warnings` exit 0. MCP/VM captured output may omit cargo stderr `Finished` lines; exit status is authoritative.
* Ownership on shared mount entries: `root:vboxsf`.
* Task 4 of Phase 00: `.gitignore` extended — `/target/`, `/config/config.json`, `credentials/*` with `!credentials/.gitkeep`, and a `!Cargo.lock` guard. Build artifacts never written to project root (named-volume redirect; `Cargo.lock` is the sole root exception, tracked).
* Task 5 of Phase 00: README 'Current Status' rewritten to the build baseline; new 'Build Checks (Docker via Alpine VM)' section added; `project-structure.md` updated with root files and placeholder note; this context file updated.
* README documents that the implementation-agent host needs no Rust/Cargo install, and distinguishes container-only build checks from native Windows/Linux runtime validation.
* Phase 00 closed: all 15 acceptance criteria checked; the TODO file was renamed with the `-DONE` suffix (`.agent/todos/20261008/20261008-todo-2-DONE.md`).
* Feature branch merged (fast-forward) into `main` and pushed to `origin/main`; final commit `86e9c51`.
* `src/main.rs` gained a 2-line crate-level `//!` overview comment (comment-only); `cargo fmt --check` re-validated with exit 0 on the final tree.

## Recent Changes (2026-10-09)

* Phase 00.1 (reproducible dev checks, branch `feat/dev-checks-script`): added `scripts/dev-checks.sh` (commit `12aaf37`) — POSIX-sh runner that executes `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` and `cargo clippy --locked -- -D warnings` inside the Compose `rust` service; per-check `PASS|FAIL <name> exit=<code> (<duration> s)` lines with merged output, `--- Summary ---` block printing `ALL CHECKS PASSED`/`N CHECK(S) FAILED`, UTC-timestamped log under gitignored `logs/checks/`, aggregate exit code (0 when all pass, 1 when any fails), no source mutation, no fmt auto-apply; includes a `FORCE_FAIL=<check>` hook (synthetic exit 7) for failure-path validation.
* First full green run via the standard invocation `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`: exit 0 with four PASS lines (`fmt-check`, `check`, `test`, `clippy`), summary `ALL CHECKS PASSED`, log `logs/checks/20261009T152555Z.log`.
* Failure-path validation with `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm -e FORCE_FAIL=test rust sh scripts/dev-checks.sh`: exit 1 with `FAIL test exit=7` plus three PASS lines, summary `1 CHECK(S) FAILED`, log `logs/checks/20261009T152623Z.log`; no source files touched in either run.
* README 'Build Checks (Docker via Alpine VM)' updated: script invocation documented as the standard way; manual per-check commands kept as clearly-labeled fallback/reference; commit `docs: document dev-checks script usage`.
* `project-structure.md` updated with the `scripts/` folder entry and the check-log note for gitignored `logs/checks/`; this context file updated; commit `docs: record dev-checks script in structure and context`.

## Immediate Next Steps

1. Plan next phase TODO (first application module) in a new chat session referencing updated project info.
2. In later phases, re-read the current repository and project info before drafting each new TODO, as requested.
3. Add the separate runtime prerequisites/permissions document and link it from README in the appropriate later documentation phase; it must state FFmpeg is a preinstalled external prerequisite, the executable runs as a normal user from a writable/readable location, and the app is not installed or registered as a Windows service.

## Scope Decisions

* The application is a normal executable, not a Windows service. Do not implement service registration, service wrappers, elevation, or installer behavior.
* FFmpeg is an external prerequisite that the user installs independently. The application must use the configured executable path and report configuration/process errors clearly; this project does not install FFmpeg.
* Documentation and automated tests should be included in phases where appropriate, not deferred exclusively to the final phase.

## Notes

`brief.md` is the source of truth; if any project-info file conflicts with it, `brief.md` wins.
* Validation scope of Phase 00 checks: build/type/test/clippy checks inside the Linux container on the Alpine VM only; no native Windows or Linux runtime validation was performed, and no application runtime exists yet.
* `context.md` contains no secrets, credentials, tokens, or machine-specific paths.
