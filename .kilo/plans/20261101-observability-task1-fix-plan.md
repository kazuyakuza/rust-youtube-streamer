# Task-1 Code Review Fix Plan — JUNIOR-EXECUTABLE

Branch: `feat/observability-game-handoff`  
Scope: Task-1 commits `3abb942..333a49b` only (exclude plan-tracking commit `c0fe375`).

## Findings Summary

Two issues require fixes. Both are local, behavior-preserving corrections inside the approved Task-1 architecture; no structural or scope decisions are needed.

1. **Capability gate leak (functional):** `pipeline_timing_events` is disabled but the collector still records the `message_render_ns` histogram and emits `"rendered"` / `"written"` per-message stage rows when another capability (e.g. `queue_metrics`) is enabled. This violates the plan semantics that `pipeline_timing_events` gates all per-message stage rows (stages 1–6) and the frame-pipeline histograms.
2. **Max-depth violation (coding rule):** `chat_stream_delivery.rs::send_stop_aware` nests an `if let` block three levels deep (`loop` → `match` arm → `if let`). Project rule limits nesting to two levels.

## Fix 1 — Gate timing rows/histograms to `pipeline_timing`

**File:** `src/app/diagnostics/collector.rs`

### Evidence

- Lines 80–86 (`note_render_completed`): `self.record_message_render(now);` is outside the `if let Some(start) = ...` block, so it runs even when `pipeline_timing` is disabled.
- Lines 94–104 (`note_write_completed`): the `last_dequeued.take()` and `"written"` event emission are outside the `if let Some(start) = ...` block, so the `"written"` row is emitted even when `pipeline_timing` is disabled.

### Change

Move `record_message_render` inside the `if let Some(start)` guard in `note_render_completed`, and move the `"written"` event emission inside the `if let Some(start)` guard in `note_write_completed`. `render_start_nanos` / `write_start_nanos` are only populated when `pipeline_timing` is enabled, so this becomes the single gate.

#### `note_render_completed` before

```rust
pub(crate) fn note_render_completed(&self) {
    let now = self.now();
    if let Some(start) = self.local.borrow_mut().render_start_nanos.take() {
        self.state_depth(|state| state.record_render(now.saturating_sub(start)));
    }
    self.record_message_render(now);
}
```

#### `note_render_completed` after

```rust
pub(crate) fn note_render_completed(&self) {
    let now = self.now();
    if let Some(start) = self.local.borrow_mut().render_start_nanos.take() {
        self.state_depth(|state| state.record_render(now.saturating_sub(start)));
        self.record_message_render(now);
    }
}
```

#### `note_write_completed` before

```rust
pub(crate) fn note_write_completed(&self) {
    let now = self.now();
    if let Some(start) = self.local.borrow_mut().write_start_nanos.take() {
        self.state_depth(|state| state.record_write(now.saturating_sub(start)));
    }
    let dequeued = self.local.borrow_mut().last_dequeued.take();
    if let Some(entry) = dequeued {
        let lag_ns = now.saturating_sub(entry.stamp_nanos);
        let details = TimingDetails::stage(entry.correlation.as_deref(), lag_ns);
        self.emit(&TimingEvent::new("written", details));
    }
}
```

#### `note_write_completed` after

```rust
pub(crate) fn note_write_completed(&self) {
    let now = self.now();
    if let Some(start) = self.local.borrow_mut().write_start_nanos.take() {
        self.state_depth(|state| state.record_write(now.saturating_sub(start)));
        let dequeued = self.local.borrow_mut().last_dequeued.take();
        if let Some(entry) = dequeued {
            let lag_ns = now.saturating_sub(entry.stamp_nanos);
            let details = TimingDetails::stage(entry.correlation.as_deref(), lag_ns);
            self.emit(&TimingEvent::new("written", details));
        }
    }
}
```

### Verification test to add

**File:** `src/app/diagnostics/collector_tests.rs`

Add helpers to `src/app/diagnostics/diagnostics_test_fixtures.rs` first:

```rust
pub(crate) fn diagnostics_with_capabilities(
    pipeline_timing: bool,
    queue_metrics: bool,
    periodic_summary: bool,
) -> DiagnosticsConfig {
    DiagnosticsConfig {
        pipeline_timing_events: Some(pipeline_timing),
        queue_metrics: Some(queue_metrics),
        periodic_summary: Some(periodic_summary),
        summary_interval_seconds: Some(1),
        output_file: None,
        max_file_bytes: Some(1_048_576),
        max_generations: Some(3),
    }
}

pub(crate) fn collector_for_config(diagnostics: &DiagnosticsConfig) -> TestCollector {
    test_collector(diagnostics, TEST_PACING_INTERVAL_NANOS)
}
```

Then add the test:

```rust
#[test]
fn queue_metrics_alone_does_not_emit_timing_rows_or_histograms() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut diagnostics = diagnostics_with_capabilities(false, true, false);
    diagnostics.output_file = Some(path.to_string_lossy().into_owned());
    let harness = collector_for_config(&diagnostics);
    let message = secret_message();

    harness.collector.report_batch_received(1);
    harness.collector.note_worker_receipt(Some("corr-1"));
    harness.advance(10);
    harness.collector.report_enqueue_success(&message);
    harness.advance(20);
    harness.collector.note_message_dequeued(&message);
    harness.collector.note_render_started();
    harness.advance(5);
    harness.collector.note_render_completed();
    harness.collector.note_write_started();
    harness.advance(7);
    harness.collector.note_write_completed();

    let content = std::fs::read_to_string(&path).expect("queue rows must be written");
    assert!(content.contains("\"event\":\"queue\""), "queue metrics still emit queue rows");
    assert!(!content.contains("\"stage\":\"dequeued\""));
    assert!(!content.contains("\"stage\":\"rendered\""));
    assert!(!content.contains("\"stage\":\"written\""));

    let snapshot = harness
        .state
        .lock()
        .expect("diagnostics state")
        .snapshot(harness.now());
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.message_render_ns.count, 0);
    assert_eq!(snapshot.render_ns.count, 0);
    assert_eq!(snapshot.write_ns.count, 0);
}
```

## Fix 2 — Reduce `send_stop_aware` nesting depth

**File:** `src/app/run/chat_stream_delivery.rs`

### Evidence

Lines 88–113: the `if let Some(diagnostics)` blocks inside the `Ok(())` and `Err(TrySendError::Full(...))` arms sit at nesting depth 3 (`loop` → match arm → `if let`).

### Change

Extract two small helper methods on `StreamWorker` that encapsulate the `Option<DiagnosticsCollector>` guard. Call them from the match arms so the match arms stay at depth 2.

#### Add helpers inside the `impl StreamWorker` block (after `send_stop_aware` or before it)

```rust
fn report_enqueue_success_if_enabled(&self, message: Option<&ChatMessage>) {
    if let (Some(diagnostics), Some(message)) = (&self.plan.diagnostics, message) {
        diagnostics.report_enqueue_success(message);
    }
}

fn report_enqueue_full_if_enabled(&self, message: &ChatMessage) {
    if let Some(diagnostics) = &self.plan.diagnostics {
        diagnostics.report_enqueue_full(message.message_id());
    }
}
```

#### `send_stop_aware` before

```rust
fn send_stop_aware(&self, mut message: ChatMessage) {
    loop {
        if self.plan.stop_flag.is_requested() {
            return;
        }
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
            Err(TrySendError::Full(returned)) => {
                if let Some(diagnostics) = &self.plan.diagnostics {
                    diagnostics.report_enqueue_full(returned.message_id());
                }
                message = returned;
                sleep_slice(CHAT_SLEEP_SLICE_MS);
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}
```

#### `send_stop_aware` after

```rust
fn send_stop_aware(&self, mut message: ChatMessage) {
    loop {
        if self.plan.stop_flag.is_requested() {
            return;
        }
        let report_message = self.plan.diagnostics.as_ref().map(|_| message.clone());
        match self.plan.sender.try_send(message) {
            Ok(()) => {
                self.report_enqueue_success_if_enabled(report_message.as_ref());
                return;
            }
            Err(TrySendError::Full(returned)) => {
                self.report_enqueue_full_if_enabled(&returned);
                message = returned;
                sleep_slice(CHAT_SLEEP_SLICE_MS);
            }
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}
```

### Notes

- The `report_message` clone is preserved because `message` is moved into `try_send`; the clone is only produced when a diagnostics collector is configured, keeping the disabled-mode cost at one `Option` read.
- Both helpers are single-section `if let` blocks and keep every function body under 50 lines.

## Verification Commands

After applying the fixes, run the same targeted tests used in the review:

```bash
alpine-vm_vm_status  # required first
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cd /rust-youtube-streamer && cargo test --locked diagnostics"
```

Expected: 47 passed (the new test plus existing 46), 0 failed.

## Junior-Implementer Constraints Reminder

- Do NOT change any config, docs, version, or plan files.
- Do NOT alter the public module layout, file split rulings, or the `Arc<Mutex<DiagnosticsState>>` mechanism.
- Do NOT add any new `#[allow(...)]` attributes.
- Keep all touched source files at or below 200 lines; `chat_stream_delivery.rs` is currently 178 lines, so the two small helpers fit comfortably.
