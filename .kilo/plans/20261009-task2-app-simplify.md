# Simplification Plan — Task 2: Application and CLI Skeleton

- Date: 2026-10-09
- Reviewer: code-simplifier (4.3 Simplification)
- Commits under review: `76bfbc6`, `a0a8a93` on `feat/phase01-config-app-skeleton`
- Baseline plan (must not be violated): `.kilo/plans/20261009-task2-app-cli-skeleton.md`
- Scope reviewed: `src/main.rs`, `src/app/{mod,cli,cli_tests,modes,logging,error}.rs`, `src/config/{mod,test_fixtures}.rs`

## 0. Verdict

No MUST-level (rule-driven) simplifications. All binding rules pass: every file ≤ 200 lines (largest 146), every function ≤ 50 lines, every function ≤ 2 params, nesting ≤ 2, all boolean conditions single-section (match guards included), no commented-out code, no `#[allow(` anywhere in `src/` (verified by grep), private-by-default surface respected, no dead helpers (every private fn reachable).

Two RECOMMENDED (DRY) items below. Both are behavior-preserving: identical stdout/stderr bytes, identical exit codes, unchanged CLI surface, unchanged `load_from_path` API, unchanged logging seam, zero new dependencies.

## 1. Hard guardrails for the implementer

- Do NOT change any user-visible string, exit code, or accepted/rejected argument form.
- Do NOT touch `src/app/logging.rs`, `src/app/modes.rs`, `src/main.rs`, or `src/config/*`.
- Do NOT add dependencies, do NOT restructure modules, do NOT add `#[allow]`, do NOT change any existing function signature except the deletions/additions written verbatim below.
- Do NOT run git commit. The caller decides commit timing.

## 2. Step R1 — Centralize the `unexpected argument '<token>'` usage-error constructor (RECOMMENDED, DRY)

Exact string `unexpected argument '...'` is currently built at three sites: `cli.rs` `help_result` (with `HELP_FLAG`), `cli.rs` `resolved_mode` (already-given branch), and `mod.rs` `non_utf8_usage_error`. One constructor on `StartupError` removes the duplicated format string.

**R1.1 — `src/app/error.rs`:** inside the existing `impl StartupError` block, insert this method immediately BEFORE `fn exit_code`:

```rust
    /// Usage failure for a token that cannot appear where it was found.
    pub(super) fn unexpected_argument(token: &str) -> Self {
        StartupError::Usage(format!("unexpected argument '{token}'"))
    }
```

**R1.2 — `src/app/cli.rs` `help_result`:** replace the `Some(_)` arm body

```rust
        Some(_) => Err(StartupError::Usage(format!(
            "unexpected argument '{HELP_FLAG}'"
        ))),
```

with

```rust
        Some(_) => Err(StartupError::unexpected_argument(HELP_FLAG)),
```

**R1.3 — `src/app/cli.rs` `resolved_mode`:** replace

```rust
    if already_given {
        return Err(StartupError::Usage(format!(
            "unexpected argument '{token}'"
        )));
    }
```

with

```rust
    if already_given {
        return Err(StartupError::unexpected_argument(token));
    }
```

**R1.4 — `src/app/mod.rs`:** delete the whole function

```rust
fn non_utf8_usage_error(token: &OsString) -> StartupError {
    StartupError::Usage(format!("unexpected argument '{}'", token.to_string_lossy()))
}
```

and change `string_token` to

```rust
fn string_token(token: &OsString) -> Result<String, StartupError> {
    token
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| StartupError::unexpected_argument(&token.to_string_lossy()))
}
```

Behavior note (no action needed): `&token.to_string_lossy()` is `&Cow<str>`, which deref-coerces to `&str`; the produced message bytes are identical. Output is already pinned by `cli_tests.rs::rejects_extra_positional_after_command` (exact strings incl. `'--help'`) and `mod.rs::evaluate_maps_non_utf8_argument_to_usage_exit`.

## 3. Step R2 — Guard the duplicated default-path string between `USAGE` and `DEFAULT_CONFIG_PATH` (RECOMMENDED, drift test)

`cli.rs` keeps `DEFAULT_CONFIG_PATH = "config/config.json"` and separately hardcodes `default: config/config.json` inside the `USAGE` text. Do not restructure the const into a formatter (allocation/shape churn); pin the consistency with a test instead.

**R2.1 — `src/app/cli_tests.rs`:** insert this test directly after `default_config_path_constant_is_stable`:

```rust
#[test]
fn usage_text_advertises_default_config_path() {
    assert!(USAGE.contains(DEFAULT_CONFIG_PATH));
}
```

Both items are already in scope through the existing `use super::*` (the file is included as `mod cli_tests` inside `cli`). Expected result: test passes immediately (no production change).

## 4. Verification (Alpine VM Docker workflow — exact order, one command per call)

1. MCP `alpine-vm_vm_status` → must report running + sshReachable (confirmed reachable at plan-writing time).
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` → expect ALL pass and app-layer test count to grow from 20 to 21 (only addition is R2.1; R1 adds/removes no tests).
5. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → MUST exit 0 with the four PASS lines (fmt-check, check, test, clippy).

Any non-zero exit or changed test count beyond +1 → stop and report to caller; do not improvise.

## 5. Candidates examined and REJECTED (do not implement; recorded for the caller)

1. **Shared not-implemented reporter for `modes::auth`/`run`** — rejected as churn: the duplicated shape is one `eprintln!` + constant return across exactly 2 sites with two distinct long messages; params are already 1; and the baseline plan §1.6 says Task 3 may re-route these very lines through logging, so a helper now would likely be reshaped immediately.
2. **Deriving `USAGE` from `DEFAULT_CONFIG_PATH` via `format!`** — rejected: turns a `const` into a function/allocation; drift risk is instead pinned by R2.1 with zero production change.
3. **Inlining `is_help_token`/`is_config_flag`** — rejected: named predicates are mandated by baseline plan §4 (single-section boolean rule).
4. **Inlining single-use helpers `unknown_option_error`, `help_result`, `finished_invocation`** — rejected: consistent file style, removes no duplication, lengthens `parse`.
5. **Relocating exit-code constants (`SUCCESS_EXIT`, `STARTUP_FAILURE_EXIT`, `USAGE_ERROR_EXIT`, `NOT_IMPLEMENTED_EXIT`) into one module** — rejected: each constant stays with the code that produces it; `modes.rs::exit_codes_remain_distinct_and_stable` already guards the cross-module contract.
6. **`TokenCursor` alias inlining; `os_args` helper in `mod.rs` tests** — rejected: ≤2 single-line sites each, below the tiny-helper threshold (Task 1 precedent: helpers only where they remove real multi-site duplication).
7. **Adding a `LogInitError` constructor to test the `LogInit → exit 1` mapping** — OUT OF SIMPLIFIER AUTHORITY: baseline plan §3 lists a "logging-init failure → 1" test, but `LogInitError` is an uninhabited enum by plan design (Task 3 owns the seam shape), so that mapping is structurally covered by `exit_code`'s arm yet not unit-testable this task. Flagged to caller as a known baseline-plan gap, not a simplification.

## 6. Impact summary

- Files touched by R1+R2: `src/app/error.rs` (+4 lines), `src/app/cli.rs` (net −4 lines), `src/app/mod.rs` (net −3 lines), `src/app/cli_tests.rs` (+4 lines). All remain far below every size rule.
- Public behavior: unchanged (CLI surface, exact messages, exit codes 1/2/3, `--help` output, `load_from_path` API, logging seam).
- Dependencies: none added. Architecture: unchanged.
