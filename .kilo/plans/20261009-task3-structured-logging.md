# Task 3 Plan — Structured Logging and Error Boundaries

- Date: 2026-10-09
- Role: architector (4.1b analysis & planning) — plan only, no code written by this agent
- Implementer target: JUNIOR developer under 50% restriction (all decisions pre-encoded; no judgment calls)
- TODO source: `.agent/todos/20261009/20261009-todo-3.md` → `### 3. Establish Structured Logging and Error Boundaries` + `## Dependencies and Design Constraints`
- Global plan source: `.kilo/plans/20261009-phase01-configuration-app-skeleton.md` (Task 3 section)
- Branch: `feat/phase01-config-app-skeleton` (Tasks 1–2 already landed; version already `0.2.0`)

> [Project Info: Active] — brief.md §3.5, §11.1, §12.1 read; context.md auto-injected.

---

## 0. Scope Card (strict)

### In scope (exactly)
1. `Cargo.toml`: add `tracing` + `tracing-subscriber` (+ lockfile refresh through the container).
2. `src/app/logging.rs`: replace the no-op seam body with real initialization; keep `pub(super) fn init_logging() -> Result<(), LogInitError>` signature and ALL call sites unchanged (`mod.rs::initialize_logging` is the only caller — verify it stays so).
3. `src/app/error.rs`: inhabit `LogInitError` (replace the uninhabited enum) + unit tests for the `StartupError::LogInit → exit 1` mapping.
4. `src/app/mod.rs`: add the three (3) `tracing::info!` events exactly as specified in §5. Nothing else changes.
5. Unit tests in `logging.rs` and `error.rs`.

### Explicitly OUT of scope (violations = failure)
- NO changes to `src/config/**` (no `tracing` calls there; config errors propagate only).
- NO changes to `src/app/cli.rs`, `src/app/cli_tests.rs`, `src/main.rs`, `src/app/modes.rs`.
- NO Tokio, NO task supervision, NO signal handling, NO YouTube lifecycle states.
- NO README, `docs/`, `.agent/**` changes (that is Task 4; note found gaps in the completion report only).
- NO log-file opening (application logs go to stderr; the chat `log_file` belongs to a later phase).
- NO Dockerfile/docker-compose/scripts changes (Acceptance Criterion "No CI... changes" from Dependencies and Design Constraints).
- NO `[DONE]` marking on the TODO file (that is sub-step 4.6, not this task's implementer step).

---

## 1. Dependency Decisions (encoded)

### 1.1 Choice and justification
- **Adopt `tracing` + `tracing-subscriber` with `env-filter`.**
  - vs `log`+`env_logger`: Cargo.lock currently has only serde/serde_json and the phase introduces no `log`-based transitive libraries; `tracing` gives structured fields with coarser alternative cost nothing (same env-var contract anyway, and `tracing`'s `output-log` compat shim would add the log crate back).
  - vs `slog`/built-in `println!`: brief §3.5 requires a maintained stack; TODO §3 names `tracing`+`tracing-subscriber` explicitly as the candidate. Higher maintenance, wider adoption, MIT/Apache-2.0 dual license.
  - `tracing` alone is **NOT** sufficient: RUST_LOG parsing/filtering lives in `tracing-subscriber`'s `EnvFilter`. This is why both crates are needed; the plan picks the minimal feature set instead of dropping a crate.

### 1.2 Exact `Cargo.toml` addition
Append to the `[dependencies]` section (do NOT touch existing serde/serde_json lines; do NOT add any other crate):

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 1.3 Feature sets (minimal set decided)
- `tracing = "0.1"`: **keep the crate's default features** (they are `std` + `attributes`, i.e. the macros we use). Do not set `--no-default-features`; the macro events are unusable without `std`. Do NOT use `release_max_level_*` features (there is no requirement to compile out log levels in release builds now).
- `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`: **keep default features** (`fmt`, `ansi`, `std`, `smallvec`, `tracing-log` stay on) and add only `env-filter`. Justification: `fmt` is on by default and is what supplies the writer/format used in §2; NOT using `default-fmt = false` + manual layers, because the task requires "no custom layers, minimal". Do NOT disable `tracing-log` — it costs nothing now and keeps future `log`-emitting deps visible. Do NOT add `json` (not requested; plain text is enough for this phase).
- `env-filter` is REQUIRED of the crate (not optional for us) because RUST_LOG support in TODO §3 maps directly to `EnvFilter`.
- Security note encoded for review: the `ansi` default feature includes the ANSI-escape-injection fix; the crate version used must be **≥ 0.3.20** (released 2025-08-29; earlier versions are vulnerable). If pinning is needed, never pin below 0.3.20.

### 1.4 Version compatibility + lockfile procedure (container rust 1.82)
Facts (verified 2026-10): `tracing` 0.1.44 (latest; 0.1.42 is yanked) and `tracing-subscriber` 0.3.23 (latest; 0.3.21 yanked) both declare MSRV 1.65 — well under the container's 1.82. Fresh resolution is therefore expected to succeed with no pins. The fallback procedure MUST still be followed exactly if resolution/build fails:

1. Edit `Cargo.toml` per §1.2.
2. Check `alpine-vm` status first (`alpine-vm_vm_status`), then run (single command, canonical invocation):
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check
   ```
   (deliberately NOT `--locked` on this first run: it resolves new graph entries into `Cargo.lock`). Exit code is authoritative; captured `Finished` lines may be absent — never conclude from stderr text alone.
3. If step 2 exits 0: verify the tracked lockfile contract:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked
   ```
   Exit 0 = lockfile is consistent; proceed to §8.
4. If step 2 fails with version-resolution/MSRV/edition errors (symptoms: `failed to parse manifest`, `requires rustc 1.8x` where x > 2, `error: cannot install a version of <dep>`), pin WITHOUT touching `Cargo.toml` (lockfile-only, exactly the tempfile 3.14.0 precedent), in this order and ONLY as far as needed:
   ```
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update -p tracing-subscriber --precise 0.3.20
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update -p tracing --precise 0.1.41
   ```
   Then, if a transitive dep is the blocker, pin the specific blocking crate the compiler names (courier the crate name + version from the error; known-safe reference points: `regex 1.10.6`, `nu-ansi-term 0.46`, `sharded-slab 0.1.7`, `thread_local 1.1.8`). Re-run step 3 after every pin.
5. Record every pin performed (crate, `--precise` version, triggering error) in the completion report item 4. `Cargo.lock` commits with the task commit; it is tracked and is the sole root-level generated artifact.

---

## 2. Seam Implementation — `src/app/logging.rs` (encoded)

### 2.1 Decisions
- **Subscriber output stream: stderr (PICKED).** `.with_writer(std::io::stderr)` is mandatory in `init_logging`: `tracing_subscriber::fmt` defaults to stdout, and all diagnosis/failure output of this program is conventionally on stderr (consistent with `report_failure`, `print_help` on stdout, placeholder handler lines). Justification for reviewer: keeps stdout clean for future stream data; logs interleaving with the one-line error funnel is acceptable (init succeeds before any other output exists).
- **Format: the `fmt()` DEFAULT "full" layout with timestamps — picked over `.compact()`.** Zero extra builder calls (minimal, per TODO "no custom layers"); it already includes a timestamp and level/target/fields; differences between full/compact are cosmetic and this choice avoids encoding an arbitrary cosmetics decision. No custom formatter, no JSON layer, no custom writer wrapper.
- **Default filter: `info`.** Constant `pub(super) const DEFAULT_FILTER: &str = "info";` (crate-private; tests in the same file see it).
- **Env source:** read once inside `init_logging` via `std::env::var("RUST_LOG").ok()` — module-private constant `const RUST_LOG: &str = "RUST_LOG";`. `NonUnicode` errors collapse to `None` (treated as unset: an unreadable-encoding operator value carries no directive information; nothing else about it is recoverable).
- **Invalid value behavior: FATAL via `LogInitError` (exit 1). PICKED over silent fallback.** Justification (encoded for the completion report and reviewer):
  - An INVALID directive is an operator configuration error, same class as "reject unsupported pixel-format values rather than silently substituting" (TODO Task 1) and fatal "Invalid configuration" (brief §12.1). Silent substitution would hide operator typos like `RUST_LOG=app=banana`; only UNSET (or empty/whitespace) routes to the documented default.
  - Consistent with `LogInitError`'s declared purpose: it is the fatal startup-failure class for logging; with the default rejection-fatal behavior the variant is genuinely reachable from the runtime path, which is exactly what makes the exit-1 mapping testable now (item 3).
  - Validation is fully delegated to `EnvFilter::try_new` — NO hand-written directive grammar, no parsing/level tables. Note (for docs later, not code): `EnvFilter` treats a bare token without `=` as a valid target name (level TRACE), so most "garbage" strings are actually valid directives; only genuinely malformed forms (e.g. `app=banana`, `=`, `app=`) reach the error path. This is accepted as correct EnvFilter semantics.
  - Recoverable vs fatal classification (state explicitly in code docs): unset/empty RUST_LOG → recoverable (use default); non-UTF-8 value → recoverable (treated as unset); malformed directive → fatal (`FilterInvalid`); duplicate subscriber install → fatal (`SubscriberInstall`) though unreachable via the single call site.
  - Value trimming: trim the raw value once (leading/trailing whitespace) before the emptiness check and before validation, so a trailing newline in an env value never becomes a spurious startup failure. The trimmed string is what is compiled into the filter.

### 2.2 File shape (target ≤ ~150 lines incl. tests; ≤ 200 hard limit)
Keep the existing module-level `//!` doc intent, updated to describe the real behavior (per §2.5). Public-in-module surface stays exactly `pub(super) fn init_logging() -> Result<(), LogInitError>` — this is THE invariant: `mod.rs::initialize_logging` is not edited at all per §6 (only tracing events added near it; the call itself unchanged).

Functions (each ≤ 50 body lines, ≤ 2 params, nesting ≤ 2, no commented-out code, private members by default):

```rust
pub(super) const DEFAULT_FILTER: &str = "info";
const RUST_LOG: &str = "RUST_LOG";

pub(super) fn init_logging() -> Result<(), LogInitError>
```
Body outline (no compilation surprises; exact API calls encoded):
1. `let raw = std::env::var(RUST_LOG).ok();`
2. `let directive = resolved_directive(raw.as_deref())?;`
3. `let subscriber = tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&directive)).with_writer(std::io::stderr).finish();`
4. `let install = tracing::subscriber::set_global_default(subscriber);`
5. `install.map_err(|error| LogInitError::SubscriberInstall { reason: error.to_string() })`

```rust
fn resolved_directive(raw: Option<&str>) -> Result<String, LogInitError>
```
Body outline (pure; the injectable seam for ALL tests — NO test touches the environment, see §6):
1. `let value = raw.unwrap_or_default().trim();`
2. `if value.is_empty() { return Ok(DEFAULT_FILTER.to_string()); }`
3. `validate(value)` — delegate below
4. `Ok(value.to_string())`

```rust
fn validate(directive: &str) -> Result<(), LogInitError>
```
Body: `EnvFilter::try_new(directive).map(|_| ()).map_err(|_| LogInitError::FilterInvalid { directive: directive.to_string() })`.
Note: the successfully built `EnvFilter` from `try_new` is discarded; `init_logging` rebuilds with `EnvFilter::new` from the validated string. Two tiny builds at startup once — acceptable, and it keeps `resolved_directive` returning a plain `String` that tests can compare exactly (EnvFilter itself is not `PartialEq`, so asserting on it would be guesswork — the String-returning shape is the testable one).

### 2.3 `LogInitError` inhabiting — `src/app/error.rs` (encoded)
Replace the last 10 lines (uninhabited enum + its `Display`) with:

```rust
#[derive(Debug)]
pub(super) enum LogInitError {
    /// A malformed logging filter directive (operator-supplied, e.g. an invalid
    /// RUST_LOG value) prevented filter construction; startup aborts so the bad
    /// value is never silently substituted.
    FilterInvalid { directive: String },
    /// The global default subscriber could not be installed. Unreachable while
    /// the startup funnel calls `init_logging` exactly once, and reported here
    /// so a future second call site can never silently drop logs.
    #[allow(dead_code)]
    SubscriberInstall { reason: String },
}
```
- `#[allow(dead_code)]` is REQUIRED on `SubscriberInstall`: the variant is never constructed by the runtime path (single call site), and `clippy --locked -- -D warnings` compiles without `cfg(test)`, so an un-allowed never-constructed variant fails dev-checks. Keep the doc comment as the rationale (rules permit minimal comments for complex rationale).
- Categories (for the doc comment/comment reviewers): `FilterInvalid` = expected operator error class (fatal, exit 1, one stderr line); `SubscriberInstall` = invariant-violation class (fatal, exit 1).
- Rewrite `Display for LogInitError` with the two real arms:
  - `FilterInvalid { directive }` → `write!(formatter, "invalid logging filter directive '{directive}'")`
  - `SubscriberInstall { reason }` → `write!(formatter, "failed to install logging subscriber: {reason}")`
  - Note the top-level `StartupError::Display` arm already wraps with `failed to initialize logging: {error}` — leave that wrapper UNCHANGED (so the final stderr line reads `error: failed to initialize logging: invalid logging filter directive 'app=banana'`).
- Do NOT change `StartupError` itself (variants and `exit_code()` already cover `LogInit(_) => STARTUP_FAILURE_EXIT`).

### 2.4 Subscription/ordering invariant (init-once) — encoded decision
- **PICKED: keep the single call site and document the invariant; add NO guard.** `tracing::subscriber::set_global_default` is itself the guard (second attempt returns `Err`, mapped to `SubscriberInstall`); adding a `Once`/`OnceLock` layer would duplicate the crate's own guarantee. Encode the invariant ONLY as (a) the doc comment in `logging.rs` first paragraph (update the existing text: the current text falsely promises "a later phase replaces this body" and "initialization cannot fail" — both now false; the new text must say: "called exactly once per process, before any configuration load or mode handler runs; the binding is the process's one logging boundary") and (b) the `SubscriberInstall` variant doc. No changes to `mod.rs` call structure.
- Ordering is already correct in `mod.rs::start_mode` and MUST remain: `initialize_logging()` → `load_configuration` → `dispatch`.

---

## 3. What Gets Logged (levels, fields, layering — encoded)

### 3.1 The three info events (in `src/app/mod.rs` only, function `start_mode`)
Add `use tracing::info;` at the top of `mod.rs` (with the other plain `use` items). Exactly three events, in execution order:

1. **Placement decision (PICKED): immediately AFTER the `initialize_logging()` block**, as the first statement before `load_configuration(...)`. Rationale to record (do not reproduce the reasoning in code): an event emitted BEFORE subscriber installation is silently dropped (no global dispatcher is set yet), and when `initialize_logging()` fails the mode never proceeds — so only the after-init placement guarantees delivery. Event — selected command + config path:
   ```rust
   info!(command = command_name(&invocation.mode), config_path = %invocation.config_path, "mode selected");
   ```
   ```rust
   fn command_name(mode: &cli::Mode) -> &'static str {
       match mode {
           cli::Mode::Auth => "auth",
           cli::Mode::Run => "run",
       }
   }
   ```
   (private; keeps the event call single-line; avoids `Debug`-format of `Mode`, which would leak the `Mode` type's debug shape instead of a stable field value).
2. **On success only of `load_configuration`** — validation outcome (no config field values, no path duplication if unchanged... path already in event 1):
   ```rust
   Ok(app_config) => {
       info!("configuration loaded and validated");
       dispatch(...)
   }
   ```
3. **After `dispatch` returns, regardless of placeholder outcome** — high-level completion:
   ```rust
   let code = dispatch(...);
   info!(command = command_name(&invocation.mode), exit_code = code, "mode finished");
   code
   ```
   (`auth`/`run` placeholders already print their own user-facing lines in `modes.rs` — untouched; a status event at info level is one structured record, not the duplicated-error pattern.)

### 3.2 Failure path layering (no duplication — PICKED decision)
- **Fatal startup failures ([`StartupError`] outfalls in `evaluate`/`start_mode`) are reported by the EXISTING single stderr line in `report_failure` ONLY.** They are NOT logged at `warn`/`error` level through tracing.
- Justification (encode verbatim for reviewer): (a) the only failure class reachable after successful subscriber installation is `Configuration` — logging it through `tracing` AND printing it in `report_failure` writes the same error twice, which TODO §3 forbids ("without duplicating the same error at multiple layers"); (b) `LogInit` failures occur while the logging machinery is being established — they cannot be delivered through tracing at all; one uniform stderr funnel keeps every fatal path identical. Stay with `report_failure` exactly as-is — DO NOT add a `tracing::error!` there.
- Mode-handler placeholder statuses (exit 3, from `modes.rs`) are messages to the operator, not "config/app setup errors"; they are covered by info event 3 only, and their existing stderr lines stay untouched. `Usage`/`Help` paths parse before logging exists — never logged.

### 3.3 Secrets rule (reviewer-checkable contract — encode verbatim into module doc)
Only these log fields may ever appear: `command` (one string constant: `auth`/`run`), `config_path` (the operator-supplied path text), `exit_code` (numeric constant), plus free-text event messages with no field values appended. Forbidden forever, and this task adds 0 events that violate it: secret/credential/token values, stream keys, ingestion URLs, config field VALUES of any kind (including title/description/privacy), and the full config file contents. Justification: the config path is already visible on stderr (usage banner) and is not a secret; field values can be rendered later behind a redaction policy if a future phase truly needs them — until then they are omitted, not staged. Reviewer check: `grep` for `info!`/`tracing` events must return ONLY the three events in `mod.rs`, each with fields from the allowed list.

---

## 4. Test Plan (encoded)

All new tests live in-module (`#[cfg(test)]`). Global-state discipline: tests MUST NOT touch `RUST_LOG` or any env var, and MUST NOT call `init_logging` (side-effectful global subscriber → cross-test nondeterminism under parallel test threads). The testable surface is the injected pure function (§2.2) and the error mapping (§2.3).

### 4.1 `src/app/logging.rs` tests
| # | Test name | Given | Expect |
|---|---|---|---|
| 1 | `default_directive_constant_is_info` | — | `resolved_directive(None).unwrap()` == `"info"` and constant equals `"info"` |
| 2 | `unset_env_resolves_to_default_filter` | — | `resolved_directive(None)` == DEFAULT_FILTER |
| 3 | `empty_and_whitespace_env_resolves_to_default_filter` | `Some("")`, `Some("  \t  ")`, `Some("\n")` | all == DEFAULT_FILTER |
| 4 | `valid_level_directive_is_kept_verbatim` | `Some("debug")` | == `"debug"` |
| 5 | `combined_directive_is_kept_verbatim` | `Some("trace,app=off")` | == `"trace,app=off"` |
| 6 | `whitespace_around_directive_is_trimmed` | `Some(" debug ")` | == `"debug"` (documented trim behavior) |
| 7 | `malformed_directive_is_rejected_as_filter_invalid` | `Some("app=banana")` | `Err(LogInitError::FilterInvalid { directive })` with `directive == "app=banana"` (match with `if let`, panic on other outcome; the helper `filter_invalid_directive(&result)` mirrors `startup_error...` helpers in `cli_tests.rs`) |

Do NOT add EnvFilter-behavior introspection tests (no PartialEq on EnvFilter; Display of EnvFilter is formatting trivia, not a contract of this task).

### 4.2 `src/app/error.rs` tests
- Exit-code mapping (the REAL, previously-impossible exit-1 test this task owes):
  - `StartupError::LogInit(LogInitError::FilterInvalid { directive: "app=banana".to_string() }).exit_code()` == `STARTUP_FAILURE_EXIT`.
  - Same assertion for `StartupError::LogInit(LogInitError::SubscriberInstall { reason: "already set".to_string() }).exit_code()`.
- Display formatting (`format!`, compared with `starts_with`/`contains` only, never full-string equality):
  - `format!("{}", StartupError::LogInit(LogInitError::FilterInvalid { ... }))` starts with `"error"`-agnostic wrapper prefix `"failed to initialize logging: "` and contains `"invalid logging filter directive"` and the directive text.
  - Same shape for `SubscriberInstall` containing `"failed to install logging subscriber"`.
- Suggested test structure: one small `#[cfg(test)] mod tests` at the bottom of `error.rs` (modeled on `mod.rs`'s existing tests module, using `use super::*`); each assertion set is one `fn` (keep each ≤ 50 lines).

### 4.3 Determinism/coverage boundaries (record in completion report)
- No unit test asserts subscriber side effects (writing to stderr, global dispatched-subscriber state); the delivery of events is covered by clippy/dev-checks and later manual validation — same boundary the global plan pre-chose ("scope tests to what is testable without subscriber side effects").
- After this task `cargo test --locked` must remain green with the existing config/CLI tests intact — no existing test's behavior may change measurably (none of them read logs; docs-only concern).

---

## 5. Concrete Steps (ordered; exit-code authoritative on every cargo step)

Precondition check (before step 1): `git status` on `feat/phase01-config-app-skeleton` shows only this task's intended changes (Tasks 1–2 code already committed; no leftover TODO/tmp files). No tmp folders anywhere — not in the repo root, not outside it.

**(Implementation host: no Rust/Cargo installed — ALL cargo runs via the Alpine VM MCP.)**

1. VM workflow constant: for every cargo execution below, call `alpine-vm_vm_status` first, then issue ONE single `alpine-vm_vm_run_command` with exactly one of these commands (no chained `&&`):
   - resolve → `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check`
   - locked → `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
   - tests → `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked`
   - full gate → `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
2. `Cargo.toml`: add the two dependency lines per §1.2 (Edit tool, no newlines issues), run the *resolve* command above. On exit 0 → run *locked*. On failure → §1.4 pin fallback, then re-run *resolve* and *locked*.

3. **Commit "build: add tracing and tracing-subscriber dependencies"** (single commit; nothing else staged; Cargo.lock diff is only graph entries; KEEP the pins if any happened §1.4). Read `.gitignore` and `git status` first; never stage `target/`, `logs/**` etc. (none should appear; `Cargo.lock` is the intended tracked change).
4. `src/app/error.rs`: inhabit `LogInitError` + Display arms + tests per §2.3/§4.2 (Edit tool).
5. `src/app/logging.rs`: replace the no-op body per §2.2 (keep signature; update the module doc per §2.4; keep the `use super::error::LogInitError;` import, add `use tracing_subscriber::EnvFilter;`).
6. `src/app/mod.rs`: add `use tracing::info;`, the `command_name` helper, and the three info events in `start_mode` per §3.1. Confirm no other change to the file (`evaluate` / `initialize_logging` / `load_configuration` / `dispatch` / `report_failure` names and bodies untouched; `report_failure` gets NO tracing call per §3.2).
7. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` (all new tests + existing must pass, including the new exit-1 mapping test).
8. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`; exit 0 (all four checks; format cleanly applied — the `with_writer(std::io::stderr)` chain + `?` funnel lines must be rustfmt-clean at write time, no manual reformat games).
9. If anything red: fix minimally per the exact failing step; NO scope creep; re-run the failed step first, then the full gate again.

10. **Commit** — message EXACTLY (sequential, never amend):
    - `feat: initialize structured logging with env filter`
    (covers the logging.rs and error.rs changes and the mod.rs events; one cohesive feature commit; no commit before BOTH gates pass).
11. Report: completion report items 1–8 per TODO ("Required Completion Report") — mention: dependency addition + selected version(s) resolved (or pin list from §1.4), the three events, the invalid-filter-is-fatal decision with justification, the no-duplication stderr rule, tests added + dev-checks PASS lines verbatim, and the explicit statement that README/`.agent` updates are Task 4's responsibility (they remain stale at this point by design).

---

## 6. Constraints Checklist (pre-commit review, junior may self-assert only through this list)

- [ ] "Structured logging initializes once" — initialized once via the single funnel; invariant documented in module doc + SubscriberInstall doc.
- [ ] `RUST_LOG` honored; unset/empty → `info` default; malformed directive → clean startup failure, not panic, not silent substitution.
- [ ] Selected command + startup/validation outcomes recorded at info level; `Modes` and `cli.rs` and `main.rs` untouched.
- [ ] No duplicate error reporting across layers (fatal path = one stderr line via the existing funnel; info tracing events are outcomes, not error duplicates).
- [ ] No secret values, tokens, stream keys, or full config contents ever reach a log field (§3.3 list; reviewer can grep).
- [ ] `init_logging()` signature unchanged, single call site unchanged, `LogInitError` name unchanged, `StartupError` untouched.
- [ ] No tracing anywhere in `src/config/`; app layer still does not spawn processes (no `Command`); no Tokio; no signals/supervision/lifecycle additions.
- [ ] Container usage: no VM cwd pollution; no new root-level files; `logs/` has no product output from this task; `Cargo.lock` tracked, `Cargo.lock`'s root-only exception holds; compose service name and flags are exact; dev-checks green BEFORE commit; no CI/pipeline changes.
- [ ] Files ≤ 200 lines (logging.rs/error.rs after edits), functions ≤ 50 lines body, ≤ 2 params, nesting ≤ 2, single-section boolean conditions, no commented-out code, self-documenting names (Set of exact helper names used: `command_name`, `resolved_directive`, `validate`; no clever names).

---

## 7. Hand-off Notes for Later Sub-steps
- Code-reviewer (4.3): check in addition the "exactly 3 info events, all inside `start_mode`" invariant, the `#[allow(dead_code)]` rationale comment on `SubscriberInstall`, and that each new test from §4.1/§4.2 is its own small `fn` (no mega-test functions).
- Docs-specialist (4.4): module-level rustdoc of `logging.rs` is ALREADY covered by §2.2/§2.4 wording (which the implementer writes into the file); docs-specialist may polish only those doc comments — README/`docs/`/`.agent` content remains Task 4's exclusive territory.
- For Task 4 (out of this scope): README will need a sentence documenting the log filter default (`info`) and RUST_LOG override — schedule there, not here.
- Not implemented in THIS step: nothing from Task 4, no `docs/` updates, no `[DONE]` markers; if ambiguity arises, the implementer returns to the caller rather than assuming.
