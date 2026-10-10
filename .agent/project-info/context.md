# Context

> Source of truth: [`brief.md`](brief.md). Living status log — read at the start of every task and update it before finishing, per [`instructions.md`](instructions.md).

## Current Work Focus

Phase 01 (Configuration and Application Skeleton) is complete: all 4 tasks (config module, application/CLI skeleton, structured logging, documentation and project metadata) are implemented, reviewed, and marked `[DONE]`; the phase TODO file is `.agent/todos/20261009/20261009-todo-3-DONE.md`, merged into `main`. Phase 01.1 (Reproducible Linux Build Artifact) is complete: `scripts/build-linux.sh`, artifact ignore semantics, build-workflow documentation, and the tracked completion report are implemented and verified; see `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md`. Phase 01.2 (Reproducible Windows Executable Build) is complete: Docker-based `x86_64-pc-windows-gnu` cross-compilation in the pinned toolchain image, `scripts/build-windows.sh` producing `dist/windows/rust-youtube-streamer-service.exe`, three-workflow documentation, final-tree verification of both build paths plus the dev-checks gate, and the tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md`; the phase TODO is `.agent/todos/20261010/20261010-todo-1-DONE.md`, and all 10 acceptance criteria are verified `[x]`. No Phase 02 work has started; the next task is Phase 02 — Renderer and Chat Store.

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
* Created `.agent/todos/20261009/20261009-todo-2.md` as the initial Phase 01 draft; the final Phase 01 TODO was `.agent/todos/20261009/20261009-todo-3-DONE.md`. Scope covers typed JSON config and validation, example config, `auth`/`run` CLI placeholders, structured logging, tests, and documentation. OAuth, YouTube API, renderer, chat, FFmpeg process execution, and platform-service behavior remain explicitly out of scope.
* Reviewed Phase 01's final repository state and completion evidence, including `Cargo.toml`, the Compose toolchain, `scripts/dev-checks.sh`, configuration/CLI implementations, documentation, and updated project structure.
* Created `.agent/todos/20261009/20261009-todo-4.md` defining Phase 01.1 — Reproducible Linux Build Artifact. The completed TODO and tracked report document `scripts/build-linux.sh`, `docs/build.md`, README and metadata updates, Docker/MCP verification, and the Linux artifact under ignored `dist/`; Cargo intermediates remain in the named volume.
* After reviewing the completed Phase 01.1 report and live build setup, created `.agent/todos/20261010/20261010-todo-1.md` for Phase 01.2 — Reproducible Windows Executable Build. It selects `x86_64-pc-windows-gnu` cross-compilation inside the existing Docker toolchain, preserves the Linux build and dev checks, writes the `.exe` under `dist/windows/`, requires runtime DLL handling to be verified, and requires a tracked completion report. No implementation or Windows build has been run in this planning step.

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

## Recent Changes (2026-10-09, Phase 01.1)

* Branch `feat/linux-release-build-artifact`; version bumped to `0.3.0` (`f9a6b0e`).
* Task 1 — `scripts/build-linux.sh` (commit `92880c7`, review fix `6118322`): POSIX-sh release builder; asserts the documented Compose working directory and a non-empty `CARGO_TARGET_DIR`; runs `cargo build --release --locked`; verifies the built binary exists non-empty; creates `dist/` if needed; copies only `dist/rust-youtube-streamer-service` and re-verifies non-empty; prints start/result messages with the exact output path and size; never suppresses Cargo diagnostics; never touches files elsewhere under `dist/`.
* Standard invocation `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` verified exit 0; observed success messages:
  * `Building Linux release executable using the tracked lockfile: cargo build --release --locked`
  * `Release build succeeded.`
  * `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (1715296 bytes).`
  * `The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe.`
* A cold full compile showed Cargo `Finished` in 33.04 s; no `target/` directory appears in the project root; Cargo intermediates remain in the named volume `rust-streamer-target`.
* Artifact observed as mode `-rwxrwx---` `root:vboxsf` on the shared mount; `dist/` is ignored via the existing generic `dist/` `.gitignore` rule; nothing was unignored; the artifact is never staged or committed.
* Failure paths verified: wrong working directory → exit 1 with `build-linux: error: expected the current working directory to be /rust-youtube-streamer (the documented Compose working directory); got: /tmp`; empty `CARGO_TARGET_DIR` → exit 1 with `build-linux: error: CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)`.
* Task 2 — `.gitignore` reviewed: the existing `dist/` rule already covers the artifact; nothing was unignored; no build artifacts were staged or committed; any Windows/macOS cross-build or multi-platform matrix remains out of scope per the TODO constraints.
* Task 3 (this change) — `docs/build.md` (purpose, prerequisites, standard command, expected output, overwrite/rebuild behavior, failure semantics table, what-the-build-does-not-prove, dev-checks comparison); README gained the `Release Build (Linux Artifact)` section + TOC entry linking `docs/build.md`; project-structure.md gained build-linux.sh/build.md/dist entries and the v0.3.0 refresh.
* Task 3 verification is a docs-only self-check (TOC anchors, relative links); the final-tree `dev-checks.sh` run and the completion report belong to Task 4.
* Task 4 — standard gate re-ran on the final tree via the Alpine VM MCP (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`): exit 0 with four PASS lines (`fmt-check`, `check`, `test`, `clippy`) and summary `ALL CHECKS PASSED`; log `logs/checks/20261010T034008Z.log`.
* Task 4 — tracking re-checks (fresh): `git check-ignore -v dist/rust-youtube-streamer-service` → `.gitignore:34:dist/`; `git ls-files dist/` → empty; `git status --porcelain --ignored=matching -- dist/` → `!! dist/` (single ignored entry); artifact re-observed at 1715296 bytes on `ls -l /rust-youtube-streamer/dist`.
* Tracked completion report created: `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` (commit `634d395`; 174 lines; files changed, exact commands/exit statuses with the dev-checks gate, artifact path/size/mode/owner, `dist/` ignore confirmation, known limitations with the container-only caveat, evidence index with commit list and linked per-task planning/adherence files).
* TODO Tasks 3–4 marked `[DONE]` (`080b534`, `4cea176`); all 11 acceptance-criteria checkboxes flipped to `[x]` (`4cea176`, mirroring the Phase-01 precedent).

## Recent Changes (2026-10-10, Phase 01.2 implementation)

* Branch `feat/windows-gnu-cross-build`; version bumped `0.3.0 → 0.4.0` (commit `87159a9`).
* Task 1 (commit `e5505e1`): the Dockerfile now installs `gcc-mingw-w64-x86-64` (Windows GNU cross toolchain: fallback linker route + `x86_64-w64-mingw32-objdump` audit tooling) and adds the Rust target `x86_64-pc-windows-gnu` on the pinned `rust:1.82` Debian base; image rebuilt; probe-verified target/linker/objdump presence inside the container.
* Environment note + incident: repo-host git `core.autocrlf=true`; a stray working-tree-wide CRLF conversion earlier today broke `sh scripts/build-linux.sh` execution until the files were re-normalized; new tracked `.gitattributes` forces LF for `*.sh` and `Dockerfile` to prevent recurrence. Root `target/` never appears (named-volume redirect holds).
* Linux artifact fresh observation on the v0.4.0 tree: 1715240 bytes (sizes change with code; no doc hardcoding).
* Task 2 (commit `e775a7b`): `scripts/build-windows.sh` added — mirrors `build-linux.sh` conventions (`build-windows: error:` prefix, exit 1, no diagnostics suppression, never deletes/prunes); first standard-command run exit 0; artifact `dist/windows/rust-youtube-streamer-service.exe` produced non-empty; DLL decision basis: static GNU runtime link — objdump import audit found no MinGW runtime DLLs, only standard Windows system DLLs (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll); probe-tested failure paths: wrong cwd → exit 1 cwd message; empty `CARGO_TARGET_DIR` → exit 1 compose-reference message; cargo build failure → diagnostics shown + exit 1.
* Task 3 (this change): `docs/build-windows.md` created; README gained the `Release Build (Windows .exe)` section + TOC entry; `docs/build.md` See Also links the Windows guide; `project-structure.md` and this context file updated. Cross-compilation ≠ native Windows runtime validation is stated wherever claims are made; native Windows validation remains NOT RUN.
* Task 4 (commits `185dadc`, `3c992a3`): fresh final-tree verifications via the Alpine VM MCP all green — Windows build `sh scripts/build-windows.sh` exit 0 (`dist/windows/rust-youtube-streamer-service.exe`, 3191192 bytes); Linux build `sh scripts/build-linux.sh` exit 0 (1715240 bytes); dev-checks gate exit 0 (`PASS fmt-check/check/test/clippy`, `ALL CHECKS PASSED`, 46 tests, log `logs/checks/20261010T183206Z.log`); `dist/` remains ignored (`git check-ignore -v` → `.gitignore:34:dist/`; `git ls-files dist/` empty; `!! dist/`); tracked completion report `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` (311 lines) with observed-only evidence and NOT-RUN limitations (native Windows execution not validated; FFmpeg external). TODO Task 4 marked `[DONE]`; all 10 Phase 01.2 acceptance-criteria checkboxes flipped `[x]`.

## Known Deviations and Limitations (Phase 01)

* `auth`/`run` are placeholders (exit 3 after config validation) until later phases; no OAuth, YouTube API, chat, renderer, or FFmpeg process behavior exists.
* Verification is container-only (Linux checks on the Alpine VM); no native Windows/Linux runtime validation was performed and must not be claimed.
* No subscriber side-effect (stderr delivery) unit test — documented boundary; filter resolution is tested as a pure function.
* The invalid-`RUST_LOG` fallback design (one warning + default `info`) is user-approved and differs from the original task-3 plan's fatal choice; the TODO's literal wording governs.

## Immediate Next Steps

1. After Phase 01.2 is complete, re-review the live repository and proceed to Phase 02 — Renderer and Chat Store.
2. Add the separate runtime prerequisites/permissions document and link it from README in the appropriate later documentation phase; it must state FFmpeg is a preinstalled external prerequisite, the executable runs as a normal user from a writable/readable location, and the app is not installed or registered as a Windows service.
3. Later phases replace the placeholder `auth`/`run` mode behaviors (exit 3) with real OAuth and streaming backends; `src/app/logging.rs` documents the logging seam (signature/call sites stay stable).
4. Native Windows runtime validation of `dist/windows/rust-youtube-streamer-service.exe` remains NOT RUN; when a Windows-host validation session happens, record its evidence and retire the NOT-RUN limitation in the docs and completion report.

## Scope Decisions

* The application is a normal executable, not a Windows service. Do not implement service registration, service wrappers, elevation, or installer behavior.
* FFmpeg is an external prerequisite that the user installs independently. The application must use the configured executable path and report configuration/process errors clearly; this project does not install FFmpeg.
* Documentation and automated tests should be included in phases where appropriate, not deferred exclusively to the final phase.

## Notes

`brief.md` is the source of truth; if any project-info file conflicts with it, `brief.md` wins.
* Validation scope of Phase 00 checks: build/type/test/clippy checks inside the Linux container on the Alpine VM only; no native Windows or Linux runtime validation was performed, and no application runtime exists yet.
* `context.md` contains no secrets, credentials, tokens, or machine-specific paths.
