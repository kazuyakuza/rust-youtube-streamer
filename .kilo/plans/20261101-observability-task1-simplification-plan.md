# Task-1 Simplification Plan — Opt-In Diagnostics (Critical Workflow step 4.3)

- Scope: code introduced by `3abb942` (config), `e23f162` (diagnostics module), `333a49b` (run wiring) — i.e. `git diff 7829e3a..HEAD` minus plan-tracking commit `c0fe375`.
- Lens: genuine duplication, over-complex constructs, dead parameters, needless visibility, roster hygiene. Behavior-neutral ONLY.
- Companion to the binding implementation plan `.kilo/plans/20261101-observability-task1-plan.md`. This plan changes NO observable: no config schema names, no validation outcomes, no event JSON bytes, no aggregation semantics, no rotation behavior, no argv vectors, no Display contracts, no exit codes, no disabled-mode cost profile.
- Implementer: JUNIOR, 50% restriction. Execute steps in order. Each step is atomic and leaves the tree compiling (`cargo check` after every step is allowed; the full gate runs once at the end of all steps). If any file exceeds 200 physical lines after `cargo fmt`, STOP per the binding plan §10 protocol and report `wc -l` for every touched file.
- All projected counts are physical lines post-`cargo fmt`; verify with `wc -l` per step.

---

## Step 1 — Fold the four `QueueEvent` constructors over one skeleton (`src/app/diagnostics/events.rs`)

**Problem.** `enqueue`, `batch`, `full`, `reconnect` each repeat the identical 8-field literal with `event: "queue"`, `epoch_seconds: epoch_seconds_now()`, `queue_depth`, and four `None`s. `events.rs` is currently at exactly 200 lines (cap); this fold creates the headroom.

**Before** (lines 94–146, 53 lines): four full struct literals.

**After** (42 lines):

```rust
impl<'a> QueueEvent<'a> {
    /// One queue row skeleton; the constructors fill their kind-specific fields.
    fn with_depth(queue_depth: u64) -> Self {
        Self {
            event: "queue",
            epoch_seconds: epoch_seconds_now(),
            correlation: None,
            batch_size: None,
            queue_depth,
            backpressure: None,
            wait_ns: None,
            backoff_ms: None,
        }
    }

    pub(crate) fn enqueue(fields: QueueEnqueueFields<'a>, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.correlation = fields.correlation;
        event.backpressure = Some(fields.backpressured);
        event.wait_ns = Some(fields.wait_ns);
        event
    }

    pub(crate) fn batch(size: u64, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.batch_size = Some(size);
        event
    }

    pub(crate) fn full(correlation: Option<&'a str>, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.correlation = correlation;
        event.backpressure = Some(true);
        event
    }

    pub(crate) fn reconnect(backoff_ms: u64, queue_depth: u64) -> Self {
        let mut event = Self::with_depth(queue_depth);
        event.backoff_ms = Some(backoff_ms);
        event
    }
}
```

**Byte-stability note.** serde_json emits fields in struct-DECLARATION order, independent of construction order; every field value is identical to today.

Projected: `events.rs` 200 → ~189.

---

## Step 2 — Remove the redundant diagnostics re-guard in `validate_output_file` (`src/config/validation_diagnostics.rs`)

**Problem.** `validate_diagnostics` already unwraps `config.diagnostics` and early-returns; `validate_output_file` re-checks the same `Option` (dead guard, duplicate logic). Folding it inline also removes one indirection. (The `is_unusable_output_file` / `is_chat_log_collision` named predicates stay exactly as-is — plan §2.3 rules 26–27.)

**Before**: `validate_diagnostics` calls `validate_output_file(config, issues)` (line 22) and `validate_output_file` (lines 58–74) re-guards `config.diagnostics.as_ref()`.

**After** — delete `fn validate_output_file` entirely and make `validate_diagnostics` end with:

```rust
pub(super) fn validate_diagnostics(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let Some(diagnostics) = config.diagnostics.as_ref() else {
        return;
    };
    validate_positive_fields(diagnostics, issues);
    validate_generation_count(diagnostics, issues);
    if is_unusable_output_file(diagnostics) {
        issues.extend(blank_issue(
            "diagnostics.output_file",
            diagnostics.output_file().unwrap_or_default(),
        ));
    }
    if is_chat_log_collision(diagnostics, &config.chat.log_file) {
        issues.push(ConfigIssue::new(
            "diagnostics.output_file".to_string(),
            CHAT_LOG_COLLISION_PROBLEM.to_string(),
        ));
    }
}
```

**Issue-order note.** Positive-field issues, then generation-range, then blank output file, then collision — unchanged sequence, so `ConfigError::Validation` Display lines are byte-identical.

Projected: `validation_diagnostics.rs` 104 → ~97.

---

## Step 3 — Extract `pop_in_flight` in `take_dequeued` (`src/app/diagnostics/state.rs`)

**Problem.** The `uncorrelated_total = self.uncorrelated_total.saturating_add(1)` bump appears twice behind two early returns; the find-remove-record chain is one logical pop.

**Before** (lines 93–111): `let Some(correlation) = ... else { bump; return; }; let Some(index) = ... else { bump; return; }; if let Some(entry) = remove(index) { record }`.

**After**:

```rust
/// Pops the matching in-flight entry and records channel latency; an
/// absent or unmatched correlation is counted, never dropped silently.
pub(crate) fn take_dequeued(&mut self, correlation: Option<&str>, now_nanos: u64) {
    self.dequeued_total = self.dequeued_total.saturating_add(1);
    match self.pop_in_flight(correlation) {
        Some(entry) => self
            .message_channel_ns
            .record(now_nanos.saturating_sub(entry.enqueued_nanos)),
        None => self.uncorrelated_total = self.uncorrelated_total.saturating_add(1),
    }
}

fn pop_in_flight(&mut self, correlation: Option<&str>) -> Option<InFlightEntry> {
    let index = self
        .in_flight
        .iter()
        .position(|entry| Some(entry.correlation.as_str()) == correlation)?;
    self.in_flight.remove(index)
}
```

**Equivalence.** `None` correlation → the position predicate never matches → bump (same as today's first guard). Some-but-unmatched → bump. Matched → remove + record with the same stamp. `map_or(backpressured, ...)` in `add_enqueued` stays untouched (its None-entry path is load-bearing for uncorrelated retained counting — do NOT simplify it away).

Projected: `state.rs` 167 → ~165.

---

## Step 4 — Unify the poison-tolerant lock idiom with `with_state` (`src/app/diagnostics/collector.rs`)

**Problem.** The same "lock, tolerate poison, act" policy is spelled twice differently: `state_depth` uses `unwrap_or_else(PoisonError::into_inner)`, `emit_summary` uses `map_or_else`. Also four call sites (`record_render`, `record_write`, `record_frame_tick`, `record_message_render`) discard the queue depth that `state_depth`'s name promises.

**After** — one helper owns the lock; `state_depth` delegates; the four discard-sites switch to `with_state`; `emit_summary` collapses to two statements. Line 19 becomes `use super::collector_stages::{CollectorLocal, CollectorSettings, DequeuedMessage};` and line 65's inline `super::collector_stages::DequeuedMessage` path becomes `DequeuedMessage`. Line 142's magic `1_000_000_000` becomes `super::NANOS_PER_SECOND` (the module-constant duplication, same value, same module scope):

```rust
fn summary_is_due(&self, now: u64) -> bool {
    let interval = u64::from(self.settings.summary_interval_seconds) * super::NANOS_PER_SECOND;
    ...unchanged...
}

fn emit_summary(&self, now: u64) {
    let snapshot = self.with_state(|state| state.snapshot(now));
    self.emit(&SummaryEvent::new(self.settings.summary_interval_seconds, snapshot));
}

pub(super) fn with_state<T>(&self, action: impl FnOnce(&mut DiagnosticsState) -> T) -> T {
    let mut state = self
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    action(&mut state)
}

pub(super) fn state_depth(&self, action: impl FnOnce(&mut DiagnosticsState)) -> u64 {
    self.with_state(|state| {
        action(state);
        state.queue_depth()
    })
}
```

**Lock-discipline note.** Exactly one `lock()` acquisition per call before and after; the guard is held for precisely the same span (action + `queue_depth`/`snapshot` read). `summary_is_due` keeps its current mutate-only-when-due shape verbatim (a `replace(now)` on every call would delay summaries — do NOT "simplify" it).

Discard-site swaps (single-line each): `self.with_state(|state| state.record_render(now.saturating_sub(start)));`, same for `record_write`, `record_frame_tick`, `record_message_render`. The `let depth = self.state_depth(...)` sites (collector.rs line 61 and all four in `collector_stages.rs`) stay as-is.

Projected: `collector.rs` 196 → 196 (net zero: +8 `with_state`, −2 `state_depth`, −6 `emit_summary`).

---

## Step 5 — Delete the dead parameter of `note_worker_receipt` (`collector_stages.rs` + call sites)

**Problem.** `_correlation: Option<&str>` is never read (the receipt half stores only the stamp; `report_enqueue_success` takes the correlation from the message itself). Binding plan §3.8 pre-dated the final shape; the parameter is a leftover.

- `src/app/diagnostics/collector_stages.rs` line 71 → `pub(crate) fn note_worker_receipt(&self) {` (body unchanged).
- `src/app/run/chat_stream_delivery.rs` line 78 → `diagnostics.note_worker_receipt();`
- `src/app/diagnostics/collector_tests.rs` lines 30 and 104 → `harness.collector.note_worker_receipt();`

Projected: `collector_stages.rs` 118 → 118; `chat_stream_delivery.rs` 178 → 178; `collector_tests.rs` unchanged (counted in Step 8).

---

## Step 6 — Drop the full-message clone from `send_stop_aware` (`chat_stream_delivery.rs` + `collector_stages.rs`)

**Problem.** `report_enqueue_success` reads ONLY `message.message_id()`, yet the current shape clones the entire `ChatMessage` (author + text included) on every send attempt whenever diagnostics is present, and uses a two-section `if let (Some, Some)` pattern. Guarded by `is_some()`, so disabled mode allocates nothing today — preserve that exactly.

**Before** (lines 93–102):

```rust
let report_message = self.plan.diagnostics.as_ref().map(|_| message.clone());
match self.plan.sender.try_send(message) {
    Ok(()) => {
        if let (Some(diagnostics), Some(reported)) =
            (&self.plan.diagnostics, &report_message)
        {
            diagnostics.report_enqueue_success(reported);
        }
        return;
    }
```

**After**:

```rust
let reported_correlation = self
    .plan
    .diagnostics
    .as_ref()
    .and_then(|_| message.message_id().map(str::to_owned));
match self.plan.sender.try_send(message) {
    Ok(()) => {
        if let Some(diagnostics) = &self.plan.diagnostics {
            diagnostics.report_enqueue_success(reported_correlation.as_deref());
        }
        return;
    }
```

(Full arm and `Disconnected` arm untouched; `sleep_slice`/loop shape preserved.)

`src/app/diagnostics/collector_stages.rs`:

```rust
pub(crate) fn report_enqueue_success(&self, correlation: Option<&str>) {
    let now = self.now();
    let (received_nanos, backpressured) = self.take_receipt();
    let entry = correlation.map(|id| InFlightEntry {
        correlation: id.to_string(),
        received_nanos,
        enqueued_nanos: now,
        backpressured,
    });
    ...unchanged (depth/emit half)...
}
```

Delete line 86 (`let correlation = message.message_id();`) and line 11 (`use crate::chat::ChatMessage;` — no other user in this file).

**Equivalence.** Diagnostics `None` → no capture, no clone, no call (today: `map` short-circuits, same). Diagnostics `Some` + id `Some` → `String` of the id, passed back as the same `Option<&str>` the collector used to derive via `message.message_id()`. Diagnostics `Some` + id `None` → `report_enqueue_success(None)`: identical uncorrelated enqueue accounting + identical QueueEvent row. The Full-retry path no longer re-clones the message per retry iteration.

Tests updated in `collector_tests.rs` lines 32/105 → `harness.collector.report_enqueue_success(message.message_id());`

Projected: `chat_stream_delivery.rs` 178 → ~180; `collector_stages.rs` (with Step 5) → ~116.

---

## Step 7 — Move the remaining `Option`-guard hooks into `runner_diagnostics.rs` (`runner.rs` + `runner_diagnostics.rs`)

**Problem.** `runner.rs` repeats the 3-line `if let Some(diagnostics) = &self.diagnostics { ... }` guard five times; the child module already exists precisely for guarded hooks (`note_frame_tick`, `maybe_emit_summary`). The other five hooks belong there too, shrinking the core loop file and making the module docs true ("The diagnostics hook methods live in the `runner_diagnostics` child module").

**After in `runner.rs`** (call sites become single lines):
- line 110–112 → `self.note_message_dequeued(&message);`
- line 124–126 → `self.note_render_started();`
- line 131–133 → `self.note_render_completed();`
- line 147–149 → `self.note_write_started();`
- line 153–155 → `self.note_write_completed();`

**After in `runner_diagnostics.rs`** — add, beside the two existing hooks (import `use crate::chat::ChatMessage;`):

```rust
impl<S: FrameSink> FramePipeline<S> {
    /// Records one dequeued message into the shared aggregate.
    pub(super) fn note_message_dequeued(&self, message: &ChatMessage) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_message_dequeued(message);
        }
    }

    pub(super) fn note_render_started(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_render_started();
        }
    }

    pub(super) fn note_render_completed(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_render_completed();
        }
    }

    pub(super) fn note_write_started(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_write_started();
        }
    }

    pub(super) fn note_write_completed(&self) {
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.note_write_completed();
        }
    }
    ...existing two hooks unchanged...
}
```

Update the module doc first sentence "Both hooks are guarded by the `Option` read that IS the entire disabled-mode cost." → "Every hook is guarded by the `Option` read that IS the entire disabled-mode cost." Call ORDER is preserved exactly (started before render, completed after, write started before `write_frame`, completed after).

Projected: `runner.rs` 163 → ~153; `runner_diagnostics.rs` 28 → ~64 (test registration stays at the file footer).

---

## Step 8 — Roster hygiene: `TestCollector::snapshot()` + `diagnostics_of` + `expect_rejected`

**8a. `src/app/diagnostics/diagnostics_test_fixtures.rs`** — the 5-line `harness.state.lock().expect("diagnostics state").snapshot(harness.now())` chain is duplicated 6× across three rosters. Add:

```rust
use super::events::SummarySnapshot;

impl TestCollector {
    ...
    pub(crate) fn snapshot(&self) -> SummarySnapshot {
        self.state
            .lock()
            .expect("diagnostics state")
            .snapshot(self.now())
    }
}
```

Narrow with it: `state` and `clock` fields → private; `now()` → private (no remaining external callers after the replacements below); `TEST_PACING_INTERVAL_NANOS`, `FixedMonotonicNanos` + its `new` → private (all uses are in this file). `collector` and `advance` stay `pub(crate)` (external test use).

**8b.** Replace every chain in:
- `collector_tests.rs` lines 84–88 and 107–111 → `let snapshot = harness.snapshot();`
- `chat_delivery_diagnostics_tests.rs` lines 51–55, 96–100, 136–140 → same one-liner; `wait_for_full_retry` body line 107–112 → `let retries = harness.snapshot().full_retry_total;` (loop/yield/return unchanged)
- `runner_diagnostics_tests.rs` lines 52–56 → same one-liner

**8c. `collector_tests.rs`** — the `config.diagnostics.as_mut().expect(...)` chain is triplicated (lines 13–17, 60–64, 71–76). Add, mirroring the EXISTING identically-named helper in `validation_diagnostics_tests.rs` (same concept, deliberately same name; the two rosters live in different module trees and cannot share one definition):

```rust
use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::AppConfig;

fn diagnostics_of(config: &mut AppConfig) -> &mut DiagnosticsConfig {
    config
        .diagnostics
        .as_mut()
        .expect("the fixture carries a diagnostics section")
}
```

Call sites: `diagnostics_of(&mut config).pipeline_timing_events = Some(false);` (in `diagnostics_config()`), `diagnostics_of(&mut config).output_file = Some(path.to_string_lossy().into_owned());`, `diagnostics_of(&mut config).pipeline_timing_events = Some(true);`. Keep `fn diagnostics_config() -> AppConfig` return type, now short.

**8d. `diagnostics_parse_tests.rs`** — the tempdir→write→expect_load_error→match tail is duplicated by `unknown_diagnostics_field_rejected` and `wrong_type_flag_rejected`. Add:

```rust
fn expect_rejected(value: &serde_json::Value) {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    assert!(matches!(
        expect_load_error(&path),
        ConfigError::Deserialization { .. }
    ));
}
```

Both tests end with `expect_rejected(&value);` (their value-mutation halves unchanged).

Projected: `diagnostics_test_fixtures.rs` 91 → ~98; `collector_tests.rs` 149 → ~139 (incl. Steps 5–7 call updates); `chat_delivery_diagnostics_tests.rs` 155 → ~138; `runner_diagnostics_tests.rs` 96 → ~92; `diagnostics_parse_tests.rs` 119 → ~122.

---

## Step 9 — Prefer-private visibility narrowing (word edits only; compiler-verified)

Every item below is used ONLY inside `src/app/diagnostics/` (verified by crate-wide grep), so `pub(crate)` is needless exposure. Change exactly these, nothing else:

| File | Item | From | To |
|---|---|---|---|
| `events.rs` | `TimingEvent`, `TimingDetails`, `QueueEvent`, `QueueEnqueueFields` structs | `pub(crate)` | `pub(super)` |
| `events.rs` | all fields of `TimingDetails` and the four event structs | `pub(crate)` | private (serde derives serialize private fields fine; constructors already exist; `QueueEnqueueFields` fields → `pub(super)` because `collector_stages.rs` fills them via struct literal) |
| `events.rs` | constructor methods (`new`, `enqueue`, `batch`, `full`, `reconnect`, `dequeued`, `stage`, `with_queue_depth`) | `pub(crate)` | `pub(super)` |
| `events.rs` | `CLOCK_DOMAIN_NOTE` const, `epoch_seconds_now` fn | `pub(crate)` | private |
| `histogram.rs` | `DurationHistogram` + `new`/`record`/`snapshot`, `HISTOGRAM_BOUNDARIES_NS`, `HISTOGRAM_BUCKET_COUNT` | `pub(crate)` | `pub(super)` |
| `histogram.rs` | `HistogramSummary`, `HistogramBuckets` (keep `pub(crate)` — external test rosters read summary fields; narrowing the alias below a `pub(crate)` field would trip `private_interfaces`) | — | unchanged |
| `state.rs` | `IN_FLIGHT_CAP`, `InFlightEntry` (+fields → `pub(super)`), `DiagnosticsState` + all `pub(crate)` methods | `pub(crate)` | `pub(super)` |
| `state.rs` | `snapshot()` returns the `pub(crate)` `SummarySnapshot` | — | unchanged (wider return is legal) |
| `collector.rs` | `MonotonicNanos` trait, `InstantMonotonicNanos` | `pub(crate)` | `pub(super)` |
| `writer.rs` | `WriterSettings` (+its three fields), `DiagnosticsFileWriter`, its `new` | `pub(crate)` | `pub(super)` (writer_tests reaches it via `use super::writer::*` as a diagnostics-level sibling) |
| `diagnostics_test_fixtures.rs` | `TestCollector` itself, `enabled_collector_for_tests`, `collector_with_output_file` | keep `pub(crate)` — used by `chat_delivery_diagnostics_tests` / `runner_diagnostics_tests` outside the module | unchanged |

Keep `pub(crate)` (external consumers verified by grep): `DiagnosticsCollector` and its per-stage `pub(crate)` methods (runner + delivery call sites), `SummarySnapshot`/`HistogramSummary` + fields (external test field reads), `diagnostics_test_fixtures` module itself (registered `pub(crate)` in `mod.rs`).

Projected: no line-count change in any file (word edits).

---

## Projected post-`cargo fmt` line counts (cap 200)

| File | Now | Projected |
|---|---|---|
| `src/app/diagnostics/events.rs` | 200 | ~189 |
| `src/app/diagnostics/collector.rs` | 196 | ~196 |
| `src/app/diagnostics/collector_stages.rs` | 118 | ~116 |
| `src/app/diagnostics/state.rs` | 167 | ~165 |
| `src/app/diagnostics/histogram.rs` | 119 | 119 |
| `src/app/diagnostics/writer.rs` | 130 | 130 |
| `src/app/diagnostics/mod.rs` | 70 | 70 |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | 91 | ~98 |
| `src/app/diagnostics/collector_tests.rs` | 149 | ~139 |
| `src/app/diagnostics/state_tests.rs` | 88 | 88 |
| `src/app/diagnostics/histogram_tests.rs` | 55 | 55 |
| `src/app/diagnostics/writer_tests.rs` | 81 | 81 |
| `src/config/validation_diagnostics.rs` | 104 | ~97 |
| `src/config/diagnostics_parse_tests.rs` | 119 | ~122 |
| `src/app/pipeline/runner.rs` | 163 | ~153 |
| `src/app/pipeline/runner_diagnostics.rs` | 28 | ~64 |
| `src/app/run/chat_stream_delivery.rs` | 178 | ~180 |
| `src/app/run/chat_delivery_diagnostics_tests.rs` | 155 | ~138 |
| `src/app/pipeline/runner_diagnostics_tests.rs` | 96 | ~92 |

All ≤ 200 with headroom; the two tight files (`collector.rs`, `chat_stream_delivery.rs`) change by ≤ +4 lines — re-verify with `wc -l` after `cargo fmt`, STOP protocol if any exceeds.

Untouched (cap-TIGHT files verified clean): `src/app/run/streaming.rs` (198), `src/app/run/chat_worker.rs` (189), `src/app/run/mod.rs` (185), `src/config/loader_tests.rs` (138), `src/config/test_fixtures.rs` (140), `src/config/model.rs`, `src/config/validation.rs` (177), `src/chat/model.rs`, `src/app/run/stop_pair.rs`, `src/app/run/chat_channel.rs`, `src/app/run/chat_worker_helpers.rs`, `src/app/run/seams.rs`, `src/app/pipeline/runner_prepare.rs`, `src/config/diagnostics_config.rs` — no simplification candidates survived the reject tests below.

## Observables-preserved ledger

| Observable | Why untouched |
|---|---|
| `diagnostics.*` JSON schema + serde names | Step 9 changes Rust visibility words only; derive output unchanged |
| Validation outcomes + issue order + Display lines | Step 2 preserves predicate sequence and issue texts verbatim |
| Event JSON rows (bytes, field order, values) | Steps 1/5/6 produce the identical field values; serde order = declaration order |
| Aggregation semantics (counters, histograms, retained-vs-enqueued, eviction, high-water, oldest-age) | Steps 3/5/6 pass the same values through shorter code paths; `add_enqueued`'s `map_or` deliberately NOT folded (load-bearing for uncorrelated retained counting) |
| Rotation behavior + disk bound | `writer.rs` line content untouched (Step 9 words only) |
| argv vectors, `ChatMessage` Display, exit codes, shutdown order | `runner.rs` Step 7 preserves call order; no sink/args code touched |
| Disabled-mode zero-alloc/zero-clock/zero-I/O contract (plan §4) | Step 6 keeps the capture behind `diagnostics.is_some()`-equivalent; Step 4 keeps one lock per call with the same span; no unconditional clock/alloc introduced anywhere |
| Lock discipline (`Arc<Mutex<DiagnosticsState>>`) | Step 4: same single acquisition, same guard span per call site |
| Test semantics | No assertion values changed; no test RENAMED (all names stay truthful: `note_worker_receipt` still stamps the receipt; `report_enqueue_success` still reports the enqueue) |

## Rejected candidates (inspected, NOT in this plan)

1. **`writer.rs` double `open_if_needed`** (lines 56/58) — collapsing to open-→rotate→open-only-when-rotated would remove one open retry on the error-swallowing failure path (different number of `fs` attempts when the first open fails). Not byte-observably different in success runs, but it is a (tiny) behavior change on the tolerated-failure path → DROPPED.
2. **Early `if !self.settings.pipeline_timing { return; }` in `note_render_completed`/`note_write_completed`** (symmetry with the started-side guards; saves one clock read + two borrows/frame when timing off but queue/summary on). Provably a no-op path, but the +6 lines push `collector.rs` over the 200 cap after Step 4 → DROPPED (see future TODO).
3. **Data-driven loop over the three positive-value rules** (`validation_diagnostics.rs`) — would blur the plan-mandated 1:1 predicate↔rule naming (rules 22–24) and diverges from the existing repeated-block style in `validation.rs` → DROPPED.
4. **Bulk `pub(crate)`→`pub(super)` on `SummarySnapshot`/`HistogramSummary`/collector stage methods** — genuine external consumers (pipeline/run test rosters) verified by grep → DROPPED.
5. **Merging the three `epoch_seconds_now()` implementations** (new `events.rs` + pre-existing `run/broadcast.rs` + `streaming/video_output.rs`) — a cross-module shared-utils extraction; touching pre-Task-1 files is out of scope → future TODO.
6. **`rotate_if_needed` thin wrapper / `from_config` if-else rewrite / `state_depth`-discards without helper** — cosmetic, zero real gain, or superseded by Step 4 → DROPPED.

## Future-TODO candidates (borderline-architectural; caller decision, NOT for this step)

- If further `collector.rs` growth is ever needed (e.g. the rejected timing-guards #2), split the frame-loop half (`note_render_*`, `note_write_*`, `note_frame_tick`, summary timer) into a `collector_frames.rs` child module — new-file decision, architect territory.
- Single shared `epoch_seconds_now()` home (rejected #5).
- Cross-tree shared test helper for the `diagnostics_of` name pattern (Step 8c keeps two same-named local helpers because `config::` and `app::diagnostics::` test trees cannot share without a wider test-utils seam).

## Execution order and self-checks for the implementer

1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9. After each step: `cargo check` (test code included: `cargo check --all-targets`). After Step 9 and the whole plan: `cargo fmt` then `wc -l` every file in the table; any count > 200 → STOP and report. Then the normal dev-checks gate (step 4.5 owns it; do not run it here).
