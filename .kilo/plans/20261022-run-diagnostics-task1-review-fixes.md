# Task-1 Review Fixes — Run-Path Streaming Failure Diagnostics

Reviewer finding: the implementation matches the binding plan and caller rulings; targeted tests pass. One project-rule issue remains and must be fixed before 4.4/4.5.

## Issue: max-depth rule violation in the gate diagnostics helpers

The max-depth rule (`.kilo/rules/max-depth.md`) limits nested blocks to 2 levels. The new code introduces two 3rd-level nests:

1. `src/app/run/gate.rs` `run_confirmation`: the `Err(error) => { if gate_diagnostics::is_confirmation_timeout(&error) { ... } ... }` arm places an `if` block at level 3.
2. `src/app/run/gate_diagnostics.rs` `log_last_stream_status`: the `Ok(Some(stream)) => match observed_status_label(&stream) { ... }` arm places a second `match` at level 3.

The single-section-boolean rule is already satisfied (the condition is a named predicate call). The fix is to extract the 3rd-level logic into helpers, keeping every function ≤2 args, ≤50 lines, and ≤2 nesting levels.

---

## Fix A — `src/app/run/gate.rs`: extract the timeout-only log wrapper

### 1. Replace the fatal arm body

In `run_confirmation`, change:

```rust
        Err(error) => {
            if gate_diagnostics::is_confirmation_timeout(&error) {
                gate_diagnostics::log_last_stream_status(plan.transport.as_ref(), &plan.stream_id);
            }
            plan.stop_flag.request_stop();
            GateFinish::Fatal(error)
        }
```

to:

```rust
        Err(error) => {
            log_last_stream_status_on_timeout(&plan, &error);
            plan.stop_flag.request_stop();
            GateFinish::Fatal(error)
        }
```

### 2. Add the helper immediately after `run_confirmation` (before the test module)

```rust
fn log_last_stream_status_on_timeout(plan: &GatePlan, error: &LiveApiError) {
    if gate_diagnostics::is_confirmation_timeout(error) {
        gate_diagnostics::log_last_stream_status(plan.transport.as_ref(), &plan.stream_id);
    }
}
```

This helper is private to `gate.rs`, takes exactly 2 arguments, keeps the single-section boolean behind the existing `is_confirmation_timeout` predicate, and removes the 3rd nesting level from `run_confirmation`.

---

## Fix B — `src/app/run/gate_diagnostics.rs`: flatten the lookup-result match

Replace `log_last_stream_status` with the following two functions:

```rust
pub(super) fn log_last_stream_status(transport: &dyn LiveApiTransport, stream_id: &str) {
    match transport.lookup_stream(stream_id) {
        Ok(Some(stream)) => log_observed_stream_status(&stream),
        Ok(None) | Err(_) => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
    }
}

fn log_observed_stream_status(stream: &LiveStream) {
    match observed_status_label(stream) {
        Some(label) => {
            info!(
                stream_status = label,
                "{}", READINESS_TIMEOUT_STREAM_STATUS_MESSAGE
            );
        }
        None => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
    }
}
```

Then delete the old `log_last_stream_status` body.

`log_observed_stream_status` is private, 1 argument, and keeps the inner `match` at level 2. The outer `match` now combines the two identical `Ok(None)`/`Err(_)` arms, removing duplication.

---

## Verification after applying fixes

1. `cargo fmt` (or run the dev-checks gate) — the edits are rustfmt-clean.
2. Targeted suites:
   - `cargo test --locked -- gate_tests streaming_diagnostics_tests`
   - `cargo test --locked -- ffmpeg_stream_tests`
3. Confirm no new warnings beyond the 6 already adjudicated acceptable.
4. Confirm file caps still hold: `gate.rs` ≈ 166 lines, `gate_diagnostics.rs` ≈ 74 lines.

No other source changes are required.
