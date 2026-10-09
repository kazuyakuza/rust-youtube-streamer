# Task 3 Simplification Plan — Structured Logging (4.3)

- Date: 2026-10-09
- Role: code-simplifier (4.3) — review complete; this file is the ONLY change authorization for the implementer
- Reviewed commits: `e3ebd64` (deps), `1b49e88` (logging feature), branch `feat/phase01-config-app-skeleton`
- Baseline plan (must not be violated): `.kilo/plans/20261009-task3-structured-logging.md`
- Baseline verified green at review time: `cargo test --locked` through the compose service → exit 0, `48 passed; 0 failed`.

## 0. Scope Card

- Exactly ONE file may change: `src/app/error.rs` (the `#[cfg(test)] mod tests` block ONLY).
- Production code is untouched: enum `LogInitError`, its `Display` impl, `StartupError`, exit-code constants, `src/app/logging.rs`, `src/app/mod.rs`, `Cargo.toml`, `Cargo.lock`.
- Hard constraints preserved by construction: 48 tests remain 48 tests (names unchanged), all assertions unchanged in meaning, zero behavior change.

## 1. Finding Summary

Rule scan (all PASS, no MUST items): file sizes 121/129/160/17 ≤ 200; every fn body ≤ 50; every fn ≤ 2 params; nesting ≤ 2; no multi-section boolean conditions; no commented-out code; no dead code. The only real simplification win is genuine duplicated test-fixture scaffolding in `error.rs` (each of the two `StartupError::LogInit(...)` constructions appears twice: once in the exit-code test, once in the Display test) — same pattern the repo already solved with `assert_blank_value_rejected`/`expect_validation_error` (`src/config/validation_tests.rs`) and `args`/`parsed_invocation`/`usage_error_message` (`src/app/cli_tests.rs`).

## 2. Step 1 (RECOMMENDED, DRY) — dedupe error.rs test fixtures

Extract the two repeated constructions into private fixture helpers inside the existing `mod tests`. No other edit anywhere.

Target content for `src/app/error.rs` lines 88–129 (the whole `#[cfg(test)] mod tests` block replaced verbatim with this):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn filter_invalid_startup_error() -> StartupError {
        StartupError::LogInit(LogInitError::FilterInvalid {
            directive: "app=banana".to_string(),
        })
    }

    fn subscriber_install_startup_error() -> StartupError {
        StartupError::LogInit(LogInitError::SubscriberInstall {
            reason: "already set".to_string(),
        })
    }

    #[test]
    fn log_init_filter_invalid_maps_to_startup_failure_exit() {
        let startup_error = filter_invalid_startup_error();
        assert_eq!(startup_error.exit_code(), STARTUP_FAILURE_EXIT);
    }

    #[test]
    fn log_init_subscriber_install_maps_to_startup_failure_exit() {
        let startup_error = subscriber_install_startup_error();
        assert_eq!(startup_error.exit_code(), STARTUP_FAILURE_EXIT);
    }

    #[test]
    fn filter_invalid_display_wraps_message_and_keeps_directive() {
        let startup_error = filter_invalid_startup_error();
        let message = format!("{startup_error}");
        assert!(message.starts_with("failed to initialize logging: "));
        assert!(message.contains("invalid logging filter directive"));
        assert!(message.contains("app=banana"));
    }

    #[test]
    fn subscriber_install_display_wraps_message_and_keeps_reason() {
        let startup_error = subscriber_install_startup_error();
        let message = format!("{startup_error}");
        assert!(message.starts_with("failed to initialize logging: "));
        assert!(message.contains("failed to install logging subscriber"));
        assert!(message.contains("already set"));
    }
}
```

Do NOT: rename the four `#[test]` fns (baseline §4.2 names + 48-test inventory), change any expected string, merge tests, touch lines 1–87, or "helpfully" simplify `logging.rs`/`mod.rs`.

## 3. Recorded Verdicts — explicit NON-goals (do not revisit)

1. `LogInitError::FilterInvalid { directive }` echo: **KEEP — useful, not redundant.** The `StartupError` wrapper adds only the class ("failed to initialize logging"); the echo names the rejected operator value, which appears nowhere else on stderr (the env var is not printed). Without it the operator cannot identify the typo. Tests already pin this contract.
2. `#[allow(dead_code)]` on `SubscriberInstall` (baseline §2.3 demanded it): **KEEP current state — no attribute.** The baseline premise ("never constructed by the runtime path") is factually wrong against the baseline's own §2.2 body: `logging.rs:41` constructs the variant via `map_err`. Dead-code analysis sees the constructor, so the attribute is unnecessary and adding it would be misleading. Gate is proven green without it. No action.
3. `validate` + `resolved_directive` pair (logging.rs): **KEEP both.** Merging saves one 7-line fn, contradicts baseline §2.2's encoded shape, and would inline the fatal-rejection semantics into resolution. No real complexity win.
4. logging.rs tests #1 vs #2 near-overlap and single-use helper `filter_invalid_directive`: **KEEP.** Both shapes are baseline §4.1-mandated; removing/merging any test breaks the 48-test constraint.
5. Shared consts for Display strings between impl and tests (`"failed to initialize logging: "` etc.): **REJECT.** Tests must pin the operator-visible strings independently; sharing a const with the impl makes the assertions tautological.
6. `"app=banana"` duplicated across `logging.rs` tests and `error.rs` tests: **KEEP.** Cross-file fixtures in private test modules; unifying creates test-module coupling for zero drift risk reduction.
7. `pub(super) const DEFAULT_FILTER`: **KEEP.** Wider than strictly needed (all uses are in-file), but baseline §2.2 encoded this exact declaration; narrowing is zero-value churn against the baseline.
8. Two `EnvFilter` builds (`try_new` discarded, `new` used): **KEEP.** Baseline §2.2 note encoded it (String-returning testable seam; EnvFilter lacks PartialEq).

## 4. Verification (exact order; exit code is authoritative; single commands only; no `&&` chains)

1. `alpine-vm_vm_status` → must show running + SSH reachable.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → expect exit 0 (target block above is rustfmt-clean as written; if fmt reports a diff, re-run `cargo fmt` inside the same compose service and re-check).
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` → expect exit 0 AND `48 passed; 0 failed` with the four `app::error::tests::*` names unchanged.
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → expect exit 0.
5. Full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0.

Any red step: fix minimally inside §2's scope, re-run the failed step, then re-run step 5. If a fix would need to escape §2's scope, STOP and return to caller.

## 5. Commit (only after step 5 is exit 0)

- Read `.gitignore`, then `git status`; stage ONLY `src/app/error.rs` (Cargo.lock must show no change — `--locked` guarantees it; if anything else appears dirty, do NOT stage it, return to caller).
- Message exactly: `refactor: dedupe startup error test fixtures`. Never amend, no force, push only per caller instruction (this step does not push).

## 6. Junior Self-Check List

- [ ] Only `src/app/error.rs` changed; only its `#[cfg(test)] mod tests` block.
- [ ] 48 tests still pass; test names identical to baseline §4.2.
- [ ] Assertions/expected strings byte-identical in meaning to before.
- [ ] Helpers are private, ≤ 2 params (0 here), nesting ≤ 2, no comments added.
- [ ] dev-checks exit 0 before commit; nothing else staged.
