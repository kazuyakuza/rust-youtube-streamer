# Task-2 Implementation Plan — Reproducible Latency, Throughput, and Backpressure Test Maps

- TODO: `.agent/todos/20261031/20261031-todo-1.md` — Task 2 (pp. 66–101) binding; global constraints pp. 13–23 bind too.
- Global plan: `.kilo/plans/20261101-observability-game-handoff.md` (Global + Task-2 pre-analysis sections) — verified against the real tree here.
- Task 1 (shipped): `.kilo/plans/20261101-observability-task1-plan.md`; the WORKING implementation lives under `src/app/diagnostics/` (this plan reads code, not only its plan).
- Branch `feat/observability-game-handoff` (HEAD `c65eb34`, v0.14.0; pre-Task-1 gate OBSERVED 497/0, post-Task-1 gate OBSERVED 540/0). DO NOT bump versions. DO NOT create/switch branches.
- Front-end determination: **NOT front-end** (Rust CLI/service; no 4.1a/4.5a sub-steps). Repeated verbatim as required.
- Implementer: JUNIOR developer under 50% restriction. Every structural choice below is FINAL and encoded; do not re-decide anything. Where a choice existed, exactly one is recorded with the rationale.
- Task 2 changes ZERO production behavior. The only edits to existing files are `#[cfg(test)]`-only registration/fixture lines (§7). Docs land at 4.4 (§4–§5); the docs commit is OUT of 4.2 (§6).

---

## 1. Pre-Analysis — Verified Current-State Facts (read from the real files, this branch)

### 1.1 Which Task-1 diagnostics artifacts provide which required metric (requirement E)

All verified in `src/app/diagnostics/` (`state.rs`, `collector.rs`, `collector_stages.rs`, `events.rs`, `histogram.rs`, `diagnostics_test_fixtures.rs`):

| Required metric | Diagnostics field (on `SummarySnapshot`, returned by `DiagnosticsState::snapshot` and exposed by `TestCollector::snapshot()`) | Meaning (verified in code) |
|---|---|---|
| Throughput (received) | `received_total` | `state.add_received(batch_size)` at `chat_stream_delivery.rs::process_batch` (live path) |
| Throughput (enqueued) | `enqueued_total` | `report_enqueue_success` → `add_enqueued` |
| Throughput (consumed/rendered) | `dequeued_total` | `take_dequeued` at the frame loop's one-per-frame `accept_next_message` |
| Queue depth | `queue_depth_estimate` (`enqueued_total − dequeued_total`) | honest counter-derived estimate; the channel exposes no live depth |
| Queue high-water mark | `in_flight_high_water` | max of in-flight map length on every enqueue |
| Queue age (oldest queued) | `oldest_in_flight_age_ns = Option<u64>` (`now − front.received_nanos`) | `Some` exactly when the in-flight map is non-empty at snapshot |
| Enqueue wait (receipt→enqueue) | queue row `wait_ns` (`QueueEvent::enqueue`) — per event, not in the snapshot | `report_enqueue_success`'s `now − last_receipt_nanos` |
| Enqueue→dequeue latency | histogram `message_channel_ns` (dequeue stamp − `InFlightEntry.enqueued_nanos`) | per message, popped by correlation |
| Dequeue→render latency | histogram `message_render_ns` (render-complete now − last dequeue stamp) | `collector.rs::note_render_completed → record_message_render` |
| Render duration | histogram `render_ns` (`note_render_started`/`note_render_completed` around `renderer.render`) | real loop anchors in `runner.rs::render_and_write_frame` |
| FFmpeg-write duration | histogram `write_ns` (`note_write_started`/`note_write_completed` around `sink.write_frame`) | real loop anchors in `runner.rs::write_frame_bytes` |
| Full-channel retries / worker-retained | `full_retry_total` / `worker_retained_total` | pinned by `add_full_retry` (`Err(TrySendError::Full)` arm) and the `backpressured` flag |
| Reconnect/backoff | `reconnect_count`, `reconnect_backoff_total_ms` | `await_reestablishment` path (NOT used by this harness; covered by existing rosters) |
| Frame interval / jitter | `frame_interval_ns`, `frame_jitter_ns` | `note_frame_tick` in the runner loop |
| Snapshot shape | `events.rs::SummarySnapshot` derives `Clone + PartialEq`; `HistogramSummary { count, min_ns: Option<u64>, max_ns, mean_ns, buckets: [(lower, Option<u64>, count); 9] }` | assertable directly from tests |

`TestCollector` (`diagnostics_test_fixtures.rs`, `#[cfg(test)]`): `collector: DiagnosticsCollector` ((Clone) — attach into any plan), `advance(nanos)` (pub(crate), mutates a shared `Arc<AtomicU64>` fake `MonotonicNanos`), `snapshot()` (reads state at the current fake clock). `enabled_collector_for_tests()` builds all-three-capabilities with a FIXED pacing interval of `40` ns; the fixed clock starts at `0` and only moves via `advance`.

### 1.2 The REAL seams available for scripted producers (evidence: file :: function, verified)

| Seam | Access from a `#[cfg(test)]` roster registered inside `chat_stream_delivery.rs` | Notes |
|---|---|---|
| Real bounded channel | `chat_worker.rs` re-exports `pub(super) use chat_channel::chat_channel;` → `ChatChannel { sender: SyncSender<ChatMessage>, receiver, terminal }`; capacity `CHAT_CHANNEL_CAPACITY = 64` (`chat_worker.rs`, `pub(super)` const, the ONLY source of the number 64) | `#[cfg(test)] pub(super) use chat_channel::ChatChannel;` exists |
| Real delivery path | `chat_stream_delivery.rs` :: `StreamWorker::process_batch` (`pub(super)`), `deliver`/`send_stop_aware` (private, but a DESCENDANT module of `chat_stream_delivery` sees private parent items) — dedup window (`ChatDedupWindow`, ≤512 ids, `chat_stream_worker.rs::StreamWorker::new`), C-1 conversion `into_chat_message`, `note_worker_receipt`, `try_send`, full-retry arm | `StreamWorker::new(plan, live_chat_id)` is `pub(super)` (`chat_stream_worker.rs`) |
| Real history-window open | first `process_batch(yielded(&response_with_token(vec![], RESUME_TOKEN_FIXTURE)))` clears `history_pending` exactly like `ChatFakes::script_token_batch_session()` (zero messages, opens the live window) | no private fn needed |
| Real consumer | `run/source.rs` :: `ChatSource` (`pub(super)`), `MessageSource::next_message()` — non-blocking `try_recv`, ONE message per call | the frame loop calls it exactly once per frame (`runner.rs::accept_next_message`) |
| Real one-message-per-frame loop | `pipeline/runner.rs` :: `FramePipeline::run` loop: stop-check → `accept_next_message` (≤1 message/frame) → `render_and_write_frame` → `PacingSchedule::next_gap` → `note_frame_tick`/`maybe_emit_summary` → `clock.sleep_nanos` | `FramePipeline` + `PipelineParts` are `pub(crate)` `#[cfg(test)]` re-exports from `pipeline/mod.rs` |
| Real pacing from config | `pipeline/pacing.rs` :: `PipelinePacing::from_config(&config.video)` (`pub(crate)`), `interval_nanos()` | FPS is read from config — never hardcoded (requirement D) |
| Deterministic loop termination | `pipeline/test_helpers.rs` :: `RecordingSink::stop_after_writes(threshold, stop_flag)` — flips the flag at the Nth write; re-exported `pub(crate)` `#[cfg(test)]` from `pipeline/mod.rs` | the sink fake also records every written frame (`SinkEvents.writes`) — the FrameSink boundary is REAL code with a recorded write |
| Real renderer double | `StubRenderer` (records every `RenderInput.visible_text`, renders a 2×2 `Frame` = `TEST_FRAME_BYTES` 12 bytes) | frame size gate needs `PipelinePacing::raw(interval, TEST_FRAME_BYTES)` … FINAL simplification below |
| Offline worker-side doubles | `run/runtime_chat_fakes.rs` :: `ChatFakes`/`worker_plan(&fakes, &channel)` (builds a complete `ChatWorkerPlan` with `diagnostics: None`) + `youtube/stream_test_fixtures.rs` builders `text_item`/`response_plain`/`response_with_token`/`response_offline`/`yielded` (fixture-shaped strings only — `author-fixture`/`text-fixture`/`live-chat-id-fixture`/`token-fixture`; never real-shaped, never reaching Google) | already used by `chat_worker_burst_tests.rs` and `chat_delivery_diagnostics_tests.rs` |
| Deterministic diagnostics collector | `diagnostics_test_fixtures.rs` :: `TestCollector` (+ `enabled_collector_for_tests()`); the collector's fake clock is advanced ONLY by `advance()` | zero real sleeps, deterministic stamps |

**Forbidden by scope:** no spawned worker threads in NEW rosters for latency scenarios (the existing cross-thread burst/retained pins `chat_worker_burst_tests.rs::worker_full_channel_retains_the_burst_without_loss_or_reorder` and `chat_delivery_diagnostics_tests.rs::worker_marks_backpressured_on_full_channel` ALREADY pin the real threaded `send_stop_aware` Full arm — do NOT duplicate them); no network, no Google contact, no real FFmpeg binary, no `tokio` test runtime (existing `stream_test_block_on` not needed here).

### 1.3 Existing fixtures / test-family inventory (verified)

- `pipeline/test_helpers.rs`: `FakeClock` (elapsed advances ONLY via `sleep_nanos`; records sleeps), `StubRenderer`, `RecordingSink` + `SinkEvents`, `TEST_INTERVAL_NANOS=40`, `TEST_FRAME_BYTES=12`.
- `pipeline/source_helpers.rs`: `ScriptedSource` (pipeline-local; NOT used by this harness — the harness needs the real `ChatSource` on the real channel).
- `run/runtime_chat_fakes.rs`: `ChatFakes`, `worker_plan`, `run_worker`, `LIVE_CHAT_ID_FIXTURE`.
- `youtube/stream_test_fixtures.rs` (+ `stream_run_fixtures.rs` `RunChatStreamLedger`): the offline scripted stream session doubles; the dedup-window bound is verified (`live_chat_dedup.rs`, `CHAT_DEDUP_WINDOW = 512`, "last 512" eviction enforced by the window itself).
- `diagnostics_test_fixtures.rs` (119 lines): `TestCollector`, `FixedMonotonicNanos` (private), `enabled_collector_for_tests`, `collector_with_output_file`, `diagnostics_with_capabilities` (pub(super) — NOT visible outside the diagnostics subtree; a small `pub(crate)` wrapper is the sanctioned test-only addition, §2.4).
- Pipeline end-to-end diagnostics pins already exist (`runner_diagnostics_tests.rs`, 2 tests) — they cover enabled/disabled mode for the REAL loop but NOT rate/backpressure scenarios; no overlap with this task.

### 1.4 Config fixture facts (requirement D inputs)

- `src/config/test_fixtures.rs::valid_config()`: `video { width: 1920, height: 1080, fps: 30 }`; `diagnostics: Some(DiagnosticsConfig { …all false…, output_file: Some("logs/diagnostics.log") })`; `chat.log_file` overridden per test to a `tempfile::TempDir` (the `temp_store()` pattern from `runner_diagnostics_tests.rs`; real `ChatStore` visible capacity = floor((1080−20−20)/40) = 26 from the real geometry).
- Pacing interval derived from `valid_config().video.fps` via `PipelinePacing::from_config` = `1_000_000_000 / 30 = 33_333_333` ns — asserted as an EXPRESSION of the fixture field, never a magic literal.
- Channel capacity read ONLY from `crate::app::run::chat_worker::CHAT_CHANNEL_CAPACITY`.

### 1.5 Line counts observed on the current tree (VM `wc -l`, 2026-11-01)

`chat_stream_delivery.rs` 188 · `chat_worker.rs` 189 · `run/mod.rs` 185 · `streaming.rs` 198 · `runner.rs` 153 · `diagnostics_test_fixtures.rs` 119 · `runtime_chat_fakes.rs` 127. Gate baseline OBSERVED 540 / 0 (README line 47, re-recorded by the diagnostics pass).

---

## 2. FINAL Harness Design

### 2.1 Module placement and roster registration (FINAL)

New files (all under `src/app/run/`, all `#[cfg(test)]`-only):

| File | Registered FROM | Content |
|---|---|---|
| `latency_load_fixtures.rs` | `chat_stream_delivery.rs` footer: `#[cfg(test)] #[path = "latency_load_fixtures.rs"] mod latency_load_fixtures;` | scenario spec types (§2.2), `PacedQueueProducerSource` re-export, `run_scenario`, accounting helper. NO `#[test]`. |
| `latency_load_source.rs` | `latency_load_fixtures.rs` footer: `#[path = "latency_load_source.rs"] mod latency_load_source;` | the `MessageSource` impl driving the real worker `process_batch` per frame. NO `#[test]`. |
| `latency_load_tests.rs` | `latency_load_fixtures.rs` footer: `#[cfg(test)] #[path = "latency_load_tests.rs"] mod latency_load_tests;` | the 6 rate/burst/backpressure scenario tests (§3). |
| `latency_load_loss_tests.rs` | `latency_load_fixtures.rs` footer: `#[cfg(test)] #[path = "latency_load_loss_tests.rs"] mod latency_load_loss_tests;` | loss invariant + shutdown + pure pins (§3). |

Rationale for the attachment point (choice recorded, not decided by the implementer): `chat_stream_delivery.rs` is where the worker-produced-message/queue seam lives and the existing worker-side diagnostics roster is already attached there (~191 lines after the +4 registration block, comfortable under the cap). Run-subtree visibility brings `ChatSource`, `chat_channel()`, `CHAT_CHANNEL_CAPACITY`, `StreamWorker`/`process_batch`, and the `youtube` fixture builders into reach. The pub(crate) `#[cfg(test)]` re-exports of `pipeline/mod.rs` (`FramePipeline`, `PipelineParts`, `PipelinePacing`, `MessageSource`, `FakeClock`, `RecordingSink`, `StubRenderer`, `TEST_FRAME_BYTES`, `TEST_INTERVAL_NANOS`, `StopFlag`, `ChatSourceError`) are crate-visible — verified for each name. Rejected alternatives: registering from `run/mod.rs` (loses `process_batch` proximity and startles a 185-line file), or driving raw `sender.try_send` from outside (skips the REAL conversion/receipt/dedup path — narrower seam, less realism).

### 2.2 Types (FINAL shapes; write exactly these, no extra public surface)

```rust
// latency_load_fixtures.rs
/// Rational messages-per-frame: NO floating point anywhere (flakiness ban).
pub(super) struct RateProfile {
    pub(super) numerator: u64,   // messages credited per frame step
    pub(super) denominator: u64, // frames needed to fully release one numerator — credit-only arithmetic
}
```
`RateProfile::advance(&self, credit: &mut u64) -> u64` — adds `numerator` into the shared credit, returns the integer release count, keeps the remainder (`credit` now holds `credit + numerator − released × denominator` with release computed by integer division). Sequence for `{2,3}`: frames → `0,1,1,0,1,1` (pinned by a pure test).

```rust
pub(super) struct RatePhase {
    pub(super) frames: u64,   // exact duration of the phase in frame steps
    pub(super) rate: RateProfile,
}

pub(super) struct LatencyScenario {
    pub(super) label: &'static str,
    pub(super) phases: Vec<RatePhase>,
}
```
`LatencyScenario::total_frames()` = Σ `phase.frames` (deterministic; also the `RecordingSink::stop_after_writes` threshold and the frame budget). A "burst" IS a one-frame `RatePhase` with a large numerator (e.g. `{64,1}` = a burst of `CHAT_CHANNEL_CAPACITY` messages in one frame) — there is no separate burst type (one injection mechanism only).

```rust
pub(super) struct ScenarioOutcome {
    pub(super) total_frames: u64,
    pub(super) pacing_interval_nanos: u64,
    pub(super) snapshot: SummarySnapshot,     // diagnostics aggregate at stop
    pub(super) leftover: Vec<ChatMessage>,    // drained from the channel AFTER stop (bounded ≤ CAPACITY)
    pub(super) kept_temp: tempfile::TempDir,  // keeps the chat-log temp dir alive for the caller's assertions
}
```

```rust
// latency_load_source.rs
pub(super) struct PacedQueueProducerSource {
    delegate: ChatSource,          // the REAL consumer over the REAL bounded channel
    sender: SyncSender<ChatMessage>, // the same channel's real sender (worker plan owns a clone)
    worker: StreamWorker,          // the REAL delivery path (dedup → convert → try_send)
    phases: Vec<RatePhase>,        // the scenario schedule
    phase_index: usize,
    frames_in_phase: u64,
    credit: u64,                   // rational-rate accumulator
    pending: VecDeque<ChatTextMessage>, // retained-at-sender backlog (FIFO carrier order)
    next_id: u64,                  // fixture carrier id counter ("latency-m-{n}")
    window_opened: bool,           // the ONE zero-carrier token batch that opens the live window
    collector: TestCollector,      // owned so advance() is deterministic per frame step
    pacing_interval_nanos: u64,
}
```
Constructor is a single ≤2-args parameter object: `PacedQueueProducerSource::new(parts: ProducerParts)` where `ProducerParts { channel: ChatChannel, fakes: ChatFakes, collector: TestCollector, scenario: LatencyScenario, pacing_interval_nanos: u64, stop_flag: StopFlag }` (field-by-field assembly inside `new`; the ≤2-args rule). The `StreamWorker` inside `new` is built as: `let mut plan = worker_plan(&fakes, &channel); plan.diagnostics = Some(collector.collector.clone()); plan.stop_flag = stop_flag.clone(); StreamWorker::new(plan, LIVE_CHAT_ID_FIXTURE.to_string())` — the SAME pattern as `chat_delivery_diagnostics_tests.rs` (diagnostics override onto the existing builder; no new fixture plumbing in `runtime_chat_fakes.rs`). `StreamWorker::plan` retains its own `sender` clone so the channel outlives every borrow.

`impl MessageSource for PacedQueueProducerSource` — `next_message` calls ONE private step fn (≤50-line bodies, ≤2 nesting):

```
step(frame_step):
  1. self.collector.advance(self.pacing_interval_nanos);   // simulated frame-time tick, FIRST
  2. let pulled = self.delegate.next_message();            // the REAL consumer; ≤1 message per frame
  3. rate release: released = active_phase.rate.advance(&mut self.credit);
     for _ in 0..released: self.pending.push_back(self.next_carrier()); // new fixture id per released message
  4. capacity-guarded injection: in_channel = self.sent − self.pulled_count;
     free = CHAT_CHANNEL_CAPACITY − in_channel;            // exact, single-threaded
     injectable = min(free, pending.len());
     drain `injectable` carriers FIFO from `pending` into
     `self.worker.process_batch(yielded(&response_plain(carriers)))`;  // the REAL path
     self.sent += injectable;
  5. pull accounting: if pulled == Ok(Some(_)) { self.pulled_count += 1; }
     (pulled_count += 1 exactly once per Some; Ok(None) adds nothing)
  6. return pulled;
```

Guarantees (each asserted by a test or by construction):
- Injection always ≤ free ⇒ `send_stop_aware`'s `try_send` NEVER returns `Full` inside this harness ⇒ zero spin, zero deadlock, deterministic step counts. The real Full-arm retain-retry stays pinned by the EXISTING threaded rosters (§1.2). This is the modeled deviation, declared in the doc map (§4): "the harness retries the sender's retained messages at frame granularity; the observable queue outcome (queue pinned at capacity, nothing dropped, eventual drain) is identical; the real threaded retry path is pinned separately" — the plan-authoritative wording.
- FIFO order preserved: `pending` re-enters first; the real channel preserves order.
- Every released message is EXACTLY in one of three accounting buckets at any instant — in `pending` (retained retrying at the sender), in the bounded channel (enqueued, not yet dequeued), or dequeued: `due_total == received_total_sent_so_far + pending.len()` and `sent == dequeued_total + leftover_at_stop` (requirement B's inequality, made precise in §3.7).

### 2.3 `run_scenario` (fixtures file)

```
run_scenario(scenario: LatencyScenario) -> ScenarioOutcome:
  1. config = valid_config(); pacing = PipelinePacing::from_config(&config.video).expect("validated FPS derives pacing");
     interval = pacing.interval_nanos();                 // requirement D: FPS from config, never a literal
  2. temp = tempfile::tempdir(); chat_store = ChatStore::new(ChatStoreSettings::from_config(&config with
     chat.log_file overridden to temp/"chat.log"))         // the temp_store() pattern; keep the temp alive
  3. harness = enabled_collector_with_pacing(interval);   // §2.4 wrapper: real Timer replaced, all caps ON
  4. fakes = ChatFakes::new();                            // scripted lookups never called; fixture strings only
  5. stop = StopFlag::new();
  6. producer = PacedQueueProducerSource::new(ProducerParts { channel: chat_channel(), fakes, collector,
     scenario, pacing_interval_nanos: interval, stop_flag: stop.clone() });
     // inside: ONE real history-window open (worker.retain_token path NOT used — instead the source performs
     // its step-4 injection ONLY, and the FIRST frame step additionally calls
     // worker.process_batch(yielded(&response_with_token(vec![], RESUME_TOKEN_FIXTURE))) once (a flag field
     // `window_opened: bool` in the source) — zero-carrier token batch, the exact script_token_batch_session shape)
  7. sink = RecordingSink::fresh().0 .stop_after_writes(scenario.total_frames() as usize, stop.clone());
  8. pipeline = FramePipeline::new(PipelineParts {
       chat_store, renderer: Box::new(StubRenderer::fresh().0), sink,
       source: Box::new(producer), clock: Box::new(FakeClock::fresh().0),
       pacing: PipelinePacing::raw(interval, TEST_FRAME_BYTES),  // tiny 2×2 stub frame; interval from CONFIG fps
       stop_flag: stop, diagnostics: Some(harness.collector.clone()),
     });
  9. pipeline.run().expect("the offline scenario run completes");   // FakeClock: zero real sleeps
 10. leftover = drain by looping receiver.try_recv() until Empty;   // proves queued messages persist at stop
 11. ScenarioOutcome { total_frames, pacing_interval_nanos: interval, snapshot: harness.snapshot(),
     leftover, kept_temp: temp }
```
`run_scenario` body stays ≤50 lines by extracting `open_log_store(&config, &temp) -> ChatStore` and `drain_remaining(&Receiver<ChatMessage>) -> Vec<ChatMessage>` into named helpers (both ≤2 args). The `PipelinePacing::raw(interval, TEST_FRAME_BYTES)` combination is REQUIRED because `StubRenderer` emits 12-byte frames while real config geometry is 1080p — the interval still comes from the configured FPS (requirement D honored; geometry substitution is a documented harness simplification, pinned by this same line: nothing hardcodes 30).

### 2.4 Sanctioned test-only fixture addition (the ONLY touch to an existing diagnostics file)

`diagnostics_test_fixtures.rs` gains ONE `pub(crate)` wrapper (enabled-capabilities collector whose fake intervals match the scenario pacing):

```rust
pub(crate) fn enabled_collector_with_pacing(pacing_interval_nanos: u64) -> TestCollector {
    test_collector(&enabled_diagnostics(None), pacing_interval_nanos)
}
```
(+7 lines → 126 ≤ 200). Rationale: the existing `enabled_collector_for_tests()` pins `pacing_interval_nanos = 40`, which would make `frame_jitter_ns` a config-derived mismatch (harmless but ugly); with the wrapper, `frame_jitter_ns.max_ns == 0` is assertable (see §3.6). Test-only file; zero production behavior.

### 2.5 Determinism model (FINAL; requirement D + no-sleep rule; rational rates)

- Clocks: pipeline pacing = `FakeClock` (advances only on `sleep_nanos`; sleeps recorded); diagnostics stamps = the `TestCollector` fake `Arc<AtomicU64>` clock advanced EXACTLY once per frame step (step 1 of `next_message`) by `interval = 1e9 / fps`. No thread sleeps anywhere in the rosters; no real-time waits; scenario wall-clock ≈ microseconds.
- Rates as rationals: a sustained rate is `RateProfile { numerator, denominator }` messages/frame with integer credit-release arithmetic (§2.2) — "near capacity" = `{1,1}`, "below" = `{1,2}`/`{1,8}`, "above" = `{2,1}`, bursts = one-frame large numerators. No floating point; no flakiness.
- Capacity decomposition (requirement D, encoded in docs §4.3 and asserted by the fps pin test): the consumer calls `next_message` ONCE per frame and accepts at most ONE message (`runner.rs::accept_next_message`), so nominal consumption capacity = **one message per frame** = `fps` messages per second (30/s at the fixture FPS); the bounded channel holds `CHAT_CHANNEL_CAPACITY` (64) entries; memory stays bounded because the channel never overflows (pinned) and the sender-side retention shifts the backlog to the (bounded) worker-side instead of growing memory.
- Latency values: because enqueue stamps land on `frame × interval` boundaries and dequeue rides the next/any later frame, `message_channel_ns` (`lag = dequeue_stamp − enqueued_nanos`) is a deterministic multiple of the config interval — assertable EXACTLY (§3.6). `render_ns`/`write_ns`/`message_render_ns` are exactly `0` (stub render/write complete inside one collector instant — which pins the hook placement: the write hook wraps the sink call, the render hook wraps the render call). `frame_interval_ns` is `0` (the diagnostics fake clock jumps once per frame at the accept anchor, so consecutive-tick diffs record 0) and `frame_jitter_ns` = `interval` by the same reasoning — asserted as such, with the honest note in §4 that MAGNITUDE evidence comes from operator runs (real `InstantMonotonicNanos`), not the offline harness. The offline harness therefore proves: stage ANCHORS, propagation, counts, verdicts, accounting — not absolute magnitudes.
- `maybe_emit_summary` fires every 1 s of fake-collector time (≈ every 30 frames) — harmless mid-run; no scenario asserts or depends on summary timing.

### 2.6 FFmpeg-write boundary (FINAL)

`RecordingSink` is the FrameSink seam: every `write_frame` call is recorded frame-by-frame (byte-identical `[0,0,0,…]` vectors of 12 bytes) and `finish()` is faked — NO FFmpeg process, NO local MP4. The `write_ns` hook on the REAL loop measures the injected boundary (`note_write_started`/`note_write_completed` in `runner.rs::write_frame_bytes`) around the fake write; the "no real process" guarantee doubles as the no-broadcast/no-credential guarantee (requirement C). The pinned real-FFmpeg argv/shapes stay untouched (§7).

---

## 3. FINAL Scenario Matrix (requirement B ↔ named tests; rosters pre-assigned)

Capacity math used by every scenario (all values derived, never literals):
- `interval = PipelinePacing::from_config(&valid_config().video).interval_nanos()` = `1_000_000_000 / fps` = 33_333_333 ns at the fixture FPS.
- `CAP = CHAT_CHANNEL_CAPACITY` (64).
- `drain_margin_phases = 4 × CAP` frames for every trailing zero-rate phase (worst case pending ≈ 66 + queue 64 = 130 ≤ 256 frames of drain capacity).
- dequeue cadence: every frame that pulls a message records `message_channel_ns` at the frame boundary — the lag for FIFO head = `interval × (pull_frame − enqueue_frame)`.

Unique-carrier bound: every scenario's total released messages ≤ 256 < `CHAT_DEDUP_WINDOW` (512) — dedup never filters, asserted indirectly by the accounting equality (`received == due − pending`).

### 3.1 `latency_load_tests.rs` (6 tests)

| # | Test name | TODO bullet | Phases (`Vec<RatePhase>`) | Expected assertions (formulas in constants) |
|---|---|---|---|---|
| 1 | `low_rate_below_capacity_drains_per_frame_with_zero_backlog` | B·low rate below capacity | `[{160, {1,16}}]` | `snapshot.received_total == enqueued_total == dequeued_total == 10`; `queue_depth_estimate == 0`; `in_flight_high_water == 1`; `oldest_in_flight_age_ns == None`; `message_channel_ns`: count 10, `min_ns == Some(interval)`, `max_ns == interval`; `render_ns.max_ns == 0`; `write_ns.max_ns == 0`; `write_ns.count == 160` (RecordingSink threshold = total_frames); every promoted bucket of `message_channel_ns` is bucket[3] = (10ms, 50ms) — bucket tuple `== (10_000_000, Some(50_000_000), 10)` |
| 2 | `single_burst_temporarily_fills_channel_then_drains` | B·burst fills | `[{1, {CAP,1}}, {CAP+2, {0,1}}]` (67 frames) | burst frame: all CAP converted+enqueued (`received == enqueued == CAP`); `high_water == CAP`; drains fully: `dequeued == received`, `leftover.is_empty()`, `depth == 0`, `oldest == None`; `message_channel_ns.min == Some(interval)`, `max == CAP × interval` (head enqueued f0 stamp 0, pulled f1; tail pulled fCAP); NO drop, NO full retry (`full_retry_total == 0` in-harness — real Full arm pinned by existing rosters) |
| 3 | `sustained_below_capacity_keeps_backlog_stable` | B·sustained below | `[{128, {1,2}}, {4, {0,1}}]` | `dequeued == received == 64`; `leftover.is_empty()`; `high_water == 1`; `min == max == interval` |
| 4 | `sustained_at_capacity_holds_one_frame_backlog` | B·sustained near | `[{128, {1,1}}]` | steady verdict: `high_water == 1`; `dequeued == received − 1`; `leftover.len() == 1`; `oldest == Some(interval)²` (single queued message enqueued at frame N−1; stamp arithmetic gives age = interval); `message_channel_ns.min == max == interval` |
| 5 | `sustained_above_capacity_pins_queue_at_capacity_and_drains_after_rate_falls` | B·sustained above (+ recovery tail) | `[{128, {2,1}}, {4×CAP, {0,1}}]` | growth-then-pin verdict: `in_flight_high_water == CAP` (queue pins at capacity — bounded memory); then drains: `dequeued == received == 256 == released_total`; `leftover.is_empty()`; `depth == 0 `; `message_channel_ns.min == Some(interval)`, `max == 2 × interval` (first two injections dequeue at f1/f2; after pin, every enqueued message dequeues exactly one frame later ⇒ all lags ∈ {interval, 2·interval}) |
| 6 | `recovery_after_burst_drains_backlog_to_zero` | B·recovery after burst | `[{1, {CAP+8, 1}}, {4×CAP, {0,1}}]` | burst exceeds free capacity: 64 enqueued at burst frame, 8 retained at sender then enqueued during drain; `received == enqueued == dequeued == CAP + 8`; `leftover.is_empty()`; `high_water == CAP` (real-world outcome: queue pinned while the surplus drains); accounting eq holds |

² exact value: age = `N×interval − (N−1)×interval` = `interval`; compute in the test from `run.total_frames`/`pacing_interval_nanos`, never a magic literal.

### 3.2 `latency_load_loss_tests.rs` (4 tests)

| # | Test name | TODO bullet | Content |
|---|---|---|---|
| 7 | `round_trip_counts_prove_no_silent_loss_and_bounded_memory` | B·bounded memory + no silent mid-session loss; mixed profile | mixed phases: `[{1, {40,1}}, {16, {2,1}}, {64, {1,2}}, {4×CAP, {0,1}}]` (total due 40+32+32 = 104). Shared helper `assert_full_round_trip_accounting(scenario, run)` (in fixtures): `snapshot.enqueued_total == snapshot.received_total` (nothing Full in-harness) ∧ `snapshot.received_total == due_total − snapshot-side pending…` — FINAL precise inequality: `due_total == received_total + still_pending_at_stop` and `received_total == dequeued_total + leftover.len()`; with the drain-complete phases above both residuals are 0 ⇒ `due_total == received_total == dequeued_total` and `leftover.is_empty()`; plus `in_flight_high_water <= CHAT_CHANNEL_CAPACITY` (bounded), plus `due_total == 104` as the mixed-profile sanity literal (computed via scenario constant `MIXED_DUE_TOTAL` assembled as `40 + 16*2 + 64/2`) |
| 8 | `stop_with_messages_queued_documents_shutdown_loss` | B·shutdown behavior with messages still queued | phases `[{1, {CAP,1}}, {23, {0,1}}]` → total_frames = 24 = RecordingSink threshold; the REAL loop breaks at the stop check, `sink.finish()` called exactly once (`SinkEvents.finish_called`), dequeued `== 23 < received == CAP`, `leftover.len() == CAP − 23 == 41` and every leftover message `to_string()` matches its fixture author/text (they persist intact in the bounded channel); terminal cell empty; `queue_depth_estimate == 41`; oldest age `== Some(24 × interval)` (all burst entries share stamp 0). Mirrors the documented shutdown behavior ("Shutdown losses: queued-but-unconsumed messages (≤64) are never logged; log/video never diverge") |
| 9 | `rate_profile_due_counts_follow_rational_schedule` | harness correctness (requirement C) | pure: `RateProfile { 2, 3 }.advance` over 6 frames yields `0,1,1,0,1,1` exactly; `RateProfile { 64, 1 }` yields 64, 0, 0; credit never goes negative nor drifts (`credit == 0` after each full cycle) |
| 10 | `scenario_pacing_interval_derives_from_configured_fps` | requirement D pin (regression guard) | assert the derivation identity: `PipelinePacing::from_config(&valid_config().video).interval_nanos() * fps == 1_000_000_000` with fps read from the fixture (`valid_config().video.fps`) — no second scenario run. In ADDITION, every scenario test carries ONE line asserting `run.pacing_interval_nanos` equals the config-derived interval (guards against any silent hardcode of 30) |

### 3.3 Existing rosters / docs NOT changed by Task 2 code (verified)

- NO edits to `chat_worker_burst_tests.rs`, `chat_delivery_diagnostics_tests.rs`, `runner_diagnostics_tests.rs`, `chat_worker.rs`, `chat_stream_worker.rs`, `chat_channel.rs`, `source.rs`, `chat_stream_delivery.rs` logic (footer registration only, §6 commit 2), `runtime_chat_fakes.rs` (its `worker_plan` is reused as-is with the diagnostics override — the pattern already used by the delivery diagnostics roster).
- NO config/example/doc file edits in 4.2. Version stays 0.14.0. No Cargo.toml/Cargo.lock change (all needed crates — `tempfile`, `serde`, `tracing` — are already in the test graph).
- Scorekeeping: `.agent/project-structure.md` run-section roster counts and the diagnostics-fixtures line are updated ONLY in the docs pass (§5), after the OBSERVED test count exists.

### 3.4 Verdict vocabulary (requirement F wording for the doc)

Stable (depth oscillates ≤1 and returns to rest), Drain (backlog reaches zero), Grow→Pin (backlog grows to CAP then stops growing — bounded), Recover (post-burst backlog returns to zero). Thresholds: NONE asserted as production limits in Task-2 tests; per TODO, thresholds are diagnostic observations only, justified by the measured fixtures above (§4.5 records each verdict as the observed behavior; the doc names them "regression guards" phrasing only).

---

## 4. Documentation Target (FINAL choice: ONE new canonical doc)

**Chosen: `docs/latency-test-map.md`** — a NEW canonical document (TOC required when > 100 lines) owned by the 4.4 docs-specialist commit. Rejected: extending `docs/production-run.md` (already 414 lines; the map + operator procedure + template + matrix would push it past maintenance comfort, and its §Manual Verification Procedure stays the canonical operator base the new doc links to, never duplicates).

### 4.1 Map table outline (`docs/latency-test-map.md` §"One synthetic message, boundary by boundary")

Columns: boundary stage | production anchor (file :: function) | app-measured metric (diagnostics field, exact name) | needs external observer?

| Boundary | Anchor | App-measured | External observer needed? |
|---|---|---|---|
| 0. provider publication (`publishedAt`, sender side) | `youtube/live_chat_stream_mapping.rs` (parse) → carried on `ChatMessage` (`with_timestamp`) | `provider_epoch_seconds` on timing rows — provider-supplied WALL CLOCK, never diffed against monotonic stamps (`CLOCK_DOMAIN_NOTE`) | YES for the send→streamList-delivery leg (upstream delivery delay) |
| 1. streamList batch receipt | `run/chat_stream_delivery.rs :: process_batch` | queue row `batch_size` / `received_total` | no |
| 2. worker per-message receipt | `chat_stream_delivery.rs :: deliver` → `note_worker_receipt` | receipt stamp → enqueue `wait_ns` | no |
| 3. enqueue accepted | `chat_stream_delivery.rs :: send_stop_aware` (`Ok(())` arm) | `enqueued_total`, `queue_depth_estimate`, `in_flight_high_water`, queue row `backpressure` | no |
| 4. dequeue by the frame loop | `pipeline/runner.rs :: accept_next_message` (via `run/source.rs :: ChatSource`) | histogram `message_channel_ns` (enqueue→dequeue) | no |
| 5. render completion | `pipeline/runner.rs :: render_and_write_frame` | `message_render_ns` (dequeue→render) + `render_ns` | no |
| 6. FFmpeg stdin write start/complete | `pipeline/runner.rs :: write_frame_bytes` | `write_ns` + `written` stage lag row | no (the app only opens the pipe; FFmpeg's own ingest into YouTube is stage 7) |
| 7. FFmpeg→YouTube ingest / live processing | external process (`streaming/ffmpeg.rs`, `args.rs`/`stream_args.rs` — byte-locked argv) | only the process run's exit/shutdown evidence lines — NOT per-message timing | YES |
| 8. viewer playback first-visible | external (watch page) | none | YES — the ONLY way to observe latency's last leg |

Narrative paragraph under the table: how one synthetic id (`latency-m-*` fixture carriers in the offline harness; a real operator-sent message in live runs) travels stages 1→6 inside one summary window and how to read the `message|queue|summary` row kinds (link `production-run.md#pipeline-observability-opt-in-diagnostics` for row semantics instead of repeating them).

### 4.2 Interpretation matrix outline (requirement I)

Five delays, per stage: (1) upstream chat delivery delay — externally inferred ONLY (provider wall-clock vs local receipt correlation, with the `CLOCK_DOMAIN_NOTE` caveat: never presented as precise); (2) application queue delay — measured directly (`wait_ns`, `message_channel_ns`, depth/high-water/oldest age); (3) frame-render delay — measured directly (`message_render_ns`, `render_ns`, interval/jitter); (4) FFmpeg-ingest delay — the app measures ONLY its stdin write (`write_ns`); FFmpeg-internal encode and YouTube ingestion are externally inferred; (5) YouTube viewer playback delay — external only, operator-run (§4.6). Table with a "Why" column citing the anchors above.

### 4.3 Capacity-math outline (requirement D)

Define from config, never hardcoded: `interval = 1_000_000_000 / fps` (validated FPS from `video.fps`, numeric example at the fixture 30 for illustration ONLY); one-message-per-frame consumer (`runner.rs::accept_next_message` consumes ≤1) ⇒ **nominal consumption capacity = one message per frame = fps messages/second**; the bounded channel capacity (64) and the retain-retry outcome under overload; why memory stays bounded (channel never overflows; sender-side retention is the worker's bounded stall, not memory growth). Include the harness's modeled-deviation note verbatim from §2.2's rationale sentence.

### 4.4 Offline scenario table outline (requirements A/B/C/F)

Test name ↔ bullet ↔ scenario phases ↔ "expected verdict column" (Stable/Drain/Grow→Pin/Recover) ↔ observable metrics (counter/depth/high-water/`message_channel_ns`/counts) — regenerated from §3's tables at docs time (the doc version quotes the OBSERVED test suite names; update the names if 4.3 renames anything). State the harness entry point (files + how to run: `cargo test --locked latency_load`).

### 4.5 Threshold stance (requirement F)

Named thresholds appear ONLY where the measured fixtures justify them, as regression-guard observations: (a) `dequeued == received` and `leftover empty` for every drain-complete scenario (loss/equity guard); (b) `in_flight_high_water ≤ CHAT_CHANNEL_CAPACITY` (bounded-memory guard); (c) `message_channel_ns.min ≥ interval` (the ≥-one-frame backpressure floor — an honest consequence of pull-then-inject ordering). NO production-latency SLO, no YouTube-behavior claim. Literal paragraph:

> "The offline fixtures measure this application only. Synthetic pipeline measurements never contact YouTube, never reproduce Google's server-side chat delivery, and never measure viewer buffering; no value produced by the automated suite is an end-to-end or viewer-latency figure."

### 4.6 Operator-run live test procedure outline (requirement G; links canonical, no duplication)

Sections (each short; setup/prerequisites LINK to `production-run.md#manual-verification-procedure-operator-run` — never repeated):
1. Separate VIEWER+SENDER account: the chat sender and the watching viewer use an account distinct from the streaming channel's account.
2. Consistent-clock method: EITHER (a) the manual stopwatch method — one operator-side phone/clock: start `run`, note the wall-clock moment T_send when the message leaves the sender account, note T_visible the moment the text first renders on the watch page, record `T_visible − T_send`; OR (b) the same-host/second-host synchronized clock method (both sides record with a shared wall clock; the two stamps are read AFTER the run and subtracted). The two clocks used are DOCUMENTED per result row.
3. Repeat count: AT LEAST five messages at a moderate, quoted-sent-time spacing (e.g. one message every ~30 s) — never treat one observation as the baseline; record all repeats.
4. Broadcast/player record: note the broadcast/player latency mode shown for the stream (the default/LOW/ULTRALOW setting available in YouTube Studio for this broadcast, as displayed) plus relevant playback conditions (network, watch-page device); never claim a value the UI does not show.
5. Diagnostics correlation per message: with `diagnostics` enabled (capabilities per the config example), find the sent message's rows in the diagnostics file/`RUST_LOG=debug` stream by matching the diagnostics row `epoch_seconds`/`provider_epoch_seconds` window against the recorded send time AND the chat-log line in the configured `chat.log_file` (same message, same second window); the row's `correlation` field is the provider message id — record it against the manual measurement row. Every internal stage duration for that message (`message_channel_ns`, `message_render_ns`, `write_ns` summary aggregates for that interval) is read from the diagnostics summary for that window and recorded SEPARATELY from the manual end-to-end figure.
6. End-to-end separation: the manual send→visible figure is recorded in its own column and NEVER summed with internal stage durations; the possible double-counting is stated once (an internal stage duration is already inside the end-to-end total).
7. Observability on vs off overhead comparison: repeat the same message profile once with `diagnostics` all-enabled and once with everything disabled (or the section absent — byte-identical run path); compare the runs qualitatively (walk-through of frame cadence stability on the watch page and the process's rough CPU/host load as observably reported by the operator's own tooling); declare honestly that sub-percent overhead was NOT instrumented beyond this comparison.
8. Recording: fill the results template (§4.7) per message/day; keep every claim operator-run (no agent-executed live claim ever).

### 4.7 Results template outline (requirement H)

Table fields (one row per repeat, one header block per session):
`run/build identifier` (version + git HEAD + branch, e.g. `v0.14.0 @ <commit>`) · `date/time` (local + UTC per row) · `configured FPS` (from the run's `config.json`, not the example) · `diagnostics settings` (which of the four capabilities + output_file + interval) · `message rate/burst profile` (how many messages, at what spacing — mirrors the offline profile shape) · `queue metrics` (received/enqueued/dequeued/full-retries/retained/depth estimate/high-water/oldest in-flight age from the matching summary row) · `internal timing summaries` (per-histogram count/min/max/mean for the five `*_ns` aggregates over the session window) · `external viewer delay` (per repeat, with the clock method noted; or "not measured this run") · `notes` (latency mode, playback conditions, incidents, duplicates/loss observed).

### 4.8 The ~35-second framing (TODO global criterion + requirement J)

Literal paragraph to land verbatim:

> "The observed ~35 seconds between sending a chat message and seeing it in the player is an operator measurement of the WHOLE end-to-end path (upstream chat delivery, application processing, encoding/ingest, YouTube live processing, and viewer playback buffering). It is a baseline to investigate stage by stage with the procedures in this document — never proof of a single-stage bottleneck, never a platform constant, and never reproduced by the offline test suite."

Automated checks stay offline (requirement J): no new test contacts Google; no test reuses real-shaped endpoint constants against the network; harness strings stay fixture-shaped (`author-fixture`, `text-fixture`, `token-fixture`, `latency-m-*`).

---

## 5. Cross-Doc Wiring (docs pass, commit per §6.4)

1. `docs/production-run.md` — `## Pipeline Observability (Opt-In Diagnostics)` section: add ONE sentence (its own short paragraph at section end, before `### Baseline Overhead When Disabled`'s subsection order is preserved… FINAL placement: as the last bullet of `### What Diagnostics Never Contain`'s FOLLOWING paragraph, a dedicated line: "The offline test map that reproduces each stage boundary with the real queue and frame loop, and the operator-run viewer-latency measurement procedure that completes the picture: [`docs/latency-test-map.md`](latency-test-map.md).").
2. `docs/configuration.md` — `## See Also`: add the latency-test-map link (one line).
3. `README.md` — Production Run section: extend the existing diagnostics pointer sentence with `; the offline latency/load test map and the operator-run viewer-latency procedure are in [`docs/latency-test-map.md`](docs/latency-test-map.md)`.
4. `.agent/project-structure.md` — run section: add the three new roster/fixture files and re-count the roster/test totals to the OBSERVED numbers; diagnostics section: note `enabled_collector_with_pacing` in the fixtures line. NO tree-shape change beyond the test files.
5. NO duplicate operator procedures ANYWHERE: the manual production procedure stays canonical in `production-run.md`; latency-test-map links it.

---

## 6. Git Boundaries — 4.2 Commit Sequence (`feat/observability-game-handoff` only; push NEVER in 4.2)

1. `chore: track Task-2 latency test-map plan` — THIS plan file (skip if the caller already tracked it).
2. `test(run): add offline paced latency harness over the real chat queue and frame loop` — `src/app/run/chat_stream_delivery.rs` (footer registration ONLY), `src/app/run/latency_load_fixtures.rs`, `src/app/run/latency_load_source.rs`, `src/app/diagnostics/diagnostics_test_fixtures.rs`.
3. `test: pin rate, burst, sustained, loss, and shutdown latency scenarios` — `src/app/run/latency_load_tests.rs`, `src/app/run/latency_load_loss_tests.rs`.
4. 4.4-owned docs commit (LANDS AFTER 4.3): `docs: add the latency test map and operator live-latency procedure` — `docs/latency-test-map.md` (new), `docs/production-run.md`, `docs/configuration.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` (dated Task-2 4.2 block), and later the 4.6 DONE-mark follow-ups. KEEP this out of 4.2.
5. The full dev-checks gate log must be captured per run under gitignored `logs/checks/` — never committed; `git status --porcelain` equals the intended file set before every commit (check-ignore compliance).

---

## 7. Byte-Stability Ledger — ZERO production behavior change in Task 2

Surfaces that MUST NOT move (list authoritative for this step):

| Surface | How guarded |
|---|---|
| Locked `args.rs`/`stream_args.rs` 33/42-token streaming argv vectors + CBR shapes | untouched; pinned by `stream_args_tests.rs`, `stream_args_health_tests.rs`, `streaming_stage_tests.rs` |
| `run` shutdown ordering, exit codes (0/1/2, single `error:` funnel), gate join position | untouched; pinned by `runtime_tests.rs`, `error_tests.rs`, `mod_tests.rs` |
| Bounded 64-entry channel, `try_send`-only delivery, 1 s stop-slices, backoff/terminal machinery | untouched — the harness READS these seams; pinned by `chat_worker_burst_tests.rs`, `chat_worker_budget_tests.rs`, `chat_worker_flow_tests.rs` |
| `ChatMessage` Display (`author: text`), normalization, message-id accessor | untouched; pinned by `model_tests.rs` |
| Secret redaction (`[redacted]`, capture-time `finish()`, Rtmps redacting Debug/Display) | untouched; harness uses fixture strings only; grep hygiene gate unchanged |
| `demo` timeline (28 messages / 496 frames) and local-recording vector | untouched |
| Config schema/validation (27 rules), `config.example.json`, version 0.14.0 | untouched |
| Diagnostics collector event shapes, CLOCK_DOMAIN_NOTE, rotation policy | untouched (only the test-fixture wrapper added, §2.4) |
| Existing diagnostics rosters (23 tests) and worker/pipeline rosters | untouched |

Permitted code edits (the ONLY ones): (a) the `#[cfg(test)]` registration block in `chat_stream_delivery.rs`; (b) the four new cfg(test) files; (c) the `pub(crate)` `enabled_collector_with_pacing` wrapper in `diagnostics_test_fixtures.rs`. ZERO new `#[allow(`, zero dependency/config/version changes, zero commented-out code.

---

## 8. Line-Budget Table + STOP Protocol

| File | Current | Projected (post-fmt) | Bound |
|---|---|---|---|
| `src/app/run/chat_stream_delivery.rs` | 188 | 192 | ≤200 (registration block +4) |
| `src/app/run/latency_load_fixtures.rs` (new) | — | ≤150 | ≤200 |
| `src/app/run/latency_load_source.rs` (new) | — | ≤125 | ≤200 |
| `src/app/run/latency_load_tests.rs` (new) | — | ≤175 | ≤200 (TIGHT — mark) |
| `src/app/run/latency_load_loss_tests.rs` (new) | — | ≤150 | ≤200 |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | 119 | ≤128 | ≤200 |

`cargo fmt` runs before every count. **STOP protocol (binding):** if any touched file's post-`cargo fmt` count exceeds 200 (or a projected-TIGHT file fires), the implementer STOPS, reports the exact `wc -l` counts for every touched file, and waits for caller adjudication of the split (the `stream_args` split precedent). Never self-stage a split, never disable a lint, never trim comments to sneak under a cap. Bodies ≤50 lines, ≤2 nesting, ≤2 args (parameter objects `ProducerParts`/`QueueEvent`-style), no commented code, self-documenting names (the roster doc-comments stay ≤6 lines each).

---

## 9. Verification (read-only VM probes; implementer runs through the container)

1. `alpine-vm_vm_status` FIRST, every time, before any compose command.
2. Targeted runs (each via `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked <filter>`, exit 0):
   - `cargo test --locked latency_load` — all 10 new tests;
   - `cargo test --locked chat_delivery_diagnostics` + `cargo test --locked diagnostics` — post-fixture-touch sanity (27 + 23 existing must stay green);
   - `cargo test --locked burst` — no interference with the threaded worker rosters.
3. Full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — exit 0, four PASS lines, `ALL CHECKS PASSED`. Baseline OBSERVED **540 / 0**; projected additional test count = **+10** (6 + 4) ⇒ projected total **550**; the OBSERVED count from the run governs and is recorded verbatim in the completion report.
4. Runtime guard: every scenario runs on `FakeClock`/`TestCollector` fakes — expect the whole new family to contribute well under 1 s to the test phase; if any latency scenario exceeds ~2 s wall time, STOP and report (evidence of an accidental real sleep/timing dependency).
5. Hygiene before each commit (§6): `git status --porcelain` matches the intended set; no `logs/`/`target/`/`dist/` staged; secret-pattern audit `ya29|AIza|key=|rtmps://` counts relative to HEAD unchanged; `#[allow(` count in `src/` = HEAD count + 0; no message-text or author-name assertions on REAL strings (fixture strings only).

---

## 10. Front-End Determination

NOT front-end — Task 2 is Rust test-harness + documentation only (no 4.1a/4.5a sub-steps executed).

---

## Traceability — TODO Task-2 requirement → plan section

| Requirement TODO p. 72–101 | Plan section |
|---|---|
| A — documented test map, one synthetic message through receipt→enqueue→dequeue→render→FFmpeg-write; app-measured vs external | §4.1 (+ offline journey = §2.3/§3) |
| B — deterministic automated tests around real queue/consumer behavior; low rate / burst / sustained below-near-above / recovery / bounded memory + no silent loss / queued shutdown | §2.2–§2.3 (real seams), §3.1–§3.2 (6 + 4 tests); the accounting inequality rows live inside §3.2's test 7 and each scenario's assertion column |
| B·Inequality precise form (enqueued/retained/dequeued, no silent loss) | §2.2 step 4 accounting + §3.2 test 7 formulas |
| C — repeatable load/stress harness, controlled rates + burst sizes, no credentials/broadcast | §2.1–§2.3 harness + `RateProfile`/`LatencyScenario`; no process; no broadcast |
| D — current configured FPS + channel capacity as inputs; never hardcode 30 FPS; one-message-per-frame ⇒ nominal capacity | §2.5 determinism model + §1.4 config facts + §3.2 test 10 pin |
| E — report throughput, high-water, age, enqueue-to-render, render, write durations | §1.1 metric map + §3 expected-value columns + §4.4 doc table |
| F — thresholds as observations/regression guards only; no synthetic-reproduces-YouTube claims | §3.4 + §4.5 |
| G — operator-run live test (separate account, clock method, repeats, latency mode, id correlation, e2e separate, on/off overhead) | §4.6 |
| H — results template (9 fields) | §4.7 |
| I — interpretation matrix (upstream / queue / render / FFmpeg-ingest / viewer playback; direct vs inferred) | §4.2 |
| J — offline automated checks; no Google-behavior claims; never real-shaped endpoint constants against the network | §2.6 + §4.8 + §9.5 hygiene |
| Global: 35 s = measurement framing; doc cross-links; no duplicate operator procedures | §4.8 + §5 |
