# Latency, Throughput, and Backpressure Test Map

The canonical characterization page for the application's latency and backpressure behavior: the offline deterministic test map that reproduces each pipeline-stage boundary over the real bounded chat queue and the real frame loop, plus the operator-run live-latency procedure that completes the end-to-end picture. This page links the operator setup and field reference in [`docs/production-run.md`](production-run.md) and [`docs/configuration.md`](configuration.md) instead of repeating them.

## Table of Contents

- [Purpose and Contract](#purpose-and-contract)
- [One Message, Boundary by Boundary](#one-message-boundary-by-boundary)
- [Capacity Math (Config-Driven, Never Hardcoded)](#capacity-math-config-driven-never-hardcoded)
- [Offline Deterministic Scenario Table](#offline-deterministic-scenario-table)
- [Harness Mechanics and the Frame-Granularity Retry](#harness-mechanics-and-the-frame-granularity-retry)
- [Operator Live Test Procedure (Operator-Run Only)](#operator-live-test-procedure-operator-run-only)
- [Results Template](#results-template)
- [Interpretation Matrix](#interpretation-matrix)
- [Threshold Stance](#threshold-stance)
- [Offline Verification Reminder](#offline-verification-reminder)
- [See Also](#see-also)

## Purpose and Contract

This map turns the opt-in diagnostics shipped in the observability work (the `## Pipeline Observability (Opt-In Diagnostics)` section of [`docs/production-run.md`](production-run.md#pipeline-observability-opt-in-diagnostics)) into a reproducible characterization of where delay and backlog form inside the chat → frame → FFmpeg pipeline — and, just as deliberately, where the application cannot see. Three sentences bound every claim on this page:

- The observed ~35-second gap between sending a chat message and seeing it in the player is a **measurement to investigate**, stage by stage — never proof of a specific bottleneck, and never a platform guarantee or constant.
- **SYNTHETIC pipeline measurements are application-internal only and MUST NOT be read as YouTube server-side delivery or viewer-buffering behavior.** The offline harness drives the real channel and the real frame loop on fake clocks; it proves stage anchors, propagation, counts, verdicts, and accounting — not absolute latency magnitudes and not any end-to-end figure.
- **Live latency claims are operator-run only.** Every automated check here is offline and no test contacts Google; the live end-to-end column is entirely operator-executed and operator-recorded (see [Operator Live Test Procedure](#operator-live-test-procedure-operator-run-only)).

The verbatim limit recorded in the diagnostics summary itself (the `CLOCK_DOMAIN_NOTE` embedded in every summary row) applies to all numbers below:

> "provider published timestamps are provider-supplied wall clock (publishedAt); all *_ns values are local monotonic deltas; subtracting stamps across clock domains is not a precise duration (uncorrectable clock skew)"

## One Message, Boundary by Boundary

One message travels a fixed boundary chain from provider publication to viewer playback. For each boundary the table lists the application-measured diagnostic metric (the exact `SummarySnapshot` counter/histogram name, or the queue/timing row field) and whether an external observer is required to measure that leg at all.

| Boundary stage | Production anchor | App-measured diagnostic metric | Needs an external observer? |
| --- | --- | --- | --- |
| Receipt (worker batch) | `src/app/run/chat_stream_delivery.rs :: process_batch` → `report_batch_received` | `received_total` counter; queue row `batch_size` | No — receipt into the app is measured directly |
| Enqueue (worker → channel) | `send_stop_aware` `Ok` arm → `report_enqueue_success` | `enqueued_total` counter; `queue_depth_estimate`; queue row `wait_ns` (receipt→enqueue) | No |
| Dequeue (frame loop pull) | `src/app/pipeline/runner.rs :: accept_next_message` via `run/source.rs :: ChatSource` | `dequeued_total` counter; `message_channel_ns` histogram (enqueue→dequeue) | No |
| Render (frame loop) | `runner.rs :: render_and_write_frame` render hooks | `render_ns` histogram; `message_render_ns` histogram (dequeue→render) | No |
| FFmpeg-write (stdin) | `runner.rs :: write_frame_bytes` write hooks | `write_ns` histogram | No — only the app's own stdin write is measured |

Backpressure and queue-shape metrics ride alongside: `in_flight_high_water` (queue high-water mark), `queue_depth_estimate` (enqueued − dequeued, a counter-derived figure — the bounded channel exposes no live depth read), `oldest_in_flight_age_ns` (`Option<u64>`, the oldest queued message's age at snapshot time), `full_retry_total` and `worker_retained_total` (distinguishing a message retained by the worker during backpressure from one successfully enqueued), and `reconnect_count` / `reconnect_backoff_total_ms` for stream drops.

Provider-side publication (`publishedAt`, carried on the message) and the two external legs are different in kind:

- Upstream delivery (provider send → this app's batch receipt) and viewer-side end-to-end (app FFmpeg write → first visible on the watch page) **require an external observer** — the app cannot see when YouTube published a message nor when a viewer's player displays the frame. The provider `publishedAt` stamp is wall-clock upstream and is never diffed against the app's monotonic durations (the `CLOCK_DOMAIN_NOTE` above).
- FFmpeg's internal encode and YouTube's live ingest are outside the app's measurement surface: the app measures only its own stdin write (`write_ns`); everything after the pipe belongs to the external-process and viewer legs.

## Capacity Math (Config-Driven, Never Hardcoded)

All rates derive from configuration — the configured FPS is the only source of the frame rate, and the channel capacity is the one constant in the code, never a literal repeated in tests or docs.

- **Pacing interval** = `1_000_000_000 / fps` nanoseconds, via `PipelinePacing::from_config(&config.video)` from the configured `video.fps`. At the repository's example/fixture `fps: 30` this is `33_333_333` ns, but that number is always an *expression of the configured field* — never a hardcoded 30-FPS assumption (pin `scenario_pacing_interval_derives_from_configured_fps`). Because `1_000_000_000` is not divisible by 30, the derivation is the integer floor (`interval == 1e9 / fps`), and the pin guards the identity `interval * fps + fps > 1_000_000_000` rather than an exact-equality that would be false for 30.
- **Nominal consumer capacity = ONE message per frame.** The current design consumes at most one queued chat message per frame iteration (`runner.rs :: accept_next_message` pulls once and accepts at most one), so nominal consumption capacity is `fps` messages per second — 30/s at the fixture FPS, whatever FPS is configured in production.
- **Channel capacity = 64**, from `chat_worker::CHAT_CHANNEL_CAPACITY` (the single source of the number; `std::sync::mpsc::sync_channel(64)`). Memory stays bounded because the channel never overflows and the sender-side retain shifts the backlog to the (bounded) worker stall rather than growing memory.
- **Rational-rate profiles.** The harness expresses input rate as `RateProfile { numerator, denominator }` = `numerator/denominator` messages per frame, using integer credit arithmetic only (no floating point anywhere). Sustained-below = `{1,2}`/`{1,16}`; near capacity = `{1,1}`; above = `{2,1}`; a one-frame burst = a large numerator such as `{64,1}` (= `CHAT_CHANNEL_CAPACITY` in one frame). The surplus above the channel capacity is retained at the sender, not dropped.

## Offline Deterministic Scenario Table

The harness runs the full scenario through `FramePipeline::run` with a `FakeClock` (pipeline pacing) and a `TestCollector` fake monotonic clock advanced exactly once per frame step by the config-derived interval — zero real sleeps, zero network, deterministic counts. Entry point and run command: `cargo test --locked latency_load` (all ten tests, in `src/app/run/`).

`latency_load_tests.rs` (rate / burst / sustained / recovery — 6 tests):

| Test name | Scenario (phases) | What it verifies | Harness metric gating the verdict |
| --- | --- | --- | --- |
| `low_rate_below_capacity_drains_per_frame_with_zero_backlog` | 161 frames @ `{1,16}` → 10 messages | Input far below capacity: every message drains in the frame after it is enqueued | STABLE: `received == enqueued == dequeued == 10`, `queue_depth_estimate == 0`, `oldest_in_flight_age_ns == None`, `in_flight_high_water == 1`, `message_channel_ns` all at one interval, `render_ns.max == 0`, `write_ns.max == 0` |
| `single_burst_temporarily_fills_channel_then_drains` | `{64,1}` for 1 frame, then idle | A burst of exactly the channel capacity momentarily fills it, then fully drains | GROW→PIN then DRAIN: `in_flight_high_water == 64`, `full_retry_total == 0` (in-harness), `dequeued == received`, leftover empty, `message_channel_ns` min one interval / max `64 × interval` |
| `sustained_below_capacity_keeps_backlog_stable` | 128 frames @ `{1,2}`, then idle | Sustained input below capacity holds the queue at rest | STABLE: `dequeued == received == 64`, `in_flight_high_water == 1`, `message_channel_ns` min == max == interval |
| `sustained_at_capacity_holds_one_frame_backlog` | 128 frames @ `{1,1}` | Input exactly at capacity keeps a steady one-frame backlog | STABLE backlog: `dequeued == received − 1`, one message left queued at stop; the pull-before-inject ordering makes `in_flight_high_water == 2` (one queued + the one the loop just pulled) and `oldest_in_flight_age_ns == Some(0)` at the final frame |
| `sustained_above_capacity_pins_queue_at_capacity_and_drains_after_rate_falls` | 128 frames @ `{2,1}`, then `4 × 64` idle drain frames | Sustained input above capacity pins the queue and drains once rate falls | GROW→PIN then DRAIN: `in_flight_high_water == 65` (full channel + the just-dequeued message), `dequeued == received == 256`, `queue_depth_estimate == 0`, leftover empty |
| `recovery_after_burst_drains_backlog_to_zero` | `{72,1}` for 1 frame (`64 + 8`), then `4 × 64` idle | A burst above capacity is retained at the sender, then recovered to zero | RECOVER: `received == enqueued == dequeued == 72`, `in_flight_high_water == 65` (channel pinned at 64 + the just-dequeued), leftover empty |

`latency_load_loss_tests.rs` (loss / shutdown / pure pins — 4 tests):

| Test name | Scenario (phases) | What it verifies | Harness metric gating the verdict |
| --- | --- | --- | --- |
| `round_trip_counts_prove_no_silent_loss_and_bounded_memory` | mixed: `{40,1}` ×1, `{2,1}` ×16, `{1,2}` ×64, then idle → 104 due | No silent mid-session loss and bounded memory over a mixed profile | Accounting equality via `assert_full_round_trip_accounting`: `enqueued == received`, `due == received`, `received == dequeued + leftover`, leftover empty, and `in_flight_high_water ≤ CHAT_CHANNEL_CAPACITY` |
| `stop_with_messages_queued_documents_shutdown_loss` | `{64,1}` for 1 frame, then 23 idle frames (24 total) | The documented shutdown behavior when messages remain queued | DOCUMENTED LOSS: run stops at the write budget; `received == 64`, `dequeued == 23`, 41 messages left in the channel (`leftover.len() == queue_depth_estimate == 41`), all rendering the fixture `author-fixture: text-fixture`; `oldest_in_flight_age_ns == Some(23 × interval)` |
| `rate_profile_due_counts_follow_rational_schedule` | none (pure pin of `RateProfile::advance`) | The rational-rate arithmetic the scenarios rest on | `{2,3}` over 6 frames yields exactly `0,1,1,0,1,1` with no drift; a `{64,1}` burst profile yields `64` each frame while its phase runs (the phase length, not the profile, bounds the total) |
| `scenario_pacing_interval_derives_from_configured_fps` | none (pure pin) | The FPS→interval derivation (regression guard against any hardcoded 30) | `interval == 1_000_000_000 / fps` with `fps` read from the fixture, and the floor-division identity `interval * fps + fps > 1_000_000_000`; every scenario test additionally asserts its `pacing_interval_nanos` equals this config-derived value |

## Harness Mechanics and the Frame-Granularity Retry

The harness is honest about the one place its model diverges from the production threading. Each frame step does the real consumer's pull **before** the capacity-guarded injection: it advances the diagnostics fake clock, pulls once through the real `ChatSource` over the real bounded channel, releases the rational rate into a sender-side FIFO backlog, then injects as many retained carriers as the channel has free slots (so `try_send` never returns `Full` inside the harness — zero spin, deterministic step counts). The consequence is the `in_flight_high_water == CAP + 1` / `== 2` shape in the at-capacity and above-capacity rows above: one message sits in the channel while the frame loop simultaneously holds the one it just dequeued. The harness retries the sender's retained messages at frame granularity; the observable queue outcome (queue pinned at capacity, nothing dropped, eventual drain) is identical; the real threaded retry path is pinned separately by the Task-1-era rosters — `chat_worker_burst_tests.rs :: worker_full_channel_retains_the_burst_without_loss_or_reorder` and `chat_delivery_diagnostics_tests.rs :: worker_marks_backpressured_on_full_channel` — which drive the production `send_stop_aware` Full arm on real worker threads and stay in those rosters, not in `latency_load`.

Two more honesty notes: the offline numbers are deterministic multiples of the config interval because enqueue stamps land on frame boundaries and dequeue rides a later frame — that yields exact `message_channel_ns` lags, but the **magnitude** evidence for real jitter/interval (`frame_interval_ns`, `frame_jitter_ns`, `render_ns`, `write_ns`) comes only from operator runs against the real monotonic clock, not from these fixtures. And the FFmpeg-write boundary uses `RecordingSink` (a recorded fake write), so no scenario spawns FFmpeg, opens a pipe, or creates a broadcast or local MP4.

## Operator Live Test Procedure (Operator-Run Only)

This procedure is performed by the human operator against a real YouTube account. It is DISTINCT from every automated check on this page: the agent did not run, and never claims, it, and its results are recorded only by the operator — never as build evidence. Prerequisites and the broadcast/chat start-up setup are NOT repeated here — see the canonical [`docs/production-run.md` Manual Verification Procedure](production-run.md#manual-verification-procedure-operator-run). The latency-specific steps:

1. **Separate viewer/chat account.** Send test chat messages from an account distinct from the streaming channel's own account; watch on the viewer side from a third surface if possible.
2. **Consistent clock or manual stopwatch.** Either use a synchronized wall clock on both the sender and the viewer (subtract the two stamps after the run), or one operator-side phone/clock for the manual stopwatch: note the send moment and the first-visible moment, and record which method each result row used. Do NOT mix the app's monotonic `*_ns` durations with these wall-clock figures as if they were one measurement (the clock-domain note above).
3. **At least five repeats.** Send at least five test messages at a consistent, quoted spacing (e.g. roughly one every 30 seconds) and record all of them — never treat a single observation as a stable baseline.
4. **Record broadcast/player latency mode + playback conditions.** Note the latency mode YouTube Studio shows for this broadcast (default/low/ultra-low as displayed) and the playback conditions (network, watch-page device). Never claim a value the UI does not show.
5. **Correlate each message by its message id.** With diagnostics enabled (capabilities per the [`configuration.md` field reference](configuration.md#diagnostics)), locate the sent message's rows in the JSONL diagnostics file by matching the row `epoch_seconds` / `provider_epoch_seconds` window against the recorded send time and the same message's chat-log line in `chat.log_file`; the row's `correlation` field is the provider message id. Record that id against the manual measurement row.
6. **Record end-to-end separately from internal durations.** For each message read the matching internal stage aggregates (`message_channel_ns`, `message_render_ns`, `write_ns`) from the summary for that window and record them in a SEPARATE column from the manual send→visible figure. An internal stage duration is already inside the end-to-end total — do not sum them together.
7. **Observability ON vs OFF overhead comparison.** Repeat the same message profile once with all diagnostics capabilities enabled and once with them disabled (or the section absent — the byte-identical run path), and compare qualitatively: frame-cadence stability on the watch page and the process's rough host/CPU load as the operator's own tooling reports it. State honestly that sub-percent overhead was NOT instrumented beyond this comparison.
8. **Closing reminder.** Re-record the ~35-second observation's context explicitly: it is the whole-path end-to-end baseline (upstream + application + encode/ingest + viewer), never a single-stage figure and never a platform constant — and this page's offline suite does not reproduce it.

## Results Template

One header block per session; one row per repeat. Copy-friendly field list (fill from the run, not the example config):

```
run/build identifier      : <version> @ <git HEAD> (<branch>)
date/time                 : <local> / <UTC> (per row)
configured FPS + capacity : fps=<from config.json>, channel=64 (CHAT_CHANNEL_CAPACITY)
diagnostics settings      : <which capabilities enabled> / summary_interval_seconds=<n> / output_file=<path>
message rate/burst profile: <how many messages, at what spacing> (mirrors the offline profile shape)
queue metrics             : received=<n> enqueued=<n> dequeued=<n> full-retry=<n> retained=<n> depth=<n> high-water=<n> oldest-in-flight-age=<n>
internal timing summaries : per-histogram count/min/max/mean for message_channel_ns, message_render_ns, render_ns, write_ns, frame_interval_ns, frame_jitter_ns
external viewer delay     : <per-repeat send→visible> (clock method noted) — or "not measured this run"
notes                     : <latency mode, playback conditions, duplicates/loss observed, incidents>
```

## Interpretation Matrix

Five delay stages, and how each can be observed. "Direct" means an app-measured diagnostic metric exists; "Externally inferred" means an operator/external observation is required, with the reason.

| Stage | Observable by the app? | How | Why / caveat |
| --- | --- | --- | --- |
| Upstream delivery (provider send → app batch receipt) | Externally inferred only | Correlate provider `publishedAt` wall clock against local receipt | Provider wall clock is not the app monotonic domain; the diff carries uncorrectable clock skew — never presented as precise |
| Application queue delay (receipt → dequeue) | Measured directly | `wait_ns`, `message_channel_ns`, `queue_depth_estimate`, `in_flight_high_water`, `oldest_in_flight_age_ns` | All same-process monotonic; bounded channel is the only queue between worker and frame loop |
| Frame-render delay (dequeue → render) | Measured directly | `message_render_ns`, `render_ns`, `frame_interval_ns`, `frame_jitter_ns` | Rendered inside the frame loop on the app anchor |
| FFmpeg + ingest delay | Split: stdin write measured, ingest inferred | `write_ns` covers ONLY the app's stdin write; FFmpeg encode + YouTube ingest are external | The app owns the pipe, not the encoder or the live ingest behind it |
| Viewer playback delay (ingest → first visible) | Externally inferred only | Operator stopwatch / synchronized-clock procedure above | The app cannot observe any viewer's player; this is the last leg of the ~35 s |

## Threshold Stance

**No production-latency threshold is asserted by this work.** The values the offline tests pin (e.g. `dequeued == received` after a drain, `in_flight_high_water ≤ CHAT_CHANNEL_CAPACITY`, `message_channel_ns.min ≥ one interval`) are regression guards on the harness's own deterministic behavior — they protect the accounting and the bounded-memory invariant, not an end-to-end service objective. Per the TODO, named thresholds become diagnostic observations or regression guards ONLY when justified by measured behavior; a viewer-latency or application-latency SLO would require operator-run measurement evidence (the procedure and template above) before any such number could be justified, and no such claim is made here.

## Offline Verification Reminder

Every automated check on this page is **offline**: the ten `latency_load` tests drive the real channel and the real frame loop on fake clocks with fixture-shaped strings only (`author-fixture` / `text-fixture` / `latency-m-*` carrier ids); no test contacts Google, opens a socket, spawns FFmpeg, or creates a broadcast. The live column of every table above is entirely operator-run. Synthetic pipeline measurements never reproduce YouTube's server-side delivery or viewer buffering, and no value produced by the automated suite is an end-to-end or viewer-latency figure.

## See Also

- [`docs/production-run.md`](production-run.md#pipeline-observability-opt-in-diagnostics) — Pipeline Observability (Opt-In Diagnostics): what each capability emits, the bounded JSONL output, and the canonical operator Manual Verification Procedure this page's procedure supplements (not duplicates).
- [`docs/configuration.md#diagnostics`](configuration.md#diagnostics) — the authoritative diagnostics field reference.
- [`docs/production-run.md`](production-run.md#live-chat-behavior-diagnostics-recovery-privacy-quota) — Live Chat behavior, the bounded channel, and the shutdown-loss facts.
- [docs/game-foundation.md](game-foundation.md) — the game-foundation handoff consuming this page's timing facts.
