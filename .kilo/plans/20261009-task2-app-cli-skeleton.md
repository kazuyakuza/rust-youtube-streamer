# Implementation Plan — Task 2: Application and CLI Skeleton

- Date: 2026-10-09
- Scope: ONLY `### 2. Establish the Application and CLI Skeleton` of `.agent/todos/20261009/20261009-todo-3.md`, plus `## Dependencies and Design Constraints`. Tasks 3 and 4 are out of scope.
- Source of truth: `.agent/project-info/brief.md` (§3.5 dependency policy, §4 module responsibilities, §5.1 config load/validate/non-zero exit, §6.1 two execution modes, §6.4 credential security).
- Phase context: `.kilo/plans/20261009-phase01-configuration-app-skeleton.md` (Task 2 section + logging-seam decision).
- Environment: no Rust/Cargo on the Windows host. ALL cargo commands run via `alpine-vm` MCP (call `alpine-vm_vm_status` first) as single untethered commands.

## 1. Encoded Decisions

### 1.1 CLI parser: STANDARD LIBRARY (no clap)

Decision: hand-rolled std parser. Rationale (brief §3.5: "Avoid adding a dependency when the standard library ... provides a sufficient solution"): the whole surface is exactly 2 commands (`auth`, `run`) + 1 option (`--config <path>`) + `--help`. A ~60-line deterministic matcher covers it with zero new dependencies, zero version-drift risk, and fully testable pure functions. `clap` is justified only for large/dynamic surfaces — rejected here.

### 1.2 Accepted CLI surface (exact)

argv tokens = `std::env::args().skip(1)`.

Accepted forms (ALL of these, exactly): `auth`, `run`, `--config <path> auth`, `auth --config <path>`, `run --config <path>`, `--config <path> run`.

- `--config` MAY appear before or after the command; MUST appear at most once; MUST have a value (next token) that is not another option.
- `--help` (token anywhere where a first positional would be, i.e. before the command is consumed) prints usage text to **stdout** and exits **0**. No `-h` alias (minimal surface).

Error behavior (message to **stderr**, single concise line, prefix `error: `):
- no args / no command after `--config <path>` → `error: expected a command ('auth' or 'run')`
- unknown command (first positional is anything else) → `error: unknown command 'X' (expected 'auth' or 'run')`
- unknown `--*` option → `error: unknown option 'X'`
- `--config` with no following value → `error: missing value for '--config'`
- duplicate `--config` → `error: '--config' given more than once`
- extra positional after the command → treated as unknown-token usage error (`error: unexpected argument 'X'`)
All of the above exit **2**.

### 1.3 Default config path

`pub(crate) const DEFAULT_CONFIG_PATH: &str = "config/config.json";` in `src/app/cli.rs`. Used whenever `--config` is absent. `config/config.example.json` is NEVER referenced, defaulted, or implicitly loaded anywhere in `src/app/`. Unit test asserts the constant's exact value.

### 1.4 `src/app/` module layout (all files ≤ 200 lines, zero deps added to Cargo.toml)

```text
src/app/mod.rs       (~90)  module decls, pub fn run(args: &ArgsTokens) -> ExitCode; startup pipeline; dispatch; tests
src/app/cli.rs       (~150) Invocation parse (pure); DEFAULT_CONFIG_PATH; exit_code mapping for usage errors; tests
src/app/modes.rs     (~80)  Mode enum; handler fns auth(&AppConfig) / run(&AppConfig) -> ExitCode; NOT_IMPLEMENTED_EXIT; tests
src/app/logging.rs   (~30)  logging seam: pub fn init_logging() -> Result<(), LogInitError>
src/app/error.rs     (~50)  StartupError { Usage(String), Configuration(ConfigError) }, LogInitError; exit-code mapping; Display
```

- `main.rs` (rewritten, ~15 lines, doc comment updated, NO `#[allow]`): `fn main() -> ExitCode { app::run(&std::env::args().skip(1).collect::<Vec<_>>()) }` via `mod app; mod config;`.
- Boundary: app may call `config` + `logging` only. NO youtube/chat/renderer/streaming modules, no network, no process spawning, no credential access.

### 1.5 Startup pipeline (single funnel)

`app::run` order (each step a short named function, depth ≤ 2):
1. `cli::parse(args)` → `Result<Invocation, StartupError::Usage>`
2. On usage error: write ONE line to stderr, return exit 2. (No config load, no logging init before this.)
3. `logging::init_logging()` → on `LogInitError` (theoretically infallible this task) map to exit 1.
4. `config::load_from_path(Path::new(&invocation.config_path))` → on `ConfigError`: wrap in `StartupError::Configuration`, write ONE line `error: {err}` (ConfigError's Display is already actionable) to stderr, return exit **1**.
5. `match invocation.mode { Auth => modes::auth(&config), Run => modes::run(&config) }` → its returned `ExitCode`.

Rules: every failure reaches `main` through exactly this one typed path; the error message is written exactly ONCE (in `app::run`); handlers and cli never duplicate stderr output; no `.unwrap()`/`.expect()`/panics on any expected path (usage parse must not panic even on weird UTF-8 tokens — `OsString` non-UTF-8 args: treat a non-UTF-8 token as unknown-argument usage error, exit 2, never panic).

### 1.6 Logging seam (Task 3 fills it; NOTHING else here)

- `src/app/logging.rs`: `pub fn init_logging() -> Result<(), LogInitError>` — body is a minimal no-op (`Ok(())`) with a doc comment stating: "Task 3 replaces this body with tracing + tracing-subscriber init honoring RUST_LOG; call sites and signature stay unchanged."
- NO `tracing`/`tracing-subscriber` dependency in this task. NO Tokio.
- The "not implemented" reports go through `eprintln!` directly in `modes.rs` this task; when Task 3 lands, its plan may route them through logging at the call sites without changing the seam signature. Do not build any log-level/filter machinery here.

### 1.7 Mode handlers

`modes.rs`:
- `pub(super) const NOT_IMPLEMENTED_EXIT: u8 = 3;` — final exit codes: **1** config/startup/log-init failure, **2** CLI usage error, **3** mode not implemented. (Distinct, consistent, unit-tested.)
- `fn auth(config: &AppConfig) -> ExitCode`: stderr line `error: authentication is not implemented yet: the OAuth authorization flow has not been built; no credentials were requested or stored` → exit 3. Takes `&AppConfig` purely to enforce that validation succeeded first; never reads network/credentials.
- `fn run(config: &AppConfig) -> ExitCode`: stderr line `error: runtime pipeline is not implemented yet: the YouTube/FFmpeg pipeline has not been built; no YouTube resources were created and FFmpeg was not started` → exit 3.
- Messages must go to **stderr** (they are failure reports; stdout stays silent/reserved).
- Tests: each handler called with a sample validated `AppConfig` (built via `src/config/test_fixtures.rs` in-crate) returns exit 3. Channel content itself is not asserted (process-free).

### 1.8 Required end state for lint suppression

The `#[allow(dead_code, unused_imports)] mod config;` in `src/main.rs` must be REMOVED. After Task 2 the app layer consumes config (`AppConfig`, `load_from_path`, `ConfigError`), so no suppressions remain anywhere in `src/`. Verified by `cargo clippy --locked -- -D warnings` exit 0 and a grep showing no `#[allow(` in `src/`.

## 2. Implementation Steps (code-writing order + per-step verification)

All commands run via alpine-vm MCP, one command per call, after `alpine-vm_vm_status`. Single-command pattern: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` (variants per step). Final check: `... run --rm rust sh scripts/dev-checks.sh` must exit 0.

1. `src/app/error.rs` — implement `StartupError`, `LogInitError`, Display, exit-code mapping (`usage/messages → 2`, `Configuration → 1`). Verify: `cargo check --locked`.
2. `src/app/cli.rs` — `DEFAULT_CONFIG_PATH`, `Mode { Auth, Run }`, `Invocation { mode, config_path: String }`, `parse(args: &[String]) -> Result<Invocation, StartupError>` implementing §1.2 exactly. Verify: `cargo check --locked`.
3. `src/app/logging.rs` — seam per §1.6. Verify: `cargo check --locked`.
4. `src/app/modes.rs` — handlers per §1.7. Verify: `cargo check --locked`.
5. `src/app/mod.rs` — `run(...)` pipeline per §1.5 + `pub(crate) use` only what main needs. Verify: `cargo check --locked`.
6. Rewrite `src/main.rs` — delegate to `app::run`; DELETE the `#[allow]`; update doc comment. Verify: `cargo check --locked` AND `grep -r "#\[allow(" src/` returns nothing (use container `sh -c` or reason from clippy; implementer may run `docker compose ... run --rm rust sh -c "grep -rn '#\[allow(' src || echo none"`).
7. Add unit tests (§3). Verify: `cargo test --locked`.
8. Full suite: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, all four PASS lines (fmt-check, check, test, clippy).
9. Manual smoke (optional, container): `... run --rm rust sh -c "./target/debug/rust-youtube-streamer-service auth || echo exit=$?"` — if binary exists after check; NOT required for acceptance (tests cover the logic).
10. Git: on `feat/phase01-config-app-skeleton`; commit exactly: `feat: add application skeleton with auth and run commands`. Files: `src/main.rs`, `src/app/mod.rs`, `src/app/cli.rs`, `src/app/modes.rs`, `src/app/logging.rs`, `src/app/error.rs`. Per gitignore-compliance: run `git status`, stage ONLY those 6 paths, `Cargo.lock` should already be current (check it stayed unchanged since no deps were added; if it changed, stop and report).

## 3. Unit tests (in-module `#[cfg(test)]`, no processes, no secrets, no env mutation beyond tempfile)

cli.rs: (a) `auth` and `run` alone; (b) option before/after command for both modes; (c) resolved path equals default when absent; (d) `--config x auth` yields `x`; (e) `--help` accepted; (f) unknown command/option, missing value, duplicate --config, extra positional, empty args → each parses to `StartupError::Usage` (not panic); (g) `DEFAULT_CONFIG_PATH == "config/config.json"`.
mod.rs: exit-code mapping — usage → 2, Configuration → 1 (construct `ConfigError::Validation` fixture → exit 1); logging-init failure → 1.
modes.rs: auth/config-validated handler → NOT_IMPLEMENTED_EXIT; run handler → NOT_IMPLEMENTED_EXIT; constants mapping stable.
Tests construct args slices directly; never read real `config/config.json`; reuse `tempfile` (already dev-dep) only if a path-on-disk case is needed (not needed for parse tests).

## 4. Constraints echo (hard, verifiable)

- No YouTube/auth/network/FFmpeg/credential behavior of any kind; handlers never contact anything.
- No service/installer/elevation; no Tokio; no tracing crates; no empty placeholder feature modules.
- CLI surface stays exactly §1.2; no extra flags/aliases.
- Rules compliance: file ≤ 200 lines, fn ≤ 50, ≤ 2 params, depth ≤ 2, single-section boolean conditions (extract named predicates, e.g. `fn is_help_token(..)`, `fn is_config_flag(..)`), no commented-out code, self-documenting names, private members default (only `pub(crate)`/`pub(super)` as needed).
- `config.example.json` never loaded implicitly; `config/config.json` not created.

## 5. Verification checklist vs TODO Task 2

- [x] plan encoded → implementer confirms: `auth`/`run` commands, `--config` both positions, default path tested, non-zero exits (2 usage / 1 startup-config / 3 not-implemented), no panics, no duplicate error logging, dispatch testable process-free, lint suppression removed, dev-checks exit 0, single commit with exact message.
