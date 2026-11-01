# Task-2 Code Review Fix Plan

**Branch:** `feat/observability-game-handoff`  
**Review step:** 4.3 (Code Review) for Task 2 only  
**Trigger:** max-arguments-per-method rule violation in test helper; all other corrected expectations verified as strict pins.

## Summary

The implementation matches the binding Task-2 plan for correctness, determinism, capacity math, loss-accounting invariants, and roster hygiene. All 10 new `latency_load` tests pass; sanity filters (`chat_delivery_diagnostics`, `diagnostics`, `burst`) remain green; the full gate observes 550/0.

One rule violation remains: the convenience helper `phase(frames, numerator, denominator)` in both test rosters has **three** positional arguments, exceeding the project’s ≤2-args rule. The fix introduces a `rate_profile` helper in `latency_load_fixtures.rs` and narrows `phase` to two arguments. No assertion changes; no pins weakened.

---

## Fix 1 — `src/app/run/latency_load_fixtures.rs` (add shared helper)

**Location:** after the `RateProfile` impl block, before `RatePhase`.

**Before shape:** helper does not exist.

**After shape:**

```rust
pub(super) fn rate_profile(numerator: u64, denominator: u64) -> RateProfile {
    RateProfile {
        numerator,
        denominator,
    }
}
```

**Projected impact:** file grows from 158 lines to ~161 lines (≤200).

---

## Fix 2 — `src/app/run/latency_load_tests.rs` (narrow `phase` to ≤2 args)

**Before shape:**

```rust
fn phase(frames: u64, numerator: u64, denominator: u64) -> RatePhase {
    RatePhase {
        frames,
        rate: RateProfile {
            numerator,
            denominator,
        },
    }
}
```

and call sites such as:

```rust
vec![phase(LOW_RATE_FRAMES, 1, 16)]
```

**After shape:**

```rust
fn phase(frames: u64, rate: RateProfile) -> RatePhase {
    RatePhase { frames, rate }
}
```

Update the `use super::{...}` block to include `rate_profile`, and rewrite call sites as:

```rust
vec![phase(LOW_RATE_FRAMES, rate_profile(1, 16))]
```

Apply the same transform to all 10 `phase(...)` invocations in this file.

**Projected impact:** file grows from 184 lines to ~188 lines (≤200); all assertions and scenario definitions remain byte-identical in meaning.

---

## Fix 3 — `src/app/run/latency_load_loss_tests.rs` (narrow `phase` to ≤2 args)

**Before shape:** identical 3-arg `phase` helper and call sites such as:

```rust
phase(1, 40, 1)
```

**After shape:**

```rust
fn phase(frames: u64, rate: RateProfile) -> RatePhase {
    RatePhase { frames, rate }
}
```

Update the `use super::{...}` block to include `rate_profile`, and rewrite call sites as:

```rust
phase(1, rate_profile(40, 1))
```

Apply the same transform to all 6 `phase(...)` invocations in this file.

**Projected impact:** file grows from 104 lines to ~108 lines (≤200); all assertions and scenario definitions remain byte-identical in meaning.

---

## Verification after fix

1. `cargo fmt`
2. Re-verify line counts with `(Get-Content file).Count` for the three touched files.
3. Run targeted filters read-only:
   - `cargo test --locked latency_load`
   - `cargo test --locked chat_delivery_diagnostics`
   - `cargo test --locked diagnostics`
   - `cargo test --locked burst`
4. Confirm all 550 tests pass via `scripts/dev-checks.sh` and record the gate log.

---

## Notes

- No production behavior changes.
- No assertion weakening; the only change is helper arity compliance.
- No new `#[allow(`, no sleeps, no commented-out code, no secrets introduced.
- `ProducerParts` remains the sanctioned ≤2-args parameter object for `PacedQueueProducerSource::new`.
