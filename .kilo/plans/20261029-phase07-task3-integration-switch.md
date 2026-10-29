# Phase 07 Task 3 — Production Run and Frame-Pipeline Integration Switch (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — **Task 3 "Production Run and Frame-Pipeline Integration Switch" only** (its bullet list is the binding requirement set; §0 maps it).
> Binding design records (already landed; this plan TRANSCRIBES them and adds Task-3's closed-form choices — it re-decides nothing already ruled):
> - `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (§5–§10 loop/bridge semantics = P7-5..P7-12, and §12 the retirement ruling).
> - `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` (the landed seam §A/§C/§L, including the **L.8 staging-retirement ledger: "all staging attributes retire at the Task-3 rewire"**).
> Owned later (NOT this task): Task 4's coverage audit, docs, builds, completion report and operator-E2E procedure. Tasks 1–2 records bind this plan; their step lists stay their own.
> Baseline: branch `feat/phase07-streamlist-retrieval`, Task 2 closed at `75788ae` (16 files), package v0.13.0, observed gate **507 passed / 0 failed** (`logs/checks/20261029T180112Z.log` — re-observed at 4.2, the fresh log governs). `git push` stays workflow step 5.
> This plan commits nothing; the 4.2 implementer commits and executes per §F/§G, writing observed results into the §L addendum. Container-only verification: every cargo command runs on the Alpine VM as `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust …`; the Windows host has NO Rust toolchain.
> Target implementer: a JUNIOR developer under 50% restriction — the plan encodes every structural choice; nothing below requires judgment.

---

## 0. Scope of this task (TODO Task-3 bullets, verbatim obligations → owning section)

| TODO Task-3 bullet | Where decided |
|---|---|
| Swap the production chat worker's retrieval mechanism from `liveChatMessages.list` polling to the streaming client per the Task-1/2 designs, keeping the spawn point (gate `Confirmed`), bounded delivery channel, stop-flag sharing, log-first `ChatStore` acceptance, one-message-per-frame pacing, and the shutdown order (worker join after sink EOF inside `gate.finish()`) byte-compatible | §B (the bridge: confined current-thread runtime thread + the resumable loop), §A.1–A.8, §D |
| Keep `demo` byte-untouched (deterministic source, schedule, local MP4 timeline and arguments) and keep every existing production wire contact unchanged | §D.1/D.7 (zero-edit ledger) |
| Ensure the frame loop remains continuous during stream establishment, reconnects, empty periods and slow startup — the video never stalls or backlogs on chat; bursts overflow only per the documented bounded policy | §A.6 (P7-9 preserved verbatim: try_send-only retain-retry, ≤64), §B.7, §E |
| Verify the latency profile end-to-end offline: from a faked/loopback push event to the message reaching the channel is bounded by the consumer loop only (no poll interval added); the operator-observable expectation text is TASK 4's docs | §E.5 (the latency pins), §I |
| Update task-level typed-error wiring: stream-class failures map to the existing `ChatRetrievalError` variants (NO new variant — the two stream classes landed in Task 2; gate-Fatal-outranks and the single-`error:`-line contract hold; no second broadcast) | §A.5 (the match-arm discipline), §B.6, §D.3/D.4 |

Out of scope for Task 3 (owning steps named): docs (`production-run.md`, structure map, `tech.md`, context.md — Task 4); release builds, the Phase-07 report, the operator manual E2E (Task 4 / OPERATOR); the live-Google TLS/handshake confirmation (operator manual E2E).

---

## §A — Transcribed rulings (the ONE source of truth; each item cites the binding record)

### A.1 The loop (record P7-5; Task-2 record §A.7)

The strategy loop is re-keyed from the poll interval to connection events. EXACT semantics:

1. **Establish** — one `open(ChatStreamRequest)` per session: Bearer metadata only (inside the landed concrete transport), the fixed `part` set, and `pageToken` sent ONLY when a retained token exists (never fabricated). Connect is already bounded at 10 s by `CHAT_STREAM_CONNECT_TIMEOUT_MS` in the landed `stream_concrete.rs` (a `tokio::time::timeout` around the connect future). An `open` error is ONE failure against the budget and enters the failure policy (§B.6).
2. **Consume** — for each delivered batch: advance `next_page_token` whenever the batch carries one; process items per the baseline/dedup rules (A.2/A.3); honor a present `offline_at`/`chatEndedEvent` (the `offline_ended` carrier flag) as the graceful end.
3. **On session end**:
   - graceful-end classes → log `CHAT_ENDED_MESSAGE`, exit cleanly (the worker returns; NO terminal-cell write);
   - transient classes → `consecutive_failures += 1`; if the budget (10) is reached → typed `ChatRetrievalError::Transport { detail = the last transient error's Display }` into the terminal cell and exit; otherwise wait `backoff_delay_ms(consecutive_failures, CHAT_SLEEP_SLICE_MS)` = 1 s × 2^(n−1) capped at 60 s (the base IS the `CHAT_SLEEP_SLICE_MS = 1_000` constant; the poll-interval concept has NO streaming counterpart), then re-establish with the last token;
   - terminal classes → write the terminal cell (`terminal_retrieval_error`), exit.
4. **NEVER periodic** — the only re-calls are the first establishment and re-establishments after a session end. An idle-but-alive session is left untouched: the stop-ticker branch polls the SAME pending future (it never restarts the read), so silence never errors.
5. **A successful delivery resets the budget** — `consecutive_failures = 0` on every delivered batch (the retired poll worker's `handle_page` reset-on-successful-page semantics; any delivered batch IS the success event).

### A.2 Resume keying + dedup across reconnects (record P7-7)

- `nextPageToken` is updated whenever a batch delivers one and is the ONLY resume key; worker-owned state; never persisted.
- The 512-window dedup (`ChatDedupWindow`) lives in the WORKER loop and never resets across reconnects (Task-1 record §12 names dedup a survivor; P7-7 "worker-owned state").
- Verification of where dedup lives today: the retired polling worker owned `ChatWorkerSession.dedup` (`chat_worker.rs` line 100) and its `deliver` loop recorded ids before conversion (`chat_worker_delivery.rs` lines 103–115) — Task 3 MOVES that exact loop into the stream worker (run-side worker-local state), keeping the greedy per-item order: skip-policy already applied by the mapper (`map_stream_batch` returns carriers filtered to text messages — skip policy applied inside `map_item`, verified in the landed `live_chat_stream_mapping.rs`), then `message_id → dedup window check → skip-or-convert → try_send delivery`.
- Re-deliveries of still-new items after a reconnect are deduped; processed messages are never duplicated into log or queue.

### A.3 Baseline/history discard — record P7-8 + the caller-directed refinement (ADJUDICATED, binding)

Official fact (record §1.1, S1): a token-less establishment "sends a series of messages containing recent chat history", with NO documented history/live boundary on the same connection.

- **Record rule (P7-8)**: the first-established session is a BASELINE session — every item it delivers is discarded (never queued, never logged); its token is retained; from the second session onward delivery is LIVE under per-item dedup; re-establishments always carry the token.
- **Refinement (this task's closed-form ruling, verified against S1 facts)**: the baseline is keyed on the first session **THAT DELIVERS ANY BATCH** — a failed connect counts as a PRE-BASELINE transient: it delivered nothing to discard, and no token exists yet, so every pre-first-batch establishment is token-less by construction and remains a history-carrying candidate. Rationale: (i) no history can exist pre-first-batch, so a failed connect has nothing to discard; (ii) the first delivered batch DOES carry official recent history, which must be discarded whole; (iii) once any token has been delivered, every later establishment is token-keyed and officially sends no replay ("resume sending messages from the point where you left off").
- **Encoded worker rule (complete and deterministic)**: a worker-lifetime flag `history_pending` starts `true`; it clears EXACTLY when a delivered batch carries a `next_page_token` (which the worker then retains). Every batch while `history_pending` is true is discarded whole. The `CHAT_BASELINE_MESSAGE` info line is emitted EXACTLY ONCE ever (the first discarded batch — with the `discarded_messages` count as today); after that first log, later history batches stay silent discards.
- A discarded batch (history, malformed, or stop-dropped) never feeds log/queue/display. The dedup window NEVER records history ids: the retired worker skipped `deliver` entirely for baseline pages (`chat_worker_delivery.rs` lines 53–61), so the window never saw them — that ordering is PRESERVED (dedup `record` runs only on the live path).

### A.4 Seam Option semantics (record A.6, landed in Task 2 — consumed as-is)

`LiveChatStreamSession::next_batch()` returns:
- `Some(Ok(YieldedChatStreamBatch))` — a decoded batch (mapped carriers + resume token + `offline_ended`);
- `Some(Err(LiveApiError))` — a classified failure (status/transport);
- `None` — EXACTLY the clean server close (`Ok(None)` from tonic's message read). Clean close = a transient Network-class session end: the loop maps `None` to the bounded re-establishment branch with the last token (no `LiveApiError` exists to map — the clean-close detail constant is §B.5).

`ChatStreamTransport::open(&mut self, request) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>` stays EXACTLY as landed — the `&mut self` receiver and the boxed-session return are NOT changed; the app-side consumes the transport by ownership (`Box`, §B.2).

### A.5 Error classes and the match-arm discipline (record §7/P7-11; Task-2 record A.4/A.5)

The funnel stays `LiveApiError → ChatRetrievalError → ChatSourceError::Retrieval → RunError::ChatRetrieval`, all variants and Display lines UNCHANGED (19 variants since Task 2; ZERO new variants in Task 3). The worker match-arm discipline is EXACTLY:

| Arriving class | Worker verdict | Mechanism (reused, not forked) |
|---|---|---|
| `ChatPrecondition` AND ≥1 batch delivered (worker-lifetime counter) | **graceful end** (chat-ended dominance; the ChatEnded graceful contract never regresses to fatal) | log `CHAT_ENDED_MESSAGE`, exit, no terminal cell |
| `ChatPrecondition` AND 0 batches delivered (establishment-time) | **terminal → `ChatUnavailable`** | `terminal_retrieval_error` |
| `InsufficientPermissions`, `ApiNotFound { resource: "live chat" }`, `AuthorizationRejected`, `InvalidChatRequest`, `ChatDisabled` (O6 only) | **terminal** (never retried) | `terminal_retrieval_error` (ChatUnavailable; `InvalidChatRequest` enters naturally: it is not in the closed transient list) |
| `QuotaLike { reason: "rateLimitExceeded" }` (the stream `RESOURCE_EXHAUSTED`) | **transient** | the CLOSED `is_transient_failure` rate-limit arm → budgeted backoff re-establishment |
| `Network`, `BackendUnavailable`, `MalformedResponse` (including the A.4 refinement `INTERNAL → MalformedResponse { field: "stream response" }`) | **transient** | closed list → budgeted re-establishment; a malformed BATCH is discarded whole; the token state stays at last-delivered; never ordinary chat |
| `QuotaLike { reason: anything else }` (the old `quotaExceeded` shape — unreachable via the gRPC classifier but kept terminal for the funnel's closedness) | **terminal → `Quota`** | `terminal_retrieval_error` quota arm |
| clean close (`None`) | **transient re-establishment** (budgeted; detail = the §B.5 constant on budget exhaustion) | the loop's clean-close arm |
| O6 resolution failure (the worker's FIRST act, D-2 unchanged) | existing behavior byte-kept | `record_terminal(terminal_retrieval_error(&error))` + return |

- `chat_failure::is_transient_failure` STAYS CLOSED — NO arm is added (the ChatPrecondition dual-branch is worker-loop policy, never the shared classifier).
- The INTERNAL→transient note (record A.4 verbatim): a mid-stream server `<INTERNAL>` maps to `MalformedResponse` (indistinguishable from a decode failure at the code level; `status.message()` is NEVER read — the §8 secret-safety rule).

### A.6 The bridge/buffer/burst contract (record P7-9/P7-10 — preserved verbatim)

- The bounded `sync_channel(64)` + the `ChatSource` `try_recv` source + the `Arc<Mutex<Option<ChatRetrievalError>>>` terminal cell + the shared `StopFlag` are BYTE-UNCHANGED (P7-9; re-use without forking — `chat_channel()` and `ChatSource` are consumed untouched).
- Delivery stays `try_send`-only: on `Full` retain the message and retry once per slice; on stop/`Disconnected` return. Burst overflow = retain+retry over the ≤64-queue + the flowing-server backpressure contract.
- The stop flag interrupts everything ≤ 1 s except the ≤ 10 s connect attempt (already timeout-bounded inside `open`).
- **Worker exits on graceful end drop the sender** (exactly today's ChatEnded semantics, verified from `chat_worker_delivery.rs` lines 62–64 and `source.rs` lines 40–43): the channel closes; already-queued messages drain through the frame loop (`try_recv`); after the queue empties the source reads `Disconnected` + empty cell → `Ok(None)` FOREVER — the frame loop never errors and continues with an empty display. This exact semantic is PRESERVED for `offline_ended` and mid-session `ChatPrecondition` ends.

### A.7 Two adjudicated implementation refinements (reviewer-facing; each recorded)

1. **Stop-tick mechanism = sleep-per-iteration, NOT `tokio::time::interval`.** P7-10 literally names `interval(CHAT_STOP_TICK_SLICE_MS)`. `interval` PANICS on a zero period, and a zero period is REQUIRED by the zero-sleep test rule (test builds must not wait 1 s). Realized with identical stop semantics: a per-iteration `tokio::time::sleep(stop_tick_slice())` branch inside a **`tokio::select! { biased; … }`** — the batch branch is polled FIRST each turn (immediate fakes always win → deterministic tests), the sleep branch is reached only when the read is pending, giving the ≤1 s stop granularity; on a tick the loop re-checks the stop flag and re-polls the SAME pending future (tonic's `Streaming::message()` progress lives in the stream state — cancel-safe; nothing is restarted). Behaviorally equivalent to P7-10's naming; recorded here for the reviewer.
2. **Thread count: ONE.** The Task-1 record §10 says the runtime runs on "the dedicated chat worker's runtime thread" and that "the worker joins its runtime thread". Landed topology: the gate thread spawns ONE worker thread whose body is `run_chat_worker`; Task 3 constructs the current-thread runtime ON THAT THREAD and `block_on`s the loop inline (no second thread, no second join). The record's "runtime thread" IS the gate-spawned worker thread; the bounded-join argument is unchanged and simpler (the join IS the runtime's end). The `tech.md` wording belongs to Task 4.

### A.8 Runtime confinement (record §10, binding)

- ONE runtime instance per chat worker: `tokio::runtime::Builder::new_current_thread().enable_all().build()` constructed ONCE on the worker thread; NEVER in the frame loop, never in `auth`, never a second instance (the §F tree grep verifies exactly two `tokio::runtime` sites: this one + the test-fixture `stream_test_block_on`).
- The runtime build returns `io::Result`: on failure the worker writes the typed `Transport` terminal (§B.4 constant) and returns — no panic anywhere in the bridge.
- Cross-thread outputs ONLY: the `try_send` into the channel, the terminal-cell write, and the `StopFlag` read. No `tokio::sync::mpsc` and no other async type crosses the app seam (record §3/C2 superseded; verified by grep).

---

## §B — THE BRIDGE (exact junior-executable design)

### B.1 File layout decision (adjudicated against the 200-line caps; keeps the ≤2-args rule)

Current facts (observed on the tree): `src/app/run/chat_worker.rs` = 185 lines (constants, channel bundle, plan/constructor, worker entry, the O6 cfg-split resolve pair, fakes/roster registration); `src/app/run/chat_worker_delivery.rs` = 187 lines (the poll cycle, dedup→convert→try_send delivery, stop-sliced sleeps, the capped backoff with its pinned `manual_clamp` body, the terminal writer).

DECISION (closed-form):

| File | Fate | Content | Line budget |
|---|---|---|---|
| `src/app/run/chat_worker.rs` | REWRITE | module doc (the streaming path), shared constants, `ChatChannel` + `chat_channel()` byte-unchanged, `ChatWorkerPlan` (new field set, §B.3), `spawn_chat_worker`, `build_chat_stage` (3-arg shape byte-compatible), `run_chat_worker` (stop check → O6 resolve → runtime build → `block_on`), the cfg-split `resolve_chat_id` pair (verbatim), the `chat_runtime()` helper, `#[cfg(test)]` registrations (fakes + 4 rosters) | ≤160 |
| `src/app/run/chat_stream_worker.rs` | NEW | the async machine: `StreamWorker` struct (plan + loop state fused), `CHAT_STOP_TICK_SLICE_MS` + cfg-split `stop_tick_slice_ms()`, `STREAM_CLEAN_CLOSE_DETAIL`, `ConsumeVerdict` enum, `run_stream_loop` (the re-establishment loop), `establish`, `consume` (the biased-select consume loop) | ≤170 |
| `src/app/run/chat_stream_delivery.rs` | NEW | `impl StreamWorker` batch/failure side: `process_batch`, `deliver`, `send_stop_aware` (verbatim body), `into_chat_message` (verbatim), `sleep_in_stop_slices` + cfg-split `sleep_slice` (verbatim), `backoff_delay_ms` (verbatim body; its `#[allow(clippy::manual_clamp)]` MOVES with it), `record_terminal` (verbatim), and the failure policy (`is_mid_session_chat_end`, `handle_failure`, `handle_clean_close`, `await_reestablishment`) | ≤170 |
| `src/app/run/chat_worker_delivery.rs` | DELETE (wholesale) | the poll machinery dies; the survivor fns listed above move byte-verbatim | — |
| `src/app/run/mod.rs` | EDIT | `mod chat_worker_delivery;` → `mod chat_stream_delivery;` + `mod chat_stream_worker;` (net +1 line; PRE-AUTHORIZED one-doc-line trim, §B.8) | ≤200 |
| `src/app/run/seams.rs` | EDIT | add `ChatStageSources` + the `ChatStreamHandle` alias; `BroadcastTransports` gains ONE field `stream` (name kept — no churn); the `use` lines gain `ChatStreamTransport` | ≤90 |

Rationale for the split: the async machine and the delivery/failure halves each sit near the cap when fused; the landed "delivery-machinery separation" shape (the AMENDMENT-2 lineage) is preserved while the poll half dies. The `impl StreamWorker` split across files follows the landed `ChatWorkerSession` precedent (`chat_worker.rs` + `chat_worker_delivery.rs`).

### B.2 The handle: stream-transport injection and the access-token handoff (verified from the actual files)

- How the worker obtains credentials TODAY: `run/mod.rs::run_outcome` owns `build_transport: Box<dyn FnOnce(AccessToken) -> BroadcastTransports + 'a>` — the ONE closure receives the verified `AccessToken` by value and builds `Arc<GoogleLiveApiTransport>` for BOTH the `live` and `chat` handles (`BroadcastTransports { live, chat }`).
- Exact new handoff (`AccessToken` is `Clone` — verified at `src/youtube/token.rs` line 16; no `secret()` read leaves the youtube module):

```rust
// src/app/run/mod.rs — the run_outcome build_transport closure:
build_transport: Box::new(|access_token| {
    let stream = Box::new(GoogleLiveChatStreamTransport::new(access_token.clone()));
    let transport = Arc::new(GoogleLiveApiTransport::new(access_token));
    BroadcastTransports {
        live: transport.clone(),
        chat: transport,
        stream,
    }
}),
```

- `seams.rs` additions (exact):

```rust
/// The chat stream side (Phase-07): the run factory hands the chat worker a
/// boxed `ChatStreamTransport`; the trait's `Send` superbound makes the box
/// thread-movable into the confined worker thread, and `open(&mut self)` is
/// the landed A.6 signature — the worker owns the value outright.
pub(super) type ChatStreamHandle = Box<dyn ChatStreamTransport>;

/// The chat stage constructor's sources (≤2-args param-object rule): the O6
/// lookup seam (unchanged) plus the Phase-07 stream seam.
pub(super) struct ChatStageSources {
    pub(super) lookup: ChatTransport,
    pub(super) stream: ChatStreamHandle,
}
```

- `crate::youtube` must export the concrete producer and the carrier: the Phase-07 re-export brace (§C.5) drops its `allow(unused_imports)` and gains `GoogleLiveChatStreamTransport` + `ChatTextMessage`.
- `BroadcastTransports` gains `pub(super) stream: ChatStreamHandle,` — every constructor site is updated in the same pass (the `run_outcome` closure + the runtime-test closure).

### B.3 The worker plan (exact field set)

```rust
pub(super) struct ChatWorkerPlan {
    pub(super) lookup: ChatTransport,       // O6 resolution (the same Arc handle as today's `transport`)
    pub(super) stream: ChatStreamHandle,    // the Phase-07 retrieval seam (boxed; owned by the loop)
    pub(super) broadcast_id: String,
    pub(super) sender: SyncSender<ChatMessage>,
    pub(super) terminal: Arc<Mutex<Option<ChatRetrievalError>>>,
    pub(super) stop_flag: StopFlag,
}
```

- `spawn_chat_worker(plan: ChatWorkerPlan) -> JoinHandle<()>` stays byte-identical.
- The 3-arg constructor SHAPE is preserved byte-compatible (the sanctioned D-T3-15 exception family), so `gate.rs` touches NOTHING:

```rust
pub(super) fn build_chat_stage(
    chat: ChatStageSources,      // was `chat: ChatTransport`; the param object keeps the ARITY
    broadcast_id: &str,
    stop_flag: StopFlag,
) -> (ChatSource, ChatWorkerPlan)
```

- The old `ChatWorkerSession` struct is DELETED (its fields split between `StreamWorker` and the retired poll state).

### B.4 The worker entry — exact body (`chat_worker.rs`)

```rust
/// The confined runtime start failure reason (a typed Transport detail; no
/// panic — a runtime that cannot start is a typed terminal).
const CHAT_RUNTIME_START_FAILURE: &str = "the confined chat streaming runtime failed to start";

fn run_chat_worker(plan: ChatWorkerPlan) {
    if plan.stop_flag.is_requested() {
        return;
    }
    let live_chat_id = match resolve_chat_id(plan.lookup.as_ref(), &plan.broadcast_id) {
        Ok(id) => {
            info!(live_chat_id = %id, "{}", CHAT_ID_RESOLVED_MESSAGE);
            id
        }
        Err(error) => {
            record_terminal(&plan.terminal, terminal_retrieval_error(&error));
            return;
        }
    };
    let Ok(runtime) = chat_runtime() else {
        record_terminal(&plan.terminal, ChatRetrievalError::Transport {
            detail: CHAT_RUNTIME_START_FAILURE.to_string(),
        });
        return;
    };
    runtime.block_on(chat_stream_worker::run_stream_loop(
        StreamWorker::new(plan, live_chat_id),
    ));
}

/// ONE runtime instance for the whole app (record §10): the current-thread
/// scheduler confined to this worker thread, built once per run.
fn chat_runtime() -> Result<tokio::runtime::Runtime, ()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ())
}
```

(`resolve_chat_id` cfg-split pair kept verbatim: production `crate::youtube::resolve_live_chat_id`, test `resolve_live_chat_id_with_interval` with `CHAT_ID_RESOLUTION_TEST_INTERVAL_MS = 0`. `record_terminal`/`terminal_retrieval_error` import from `super::chat_stream_delivery` and `super::chat_failure`.)

### B.5 The async machine — exact skeleton (`chat_stream_worker.rs`)

```rust
//! [module doc: the resumable streamList loop of the Phase-07 chat worker
//! (§A.1..A.4): establish → consume → bounded transient re-establishment
//! with the last token; history discard keyed on the first delivered batch;
//! worker-owned token/dedup/history state; the only cross-thread outputs
//! are the bounded channel, the terminal cell, and the stop-flag read.]

pub(super) const CHAT_STOP_TICK_SLICE_MS: u64 = 1_000; // P7-6 stream family

#[cfg(not(test))]
fn stop_tick_slice_ms() -> u64 { CHAT_STOP_TICK_SLICE_MS }
#[cfg(test)]
fn stop_tick_slice_ms() -> u64 { 0 } // the zero-sleep test rule

/// The clean close carries no status to classify (record §A.6); this is the
/// ONLY sanctioned non-secret detail for its budget-exhaustion terminal.
const STREAM_CLEAN_CLOSE_DETAIL: &str = "the stream connection closed without an error status";

pub(super) struct StreamWorker {
    plan: ChatWorkerPlan,
    dedup: ChatDedupWindow,
    live_chat_id: String,
    page_token: Option<String>, // P7-7: the ONLY resume key, worker-owned
    history_pending: bool,      // §A.3: true until a delivered token
    baseline_logged: bool,      // the ONE-ever CHAT_BASELINE_MESSAGE charge
    delivered_batches: u32,     // §A.5 ChatPrecondition branch key
    consecutive_failures: u32,
}

impl StreamWorker {
    fn new(plan: ChatWorkerPlan, live_chat_id: String) -> Self {
        Self {
            plan,
            dedup: ChatDedupWindow::new(),
            live_chat_id,
            page_token: None,
            history_pending: true,
            baseline_logged: false,
            delivered_batches: 0,
            consecutive_failures: 0,
        }
    }
}

/// How one consuming session ended (record §A.6 Option semantics).
enum ConsumeVerdict {
    GracefulEnd,
    Stopped,
    CleanClose,
    Failed(LiveApiError),
}

/// The re-establishment loop (P7-5): establish → consume → classify the
/// end; transient paths count the failure, wait the capped backoff, and
/// re-establish with the last token. Terminal/graceful/stop exit.
pub(super) async fn run_stream_loop(mut worker: StreamWorker) {
    loop {
        if worker.plan.stop_flag.is_requested() {
            return;
        }
        let session = match worker.establish().await {
            Ok(session) => session,
            Err(error) => {
                if worker.handle_failure(error) {
                    return;
                }
                continue;
            }
        };
        match worker.consume(session).await {
            ConsumeVerdict::GracefulEnd => {
                info!("{}", CHAT_ENDED_MESSAGE);
                return;
            }
            ConsumeVerdict::Stopped => return,
            ConsumeVerdict::Failed(error) => {
                if worker.handle_failure(error) {
                    return;
                }
                continue;
            }
            ConsumeVerdict::CleanClose => {
                if worker.handle_clean_close() {
                    return;
                }
                continue;
            }
        }
    }
}

/// One quota-billed open (P7-5 step 1): the chat id ALWAYS; the retained
/// token ONLY when one exists (never fabricated).
async fn establish(&mut self) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
    self.plan
        .stream
        .open(ChatStreamRequest {
            live_chat_id: self.live_chat_id.clone(),
            page_token: self.page_token.clone(),
        })
        .await
}

/// The consuming loop (P7-10 as adjudicated in §A.7.1): biased select over
/// the pending batch read and the one-slice stop-tick branch; the SAME
/// pending future is re-polled on every tick (never restarted).
async fn consume(&mut self, mut session: Box<dyn LiveChatStreamSession>) -> ConsumeVerdict {
    loop {
        if self.plan.stop_flag.is_requested() {
            return ConsumeVerdict::Stopped;
        }
        let delivery = tokio::select! {
            biased;
            batch = session.next_batch() => match batch {
                Some(delivery) => delivery,
                None => return ConsumeVerdict::CleanClose,
            },
            _ = tokio::time::sleep(Duration::from_millis(stop_tick_slice_ms())) => {
                if self.plan.stop_flag.is_requested() {
                    return ConsumeVerdict::Stopped;
                }
                continue;
            }
        };
        match delivery {
            Ok(batch) => {
                self.delivered_batches = self.delivered_batches.saturating_add(1);
                self.consecutive_failures = 0; // §A.1.5 reset-on-delivery
                let offline_ended = batch.offline_ended;
                self.process_batch(batch);
                if offline_ended {
                    return ConsumeVerdict::GracefulEnd;
                }
            }
            Err(error) => return ConsumeVerdict::Failed(error),
        }
    }
}
```

(The graceful end is logged EXACTLY ONCE per end: in `run_stream_loop`'s `GracefulEnd` arm and in `handle_failure`'s mid-session-precondition arm — never both. Imports: `std::time::Duration`, `tracing::info`, `crate::youtube::{ChatDedupWindow, ChatStreamRequest, LiveChatStreamSession, LiveApiError}`, `super::chat_worker::{ChatWorkerPlan, CHAT_ENDED_MESSAGE}`, and `use tokio::select!;` is implicit in the macro path.)

### B.6 The failure policy (`chat_stream_delivery.rs`) — exact bodies

```rust
/// §A.5: FAILED_PRECONDITION is graceful EXACTLY when the chat had been
/// alive long enough to deliver a batch (named single-section predicate).
fn is_mid_session_chat_end(error: &LiveApiError, delivered_batches: u32) -> bool {
    matches!(error, LiveApiError::ChatPrecondition { .. }) && delivered_batches > 0
}

impl StreamWorker {
    /// One establishment/consume failure; `true` = the loop returns
    /// (terminal written, or graceful end). `is_transient_failure` STAYS
    /// CLOSED (record §7) — no arm is added.
    fn handle_failure(&mut self, error: LiveApiError) -> bool {
        if is_mid_session_chat_end(&error, self.delivered_batches) {
            info!("{}", CHAT_ENDED_MESSAGE);
            return true;
        }
        if !is_transient_failure(&error) {
            record_terminal(&self.plan.terminal, terminal_retrieval_error(&error));
            return true;
        }
        self.await_reestablishment(error.to_string())
    }

    /// Clean close = the transient re-establishment arm (§A.4/A.5).
    fn handle_clean_close(&mut self) -> bool {
        self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string())
    }

    /// The shared transient path (P7-5.3): budget exhaustion writes the
    /// Transport terminal keyed on the LAST transient detail; otherwise the
    /// capped 1 s × 2^(n−1) backoff precedes re-establishment. The poll
    /// worker's budget test (>=) is kept byte-exact.
    fn await_reestablishment(&mut self, last_detail: String) -> bool {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES {
            record_terminal(
                &self.plan.terminal,
                ChatRetrievalError::Transport { detail: last_detail },
            );
            return true;
        }
        let delay = backoff_delay_ms(self.consecutive_failures, CHAT_SLEEP_SLICE_MS);
        sleep_in_stop_slices(delay, &self.plan.stop_flag);
        false
    }
}
```

(Imports here: `super::chat_failure::{is_transient_failure, terminal_retrieval_error}` (both `pub(super)`, unchanged), `super::chat_worker::{CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES, CHAT_SLEEP_SLICE_MS}`, `super::chat_stream_worker::{StreamWorker, STREAM_CLEAN_CLOSE_DETAIL}`, `crate::app::pipeline::ChatRetrievalError`, `tracing::info`, `crate::youtube::LiveApiError`.)

### B.7 The batch processing + delivery (`chat_stream_delivery.rs`) — exact bodies

```rust
impl StreamWorker {
    /// §A.3 exact rule: advance/retain the token FIRST (clearing the
    /// history flag exactly when a token arrives); a batch under the
    /// pending-history flag is discarded whole; else the per-item live path.
    fn process_batch(&mut self, batch: YieldedChatStreamBatch) {
        if batch.next_page_token.is_some() {
            self.page_token = batch.next_page_token;
            self.history_pending = false;
        }
        if self.history_pending {
            if !self.baseline_logged {
                self.baseline_logged = true;
                info!(
                    discarded_messages = batch.messages.len(),
                    "{}", CHAT_BASELINE_MESSAGE
                );
            }
            return;
        }
        self.deliver(batch.messages);
    }

    /// The greedy per-item path MOVED VERBATIM from the retired poll
    /// delivery (dedup → convert → stop-aware try_send); `None` ids are
    /// always-unseen (the mapper can produce them — stream roster pin 12).
    fn deliver(&mut self, messages: Vec<ChatTextMessage>) {
        for carrier in messages {
            let unseen = match carrier.message_id.as_deref() {
                Some(id) => self.dedup.record(id),
                None => true,
            };
            if !unseen {
                continue;
            }
            let message = into_chat_message(&carrier);
            self.send_stop_aware(message);
        }
    }

    /// Stop-aware bounded send (P7-9 byte-unchanged D-9 shape): try_send
    /// only; full → retain and retry once per slice; stop/disconnect →
    /// return (never blocks shutdown; runs on the confined runtime thread,
    /// so the ≤1 s std sleep is the sanctioned slice mechanism as today).
    fn send_stop_aware(&self, mut message: ChatMessage) {
        loop {
            if self.plan.stop_flag.is_requested() {
                return;
            }
            match self.plan.sender.try_send(message) {
                Ok(()) => return,
                Err(TrySendError::Full(returned)) => {
                    message = returned;
                    sleep_slice(CHAT_SLEEP_SLICE_MS);
                }
                Err(TrySendError::Disconnected(_)) => return,
            }
        }
    }
}

/// The C-1 conversion point (record D-4/D-8) — byte-verbatim from the
/// retired file: the app side builds `ChatMessage`; normalization runs in
/// `ChatMessage::new`.
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

/// Bounded stop-sliced wait (D-6) — byte-verbatim pair (the sync slice fn
/// with the existing cfg-split: production `thread::sleep`, test no-op).
pub(super) fn sleep_in_stop_slices(total_ms: u64, stop_flag: &StopFlag) {
    let mut remaining_ms = total_ms;
    while remaining_ms > 0 {
        if stop_flag.is_requested() {
            return;
        }
        let slice_ms = remaining_ms.min(CHAT_SLEEP_SLICE_MS);
        sleep_slice(slice_ms);
        remaining_ms -= slice_ms;
    }
}

pub(super) fn backoff_delay_ms(attempt: u32, base_ms: u64) -> u64 {
    // Phase-06 Task 3: the D-6 pinned max/min body (record §4.3.7) — the
    // clippy-named allow keeps the plan's shape (plan §6(f)).
    #[allow(clippy::manual_clamp)]
    let base = base_ms.max(CHAT_SLEEP_SLICE_MS).min(CHAT_BACKOFF_MAX_MS);
    let doubled = base.saturating_mul(2u64.saturating_pow(attempt - 1));
    doubled.min(CHAT_BACKOFF_MAX_MS)
}

/// Stores the terminal failure exactly once — byte-verbatim.
pub(super) fn record_terminal(
    cell: &Mutex<Option<ChatRetrievalError>>,
    error: ChatRetrievalError,
) {
    if let Ok(mut recorded) = cell.lock() {
        *recorded = Some(error);
    }
}
```

(Imports: `std::sync::mpsc::TrySendError`, `std::sync::Mutex`, `tracing::info`, `crate::app::pipeline::{ChatRetrievalError, StopFlag}`, `crate::chat::ChatMessage`, `crate::youtube::{ChatDedupWindow, ChatTextMessage, YieldedChatStreamBatch}`, `super::chat_worker::{ChatWorkerPlan — only if the method impls reference it through the field, CHAT_BACKOFF_MAX_MS, CHAT_BASELINE_MESSAGE, CHAT_SLEEP_SLICE_MS}`, `StreamWorker` from `super::chat_stream_worker`.)

### B.8 Run-runtime wiring edits (`run/mod.rs` + `seams.rs` — exhaustive)

1. Module block: replace `mod chat_worker_delivery;` with `mod chat_stream_delivery;` and `mod chat_stream_worker;` (alphabetical order: after `chat_worker`).
2. Module doc: the Phase-06 stage-7.5 sentence (lines 12–14) condenses to ONE line — `//! Phase-07: the chat worker's retrieval is the streamList gRPC client behind a confined current-thread runtime on the worker thread.` — keeping the file ≤200 (net +1 line).
3. The `build_transport` closure per §B.2 (the stream transport built from `access_token.clone()` FIRST; the `Arc<GoogleLiveApiTransport>` after — one token, two constructors).
4. The `run_runtime` chat-stage segment becomes: `let (source, chat_worker) = chat_worker::build_chat_stage(ChatStageSources { lookup: transports.chat, stream: transports.stream }, &broadcast_id, start.stops.stop_flag.clone());` — everything else in `run_runtime`/`StreamingExecution`/gate plan is byte-unchanged.

### B.9 Constants ledger after the switch (P7-6)

| Constant | Value | Home | Note |
|---|---|---|---|
| `CHAT_CHANNEL_CAPACITY` | 64 | chat_worker.rs | unchanged |
| `CHAT_SLEEP_SLICE_MS` | 1_000 | chat_worker.rs | the retain-retry slice AND the backoff base |
| `CHAT_BACKOFF_MAX_MS` | 60_000 | chat_worker.rs | unchanged |
| `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES` | 10 | chat_worker.rs | unchanged |
| `CHAT_STREAM_CONNECT_TIMEOUT_MS` | 10_000 | stream_client.rs | landed Task 2 |
| NEW `CHAT_STOP_TICK_SLICE_MS` | 1_000 | chat_stream_worker.rs (private) | §A.7.1 |
| NEW `STREAM_CLEAN_CLOSE_DETAIL` | fixed string | chat_stream_worker.rs (private) | §B.5 |
| NEW `CHAT_RUNTIME_START_FAILURE` | fixed string | chat_worker.rs (private) | §B.4 |
| `DEFAULT_CHAT_POLL_INTERVAL_MS` | — | DELETED | no poll interval exists |
| `CHAT_ID_RESOLVED_MESSAGE` / `CHAT_BASELINE_MESSAGE` / `CHAT_ENDED_MESSAGE` | unchanged texts | chat_worker.rs | the pinned rosters keep them |

### B.10 Cap contingency (pre-authorized; same class as Task-2's C.8 doc trim)

If rustfmt pushes any rewritten/new source file past 200 lines: trim THAT file's module doc to terse one-sentence lines (no code moves, no new split). A deeper split ⇒ STOP and ask the caller.

---

## §C — RETIREMENT STEP-SET (Task-1 record §12 + the L.8 ledger)

Ruling from the record: the retirement set executes ONLY HERE (Task-2 record §D), in this task, atomically with the rewire (nothing ever sits as dead code between steps).

### C.1 Trait shrink — `src/youtube/api_chat_transport.rs` (REWRITE)

- DELETE `fn list_messages(&self, command: &ChatListCommand) -> Result<LiveChatMessageList, LiveApiError>;` from `LiveChatTransport`.
- DELETE the `ChatListCommand, LiveChatMessageList` import line.
- REWRITE the module/trait docs: the seam is now `lookup_broadcast_chat_id` ONLY (O6); the O7 list retired per the Task-1 record §12 ruling (name the record path + the 2026-10-29 user ruling). Keep the "no streaming/concurrency semantics" and the offline-test sentences that still hold.

### C.2 Concrete REST impl — `src/youtube/google_live_chat_transport.rs` (REWRITE)

- DELETE the whole `list_messages` method (its `chat_messages_path` + `CHAT_MESSAGES_RESOURCE` usage dies with it).
- KEEP `lookup_broadcast_chat_id` VERBATIM (the O6 wire path `broadcast_chat_lookup_path`; the shared engine + status-first classifier sentences stay true).
- REWRITE the doc to lookup-only.

### C.3 REST list models — `src/youtube/live_chat_models.rs` (REWRITE to the O6 survivor)

- DELETE `LiveChatMessageList`, `LiveChatMessageRecord`, `LiveChatMessageSnippet`, `TextMessageDetails`, `LiveChatAuthorDetails`, `ChatListCommand`, `chat_messages_path`, `MAX_CHAT_RESULTS`, `CHAT_MESSAGES_RESOURCE`, and the `serde::Deserialize` derive + import (the file no longer deserializes anything).
- KEEP `broadcast_chat_lookup_path` VERBATIM + its doc (the `encode_query_value` import survives).

### C.4 Poll-loop wiring — the worker file fates

`chat_worker_delivery.rs` DELETED wholesale (§B.1); the retired fns there (`poll_cycle`, `handle_page`, `handle_list_failure`, `retry_transient`, `is_graceful_end`, `poll_interval_ms`, `ChatWorkerSession`) die; the survivor machinery lists move byte-verbatim into `chat_stream_delivery.rs` (§B.6/§B.7). `DEFAULT_CHAT_POLL_INTERVAL_MS` is deleted.

### C.5 Staging attributes — the L.8 retirement ledger (all 9, exact anchors)

In `src/youtube/mod.rs` (module declarations block):

- DELETE `#[allow(dead_code)]` + its preceding comment line (`// Staged for the Task-3 streamList worker rewire (Phase-07); retire when the rewire consumes this surface.`) on FIVE decls: `live_chat_stream_mapping`, `stream_client`, `stream_concrete`, `stream_proto`, `stream_transport`. The decl lines themselves STAY.
- In `src/youtube/live_error/mod.rs`: DELETE the `#[allow(dead_code)]` + comment above `mod grpc_status;`, AND the two item-level `#[allow(dead_code)]` + comment lines above the `ChatPrecondition` and `InvalidChatRequest` variants (their typed doc comments STAY; only the staging attributes + the staging comment lines go).
- The Phase-07 re-export brace (`src/youtube/mod.rs` lines 154–161): REMOVE `#[allow(unused_imports)]` (the brace becomes live when the worker imports through it) and REWRITE the comment:

```rust
// Phase-07 landed surface (Task-3 rewire; record P7-12): the run chat
// worker consumes the stream seam, the concrete transport, and the
// mapped carrier.
pub(crate) use stream_transport::{
    ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession, YieldedChatStreamBatch,
};
pub(crate) use stream_concrete::GoogleLiveChatStreamTransport;
pub(crate) use live_chat_mapping::ChatTextMessage;
```

- Phase-06 re-export blocks that retire WITH the retired surfaces (same file):
  - the `map_page`/`ChatPageOutcome` brace (lines 135–138, incl. its `allow(unused_imports)` + comment): DELETE whole block.
  - the `parse_rfc3339_seconds` staging brace (lines 141–144): DELETE — no crate-root consumer remains (the stream mapper consumes the parser INTERNALLY in youtube).
  - the `chat_messages_path` brace (lines 145–147): DELETE.
  - `pub(crate) use live_chat_models::{ChatListCommand, LiveChatMessageList};` (line 139): DELETE.
- Post-edit inventory: `git grep -n "allow(" src/youtube` shows the STANDING set only: the two V2 module stagings (`live_lifecycle`, `live_models`) + `AuthorizationError::ProviderDeclined` (`error.rs`) + the two `authorization.rs` reservations = **5 dead-code attribute lines in src/youtube; ZERO staging; ZERO new**; `#[allow(clippy::manual_clamp)]` counts as 1 line in `src/` — MOVED (not added) into `chat_stream_delivery.rs`.

### C.6 Mapping-layer surviving surgery

- `src/youtube/live_chat_mapping.rs` (REWRITE to "the carrier + parser seam"): KEEP the `#[path = "live_chat_rfc3339.rs"] mod live_chat_rfc3339;` child + its `parse_rfc3339_seconds` re-export (the stream mapper consumes both), KEEP `ChatTextMessage` (verbatim) and `non_blank_text` (`pub(super)`, verbatim). DELETE `ChatPageOutcome`, `map_page`, `map_record`, `is_text_message`, `non_blank_message_text`, `resolve_author`, `page_marked_ended`, `list_shows_offline`, `marks_ended`, the `live_chat_models` imports, and the two type-string constants. Rewrite the doc to the survivor state.
- The skip-policy COVERAGE over the proto wire lives in the Task-2 roster `live_chat_stream_mapping_tests` (12 tests, unchanged).

### C.7 Roster sunset (the 51-run-note era; the OBSERVED surviving count is recorded at gate, never doc-hardcoded)

| Roster | Ruling | Detail |
|---|---|---|
| `src/youtube/live_chat_models_tests.rs` | DELETE (6 tests) | all six pin the deleted list models/decode forms |
| `src/youtube/live_chat_mapping_tests.rs` | DELETE file (11 tests) | ten pin `map_page` (retired); the ONE parser test `rfc3339_parsing_covers_documented_utc_forms` MOVES into the NEW `src/youtube/live_chat_rfc3339_tests.rs` (registered from `live_chat_rfc3339.rs` via `#[cfg(test)] #[path = "live_chat_rfc3339_tests.rs"] mod live_chat_rfc3339_tests;`) — body verbatim including the pinned `1_792_931_445` literals |
| `src/youtube/live_chat_wire_checks_tests.rs` | REWRITE (6 → 3) | DELETE the four O7 pins (`chat_list_sends_bearer_only_with_documented_query`, `chat_list_appends_page_token_only_when_present`, `chat_list_response_decodes_page_fields`, `chat_list_malformed_body_is_typed_error`) and the `list_command` helper; KEEP `broadcast_chat_lookup_reads_snippet_live_chat_id` — EXTENDED to also assert the captured head's `authorization` == `Bearer access-token-fixture` and that the head contains NO `key=` substring (restores the REST-side Bearer-only loopback coverage per Task-1 record §8) — and KEEP `chat_disabled_error_body_classifies_as_live_chat_disabled` |
| `src/youtube/live_chat_wire_tests.rs` | KEEP VERBATIM | the `ChatWireServer` + capture helpers live for the two surviving O6 pins |
| `src/youtube/live_chat_test_fixtures.rs` | REWRITE (shrink) | `ScriptedLiveChatTransport` loses `list_messages` + `list_results`; DELETE `live_chat_list_body` (only the retired O7 pins consumed it); keep the three fixture constants + the `next_result` helper |
| `src/youtube/live_chat_dedup_tests.rs` | KEEP | unchanged (3) |
| `src/youtube/live_chat_resolution.rs` + `_tests.rs` | KEEP | O6 stays byte-green unchanged |
| `src/youtube/stream_*` rosters + `live_error_grpc_status_tests` | KEEP | Task-2 rosters unchanged |
| NEW `src/youtube/stream_run_fixtures.rs` | NEW (fixture module; 0 tests) | registered via `#[cfg(test)] #[path = "stream_run_fixtures.rs"] mod stream_run_fixtures;` from `stream_test_fixtures.rs` (the parent file sits at 198 lines). Purpose: the RUN-side offline stream double so `tonic::async_trait` STAYS OUT of `src/app` (Task-2 §F.8 hygiene pin intact). Shape: `RunChatStreamLedger` — an Arc-shared script/observation cell (`Mutex`-guarded): a queue of session specs `Result<VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>, LiveApiError>`, the recorded `ChatStreamRequest` opens, an operations label ledger, and `Option<(usize, StopFlag)> stop_on_open_from` (the flip hook fires inside `open` from the armed 1-based index onward). `ScriptedRunStreamTransport { ledger: Arc<RunChatStreamLedger> }` implements `ChatStreamTransport` through the ledger: session opening pops the next spec; batches pop from the session queue, empty ⇒ `None` (clean close). EXHAUSTION = the fake PANICS with an `expect("the test scripted a queued session")`-style message (the `live_chat_test_fixtures::next_result` precedent — a deterministic test failure, never a fabricated retry: a typed-exhaustion error would make the harness's re-establishment loop spin forever; every scripted run therefore TERMINATES deterministically). Script/observe fns ≤2 args: `script_session(deliveries)`, `script_open_failure(error)`, `flip_stop_on_open_from(from_call, flag)`, `operations() -> Vec<&'static str>`, `opened_requests() -> Vec<ChatStreamRequest>` |
| `src/app/run/chat_worker_tests.rs` | REWRITE | §E.1 |
| `src/app/run/chat_worker_flow_tests.rs` | REWRITE | §E.2 |
| `src/app/run/chat_worker_budget_tests.rs` | REWRITE | §E.3 |
| `src/app/run/gate_chat_tests.rs` | REWRITE | §E.4 |
| `src/app/run/runtime_chat_fakes.rs` | REWRITE | lookup-only `ScriptedRunChatTransport` (the list arm + `chat_page`/`text_message_item` + the stall/flip/list fields/methods gone — the stream flip hook lives in the youtube-side run double); a `ChatFakes` param object `{ lookup: Arc<ScriptedRunChatTransport>, stream_ledger: Arc<RunChatStreamLedger> }` (keeps every helper ≤2 params); `worker_plan(fakes: &ChatFakes, channel: &ChatChannel) -> ChatWorkerPlan` boxes `ScriptedRunStreamTransport::new(Arc::clone(&fakes.stream_ledger))` and clones the lookup Arc; the `run_worker(fakes) -> ChatChannel` harness stays |
| `src/app/run/runtime_tests.rs` | EDIT (test 1 ONLY) | the orchestration script: the stream ledger scripted with ONE offline-ended batch (graceful end — `yielded(&response_offline(vec![]))`); asserts become `chat.operations() == ["lookup_broadcast_chat_id"]` and `stream_ledger.operations() == ["open"]`; tests 2–5 untouched |
| `src/app/run/chat_source_tests.rs`, `gate_tests.rs`, `mod_tests.rs`, `streaming_stage_tests.rs`, `streaming_diagnostics_tests.rs`, `error_tests.rs`, `credentials_tests.rs` | KEEP | untouched |
| NEW `src/app/run/chat_failure_tests.rs` | NEW (2 tests) | preserves the retired rosters' pure coverage: (1) the CLOSED transient matrix pin — `Network`/`BackendUnavailable`/`MalformedResponse` transient AND `QuotaLike { "rateLimitExceeded" }` transient AND `QuotaLike { "quotaExceeded" }` NOT transient AND the two new classes NOT transient; (2) the three-way terminal mapping pin — `QuotaLike { "quotaExceeded" }` → `Quota`; `ChatPrecondition`/`InvalidChatRequest`/`InsufficientPermissions` → `ChatUnavailable`. Registered from `chat_failure.rs` via `#[cfg(test)] #[path = "chat_failure_tests.rs"] mod chat_failure_tests;` |

### C.8 Naming ruling (caller item g)

`GoogleLiveApiTransport` / `LiveChatTransport` / `google_live_chat_transport.rs` keep their names (no churn); the trait keeps exactly `lookup_broadcast_chat_id`. Stream-family names stay as landed (`GoogleLiveChatStreamTransport::new(AccessToken)`).

### C.9 Post-retirement dead-code verification (in-container greps; each expected ZERO)

```
git grep -n "list_messages" -- src
git grep -n "chat_messages_path" -- src
git grep -n "LiveChatMessageList" -- src
git grep -n "ChatListCommand" -- src
git grep -n "map_page" -- src
git grep -n "ChatPageOutcome" -- src
git grep -n "polling_interval_millis" -- src
git grep -n "DEFAULT_CHAT_POLL_INTERVAL" -- src
git grep -n "Staged for the Task-3" -- src
```

(Structure-map/dead-code-inventory DOC updates are TASK 4's.)

---

## §D — Regression locks (exact invariants; verified by the §F tests/greps or by the zero-edit ledger)

1. **`demo` byte-untouched**: zero edits in `src/app/demo/`, `src/streaming/*` (args/stream_args/ffmpeg and rosters). The demo argv and streaming-vector pins run green through the full gate.
2. **Gate spawn/join byte-compatible**: `gate.rs`, `streaming.rs`, `source.rs` (`ChatSource`), `pipeline/source_error.rs` — ZERO code edits. `GatePlan.chat_worker: Option<ChatWorkerPlan>` unchanged; `finish()` joins gate-thread-then-worker as landed; `StreamingExecution.source` ownership unchanged.
3. **Single-`error:`-line + exit tables + gate-Fatal-outranks**: `RunError` untouched; the worker never prints (only the terminal cell); `primary_shutdown_outcome` untouched.
4. **Typed-error funnel**: no new `LiveApiError`/`ChatRetrievalError` variant; the `chat_failure.rs` classifier stays CLOSED (only the NEW pure test roster attaches); Display lines and rosters unchanged.
5. **The ChatSource/sourcing pipeline unchanged except internals**: one-message-per-frame pacing, log-first `ChatStore` acceptance, and FIFO eviction all live in `pipeline/` — untouched; the chat store/source rosters green.
6. **Redaction/secrets**: no token/header/payload/RTMPS value in any log field the new code emits; the 3 info lines carry only the chat id / discarded count / fixed texts (the existing pinned constants).
7. **Wire contacts unchanged**: the O6 lookup path/headers byte-kept; the stream establishment wire form stays Task-2-pinned (the golden proto + `stream_client` rosters green).
8. **Zero new `#[allow(`**: the inventory strictly shrinks (§C.5); the `manual_clamp` line relocates with its function.

---

## §E — Test rosters for Task 3 (offline; zero sleeps except the ONE documented 250 ms stall-precedent pin; async pieces run through the worker's confined runtime in test (zero-sleep) mode or `stream_test_block_on`; all fixtures are `*-fixture` shaped; no test contacts Google)

Scripting rule for every roster: the scripted run ENDS deterministically — the LAST scripted session closes with a graceful state (an `offline`-ended batch: `yielded(&response_offline(vec![]))`) or a typed terminal delivery, so the worker returns; the run double panics on script exhaustion (the sister fixtures' `expect` precedent), never fabricating a state.

### E.1 `chat_worker_tests.rs` (rewrite; 6 tests)

1. `worker_offline_ended_batch_disconnects_without_terminal_error` — one open → a history-tokenless batch with `offline_ended` → the worker returns; terminal empty; the sender dropped → `try_recv` → `Disconnected`.
2. `worker_mid_session_precondition_is_graceful` — session #1 delivers a token-bearing batch (history cleared), then `Err(ChatPrecondition)` within the same session; assert no terminal; the operations ledger shows exactly one `open`.
3. `worker_establishment_precondition_writes_the_chat_unavailable_terminal` — open #1 → `Err(ChatPrecondition)` at zero delivered batches; terminal `ChatUnavailable` with the EXACT detail `live chat is unavailable: live chat precondition failed (reason 'LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED')`.
4. `worker_resolution_absence_after_budget_maps_chat_unavailable` — six `Ok(None)` lookups (the O6 budget, unchanged); terminal detail EXACTLY `YouTube live chat not found; deleted or never created`; no stream open recorded.
5. `worker_lookup_ending_failure_writes_the_chat_unavailable_terminal` — the O6 resolution aborts on `ChatDisabled` (unchanged); terminal `ChatUnavailable`.
6. `worker_event_constants_are_pinned_and_secret_free` — the three message constants verbatim + the rt mps/Bearer/token negative asserts (carried verbatim from the old roster).

### E.2 `chat_worker_flow_tests.rs` (rewrite; 6 tests)

1. `worker_delivers_live_batches_in_api_order_with_metadata` — session #1: a token-bearing history batch (`text_item("h-1")`) → whole discard; session #2: a batch `["m1","m2"]` (m2 published at the fixture RFC3339) → drained in order; author/text/id/timestamp asserts (the epoch via `parse_rfc3339_seconds` — the pinned `1_792_931_445` value for the same literal).
2. `worker_discards_the_entire_history_batch_before_the_first_token` — session #1 history (id `h-1`) with token → NOTHING queued or logged; session #2 delivers `m-1` → exactly one drained message.
3. `worker_dedup_drops_replayed_ids_across_sessions` — session #2 yields m1; session #3 yields m1 (replay) then m2 → drained exactly `["m1 first", "m2"]` — the window survived the reconnect.
4. `worker_token_progresses_verbatim_without_inference` — session #1 yields an EMPTY batch with token `RESUME_TOKEN_FIXTURE`; assert the ledger's `opened_requests()` == `[page_token None, page_token Some(fixture)]`; emptiness never ends the loop.
5. `worker_failed_connect_before_the_first_batch_reestablishes_tokenless_then_discards_history` — open #1 → `Err(Network)`; open #2 (token-less) delivers a history batch carrying its token → discarded; open #3 token-keyed → live; `opened_requests()` = `[None, None, Some]`; terminal empty; nothing queued (the §A.3 refinement pin).
6. `worker_malformed_batch_is_discarded_and_never_ordinary_chat` — session #2 yields `Err(MalformedResponse { field: "stream response" })` (transient) → re-establish; session #3 delivers `m-1` → drained exactly once; the ledger shows ONLY the scripted opens (no periodic calls).

### E.3 `chat_worker_budget_tests.rs` (rewrite; 7 tests)

1. `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal` — ten opens → `Network { reason: "network-fixture-N" }`; terminal `Transport` detail EXACTLY `unable to reach the YouTube API: network-fixture-10`; the ledger shows EXACTLY 10 opens (bounded; never periodic).
2. `worker_transient_failures_recover_within_the_budget` — two failing opens, then one live session ending gracefully → clean; nothing queued.
3. `worker_backoff_delay_doubles_and_caps_at_the_maximum` — the verbatim port plus the base-1 s family: `backoff_delay_ms(1, CHAT_SLEEP_SLICE_MS) == 1_000`, `(2, …) == 2_000`, and the existing cap cases.
4. `worker_stop_flag_ends_the_loop_before_any_transport_open` — a preset flag; 0 lookups; 0 opens.
5. `worker_stop_during_delivery_drops_pending_messages_without_blocking` — `flip_stop_on_open_from(2, …)` arms the flip inside the second open; the live batch then hits `send_stop_aware` after the flag set → the pending item dropped; terminal empty; the run returns promptly.
6. `worker_stalled_stream_cannot_prevent_a_bounded_join` — the ledger double's `next_batch` REAL-stalls (`std::thread::sleep(250 ms)` inside the fake — the documented stall precedent; the stalled session's script is EMPTY so the post-stall delivery is the clean `None`); the test thread flips the stop at `STALLED_MILLIS / 2` and joins the spawned worker with a ≤5 s wall bound. This is the ONLY real-sleep pin; named in the test docs.
7. `worker_never_reestablishes_after_a_graceful_end` — a graceful offline batch → the worker returns with the ledger open count EXACTLY 1.

### E.4 `gate_chat_tests.rs` (rewrite; 2 tests)

1. `confirmed_gate_spawns_and_joins_the_chat_worker` — an offline-ended batch script; `GateFinish::Confirmed`; the ledger operations `["open"]`.
2. `cancelled_gate_never_spawns_the_chat_worker` — an empty ledger; `GateFinish::Cancelled`; zero open calls.

### E.5 The latency property pins (TODO bullet 4 — offline; both live in `chat_worker_budget_tests.rs`)

- `worker_delivery_after_a_stalled_read_adds_no_poll_interval` — a live session's `next_batch` real-stalls 250 ms (the precedent), then yields one live message; assert the message IS in the receiver AND the worker's wall time `< 1 s` (with the `>= STALLED_MILLIS / 2` lower sanity) — proving NO server-interval-style wait was added (the retired polling path's first honor-sleep made this exceed 10 s). Same reserved real-stall allowance as E.3-6 (the roster documents it).
- The "next frame iteration" operator expectation is documented by TASK 4 (`production-run.md`) — the frame cadence belongs to the pipeline rosters, not this roster.

### E.6 Recorded-but-untouched pins (drift-checks only)

- The streaming-stage argv pin (`streaming_stage_tests`), the demo rosters, the RTMPS vector pins, redaction pins and exit-code pins run green unchanged through the full gate; Task 3 only RECORDS them at §L.

---

## §F — Commands (container-only; ordered)

1. Preconditions: `git status --porcelain` EMPTY; branch `feat/phase07-streamlist-retrieval`; `git log --oneline -3` ends at the Task-2 close. Any other state ⇒ STOP and ask the caller.
2. Commit 1 (plan tracking): `git add .kilo/plans/20261029-phase07-task3-integration-switch.md` → `git commit -m "chore: track the phase07 task3 integration switch plan"`.
3. Land §B/§C in this order (run `cargo fmt --check` after each batch until clean): (a) youtube retirement (C.1–C.3, C.6, the C.7 youtube rows, then the C.5 staging deletions); (b) the run-side worker rewrite (`mod.rs`/`seams.rs`, then the three worker files per §B); (c) the rosters (E.1–E.4, `chat_failure_tests`, `stream_run_fixtures`, `live_chat_rfc3339_tests`).
4. Targeted runs (fast feedback; all under the Compose prefix):
   `cargo test --locked chat_worker`, `cargo test --locked chat_source`, `cargo test --locked chat_failure`, `cargo test --locked runtime`, `cargo test --locked gate_chat`, `cargo test --locked live_chat`, `cargo test --locked stream`, `cargo test --locked grpc_status`, `cargo test --locked demo`, `cargo test --locked streaming_stage` — all green; counts observed.
5. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → 0.
6. FULL GATE: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, four PASS lines, `ALL CHECKS PASSED`; record the log path + the OBSERVED total. Arithmetic sanity (NOT doc-governing): baseline 507 − (6 models + 11 mapping + 3 O7 wire + 18 old worker rosters) + (6+6+7+2 rewritten worker rosters + 2 chat_failure + 1 rfc3339) ≈ 495; the OBSERVED gate count governs.
7. Windows-gnu check: `docker compose … run --rm rust cargo check --target x86_64-pc-windows-gnu` → 0. NO release builds (Task 4 owns them).
8. Hygiene/retirement greps: §C.9 (all zero); `git grep -n "allow(" src/youtube` = the §C.5 standing set; `git grep -rn "tonic::async_trait" src/app` = 0; `git grep -n "tokio::runtime" src` = exactly TWO sites (`chat_worker.rs` + `stream_test_fixtures.rs`); the secret sweep over the diff (`ya29|AIza|rtmps://`) = zero.
9. NO docs/structure-map/tech.md edits (Task 4). Commit 2: `git add` LIMITED to the task file set (§G) → `git commit -m "feat: switch the run chat worker to the streamList stream client and retire the polling surfaces"`.

---

## §G — Commit list + file-by-file touch table

Commits (4.2 order): (1) plan tracking; (2) the rewire + retirement code commit; (3) the §L addendum commit (written after the gate; a separate `docs:` commit is allowed per the Task-1 §13.5 precedent). NO push (workflow step 5).

| File | Action |
|---|---|
| `src/app/run/chat_worker.rs` | REWRITE |
| `src/app/run/chat_stream_worker.rs` | NEW |
| `src/app/run/chat_stream_delivery.rs` | NEW |
| `src/app/run/chat_worker_delivery.rs` | DELETE |
| `src/app/run/mod.rs` | EDIT |
| `src/app/run/seams.rs` | EDIT |
| `src/app/run/chat_failure_tests.rs` | NEW |
| `src/app/run/chat_worker_tests.rs` / `chat_worker_flow_tests.rs` / `chat_worker_budget_tests.rs` / `gate_chat_tests.rs` | REWRITE |
| `src/app/run/runtime_chat_fakes.rs` | REWRITE |
| `src/app/run/runtime_tests.rs` | EDIT (test 1) |
| `src/youtube/api_chat_transport.rs` | REWRITE |
| `src/youtube/google_live_chat_transport.rs` | REWRITE |
| `src/youtube/live_chat_models.rs` | REWRITE |
| `src/youtube/live_chat_models_tests.rs` | DELETE |
| `src/youtube/live_chat_mapping.rs` | REWRITE |
| `src/youtube/live_chat_mapping_tests.rs` | DELETE |
| `src/youtube/live_chat_rfc3339_tests.rs` | NEW |
| `src/youtube/live_chat_test_fixtures.rs` | REWRITE |
| `src/youtube/live_chat_wire_tests.rs` | KEEP |
| `src/youtube/live_chat_wire_checks_tests.rs` | REWRITE (6→3) |
| `src/youtube/mod.rs` | EDIT |
| `src/youtube/live_error/mod.rs` | EDIT |
| `src/youtube/stream_run_fixtures.rs` | NEW (fixture module) |
| untouched: `chat_source_tests.rs`, `gate.rs`, `streaming.rs`, `source.rs`, all of `pipeline/`, `demo/`, `streaming/`, the KEEP rows above, `Cargo.toml`, `Cargo.lock` | — |

Counts: 4 new source/fixture files + 1 new roster file; 3 deleted files; 15 rewritten/edited files.

---

## §H — Compliance checklist (every TODO Task-3 bullet → owning section)

- [ ] Swap the production retrieval per the Task-1/2 designs with byte-compatible spawn/channel/stop/store/pacing/shutdown | §0, §B, §D
- [ ] `demo` byte-untouched + production wire contacts unchanged | §D.1, §D.7
- [ ] Frame-loop continuity through establishment/reconnect/empty/slow startup; the bounded burst policy | §A.6, §B.7, §E.2/E.3
- [ ] Offline latency verification; the operator-observable expectation documented (Task 4) | §E.5, §I
- [ ] Typed-error wiring: no new variant; the closed classifier; gate-Fatal outranks; the single error line; no second broadcast | §A.5, §B.6, §D.3/D.4
- Code rules: files ≤200 (§B.1 budgets + §B.10 contingency), fns ≤50 body lines, ≤2 params (param objects: `ChatStageSources`, `ChatWorkerPlan`, `ChatFakes`), depth ≤2 (the let-else in §B.4 is the single-level exception, bounded), no commented-out code, self-documenting names, zero new `#[allow(` (§C.5), zero-sleep offline tests (the ONE named real-stall precedent in E.3-6/E.5), no test contacts Google, no secrets in any fixture.

---

## §I — Explicitly NOT done by this pass (owner named) + the 4.6 flip matrix

NOT done (owners):
- Docs (`production-run.md` latency/quota/reconnect text, the structure-map re-cut, `tech.md` confinement bullet, `context.md` addendum): **Task 4**.
- Release artifact rebuilds + `ls -l` cross-checks + the Phase-07 completion report + the operator-run manual E2E: **Task 4 / OPERATOR**.
- The real-Google TLS/handshake + long-lived-connection validation: **operator manual E2E** (the offline limitation recorded in Task-1 P7-12).
- Structure-map dead-code-inventory and roster-count lines: **Task 4** (recorded-at-gate values).

4.6 criteria-flip matrix (flip ONLY the PROVABLE at Task-3 end; the observed green gate is the evidence):

| AC | Candidate | Ruling |
|---|---|---|
| AC-1 — production `run` receives chat via streamList; latency no longer paced by `pollingIntervalMillis` | flip `[x]` | **PROVABLE** — the wiring is complete, the rosters pin the mechanism, the gate is green; the live behavior confirmation stays implicit in Task-4's report and the operator E2E. CALLER-VISIBLE DECISION (recorded here for 4.6): the flip is granted on the offline-wiring evidence, following the Phase-06 precedent ("implemented + offline-pinned; live confirmation awaits the operator manual E2E"); if the caller prefers strictness, leave unchecked with the annotation "wired; live confirmation awaited" |
| AC-2 — established once / token-keyed re-establishment only / never periodic; quota semantics recorded as stated | flip `[x]` | PROVABLE — the loop + the ledger pins; the quota semantics were recorded by the Task-1 record §11 |
| AC-4 — supported text messages still map correctly; non-text never ordinary chat | flip `[x]` | PROVABLE — the Task-2 mapper rosters + this task's integration + the unchanged conversion point |
| AC-5 — MessageSource→ChatStore→renderer flow; never blocks; bursts per policy; demo byte-stable | flip `[x]` | PROVABLE — unchanged source/pipeline pins + the delivery pins + the untouched demo |
| AC-6 — reconnects/backoff/buffers/cancel/shutdown bounded; no replay; dedup across reconnects; class verdicts | flip `[x]` | PROVABLE — §E.1–E.3 |
| AC-7 — the async runtime confined AND documented | NOT | the code confinement lands here; the `tech.md` documentation is Task 4 |
| AC-9 — failures typed/actionable/secret-safe reaching the run error policy | NOT | the funnel is offline-pinned, but the operator-facing "actionable" documentation is Task 4 |
| AC-10..AC-13 | NOT | the gate re-verdict, builds, docs, and report are Task-4 |

---

## §J — Notes for 4.5b (adherence hints, non-binding)

- The two adjudicated refinements (§A.7.1 stop-ticker shape, §A.7.2 single-thread confinement) and the §A.3 baseline refinement are the ONLY deviations from a strict P7-10/P7-8 transcription; everything else is verbatim transcription.
- The §F.6 arithmetic is a sanity estimate; observed values govern everywhere.

---

## §L — Addendum: OBSERVED records (written by the 4.2 implementer; structure now, content only after execution)

### L.1 Preconditions (§F.1) — branch/tree observations.
### L.2 Code-landing order + any fmt/clippy mechanical fix-ups (allowed class: import-path/rename fixes only; each recorded).
### L.3 Targeted roster results (§F.4 commands; observed pass/fail counts).
### L.4 FULL GATE (log path; the OBSERVED test total; any failure and its resolution).
### L.5 windows-gnu check verdict.
### L.6 Hygiene/retirement greps (each §C.9 pattern + §F.8 counts with observed values).
### L.7 File inventory (actual created/modified/deleted counts vs §G).
### L.8 Staging-retirement completion statement (the L.8 ledger closure: all 9 attributes observed gone; the brace used by the worker's imports).
### L.9 Deviations/STOP evidence (none expected; caller-facing options if any).

---

## Footer

Plan: `.kilo/plans/20261029-phase07-task3-integration-switch.md` (this file).
Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — Task 3. Design records: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (P7-5..P7-12, §12), `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` (§A/§C/§L + the L.8 staging ledger). Global plan: `.kilo/plans/20261029-phase07-streamlist-migration.md`.
Precedents followed: the `#[cfg(test)] #[path]` sibling wiring, `#[cfg(test)]` fixture modules registered at the crate root with test-only re-export braces, verbatim-moved machinery over forks, pinned constants instead of literals, and OBSERVED-at-gate test counts.
Ended: 2026-10-29 (produced by the Task-3 4.1b architector pass).
