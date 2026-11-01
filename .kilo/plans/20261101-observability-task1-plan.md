# Task-1 Implementation Plan — Opt-In, Low-Overhead Pipeline Observability

- TODO: `.agent/todos/20261031/20261031-todo-1.md` — Task 1 (pp. 26–64) is the binding scope.
- Global plan: `.kilo/plans/20261101-observability-game-handoff.md` (pre-analysis sections are binding).
- Branch `feat/observability-game-handoff` (v0.14.0, already bumped at step 3 — DO NOT bump again).
- Front-end determination: **NOT front-end** (Rust CLI; no 4.1a/4.5a sub-steps). Repeated verbatim as required.
- Implementer: JUNIOR developer under 50% restriction; every structural choice below is already made. Do not re-decide anything marked FINAL.
- This step changes NO config/docs/version files except as listed in §7/§8.

---

## 1. Pre-Analysis — Verified Current-State Facts (read from the real files)

All facts below were verified by reading the actual files on this branch.

### 1.1 Pipeline seams (instrumentation anchors)

| # | TODO stage |_where it lives today (verified fn / field)_ | Diagnostics hook |
|---|---|---|---|
| 1 | Provider `publishedAt` | `src/youtube/live_chat_mapping.rs` — `ChatTextMessage { message_id: Option<String>, published_epoch: Option<u64> }` (known-good `live_chat_rfc3339` parser); moved onto `ChatMessage` by `into_chat_message` (`src/app/run/chat_stream_delivery.rs`, "C-1 conversion point", uses `with_message_id` + `with_timestamp` already). **No new capture code needed** — the provider wall-clock is already carried per message. | existing field; surfaced in event rows + docs as provider wall-clock |
| 2 | Local streamList receipt | `src/app/run/chat_stream_delivery.rs::process_batch` (live path after the history-pending discard) and the `deliver` loop | counter + monotonic stamp recorded at batch level |
| 3 | Enqueue into bounded channel | `src/app/run/chat_stream_delivery.rs::send_stop_aware` — the `try_send` loop (`Ok(())` / `TrySendError::Full(returned)` / `Disconnected`) | success + full-retry + retained-under-backpressure events |
| 4 | Dequeue by frame loop | `src/app/pipeline/runner.rs::accept_next_message` (the single `MessageSource::next_message` call point; `ChatSource` (`src/app/run/source.rs`) only does `try_recv`) | dequeue event + in-flight map pop |
| 5 | Render completion containing the message | `src/app/pipeline/runner.rs::render_and_write_frame` right after `renderer.render(...)` returns `Ok` | render-complete event + render duration histogram |
| 6 | FFmpeg stdin frame write | `src/app/pipeline/runner.rs::write_frame_bytes` around `self.sink.write_frame(frame.pixels())` | write-start/write-complete events + write duration histogram |

Frame-loop interval/jitter: `runner.rs::run` loop between `PacingSchedule::next_gap(...)`/`clock.sleep_nanos(...)` (`src/app/pipeline/pacing.rs`, `clock.rs` `FrameClock`). Collector keeps consecutive-tick stamps internally — the runner only passes `now_nanos` it already computes for pacing.
Reconnect count/duration: `chat_stream_delivery.rs::await_reestablishment` (the capped `backoff_delay_ms` + `sleep_in_stop_slices` path) — the observable app-side wait is the backoff slices.
Provider ids as correlation keys: `ChatMessage` already carries `message_id: Option<String>` but exposes NO accessor — `src/chat/model.rs` has `author()/text()/timestamp()` only. **One new `pub(crate) fn message_id()` accessor is required** (Display stays `author: text`, byte-untouched).

### 1.2 Configuration model / validation / loader / error (verified)

- `src/config/model.rs` — 176 lines. `AppConfig { youtube, oauth, video, renderer, chat, ffmpeg }`, every struct `#[serde(deny_unknown_fields)]`. Optional-section precedent: `FfmpegConfig.audio: Option<AudioConfig>` with `audio_enabled()` accessor. **model.rs has ~24 lines of headroom — the DiagnosticsConfig struct does NOT fit; a sibling file is pre-planned.**
- `src/config/validation.rs` — 171 lines, groups named predicates (`validate_video`, `validate_renderer`, `validate_required_strings`, `validate_oauth`, `validate_privacy_status`); 21 logical rules counted per the docs convention. Issue type: `ConfigIssue::new(field, problem)` + shared `positive_value_issue`/`blank_issue` helpers with `u32` args.
- `src/config/loader.rs` — 51 lines; registers 3 test rosters via the `#[cfg(test)] #[path = "..._tests.rs"]` footer pattern (`loader_tests.rs`, `broadcast_made_for_kids_parse_tests.rs`, `ffmpeg_audio_parse_tests.rs`).
- `src/config/error.rs` — 91 lines. `ConfigError { Io, Deserialization, Validation }` — **no new variants needed**; new diagnostics rules reuse `Validation { issues }`.
- `src/config/loader_tests.rs` — **exactly 200 lines (at the cap)**. Its embedded `example_json()` / `example_json_value()` / `write_temp_config()` / `expect_load_error()` helpers (~60 lines) are shared by the parse rosters; adding the diagnostics block (+9 lines) would break the cap → helper extraction is pre-planned into `src/config/test_fixtures.rs` (currently 54 lines, the sanctioned fixture-builder home).
- `src/config/test_fixtures.rs` — `valid_config()` fixture: `made_for_kids: Some(false)`, `audio: Some(AudioConfig { enabled: Some(true) })` — fixture-mirrors-example precedent; it must gain the new `diagnostics` field.
- `config/config.example.json` — 43 lines; sections: youtube / oauth / video / renderer / chat / ffmpeg (with `audio.enabled: true`).

### 1.3 Wiring chain (verified)

- `src/app/run/mod.rs` — **199 lines (at the cap)**. `run_runtime` builds `ChatStageSources { lookup, stream }` (line ~148), calls `chat_worker::build_chat_stage(chat, &broadcast_id, stop_flag)` → `(ChatSource, ChatWorkerPlan)`, spawns `GatePlan { chat_worker: Some(chat_worker), ... }`, and constructs `StreamingExecution { stop_flag, source, canceller, destination, stream_id, watch_url, gate }` for the runner.
- `src/app/run/chat_worker.rs` — **199 lines (at the cap)**. `ChatWorkerPlan { lookup, stream, broadcast_id, sender, terminal, stop_flag }`; `spawn_chat_worker`; `build_chat_stage` (3-arg sanctioned-exception family D-T3-15; signature MUST NOT grow); `run_chat_worker` with the contained-panic `catch_unwind`; `chat_runtime()`; `resolve_chat_id` cfg-split. The 4 test rosters + fakes are footer-registered here.
- `src/app/run/chat_stream_delivery.rs` — 195 lines. `process_batch` (history-first), `deliver` (dedup → C-1 convert → `send_stop_aware`), failure half (`handle_failure`/`handle_clean_close`/`await_reestablishment`), and the helper block (`sleep_in_stop_slices`, `backoff_delay_ms` with its PRE-EXISTING `#[allow(clippy::manual_clamp)]`, `record_terminal`, `transport_terminal`, `CHAT_WORKER_PANIC_DETAIL`) — the helper block is the sanctioned extraction target.
- `src/app/run/chat_stream_worker.rs` — 153 lines (`StreamWorker` struct + `run_stream_loop` + biased-select `consume`). **The select loop stays byte-untouched**; reconnect counting happens in `chat_stream_delivery`'s `await_reestablishment`.
- `src/app/run/seams.rs` — 70 lines. `ChatStageSources { lookup, stream }` (the ≤2-args parameter object extended for every stage addition — the sanctioned extension point).
- `src/app/run/streaming.rs` — 194 lines. `StreamingRunner::run(&self, app_config, execution)`, `StreamingExecution` bundle, `prepared_stream_pipeline` usage, shutdown-evidence emitters. Gains exactly one field pass-through.
- `src/app/pipeline/runner.rs` — **199 lines (at the cap)**. `PipelineParts` (7 fields, `pub(crate)`), `PipelineConstruction`, `StreamParts`, `FramePipeline` (`private` fields; `run`/`accept_next_message`/`render_and_write_frame`/`write_frame_bytes`), and the two constructors `prepared_pipeline`/`prepared_stream_pipeline` + `build_chat_store`/`build_renderer` (lines 144–198). FramePipeline/PipelineParts are re-exported `#[cfg(test)]`-gated from `pipeline/mod.rs` (57 lines; footer registrations incl. the AMENDMENT-2 comment).
- `src/app/pipeline/clock.rs` — 39 lines: `FrameClock` trait (`now_nanos`, `sleep_nanos`), `SystemFrameClock` with `Instant` anchor. Monotonic-only precedent to mirror.
- `src/app/pipeline/source.rs` / `source_error.rs` — the `MessageSource` seam; unchanged.
- `src/chat/model.rs` — 81 lines; `src/chat/store.rs` — 158 lines (log-append-first `accept`; untouched).
- `src/streaming/ffmpeg.rs` — 161 lines (`write_frame` `write_all` loop; capture-time `[redacted]` `finish()`; untouched). `src/streaming/sink.rs` — 92 lines (`FrameSink`, untouched). `src/streaming/args.rs`/`stream_args.rs` (the 33/42-token locked vectors) — untouched.
- `src/app/mod.rs` — 176 lines; declares `auth cli demo error logging pipeline run`; gains `mod diagnostics;` (+1).
- `src/app/run/gate.rs` — **exactly 200 lines; untouched by this task** (the chat plan rides inside `ChatWorkerPlan` transitively).

### 1.4 Established precedents the plan reuses

- Test-roster attachment: `#[cfg(test)] #[path = "name_tests.rs"]` registrations at file footers (loader.rs, ffmpeg.rs, chat_worker.rs, runner/mod.rs).
- `#[path]` sibling splits for caps: `stream_args.rs` (94+139 split), `ffmpeg_real_audio_tests.rs` split, `gate_diagnostics.rs`, `chat_stream_delivery.rs` split.
- Fixture-mirrors-example: `made_for_kids`/`audio` in `test_fixtures.rs`.
- Test-only fakes registered from production files: `runtime_chat_fakes.rs`, `stream_test_fixtures.rs`, `live_test_fixtures.rs`, `test_helpers.rs`.
- Gate: `alpine-vm_vm_status` first → `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`; baseline **OBSERVED 497 tests / 0 failed** before this TODO (README line 47 + context.md). Task 1 will raise the count; **the OBSERVED count governs** everywhere.
- Incremental commits: `chore: track ... plan` first, then feat/test/docs commits with meaningful conventional messages.

---

## 2. FINAL Diagnostics Configuration Design

### 2.1 JSON section (exact)

Add to `config/config.example.json` as a new top-level section AFTER `ffmpeg` (all four capability features explicitly present and false; tunables at their defaults):

```json
  "diagnostics": {
    "pipeline_timing_events": false,
    "queue_metrics": false,
    "periodic_summary": false,
    "summary_interval_seconds": 30,
    "output_file": "logs/diagnostics.log",
    "max_file_bytes": 1048576,
    "max_generations": 3
  }
```

Semantics (all capabilities independently switchable; every feature disabled by default):

- `pipeline_timing_events` (bool, default absent/false) — per-message stage event rows (stages 1–6) plus the in-memory frame-pipeline histograms (frame-loop interval/jitter, render duration, FFmpeg write duration). Frame metrics are **aggregation-only, never per-frame rows** (TODO: "avoid one log line per frame").
- `queue_metrics` (bool, default absent/false) — queue/backpressure counters: received (provider batch size, pre-dedup), enqueued, dequeued, full-channel retry events, worker-retained-under-backpressure count, channel in-flight high-water mark, oldest in-flight age, reconnect count + cumulative backoff.
- `periodic_summary` (bool, default absent/false) — on every `summary_interval_seconds` period, one summary row (histograms count/min/max/mean + fixed histogram buckets + all counters) emitted to the same sink. If timing/queue are OFF but summary is ON, the summary legitimately shows zero counters (harmless, documented — no extra coherence rules).
- `summary_interval_seconds` (u32, default `30` via named constant when absent) — the summary period; validated positive when present.
- `output_file` (string, optional) — structured JSONL diagnostics output file; **separate from the chat log** (validation FORCES distinctness). When present but all capability flags are false/absent, **no file is ever created** (the collector is `None`). When absent but any capability is ON, events/summaries are emitted as `tracing::debug!(target: "diagnostics", ...)` events (one JSON text per row; invisible at the default `info` filter — operator opts in via `RUST_LOG=debug`).
- `max_file_bytes` (u32, default 1_048_576 when absent) — per-generation size cap; validated positive.
- `max_generations` (u32, default 3 when absent) — kept rotated generation count; validated positive and ≤ 1000.

### 2.2 Serde struct (exact; new file `src/config/diagnostics_config.rs`)

`src/config/model.rs` gains ONLY: the `use` import, the `pub diagnostics: Option<DiagnosticsConfig>` field on `AppConfig` with a two-line doc comment, and nothing else (keeps model.rs ≤ 200). The struct + accessors live in the new file:

```rust
// src/config/diagnostics_config.rs  (declared in src/config/mod.rs; pub use re-exported)
use serde::Deserialize;

pub const DEFAULT_SUMMARY_INTERVAL_SECONDS: u32 = 30;
pub const DEFAULT_MAX_FILE_BYTES: u32 = 1_048_576;
pub const DEFAULT_MAX_GENERATIONS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsConfig {
    pub pipeline_timing_events: Option<bool>,
    pub queue_metrics: Option<bool>,
    pub periodic_summary: Option<bool>,
    pub summary_interval_seconds: Option<u32>,
    pub output_file: Option<String>,
    pub max_file_bytes: Option<u32>,
    pub max_generations: Option<u32>,
}

impl DiagnosticsConfig {
    // public introspection accessors (used by validation and by the runtime
    // collector); each unwraps with its named default:
    pub fn pipeline_timing_enabled(&self) -> bool   { /* Option<bool>::unwrap_or(false) */ }
    pub fn queue_metrics_enabled(&self) -> bool    { /* same shape */ }
    pub fn periodic_summary_enabled(&self) -> bool { /* same shape */ }
    pub fn summary_interval(&self) -> u32          { /* unwrap_or(DEFAULT_SUMMARY_INTERVAL_SECONDS) */ }
    pub fn output_file(&self) -> Option<&str>      { /* as_deref() */ }
    pub fn max_file_bytes(&self) -> u32            { /* unwrap_or(DEFAULT_MAX_FILE_BYTES) */ }
    pub fn max_generations(&self) -> u32           { /* unwrap_or(DEFAULT_MAX_GENERATIONS) */ }
}
```

Field doc comments: ONE short line each (the budget is tight; the full semantics live in `docs/configuration.md`). Accessors follow the `FfmpegConfig::audio_enabled()` precedent (single-expression bodies, no panics).

Registration: `src/config/mod.rs` adds `pub(crate) mod diagnostics_config;` + `pub use diagnostics_config::{DiagnosticsConfig, DEFAULT_MAX_FILE_BYTES, DEFAULT_MAX_GENERATIONS, DEFAULT_SUMMARY_INTERVAL_SECONDS};` — wait, keep it per existing style: `mod diagnostics_config; pub use diagnostics_config::DiagnosticsConfig;` and the three `DEFAULT_*` constants re-exported too (they are referenced by the diagnostics module as defaults; simplest: the diagnostics module imports them directly as `crate::config::diagnostics_config::{...}` — then only `DiagnosticsConfig` itself is re-exported). FINAL: declare `pub(crate) mod diagnostics_config;` in `src/config/mod.rs`; `src/config/model.rs` does `use crate::config::diagnostics_config::DiagnosticsConfig;`; external consumers import `crate::config::diagnostics_config::DiagnosticsConfig`. Do NOT add a re-export chain beyond that.

### 2.3 Validation predicates + recut rule count

New child file `src/config/validation_diagnostics.rs`, registered from `validation.rs` as a `#[path]` child sibling (`#[path = "validation_diagnostics.rs"] mod validation_diagnostics;` at validation.rs footer before its tests registration) — it must be able to call the private `is_blank`, `positive_value_issue`, and `ConfigIssue::new`, so child-of-validation is required (child modules see parent-private items). Add exactly ONE call line in `validation.rs::validate`: `validate_diagnostics(config, &mut issues);`.

Six new logical rules (recut count **21 → 27**), each a named predicate following the existing style:

| # | Predicate (named fn in `validation_diagnostics.rs`) | Rule |
|---|---|---|
| 22 | `is_inconsistent_summary_interval` | `diagnostics.summary_interval_seconds`, when present, must be positive (> 0) — message "must be positive (got {value})" reusing the shared `positive_value_issue` text |
| 23 | `is_inconsistent_max_file_bytes` | `diagnostics.max_file_bytes`, when present, must be positive |
| 24 | `is_inconsistent_max_generations` | `diagnostics.max_generations`, when present, must be positive |
| 25 | `is_out_of_range_generation_count` | `diagnostics.max_generations`, when present, must be at most `MAX_GENERATIONS_LIMIT = 1000` (bounds the per-rotation rename loop) — message `"must be at most 1000 (got {value})"` |
| 26 | `is_unusable_output_file` | `diagnostics.output_file`, when present, must not be empty/whitespace-only — reuses `blank_issue`/`BLANK_VALUE_PROBLEM` text |
| 27 | `is_chat_log_collision` | `diagnostics.output_file`, when present, must differ from `chat.log_file` — message mirrors the oauth collision phrasing: `"must be different from the chat log file (diagnostics output must stay separate from the ordinary chat log)"` |

All diagnostics rules are Section-absent-neutral: when `AppConfig.diagnostics` is `None` (or a field is `None`), no rule fires, so pre-0.14.0 operator configs keep parsing with zero behavior change.

### 2.4 Error surface

**No new `ConfigError` variants, no new `ConfigIssue` shape.** New issues ride `ConfigError::Validation { issues }` with dotted field paths `diagnostics.<field>` (Display contract unchanged; one actionable line, exit 1 — the CLI contract is preserved by construction). The `deny_unknown_fields` schema rejects unknown diagnostics keys and wrong types at deserialize time (pinned by the new parse roster).

---

## 3. FINAL Observability Architecture

### 3.1 Module ownership & file layout

New module `src/app/diagnostics/` (declared in `src/app/mod.rs` as `mod diagnostics;`). All items `pub(crate)` at most; internals private per the prefer-private rule. Every source file ≤ 200 lines; test rosters attached via the `#[cfg(test)] #[path = "..."]` footer pattern.

| File | Content | Budget |
|---|---|---|
| `diagnostics/mod.rs` | module docs (scope: read-only, opt-in, bounded, secret-free), submodule declarations, `pub(crate) use` re-exports, `DiagnosticsCollector::from_config(&AppConfig) -> Option<DiagnosticsCollector>` and the single named predicate `is_any_capability_enabled(&Option<DiagnosticsConfig>) -> bool` (single-section boolean rule) | ≤ 60 |
| `diagnostics/events.rs` | the `#[derive(Serialize)]` event structs (below) + `CLOCK_DOMAIN_NOTE: &str` constant | ≤ 120 |
| `diagnostics/histogram.rs` | `DurationHistogram` + boundaries + `HistogramSummary` | ≤ 100 |
| `diagnostics/state.rs` | `DiagnosticsState` (the bounded aggregate + in-flight map), `InFlightEntry`, `snapshot()` | ≤ 170 |
| `diagnostics/collector.rs` | `MonotonicNanos` trait + `InstantMonotonicNanos` + `DiagnosticsCollector` + `CollectorLocal` | ≤ 185 |
| `diagnostics/writer.rs` | `DiagnosticsFileWriter` (lazy open, JSONL, rotation) | ≤ 120 |
| `histogram_tests.rs`, `state_tests.rs`, `collector_tests.rs`, `writer_tests.rs`, `diagnostics_test_fixtures.rs` | rosters + the `#[cfg(test)]` fixed clock/harness | each ≤ 170 |

**Bounded cross-thread mechanism (FINAL choice; no new deps):** one `Arc<Mutex<DiagnosticsState>>` shared between the chat worker thread and the main-thread frame loop, pre-declared in the wiring. Chosen because events are sparse per message (human chat rate) plus ≤3 in-memory writes per frame when timing is enabled at 30 FPS — a tiny Mutex critical section is negligible and runnable on both the sync loop and the confined async worker without any `crossbeam`/`watch` dependency. Memory is bounded by construction: fixed counters (`u64`), fixed histogram arrays, and an in-flight `VecDeque` capped at 128 entries (2 × the 64-channel capacity; messages cannot exceed 64 in flight; the slack absorbs uncorrelated `None`-id variance), evicting oldest on overflow. No unbounded growth anywhere.

### 3.2 Monotonic clock seam

```rust
// diagnostics/collector.rs
pub(crate) trait MonotonicNanos { fn now_nanos(&self) -> u64; }

pub(crate) struct InstantMonotonicNanos;   // zero-sized
```

`now_nanos()` = `static GLOBAL_ANCHOR: std::sync::LazyLock<std::time::Instant>` (std, stable since 1.80; image is pinned 1.82) then `GLOBAL_ANCHOR.elapsed().as_nanos()` saturating to `u64`. REQUIRED: the anchor is process-global so tokens stamped on the worker thread are diffable on the main thread — two separate `Instant::now()` anchors would make cross-instance subtraction invalid. Persisted in-flight stamps are `u64` nanos from this one anchor; same-domain monotonic durations only (TODO clock rule §6). The summary's event rows stamp `epoch_seconds` (std `SystemTime`, wall-clock) **only in the emitted row text** — never diffed against monotonic values. `CLOCK_DOMAIN_NOTE` (in `SummaryEvent` + docs, literal text):

> `"provider published timestamps are provider-supplied wall clock (publishedAt); all *_ns values are local monotonic deltas; subtracting stamps across clock domains is not a precise duration (uncorrectable clock skew)"`

Frame-loop pacing is untouched (`FrameClock` remains the pacing seam; the collector NEVER reads the pacing clock — its own `InstantMonotonicNanos` drifts independently; interval diffs stay within the collector).

### 3.3 Event model (structs + fields; JSONL rows)

All events serialize as one-line JSON objects (serde_json — already a main dep; NO new dependency). Envelope fields per row: `event` (kind name), `epoch_seconds` (wall-clock), then kind-specific fields:

- `TimingEvent` → `event: "message"`: fields `correlation: Option<String>` (provider message id; verbatim), `provider_epoch_seconds: Option<u64>` (the message's already-carried `ChatMessage::timestamp`, clearly provider wall-clock in docs and via `CLOCK_DOMAIN_NOTE` in summaries), `stage: &str` (`"dequeued" | "rendered" | "written"`), `lag_ns: Option<u64>` (dequeue→stage; written at render/write completion), `queue_depth: Option<u64>` (at dequeue).
- `QueueEvent` → `event: "queue"`: `correlation: Option<String>`, `batch_size: Option<u64>` (received), `queue_depth: u64` (enqueued_total − dequeued_total), `backpressure: Option<bool>`, `wait_ns: Option<u64>` (enqueue wait), `backoff_ms: Option<u64>` (reconnect).
- `SummaryEvent` → `event: "summary"`: `period_seconds: u32`, `clock_note: CLOCK_DOMAIN_NOTE`, and the `DiagnosticsState::snapshot()` payload: counters (received/enqueued/dequeued/uncorrelated dequeues/full-retries/worker-retained/reconnect_count/reconnect_backoff_total_ms/in_flight_high_water), `queue_depth_estimate`, `oldest_in_flight_age_ns: Option<u64>`, and per-histogram `HistogramSummary { count, min_ns, max_ns, mean_ns, buckets: [(lower_ns, upper_ns, count); 9] }` for `message_channel_ns` (enqueue→dequeue), `message_render_ns` (dequeue→render-complete), `frame_interval_ns`, `frame_jitter_ns` (|interval − pacing interval|), `render_ns`, `write_ns`.

Never present in any row: message text, author, any token/secret, RTMPS URL, stream key (requirement 4; pinned by a roster test asserting no author-like field/`author:` substring in rows).

### 3.4 Bounded aggregation (histogram + state)

`diagnostics/histogram.rs`:

```rust
pub(crate) const HISTOGRAM_BOUNDARIES_NS: [u64; 8] =
    [1_000_000, 5_000_000, 10_000_000, 50_000_000, 100_000_000, 500_000_000, 1_000_000_000, 5_000_000_000];
// 9 fixed buckets: [0,1ms) [1,5) [5,10) [10,50) [50,100) [100,500) [500,1000) [1000,5000) [5000,∞) ms

pub(crate) struct DurationHistogram {
    count: u64, min_ns: Option<u64>, max_ns: u64, total_ns: u64,
    buckets: [u64; 9],
}
// record(ns) — saturating total; single comparisons; O(1) memory.
// snapshot() -> HistogramSummary (count/min/max/mean + bucket tuples).
```

`diagnostics/state.rs` — `DiagnosticsState` (all accessed under one Mutex):

```rust
pub(crate) struct InFlightEntry {
    correlation: String,          // provider message id
    received_nanos: u64,          // worker receipt stamp (stage 2)
    enqueued_nanos: u64,          // successful enqueue stamp (stage 3)
    backpressured: bool,          // true when at least one Full retry occurred
}
pub(crate) struct DiagnosticsState {
    received_total: u64, enqueued_total: u64, dequeued_total: u64, uncorrelated_total: u64,
    full_retry_total: u64, worker_retained_total: u64,
    reconnect_count: u64, reconnect_backoff_total_ms: u64,
    in_flight: VecDeque<InFlightEntry>,           // cap IN_FLIGHT_CAP = 128; evict oldest
    in_flight_high_water: u64,
    message_channel_ns: DurationHistogram, message_render_ns: DurationHistogram,
    frame_interval_ns: DurationHistogram, frame_jitter_ns: DurationHistogram,
    render_ns: DurationHistogram, write_ns: DurationHistogram,
}
```

Retention distinction (requirement 7): a Full retry sets `backpressured = true` on the in-flight entry (created even before the eventual success, capped by the map bound); on enqueue success with `backpressured = true`, `worker_retained_total` increments while `enqueued_total` increments for EVERY success — summaries report both, so "worker retained during backpressure" ≠ "successfully enqueued".
Queue depth = `enqueued_total − dequeued_total` (counter arithmetic; std `sync_channel` exposes no stable depth — documented honestly as an estimate); oldest in-flight age = `now − in_flight.front().received_nanos` at snapshot time.

### 3.5 Collector API and where each stage is recorded (file :: function anchors)

`DiagnosticsCollector` is `#[derive(Clone)]`, holding `settings` (extracted bool flags + file settings), `Arc<Mutex<DiagnosticsState>>`, `RefCell<CollectorLocal>` (per-handle bookkeeping: last tick nanos, last dequeued correlation, write-start stamp, last-summary stamp), and `Box<dyn MonotonicNanos>`. It is `Send` (Boxed trait is `Send`-bounded) so it lives inside `ChatWorkerPlan` on the worker thread.

Methods (each ≤ 50 lines, ≤ 2 args; disabled mode = caller guards the `Option`, so these are never invoked when capabilities are off):

| Method | Recorded at (exact anchor) | Updates |
|---|---|---|
| `report_batch_received(&self, size: u64)` | `src/app/run/chat_stream_delivery.rs :: process_batch` (live path) | `received_total += size` |
| `note_worker_receipt(&self, correlation: Option<&str>)` | `chat_stream_delivery.rs :: deliver`, top of the per-message loop | per-message monotonic receipt stamp stored in `CollectorLocal::last_receipt` (consumed by `report_enqueue_success`) |
| `report_enqueue_full(&self, correlation: Option<&str>)` | `chat_stream_delivery.rs :: send_stop_aware`, `Err(TrySendError::Full(returned))` arm | `full_retry_total += 1`; set `backpressured` for that correlation |
| `report_enqueue_success(&self, message: &ChatMessage)` | `chat_stream_delivery.rs :: send_stop_aware`, `Ok(())` arm | `enqueued_total += 1`; if backpressured → `worker_retained_total += 1`; push `InFlightEntry` (received = worker receipt stamp captured at `report_batch_received`+`deliver` loop entry), enqueue-wait histogram record + QueueEvent row |
| `report_reconnect(&self, backoff_ms: u64)` | `chat_stream_delivery.rs :: await_reestablishment` (called with the computed `backoff_delay_ms` value) | `reconnect_count`, `reconnect_backoff_total_ms` + QueueEvent row |
| `note_message_dequeued(&self, message: &ChatMessage)` | `src/app/pipeline/runner.rs :: accept_next_message`, on `Ok(Some(message))` | pop the matching in-flight entry by `message.message_id()` (linear scan ≤ 128), record `message_channel_ns` ( dequeue stamp − entry.enqueued_nanos), `dequeued_total += 1`; `None` id → `uncorrelated_total += 1` (honest: uncorrelatable latency); TimingEvent row (`stage: "dequeued"`) |
| `note_render_completed(&self)` | `runner.rs :: render_and_write_frame`, right after `render(...)` Ok | `render_ns` record (now − local render-start stamp in `CollectorLocal`); if a message was dequeued this iteration → `message_render_ns` (now − dequeue stamp) + TimingEvent row (`stage: "rendered"`) |
| `note_write_started(&self)` / `note_write_completed(&self)` | `runner.rs :: write_frame_bytes` immediately before/after `self.sink.write_frame(...)` | `write_ns` (complete − start) + TimingEvent row (`stage: "written"`) |
| `note_frame_tick(&self)` | `runner.rs :: run` loop, once per iteration right before `clock.sleep_nanos(gap_nanos)` (the runner already has `gap_nanos`/now there) | `frame_interval_ns` (now − last tick) and `frame_jitter_ns` (|interval − pacing interval| — pacing interval passed once at collector wiring as a ctor arg), NO row emission |
| `maybe_emit_summary(&self)` | runner loop, immediately after `note_frame_tick` | when `now − last_summary ≥ summary_interval()` → `snapshot()` → one SummaryEvent row |

Summary timer lives on the main frame loop (it ticks at frame cadence anyway); the worker thread NEVER runs the timer — its counters simply aggregate and appear in the next snapshot. Honest limitation (documented): the final partial interval's data is not emitted at shutdown; nothing flushes at Ctrl+C.

### 3.6 Emission sink + rotation policy (FINAL)

`diagnostics/writer.rs`:

```rust
pub(crate) struct DiagnosticsFileWriter {
    path: PathBuf,
    max_file_bytes: u32,
    max_generations: u32,
    writer: Option<std::io::BufWriter<std::fs::File>>,
    written_bytes: u64,
}
```

- `write_event(&mut self, line: &str)` (each JSON row + `'\n'`): lazy-open on first row (`OpenOptions::new().create(true).append(true)`), **explicit flush after every line** (small volume; crash-safe tailing), then the bounded-rotation check.
- **Rotation policy (selected): size-cap roll with N generations.** Before writing a row, if `written_bytes + row_len + 1 > max_file_bytes` AND `written_bytes > 0`: close the current file; if generation `max_generations − 1` exists delete it; shift generation i → i+1 for i from `max_generations − 2` down to 1 (each shift is `std::fs::rename` skipped when the source does not exist — existence checked via `Path::exists`); rename `<output_file>` → `<output_file>.1`; open a fresh `<output_file>`. Behavior guarantees: generations live at `<output_file>.1 … .<max_generations − 1>`; every generation ≤ `max_file_bytes` (a single row larger than the cap is impossible for the fixed small structs — max row size ~2 KB); **total disk bound = `max_file_bytes × max_generations` bytes, never unbounded**. Rotation I/O failures are swallowed with `let _ =` and rotation proceeds to a fresh file, because diagnostics I/O must never disturb the run (matches the poison-tolerant Mutex convention; documented in the writer module docs).
- The chat log (`chat.log_file`) is never touched — validation forbids path collision and the writer only opens `diagnostics.output_file`.
- When `output_file` is absent and some capability is enabled: rows go through `tracing::debug!(target: "diagnostics", "{}", row_json)` (bounded volume: per-message rows + periodic summary — no per-frame rows).

### 3.7 Wiring decisions (exact)

1. `run/mod.rs`: `let diagnostics = DiagnosticsCollector::from_config(start.config);` — the ONE construction point for `run`; clones shared into BOTH the worker plan and the runner (see 3.8). For `demo`, `prepared_pipeline` builds its own collector from config (single-thread; no in-flight worker entries).
2. `seams.rs`: `ChatStageSources` gains `pub(super) diagnostics: Option<DiagnosticsCollector>`.
3. `chat_worker.rs`: `ChatWorkerPlan` gains `pub(super) diagnostics: Option<DiagnosticsCollector>`; `build_chat_stage` sets `plan.diagnostics = chat.diagnostics` (the SAME collector object — both sides share the `Arc<Mutex<DiagnosticsState>>`; the worker plan receives a clone of the same collector that rides `StreamingExecution`, so a queue event recorded from the worker thread is visible to the summary timer on the frame loop) — signature (3 args, sanctioned family) unchanged; the return tuple stays a 2-tuple.
4. `streaming.rs`: `StreamingExecution` gains `pub(super) diagnostics: Option<DiagnosticsCollector>`; `PreparedStreaming::run` copies it into `StreamParts::diagnostics`.
5. `run/mod.rs::run_runtime`: constructs the collector once; `ChatStageSources { ..., diagnostics: diagnostics.clone() }`; `StreamingExecution { ..., diagnostics }`.
6. `runner.rs` (`PipelineParts` / `PipelineConstruction` unchanged; `StreamParts` gains `pub(crate) diagnostics: Option<DiagnosticsCollector>`): both `prepared_pipeline` (demo) and `prepared_stream_pipeline` (run) populate `parts.diagnostics` — demo from `DiagnosticsCollector::from_config(config)`, run from `StreamParts`.
7. `src/chat/model.rs`: add `pub(crate) fn message_id(&self) -> Option<&str>` (correlation-key accessor; Display untouched).

Cap-driven splits (all pre-decided, see §10):
- `run/mod.rs` (199 lines) — extract `install_stop_pair` (+ its doc) into `src/app/run/stop_pair.rs`, registered from `mod.rs` as `#[path = "stop_pair.rs"] mod stop_pair;` with `pub(super) fn install_stop_pair`; `run/mod.rs` calls it as `stop_pair::install_stop_pair()`. Rationale: run/mod.rs must also add the collector construction + wiring (~5 lines); the extraction frees ~16 lines.
- `chat_worker.rs` (199 lines) — extract `ChatChannel` + `chat_channel()` + `chat_runtime()` into `src/app/run/chat_channel.rs`, registered `#[path = "chat_channel.rs"] mod chat_channel;` from chat_worker.rs with a `use chat_channel::{chat_channel, chat_runtime};` re-import so existing call sites are unchanged; keep `pub(super)` visibility identical.
- `chat_stream_delivery.rs` (195 lines) — extract the helper block (`sleep_in_stop_slices`, `backoff_delay_ms` WITH its pre-existing `#[allow(clippy::manual_clamp)]` — relocation, not a new allow, `sleep_slice` cfg pair, `record_terminal`, `transport_terminal`, `CHAT_WORKER_PANIC_DETAIL`) into `src/app/run/chat_worker_helpers.rs`, registered `#[path = "chat_worker_helpers.rs"] pub(super) mod chat_worker_helpers;`? NO — registered as a plain child `mod chat_worker_helpers;` from delivery, with `pub(super) use chat_worker_helpers::{record_terminal, transport_terminal, CHAT_WORKER_PANIC_DETAIL};` at the top of delivery so `chat_worker.rs`'s existing `use super::chat_stream_delivery::{record_terminal, transport_terminal, CHAT_WORKER_PANIC_DETAIL};` (line 33) stays byte-identical. `backoff_delay_ms`/`sleep_in_stop_slices` move with `pub(super)` visibility; if any test imports them via `super::chat_stream_delivery::...`, update that import path to the helper module (grep `backoff_delay_ms`, `sleep_in_stop_slices` in `src/app/run/*_tests.rs` at implementation time).
- `runner.rs` (199 lines) — extract lines 140–198 (`prepared_pipeline`, `prepared_stream_pipeline`, `build_chat_store`, `build_renderer`, and the module docs above them) into `src/app/pipeline/runner_prepare.rs`, registered from `pipeline/mod.rs` (the AMENDMENT-2 precedent location, keeping runner.rs private-typed): `#[cfg(test)]`-free `mod runner_prepare;` — wait, `prepared_pipeline`/`prepared_stream_pipeline`/`StreamParts` construction DO need access to `FramePipeline` internals? No — they only call `FramePipeline::new(PipelineParts { ... })` (pub(crate)) and build field aggregates. So register from `pipeline/mod.rs` and update its re-export line to `pub(crate) use runner_prepare::{prepared_pipeline, prepared_stream_pipeline};` while keeping `pub(crate) use runner::{PipelineConstruction, StreamParts};`. Runner-diagnostic HOOK impls (inherent methods on `FramePipeline`) need PRIVATE field access → they live in `src/app/pipeline/runner_diagnostics.rs` registered as a CHILD of `runner.rs` (`#[path = "runner_diagnostics.rs"] mod runner_diagnostics;` at runner.rs top): a descendant module sees the parent's private fields, and an inherent `impl<S: FrameSink> FramePipeline<S>` block in a descendant module is legal (same crate). Runner.rs then shrinks to ~160 and admits ~15 lines of hook call sites.

### 3.8 Hook call sites (verbatim shape; small edits)

In `runner.rs`:

```rust
// in run(), loop body, before sleep:
    self.note_frame_tick();
    self.maybe_emit_summary();
// in accept_next_message(), after Ok(Some(message)):
    if let Some(diagnostics) = &self.diagnostics { diagnostics.note_message_dequeued(&message); }
// in render_and_write_frame(), after render Ok:
    if let Some(diagnostics) = &self.diagnostics { diagnostics.note_render_completed(); }
// in write_frame_bytes(), around the write:
    if let Some(diagnostics) = &self.diagnostics { diagnostics.note_write_started(); }
    ...write...
    if let Some(diagnostics) = &self.diagnostics { diagnostics.note_write_completed(); }
```

All guards are single-section booleans (`if let Some(...)`); when `self.diagnostics` is `None` the branch is one checked `Option` read — that is the ENTIRE disabled-mode cost (§4). The `FramePipeline` struct gains `diagnostics: Option<DiagnosticsCollector>` (private field); `PipelineParts` gains the same `pub(crate)` field.

In `chat_stream_delivery.rs` (`send_stop_aware` loop):

```rust
    if let Some(diagnostics) = &self.plan.diagnostics {
        diagnostics.report_enqueue_full(message.message_id());   // Full arm
    }
    match self.plan.sender.try_send(message) {
        Ok(()) => { if let Some(diagnostics) = &self.plan.diagnostics { diagnostics.report_enqueue_success(&message); } return; }
```

…shape adjusted so borrow rules hold (the implementer may hoist the `Option` clone into a local — pre-approved local detail). Receipt anchor: `deliver`'s loop records the worker receipt stamp inside `report_batch_received`-adjacent per-message stamp: FINAL implementation shape: `send_stop_aware` takes the message AFTER the C-1 conversion; the collector stamps receipt at the top of the `deliver` iteration via `collector.note_worker_receipt(correlation: Option<&str>)` (new pub(crate) method storing stamp in an in-flight-pending slot keyed by correlation in state? NO — simpler FINAL: the stamp used for the enqueue-wait is stamped by `note_worker_receipt` into `CollectorLocal` keyed on the LAST correlation, and `report_enqueue_success` uses it; with a single worker thread and sequential sends this is correct). Splitting hairs — FINAL simple contract:
- `note_worker_receipt(&self, correlation: Option<&str>)` — called at the top of `deliver`'s per-message loop; records `received_total += 1` and stores `(correlation_owned, receipt_nanos)` in `CollectorLocal::last_receipt`.
- `report_enqueue_full` / `report_enqueue_success` read `last_receipt` for the wait computation and clear it on success. Batch semantics: `report_batch_received(size)` is the RECEIVED counter at batch level; `note_worker_receipt` per live message is the per-message stamp. Both are recorded (batch size for per-interval received metric; receipt stamp per message) — the collector methods stay ≤ 2 args and ≤ 50 lines.

Reconnect anchor (`await_reestablishment`): after computing `let delay = backoff_delay_ms(...)` and BEFORE the wait: `if let Some(diagnostics) = &self.plan.diagnostics { diagnostics.report_reconnect(delay); }`.

---

## 4. Disabled-by-Default Guarantee

Mechanism (three layers, all test-pinned):

1. `DiagnosticsCollector::from_config(&AppConfig) -> Option<DiagnosticsCollector>` returns `None` unless the single named predicate `is_any_capability_enabled` (any of `pipeline_timing_events`, `queue_metrics`, `periodic_summary` true) holds. `None` propagates: no `Arc<Mutex<DiagnosticsState>>` is ever allocated, no `Box<dyn MonotonicNanos>`, no `BufWriter`, no file path touched.
2. Every call site is `if let Some(diagnostics) = ...` — when `None`: no clock read, no formatting, no lock, no I/O. There are no unconditional `format!`/`Instant::now()` calls on the hot path.
3. `output_file` presence alone does NOT enable anything: capabilities gate the writer, so `auth`, `demo`, and `run` with every flag falsy and even `output_file` configured create ZERO artifacts and behave byte-identically to today (requirement 11 + acceptance criterion 1).

Unavoidable minimal baseline overhead (documented in `docs/production-run.md` + collector module docs, literal sentence for the docs pass):

> "With every diagnostics feature disabled, the instrumentation costs one `Option`-check per instrumentation site per frame loop iteration (a handful of branch instructions at 30 FPS), one config-field read once at startup, and nothing else — no clock reads, no allocations, no Mutex locks, no file I/O, and no other clock, alloc or I/O call is made."

---

## 5. Clock / Correlation Decisions

- Monotonic: std `Instant` via the process-global `LazyLock<Instant>` anchor (`InstantMonotonicNanos`); ALL `*_ns` durations are same-domain monotonic deltas on that anchor.
- Wall-clock: (a) provider `publishedAt` — already parsed and carried (`ChatMessage::timestamp`, epoch seconds), surfaced in timing rows as `provider_epoch_seconds` and labeled provider-supplied wall-clock; (b) `epoch_seconds` per emitted row (std `SystemTime`) for operator correlation across tools. Cross-domain subtraction is NEVER presented as a precise duration: no code computes provider-epoch − row-epoch as "latency"; the caveat text (`CLOCK_DOMAIN_NOTE`) is embedded in every SummaryEvent row and quoted in `docs/production-run.md`.
- Correlation flow without touching Display: provider message id travels inside `ChatMessage` (`message_id`, already attached by `into_chat_message`); the new `message_id()` accessor only READS it. `Display` stays `author: text` (model_tests pin). Demo ids (`demo-NN`) appear as correlations in demo-enabled diagnostics — non-secret, pinned harmless.
- Secret-safety: no message text, authors, OAuth material, RTMPS URLs, or stream keys are ever in `CollectorLocal`, `DiagnosticsState`, event structs, or the writer (requirement 4 + acceptance criterion 4; pinned by roster tests + grep in the verification step).

---

## 6. Test Plan (zero sleep, deterministic injected values)

New rosters (all `#[cfg(test)] #[path = ...]` footer-registered):

**Config (2 new files + fixture/loader edits):**

| File | Tests (names) | Pins |
|---|---|---|
| `src/config/diagnostics_parse_tests.rs` (registered from loader.rs) | `loads_example_diagnostics_section` · `diagnostics_section_absent_parses_to_none` · `diagnostics_fields_absent_parse_to_none` · `enabled_flags_parse_to_true` · `unknown_diagnostics_field_rejected` · `wrong_type_flag_rejected` | serde schema shape, defaults as `None`, unknown/typed rejects against temp-file JSON + the extracted example mirror |
| `src/config/validation_diagnostics_tests.rs` (registered from `validation_diagnostics.rs`) | `blank_output_file_reports_issue` · `output_file_must_differ_from_chat_log` · `nonpositive_summary_interval_reports_issue` · `nonpositive_max_file_bytes_reports_issue` · `nonpositive_max_generations_reports_issue` · `generation_count_over_limit_reports_issue` · `section_absent_passes_every_rule` | the six new rules + Section-absent neutrality; error Display contains the field path |

Fixture/loader work BEFORE rosters compile (exact order):
1. Move `example_json()`, `example_json_value()`, `write_temp_config()`, `expect_load_error()` from `loader_tests.rs` into `src/config/test_fixtures.rs` as `pub(crate)` (updating the two existing parse rosters' imports — grep `example_json(` in `src/config/`). `loader_tests.rs` drops to ~150 lines, then gains the diagnostics block inside the (now-shared) `example_json()` mirror (`loader_tests.rs` stays under the cap — the mirror gains the new section AFTER the extraction; the roster does NOT split first).
2. `test_fixtures.rs::valid_config()` gains `diagnostics: Some(DiagnosticsConfig { pipeline_timing_events: Some(false), queue_metrics: Some(false), periodic_summary: Some(false), summary_interval_seconds: Some(30), output_file: Some("logs/diagnostics.log"), max_file_bytes: Some(1_048_576), max_generations: Some(3) })` — fixture mirrors example.
3. `loads_valid_example_config` auto-covers the mirror equality via `valid_config()`.

**Diagnostics module rosters (5 new files):**

| File | Tests | Pins |
|---|---|---|
| `histogram_tests.rs` | `records_count_min_max_mean` · `buckets_fit_fixed_ladder` · `over_5s_lands_in_last_bucket` · `total_saturates_without_panic` | aggregation, boundaries, saturation |
| `state_tests.rs` | `inflight_pop_matches_correlation` · `inflight_evicts_oldest_at_cap` (pump 129 entries) · `backpressured_message_counts_as_retained_and_enqueued` · `high_water_tracks_peak_depth` · `oldest_age_reads_front_stamp` · `queue_depth_is_enqueued_minus_dequeued` | requirement 7 set, bounded memory |
| `collector_tests.rs` (uses `diagnostics_test_fixtures.rs::FixedMonotonicNanos { shared_cell }`) | `from_config_disabled_returns_none_for_absent_section` · `from_config_disabled_returns_none_for_false_flags` · `from_config_disabled_ignores_output_file` (the no-artifact guarantee with the file configured) · `from_config_enabled_returns_some` · `stages_propagate_receive_enqueue_dequeue_render_write` (fixed clock advanced by hand between calls; asserts histogram values equality) · `uncorrelated_message_is_counted_not_dropped` · `emits_summary_after_interval_and_not_earlier` (clock-steps) · `rows_never_carry_message_text_or_author` | deterministic-clock injection seam = the `MonotonicNanos` trait + `from_config` vs the test ctor `DiagnosticsCollector::for_tests(settings, core, clock)` (pub(crate), cfg(test)-documented use) |
| `writer_tests.rs` (tempfile) | `first_row_creates_file_only_on_demand` · `rotation_shifts_oldest_generation_out` · `generations_stay_within_disk_bound` · `file_keeps_distinct_path_from_chat_log` (never opens the chat-log path) | rotation generations, bounded disk |
| `diagnostics_test_fixtures.rs` | no `#[test]` | `FixedMonotonicNanos` (shared mutable nanos), `enabled_collector_for_tests()`, `empty_core()` helpers |

**Pipeline / worker integration rosters (3 new files):**

| File | Tests | Pins |
|---|---|---|
| `src/app/pipeline/runner_diagnostics_tests.rs` (registered from `runner_diagnostics.rs` or mod.rs — implementer attaches at the sibling registration point used by that file) | `pipeline_records_stage_cycle_for_one_message` (existing `PipelineParts` harness + `FakeClock`/`RecordingSink` from `test_helpers.rs`; shared `Core` asserts dequeued=1, render/write counts=1; NO sleeps — `test_helpers` pacing is already sleep-free) · `pipeline_with_none_collector_does_no_io_and_no_state` (recording sink + None → no diagnostics file exists in the temp dir) | end-to-end enabled/disabled behavior through the REAL loop |
| `src/app/run/chat_delivery_diagnostics_tests.rs` (registered from `chat_stream_delivery.rs` footer) | `worker_reports_batch_received_and_enqueue_success` · `worker_marks_backpressured_on_full_channel` (uses the existing burst/ledger fakes pattern from `runtime_chat_fakes.rs`/`chat_worker_burst_tests.rs`) · `worker_reports_reconnect_with_backoff` · `terminal_helpers_still_exported_from_delivery` (compiles the re-export path) | worker-side events + re-export stability |
| (existing) | see update list below | — |

**Existing tests/artifacts to update (grep first, edit precisely):**
- `src/app/pipeline/test_helpers.rs` / `runner_tests.rs` / any `PipelineParts {` literal: add `diagnostics: None`.
- `StreamingExecution {` literals (e.g. `runtime_runner_fixture.rs`): add `diagnostics: None`.
- `ChatWorkerPlan {` literals (e.g. `gate_chat_tests.rs`, `runtime_chat_fakes.rs` call sites): add `diagnostics: None`.
- `AppConfig {` literals outside `test_fixtures.rs` (grep the whole `src/`): add `diagnostics: None` — expected: none or few; fixture-based tests need none.
- `chat_worker_budget_tests.rs` if it imports `backoff_delay_ms` via the old path: repoint to `chat_worker_helpers`.
- `src/chat/model_tests.rs`: add one small `message_id_accessor_returns_attached_id` test (Display assertions untouched).

Zero-sleep rule: all new tests drive monotonic values through injected clocks/injected stamps; rotation tests use tempfile + explicit byte counts; NO `thread::sleep`, no real `Instant` waits anywhere in the new tests.

---

## 7. Docs Update List (content outlines; the docs SPECIALIST lands these at 4.4, committed per §8)

1. **`docs/configuration.md`**:
   - New `### diagnostics` subsection under Field Reference (after `### ffmpeg`), one bullet per field with type/default/example, including the literal semantics constraints:
     - "`diagnostics.output_file` (string, optional) — structured JSONL diagnostics output; must be non-blank and different from `chat.log_file` (the diagnostics file stays strictly separate from the ordinary chat log). When absent, enabled diagnostics are emitted as tracing `debug` events (visible only with `RUST_LOG` at `debug` or higher)."
     - "Every diagnostics feature is disabled by default. With all four features false/absent, no diagnostics state is built, no `RUST_LOG` row is formatted, and no diagnostics file is ever created — even when `output_file` is configured."
   - Validation Rules section: "21 rules" → "27 rules", extending the semantic list with the six diagnostics rules verbatim (positive interval/max-file/max-generations, generations ≤ 1000, non-blank output file, output file ≠ chat log).
2. **`docs/production-run.md`**: new H2 `## Pipeline Observability (Opt-In Diagnostics)` placed between `## Known Stream-Health Limitation (Pure-Black MVP Content)` and `## Manual Verification Procedure (Operator-Run)`, with a matching TOC entry. Outline (6 H3s): What It Measures (the six stage list + frame metrics); Enabling It (config snippet identical to the example section); Diagnostic Output (JSONL file location, row kinds, rotation policy + the literal disk bound sentence: "total diagnostics disk usage never exceeds `max_file_bytes × max_generations` bytes"); What Diagnostics Never Contain (literal: "diagnostic output never includes message text, author names, access or refresh tokens, OAuth client secrets, RTMPS URLs, or stream keys; the only per-message correlation value is the provider's message identifier"); Clock Domains (the `CLOCK_DOMAIN_NOTE` sentence verbatim); Baseline Overhead When Disabled (the §4 sentence verbatim). Plus one honest limitation line: the final partial summary interval is not emitted at shutdown, and viewer-side latency remains an external operator measurement (placeholder pointer for Task 2's procedure).
3. **`README.md`**: one pointer sentence at the end of the Production Run section linking `docs/production-run.md#pipeline-observability-opt-in-diagnostics`; the test-count claim (line 47) is refreshed to the OBSERVED gate count at the 4.4/4.6 pass (4.2 records the observed count in its report context instead of editing the README mid-task — caller decides at 4.6; the plan does NOT edit README counts before the gate evidences them).

---

## 8. Git Boundaries — 4.2 Commit Sequence

On `feat/observability-game-handoff` only; push NEVER happens in 4.2 (step 5 owns merge/push). `git status` is checked before each commit; `.gitignore` compliance verified (nothing staged from `logs/`, `target/`, `dist/`, `config/config.json`).

1. `chore: track Task-1 observability implementation plan` — `.kilo/plans/20261101-observability-task1-plan.md` (this file; skip if the caller already tracked it).
2. `feat(config): add opt-in diagnostics section with strict validation` — `src/config/{model.rs, mod.rs, diagnostics_config.rs, validation.rs, validation_diagnostics.rs, loader.rs, test_fixtures.rs, loader_tests.rs, diagnostics_parse_tests.rs, validation_diagnostics_tests.rs}`, `config/config.example.json`.
3. `feat(diagnostics): bounded collector, histograms, and rotated JSONL diagnostics file` — `src/app/mod.rs`, `src/app/diagnostics/**` (6 sources + 5 rosters/fixtures), `src/chat/model.rs` (accessor + its model_tests line).
4. `feat(run): wire diagnostics through chat worker, delivery, and frame loop` — `src/app/run/{mod.rs, stop_pair.rs, seams.rs, streaming.rs, chat_worker.rs, chat_channel.rs, chat_stream_delivery.rs, chat_worker_helpers.rs, chat_delivery_diagnostics_tests.rs}`, `src/app/pipeline/{mod.rs, runner.rs, runner_diagnostics.rs, runner_diagnostics_tests.rs, runner_prepare.rs}`, and the existing-roster updates listed in §6.
5. `test: pin diagnostics disabled-mode, rotation, and stage propagation` — ONLY if any test-only edits were deferred out of commits 2–4; otherwise folded and skipped.
6. `docs: document diagnostics configuration and operator observability guide` — `docs/configuration.md`, `docs/production-run.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/context.md` (dated Task-1 4.2 block) — lands after the 4.4 docs pass per the caller-ordered docs-commit precedent.
7. Closing dev-checks gate log path recorded (gitignored `logs/checks/`) — never committed.

---

## 9. Byte-Stability Ledger

| Surface | Guarded by |
|---|---|
| `args.rs` local vector / `stream_args.rs` locked 33/42-token streaming vectors (incl. CBR audio shapes) | untouched; pinned by `stream_args_tests.rs`, `stream_args_health_tests.rs`, `streaming_stage_tests.rs` (argv capture) |
| `run` shutdown ordering + exit mapping (0 clean / 0 pre-streaming / 1 fatal · single `error:` funnel) | untouched — only additive field pass-throughs; pinned by `runtime_tests.rs` (shutdown-order proof), `run/mod_tests.rs`, `error_tests.rs` |
| `ChatMessage` Display `“author: text”` | accessor-only change; pinned by `model_tests.rs` display tests |
| Secret redaction (`RtmpsIngestion`/`RtmpsDestination`, `[redacted]` stderr tail, capture-time redaction in `finish()`) | untouched + diagnostics rows asserted secret-free (`rows_never_carry_message_text_or_author`, writer content checks); redaction pins unchanged (`streaming_diagnostics_tests.rs`, `ffmpeg_stream_tests.rs`) |
| Demo timeline (28 messages / 496 frames) and demo exit contracts | untouched (hooks are `Option` no-ops even when enabled — timeline code unmodified); pinned by `schedule_tests.rs`, `source_tests.rs`, `mod_tests.rs` (demo) |
| Bounded 64-entry channel, stop-slices, backoff/terminal machinery semantics | capacity constants untouched; delivery edits are additive `if let Some` guards; pinned by `chat_worker_burst_tests.rs` (retain-retry), `chat_worker_budget_tests.rs`, `chat_worker_tests.rs`/`chat_worker_flow_tests.rs` |
| Gate (200-line file) + confirmation budget + watch link | zero edits to `gate.rs`; pinned by existing `gate_tests.rs` |
| Config validation Display/error surface for pre-existing 21 rules | new rules are ADDITIVE predicates with `diagnostics.*` paths; pinned by existing `validation_tests.rs`, `loader_tests.rs` + new roster's "section_absent_passes_every_rule" |
| Zero new `#[allow(` attributes | the only relocated allow is the PRE-EXISTING `#[allow(clippy::manual_clamp)]` inside `backoff_delay_ms` (moved into `chat_worker_helpers.rs` — a relocation, contract-annotated); if the clippy gate surfaces a NEW compiler-arbitrated need at implementation time, implementer must STOP and report (caller adjudicates) | |

---

## 10. Line-Budget Table + STOP Protocol

Projected post-formatting line counts (cap 200; exclude-test rosters noted where applicable):

| File | Current | Projected |
|---|---|---|
| `src/config/model.rs` | 176 | ≤ 195 |
| `src/config/mod.rs` | 33 | ≤ 38 |
| `src/config/diagnostics_config.rs` (new) | — | ≤ 140 |
| `src/config/validation.rs` | 171 | ≤ 178 |
| `src/config/validation_diagnostics.rs` (new) | — | ≤ 110 |
| `src/config/validation_diagnostics_tests.rs` (new) | — | ≤ 170 |
| `src/config/loader.rs` | 51 | ≤ 64 |
| `src/config/test_fixtures.rs` | 54 | ≤ 130 |
| `src/config/loader_tests.rs` | 200 | ≤ 160 |
| `src/config/diagnostics_parse_tests.rs` (new) | — | ≤ 160 |
| `src/chat/model.rs` | 81 | ≤ 88 |
| `src/app/diagnostics/mod.rs` (new) | — | ≤ 60 |
| `src/app/diagnostics/events.rs` (new) | — | ≤ 120 |
| `src/app/diagnostics/histogram.rs` (new) | — | ≤ 100 |
| `src/app/diagnostics/state.rs` (new) | — | ≤ 175 |
| `src/app/diagnostics/collector.rs` (new) | — | ≤ 185 |
| `src/app/diagnostics/writer.rs` (new) | — | ≤ 125 |
| 5 diagnostics test files (new) | — | ≤ 170 each |
| `src/app/pipeline/runner.rs` | 199 | ≤ 185 |
| `src/app/pipeline/runner_prepare.rs` (new) | — | ≤ 75 |
| `src/app/pipeline/runner_diagnostics.rs` (new) | — | ≤ 85 |
| `src/app/pipeline/runner_diagnostics_tests.rs` (new) | — | ≤ 150 |
| `src/app/pipeline/mod.rs` | 57 | ≤ 61 |
| `src/app/run/mod.rs` | 199 | ≤ 192 |
| `src/app/run/stop_pair.rs` (new) | — | ≤ 35 |
| `src/app/run/seams.rs` | 70 | ≤ 80 |
| `src/app/run/streaming.rs` | 194 | ≤ 199 (TIGHT — mark) |
| `src/app/run/chat_worker.rs` | 199 | ≤ 192 |
| `src/app/run/chat_channel.rs` (new) | — | ≤ 55 |
| `src/app/run/chat_stream_delivery.rs` | 195 | ≤ 185 |
| `src/app/run/chat_worker_helpers.rs` (new) | — | ≤ 80 |
| `src/app/run/chat_delivery_diagnostics_tests.rs` (new) | — | ≤ 160 |

**STOP protocol (binding):** if any file's post-`cargo fmt` count exceeds 200 (or the mark on `streaming.rs` fires: run `wc -l` inside the container *before* committing), the implementer STOPS immediately, reports the exact `wc -l` counts for every touched file, and waits for caller adjudication of the split (the `stream_args`/`ffmpeg_real_audio` precedents). Never self-stage a split, never disable a lint, never over-trim comments to sneak under a cap.

---

## 11. Verification Commands

1. `alpine-vm_vm_status` FIRST (call it every time before any compose command).
2. Standard gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — expected exit 0, four PASS lines, `ALL CHECKS PASSED`; log path under gitignored `logs/checks/`.
3. Targeted spot runs (each: same compose wrapper, `cargo test --locked <filter>` exit 0):
   - `cargo test --locked diagnostics` (new config + module rosters)
   - `cargo test --locked validation_diagnostics`
   - `cargo test --locked runner` / `cargo test --locked chat_delivery_diagnostics`
   - `cargo test --locked chat_worker` + `cargo test --locked chat_worker_helpers` (relocation + re-export stability)
4. Baseline honesty: the pre-TODO OBSERVED gate is **497 passed / 0 failed** (README line 47 + context.md). Task 1 adds `~35–45` new tests; **the OBSERVED count from the run governs** — record it verbatim in the completion report; do NOT hand-arithmetize a "7xx expected".
5. Hygiene gate before each commit: `git status --porcelain` exactly equal to the intended file set; `git check-ignore` for any produced log; grep audit: `ya29|AIza|key=|rtmps://` counts relative to HEAD must be unchanged in new files; `#[allow(` count in `src/` must be HEAD-count + ZERO new (the relocated `manual_clamp` allow is net-zero).

---

## 12. Traceability — TODO Task-1 requirements → plan sections

| TODO requirement (pp. 32–55) | Plan section |
|---|---|
| Review chat worker / handoff / frame loop / renderer / FFmpeg writer / config model / logging init / production docs before choosing points | §1 (all files read; anchors named per function) |
| Dedicated diagnostics config section; every feature explicitly configurable and disabled by default; strict validation conventions; example + docs updated | §2 (2.1/2.2/2.3/2.4), §7.1 |
| Independently switchable: timing events / periodic summaries / queue diagnostics / separate structured file | §2.1 flags; §3.6 sink; §3.5 (aggregation vs emission split) |
| Per-message timing/correlation without message text/secrets/PII; provider id as key | §3.3 (correlation field), §3.4, §5, §9 (display/redaction rows) |
| Six stage timestamps (publishedAt / streamList receipt / enqueue / dequeue / render / FFmpeg write) | §1.1 mapping table + §3.5 anchors |
| Monotonic for local durations; wall-clock only where needed; never present cross-domain subtraction as precise | §3.2, §5 (CLOCK_DOMAIN_NOTE literal) |
| Queue depth, high-water, enqueue wait/failure/full events, oldest age, received/rendered per interval, reconnect count/duration; retained vs enqueued distinction | §3.4 state fields + §3.5 methods |
| Frame pipeline measurements (interval/jitter, render/write durations), no per-frame log lines | §3.5 `note_frame_tick` (aggregation-only) + §3.3 |
| Bounded aggregation (count/min/max/mean + buckets), bounded memory | §3.4 histogram; §3.1 in-flight cap; §3.6 disk bound |
| Diagnostics file: configurable location, separate from chat log, bounded rotation | §2.1 `output_file` + rule 27; §3.6 rotation |
| Disabling all options ⇒ no file, no high-frequency formatting; baseline overhead documented | §4 (three-layer guarantee + verbatim sentence) |
| No metrics server/dashboard/external telemetry/new deps; prefer tracing + std | §3.1 (std Mutex + tracing debug sink + serde_json only) |
| Unit tests: timestamp propagation, durations, aggregation, defaults/validation, disabled-mode; deterministic clocks; no sleeps | §6 (roster table, injection seams, zero-sleep rule) |
| Preserve error redaction + one-line CLI contracts | §9 ledger + §11 hygiene gate |

Acceptance criteria (pp. 59–64): disabled ⇒ documented behavior + no artifact (§4.3, pinned by `from_config_disabled_*` + `pipeline_with_none_collector_*`); enabled ⇒ correlated data separating delivery/worker delay, channel backlog, frame/render delay, FFmpeg write delay (§3.3–3.5 event + histogram coverage); bounded memory + bounded disk (§3.1, §3.6); no chat body/tokens/… in output (§3.3 never-fields + §6 pins); automated tests cover enabled/disabled/invalid/aggregation/backpressure (§6); gates pass (§11).

---

## 13. Execution Order for the Implementer (summary)

1. Config layer (§2 + fixture extraction + rosters) → targeted config tests.
2. Diagnostics module (§3.1–3.6 + rosters) → targeted module tests.
3. Wiring layer (§3.7 splits + hooks + §3.8 call sites + existing-roster updates) → targeted pipeline/worker tests.
4. Full gate (§11) → any STOP adjudications.
5. Commit per §8 (1–5); docs content handed to 4.4.

JSON note: this plan file was written with real newlines throughout (rule: no literal `\n` content).
