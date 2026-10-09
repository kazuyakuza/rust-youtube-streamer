# Context

> Source of truth: [`brief.md`](brief.md). Living status log — read at the start of every task and update it before finishing, per [`instructions.md`](instructions.md).

## Current Work Focus

Phase 01 (Configuration and Application Skeleton) is complete: all 4 tasks (config module, application/CLI skeleton, structured logging, documentation and project metadata) are implemented, reviewed, and marked `[DONE]`; the phase TODO file has been renamed with the `-DONE` suffix (`.agent/todos/20261009/20261009-todo-3-DONE.md`) and branch `feat/phase01-config-app-skeleton` has been merged into `main`. Phase 01 is defined in `.agent/todos/20261009/20261009-todo-3.md` (revising the earlier `todo-2` draft; config instructions moved into `docs/configuration.md`, README gained a TOC).

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
* alpine-vm MCP output contract update landed and was verified (dual channels + combined output + exit codes; cargo stderr visible); MCP request TODO closed with `-DONE` (`.agent/todos/20261009/20261009-todo-1-DONE.md`).

## Recent Changes (2026-10-09, continued)

* Reviewed the updated `main` repository after Phase 00 and Phase 00.1 completion, including the project brief, current Cargo/Docker baseline, developer-check script, structure map, and Critical Workflow conventions.
* Created `.agent/todos/20261009/20261009-todo-2.md` defining Phase 01 — Configuration and Application Skeleton. Scope covers typed JSON config and validation, example config, `auth`/`run` CLI placeholders, structured logging, tests, and documentation. OAuth, YouTube API, renderer, chat, FFmpeg process execution, and platform-service behavior remain explicitly out of scope.

## Recent Changes (2026-10-09, Phase 01 execution)

All Phase 01 work ran on branch `feat/phase01-config-app-skeleton` via the Critical Workflow (planner + architector + implementer + code-reviewer/code-simplifier + docs-specialist, one 4.1–4.6 cycle per task). Shipped state:

* Version bump `chore: bump version to 0.2.0` (`c62e937`); `Cargo.lock` refreshed via container `cargo check` and re-verified with `--locked`.
* Task 1 — `src/config` module: strongly typed structs mirroring `config/config.example.json` (`serde` derive, `deny_unknown_fields`, `u32` numerics), explicit-path loading API `load_from_path`, 3-step read/parse/validate pipeline, aggregated typed errors `ConfigError { Io, Deserialization, Validation }` with `ConfigIssue` field-path Display contracts (no panics, no thiserror — hand-written Display per dependency policy), 18 distinct validation checks (5 positive numeric values incl. width/height/fps/font size/line height; exact `rgb24`; 1 vertical layout ≥ 1 visible line; 10 non-blank required strings incl. FFmpeg fields; 1 privacy enum ∈ private|public|unlisted; no existence checks for font/FFmpeg files). 16 unit tests via temp dirs. New deps: serde 1.0.229 (derive), serde_json 1.0.151, dev-dep tempfile pinned 3.14.0 via lockfile `--precise` (tempfile 3.27 needs edition 2024, unsupported by pinned rust:1.82).
* Task 2 — `src/app` skeleton: std-library CLI parser (no clap; dependency policy), commands `auth`/`run`, `--config <path>` before/after command (at most once), `--help` before command (stdout, exit 0), default path constant `config/config.json` (tested; `config.example.json` never implicitly loaded); usage errors → one `error:` line on stderr, exit 2; config/log-init failures → exit 1 single funnel; placeholder modes (config validated first) report not-implemented and exit 3; no YouTube/FFmpeg/network credentials touched; `#[allow(` removed everywhere in `src/` (Task 1's temporary allow retired).
* Task 3 — structured logging: `tracing 0.1.44` + `tracing-subscriber 0.3.23` (`env-filter`; MSRV-safe, no pins; >= 0.3.20 ANSI-CVE floor). Subscriber on stderr (default fmt layout), installed once (`set_global_default` is the guard), seam signature unchanged. `RUST_LOG` honored case-sensitively; default filter `info`; unset/non-unicode/empty/whitespace → default silently; INVALID non-empty value → exactly one stderr `warning:` line + default (user-approved policy superseding the plan's fatal-filter choice; `FilterInvalid` variant removed; `LogInit → exit 1` mapping test retained via `SubscriberInstall`). Exactly 3 info events in the funnel with whitelisted fields only (`command`, `config_path`, `exit_code`); no secrets/tokens/stream keys/config values logged; fatal startup failures keep the single stderr `error:` line (no duplicate cross-layer reporting). No Tokio.
* Tests: final suite counts 46 tests, all passing; full standard gate `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` exit 0 (`ALL CHECKS PASSED`) repeatedly, including on the final docs tree (log `logs/checks/20261009T225539Z.log`).
* Task 4 — README updated in place (TOC added; Current Status rewritten to Phase 01 foundation; new `Commands and Configuration` + `Logging (RUST_LOG)` sections; FFmpeg stated as separately provided external prerequisite not installed or launched by this phase; `Build Checks` section preserved) with commits `5eb824b`, `635a044` (`docs/configuration.md`, 118 lines, TOC, gitignored `config.json` copy/edit steps, `--config` option, field reference, back-link to README).
* Planner-owned metadata updated: `.agent/project-structure.md` (full module map) and this context file.
* Each task's planning documents tracked under `.kilo/plans/` (global phase plan + per-task plans/fix plans); TODO Tasks 1–3 headers marked `[DONE]` (`f98bc78`, `776a41a`, `b200aed`).

## Known Deviations and Limitations (Phase 01)

* `auth`/`run` are placeholders (exit 3 after config validation) until later phases; no OAuth, YouTube API, chat, renderer, or FFmpeg process behavior exists.
* Verification is container-only (Linux checks on the Alpine VM); no native Windows/Linux runtime validation was performed and must not be claimed.
* No subscriber side-effect (stderr delivery) unit test — documented boundary; filter resolution is tested as a pure function.
* The invalid-`RUST_LOG` fallback design (one warning + default `info`) is user-approved and differs from the original task-3 plan's fatal choice; the TODO's literal wording governs.

## Immediate Next Steps

1. None for Phase 01 — completed and merged.
2. Re-read the live repository and all relevant project-info/workflow files before drafting each later phase TODO.
3. Add the separate runtime prerequisites/permissions document and link it from README in the appropriate later documentation phase; it must state FFmpeg is a preinstalled external prerequisite, the executable runs as a normal user from a writable/readable location, and the app is not installed or registered as a Windows service.
4. Later phases replace the placeholder `auth`/`run` mode behaviors (exit 3) with real OAuth and streaming backends; `src/app/logging.rs` documents the logging seam (signature/call sites stay stable).

## Scope Decisions

* The application is a normal executable, not a Windows service. Do not implement service registration, service wrappers, elevation, or installer behavior.
* FFmpeg is an external prerequisite that the user installs independently. The application must use the configured executable path and report configuration/process errors clearly; this project does not install FFmpeg.
* Documentation and automated tests should be included in phases where appropriate, not deferred exclusively to the final phase.

## Notes

`brief.md` is the source of truth; if any project-info file conflicts with it, `brief.md` wins.
* Validation scope of Phase 00 checks: build/type/test/clippy checks inside the Linux container on the Alpine VM only; no native Windows or Linux runtime validation was performed, and no application runtime exists yet.
* `context.md` contains no secrets, credentials, tokens, or machine-specific paths.
