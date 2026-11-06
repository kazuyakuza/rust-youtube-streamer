# Task-2 Implementation Plan — Reconnect rows gain a non-secret `reason` label

Source TODO: `.agent/todos/20261106/20261106-todo-1.md` §Task 2 (Label reconnect diagnostic rows with a non-secret cause).
Global plan: `.kilo/plans/20261106-diagnostics-followup.md` §Per-Task Pre-Analysis/Task 2.
Branch state verified at planning time: `fix/diagnostics-followup`, HEAD `eeffc4e` (v0.14.1 — **no further version bump; 0.14.1 stays**, Task 1's step-3 bump already landed), working tree clean, dev-check gate OBSERVED 555 tests / 0 failures.
Workflow cycle: 4.1b (this plan) → 4.2 implementer → 4.3 review/simplify → 4.4 docs → 4.5b audit → 4.6 close-out.
Front-end determination: **NOT front-end (no 4.1a)**.
Approval: per-task plan **AUTO-APPROVED** (per caller instruction).

Constraints honored: ≤2 args/method, ≤200 lines/file post-fmt, no new deps, no new `#[allow(`, no sleeps, offline-only tests, secrets never in labels/rows.

---

## 1. Verified Trace (actual code, cited)

### 1a. Where the classification exists today

- `src/app/run/chat_stream_worker.rs`
  - `ConsumeVerdict` enum (L59–64) with the `CleanClose` arm; `consume()` returns `ConsumeVerdict::CleanClose` at **L130** when `session.next_batch()` resolves `None`.
  - `run_stream_loop()` L95–100: `ConsumeVerdict::CleanClose` → `worker.handle_clean_close()`.
  - `ConsumeVerdict::Failed(error)` L89–94 → `worker.handle_failure(error)`.
  - `STREAM_CLEAN_CLOSE_DETAIL: &str` L29–30 — the ONLY sanctioned non-secret detail for the clean-close arm ("the stream connection closed without an error status").
- `src/app/run/chat_stream_delivery.rs`
  - `handle_clean_close()` **L155–157**: `self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string())`.
  - `handle_failure()` L142–152: non-terminal, non-graceful errors pass `is_transient_failure` (`src/app/run/chat_failure.rs:19`) and fall through to `self.await_reestablishment(error.to_string())` at **L151**. The detail for this arm is **`error.to_string()` = raw provider error text — NOT permitted as a row label** (secret/provider-internals rule; TODO §Context + `<error:provider internals>`).
- **Both arms converge** on `await_reestablishment(&mut self, last_detail: String)` **L163–175**; the single diagnostics call is `report_reconnect(delay)` at **L170–172**. Today ONE row shape serves both causes — exactly the operator finding (no cause label).
- Backoff arithmetic: `backoff_delay_ms` (`src/app/run/chat_worker_helpers.rs:27–34`) → attempt 1 = 1000 ms (pinned in `chat_worker_budget_tests.rs:74`). Matches the operator log's flat `backoff_ms:1000`.

### 1b. Truthful mapping of the operator-observed pattern (`logs/run/20261106-021348.log`)

Every `batch_size:0` row followed by `backoff_ms:1000` maps to:

1. A session delivered an empty batch → `process_batch(batch)` (delivery **L37–54**): once `history_pending` is false, L50–52 calls `diagnostics.report_batch_received(0)` → `QueueEvent::batch` row (collector_stages **L72–77**). Discarded history batches (first token batch) emit NO batch row (L38–47 returns before the report).
2. The same session then hit end-of-stream → `next_batch()` → `None` → `ConsumeVerdict::CleanClose` (worker L130) → `handle_clean_close` → `await_reestablishment` → `report_reconnect(1000)`.
3. `consecutive_failures` resets to 0 on every `Ok` batch (worker `consume()` L142), so each cycle reconnects at the 1000 ms floor — the observed flat backoff over `reconnect_count` 1→17.

Conclusion: ALL observed reconnect rows came through the **clean-close arm**, and `handle_clean_close` DOES emit a reconnect row (via the shared await/report path). The transient arm today appears only on classified transient retrieval errors (`LiveApiError` variants; the run never showed one). The plan covers both arms.

### 1c. The reconnect reporting surfaces (collector side)

- `src/app/diagnostics/collector_stages.rs` `report_reconnect(&self, backoff_ms: u64)` **L116–121** — the ONLY reconnect report fn; called ONLY from delivery L171 (grep-verified). Extends to 2 args: `(backoff_ms, reason)` ≤2 ✓.
- `src/app/diagnostics/state.rs` `add_reconnect(&mut self, backoff_ms: u64)` **L86–89**: increments `reconnect_count`/`reconnect_backoff_total_ms` only — **stays 1-arg, no change** (summary counters are label-free; TODO requires the reason on the ROW only).
- `src/app/diagnostics/events.rs`:
  - `QueueEvent` struct L80–90 — fields `event, epoch_seconds, correlation, batch_size, queue_depth, backpressure, wait_ns, backoff_ms`, all serde-serialize always; typed-`Option` fields (`backpressure`, `wait_ns`, `batch_size`, `correlation`) **serialize as `null` unconditionally** (operator log confirms `backpressure:null`/`wait_ns:null` on queue rows). There is NO `skip_serializing_if` in events.rs today.
  - `with_depth(queue_depth)` skeleton L101–112 initializes every kind-specific field to `None`.
  - `QueueEvent::reconnect(backoff_ms, queue_depth)` L135–139 — 2 args, leaves `backoff_ms` set, everything else `None`. The row therefore serializes today with `"backpressure":null,"wait_ns":null,"backoff_ms":1000`.
- Serializer: `collector.rs emit()` L154–162 → `serde_json::to_string` → field order = struct declaration order ⇒ a field declared AFTER `backoff_ms` appends at the row's end.
- `skip_serializing_if` precedent in repo: `src/youtube/live_models/request_bodies.rs:97` (`#[serde(skip_serializing_if = "Option::is_none")]`).
- JSONL row-pin test precedent: `collector_provider_tests.rs` (tempdir output file + `content.matches(...)` counting) and `collector_tests.rs` ("not content.contains" byte-absence pins); delivery-level file pin precedent: `chat_delivery_diagnostics_tests.rs::worker_stamp_survives_the_bounded_in_flight_entry` (L59–90).

### 1d. Existing test expectations that touch reconnect rows (honest re-pin audit)

- `chat_delivery_diagnostics_tests.rs::worker_reports_reconnect_with_backoff` L143–160: asserts `snapshot.reconnect_count == 1` and `reconnect_backoff_total_ms == 1_000`. State counters are UNCHANGED ⇒ **stays green, no edit** (recorded here so 4.3 does not "fix" it).
- `chat_worker_budget_tests.rs` (backoff/backoff-tests) and `chat_worker_flow_tests.rs`: assert counters/loop behavior, not row shape ⇒ untouched.
- No test greps JSONL for `"backoff_ms"` today (grep-verified) ⇒ no historical row pins exist to re-pin; the new pins are additive.
- `collector_tests.rs::emits_summary_after_interval_and_not_earlier` L93–115 (summary row shape) and `queue_metrics_alone_does_not_emit_timing_rows_or_histograms` L131–153 (queue-row content) must stay green — they are the byte-stability witnesses for the summary split below.

---

## 2. Final shape / skip decision (encoded)

**`QueueEvent` field:** add `reason: Option<&'a str>` declared **LAST** (after `backoff_ms`) with:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
reason: Option<&'a str>,
```

Rationale, pinned:

- `skip_serializing_if = "Option::is_none"` is REQUIRED on this field ONLY. The `with_depth` skeleton initializes `reason: None`; because of the skip attr, batch/enqueue/full rows **never emit a `reason` key at all** (byte-absence, not `null` emission). Do NOT add the attr, nor remove it, from any other field — `backpressure`/`wait_ns`/`batch_size`/`correlation` keep their pinned `null`-emission shape.
- The reconnect constructor attaches the label via a builder ((`mut self`) style mirroring the existing `TimingDetails::with_queue_depth` precedents at events.rs L45–48), keeping `QueueEvent::reconnect(backoff_ms, queue_depth)` at ≤2 args:

```rust
pub(super) fn with_reason(mut self, reason: &'static str) -> Self {
    self.reason = Some(reason);
    self
}
```

- A param-object (second `Queue*Fields` struct) was considered and REJECTED: one field, larger line cost, no additional clarity over the established `with_*` builder lineage in this file.
- Declaration order ⇒ serialized key order ⇒ `"reason"` appends at row end, after `backoff_ms`. This keeps the row's existing key prefix untouched (compatible append).

**`report_reconnect`:**

```rust
pub(crate) fn report_reconnect(&self, backoff_ms: u64, reason: &'static str) {
    let depth = self.state_depth(|state| state.add_reconnect(backoff_ms));
    if self.settings.queue_metrics {
        self.emit(&QueueEvent::reconnect(backoff_ms, depth).with_reason(reason));
    }
}
```

(≤2 args ✓; `state.add_reconnect` call unchanged.)

**Delivery threading (every call site passes a label — the classifier enforces it, never the log format):**

- `await_reestablishment(&mut self, last_detail: String, reason: &'static str)` (2 args ✓).
- `handle_clean_close` L156 → `self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string(), CHAT_CLEAN_CLOSE_REASON);`
- `handle_failure` L151 (transient arm only) → `self.await_reestablishment(error.to_string(), CHAT_TRANSIENT_FAILURE_REASON);`
- L171 → `diagnostics.report_reconnect(delay, reason);`

**Unknown/other-cause honesty:** the `reason` field REMAINS `Option` + skip so a future cause arm that reaches the row without a label still writes the row (no missing-row regression); TODAY both production call sites always label. `None` is reachable in production code paths only through the skeleton (other row kinds).

### Exact label words (final choice)

| Case | Constant (in `src/app/run/chat_worker.rs`, beside the `CHAT_*` message constants group, `pub(super)`) | Value |
|---|---|---|
| Server clean close re-establishment (`handle_clean_close`) | `pub(super) const CHAT_CLEAN_CLOSE_REASON: &str = "clean_close";` | `"clean_close"` |
| Transient failure re-establishment (`handle_failure` transient arm) | `pub(super) const CHAT_TRANSIENT_FAILURE_REASON: &str = "transient_failure";` | `"transient_failure"` |

Rationale: lowercase-underscore per TODO; short, stable, grep-safe for operators; derived from the delivery classification (`ConsumeVerdict::CleanClose` vs `is_transient_failure` arms); no provider internals (unlike `error.to_string()`); plain words, non-secret. Constants live in `chat_worker.rs` next to `CHAT_ENDED_MESSAGE` (same kind of operator-facing fixed word), re-imported by delivery through its existing `use super::chat_worker::{...}` block, giving the new test module direct `super::` access to pin the label words by constant (not by duplicated string literal).

---

## 3. Proactive line-budget decision: events.rs summary split (encoded NOW)

`events.rs` is 194 lines; Task-2 additions (+1 field, +1 init, +4 builder) would land at 200/201 — cap-firing territory. Decision encoded: **split the summary half out** (ROSTER/sibling precedent: `collector_stages.rs` split from `collector.rs`).

- **New file `src/app/diagnostics/events_summary.rs`** (~55 lines) receives, moved verbatim: `CLOCK_DOMAIN_NOTE` (events.rs L10), `SummaryEvent` + `impl` (L142–163), `SummarySnapshot` (L165–185), and its needed imports (`serde::Serialize`, `crate::app::diagnostics::histogram::HistogramSummary`, and `epoch_seconds_now` from the sibling — change events.rs's `fn epoch_seconds_now()` to `pub(super) fn`). `CLOCK_DOMAIN_NOTE` stays private inside the new file (only used there). `pub(crate)` visibility of `SummaryEvent`/`SummarySnapshot` unchanged. No struct/derive/serde attr changes ⇒ serialization bytes identical.
- **`events.rs` afterwards:** TimingEvent/TimingDetails + QueueEvent + `epoch_seconds_now` + header note; ≈155 lines post-additions. Comfortable slack.
- Import-path updates (in-place, no logic edits):
  - `src/app/diagnostics/mod.rs` — add `mod events_summary;` (alphabetical, after `mod events;`).
  - `src/app/diagnostics/collector.rs:12` → split into `use crate::app::diagnostics::events::{TimingDetails, TimingEvent};` + new line `use crate::app::diagnostics::events_summary::SummaryEvent;` (195 → 196).
  - `src/app/diagnostics/diagnostics_test_fixtures.rs:14` → `pub(crate) use super::events_summary::SummarySnapshot;` (in place).
  - `src/app/diagnostics/state.rs:8` → `use crate::app::diagnostics::events_summary::SummarySnapshot;` (in place).

### Line budget table (post-fmt `cargo fmt --check`; counts exclude nothing, true file lines)

| File | Before | After (planned) | Slack | Change nature |
|---|---|---|---|---|
| `src/app/diagnostics/events.rs` | 194 | ~155 | ~45 | summary split + reason field/init/builder |
| `src/app/diagnostics/events_summary.rs` | — | ~55 | large | new (moved summary half) |
| `src/app/diagnostics/mod.rs` | 74 | 75 | large | +1 `mod` decl |
| `src/app/diagnostics/collector.rs` | 195 | 196 | 4 | +1 import line (in-place edit otherwise) |
| `src/app/diagnostics/collector_stages.rs` | 143 | 143 | 57 | signature + emit line, in-place |
| `src/app/diagnostics/state.rs` | ~159 | ~159 | ~40 | import path in place |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | 147 | 147 | 53 | import path in place |
| `src/app/run/chat_worker.rs` | 189 | 194 | 6 | +2 consts, +3 test-mod decl |
| `src/app/run/chat_stream_delivery.rs` | 198 | 198 | 2 | ALL in-place edits; import block stays 3 wrapped lines (verified fits ≤100 col) |
| `src/app/run/chat_reconnect_reason_tests.rs` (new) | — | ~95 | large | new test module |

**STOP protocol (pre-adjudicated):** if `cargo fmt --check` reflows `chat_stream_delivery.rs` or `collector.rs` past 200 lines, or the summary split requires touching any file NOT in the table above, implementer STOPS and reports to the caller instead of improvising. Do not delete blank lines/comments to make room (no-commented-code / preserve-rules).

---

## 4. 4.2 Implementation steps (exact, ordered)

Use single unchained commands (bash tool rule). No PowerShell cmdlets.

1. **Split (per §3):** create `events_summary.rs`; trim `events.rs`; update the three import sites + mod decl. Run `cargo fmt --check` then `cargo test` (must be 555/0 — split is behavior-neutral) before touching Task-2 code.
2. **events.rs QueueEvent:** add `reason: Option<&'a str>,` as the LAST struct field with `#[serde(skip_serializing_if = "Option::is_none")]`; add `reason: None,` as the LAST line of `with_depth`; add `with_reason` (snippet §2).
3. **collector_stages.rs:** extend `report_reconnect` per snippet §2.
4. **chat_worker.rs:** add the two consts (§2 table) below the `CHAT_ENDED_MESSAGE` declaration; extend delivery's existing import block to include `CHAT_CLEAN_CLOSE_REASON, CHAT_TRANSIENT_FAILURE_REASON` (alphabetical placement keeps the wrapped 3-line shape).
5. **chat_stream_delivery.rs (in-place edits only):** `handle_clean_close`/`handle_failure` call sites, `await_reestablishment` signature, `report_reconnect(delay, reason)` call.
6. **Tests (per §5):** create `chat_reconnect_reason_tests.rs`; declare it from `chat_worker.rs` ending block: `#[cfg(test)] #[path = "chat_reconnect_reason_tests.rs"] mod chat_reconnect_reason_tests;`.
7. Verify: `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test` (expect 558/0 OBSERVED).
8. Commits (sequential, per repo history conventions):
   - `fix: split summary events and skip-none the reconnect reason shape` (steps 1–3+5 production code; if the reviewer prefers, the split may ride its own commit: `refactor: split the summary half out of diagnostics events` FIRST — either order is acceptable, encoded default = split first as its own refactor commit, then the label commit `fix: label reconnect rows with clean-close vs transient failure reasons`).
   - `test: pin reconnect cause labels and reason-free other queue rows` (step 6).
9. 4.3 (review/simplify), 4.4 docs (§6), 4.5b audit, 4.6 close-out follow; no version bump; no release-artifact rebuild inside Task 2.

## 5. Test plan (offline; names, rosters, honest re-pins)

New module `src/app/run/chat_reconnect_reason_tests.rs`, mirroring `chat_delivery_diagnostics_tests.rs` imports (`ChatFakes`, `worker_plan`, `chat_channel`, `spawn_chat_worker`, `collector_with_output_file`, `yielded`, `response_*`, `text_item`, `LiveApiError`, plus `RESUME_TOKEN_FIXTURE` from the youtube fakes). Local helper `script_lookup(fakes)` and `fn network_failure()` duplicated from the sibling test file (test-module isolation; `pub(super)`-visibility rule respected — reuse the sibling's `super::` exports only if already exported, otherwise local fixture fns). File-based harness (`collector_with_output_file`) — NO sleeps anywhere (test slice is no-op).

1. **`clean_close_reconnect_rows_carry_the_clean_close_label`** — mirrors the operator-observed production pattern:
   - Roster: `script_lookup`; `fakes.script_token_batch_session()` (history discarded — no batch row); `fakes.stream_ledger.script_session(vec![Some(Ok(yielded(&response_with_token(vec![], RESUME_TOKEN_FIXTURE))))])` (one empty token batch ⇒ batch row `0`, then session exhaust → `None` → CleanClose ⇒ reconnect row); `fakes.script_graceful_end_session()` (terminal offline-ended batch).
   - Asserts: exactly **2** reconnect rows — each matching line ends with `"backoff_ms":1000,"reason":"clean_close"}` (key-order + value pin in one line-level assertion); snapshot `reconnect_count == 2`, `reconnect_backoff_total_ms == 2_000`; the batch rows in the file contain no `"reason"` (substring count over the two batch lines).
2. **`transient_failure_reconnect_rows_carry_the_transient_failure_label`** — mirrors the existing reconnect roster (`script_open_failure(network_failure())` then `script_graceful_end_session()`):
   - Asserts: exactly 1 reconnect row ending with `"backoff_ms":1000,"reason":"transient_failure"}`; `reconnect_count == 1`, total == 1_000.
3. **`other_queue_rows_carry_no_reason_key`** (byte-absence shape pin):
   - Roster: `script_lookup`; token batch session; one plain message session `response_plain(vec![text_item("m-1")])`; graceful end.
   - Asserts: file contains ≥1 `"event":"queue"` row AND `!content.contains("\"reason\"")` — no batch/enqueue row carries the key; (the timing/summary rows are already covered by other pins; this pin is the queue-row absence guarantee).

**Honest re-pins:** NONE required — `worker_reports_reconnect_with_backoff` (`chat_delivery_diagnostics_tests.rs:143`) and all budget/flow tests assert state counters which are unchanged; they must remain untouched and stay green (this is an explicit 4.3 review checkpoint, not an edit).

**Gate arithmetic:** baseline 555/0 (OBSERVED), +3 new tests ⇒ expected **558 tests / 0 failures**. OBSERVED gate output governs; any deviation = STOP, not silent rationalization.

## 6. 4.4 Docs duty (exact lines — one truth sentence)

- **`docs/production-run.md` line 336** ("Queue/backpressure diagnostics" bullet, inside the canonical §Pipeline Observability): after "(with the backoff wait)" append one sentence:
  "Each reconnect row also carries a non-secret `reason` label — `clean_close` for a server clean-close re-establishment and `transient_failure` for a re-establishment after a classified transient retrieval error — and no other diagnostic row kind carries a `reason` key at all."
  Commit: `docs: document reconnect row cause labels`. No duplicated procedures (the doc IS the canonical section — link requirement satisfied by staying in it).
- **`docs/configuration.md:90`**: UNCHANGED (flag semantics only; no duplication).
- **`docs/latency-test-map.md:43`**: UNCHANGED (summary counters unchanged by this task).
- 4.6 release/artifact rebuild per global plan Step 5 remains the caller's close-out duty (both platforms, `ls -l` cross-check).

## 7. Byte-stability ledger (what must serialize identically)

| Row kind | Status | Mechanism |
|---|---|---|
| Queue row — `batch` (`QueueEvent::batch`) | byte-identical | `reason: None` from skeleton + skip ⇒ key absent |
| Queue row — `enqueue` (`QueueEvent::enqueue`) | byte-identical | same |
| Queue row — `full` (`QueueEvent::full`) | byte-identical | same |
| Queue row — `reconnect` | Compatible APPEND: existing keys/order unchanged; new trailing `,"reason":"<label>"` only when a label is attached (every production call site attaches one) | skip + last-declared field |
| Timing rows — `dequeued`/`rendered`/`written` | byte-identical | TimingEvent untouched |
| Summary row | byte-identical | structs moved verbatim into `events_summary.rs`; `emits_summary_after_interval_and_not_earlier` + `rows_never_carry_message_text_or_author` + `queue_metrics_alone_does_not_emit_timing_rows_or_histograms` are the witnesses |
| Summary counters (`reconnect_count`, `reconnect_backoff_total_ms`) | byte-identical | `add_reconnect` untouched (1-arg) |
| Secrets | unchanged guarantee | labels are fixed const words; never `error.to_string()`; no raw provider internals ever reach a row value |

## 8. Verification commands (offline; single, unchained)

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (expect 558/0)
- `cargo build` (compile smoke)
- Optional targeted pins: `cargo test clean_close_reconnect_rows -- --nocapture` and `cargo test other_queue_rows -- --nocapture`
- Line-cap cross-check: `cargo fmt --check` plus reading each touched file's line count; `grep -n "#[allow(" src/app/diagnostics/* src/app/run/chat_worker.rs src/app/run/chat_stream_delivery.rs` must show no NEW `#[allow(` (the pre-existing `clippy::manual_clamp` in `chat_worker_helpers.rs:30` stays untouched).
- No `Cargo.toml` dependency edits; `git diff --stat` audit at close-out: only the files in §3's table + 2 new files + doc line.

## 9. Traceability — TODO Task-2 requirements → plan sections

| TODO requirement | Plan coverage |
|---|---|
| Reason on reconnect rows ONLY, from today's delivery classification, fixed lowercase-underscore labels | §2 (constants + threading), §1a/§1b (classification trace) |
| Exact label words chosen in Task-1's implementation plan slot (this 4.1b) | §2 table — `"clean_close"` / `"transient_failure"` final |
| Unknown/other causes still write the row; classifier (not format) enforces labels | §2 "Unknown/other-cause honesty" |
| Offline pins: clean-close case, transient-failure case, other-rows' unchanged shape | §5 tests 1–3 |
| Observability doc row description, one truth sentence, link-not-duplicate | §6 (production-run.md:336) |
| Gates pass / prior tests stay green or re-pinned truthfully | §5 honest re-pins (none needed), §8 commands |
| No secrets/provider internals in labels or rows | §2 rationale, §7 secrets row |
| ≤2 args, ≤200 lines, no deps, no allows, no sleeps | §3 budget + STOP, §2 signatures, §8 audit |
