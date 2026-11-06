# Task-1 4.3 Simplification Plan — provider micro-fraction parser + stage-row stamp wiring

Scope: commits `c17b137` + `6e3cbd5` on `fix/diagnostics-followup` (HEAD `6e3cbd5`). Binding impl plan: `20261106-followup-task1-provider-timestamp-plan.md`. Workflow step 4.3. This file is the ONLY write of the step; all edits below are for the junior implementer pass.

Lens: behavior-neutral. Untouchable: JSON row bytes/keys/values, gating semantics, lock spans, epoch math, epoch discard, serde defaults/renames, test assertion VALUES, ≤2-arg shapes, no test renames, no `#[allow(`, post-fmt ≤200 lines everywhere.

VM validation: none needed. Every finding below is static (read at `6e3cbd5`); the 4.2 gate already passed at this HEAD; validation for this plan rides the steps' own gates (§5).

## 0. Candidate ledger

Adopted:

- A1 — `TimingDetails::stage(..).with_provider(..)` chain duplicated at BOTH stage sites (`collector.rs:102–103` written, `collector_stages.rs:122–123` rendered) → fold into `DequeuedMessage::stage_details(lag_ns)` (S1).
- A2 — `drive_stage_cycle` (collector_provider_tests) duplicates `drive_full_stage_cycle` (collector_tests) call-for-call (same 12 calls, same advances 10/20/5/7), and `queue_metrics_alone_does_not_emit_timing_rows_or_histograms` (collector_tests:157–170) duplicates the cycle a third time → move the parameterized helper into `diagnostics_test_fixtures.rs` (S2).
- A3 — `1_792_931_445`/`"1792931445"` duplicated across `collector_provider_tests.rs` (const + match string) and `chat_delivery_diagnostics_tests.rs:90` (match string); shared compilable parent exists: `diagnostics_test_fixtures` is `#[cfg(test)] pub(crate) mod` (mod.rs:21–22) and already imported by the run roster (its line 9–11) → single `pub(crate) const PROVIDER_EPOCH_FIXTURE` (S3). The 2026-11-01 subtree rule stays honored: no cross-subtree coupling — `chat_worker_flow_tests.rs:14 PINNED_PUBLISHED_EPOCH` predates Task 1 (commit `0647860`), OUT of scope, untouched.
- A4 — `Some(1_792_931_445)` repeated 5× in `live_chat_rfc3339_tests.rs` forces 5 multi-line wrapped asserts; roster-local `const` matches the sibling precedent (`live_chat_stream_mapping_tests.rs:14`) and collapses each assert to one line (S4). youtube↔app remain separate (roster-local, not cross-module).

Rejected (with numbers):

- R1 — `send_stop_aware` double `…diagnostics.as_ref().and_then(|_| …)` mirror capture (chat_stream_delivery.rs:93–102) → helper `correlation_and_stamp` extracted: net post-rustfmt **+4 lines on a 198/200 file** (10 chain lines → 2 call lines, but +12 wrapped helper + blank). Duplication is fmt-inflation of two one-logical-line captures; the shape is plan-prescribed (§1.5) and each loop iteration must own the `String` before `try_send` moves `message`. A `.map(|_| (tuple))` single capture introduces nested-Option shuffles and a depth-3 `if let` inside `match` inside `loop` (max-depth rule). NOT folded. If the caller wants it folded anyway, it requires a sibling split first (STOP-protocol adjudication), not a 4.3 micro-fold.
- R2 — Fold `collector_provider_tests.rs` INTO `collector_tests.rs`: VERIFIED NOT FOLDABLE. New content is ~40 lines (`fixture_message` 5 + test 18 + test 15 + blanks), not the guessed 14 → 187 + 40 = ~227 > 200. Even AFTER S2/S3 slim both files, merging lands ~195/200 with 11 tests — cap-fire magnet; the separate ~50-line roster is strictly better budget hygiene. Keep separate.
- R3 — Fold `fixture_message` into an existing helper: no identical helper exists (`secret_message` pins different author/text literals `"SECRET-AUTHOR"`/`"SECRET-TEXT"` that assertions match verbatim — not mergeable). Keep.
- R4 — `events.rs` (194) line relief: its bulk is serde structs with pinned field order + the untouched `with_provider` builder (still needed; single construction point after S1). No safe fold. Status quo + risk note (§6).
- R5 — `state_dequeue` tuple (collector.rs:185–190): 1 consumer, plan-mandated (§1.3), keeps the lock span single-acquisition. Keep.
- R6 — `take_dequeued` match-arm blocks (state.rs:101–111): each arm has 2 statements; `map`/`and_then` shuffles would be less clear. Keep.
- Dead params / redundant borrows / visibility audit: NONE found — every new param (`provider_epoch_seconds` on `report_enqueue_success`, `stamp` on `report_enqueue_success_if_enabled`) is consumed; new items are at minimum visibility (`state_dequeue` private, `with_provider`/`stage_details`/`record_message_render` `pub(super)`, `DequeuedMessage`/`InFlightEntry` fields `pub(super)`). No action.

## 1. S1 — single stage-row builder on `DequeuedMessage` (A1)

1a. `src/app/diagnostics/collector_stages.rs` — insert this block immediately AFTER `DequeuedMessage`'s closing `}` (line 61) and BEFORE `impl DiagnosticsCollector {`:

```rust
impl DequeuedMessage {
    /// The shared rendered/written stage-row payload for one dequeued entry.
    pub(super) fn stage_details(&self, lag_ns: u64) -> TimingDetails<'_> {
        TimingDetails::stage(self.correlation.as_deref(), lag_ns)
            .with_provider(self.provider_epoch_seconds)
    }
}
```

1b. Same file, `record_message_render` — replace its last three lines:

```rust
        let details = TimingDetails::stage(entry.correlation.as_deref(), message_ns)
            .with_provider(entry.provider_epoch_seconds);
        self.emit(&TimingEvent::new("rendered", details));
```

with exactly:

```rust
        self.emit(&TimingEvent::new("rendered", entry.stage_details(message_ns)));
```

1c. `src/app/diagnostics/collector.rs` — in `note_write_completed`, replace:

```rust
            if let Some(entry) = dequeued {
                let lag_ns = now.saturating_sub(entry.stamp_nanos);
                let details = TimingDetails::stage(entry.correlation.as_deref(), lag_ns)
                    .with_provider(entry.provider_epoch_seconds);
                self.emit(&TimingEvent::new("written", details));
            }
```

with exactly:

```rust
            if let Some(entry) = dequeued {
                let lag_ns = now.saturating_sub(entry.stamp_nanos);
                self.emit(&TimingEvent::new("written", entry.stage_details(lag_ns)));
            }
```

Behavior: identical values (same lag inputs, same details, same emit order inside the same lock/borrow scopes). `collector.rs` still uses `TimingDetails` (dequeued row) and `DequeuedMessage` (already imported, line 17) — no import edits. `collector_stages.rs` imports already carry `TimingDetails`/`TimingEvent` (line 9) — no import edits. Do NOT delete the `state.rs`/`with_provider` builder.

Checkpoint: `cargo check` clean; `cargo test -q collector` green; `collector.rs` = 194, `collector_stages.rs` = 140 post-fmt.

## 2. S2 — shared `drive_stage_cycle` fixture (A2)

2a. `src/app/diagnostics/diagnostics_test_fixtures.rs` — add to the imports: `use crate::chat::ChatMessage;` (with the other `crate::` uses). Append after `collector_with_output_file` (line 79):

```rust
/// The deterministic full stage cycle: receive → receipt → enqueue →
/// dequeue → render → write, with the provider stamp read from `message`.
pub(crate) fn drive_stage_cycle(harness: &TestCollector, message: &ChatMessage) {
    harness.collector.report_batch_received(1);
    harness.collector.note_worker_receipt();
    harness.advance(10);
    harness
        .collector
        .report_enqueue_success(message.message_id(), message.timestamp());
    harness.advance(20);
    harness.collector.note_message_dequeued(message);
    harness.collector.note_render_started();
    harness.advance(5);
    harness.collector.note_render_completed();
    harness.collector.note_write_started();
    harness.advance(7);
    harness.collector.note_write_completed();
}
```

2b. `src/app/diagnostics/collector_tests.rs` — DELETE `fn drive_full_stage_cycle` (lines 32–48 plus its trailing blank). Its two call sites `drive_full_stage_cycle(&harness);` (lines 82, 140) become `drive_stage_cycle(&harness, &secret_message());` (the glob import at line 10 already exposes it). In `queue_metrics_alone_does_not_emit_timing_rows_or_histograms`: delete `let message = secret_message();` (line 155) and replace the 14-line inline cycle (lines 157–170) with `drive_stage_cycle(&harness, &secret_message());`. All assertions stay byte-identical.

2c. `src/app/diagnostics/collector_provider_tests.rs` — DELETE the local `fn drive_stage_cycle` (lines 12–26 plus trailing blank) and widen the fixture import to:

```rust
use super::diagnostics_test_fixtures::{
    collector_with_output_file, drive_stage_cycle, TestCollector,
};
```

Call sites (lines 40, 58) already read `drive_stage_cycle(&harness, &message);` — unchanged. `use crate::chat::ChatMessage;` stays (used by `fixture_message`).

Behavior: identical call sequences and advance values at all four cycle sites; test names untouched; test count unchanged.

Checkpoint: `cargo test -q collector` + `cargo test -q provider` green; `collector_tests.rs` ≈ 155, `diagnostics_test_fixtures.rs` ≈ 144, `collector_provider_tests.rs` ≈ 50.

## 3. S3 — shared provider-epoch fixture constant (A3)

3a. `src/app/diagnostics/diagnostics_test_fixtures.rs` — above `TEST_PACING_INTERVAL_NANOS` (line 16) add:

```rust
/// The pinned epoch seconds of `PUBLISHED_AT_FIXTURE` (`2026-10-25T12:30:45Z`).
pub(crate) const PROVIDER_EPOCH_FIXTURE: u64 = 1_792_931_445;
```

3b. `src/app/diagnostics/collector_provider_tests.rs` — DELETE `const PROVIDER_EPOCH_FIXTURE: u64 = 1_792_931_445;` (line 10) and add `PROVIDER_EPOCH_FIXTURE` to the S2c import list. In `timestamp_bearing_message_populates_all_three_stage_rows`, replace the assert block (lines 42–48) with exactly:

```rust
    let stamped_row = format!("\"provider_epoch_seconds\":{PROVIDER_EPOCH_FIXTURE}");
    assert_eq!(content.matches(stamped_row.as_str()).count(), 3);
```

`timestamp_absent_rows_stay_null_on_all_three_stages` keeps its `"\"provider_epoch_seconds\":null"` literal assert UNCHANGED (no constant involved).

3c. `src/app/run/chat_delivery_diagnostics_tests.rs` — add `PROVIDER_EPOCH_FIXTURE` to the existing `use crate::app::diagnostics::diagnostics_test_fixtures::{…}` list (rustfmt will order it). In `worker_stamp_survives_the_bounded_in_flight_entry`, replace the assert block (lines 88–93) with exactly:

```rust
    let stamped_row = format!("\"provider_epoch_seconds\":{PROVIDER_EPOCH_FIXTURE}");
    assert_eq!(content.matches(stamped_row.as_str()).count(), 2);
```

Behavior: the matched string evaluates to the identical byte sequence as today; the pinned count 3/2 is unchanged. Single source for the value across both app rosters.

Checkpoint: `cargo test -q provider` + `cargo test -q chat_delivery` green.

## 4. S4 — roster-local epoch constant in the rfc3339 tests (A4)

`src/youtube/live_chat_rfc3339_tests.rs` — after `use super::parse_rfc3339_seconds;` add:

```rust
const PUBLISHED_EPOCH: u64 = 1_792_931_445;
```

In BOTH tests, replace every `Some(1_792_931_445)` expected value (5 sites: lines 11, 15, 25, 38, 42) with `Some(PUBLISHED_EPOCH)`, collapsing each affected 3-line wrapped `assert_eq!` to one line where it fits ≤100 cols (it does for all five: longest becomes 86 cols). The `Some(0)` and all `None` asserts stay byte-identical. Name mirrors the sibling precedent `live_chat_stream_mapping_tests.rs:14`. No cross-subtree import.

Checkpoint: `cargo test -q rfc3339` green; file ~36/200.

## 5. Gates (Alpine VM, one command per call, `alpine-vm_vm_status` first)

1. `docker compose run --rm rust sh -c "cargo fmt --check"`
2. `docker compose run --rm rust sh -c "cargo test -q collector"`
3. `docker compose run --rm rust sh -c "cargo test -q provider"`
4. `docker compose run --rm rust sh -c "cargo test -q rfc3339"`
5. `docker compose run --rm rust sh -c "cargo test -q chat_delivery"`
6. Full: `sh scripts/dev-checks.sh` — acceptance: 555/0 (count unchanged; these folds add and remove zero tests), every file ≤200 post-fmt, `git grep -c "allow(" -- src/` unchanged.

Order: S1 → S2 → S3 → S4, one commit per step (or one commit `refactor: dedup stage-row builder and provider-epoch test fixtures per 4.3` if the caller prefers a single step commit — caller's call, both pass the gate).

## 6. Post-fold line budget (verified arithmetic, ±1 fmt)

| File | Now | After |
|---|---|---|
| collector.rs | 197 | 194 |
| collector_stages.rs | 134 | 140 |
| events.rs | 194 | 194 (untouched) |
| chat_stream_delivery.rs | 198 | 198 (untouched — R1) |
| diagnostics_test_fixtures.rs | 124 | ~145 |
| collector_tests.rs | 187 | ~155 |
| collector_provider_tests.rs | 65 | ~48 |
| chat_delivery_diagnostics_tests.rs | 176 | ~173 |
| live_chat_rfc3339_tests.rs | 44 | ~36 |

Residual risk notes for the reviewer/caller (no 4.3 action): `chat_stream_delivery.rs` at 198 and `events.rs` at 194 have no safe fold inside Task-1 scope; ANY next touch of either needs sibling-split adjudication by the caller (STOP protocol, precedent R3). 4.2 verification item "`with_provider` used at BOTH stage sites" — after S1 both stage rows still flow through `with_provider`, via the single `DequeuedMessage::stage_details` construction point; the checklist line should be read accordingly.

## 7. STOP conditions for the implementer

Halt and report if: any file >200 post-fmt; a fold requires touching `events.rs`, `chat_stream_delivery.rs`, queue-row shapes, or any assertion value; `format!` const-interpolation is rejected by the toolchain (fallback: revert ONLY S3c/3b asserts to the literal strings and keep S3a/3b-const for the `.with_timestamp` pin, then report); any test count moves off 555.
