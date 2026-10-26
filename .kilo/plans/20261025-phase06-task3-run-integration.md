# Implementation Plan — Phase 06, Task 3: Production Run and Frame-Pipeline Integration

> TODO (authority): `.agent/todos/20261025/20261025-todo-2.md` — **Task 3 "Production Run and Frame-Pipeline Integration" bullets only.**
> Binding research + design record: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (D-1..D-14; §3 worker shape; §4 inherited obligations — this plan TRANSCRIBES them; §1 official citations are inherited verbatim, never re-verified here).
> Binding Task-2 carry-forwards: `.kilo/plans/20261025-phase06-task2-adherence.md` §7.1 (C-1..C-5) — transcribed verbatim in §2.6.
> Landed Task-2 surface: `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md` (the pinned `src/youtube/` chat seam).
> Global plan (binding context): `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md` (§ Per-Task Pre-Analysis "Task 3").
> Workflow: `.kilo/commands/critical-workflow.md` — this plan covers ONLY step 4.1b analysis for **Task 3** plus the 4.2 execution recipe; 4.3/4.4/4.5b/4.6 applicability is codified in §11.
> Baseline (caller-stated, 2026-10-26): branch `feat/phase06-youtube-live-chat`, HEAD `868b015`, v0.12.0, tree clean, gate baseline **429 passed / 0 failed** (`ALL CHECKS PASSED`, log `logs/checks/20261026T031702Z.log`).
>
> **AMENDMENT 1 (2026-10-26, caller rulings at 4.2 STOP; binding):**
> 1. §3.4/§5.1 item 5 — the retrieval-failure test does NOT append to `src/app/pipeline/runner_tests.rs` (194 lines; +28 would breach the 200-line cap): it lands in the NEW sibling roster `src/app/pipeline/runner_retrieval_tests.rs`, registered `#[cfg(test)] #[path]` from `runner.rs` (repo-standard split precedent). Test name/body unchanged.
> 2. §4.10 — `src/app/run/mod.rs` (185 lines) projects > 200 with the full wiring delta: the plan's own contingency GOVERNS — keep mod.rs deltas minimal (2 `mod` decls + the single `build_chat_stage` call + doc line) and place the wiring body in `chat_worker.rs`'s `build_chat_stage`; if mod.rs still exceeds 200 after that, STOP with exact counts (do NOT improvise).
> 3. §5.2 roster A — if `src/app/run/chat_worker_tests.rs` exceeds 200 lines with the 10-test roster, split it test-only into `chat_worker_tests.rs` + NEW sibling `chat_worker_flow_tests.rs` (all exact test names preserved; report the split in the 4.2 summary).
>
> **AMENDMENT 2 (2026-10-26, caller rulings at the mid-(c) STOP; binding):**
> 4. §4.3 (`src/app/run/chat_worker.rs`, 335 post-rustfmt lines, plan-prescribed content cannot honestly fit ≤200): SPLIT into TWO production siblings, registrations (`chat_worker_tests`, `chat_worker_budget_tests`, `gate_chat_tests` decls) STAY in `chat_worker.rs`:
>    - `chat_worker.rs` keeps: the module doc, imports, the 8 constants + 3 info constants, `ChatWorkerPlan`/channel bundle/`chat_channel`/`spawn_chat_worker`/`build_chat_stage`, the session struct + `run_chat_worker` + poll loop, the 22-line cfg-split resolution pair.
>    - NEW sibling `chat_worker_delivery.rs` holds: `deliver`/`into_chat_message`/`send_stop_aware`, the bounded sleeps/backoff helpers, `record_terminal`, the cfg-split sleep helper. Visibility `pub(super)` per the established sibling pattern.
>    - Both files must land ≤ 200 lines; report final counts.
> 5. Confirmed incidental implementations (caller-confirmed, record in the 4.2 summary): (i) `handle_page` delivers accepted messages of a non-baseline page BEFORE the `chat_ended` early-return — roster A test 2 (`receiver yields ONLY the live item` from an ended-marked page) mandates delivery-before-end; (ii) roster A test 1 scripts an initial baseline page before its 2-item live page (baseline-on-first-successful-page policy); pinned assertions unchanged.

## 0. Scope of this task (TODO Task 3 bullets → plan sections)

| TODO Task-3 bullet | Covered by |
| --- | --- |
| Replace `ChatFreeSource` with the real YouTube Live Chat source wired to the resolved chat ID + authorized transport; no second broadcast; Phase-05 resource lifecycle untouched | §4.4 (chat stage), §4.5 (source), §2.2 (placement) |
| Smallest typed backward-compatible widening of `MessageSource::next_message`'s failure type; update BOTH production and test implementations; never collapse provider/network failures into chat-log errors | §4.1 (seam), §3.2 (mechanical fallout) |
| Keep `demo` on its deterministic source; preserve the local MP4 timeline/arguments exactly | §3.2 row 2, §9 ledger |
| Worker/source stopped by the existing Ctrl+C path; blocked network ops / channel sends cannot prevent bounded shutdown | §4.3 (worker), §2.4 (stop bundle), §5.2 roster B |
| Preserve one-message-per-frame pacing + `ChatStore` log-first contract; log-write failure stays explicit fatal | §4.1 runner mapping, §9 ledger |
| Frame loop continuous when no chat arrives (slow API, reconnects, empty chat) | §4.5 (try_recv), §5.2 roster C |
| Startup/shutdown order for worker, frame loop, FFmpeg stdin EOF, FFmpeg wait, API connection; worker errors reach the funnel WITHOUT masking the primary failure | §2.2/§2.3 (order + refinement), §4.6 (funnel + precedence) |
| Do not change Phase-05 stream-health settings as a substitute | §9 STOP-guards |

Out of scope for Task 3 (per the caller's split): docs, builds, artifact rebuilds, the completion report, the manual E2E, `.agent/*` metadata — ALL Task 4. Task 3 lands production code + offline tests + the gate.

---

## 1. Current-state analysis (verified on the tree at HEAD `868b015`)

- `src/app/run/source.rs` holds `ChatFreeSource` (17 lines): `next_message` always `Ok(None)`; used by `streaming.rs` (line 86) and `streaming_stage_tests.rs` (line 47).
- `src/app/pipeline/source.rs` (20 lines): `MessageSource::next_message(&mut self) -> Result<Option<ChatMessage>, ChatError>` — the D-8 widening point; called ONLY at the top of each frame iteration in `runner.rs::accept_next_message` (line 95).
- `FramePipelineError` (`src/app/pipeline/error.rs`, 5 variants: Chat/Render/FrameByteOverrun/FrameSizeMismatch/Sink); `RunError` (`src/app/run/error.rs`, 8 variants; exit mapping = cancellation 0, everything else 1).
- Run flow (`run/mod.rs`): stage 1 Ctrl+C pair (`install_stop_pair` flips `StopFlag` + `WaitCanceller` in ONE handler) → stages 2–7 (credentials → scope gate → transport factory → `prepare_broadcast` → watch URL + destination reveal) → stage 8 `spawn_startup_gate` → `runner.run(PreparedStreaming)`: FFmpeg start (`prepared_stream_pipeline`) → frame loop → on pipeline error cancel the wait → `gate.finish()` → `primary_shutdown_outcome` (Fatal outranks; D-T3-6 rule 6). The gate runs on its own std thread and confirms ASYNCHRONOUSLY while the frame loop runs (gate.rs module doc: video must flow before `streamStatus == active` — a blocking barrier would deadlock).
- Task-2 landed surface consumed by this plan: `LiveChatTransport` (O6 `lookup_broadcast_chat_id` + O7 `list_messages`, both `Result<_, LiveApiError>`), `resolve_live_chat_id` (+ private cfg-split core `resolve_live_chat_id_with_interval`), `ChatListCommand`/`LiveChatMessageList` (`next_page_token`/`polling_interval_millis`/`offline_at` exposed), `map_page`/`ChatPageOutcome`/`ChatTextMessage`, `ChatDedupWindow`, `GoogleLiveApiTransport` (implements BOTH transport traits), `LiveApiError::ChatDisabled`/`ChatEnded` (17 variants total), and the staged `src/youtube/mod.rs` allows (6 module-level `dead_code` + 4 `unused_imports` re-export stagings, comments naming Task 3).
- `ChatMessage` (`src/chat/model.rs`): `new(author, text)` normalizes single-line; `with_message_id(&str)`; `with_timestamp(Option<u64>)` carries an item-level `#[allow(dead_code)]` (C-2 retirement).
- `ChatStore::accept` (log-first, reject-on-log-failure) — untouched by Task 3.
- MessageSource implementers today: `ChatFreeSource` (replaced), `DemoMessageSource`, test fake `ScriptedSource` (`source_helpers.rs`).

## 2. Binding design (every decision picked ONE way)

### 2.1 Transcribed verbatim from the record (NOT re-adjudicated)

D-1 (list polling), D-2 (O6 resolution: 6 × 5 000 ms; absence → typed fatal surfacing as `ChatUnavailable`), D-3 (wire form — Task 2 landed), D-4 (skip/accept policy — Task 2 landed), D-5 (channel capacity 64, dedup window 512, baseline discard, `try_recv` drain), D-6 (poll cycle, backoff = max(server interval, 1 s) × 2^(attempt−1) capped 60 000 ms, budget 10 consecutive transients, sleep slices 1 000 ms, graceful vs terminal classes), D-7 (typed classes — Task 2 landed), D-8 (seam widening shapes — §4.1), D-11 (ZERO new dependencies; `Cargo.toml`/`Cargo.lock` diff EMPTY), D-12 (constants ledger — §4.3), D-13 (logging/redaction surface — §4.3.7), D-14 (offline test posture).

### 2.2 D-T3-A — Worker spawn topology: the gate thread spawns the chat worker at `Confirmed`

D-9 pins "join-spawn AFTER the gate has confirmed `streamStatus == active`". The ONLY thread that knows the confirmation moment is the gate thread, so:

- The chat-id RESOLUTION and the worker construction are prepared by the orchestration (stage 7.5, between the destination reveal and the gate spawn); the worker THREAD is spawned by the gate thread immediately upon `GateFinish::Confirmed`.
- Pre-confirmation, the frame loop runs with the real `ChatSource` whose channel is open-but-empty (`try_recv` → `Err(Empty)` → `Ok(None)`) — the Phase-05 video-before-confirmation contract (gate.rs module doc) is preserved; a literal "start the frame loop after confirmation" would deadlock (the stream only goes active once FFmpeg pushes frames).
- The worker's FIRST act on its own thread is the O6 resolution (D-2), then the poll loop. This keeps D-2's "resolution after the live transition" (the field appears at the transition; polling it pre-transition would false-fail the 30 s budget) and keeps D-2's failure funnel (`ChatUnavailable` via the D-8 funnel, NOT `RunError::Broadcast`).
- `GatePlan` gains `chat_worker: Option<ChatWorkerPlan>`; on `Ok(())` the gate thread does `plan.chat_worker.map(spawn_chat_worker)` and stores the `JoinHandle` in the shared gate state; `SpawnedGate::finish()` joins the gate thread, then joins the worker (bounded). `None` (all current gate tests) spawns nothing.

### 2.3 D-T3-B — Shutdown order: ONE sanctioned refinement of D-9

D-9's literal order (worker join BEFORE FFmpeg stdin EOF) is infeasible without changing the pipeline contract: `FramePipeline::run` is a monolith that owns the loop break AND `sink.finish()` (stdin EOF → FFmpeg wait), and the message-seam contract (call at the top of each iteration) leaves no seam to join between them. The pipeline contract is pinned (one-message-per-frame pacing; D-8 allows ONLY the error-type widening). Therefore the production order is:

1. The single Ctrl+C handler flips the frame-loop `StopFlag` and the `WaitCanceller` (unchanged — the worker SHARES the same `StopFlag` Arc, so one flip stops loop AND worker).
2. The frame loop ends (≤ 1 frame interval).
3. `pipeline.run()` finalizes the sink: stdin EOF → FFmpeg wait (existing; unbounded wait is the carried Phase-03 limitation). The worker stops CONCURRENTLY during this window (its sleeps are ≤ 1 s stop-slices; its sends are stop-aware `try_send`) — "chat stops accepting first" is satisfied at flag level.
4. `gate.finish()` joins the gate thread AND the chat worker (worker join ≤ ~1 s wake + any in-flight request; D-T2-16(a) carried: no request-level timeout).
5. `primary_shutdown_outcome` picks the primary error.

Recorded as a sanctioned refinement (the D-T3 gate-refinement precedent) with rationale: the worker is flag-stopped before/at the frame-loop stop; ONLY the frame loop writes the store/log, so log/video consistency holds regardless of the join position; messages still queued in the channel at shutdown are never consumed (a bounded ≤ 64-message shutdown loss window; the log never receives them, so log and video never diverge).

On a pipeline-fatal path the runner additionally flips the SAME stop flag before joining (a pipeline error does NOT flip it today — the worker would otherwise never stop and the join would hang): `if stream_outcome.is_err() { stop_flag.request_stop(); canceller.request_cancel(); }`.

### 2.4 D-T3-C — Stop bundle: the worker shares the frame-loop `StopFlag`

The handler flips `StopFlag` + `WaitCanceller` exactly as today (run/mod.rs `install_stop_pair` — byte-unchanged). The `ChatWorkerPlan` holds a CLONE of the same `StopFlag`. The gate's Fatal arm keeps flipping the same flag (existing). No third flag object is created (D-6's "(c) the chat-worker stop flag" is satisfied by the SAME shared flag; one handler, one flag, three consumers).

### 2.5 D-T3-D — Error funnel and precedence (D-9 pin)

- Worker terminal-fatal errors go into the shared terminal cell → the source surfaces `ChatSourceError::Retrieval(e)` → `FramePipelineError::ChatRetrieval(e)` → `RunError::ChatRetrieval(e)` → exit 1, ONE redacted `error:` line, NO new exit code.
- Precedence PIN (record D-9): a chat failure NEVER outranks `GateFinish::Fatal`. `primary_shutdown_outcome`'s existing arm order is unchanged; the pipeline-error arm gains a special-case mapper so `FramePipelineError::ChatRetrieval(e)` maps to `RunError::ChatRetrieval(e)` (not `RunError::Pipeline`) — only when the gate did not fail.
- Graceful classes (`offlineAt`, `chatEndedEvent`, `LiveApiError::ChatEnded`) NEVER write the cell; the worker exits cleanly; the source yields `Ok(None)` forever; frames keep flowing until Ctrl+C.

### 2.6 Task-2 carry-forwards (transcribed; binding)

- **C-1 conversion point:** the worker constructs `ChatMessage::new(&carrier.author, &carrier.text)` (normalization runs there), then `.with_message_id(id)` verbatim when `Some`, then `.with_timestamp(Some(seconds))` when `Some`. The `src/youtube/mod.rs` re-exports are the PINNED consumption surface — do not re-pick or remove without a caller ruling (see §4.9 for the two by-design-unconsumed helpers).
- **C-2:** `ChatMessage::with_timestamp`'s item-level `#[allow(dead_code)]` retires at the worker consumer.
- **C-3:** the worker/channel wiring exactly as C-3 lists (channel 64, per-session dedup, baseline discard, server-interval honor-loop with 1 000 ms stop-slices, backoff capped 60 000 ms, budget 10, terminal cell, graceful/fatal classes, stage-11 placement, join ≤ ~1 s, `RunError::ChatRetrieval` funnel, gate-Fatal precedence).
- **C-4:** the seam widening (D-8) with the enumerated mechanical fallout (§3.2).
- **C-5:** the 6 module-level `dead_code` stagings and the consumed re-export stagings in `src/youtube/mod.rs` retire when the worker roots the chain (§4.9 prescribes the exact retire/update act).

---

## 3. File inventory (exact)

### 3.1 New production files

| File | Contents | Projected lines |
| --- | --- | --- |
| `src/app/pipeline/source_error.rs` | `ChatSourceError`, `ChatRetrievalError`, Displays, `Error` impls, the pure `pipeline_source_failure` mapper | ~110 |
| `src/app/run/chat_worker.rs` | Constants (D-12), `ChatChannel` + `chat_channel()`, `ChatWorkerPlan`, `spawn_chat_worker`, the worker body (resolution + poll loop + delivery + conversion), stop-aware sleep helpers, the 3 info-event constants | ~195 (HARD CAP — if rustfmt lands >200: STOP → caller, exact counts) |
| `src/app/run/chat_failure.rs` | The closed `LiveApiError` transient/graceful predicates + `terminal_retrieval_error` mapping + the reason-string constants | ~90 |

### 3.2 Edited production files (the complete mechanical-fallout enumeration)

| # | File | Change |
| --- | --- | --- |
| 1 | `src/app/pipeline/source.rs` | Trait error type → `ChatSourceError`; doc rewritten (widened contract per D-8) |
| 2 | `src/app/pipeline/mod.rs` | `mod source_error;` + `pub(crate) use source_error::{ChatRetrievalError, ChatSourceError};` |
| 3 | `src/app/pipeline/runner.rs` | `accept_next_message` maps via `pipeline_source_failure` (import +1; body shape in §4.1.3) |
| 4 | `src/app/pipeline/error.rs` | + `ChatRetrieval(ChatRetrievalError)` variant + Display arm + `source()` arm |
| 5 | `src/app/demo/source.rs` | `DemoMessageSource::next_message` signature → `Result<Option<ChatMessage>, ChatSourceError>`; body/behavior UNCHANGED (returns `Ok(...)` only) |
| 6 | `src/app/pipeline/source_helpers.rs` | `ScriptedSource` signature widened; + `failing_retrieval(ChatRetrievalError)` constructor |
| 7 | `src/app/run/source.rs` | FULL REWRITE: `ChatSource` (the worker-aware production source; §4.5) |
| 8 | `src/app/run/chat_worker.rs` | NEW (§4.3) |
| 9 | `src/app/run/chat_failure.rs` | NEW (§4.3.6) |
| 10 | `src/app/run/error.rs` | + `ChatRetrieval(ChatRetrievalError)` variant + Display arm + the `pipeline_error` mapper fn (§4.6) |
| 11 | `src/app/run/seams.rs` | + `ChatTransport` alias, `BroadcastTransports` pair, `TransportFactory` retypes (§4.7) |
| 12 | `src/app/run/streaming.rs` | `StreamingExecution` gains `source: ChatSource`; the loop uses it; the error path flips the stop flag; module doc updated (§4.8) |
| 13 | `src/app/run/gate.rs` | `GatePlan.chat_worker: Option<ChatWorkerPlan>`, gate-state worker handle, spawn-on-Confirmed, join in `finish()` (§4.4) |
| 14 | `src/app/run/mod.rs` | Stage-7.5 chat wiring via `chat_worker::build_chat_stage`; `mod chat_worker; mod chat_failure;`; module doc updated (§4.10) |
| 15 | `src/chat/model.rs` | `with_timestamp`: retire the `#[allow(dead_code)]` + doc re-cut (§4.2) |
| 16 | `src/youtube/live_chat_resolution.rs` | `resolve_live_chat_id_with_interval` + its use-site doc note widened to `pub(crate)` |
| 17 | `src/youtube/mod.rs` | C-5 retirement act (§4.9) |

### 3.3 New test/fixture files

| File | Tests |
| --- | --- |
| `src/app/pipeline/source_error_tests.rs` | 3 (§5.1) |
| `src/app/run/chat_source_tests.rs` | 5 (§5.3) |
| `src/app/run/chat_worker_tests.rs` | 10 (§5.2 roster A) |
| `src/app/run/chat_worker_budget_tests.rs` | 6 (§5.2 roster B) |
| `src/app/run/gate_chat_tests.rs` | 2 (§5.4) |
| `src/app/run/runtime_chat_fakes.rs` | 0 (the `#[cfg(test)]` scripted chat fake + page builders, Mutex-based) |

### 3.4 Edited test files

| File | Change |
| --- | --- |
| `src/app/run/streaming_stage_tests.rs` | `ChatFreeSource` → `chat_channel()` + `ChatSource` construction (the run is pre-stopped; the source is never polled) |
| `src/app/run/gate_tests.rs` | `gate_plan(...)` gains `chat_worker: None` ONLY |
| `src/app/run/error_tests.rs` | `all_run_errors()` gains the ChatRetrieval entry + 1 new precedence test (§5.5) |
| `src/app/run/runtime_tests.rs` | `ScriptedRun` parameter object (the transport-pair factory fallout) + the Confirmed test asserts the chat ledger (§5.6) |
| `src/app/run/runtime_runner_fixture.rs` | Destructures the new `source` field (bind-and-ignore) |
| `src/app/pipeline/runner_retrieval_tests.rs` | NEW sibling roster — 1 retrieval-failure test (§5.1 item 5; AMENDMENT 1 ruling 1) |
| `src/app/pipeline/error_tests.rs` | + 1 Display/source pin for the new variant (§5.1) |

Projected test total: 429 + 28 = **457 — PROJECTED arithmetic only; the OBSERVED gate count governs** (standing phase rule).

---

## 4. Code specifications (shapes frozen where they matter)

### 4.1 The seam widening (record D-8, carry-forward C-4)

**4.1.1 `src/app/pipeline/source_error.rs`** — exact enum shapes (record-pinned; do not re-shape):

```rust
use std::fmt;

use super::error::FramePipelineError;
use crate::chat::ChatError;

/// The widened `MessageSource` failure contract (record D-8): chat-domain
/// store errors stay in the `Chat` class; retrieval (YouTube/network)
/// failures are typed separately and NEVER collapse into chat-log errors.
#[derive(Debug)]
pub(crate) enum ChatSourceError {
    Chat(ChatError),
    Retrieval(ChatRetrievalError),
}

/// A typed chat-retrieval failure carried from the worker's terminal
/// cell (record D-8). `detail` carries ONLY the redacted `Display` of the
/// inner typed `LiveApiError` — never a body, never a token.
#[derive(Debug)]
pub(crate) enum ChatRetrievalError {
    ChatUnavailable { detail: String },
    Quota { detail: String },
    Transport { detail: String },
}
```

Display lines (PINNED verbatim; hand-written helpers or inline writes, no panics):

- `ChatUnavailable { detail }` → `live chat is unavailable: {detail}`
- `Quota { detail }` → `live chat retrieval stopped by YouTube quota: {detail}`
- `Transport { detail }` → `live chat transport failed after repeated attempts: {detail}`
- `ChatSourceError` delegates BOTH arms verbatim to the inner Display (`write!(formatter, "{error}")`).

Impls: `std::error::Error` for both (`ChatSourceError::source()` → the inner error; `ChatRetrievalError::source()` → `None`). No `Clone` needed.

The pure mapper consumed by the runner (keeps `runner.rs` within its line cap):

```rust
pub(crate) fn pipeline_source_failure(error: ChatSourceError) -> FramePipelineError {
    match error {
        ChatSourceError::Chat(chat_error) => FramePipelineError::Chat(chat_error),
        ChatSourceError::Retrieval(retrieval) => FramePipelineError::ChatRetrieval(retrieval),
    }
}
```

**4.1.2 `src/app/pipeline/source.rs`** — the trait becomes:

```rust
pub(crate) trait MessageSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError>;
}
```

Doc rewrite: the seam feeds `demo` (deterministic schedule) and the production run (the worker-backed `ChatSource`); the failure type is the D-8 widening — `Chat` = the chat-domain store error (log-first contract), `Retrieval` = the typed YouTube/chat retrieval failure; the call point stays the TOP of every frame iteration, at most one due message per frame.

**4.1.3 `src/app/pipeline/runner.rs`** — `accept_next_message` becomes (behavior identical for demo; the log-first `ChatStore::accept` contract untouched):

```rust
    fn accept_next_message(&mut self) -> Result<(), FramePipelineError> {
        let due = self.source.next_message().map_err(pipeline_source_failure)?;
        if let Some(message) = due {
            self.chat_store
                .accept(message)
                .map_err(FramePipelineError::Chat)?;
        }
        Ok(())
    }
```

with `use crate::app::pipeline::source_error::pipeline_source_failure;` (or the mod.rs re-export). Line-cap note: `runner.rs` sits at 199; this edit is ±1 line. If rustfmt lands it at >200, STOP → caller with exact counts (the caller-ruled split precedent).

**4.1.4 `src/app/pipeline/error.rs`** — new variant appended AFTER `Sink`:

```rust
    /// A typed chat-retrieval failure surfaced by the production source
    /// (record D-8); distinct from the chat-store `Chat` class.
    ChatRetrieval(ChatRetrievalError),
```

Display arm: `FramePipelineError::ChatRetrieval(error) => write!(formatter, "chat retrieval failed: {error}")`. `source()` arm: `Some(error)`.

### 4.2 `src/chat/model.rs` — C-2 retirement

Remove the `#[allow(dead_code)]` above `with_timestamp` and re-cut its doc comment to:

```rust
    /// Attaches the timestamp verbatim, builder style: seconds since the
    /// UNIX epoch, or `None` to leave the message without one. The Phase-06
    /// chat worker attaches the provider's published epoch; the demo
    /// timeline attaches none (the store's own-timestamp record stays
    /// covered by the store tests).
```

Nothing else in `src/chat/` changes.

### 4.3 `src/app/run/chat_worker.rs` — the bounded worker

**4.3.1 Module doc** records: D-5/D-6/D-9/D-12 transcription; the spawn-at-confirmation topology (§2.2); the join-position refinement (§2.3); the shutdown message-loss window; the ≤30 s one-shot resolution shutdown-latency limitation (§4.3.4); D-13 redaction (message text/authors/user-message ids NEVER logged).

**4.3.2 Constants (D-12; module-private except the channel capacity):**

```rust
pub(super) const CHAT_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_CHAT_POLL_INTERVAL_MS: u64 = 10_000;
const CHAT_SLEEP_SLICE_MS: u64 = 1_000;
const CHAT_BACKOFF_MAX_MS: u64 = 60_000;
const CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES: u32 = 10;
```

Plus the D-13 info-event constants (pinned by a test):

```rust
pub(super) const CHAT_ID_RESOLVED_MESSAGE: &str = "live chat id resolved";
pub(super) const CHAT_BASELINE_MESSAGE: &str =
    "live chat baseline established; the history page was discarded";
pub(super) const CHAT_ENDED_MESSAGE: &str =
    "live chat ended; the stream continues without new chat messages";
```

**4.3.3 Channel bundle + plan (≤2-args via literals; fields `pub(super)`):**

```rust
pub(super) struct ChatChannel {
    pub(super) sender: SyncSender<ChatMessage>,
    pub(super) receiver: Receiver<ChatMessage>,
    pub(super) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
}

pub(super) fn chat_channel() -> ChatChannel {
    let (sender, receiver) = sync_channel(CHAT_CHANNEL_CAPACITY);
    ChatChannel { sender, receiver, terminal: Arc::new(Mutex::new(None)) }
}

pub(super) struct ChatWorkerPlan {
    pub(super) transport: ChatTransport,
    pub(super) broadcast_id: String,
    pub(super) sender: SyncSender<ChatMessage>,
    pub(super) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
    pub(super) stop_flag: StopFlag,
}

pub(super) fn spawn_chat_worker(plan: ChatWorkerPlan) -> JoinHandle<()> {
    thread::spawn(move || run_chat_worker(plan))
}

/// The stage-7.5 constructor consumed by `run_runtime` (3-arg sanctioned
/// exception family, record D-T3-15): splits the channel bundle into the
/// pipeline source and the gate-carried worker plan.
pub(super) fn build_chat_stage(
    chat: ChatTransport,
    broadcast_id: &str,
    stop_flag: StopFlag,
) -> (ChatSource, ChatWorkerPlan) { ... }
```

`ChatTransport` = `super::seams::ChatTransport` (`Arc<dyn LiveChatTransport + Send + Sync>`). Imports: `crate::app::pipeline::{ChatRetrievalError, MessageSource, StopFlag}` (the source impl), `crate::chat::ChatMessage`, `crate::youtube::{map_page, resolve_live_chat_id, ChatDedupWindow, ChatListCommand, ChatPageOutcome, ChatTextMessage, LiveApiError, LiveChatTransport, LiveChatMessageList}`.

**4.3.4 Worker body.** A small session struct keeps every fn ≤50 lines / ≤2 args:

```rust
struct ChatWorkerSession {
    plan: ChatWorkerPlan,
    dedup: ChatDedupWindow,
    page_token: Option<String>,
    poll_interval_ms: u64,
    consecutive_failures: u32,
}
```

`run_chat_worker(plan)`:
1. Stop check → return.
2. Resolution via the cfg-split wrapper (below). `Ok(id)` → `info!(live_chat_id = %id, "{}", CHAT_ID_RESOLVED_MESSAGE)`. `Err(error)` → `record_terminal(&plan.terminal, terminal_retrieval_error(&error)); return;`
3. `ChatWorkerSession::new(plan, id)` → loop of one-shot `poll_cycle()` calls until a stop/terminal/graceful return.

The cfg-split resolution (the D-I zero-sleep precedent; consumes the Task-2 seam in production — retires its staging allow):

```rust
#[cfg(not(test))]
fn resolve_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    crate::youtube::resolve_live_chat_id(transport, broadcast_id)
}

#[cfg(test)]
const CHAT_ID_RESOLUTION_TEST_INTERVAL_MS: u64 = 0;

#[cfg(test)]
fn resolve_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    crate::youtube::resolve_live_chat_id_with_interval(
        transport,
        broadcast_id,
        CHAT_ID_RESOLUTION_TEST_INTERVAL_MS,
    )
}
```

RECORDED LIMITATION: the production resolution sleeps raw 5 000 ms slices (Task-2's no-canceller decision) — a Ctrl+C during the one-shot ≤30 s resolution window delays the worker join by at most the remaining budget (bounded; documented in the module doc; the poll loop itself is ≤1 s stop-sliced).

**4.3.5 `poll_cycle` (one D-6 cycle):**
1. `stop_flag.is_requested()` → return.
2. `list_messages(&ChatListCommand { live_chat_id: id.clone(), page_token: page_token.clone() })`:
   - `Ok(page)`: `consecutive_failures = 0`; interval update `poll_interval_ms = page.polling_interval_millis.filter(|ms| *ms > 0).unwrap_or(poll_interval_ms)` (a degenerate `0` is treated as absent guidance — documented; never shorter than real server guidance); `let outcome = map_page(&page);`
     - Baseline: on the FIRST successful response record `chat_worker::CHAT_BASELINE` info with `discarded_messages = outcome.messages.len()` and SKIP delivery entirely; advance `page_token = page.next_page_token`.
     - `outcome.chat_ended` → `info!("{}", CHAT_ENDED_MESSAGE);` return (graceful — no cell write).
     - Otherwise `deliver(&outcome.messages, ...)` (dedup → convert → stop-aware send), then `page_token = page.next_page_token`, then `sleep_in_stop_slices(poll_interval_ms)`.
   - `Err(error)`:
     - `is_graceful_end(&error)` (`matches!(error, LiveApiError::ChatEnded)`) → `info!("{}", CHAT_ENDED_MESSAGE);` return (graceful).
     - `is_transient_failure(&error)` → `consecutive_failures += 1`; if `consecutive_failures >= CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES` → `record_terminal(..., ChatRetrievalError::Transport { detail: error.to_string() })`; return. Else `sleep_in_stop_slices(backoff_delay_ms(consecutive_failures, poll_interval_ms))`.
     - Otherwise (terminal class) → `record_terminal(&plan.terminal, terminal_retrieval_error(&error)); return;`

`record_terminal(cell, error)` locks the cell and stores the error (ignore a poisoned lock — the existing `if let Ok` convention).

**4.3.6 `src/app/run/chat_failure.rs`:**

```rust
const RATE_LIMIT_EXCEEDED_REASON: &str = "rateLimitExceeded";

/// D-6's CLOSED transient class: Network / 5xx-backend / malformed
/// responses, plus the chat rate-limit QuotaLike reason; everything else
/// on a chat call is terminal (unknown quota reasons included — the app
/// never hammers a quota-shaped failure).
pub(super) fn is_transient_failure(error: &LiveApiError) -> bool {
    match error {
        LiveApiError::Network { .. }
        | LiveApiError::BackendUnavailable { .. }
        | LiveApiError::MalformedResponse { .. } => true,
        LiveApiError::QuotaLike { reason } => reason == RATE_LIMIT_EXCEEDED_REASON,
        _ => false,
    }
}

/// D-8's three-way terminal mapping. `detail` = the inner Display only.
pub(super) fn terminal_retrieval_error(error: &LiveApiError) -> ChatRetrievalError {
    match error {
        LiveApiError::QuotaLike { .. } => ChatRetrievalError::Quota { detail: error.to_string() },
        _ => ChatRetrievalError::ChatUnavailable { detail: error.to_string() },
    }
}
```

Mapping table (pinned): `ChatDisabled` → ChatUnavailable; `ApiNotFound` (any resource, incl. the resolution absence) → ChatUnavailable; `InsufficientPermissions`/`AuthorizationRejected` → ChatUnavailable; `QuotaLike` → Quota; `ChatEnded` never reaches here (graceful, handled first); every other variant (defensive catch-all) → ChatUnavailable. The budget-exhausted `Transport` variant is constructed DIRECTLY in the poll cycle (not via this mapper). Module doc records the rationale + D-6's closed transient list.

**4.3.7 Delivery + conversion (C-1) + bounded sleeps:**

```rust
fn deliver(session: &mut ChatWorkerSession, messages: &[ChatTextMessage]) {
    for carrier in messages {
        let unseen = match carrier.message_id.as_deref() {
            Some(id) => session.dedup.record(id),
            None => true, // no provider id → undedupable; deliver (documented)
        };
        if !unseen {
            continue;
        }
        let message = into_chat_message(carrier);
        send_stop_aware(&session.plan, message);
    }
}

/// The C-1 conversion point (record D-4/D-8): the app side builds
/// `ChatMessage`; normalization runs in `ChatMessage::new`.
fn into_chat_message(carrier: &ChatTextMessage) -> ChatMessage {
    let mut message = ChatMessage::new(&carrier.author, &carrier.text);
    if let Some(id) = carrier.message_id.as_deref() {
        message = message.with_message_id(id);
    }
    if let Some(seconds) = carrier.published_epoch {
        message = message.with_timestamp(Some(seconds));
    }
    message
}

/// Stop-aware bounded send (D-9): `try_send` only; a full channel waits a
/// stop-slice and retries; the stop flag or a disconnected receiver ends
/// the loop and the remaining page is dropped (never blocks shutdown).
fn send_stop_aware(plan: &ChatWorkerPlan, message: ChatMessage) { ... }

fn sleep_in_stop_slices(total_ms: u64, stop_flag: &StopFlag) { ... } // ≤1 s slices, flag-checked BEFORE each slice

fn backoff_delay_ms(attempt: u32, base_ms: u64) -> u64 {
    let base = base_ms.max(CHAT_SLEEP_SLICE_MS).min(CHAT_BACKOFF_MAX_MS);
    let doubled = base.saturating_mul(2u64.saturating_pow(attempt - 1));
    doubled.min(CHAT_BACKOFF_MAX_MS)
}
```

Pure-fn pins: `backoff_delay_ms(1, 10_000) == 10_000`, `(2, 10_000) == 20_000`, `(7, 10_000) == 60_000` (cap), `(1, 120_000) == 60_000` (capped base).

cfg-split sleep (the zero-sleep test rule; tests never sleep):

```rust
#[cfg(not(test))]
fn sleep_slice(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
#[cfg(test)]
fn sleep_slice(_ms: u64) {}
```

### 4.4 `src/app/run/gate.rs` — the spawn-at-confirmation hook

- `GatePlan` gains `pub(super) chat_worker: Option<ChatWorkerPlan>,` (after `stop_flag`).
- `GateState` gains `chat_worker: Option<JoinHandle<()>>,`.
- In `run_confirmation`'s `Ok(())` arm, after the two existing info events:

```rust
            let chat_worker = plan.chat_worker.map(spawn_chat_worker);
            record_chat_worker(state, chat_worker);
            GateFinish::Confirmed
```

with

```rust
fn record_chat_worker(state: &Mutex<GateState>, chat_worker: Option<JoinHandle<()>>) {
    if let Ok(mut recorded) = state.lock() {
        recorded.chat_worker = chat_worker;
    }
}
```

(The disjoint field move across match arms — `plan.chat_worker` moved in the Ok arm, `plan.stop_flag` used in the Err arm — compiles; the plan is owned by the thread closure.)

- `SpawnedGate::finish()` gains the worker join BEFORE the outcome read, and it must run even when the gate thread itself panicked:

```rust
    pub(super) fn finish(self) -> GateFinish {
        let gate_panicked = self.handle.join().is_err();
        join_recorded_chat_worker(&self.state);
        if gate_panicked {
            return GateFinish::Panicked;
        }
        match self.state.lock() {
            Ok(mut state) => state.outcome.take().unwrap_or(GateFinish::Panicked),
            Err(_) => GateFinish::Panicked,
        }
    }

fn join_recorded_chat_worker(state: &Arc<Mutex<GateState>>) {
    let worker = state
        .lock()
        .ok()
        .and_then(|mut recorded| recorded.chat_worker.take());
    if let Some(worker) = worker {
        let _ = worker.join();
    }
}
```

Module doc gains the worker-spawn duty + the join position (§2.3 refinement note). Existing O4-only behavior, the zero-sleep test rule, and the diagnostics sibling stay byte-unchanged.

### 4.5 `src/app/run/source.rs` — full rewrite (`ChatSource`)

```rust
//! The production chat-backed message source (Phase 06 Task 3; record
//! D-5/D-8/D-9): it owns the bounded channel's receiver + the worker's
//! shared terminal cell and drains NON-blocking (`try_recv`), so the
//! frame loop NEVER waits on chat: `Empty` → `Ok(None)` (frames keep
//! flowing through idle/slow/reconnecting chat), a queued message →
//! `Some(message)` (one per frame), `Disconnected` → consult the cell —
//! a stored retrieval error surfaces ONCE (`take`), then `Ok(None)`
//! forever; an empty cell (graceful `ChatEnded`/`offlineAt` end, or the
//! pre-spawn/post-shutdown states) yields `Ok(None)` forever.

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::app::pipeline::{ChatRetrievalError, ChatSourceError, MessageSource};
use crate::chat::ChatMessage;

pub(super) struct ChatSource {
    receiver: Receiver<ChatMessage>,
    terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
}

impl ChatSource {
    pub(super) fn new(receiver: Receiver<ChatMessage>, terminal: Arc<Mutex<Option<ChatRetrievalError>>>) -> Self {
        Self { receiver, terminal }
    }
}

impl MessageSource for ChatSource {
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError> {
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                let failure = self.terminal.lock().ok().and_then(|mut cell| cell.take());
                match failure {
                    Some(error) => Err(ChatSourceError::Retrieval(error)),
                    None => Ok(None),
                }
            }
        }
    }
}
```

(The `new` signature is 2 args ✓. The `Disconnected`-consult uses the single-section named style; if the nested match trips the depth rule, extract `take_terminal_failure(&self) -> Option<ChatRetrievalError>`.)

### 4.6 `src/app/run/error.rs` + `src/app/run/streaming.rs` — the funnel

`error.rs`: append `ChatRetrieval(ChatRetrievalError),` to `RunError` (after `GateThreadStopped`); Display arm `RunError::ChatRetrieval(error) => write!(formatter, "{error}")` (the `ChatRetrievalError` line is self-contained and actionable); import `ChatRetrievalError` from `crate::app::pipeline`. Exit code: falls through to `STARTUP_FAILURE_EXIT` (1) — NO new code; `is_stop_request_cancellation` unchanged.

`streaming.rs` gains the pipeline→run mapper (lives beside `RunError`'s helpers):

```rust
/// D-8/D-9 funnel: a chat-retrieval pipeline failure maps to the typed
/// chat variant; every other pipeline failure keeps the existing variant.
pub(super) fn pipeline_error(error: FramePipelineError) -> RunError {
    match error {
        FramePipelineError::ChatRetrieval(retrieval) => RunError::ChatRetrieval(retrieval),
        other => RunError::Pipeline(other),
    }
}
```

`primary_shutdown_outcome`'s `(Err(error), _)` arm becomes `Err(pipeline_error(error))`; every other arm and the arm ORDER are byte-unchanged (the Fatal-outranks matrix holds — §2.5).

`PreparedStreaming::run`:
- Destructures the new `source` from the bundle.
- `StreamParts { source: Box::new(source), stop_flag: stop_flag.clone(), destination }`.
- The error path becomes:

```rust
        if stream_outcome.is_err() {
            stop_flag.request_stop();
            canceller.request_cancel();
        }
```

(the stop-flag flip stops the shared-flag worker — §2.3; the canceller line is the existing D-T3-6 rule 2).
- Module doc: "chat-free loop" → "chat-backed loop"; the shutdown-order sentence gains the worker (flag-stopped concurrently; joined in `gate.finish()` after the sink EOF — the §2.3 refinement).

### 4.7 `src/app/run/seams.rs` — the transport pair

```rust
use crate::youtube::{..., LiveChatTransport, ...};

/// The chat-side shared transport handle (same Send+Sync wrapper form).
pub(super) type ChatTransport = Arc<dyn LiveChatTransport + Send + Sync>;

/// The transport factory's result: BOTH handles coerce from the ONE
/// concrete `Arc<GoogleLiveApiTransport>` (one token, one instance).
pub(super) struct BroadcastTransports {
    pub(super) live: BroadcastTransport,
    pub(super) chat: ChatTransport,
}

pub(super) type TransportFactory<'a> =
    Box<dyn FnOnce(AccessToken) -> BroadcastTransports + 'a>;
```

### 4.8 `src/app/run/streaming.rs` — `StreamingExecution`

Gains `pub(super) source: ChatSource,` (placed after `stop_flag`). Doc: the pre-built production source rides the bundle (constructed in stage 7.5). The `RecordingStreamingRunner` fixture destructures it bind-and-ignore (`source: _source`).

### 4.9 `src/youtube/mod.rs` + `live_chat_resolution.rs` — consumption + C-5 retirement

`live_chat_resolution.rs`: `fn resolve_live_chat_id_with_interval` → `pub(crate) fn` + a one-line doc note ("the Phase-06 chat worker's cfg-split test path re-injects the interval; production keeps the 5 000 ms default").

`mod.rs` (the C-5 act; exact):
- DELETE the 6 module-level `#[allow(dead_code)]` attributes + the 2 Task-3 comments above `api_chat_transport` / `google_live_chat_transport` / the chat-mod block (the worker roots the chain).
- The re-export block becomes:

```rust
pub(crate) use api_chat_transport::LiveChatTransport;
pub(crate) use live_chat_dedup::ChatDedupWindow;
pub(crate) use live_chat_mapping::{map_page, ChatPageOutcome, ChatTextMessage};
pub(crate) use live_chat_models::{ChatListCommand, LiveChatMessageList};
pub(crate) use live_chat_resolution::resolve_live_chat_id;
// Pinned Task-3 consumption surface (record C-1); the mapping helper runs
// inside youtube and stays unconsumed at the crate root.
#[allow(unused_imports)]
pub(crate) use live_chat_mapping::parse_rfc3339_seconds;
// Same pinned-surface staging: the chat path builder runs inside youtube.
#[allow(unused_imports)]
pub(crate) use live_chat_models::chat_messages_path;
#[cfg(test)]
pub(crate) use live_chat_resolution::resolve_live_chat_id_with_interval;
```

(Keep rustfmt's brace ordering; the CONTENT is what is prescribed. The 2 retained braces are pre-existing allow lines updated, NOT new allows.) The `#[cfg(test)]` re-export mirrors the existing `wait_until_stream_active_with_polling` test-only export. Module doc: append ONE compact paragraph naming the worker consumption (≤ 6 lines).

### 4.10 `src/app/run/mod.rs` — stage-7.5 wiring

- `mod chat_failure;` + `mod chat_worker;` (alphabetical in the decl block).
- In `run_outcome`'s production closure: `build_transport` returns `BroadcastTransports`:

```rust
        build_transport: Box::new(|access_token| {
            let transport = Arc::new(GoogleLiveApiTransport::new(access_token));
            BroadcastTransports {
                live: transport.clone(),
                chat: transport,
            }
        }),
```

- In `run_runtime`, after the destination reveal and BEFORE the gate spawn:

```rust
    let (source, chat_worker) = chat_worker::build_chat_stage(
        transports.chat,
        prepared.broadcast_id(),
        start.stops.stop_flag.clone(),
    );
    let gate = spawn_startup_gate(GatePlan {
        transport: transports.live,
        cancellation: start.stops.canceller.clone(),
        stream_id: stream_id.clone(),
        watch_url: watch_url.clone(),
        stop_flag: start.stops.stop_flag.clone(),
        chat_worker: Some(chat_worker),
    });
    runner.run(
        start.config,
        StreamingExecution {
            stop_flag: start.stops.stop_flag,
            canceller: start.stops.canceller,
            destination,
            source,
            stream_id,
            watch_url,
            gate,
        },
    )
```

(`transports.live` for `StageRuntime` earlier in the fn is unchanged.) Module doc: re-cut the "Live Chat is OUT of Phase 05" sentence — the run now wires the real chat worker at stage 7.5; the worker thread joins at `Confirmed`; the gate/refinement notes are named (§2.2/§2.3). Keep the doc ≤ +8 lines (the file sits at 185; the §3.2 deltas hold it ≤ 200 — if over, move the wiring into `build_chat_stage`'s file and STOP→caller with counts).

---

## 5. Offline test rosters (exact names; all fake/loopback; zero Google contact)

### 5.1 Pipeline seam fallout

`src/app/pipeline/source_error_tests.rs` (registered `#[cfg(test)] #[path]` from `source_error.rs`):
1. `chat_retrieval_error_display_lines_are_stable_and_secret_free` — the 3 exact lines; no `rtmps`/`Bearer`/token substrings.
2. `chat_source_error_delegates_both_classes_verbatim`.
3. `pipeline_source_failure_maps_chat_and_retrieval_classes` — `Chat(_) → FramePipelineError::Chat(_)`; `Retrieval(_) → FramePipelineError::ChatRetrieval(_)`.

`src/app/pipeline/error_tests.rs` (+1, APPEND):
4. `chat_retrieval_pipeline_error_display_and_source_are_pinned` — `"chat retrieval failed: live chat is unavailable: X"` shape + `source()` → Some.

`src/app/pipeline/runner_retrieval_tests.rs` (NEW sibling roster — AMENDMENT 1 ruling 1; registered `#[cfg(test)] #[path]` from `runner.rs`, repo-standard split precedent; the original `runner_tests.rs` (194 lines) is NOT extended):
5. `run_maps_retrieval_failure_to_the_chat_retrieval_variant` — `ScriptedSource::failing_retrieval(ChatRetrievalError::Transport { detail: "fixture".into() })` → `Err(FramePipelineError::ChatRetrieval(_))`; no render input; no finish. Full `FramePipeline`/`PipelineParts` assembly is mandatory (exercise-through-the-runner).

`source_helpers.rs`: `failing_retrieval(failure: ChatRetrievalError)` constructor; the existing `failing(ChatError)` keeps returning `Err(ChatSourceError::Chat(e))`. All existing runner/demo tests keep their semantics (the Chat-class path is byte-identical).

### 5.2 `src/app/run/chat_worker_tests.rs` (roster A — message path; registered from `chat_worker.rs`)

1. `worker_delivers_mapped_messages_in_page_order_with_metadata` — fake scripts [lookup → Some, list page-1 = 2 text items (ids, one multi-line text, one `published_at = "2026-10-26T01:02:03Z"`), list page-2 = `offlineAt`-ended]; drain the receiver: 2 `ChatMessage`s in order; the multi-line text arrives NORMALIZED (single line), the id verbatim, `timestamp() == parse_rfc3339_seconds("2026-10-26T01:02:03Z")` (computed, no magic literal).
2. `worker_discards_the_entire_baseline_history_page` — page-1 = 1 history item + token; page-2 = 1 live item + ended; receiver yields ONLY the live item.
3. `worker_dedup_drops_replayed_ids_across_pages` — page-1 [id=m1]; page-2 [id=m1 again, id=m2]; receiver yields m1 once + m2.
4. `worker_empty_page_advances_the_token_without_inference` — page-1 = empty items + `nextPageToken = "token-1-fixture"`; page-2 = ended; the fake's recorded commands show the SECOND command carried `page_token == Some("token-1-fixture")` (D-6: emptiness never infers the end).
5. `worker_graceful_ended_page_disconnects_without_terminal_error` — first page carries `offlineAt` → worker exits; cell empty; `try_recv` → Disconnected.
6. `worker_chat_ended_error_class_is_graceful` — list → `Err(LiveApiError::ChatEnded)` → exit; cell empty.
7. `worker_chat_disabled_failure_writes_the_chat_unavailable_terminal` — list → `Err(LiveApiError::ChatDisabled)` → cell = `ChatUnavailable { detail: "live chat is disabled for this broadcast; enable chat and reuse its id" }` (exact).
8. `worker_resolution_absence_after_budget_maps_chat_unavailable` — fake lookup → `Ok(None)` ×6 (test interval 0) → cell = `ChatUnavailable` with the `ApiNotFound { resource: "live chat" }` Display detail ("YouTube live chat not found; deleted or never created"); exit.
9. `worker_rate_limit_exceeded_is_transient_while_quota_exhaustion_is_terminal` — scripts [list → `QuotaLike { reason: "rateLimitExceeded" }` ×2, list → ended page] → NO terminal (recovered); a second worker with [list → `QuotaLike { reason: "quotaExceeded" }`] → cell = `Quota`.
10. `worker_event_constants_are_pinned_and_secret_free` — the 3 D-13 constants verbatim; no `rtmps`/`Bearer`/token substrings.

### 5.2b `src/app/run/chat_worker_budget_tests.rs` (roster B — boundedness/cancellation; registered from `chat_worker.rs`)

1. `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal` — list → `Network` ×10 → cell = `Transport` (detail = the last Network Display); exit.
2. `worker_transient_failures_recover_within_the_budget` — [Network ×2, page with `offlineAt`] → cell EMPTY; worker exited gracefully (the recovery page is the baseline page — its items are discarded by design; the test pins no terminal + clean exit).
3. `worker_backoff_delay_doubles_and_caps_at_the_maximum` — the 4 pure-fn pins (§4.3.7).
4. `worker_stop_flag_ends_the_loop_before_any_transport_call` — flag PRE-set → `run_chat_worker` returns with the fake's operation ledger EMPTY.
5. `worker_stop_during_delivery_drops_pending_messages_without_blocking` — the fake flips the shared stop flag inside `list_messages`, then returns a page with 1 message → the receiver never receives it; the worker returns.
6. `worker_blocked_list_call_cannot_prevent_bounded_shutdown` — the fake's `list_messages` sleeps ~250 ms (a stall); `spawn_chat_worker` on a thread; the test flips the flag during the stall; `join()` completes with a generous wall bound (Instant-measured ≤ 5 s) — a blocked in-flight call cannot hang the shutdown indefinitely (D-T2-16(a) carried limitation documented in the module docs).

### 5.3 `src/app/run/chat_source_tests.rs` (roster C; registered from `source.rs`)

1. `source_yields_none_while_the_channel_is_open_and_empty` — the pre-spawn state (sender alive, nothing sent) → `Ok(None)` (the frame-loop continuity pin).
2. `source_delivers_queued_messages_in_order_without_blocking` — sender sends 2 → two `Some`s in order, then `Ok(None)`.
3. `source_surfaces_the_terminal_error_once_then_yields_none` — sender dropped + cell = `Transport { detail: "fixture" }` → first call `Err(ChatSourceError::Retrieval(_))`, second call `Ok(None)`.
4. `source_disconnected_without_terminal_error_yields_none_forever` — sender dropped, cell empty (the graceful `ChatEnded` shape) → `Ok(None)` repeatedly.
5. `chat_source_integrates_through_store_and_renderer_in_the_frame_pipeline` — `chat_channel()`; the sender delivers 2 messages (the worker's job simulated); `ChatSource` + `ChatStore` (temp-log `valid_config` pattern) + `StubRenderer` + `RecordingSink::stop_after_writes(3)` assembled into the `#[cfg(test)]` `FramePipeline`/`PipelineParts` (pub(crate) test exports — reachable from the run subtree) → the rendered inputs show `author: text` lines exactly as `visible_text` joins them.

### 5.4 `src/app/run/gate_chat_tests.rs` (registered `#[cfg(test)] #[path]` from `gate.rs`; uses the Mutex-based `runtime_chat_fakes`)

1. `confirmed_gate_spawns_and_joins_the_chat_worker` — `GatePlan.chat_worker = Some(plan)` with the chat fake scripted [lookup → Some("live-chat-id-fixture"), list → ended page]; `finish()` → `GateFinish::Confirmed` AND the chat ledger == `["lookup_broadcast_chat_id", "list_messages"]` (the D-9 spawn-at-confirmation pin; joins cleanly).
2. `cancelled_gate_never_spawns_the_chat_worker` — pre-cancelled canceller script → `GateFinish::Cancelled` AND the chat ledger empty.

`gate_tests.rs`: ONLY the `gate_plan` helper gains `chat_worker: None` (all 7 existing tests keep passing byte-identically).

### 5.5 `src/app/run/error_tests.rs` (updates + 1 new)

- `all_run_errors()` gains `RunError::ChatRetrieval(ChatRetrievalError::Transport { detail: "fixture".to_string() })` (the exit-1 loop covers it).
- NEW `chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate`:
  - the Display delegates verbatim (`"live chat transport failed after repeated attempts: fixture"`);
  - `primary_shutdown_outcome(Err(FramePipelineError::ChatRetrieval(_)), GateFinish::Fatal(gate_fatal()))` → `Err(RunError::Broadcast(_))` (gate outranks — D-9 pin);
  - `primary_shutdown_outcome(Err(FramePipelineError::ChatRetrieval(_)), GateFinish::Confirmed)` → `Err(RunError::ChatRetrieval(_))` (the chat funnel is primary when the gate is healthy);
  - the exit code is 1 and NOT the cancellation class.

### 5.6 `src/app/run/runtime_tests.rs` + fixtures

- `runtime_chat_fakes.rs` (NEW, `#[cfg(test)]`, registered from `chat_worker.rs`): `ScriptedRunChatTransport` (Mutex queues + `operations` recorder + received-`ChatListCommand` recorder + an optional `stop_on_list: Mutex<Option<StopFlag>>` hook + an optional stall field for roster B test 6); page builders `chat_page(items: Vec<serde_json::Value>, token: Option<&str>, interval: Option<u64>, offline_at: Option<&str>) -> LiveChatMessageList` built via `serde_json::from_value` (visibility-safe — no field access needed); `text_message_item(id, author, text, published_at)` JSON helper; fixture strings `*-fixture` shaped only.
- The helper becomes a 1-arg parameter object (≤2-args rule):

```rust
struct ScriptedRun<'a> {
    transport: &'a Arc<ScriptedTransport>,
    chat: &'a Arc<ScriptedRunChatTransport>,
    verifier: &'a FakeVerifier,
    runner: &'a RecordingStreamingRunner,
}
fn run_with_scripted(script: ScriptedRun<'_>) -> Result<(), RunError>
```

and the factory closure: `Box::new(move |_access_token| BroadcastTransports { live: Arc::clone(script transport…), chat: Arc::clone(chat…) })`.
- `run_runtime_orchestrates_credentials_then_resources_then_streaming` EXTENDS: script the chat fake [lookup → Some, ended page]; after the existing asserts add `assert_eq!(chat.operations(), ["lookup_broadcast_chat_id", "list_messages"])` (the worker ran post-confirmation; the BROADCAST ledger assertion stays byte-identical).
- The other 4 tests pass an unscripted chat fake (never called — the ledger stays empty; assert nothing new).

### 5.7 Demo regression surface (signature-only; all existing tests keep passing)

`demo/source.rs` signature update is transparent to `source_tests.rs` (`.expect("demo source never fails")` still compiles — `ChatSourceError: Debug`). The demo schedule/MP4 pins are untouched (§9).

---

## 6. Implementation order (tiny, ordered; compile-green at each lettered checkpoint)

1. **State check:** `git status --porcelain` → EMPTY; branch `feat/phase06-youtube-live-chat`; `git log --oneline -3` shows the Task-2 tip (`f369410`) then the plan-tracking commits (caller-pinned HEAD `868b015`). If mismatched → STOP, report.
2. **Commit the plan file ONLY:** `git add .kilo/plans/20261025-phase06-task3-run-integration.md` → commit `chore: track the phase06 task3 run integration plan`. Verify `git show --stat --oneline HEAD` lists exactly this file.
3. **(a) Pipeline seam:** create `source_error.rs`; widen `source.rs`; edit `pipeline/mod.rs` re-exports; edit `runner.rs`; edit `pipeline/error.rs`; update `source_helpers.rs`; update `demo/source.rs` + `run/source.rs` + `streaming_stage_tests.rs` signatures so the crate compiles (temporarily: `run/source.rs` may keep a minimal compiling shape — it is fully rewritten in (d)).
4. **(b) YouTube widenings:** `live_chat_resolution.rs` `pub(crate)` widening + `youtube/mod.rs` re-export/staging edits per §4.9 (retire the module stagings ONLY after (c) consumes them — land §4.9's deletion in step (e)).
5. **(c) Run production:** `chat_failure.rs`; `chat_worker.rs` (with cfg-split resolution + sleeps); `run/source.rs` full rewrite; `run/error.rs`; `seams.rs`; `streaming.rs`; `gate.rs`; `run/mod.rs` wiring.
6. **(d) Tests:** `runtime_chat_fakes.rs`; all §5 rosters; the edited test files.
7. **(e) Retirement act:** the §4.9 staging deletions in `youtube/mod.rs` + the `with_timestamp` allow retirement in `chat/model.rs` (LAST — after consumption compiles).
8. **(f) Normalize + gate:** `cargo fmt --check` (mechanical rustfmt normalization on touched files only); `cargo clippy --locked -- -D warnings` (if clippy demands a NEW allow anywhere, apply ONLY the exact named allow with a comment naming Task 3, and record it — the Task-2 contingency precedent); targeted filters (§7); full gate (§7).
9. **Commit code+tests:** `git add` exactly the §3 files → commit `feat: wire the youtube live chat worker into the run frame pipeline`.
10. Final: `git status --porcelain` → empty; `git show --stat --oneline HEAD` lists exactly the §3 files. NO push, NO merge, NO TODO marks, NO docs edits.

---

## 7. Verification (4.2; ALL via Alpine VM MCP)

Pre-flight: clean tree, pinned branch/HEAD.

Targeted filters (any failure → fix → re-run before the gate):
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_worker`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_source`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked source_error`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked gate`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked runtime`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked runner`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked demo`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`

FULL gate on the final tree: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0 + `ALL CHECKS PASSED` + observed count (projected 457; OBSERVED GOVERNS; record the exact number + log path).

NO builds (builds belong to Task 4 per the TODO split and the phase convention).

Regression guards (greps + diff over the Task-3 span):
- `git diff --stat` spans ONLY §3 files (+ the plan file). Zero edits outside them.
- `git diff -- Cargo.toml Cargo.lock` → EMPTY (D-11).
- Demo/args pins green (§5.7 + the existing `args_tests`/`stream_args_tests`/`mod_tests` rosters inside the gate).
- `allow(` audit in `src/`: the count DROPS by 7 (6 module stagings + 3 single re-export stagings + the `with_timestamp` item = 10 removed; 2 brace stagings UPDATED in place, comment re-cut only) — the exact expected post-state: the pre-existing non-chat reservations + the 2 updated chat braces + `live_lifecycle`/`live_models` module stagings remain; ZERO new allow lines.
- Secret audit over the span: `ya29`/`AIza`/`Bearer ` real-shaped → 0 (fixture strings only); `rtmps://` only the pre-existing `.example.test` fixtures; no stream keys.

---

## 8. Git handling (4.2)

1. State check (§6 step 1) — STOP if unpinned/dirty.
2. Plan-tracking commit: `chore: track the phase06 task3 run integration plan`.
3. Implement §6 steps (a)–(e).
4. Verification chain (§7).
5. Feature commit: `feat: wire the youtube live chat worker into the run frame pipeline` — stage ONLY the §3 files.
6. `git status --porcelain` → empty; `git show --stat --oneline HEAD` lists exactly the §3 files. NO push (workflow step 5 owns pushes), no merge, no TODO `[DONE]` marks.

## 9. Byte-stability ledger (STOP-guards — pre-frozen pins that must NOT change)

1. `demo` schedule (`schedule.rs` timeline), `DemoMessageSource` behavior, `DemoError`/exit contracts, and the local MP4 argv (`args.rs::build_arguments`) — the ONLY demo change is the `next_message` SIGNATURE (§3.2 row 5). If any demo test fails → the seam change was done wrong → STOP.
2. The production RTMPS vector (`stream_args.rs` 33-token audio-OFF / 42-token with-audio CBR shapes) and all streaming files — ZERO edits (§3.2 has none; verify by diffstat).
3. Secret redaction: no token/header/body/stream-key/RTMPS URL ever enters a log line or error payload; the new Display lines carry ONLY redacted `LiveApiError` Display text; D-13 holds (message text/authors/user-message ids never logged; the chat-id MAY appear only in the `live_chat_id` tracing field of the resolution line).
4. Clean Ctrl+C stop → exit 0 + `RUN_STOP_MESSAGE` verbatim; pre-streaming stop → exit 0 + `RUN_STARTUP_STOP_MESSAGE` verbatim (`mod_tests.rs` pins stay green).
5. The single-`error:`-line contract and the exit-code table (no new code; `ChatRetrieval` → 1).
6. `ChatStore` log-first acceptance + the log record format — untouched.
7. `primary_shutdown_outcome`'s arm order and the existing 7 matrix rows — byte-stable; the chat rows are ADDITIVE.
8. Phase-05 startup stages 1–10 order — unchanged; the chat construction inserts as stage 7.5 without reordering anything.
9. O1–O5 request forms, `classify.rs` arms (Task 3 adds NO new classify arms), auth/verifier/credential surfaces — untouched.
10. The gate's O4-only confirmation, the zero-sleep test rule, the timeout diagnostics lines (`gate_diagnostics.rs`) — untouched.
11. `MessageSource` keeps the `Result<Option<ChatMessage>, _>` shape, the top-of-frame call point, and one-message-per-frame pacing.
12. ZERO dependency/lockfile changes; ZERO new `#[allow(` attributes; ZERO commented-out code.

STOP procedure (implementer): on any gate failure the plan's own pins cannot explain, or any line-cap overflow beyond the prescribed contingencies, or any pre-frozen pin invalidated — STOP, capture the exact output/counts, and report to the caller (never improvise a re-pick).

## 10. Recorded deviations/limitations this task documents (for 4.3/4.5b review)

| # | Item | Class |
| --- | --- | --- |
| 1 | Worker join position: after the sink EOF (§2.3) vs D-9's literal "join → EOF" | Sanctioned refinement (pipeline monolith + pinned pacing contract; flag-stop precedes; log/video consistency preserved; documented in `chat_worker.rs` + `streaming.rs` module docs) |
| 2 | Chat-id resolution runs on the worker thread post-confirmation with RAW 5 s sleeps — a Ctrl+C during the one-shot ≤30 s window delays the join by the remaining budget | Recorded limitation (Task-2's no-canceller decision; consuming the Task-2 seam per C-5 beats duplicating the loop; bounded) |
| 3 | ≤64-message shutdown loss window (queued-but-unconsumed channel messages at stop) | Recorded limitation (the log never receives them; log/video never diverge) |
| 4 | The 2 chat re-export stagings (`parse_rfc3339_seconds`, `chat_messages_path`) remain with re-cut comments (pinned surface, by design unconsumed at the crate root) | Recorded deviation candidate for 4.5b adjudication |
| 5 | `resolve_live_chat_id_with_interval` + the `#[cfg(test)]` re-export widened `pub(crate)` | Compiler/test-driven one-liner (the zero-sleep precedent family) |

## 11. Later sub-step applicability

- **4.3 review/simplification:** REQUIRED (new src files); reviewers bound to §3's inventory + §9's ledger; contested structural re-picks → caller.
- **4.4 documentation:** NOT REQUIRED in Task 3 (all phase docs, `.agent/project-structure.md`, tech/context, README test-count belong to Task 4 per the global plan; docs-specialist returns "not required", drift-check only).
- **4.5b adherence:** architector re-check against THIS plan + the record §4 + the Task-2 carry-forwards.
- **4.6 completion:** implementer marks Task 3 `[DONE]` in the TODO (+ touched acceptance criteria), commits.

## 12. Explicitly NOT done by Task 3 (flagged for Task 4 / later)

- NO user/operator docs (`docs/production-run.md` chat sections, the D-10 quota table, the D-13 log-line enumeration, the manual E2E procedure), NO `.agent/project-structure.md`/tech/context updates, NO README test-count re-measure, NO completion report, NO builds/artifact rebuilds, NO `ls -l` cross-checks — ALL Task 4.
- NO chat replay/history restore, NO moderation, NO config fields (rule count stays 21), NO Tokio (D-11), NO `live_lifecycle`/`live_models` V2 residual retirement (their consumers still do not exist — the standing V2 comments govern).
- NO real-Google verification of any kind (the manual E2E stays operator-run and is documented in Task 4).

## 13. Footer

Plan: `.kilo/plans/20261025-phase06-task3-run-integration.md`
Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — Task 3 only.
Records: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md`; `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`; `.kilo/plans/20261025-phase06-task2-adherence.md` (C-1..C-5); global plan `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
Produced 2026-10-26 by the Task-3 step-4.1b architector pass. Not committed (tracking belongs to the 4.2 implementer, §6 step 2).
