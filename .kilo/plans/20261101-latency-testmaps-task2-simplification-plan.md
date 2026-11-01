# Task-2 Code Simplification Plan (step 4.3) — Latency Harness

- Scope: Task-2 test commits `748ab1f` + `4eb310f` only (`src/app/run/latency_load_fixtures.rs`, `latency_load_source.rs`, `latency_load_tests.rs`, `latency_load_loss_tests.rs`, the sanctioned `enabled_collector_with_pacing` block in `src/app/diagnostics/diagnostics_test_fixtures.rs`, and the footer registration in `chat_stream_delivery.rs`).
- Binding design plan: `.kilo/plans/20261101-latency-testmaps-task2-plan.md` (not re-litigated; every step below is consistent with §2–§3 and the §8 line budget).
- Behavior-neutral: test code only. ZERO assertion values, messages, operands, or phase inputs change. No renames. Production code untouched beyond the already-sanctioned fixture block (which this plan does NOT modify further).
- Executed by: junior implementer, 50% restriction. Follow the steps exactly, in order. No judgment calls.

## Baseline evidence (this branch, VM, 2026-11-01)

`cargo test --locked latency_load` via `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked latency_load`:
`10 passed; 0 failed; 0 ignored; finished in 0.09s` (540 filtered out). No compiler warning points at any `latency_load*` file (the 12 emitted warnings are all pre-existing, in unrelated files — leave them).

Current raw line counts (match post-`cargo fmt` on HEAD): fixtures 158 · source 175 · tests 184 · loss 104 · diagnostics fixtures 124 · chat_stream_delivery 192.

---

## Step A — Fold the two test files' duplicated local helpers into `latency_load_fixtures.rs` (single compile unit; do A1+A2+A3 together, verify only after A3)

The two test modules each define byte-identical `phase`, `scenario`, `configured_interval_nanos`, plus a duplicated `CHANNEL_CAPACITY` const, and the loss module inlines a copy of `assert_scenario_pacing`. The interval-derivation expression (`PipelinePacing::from_config(&valid_config().video).expect(...).interval_nanos()`) exists three times (fixtures `run_scenario` + both test files). Fold all five into the shared parent module. Do NOT do this step partially: an unused private helper fires `dead_code` and the toolchain lints deny warnings; the whole step is one atomic unit.

### A1. `src/app/run/latency_load_fixtures.rs` — add the shared block

Insert the following EXACT block between the `impl LatencyScenario { ... }` block (ends line 60) and the `pub(super) struct ScenarioOutcome` line, keeping one blank line before and after:

```rust
const CHANNEL_CAPACITY: u64 = CHAT_CHANNEL_CAPACITY as u64;

fn configured_interval_nanos() -> u64 {
    PipelinePacing::from_config(&valid_config().video)
        .expect("validated FPS derives pacing")
        .interval_nanos()
}

fn phase(frames: u64, numerator: u64, denominator: u64) -> RatePhase {
    RatePhase {
        frames,
        rate: RateProfile {
            numerator,
            denominator,
        },
    }
}

fn scenario(label: &'static str, phases: Vec<RatePhase>) -> LatencyScenario {
    LatencyScenario { label, phases }
}

fn assert_scenario_pacing(run: &ScenarioOutcome) {
    assert_eq!(
        run.pacing_interval_nanos,
        configured_interval_nanos(),
        "the scenario interval derives from the configured FPS"
    );
}
```

(`CHANNEL_CAPACITY`, `phase`, `scenario`, `configured_interval_nanos`, `assert_scenario_pacing` are copied verbatim from `latency_load_tests.rs` lines 12/15–23/25–27/29–33/35–41; the `assert_eq!` value pair and message string are unchanged.)

Then inside `run_scenario`, replace these two lines (current 72–73):

```rust
    let pacing = PipelinePacing::from_config(&config.video).expect("validated FPS derives pacing");
    let pacing_interval_nanos = pacing.interval_nanos();
```

with this one line:

```rust
    let pacing_interval_nanos = configured_interval_nanos();
```

Derivation is identical (same `expect` message, same `valid_config().video` source; `config` stays because it is still used by `open_log_store`). No import changes needed in fixtures (`PipelinePacing`, `valid_config`, `CHAT_CHANNEL_CAPACITY` are already imported there).

### A2. `src/app/run/latency_load_tests.rs` — remove the locals, import from `super`

Delete exactly:
- lines 6–8 (`use crate::app::pipeline::PipelinePacing;`, `use crate::app::run::chat_worker::CHAT_CHANNEL_CAPACITY;`, `use crate::config::valid_config;`) — nothing else in the file uses them;
- the single `use super::{run_scenario, LatencyScenario, RatePhase, RateProfile, ScenarioOutcome};` line (line 10), replaced by:

```rust
use super::{
    assert_scenario_pacing, configured_interval_nanos, phase, run_scenario, scenario, CHANNEL_CAPACITY,
};
```

- the `const CHANNEL_CAPACITY` line (line 12) — KEEP `const LOW_RATE_FRAMES` (line 13);
- the whole helpers region: `fn phase` (15–23), `fn scenario` (25–27), `fn configured_interval_nanos` (29–33), `fn assert_scenario_pacing` (35–41), plus the blank lines between/after them (lines 15–42), leaving exactly ONE blank line between the `const LOW_RATE_FRAMES` line and the first `#[test]`.

`LatencyScenario`, `RatePhase`, `RateProfile`, `ScenarioOutcome` are no longer named in this file (call sites use `phase(...)`/`scenario(...)`/`&run` inference only) — do NOT re-add them to the import. Every test body below stays byte-identical: same names resolve through `use super::`; `phase`/`scenario`/`assert_scenario_pacing` receive the same arguments; all assertion values/messages untouched.

### A3. `src/app/run/latency_load_loss_tests.rs` — remove the locals, reuse the pacing helper

Delete exactly:
- `use crate::app::pipeline::PipelinePacing;` (line 6) — KEEP `use crate::config::valid_config;` (line 7; still used by `scenario_pacing_interval_derives_from_configured_fps`);
- the `use super::{assert_full_round_trip_accounting, run_scenario, LatencyScenario, RatePhase, RateProfile};` block (lines 9–11), replaced by:

```rust
use super::{
    assert_full_round_trip_accounting, assert_scenario_pacing, configured_interval_nanos, phase,
    run_scenario, scenario, CHANNEL_CAPACITY, RateProfile,
};
```

- `const CHANNEL_CAPACITY` (line 13) — KEEP `const MIXED_DUE_TOTAL` (line 14) exactly as written (its `40 + 16 * 2 + 64 / 2` literals are pinned expected-value math, NOT the channel capacity; do not touch — reviewer lane);
- `fn phase` (16–24), `fn scenario` (26–28), `fn configured_interval_nanos` (30–34) plus the blank lines between them (16–35), leaving one blank line before the first `#[test]`.

In `round_trip_counts_prove_no_silent_loss_and_bounded_memory`, replace the five-line inline pacing assert (49–53):

```rust
    assert_eq!(
        run.pacing_interval_nanos,
        configured_interval_nanos(),
        "the scenario interval derives from the configured FPS"
    );
```

with:

```rust
    assert_scenario_pacing(&run);
```

The moved assert executes the identical `assert_eq!` with identical operands and identical message string. `RateProfile` stays imported (directly named by `rate_profile_due_counts_follow_rational_schedule`).

CONFLICT RULE (reviewer lane): the only assertion the fold moves is the pacing-derivation one. If the concurrent code-reviewer flagged that assert's message text or operands, take the reviewer's final wording and place it inside the shared `assert_scenario_pacing` helper in `latency_load_fixtures.rs` (both call sites inherit it) — never revert this fold to reinstate the inline copy.

### Step A verification

1. `cargo fmt` (in the container).
2. `wc -l` the three touched files (table below must hold; STOP protocol per plan §8 if any > 200).
3. `cargo test --locked latency_load` → expect exactly `10 passed; 0 failed` and the SAME 10 test names (no renames).

---

## Step B — Remove the dead `impl MessageSource for PacedQueueProducerSource`

`src/app/run/latency_load_source.rs` lines 153–157:

```rust
impl MessageSource for PacedQueueProducerSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        self.step()
    }
}
```

Delete this impl block and one adjacent blank line (leave a single blank line between the closing `}` of the inherent impl and the `/// Shares the producer ...` doc comment).

Evidence it is dead: exhaustive grep of `src/` shows `PacedQueueProducerSource` is constructed only inside `Rc<RefCell<...>>` (`latency_load_fixtures.rs:84`) and consumed only via `SharedProducer` (`latency_load_fixtures.rs:99`, which calls the inherent `step()` through its own `MessageSource` impl) and `drain_leftover()` (inherent). No `Box::new(producer)`-style direct trait object exists; the impl is a leftover of the plan §2.3 sketch that the final `SharedProducer` shape replaced. `MessageSource` and `ChatSourceError` imports stay used (`SharedProducer` impl + `step()` signature) — do not remove imports.

### Step B verification

`cargo fmt` · `wc -l` source (expect ~169) · `cargo test --locked latency_load` → `10 passed; 0 failed`.

---

## Step C — Narrow harness-internal visibility to the minimum that compiles

In `src/app/run/latency_load_fixtures.rs` ONLY: delete every `pub(super)` visibility marker (21 occurrences: `RateProfile` + 2 fields + `advance`, `RatePhase` + 2 fields, `LatencyScenario` + 2 fields + `total_frames` + `due_total`, `ScenarioOutcome` + 5 fields, `run_scenario`, `assert_full_round_trip_accounting`), making all of them module-private (e.g. `pub(super) struct RateProfile` → `struct RateProfile`; `pub(super) numerator: u64,` → `numerator: u64,`).

Why private compiles: every consumer (`latency_load_tests`, `latency_load_loss_tests`, `latency_load_source`) is a DESCENDANT module of `latency_load_fixtures` (registered via `#[path]` at its footer), and Rust descendants can name ancestor-private items. The parent `chat_stream_delivery` references none of them (footer registration only — verified by grep, §Baseline). No `pub(super)` item is used outside the fixture subtree.

DO NOT touch visibility anywhere else:
- `latency_load_source.rs` `pub(super)` markers are already minimal (a parent module cannot see a child's private items — `latency_load_fixtures` needs `ProducerParts`/`PacedQueueProducerSource`/`SharedProducer` and the `ProducerParts` fields);
- `diagnostics_test_fixtures.rs` stays as committed: `enabled_collector_with_pacing`, `TestCollector`, its `collector` field, and `pub(crate) use super::events::SummarySnapshot` are all load-bearing — `mod events;` is private to the diagnostics subtree (verified `diagnostics/mod.rs:10`), so the `pub(crate)` re-export is the ONLY crate-visible path `run` can use, and `Clone` is required by the `snapshotter`/producer split in `run_scenario`.

### Step C verification

`cargo fmt` · `wc -l` fixtures (unchanged count vs Step A) · `cargo test --locked latency_load` → `10 passed; 0 failed` · `cargo test --locked diagnostics` and `cargo test --locked burst` sanity (these files are untouched by this plan; must stay green per binding plan §9.2).

---

## Projected post-fmt line counts (STOP protocol at >200 per plan §8)

| File | Now | Post-A | Post-B | Post-C | Bound |
|---|---|---|---|---|---|
| `src/app/run/latency_load_fixtures.rs` | 158 | ~186 | 186 | 186 | ≤200 ✓ (was projected ≤150 in plan §8; growth is the folded shared helpers, still 14 under the hard cap) |
| `src/app/run/latency_load_source.rs` | 175 | 175 | ~169 | 169 | ≤200 ✓ |
| `src/app/run/latency_load_tests.rs` (TIGHT) | 184 | ~154 | 154 | 154 | ≤200 ✓ (frees ~30 lines in the tight file — plan §4.4 doc regeneration unaffected) |
| `src/app/run/latency_load_loss_tests.rs` | 104 | ~79 | 79 | 79 | ≤200 ✓ |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | 124 | untouched | — | — | ≤200 ✓ |
| `src/app/run/chat_stream_delivery.rs` | 192 | untouched | — | — | ≤200 ✓ |

If `cargo fmt` re-wraps a `use super::{...}` list differently than written, the count may shift by ±1 per file — allowed; still ≤200.

## Observables-preserved ledger

- No test renamed; all 10 names identical (plan §3 pin; §4.4 doc table stays valid).
- No assertion value, operand, message, or macro changed anywhere. The one `assert_eq!` that MOVES (`assert_scenario_pacing`) relocates byte-for-byte and absorbs the loss file's identical inline copy (net: the same assertion executes, once per test, as before).
- No scenario phase inputs changed: `phase(...)`/`scenario(...)` call sites are textually identical; only their definitions move to the shared parent.
- `run_scenario` derives `pacing_interval_nanos` from `PipelinePacing::from_config(&valid_config().video)` with the same `expect` message; same value (`33_333_333` ns at fixture FPS); `open_log_store`, tempdir prefix, pipeline wiring, drain order unchanged.
- Step B removes code with zero call sites (grep-proven); the pipeline's source stays `SharedProducer` — same `step()` sequence per frame: advance → pull → open-window → release → enqueue → inject.
- Step C is compile-time visibility only; no runtime effect (test-only subtree).
- No sleeps introduced (suite still 0.09 s class); ≤2 args, ≤2 depth, ≤50-line bodies all preserved; no commented-out code; no new `#[allow(`.
- Post-A/B/C suite result must remain `10 passed; 0 failed` with no NEW compiler warnings attributable to `latency_load*` files.

## Candidates inspected and REJECTED (with reasons)

1. **Fold repeated `message_channel_ns.min_ns/max_ns` latency-pair asserts (5 tests) into a helper** — would relocate assertion lines the concurrent reviewer may edit in the same lanes (conflict-advoid) and needs a 3-arg helper or a param type for the ≤2-args rule; savings (~10 lines) do not justify the cross-lane collision risk. Not encoded.
2. **Named const for `128` sustained frames (`SUSTAINED_FRAMES`, tests 3/4/5 + `2 * 128` expected value)** — touching `assert_eq!(snapshot.received_total, 2 * 128)` rewrites an expected-value expression the reviewer is re-deriving simultaneously. Not encoded; note for a future docs/gate round.
3. **`run.leftover.len() as u64` accessor on `ScenarioOutcome`** — 2 occurrences of a 2-token cast; a method adds surface, not removes it.
4. **Condense the `PacedQueueProducerSource` doc-comment "(the frame-granularity retry model)"** — this is the honest threaded-retry deviation statement; it is already minimal and truthful; shortening risks weakening the drift declaration required by plan §2.2/§4.3. KEPT verbatim.
5. **Flatten `release_count`'s `loop` to the plan-sketch single-phase `if let`** — the loop is the implementation's defense against zero-frame phases; plan §2.2 step 3 describes only the active-phase advance. Semantics-identical today, but the fold changes control flow for no duplication win. Left.
6. **Remove `_kept_temp` underscore / rename to plan's `kept_temp`** — the field is never read (its purpose is lifetime extension); the `_` prefix is exactly what keeps the tree at zero `#[allow(dead_code)]` (§7 hygiene). Left.
7. **Narrow `pub(crate) use super::events::SummarySnapshot;` in the diagnostics fixtures** — required: `mod events` is private to the diagnostics subtree; this re-export is the only crate-visible name path for `run`. Left (minimal exposure already).
8. **Narrow `latency_load_source.rs` `pub(super)` markers** — already the minimum that compiles (parent cannot see child-privates). Left (Step C documents this).
9. **Second `TestCollector::clone`/`collector.clone()` pair in `run_scenario`** — needed to keep the snapshot handle (`snapshotter`), the producer clock, and the pipeline diagnostics hook sharing one fake clock/state; a real, load-bearing clone, not redundant. Left.
10. **`ProducerParts.pacing_interval_nanos` "redundant" with the harness clock** — the producer advances the clock BY that interval per step; it is the mechanism, not a duplicate. Left.

## End state

Three steps, each independently verifiable, total effect: 4 duplicated helper definitions + 1 duplicated const + 1 duplicated assert + 1 dead trait impl + 21 over-broad visibility markers removed; `latency_load_tests.rs` 184 → ~154 (the tight file gains headroom for §4.4-name stability); behavior of the 10 tests identical.
