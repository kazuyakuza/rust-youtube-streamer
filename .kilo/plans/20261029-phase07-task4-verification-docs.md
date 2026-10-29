# Phase 07 Task 4 — Offline Tests, Documentation, Builds, and Manual Verification (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — **Task 4 only** ("Offline Tests, Documentation, Builds, and Manual Verification", TODO lines 73–87).
> Binding records consumed: Task-1 research/design record `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (§11 quota, §12 retirement, P7-1..P7-12, §14 probe); Task-2 plan `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` (§E rosters, L.8 ledger); Task-3 plan `.kilo/plans/20261029-phase07-task3-integration-switch.md` (§E rosters, §C retirement, §L addendum); global plan `.kilo/plans/20261029-phase07-streamlist-migration.md` (User Rulings, per-task plans AUTO-APPROVED).
> Landed state at planning: HEAD `ca9c55f` on `feat/phase07-streamlist-retrieval` (Task 3 done + adherence tracked); v0.13.0; gate state at Task-3 close **493 passed / 0 failed** (`logs/checks/20261029T195041Z.log` governs; no release builds exist for the Phase-07 tree yet).
> Produced by the Task-4 4.1b architector pass (2026-10-29). The 4.2 implementer commits this plan (§F commit 1) and executes the ordered steps (§1).

---

## 0. TODO Task-4 bullet ↔ owning-section map

| TODO Task-4 bullet (line) | Owning section |
|---|---|
| Coverage-audit the existing offline rosters; add deterministic tests ONLY for genuine gaps — the 6 named buckets (75–81) | §A (audit + verdicts §A.1–§A.6; the ONE new test encoded §A.7) |
| Offline tooling only: fakes and loopback … no test contacts Google (82) | §A.8 (tooling constraints carried into the new test's doc header) |
| Run the standard gate (83) | §B.1 |
| Rebuild … `sh scripts/build-windows.sh`; then `sh scripts/build-linux.sh`; verify both artifacts non-empty; record exact sizes with `ls -l` cross-checks (84) | §B.2–§B.4 (TODO-faithful order + wording-anomaly note) |
| Update operator/user documentation: `production-run.md` (behavior/latency/reconnect/recovery/privacy/quota; supersede polling-burn), authentication/cloud docs only if scope changed, `configuration.md` only if a real new setting justified, README links; `project-structure.md`, `tech.md`, `context.md` (+ the operator 2026-10-29 polling-run confirmation note) (85) | §C.a–§C.g |
| Create a tracked Phase 07 completion report under `.agent/reports/` (86) | §C.i |
| Provide the safe operator-run manual E2E procedure (87) | §D (+ its `production-run.md` landing in §C.a step 15) |
| Acceptance criteria (89–103) to flip | the §H compliance checklist + the §I flip matrix |

---

## 1. Ordered execution steps (4.2 runbook)

1. Precondition: `git status --porcelain` EMPTY; branch `feat/phase07-streamlist-retrieval`; `git log --oneline -3` ends at the Task-3 close chain (`37ff256`/`ca9c55f`). Otherwise STOP and ask the caller.
2. Read this plan; confirm nothing staged.
3. Commit 1: `git add .kilo/plans/20261029-phase07-task4-verification-docs.md` → `git commit -m "chore: track the phase07 task4 verification docs plan"`.
4. §A.7: land the ONE new roster file `src/app/run/chat_worker_burst_tests.rs` + the 3-line registration block in `src/app/run/chat_worker.rs`; run `cargo fmt --check` until clean; targeted `cargo test --locked chat_worker_burst` green (1/1).
5. Commit 2: `git add src/app/run/chat_worker.rs src/app/run/chat_worker_burst_tests.rs` → `git commit -m "feat: pin the chat-worker burst retention at the full channel"`.
6. §B.1: FULL gate; record log path + OBSERVED count (expected shape 494; the fresh log governs; never hardcode).
7. §B.2–§B.4 builds in the TODO-faithful order (Windows first, then Linux); `ls -l` cross-checks; record sizes.
8. §C docs edits in the order a → i, exactly as encoded; `cargo fmt --check` NOT affected (docs only).
9. §E hygiene audit (§E commands) — record observed values into the report (§C.i) and the plan §L addendum.
10. Commit 3 (operator docs): `git add docs/production-run.md docs/google-cloud-prerequisites.md README.md` → `git commit -m "docs: document the stream-backed live chat retrieval for operators"`.
11. Commit 4 (metadata + reports): `git add .agent/project-structure.md .agent/project-info/tech.md .agent/project-info/context.md .agent/reports/20261026-phase-06-youtube-live-chat-integration.md .agent/reports/20261029-phase-07-streamlist-migration.md` → `git commit -m "docs: record phase 07 project metadata and completion report"`.
12. Update §L addendum (observed records); commit it folded into commit 4 if landed after the gate (§F).
13. NO push (workflow step 5). NO TODO file edits (4.6 owns them).

STOP conditions (surface to the caller, never improvise):
- any gate/build failure after the encoded fixes are exhausted;
- a §C anchor is missing or its surrounding context no longer matches (transcription drift);
- a drift-check reveals a doc claiming polling-is-selected beyond the two `google-cloud-prerequisites.md` lines encoded in §C.c — record + ask;
- any test-file line-count overflow forcing a structural choice beyond §A.7's encoding.

---

## §A — COVERAGE AUDIT (the first duty): the TODO's 6 test buckets vs the landed rosters

Method encoded for 4.2: verify each row below against the live tree (grep the roster files named; e.g. a plain `grep -n "fn worker_" src/app/run/*_tests.rs`), then record the verdict table into the completion report's "Test Coverage Audit" section (§C.i row list). Counts in this §A are source-read facts verified at planning time.

### A.1 Bucket 1 — stream establishment + initial-page/baseline semantics (no history replay; documented resume behavior)

**VERDICT: COVERED. No new test.** Existing coverage (file + test names):

| Test | File | Property pinned |
|---|---|---|
| `worker_delivers_live_batches_in_api_order_with_metadata` | `src/app/run/chat_worker_flow_tests.rs` | live batch → queued in API order with author/text/id/epoch metadata |
| `worker_discards_the_entire_history_batch_before_the_first_token` | same file | the Task-3-adjudicated baseline rule: history batch discarded WHOLE, token retained, flag cleared once |
| `worker_failed_connect_before_the_first_batch_reestablishes_tokenless_then_discards_history` | same file | open #1 fails → open #2 tokenless → history discarded; `opened_requests()` = `[None, None, Some]` |
| `worker_token_progresses_verbatim_without_inference` | same file | token carried verbatim; emptiness never ends the loop; ledger `[page_token None, Some(fixture)]` |
| `worker_transient_failures_recover_within_the_budget` | `src/app/run/chat_worker_budget_tests.rs` | recovery session established tokenless; "the recovery session is the baseline session: discarded by design" |
| `stream_list_request_with_resume_token_appends_field_99` / `stream_list_request_encodes_the_documented_wire_form` | `src/youtube/stream_proto_tests.rs` | the documented resume-token wire form (proto field 99) |
| `the_next_page_token_carries_forward_when_present_and_stays_none_when_absent` | `src/youtube/live_chat_stream_mapping_tests.rs` | token carry semantics at the mapping layer |
| `stream_request_carries_the_bearer_metadata_only` + the client roster (6) | `src/youtube/stream_client_tests.rs` | establishment request shape: Bearer only, no API key |
| `worker_event_constants_are_pinned_and_secret_free`, `worker_offline_ended_batch_disconnects_without_terminal_error` | `src/app/run/chat_worker_tests.rs` | baseline/ended constants; graceful `offline_at` end |

The establishment machinery's first-session baseline rule (record §5/P7-8) is the Task-3-adjudicated history-FIRST `process_batch` (chat_stream_delivery.rs:27–55), pinned by the flow roster rows 2–3 above.

### A.2 Bucket 2 — reconnect + resume + dedup + bounded backoff / terminal-class mapping

**VERDICT: COVERED. No new test.**

| Test | File | Property pinned |
|---|---|---|
| `worker_dedup_drops_replayed_ids_across_sessions` | `chat_worker_flow_tests.rs` | dedup window SURVIVES reconnect; replay never queues |
| `worker_malformed_batch_is_discarded_and_never_ordinary_chat` | same file | malformed → transient → re-establish; never ordinary chat |
| `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal` | `chat_worker_budget_tests.rs` | 10 opens exactly (bounded, never periodic) → `Transport` terminal with the exact detail |
| `worker_backoff_delay_doubles_and_caps_at_the_maximum` | same file | backoff family (1 s base × 2^(n−1), 60 s cap) |
| `worker_never_reestablishes_after_a_graceful_end` | same file | graceful end ⇒ ledger `["open"]` exactly |
| `worker_mid_session_precondition_is_graceful` / `worker_establishment_precondition_writes_the_chat_unavailable_terminal` | `chat_worker_tests.rs` | the `ChatPrecondition` dual branch (≥1 batch graceful; 0 batches fatal `ChatUnavailable`) |
| `closed_transient_matrix_is_pinned` / `terminal_mapping_is_three_way_with_the_stream_classes` | `src/app/run/chat_failure_tests.rs` | classifier CLOSED list; three-way terminal mapping incl. stream classes |
| `live_error_grpc_status_tests.rs` roster (12, incl. `permission_denied…`→terminal, `resource_exhausted…`→rate-limit-transient, `failed_precondition…`→the documented ambiguity, `internal_becomes_malformed_stream_response`) | `src/youtube/live_error_grpc_status_tests.rs` | every §7/P7-11 status row |

### A.3 Bucket 3 — push-event latency (arrival → channel delivery without any added interval)

**VERDICT: COVERED — the caller-prelisted E.5 pins verified present (fresh read of `chat_worker_budget_tests.rs`).**
- `worker_delivery_after_a_stalled_read_adds_no_poll_interval` (lines 164–191): 250 ms real stall → one live message drained → wall clock `>= STALLED_MILLIS/2` and `< 1 s`. NO poll interval added.
- `worker_stalled_stream_cannot_prevent_a_bounded_join` (lines 126–146): stop mid-read → joined ≤ 5 s wall bound.
No new test (a duplicate pin would add nothing the roster does not assert).

### A.4 Bucket 4 — runtime-bridge confinement (cancellation during connect/read, bounded shutdown, no cross-thread borrow)

**VERDICT: COVERED (by named pins + the compiler gate). No new test.** Adjudication of the caller's prelisted "connect-window cancellation may not exist" candidate:

- The stop-flip hook DOES exercise cancellation inside the connect window: `flip_stop_on_open_from` records the flip INSIDE `open()` (fresh read `src/youtube/stream_run_fixtures.rs:80–94`: `record_open` → call-index → `stop()` executes mid-open, then the open resolves). The pinned consumption is `worker_stop_during_delivery_drops_pending_messages_without_blocking` (`flip_stop_on_open_from(2, …)`, the re-establishment window) plus `worker_stop_flag_ends_the_loop_before_any_transport_open` (pre-open stop, 0 lookups 0 opens). Cancelling during READ is pinned by `worker_stalled_stream_cannot_prevent_a_bounded_join` (§A.3).
- A "prompt-abort mid-connect" test MUST NOT be added: the binding design (record §5/P7-10) explicitly documents the connect window as the ONE stop-unpollable bound — `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000` (`src/youtube/stream_client.rs:35`, pinned by `stream_endpoint_builds_offline_with_connect_timeout_and_tls_shape`). A connect-window stop resolves bounded (≤ 10 s), never hangs; asserting prompt abort would contradict the recorded design. A `from_call: 1` variant adds no semantic the from-2 pin and the pre-set pin do not already exercise (same `establish()` path, same flag check granularity).
- Bounded shutdown: `worker_stalled_stream_cannot_prevent_a_bounded_join` + `confirmed_gate_spawns_and_joins_the_chat_worker` / `cancelled_gate_never_spawns_the_chat_worker` (`gate_chat_tests.rs`).
- "No cross-thread borrow leaks into the seam": covered structurally — `stream_seam_objects_stay_send_bounded` (`src/youtube/stream_transport_tests.rs`) is the compile pin for the seam's `Send` bounds; any cross-thread borrow of a non-`Send` state fails compilation (`cargo check`/test/clippy = the structural assertion). The 4.2 evidence commands (§E) record the two grep pins: `tonic::async_trait` in `src/app` = 0; `tokio::runtime` in `src` = exactly 2 sites (`chat_worker.rs` + `stream_test_fixtures.rs`).

### A.5 Bucket 5 — burst/backpressure behavior at the bounded channel per the documented policy

**VERDICT: GENUINE GAP — ONE new test (encoded exactly in §A.7).**
Verified by fresh greps: the `send_stop_aware` `TrySendError::Full` retain-retry path exists in `src/app/run/chat_stream_delivery.rs` (lines 77–93: `Full(returned)` → retain → retry) but NO test in the whole tree exercises a full 64-entry channel (grep `Full` in test files = 0 hits; the retired Phase-06 rosters did not carry one either — fresh `git show dc3312a^:src/app/run/*` verified). The documented policy (record §6/P7-9): a burst into a full channel is RETAINED (never silently dropped mid-session) and stalls the consumer side while the stream's flow control stalls reads; deliver in order.

### A.6 Bucket 6 — preservation pins (`demo` schedule/vector, RTMPS argv, secret redaction, clean-stop exit codes)

**VERDICT: COVERED by Task-3's zero-diff ledger + the gate; drift-check/record only, no new test.**
- `demo` byte-stable: fresh zero-diff over the Task-3 chain for `src/app/demo/` + `src/streaming/` (Task-3 adherence §5) — re-verify at 4.2 with `git diff dc3312a^..HEAD -- src/app/demo src/streaming` (expect EMPTY) and the green demo pins (`schedule_tests`/`source_tests`/`mod_tests`, the streaming-stage argv pin `streaming_stage_tests`, `stream_args_tests`, `stream_args_health_tests`).
- Redaction/exit contracts: `error_tests.rs` (Display/exit/selector matrices), `source_error_tests.rs` 3, `run_error_display_lines_are_stable_and_secret_free`, `worker_event_constants_are_pinned_and_secret_free`, the stream-side `chat_precondition_and_invalid_request_display_lines_are_secret_free` + `mapped_stream_statuses_never_embed_the_server_message`.
- 4.2 records the targeted roster list + the full-gate green verdict.

### A.7 THE new test — encoded exactly

**File (NEW): `src/app/run/chat_worker_burst_tests.rs`** (target ≤ 120 fmt'd lines; hard cap 200).

```rust
//! Roster C — the burst/backpressure pin (TODO Task 4 coverage audit §A.5):
//! a live burst lands into a FULL 64-entry channel; the stop-aware
//! `try_send` retain-retry must retain every item and deliver them in
//! order — nothing silently dropped, no reorder. Offline only; zero
//! sleeps (the test-build `sleep_slice` is a no-op, so retries spin until
//! the draining receiver frees capacity — deterministic progress).

use crate::app::run::chat_worker::runtime_chat_fakes::{ChatFakes, worker_plan};
use crate::app::run::chat_worker::{CHAT_CHANNEL_CAPACITY, spawn_chat_worker};
use crate::chat::ChatMessage;
use crate::youtube::{response_offline, response_plain, text_item, yielded};

use super::chat_channel;
use super::chat_worker_tests::{drain, lookup_found, terminal_of};

#[test]
fn worker_full_channel_retains_the_burst_without_loss_or_reorder() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_plain(vec![
            text_item("burst-1"),
            text_item("burst-2"),
        ])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let channel = chat_channel();
    for index in 0..CHAT_CHANNEL_CAPACITY {
        let filler = ChatMessage::new(&format!("prefill-{index}"), "capacity");
        channel
            .sender
            .send(filler)
            .expect("the fresh channel holds the prefill");
    }
    let plan = worker_plan(&fakes, &channel);
    let handle = spawn_chat_worker(plan);

    let total = CHAT_CHANNEL_CAPACITY + 2;
    let mut drained: Vec<ChatMessage> = Vec::new();
    while drained.len() < total {
        drained.push(
            channel
                .receiver
                .recv()
                .expect("the pinned burst drains message by message"),
        );
    }
    handle.join().expect("the worker thread joins");

    for (index, message) in drained.iter().enumerate().take(CHAT_CHANNEL_CAPACITY) {
        assert_eq!(
            message.to_string(),
            format!("prefill-{index}: capacity"),
            "the prefill drains first, in order"
        );
    }
    let burst_one = format!("{:?}", drained[CHAT_CHANNEL_CAPACITY]);
    let burst_two = format!("{:?}", drained[CHAT_CHANNEL_CAPACITY + 1]);
    assert!(burst_one.contains("burst-1"), "the first burst item arrives: {burst_one}");
    assert!(burst_two.contains("burst-2"), "the second burst item arrives: {burst_two}");
    assert!(
        terminal_of(&channel).is_none(),
        "a retained burst is not an error"
    );
    assert!(
        drain(&channel.receiver).is_empty(),
        "nothing extra is queued beyond the pinned total"
    );
}
```

**Registration block — `src/app/run/chat_worker.rs`**: append directly after the `chat_worker_budget_tests` block (current file = 183 lines; the block adds 3 → 186 ≤ 200):

```rust
#[cfg(test)]
#[path = "chat_worker_burst_tests.rs"]
mod chat_worker_burst_tests;
```

Encoding notes the implementer must honor: (i) prefill via `send` on a FRESH channel (never blocks; the receiver stays undrained until the loop runs); (ii) the worker MUST run via `spawn_chat_worker` on its own thread — `run_chat_worker(plan)` inline would deadlock the test thread; (iii) the drain loop `recv()` on the blocking receiver IS the release mechanism that frees slots one by one (progress proof built into the loop); (iv) imports reuse the existing crate-root fixture re-exports (same names as `chat_worker_budget_tests.rs`); (v) ZERO sleeps — do NOT improvise a timed variant, do NOT add a second stall; (vi) the helper `drain` is used ONLY on the empty channel at the end (its `try_recv` shape returns empty); (vii) test name frozen (the `worker_` prefix keeps it inside the `cargo test --locked chat_worker` filter family).

Expected gate total: **493 + 1 = 494** — RECORDED from the fresh §B.1 log; any unexplained deviation from 493+1 or any doc-hardcoded prediction is forbidden (the task's observed-only rule; reconcile the count against the roster arithmetic before recording).

### A.8 Offline tooling constraints carried to the new test (TODO bullet, line 82)

- The new roster uses ONLY in-process scripted fakes (`RunChatStreamLedger` + `ChatFakes`) — no network, no `127.0.0.1` server, no Google contact, no credentials. No new loopback gRPC server is needed (the Task-2/3 double already covers the seam; a loopback gRPC test-runtime server remains PERMITTED-but-not-required and must NOT be added here).
- No real account/broadcast/credentials; fixture strings `-fixture` shaped; no secret-shaped material.

---

## §B — THE GATE + BUILDS (container-only; every command via the Alpine VM MCP; `vm_status` checked first)

### B.1 FULL gate

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

- Expected: exit 0, four PASS lines (`fmt-check`, `check`, `test`, `clippy`), summary `ALL CHECKS PASSED`.
- Record: the UTC log path printed by the script (`logs/checks/<UTC>.log`) + the OBSERVED `test result: ok. N passed; 0 failed` count. **The fresh log governs** — no doc may hardcode a predicted total (README's count line uses the OBSERVED integer; the report quotes the log path).
- If any check fails: fix per plan (fmt → re-run `cargo fmt --check`; test → re-read the roster; clippy → the existing rule-following re-shape precedents; NO new `allow(`), then re-run; if unfixable within the encoded scope → STOP + ask.

### B.2 Build order — TODO-faithful (wording-anomaly note encoded)

TODO line 84 reads "Rebuild Linux: `… sh scripts/build-windows.sh`; then `sh scripts/build-linux.sh`" — the heading says Linux but the FIRST command is the Windows script. **Honor the TODO's exact wording: run BOTH, Windows first, then Linux; record both runs in the TODO-faithful order and note the anomaly in the report.** (The release-content consequence: the FINAL Windows artifact recorded by the gate order is built BEFORE the Linux artifact; both are built on the SAME tree, so the order does not change either binary's content validity.)

Windows (first, TODO-faithful):
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
```

Linux (second, TODO-faithful):
```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
```

### B.3 `ls -l` cross-checks (both artifacts)

```
ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows
```

- Assert BOTH artifacts exist and are non-empty: `dist/rust-youtube-streamer-service` and `dist/windows/rust-youtube-streamer-service.exe`.
- Record the observed byte sizes from `ls -l` (the sizes are recorded from observations — NEVER invented or carried from a prior phase; the Phase-06 values 5331832/8764036 are stale historical facts, not this phase's).
- Cross-check: each recorded `ls -l` size must match the build script's printed size (e.g. `Build succeeded: … (N bytes)`). A mismatch = a build-hygiene STOP.

### B.4 No-build claim rules

- Builds are container-only facts; cross-compilation is NOT native Windows runtime validation; building validates no runtime chat behavior (the streaming build's chat live-confirmation remains the operator E2E).
- NOTHING under `dist/` is ever staged/committed (§E audit re-proves).

---

## §C — DOC EDITS (anchor + full replacement text; junior-exact; apply top-down)

Rules: (i) every edit below is an EXACT replace of the quoted anchor snippet (unique in the file; if an anchor does not resolve → STOP + ask); (ii) nothing outside the quoted anchors changes — every unrelated section stays byte-stable (notably the Ingest Probe Procedure and the Known Stream-Health Limitation sections, and the never-logged list in Privacy); (iii) real newline characters, no literal `\n`; (iv) plural/spacing exactly as written.

### C.a `docs/production-run.md`

**C.a.1 — Overview paragraph (anchor, line 29):**
from
`The frame loop is continuous and chat-backed: the \`run\` command polls the created broadcast's Live Chat and renders newly received ordinary messages (author and text) onto the frames;`
to
`The frame loop is continuous and chat-backed: the \`run\` command reads the created broadcast's Live Chat over a server-streamed connection and renders newly received ordinary messages (author and text) onto the frames;`
(the rest of the paragraph onward is byte-stable).

**C.a.2 — Log-table row 7 Notes column (anchor, line 69):**
from
`info; fires once at the worker's first successful poll; carries \`discarded_messages\``
to
`info; fires once when the worker's first baseline (history-bearing) stream batch is discarded; carries \`discarded_messages\``

**C.a.3 — Log-table ordering paragraph (anchor inside line 76):**
from
`(row 7 at the worker's first successful poll;`
to
`(row 7 at the worker's first baseline batch;`

**C.a.4 — Startup item 10, stage-7.5 clause (anchor, line 53):**
from
`first resolves the broadcast's live chat id with a bounded 30-second budget, then polls on the server-instructed interval; on Ctrl+C the worker stops FIRST, before FFmpeg's stdin is closed, and the join is bounded — ≤ ~1 s wake, ≤ ~30 s during the one-shot chat-id resolution window).`
to
`first resolves the broadcast's live chat id with a bounded 30-second budget, then establishes the server-streamed Live Chat connection (pushed message batches are mapped into the bounded screen channel; the link is re-established only after a drop, carrying the accumulated \`nextPageToken\` — it is NEVER re-called on a fixed schedule); on Ctrl+C the worker stops FIRST, before FFmpeg's stdin is closed, and the join is bounded — ≤ ~1 s wake, ≤ ~10 s inside a stream-connect window, ≤ ~30 s during the one-shot chat-id resolution window).`

**C.a.5 — Live Chat → Prerequisites scope bullet (anchor, line 141):** replace the whole bullet (`- Scopes: … keep working.`) with:

`- Scopes: the already-granted \`https://www.googleapis.com/auth/youtube.force-ssl\` is documented-sufficient for BOTH chat calls the worker makes — \`liveChatMessages.streamList\` (the server-streamed retrieval) and the \`liveBroadcasts.list\` chat-id lookup (official scope bindings re-verified 2026-10-29; the Phase 07 record §1.5, \`.kilo/plans/20261029-phase07-task1-streamlist-research-design.md\`). Re-running \`auth\` is NOT required for chat: there has been no scope change since Phase 04, so the existing credentials keep working.`

**C.a.6 — Live Chat → Prerequisites no-config bullet (anchor, line 144):** replace the whole bullet with:

`- There is NO new configuration field for chat: message pacing is SERVER-PUSHED (messages arrive on the established stream; no poll interval exists at all) — nothing to configure, and no operator override exists. The re-establishment bounds (connect timeout, backoff cap, consecutive-failure budget) are internal constants, not settings.`

**C.a.7 — Live Chat → Expected Behavior retrieval bullet (anchor, line 148):** replace the whole bullet with:

`- Retrieval method: the worker uses the official \`liveChatMessages.streamList\` server-streaming method — a gRPC over HTTP/2+TLS client built on the published proto, authenticated Bearer-only (never an API key). It establishes ONE connection for the session and receives message batches as they are pushed. If the connection drops, the worker re-establishes with the last \`nextPageToken\` and the server resumes from where the client left off; the retrieval is NEVER re-called on a fixed/periodic schedule. The Phase 06 \`liveChatMessages.list\` polling surface was RETIRED in the Phase 07 migration (user ruling, 2026-10-29; the Phase 07 record §12, \`.kilo/plans/20261029-phase07-task1-streamlist-research-design.md\`).`

**C.a.8 — Live Chat → Expected Behavior worker-timeline bullet (anchor, line 149):** replace the whole bullet with:

`- Worker timeline (operator terms): the worker thread starts ONLY after the confirmation gate observes \`streamStatus == active\`. Its FIRST act resolves the broadcast's \`snippet.liveChatId\` via \`liveBroadcasts.list(part=snippet)\` with a bounded budget (6 attempts × 5 s = 30 s); absence after the budget is a fatal typed error. The worker then establishes the streamed connection. The FIRST-established session is the BASELINE session: ALL items it delivers are discarded (they are history the connect pushes — never queued, never logged), so messages sent before this moment do not appear, and — because the API documents no boundary between pushed history and pushed live traffic — messages posted DURING the first session are lost too (documented limitation; the window's length is the session's own lifetime). The session's final token is retained. Fresh sessions after a reconnect (they always carry the token) deliver live: ordinary text messages (author + text) are accepted at most one per frame, appended to the chat log, and rendered. Replayed message IDs are dropped (bounded dedup window that survives reconnects). While chat is idle the connection stays silent but stays open (no reconnect churn); the frames continue (black when no visible text).`

**C.a.9 — NEW latency bullet directly after the C.a.8 bullet (inside Expected Behavior):**

`- Latency: a pushed message is mapped and queued the moment its batch arrives and appears on the watch-page video at the NEXT frame iteration (at the configured 30 FPS: sub-second) AFTER YouTube-side delivery latency, which the application does not control and which is the largest contributor when chat is quiet. The official documentation characterizes the method as low-latency push ("as soon as they are available"); no operator tuning exists and none is needed. The offline pin for the no-added-interval property is \`worker_delivery_after_a_stalled_read_adds_no_poll_interval\` (arrival → channel within a sub-second wall bound).`

**C.a.10 — Live Chat → Diagnostics: appended paragraph after the existing `The detail is the redacted Display …never a raw response body.` paragraph (line 160):**

`On a connection drop the operator CAN observe: nothing while reconnect churn stays inside the budget — a silent re-establishment logs no event and prints no line; the only externally visible sign is a pause in new messages while idle traffic resumes. The operator CANNOT observe: whether the link is currently established, individual reconnect attempts, or the handshake itself. When the bounded persistence budget exhausts (10 consecutive transient failures, backoff capped at 60 s), the typed \`live chat transport failed after repeated attempts:\` error line surfaces and the run exits 1. Honest limitation: the real TLS/gRPC handshake against Google is validated ONLY by the operator E2E below — the offline tests pin everything around it (proto wire bytes, Bearer metadata, status classification, the resumable loop) with scripted offline doubles.`

**C.a.11 — Live Chat → Failure Recovery: replace the table + the "Honest boundedness facts" paragraph (anchors = the current table rows at lines 164–173).** Table:

| Situation | Documented symptom | Outcome | Recovery |
| --- | --- | --- | --- |
| Broadcast ended / chat closed | `chatEndedEvent`, `offlineAt`, or the `FAILED_PRECONDITION` chat-reason AFTER live deliveries (`liveChatEnded`) | GRACEFUL: one info line, the worker stops, video keeps flowing until Ctrl+C | none needed |
| Chat disabled at establishment | the `FAILED_PRECONDITION` chat-reason (`LIVE_CHAT_DISABLED`/`LIVE_CHAT_ENDED` — ambiguous by gRPC code per the official footnote) AT ZERO delivered batches | fatal (exit 1) | confirm chat is enabled on the broadcast, then re-run |
| Wrong/expired chat id | `NOT_FOUND` / the `liveChatNotFound` class | fatal (exit 1) | re-run `run` (each run resolves its own broadcast's chat id) |
| Permissions revoked mid-run | `PERMISSION_DENIED` / 401-403 class | fatal (exit 1) | re-run `auth` ([`docs/authentication.md`](authentication.md) recovery table) |
| Quota exhausted | the typed Quota terminal (the official gRPC-path exhaustion surface is NOT documented — recorded unknown; see Quota below) | fatal (exit 1) | wait for the midnight PT reset, or apply for Google's quota-extension form |
| Dropped/stalled stream link | silent push pause; bounded re-establishment with the retained token (backoff 1 s × 2^(attempt−1) capped 60 s; ≤ 10 consecutive transient failures) | recoverable — invisible while inside the budget | resolve connectivity; if the budget exhausts, the Transport error ends the run — re-run |
| Malformed/partial batch | a batch that fails typed decoding | transient — the whole batch is discarded, never ordinary chat | none (auto re-establishment with the retained token) |
| Chat-id resolution budget | no `liveChatId` after 6 × 5 s | fatal (exit 1) | re-run after the broadcast is confirmed live |

Then the boundedness paragraph (replace the existing one at line 173) with:

`Honest boundedness facts: a Ctrl+C stops the chat worker FIRST (before FFmpeg's stdin EOF); the join is bounded (≤ ~1 s wake; ≤ ~10 s worst case inside a connect window; ≤ ~30 s worst case during the one-shot chat-id resolution window, where the worker sleeps in raw 5 s slices); every re-establishment wait is stop-pollable within one 1 s slice except the one-shot ≤ 10 s connect attempt; and up to 64 queued-but-undelivered messages can be lost at shutdown — they are never logged, so the chat log and the video never diverge.`

**C.a.12 — Live Chat → Privacy: BYTE-STABLE (no edit).** The never-logged list and the chat-log claims remain exactly as landed.

**C.a.13 — Live Chat → Quota: replace the H3 heading + body (anchor = lines 181–192).** New heading + body:

`### Quota (What Chat Streaming Costs)

Chat streaming costs **5 units per connection call**: the initial establishment and every re-establishment after a drop are new calls. The one-time chat-id resolution lookup costs 1 unit. The default project quota is 10,000 units/day, reset at midnight Pacific Time.

The accountable worst case under the worker's bounds: a sustained outage re-establishes at most 10 times (consecutive-failure budget) plus the initial attempt — ≤ 11 connection calls ≤ 55 units — before the typed transport error ends the run.

Officially NOT documented (recorded as unknown — never invented and never promised): whether pushes on an ESTABLISHED connection bill per item/per batch; whether connection lifetime itself bills continuously; any server keepalive/idle-drop timing; the appearance of `quotaExceeded`/`dailyLimitExceeded` on the gRPC path. The worker therefore classifies mid-stream quota-shaped failures into its existing bounded rate-limit/transient arm and documents the unknown honestly.

Operator guidance: the Phase 06 per-interval poll-burn profile is GONE (no periodic calls exist — the polling-burn table of the previous build is superseded by this subsection); a session's accountable documented costs are the establishment calls (5 units each) and the 1-unit resolution. The docs make NO quota-neutral promise beyond these documented call costs; a quota extension (free, manually reviewed) remains the only path above the default ceiling.`

(The new H3 slug changes: TOC lists H2s only — NO production-run TOC change. The `google-cloud-prerequisites.md` pointer that names the old H3 is updated in §C.c.)

**C.a.14 — Manual Verification Procedure step 6 (anchor, line 330):** replace the WHOLE step with the §D text (landed here; §D is the authoritative enumerated step list).

**C.a.15 — Manual Verification Procedure: NEW optional step after the updated step 6 (before step 7):** insert the §D "6b" block (landed here; §D is authoritative).

Byte-stable zones (DO NOT TOUCH): TOC; Prerequisites items 1–6 (except nothing); "What the Logs Look Like" rows 1–6, 8–10 and the standing-secret paragraph; Shutdown Behavior; Broadcast/Stream Fate; the common-ingestion-failures list; Ingest Probe Procedure (entire H2); Known Stream-Health Limitation (entire H2); steps 1–5 and 7–10 of the Manual Verification Procedure; Out of Scope; See Also.

### C.b `docs/configuration.md` — DRIFT-CHECK ONLY (expect NO edit)

Drift-check procedure (record the verdict, edit nothing unless it fails):
1. `grep -n "chat\|Chat\|poll" docs/configuration.md` — expected hits: the `### chat` section's `chat.log_file` field bullet, the validation-rule line, the `run` command bullet's "(Phase 06 chat worker wired into `run`; details in production-run.md)" pointer, and the `demo` bullet. NONE of these claims polling or a chat retrieval method.
2. Verify NO new config field is justified (the TODO's own condition): the re-establishment bounds are internal constants (`CHAT_STREAM_CONNECT_TIMEOUT_MS`, `CHAT_BACKOFF_MAX_MS`, budget) — NOT settings; `chat` section = `log_file` ONLY.
3. Expected verdict: **NO EDIT** — the pointer sentence's substance (details live in production-run.md) stays truthful with §C.a's edits. If the check FAILS (any polling/method claim found), STOP + ask the caller (never improvise wording).

### C.c `docs/authentication.md` + `docs/google-cloud-prerequisites.md`

- `docs/authentication.md` — DRIFT-CHECK ONLY, expect **NO EDIT**: the scope-sufficiency sentence ("the minimal YouTube API permission covering the needs of the subsequent broadcast/chat phases") remains true per the Phase-07 §1.5 re-verification (the stream method binds the same scope trio); no method name is asserted in this file.
- `docs/google-cloud-prerequisites.md` — DRIFT-CHECK with TWO SANCTIONED TRUTH-RESTORATIONS. Basis recorded explicitly: the researched scope/permission picture did NOT change (so no scope-driven edit), but lines 44 and 140 assert the now-false "polling is the selected method" fact against the landed tree; accuracy mandates (TODO line 33 + AC-12) and the Phase-06 Task-4 truth-restoration precedent govern. Two exact edits:
  - **C.c.1 — line 44 row:** replace
    `| \`liveChatMessages\` (\`liveChatMessages.list\` polling is the selected method) | the production \`run\` command's live-chat worker (Phase 06: implemented and documented; see [\`docs/production-run.md\`](production-run.md)) | Phase 06 |`
    with
    `| \`liveChatMessages\` (\`liveChatMessages.streamList\` server-streaming is the selected method; the \`list\` polling surface was retired in Phase 07) | the production \`run\` command's live-chat worker (Phases 06–07: implemented and documented; see [\`docs/production-run.md\`](production-run.md)) | Phases 06–07 |`
  - **C.c.2 — line 140 bullet:** replace the whole bullet with:
    `- **Live-chat reading costs per connection, not per poll.** The production worker establishes the official \`liveChatMessages.streamList\` server-streaming connection (5 units per establishment call); it is re-established ONLY after a drop (≤ 11 connection calls ≈ 55 units inside one sustained outage window under the worker's bounded budget) and makes NO per-interval polling calls. The one-time chat-id resolution lookup costs 1 unit. Push-billing on an established connection is officially undocumented — never invented here. The detailed quota accounting for a real broadcast lives in the "Quota (What Chat Streaming Costs)" subsection of [\`docs/production-run.md\`](production-run.md) and in the Phase 07 completion report. A quota extension (free, manually reviewed) remains the only path above the default ceiling.`
  - Line 130's table row (`liveChatMessages.list` / `streamList` | 5) is FACT and stays byte-stable.
  - Record both edits + their drift-check basis in §L.

### C.d `README.md`

1. **Current Status paragraph (line 44):** in-place, replace the sentence beginning `Phase 06 (Live Chat integration) is landed: the real chat worker is wired into `run` behind offline tests; the operator-facing chat documentation and the Phase 06 completion report are landed; the real-broadcast manual verification remains OPERATOR-RUN and was not executed by the agent.` with:
   `Phase 06 (Live Chat integration) landed, and Phase 07 (streamList migration, v0.13.0) swapped its retrieval to the official server-streaming `liveChatMessages.streamList` gRPC client: the connection is established once per session and re-established only after drops carrying the accumulated `nextPageToken` (never re-called on a schedule), pushed messages render on the next frame iteration after arrival, and the polling surfaces were retired. The operator-facing chat documentation and the Phase 06/07 completion reports are landed; the real-broadcast manual verification (now including the streaming build's push-latency check) remains OPERATOR-RUN and was not executed by the agent.`
2. **Test-count claim (line 46):** replace
   `The containerized test suite is green (`ALL CHECKS PASSED`, container-only; 458 tests, re-recorded by the Phase 06 verification pass).`
   with
   `The containerized test suite is green (`ALL CHECKS PASSED`, container-only; ⟨OBSERVED_GATE_TOTAL⟩ tests, re-recorded by the Phase 07 verification pass).`
   where `⟨OBSERVED_GATE_TOTAL⟩` is the integer READ from the §B.1 fresh log (`test result: ok. N passed`). Record the log path once in the report; the README carries only the integer.
3. **Key Technologies bullet (line 35):** replace the whole `- **Tokio** — NOT in the dependency tree: …` bullet with:
   `- **Tokio (`tokio` 1.53.1 — locked)** — the streaming chat client's runtime, CONFINED to the single dedicated chat worker thread as a current-thread runtime; the frame pipeline and FFmpeg supervision remain synchronous (`std::process` + std threads, with the `ctrlc` crate for Ctrl+C). The pin rationale (1.54.x needs Rust ≥ 1.85 > the pinned 1.82; crates.io registry metadata, accessed 2026-10-29) is recorded in the Phase 07 research record §4.`
4. **New dependency bullet directly after the Tokio bullet:**
   `- **gRPC + Protocol Buffers (`tonic` 0.13.1 + `prost` 0.13.5 — locked)** — the official `liveChatMessages.streamList` server-streaming client (rustls `ring` provider + bundled `webpki-roots`, matching the existing ureq TLS family; Bearer-only OAuth; NO API key). Hand-authored prost message types transcribed from Google's officially published proto (no codegen step, no `protoc`); `tonic::async_trait` re-exports the seam traits. Registry access dates 2026-10-29; versions from the tracked lockfile pins.`
5. **Production Run section (line 120):** the sentence "A bounded live-chat worker polls the created broadcast's Live Chat" carries the now-false polling fact — replace the sentence
   `A bounded live-chat worker polls the created broadcast's Live Chat and feeds incoming ordinary messages into the frame pipeline: they render as author/text when present, and the frames stay black while chat is idle or slow.` — with
   `A bounded live-chat worker consumes the created broadcast's Live Chat over the server-streamed connection and feeds incoming ordinary messages into the frame pipeline: they render as author/text when present, and the frames stay black while chat is idle.` — keep the operator-docs pointer sentence as-is; the Commands bullet (line 61) stays as-is (it makes no method claim).
6. TOC: NO change (no README heading added or renamed).

### C.e `.agent/project-structure.md` — full re-cut for the landed tree

Commands first (record OBSERVED, never invent):
- `git ls-files src/app/run` → count files (expected 30: 13 sources + 4 fixtures + 13 rosters — verify, don't trust).
- `git ls-files src/youtube` → count (expected 82 entries) and recount sources vs rosters.
- `grep -c "#\[test\]" per roster` → per-roster totals; `cargo test` totals are OBSERVED at gate.
- `grep -rn "#\[allow(" src/` → the dead-code re-cut (expected: 8 `#[allow(dead_code)]` attribute lines + the `#[allow(clippy::manual_clamp)]` at `chat_stream_delivery.rs` (moved with its function) + 1 comment mention in `src/youtube/mod.rs`).

Edits (each an in-place row replacement; anchor-preserving):
1. Root files Cargo.toml line: `v0.12.0` → `v0.13.0` + dependency list sentence gains ", and the Phase-07 stream-family (tokio 1.53.1 confined to the chat worker thread; tonic 0.13.1 + prost 0.13.5 for the official gRPC stream client; rustls-ring/webpki-roots TLS)".
2. The `app/` map intro: mention "the production `run` mode … the Phase-07 stream-backed chat worker".
3. The `run/` line: "28 files: 12 sources + 4 fixtures + 12 test rosters" → OBSERVED re-cut (say "30 files: 13 sources + 4 fixtures + 13 test rosters" only if the count matches).
4. `chat_worker.rs` row: REPLACE its description with the stream-worker splitting: "the bounded Phase-07 chat worker (spawn-at-confirmation; O6 resolution budget; `chat_runtime()` ONE current-thread Tokio runtime confined to THIS thread; `block_on(run_stream_loop(...))`; 64-entry channel + constants family + the three pinned info constants)".
5. `chat_worker_delivery.rs` row: DELETE (file is gone).
6. ADD two rows after `chat_worker.rs`:
   - `chat_stream_worker.rs` - the confinement-respecting resumable stream loop (`StreamWorker::new`/`run_stream_loop`; establish → select-biased stop tick `CHAT_STOP_TICK_SLICE_MS = 1_000` → consume; per-connection state inside the async task)
   - `chat_stream_delivery.rs` - the delivery/failure machinery (`process_batch` history-FIRST baseline rule; `deliver`/`send_stop_aware` try_send-only; `handle_failure`/`handle_clean_close` → bounded re-establishment with the token; `record_terminal`; `backoff_delay_ms` (the moved `manual_clamp` body))
7. Roster rows: the `*_tests.rs` counts line re-cut (drop the Phase-06 numbers; ADD chat_worker_burst_tests 1; chat_worker_flow 6; chat_worker_budget 8; chat_worker 6; gate_chat 2; chat_failure 2 … record.
8. `youtube/` intro line: `80 files: 41 sources + 39` → OBSERVED; plus "the Phase-07 stream family (proto/client/concrete/transport/mapping/grpc_status + fixtures)".
9. `api_chat_transport.rs` / `google_live_chat_transport.rs` / `live_chat_models.rs` / `live_chat_mapping.rs` rows: REWRITE to the surviving surfaces (trait = lookup-only O6 seam; concrete = the O6 GET wire form with the Bearer-only doc; models = the O6 lookup survivor; mapping = the `ChatTextMessage` trio + the RFC3339 parser re-export). DELETE the `live_chat_models_tests` / `live_chat_mapping_tests` rows; ADD the new sentinel rows: `stream_proto.rs` (prost structs from the published proto + the request builder; golden pins), `stream_transport.rs` (the mockable async seam traits), `stream_client.rs` (Bearer metadata builder + endpoint + session adapter), `stream_concrete.rs` (concrete transport), `live_chat_stream_mapping.rs` (map proto batch → carriers), `live_error/grpc_status.rs` (`classify_stream_status` + the 2 new variants), `stream_test_fixtures.rs` + `stream_run_fixtures.rs` (the offline doubles with the stop/stall hooks), and the sibling rosters with their pin counts.
10. The "Dead-code staging inventory" paragraph: re-cut from the fresh greps (recorded values, never invented; state "as of 2026-10-29, the Phase-07 tree: 8 dead-code attribute lines + the clippy `manual_clamp` line relocated with its function to `chat_stream_delivery.rs` + 1 comment mention").
11. `docs/` line (line 134): add "the surviving production-run.md Quota subsection renamed to 'What Chat Streaming Costs' and the polling-era content superseded by the stream-backed section (Phase 07)".
12. `live_error/mod.rs` row count: `17 variants` → `19 variants` (the 2 new stream classes).

### C.f `.agent/project-info/tech.md`

1. **Stack bullet (line 8):** replace
   `* The frame pipeline and FFmpeg supervision are synchronous (std::process + std threads); the Phase 06 live-chat retrieval also landed WITHOUT Tokio — a bounded std thread over a fixed-capacity \`std::sync::mpsc::sync_channel(64)\` — so Tokio never entered the dependency tree (task-2 decision S2; Phase-06 D-1 verdict).`
   with
   `* The frame pipeline and FFmpeg supervision stay synchronous (std::process + std threads). The Phase 07 live-chat retrieval runs the official server-streaming `streamList` gRPC client, which REQUIRES an async stack: ONE `tokio` current-thread runtime built and `block_on`-driven INLINE on the ONE dedicated chat worker thread (the sync-preamble label: the pipeline stays sync-std; the runtime is confined to the worker — it never touches the frame loop, `auth`, or `demo`). Worker→frame-loop delivery remains the phase 06 boundary: the bounded `std::sync::mpsc::sync_channel(64)` + `try_recv` + terminal cell are byte-unchanged.`
2. **Dependencies bullet (line 29):** append after the existing Phase-06 bullet a NEW bullet:
   `* Decided during Phase 07 (streamList migration): `tonic 0.13.1` + `prost 0.13.5` + `tokio 1.53.1` (crate family rationale: the officially documented client form is gRPC server-streaming with tools' generated stubs avoided — `tonic`+`prost` implement the protocol; `tonic` features client-only `channel/prost/codegen/tls-ring/tls-webpki-roots`; `tokio` features trimmed to `rt+time+macros` because ONLY the current-thread scheduler, the stop-tick timer and the `select!` macro are used). MSRV pins: tonic 0.13.1 (rust_version 1.75 — 0.14.x needs 1.88), prost 0.13.5 (1.71.1), tokio 1.53.1 (1.71 — 1.54.x needs 1.85 > the pinned image), plus the probe-mandated transitive `--precise` pins (`idna_adapter` 1.2.0, `indexmap` 2.12.1, `hyper-util` 0.1.17, `zeroize` 1.8.1, `tokio-util` 0.7.19); rustls `ring` + bundled `webpki-roots` matches the existing ureq TLS family on the Windows-GNU cross-build (registry probes recorded in the Phase 07 Task-1 record §14, access dates 2026-10-29).`
3. **Dependencies bullet line 29 — REPLACE** the Phase-06 sentence about retrieval method POLLING being the live-chat mechanism:
   `Retrieval method (Task-1 record D-1): … each poll costs 5 units …`
   with
   `Retrieval method (the Phase 07 record §12 supersedes Phase 06's D-1 by USER RULING 2026-10-29): the production worker streams via `streamList`; the `list` polling surfaces are retired; the chat-id resolution is one-time `liveBroadcasts.list(part=snippet)` (1 unit) and streamList costs 5 units per connection call (est/re-establishment).`
4. **NEW confinement bullet in Lifecycle (after the Lifecycle opening paragraph, line 33):**
   `* Streaming-bridge confinement (Phase 07): ONE runtime instance per run (built ONCE on the worker thread, `current_thread` + `enable_all`); ONE worker thread; every async wait is stop-pollable ≤ 1 s via the select-biased stop tick (`CHAT_STOP_TICK_SLICE_MS = 1_000`) EXCEPT the one-shot connect attempt (`CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000`); the bounded-join argument: gate joins worker → worker joins its runtime thread; cross-thread outputs = `try_send` into the bounded channel + the terminal cell + the shared StopFlag read. Documented limit: the official keepalive/idle-drop behavior is UNDOCUMENTED — re-establishment is event-driven only (a sparse-but-alive stream is never redirected); per the official gRPC footnote, `LIVE_CHAT_DISABLED` vs `LIVE_CHAT_ENDED` is not distinguishable by code (the establishment-fatal / mid-session-graceful split is the deliberate typed decision).`

### C.g `.agent/project-info/context.md`

1. **NEW dated block** inserted directly after the existing `## Phase 07 Task 1 — streamList client-path research record landed (2026-10-29)` section, before `## Recent Changes (2026-10-08)`:

```
## Phase 07 Tasks 2–4 + closure — streamList migration landed and verified (2026-10-29)

* Task 2 (client + bridge, commit `4229e36` close): 12 new `src/youtube/` stream files (prost models transcribed from the official proto + golden pins, mockable `ChatStreamTransport` seam, concrete tonic transport with the 10 s connect bound, proto→carrier mapping reusing the Phase-06 mapping layer, `classify_stream_status` with the 2 new `LiveApiError` variants, offline doubles); 49 new offline zero-sleep tests; the Task-3 staging ledger (9 `allow(` attrs) closed. Gate 507/0.
* Task 3 (integration switch, `37ff256` close): the production worker swapped to the resumable stream loop (`chat_stream_worker.rs` + `chat_stream_delivery.rs`; the old poll worker delivery file DELETED); the `LiveChatTransport` trait shrank to the O6 lookup; the REST list models/mappers/rosters retired; the Task-2 staging ledger closed; 29 → OBSERVED run-roster recut. Gate 493/0 (log `logs/checks/20261029T195041Z.log` governs). Recommended flips at Task-3 4.6: AC-1 (ratified, with the annotation), AC-2, AC-4, AC-5, AC-6.
* Task 4 (verification/docs, THIS pass commit range): coverage audit → ONE genuine new test (`chat_worker_burst_tests.rs`: the full-channel burst retain-retry pin); FULL gate OBSERVED ⟨GATE_TOTAL⟩ / 0 (`⟨GATE_LOG_PATH⟩`); both release artifacts rebuilt (Windows first per the TODO's wording; Linux OBSERVED ⟨LINUX_SIZE⟩ B, Windows ⟨WIN_SIZE⟩ B, `ls -l`-cross-checked); operator docs re-cut to the stream-backed behavior (latency, re-establishment diagnostics, recovery table, quota subsection renamed to "What Chat Streaming Costs" with the documented 5-unit-per-connection facts + the unknowns; the polling-burn table superseded); structure map/tech/context re-cut (v0.13.0). Report: `.agent/reports/20261029-phase-07-streamlist-migration.md`.
* OPERATOR CONFIRMATION (2026-10-29, user-supplied, never agent-observed): the operator's real unlisted broadcast on the POLLING build v0.12.0 (`logs/run/20261029-153416.log` + watch-page screenshot) confirmed both live-render criteria of Phase 06 (messages render from two accounts): recorded in context + the Phase 06 report addendum; the criteria stay unchecked in the Phase 06 TODO. This confirms the baseline rendering path the Phase 07 swap preserves; the STREAMING build's live confirmation remains the operator E2E.
* NOT done by the workflow yet: 4.3 review/simplification, 4.4 docs pass, 4.5b adherence, 4.6 marks, workflow step 5 (rename/merge/push `origin` ONLY). The streaming handshake and push latency stay operator-verified-only.
```
(Replace the ⟨…⟩ placeholders with the OBSERVED gate/build values from §B; every recorded value from an observed command.)

2. **Immediate Next Steps: prepend ONE new item 1** (renumber the existing list down by one — the file's duplicate `2.` precedent shows renumbering is not fixed; encode: insert the new item BEFORE the current list item 1 text without touching the other items' `N.` markers — NOTE the file already carries a duplicate-2 numbering defect; the insertion rule is: insert ABOVE the first item, keep all other lines byte-stable):
   `1. Phase 07 is CLOSED on the feature branch (pre-merge): TODO Task 4 [DONE]-adjacent records — the streamList migration is fully landed and offline-verified (gate OBSERVED ⟨GATE_TOTAL⟩/0), operator docs live in `docs/production-run.md` (stream-backed semantics + quota), completion report `.agent/reports/20261029-phase-07-streamlist-migration.md`. OPERATOR NEXT STEPS: rebuild/re-deploy the streaming build from the freshly recorded `dist/` artifacts and run the updated manual E2E (`docs/production-run.md` §Manual Verification): unlisted broadcast → confirm video before messages → separate-account text → confirm author/text appears PROMPTLY (push latency check) → idle periods → OPTIONAL network-interruption re-establishment check → Ctrl+C (exit 0) → broadcast auto-completes. The streaming build's live latency/quota facts (the officially undocumented per-push billing) stay as recorded; real-Google statements are operator observations only.`

3. Nothing else in `context.md` changes (the historical blocks, Known Deviations, Scope Decisions, Notes stay byte-stable).

### C.h `.agent/reports/20261026-phase-06-youtube-live-chat-integration.md` — DOCUMENTING-ONLY addition

Naming note for the caller: this report has NO section literally titled "acceptance criteria" — the criteria-bearing section is `## Manual Verification Procedure (operator-run)`, which is where the two unchecked live-render criteria are described. **Decision: insert the dated block directly AFTER that section (before `## Checks Actually Performed vs NOT Performed`), nothing else restructured.**

Insert:

```
## Addendum (2026-10-29): Operator confirmation of the two live-render criteria ON THE POLLING BUILD

- Evidence (operator-supplied fact, never agent-observed): the operator's real unlisted-broadcast v0.12.0 run (`logs/run/20261029-153416.log`) plus the operator's watch-page screenshot (author/text lines visible from two accounts).
- The two acceptance criteria left unchecked as "awaits operator manual E2E" are thereby OPERATOR-CONFIRMED on the POLLING build. The TODO-file checkboxes stay as-is per the record's rule — this note documents the confirmation without rewriting the TODO's criteria state.
- Scope of the confirmation: the Phase 06 POLLING build's end-to-end chat rendering (it is the rendering path whose seam the Phase 07 streamList swap preserves byte-compatibly). The STREAMING build's own live confirmation (push latency, re-establishment) remains the operator E2E of the Phase 07 procedure in `docs/production-run.md`.
```

### C.i NEW `.agent/reports/20261029-phase-07-streamlist-migration.md` — the completion report (structure)

Write the full report with these sections in this order (format = the Phase-06 report's section skeleton; every value from §A–§E OBSERVED evidence; no invented numbers):

1. `## Overview` — branch/version v0.13.0; phase summary (the streamList mechanism, the confined runtime, what got retired); the commit chain per task (the observed hashes: Task-1 `f8628e2/ed583a7/aea846c/0024642`, Task-2 `dccacbc/03e696f/229e9cb/3f6d1b0/75788ae/35adefa/4229e36`, Task-3 `0266eff/dc3312a/45fe2b9/0647860/ff05536/37ff256/ca9c55f`, Task-4 commits from §F (the report's own final hash stays unknowable from inside the record and is captured by the later review/adherence passes).
2. `## Official Sources and Access Dates` — the Task-1 §1 table (S1–S12, access dates 2026-10-29).
3. `## Third-Party Directive Adjudications` — the C1–C5 table summary (per DIRECTIVE: verdict + one-line basis), cited to the record §3.
4. `## Selected Path and Toolchain Pins` — Path A; tonic 0.13.1/prost 0.13.5/tokio 1.53.1 + the 5 transitive pins + the §14 probe verdicts (both probes exit 0).
5. `## Quota Semantics and Documented Unknowns` — the §11 content verbatim-shaped (5 units/call; ≤ 11 calls ≤ 55 units per outage window; the 4 unknowns; no invented math).
6. `## Buffering, Backpressure, Reconnect, and Shutdown Decisions` — P7-9/P7-10 + the §5/§6 summaries (bounded sync_channel(64) untouched; retain-retry; select-biased stop tick; the ≤10 s connect exception).
7. `## Files Changed (per task)` — the per-task commit files tables (Task-2 chain 16 files; Task-3 chain 29 files + 3 deletions; Task-4: the burst roster + the docs/metadata set).
8. `## Test Coverage Audit` — the §A verdict rows (6 buckets; 5 covered-by-roster citations + the ONE gap and its new test) + the OBSERVED gate total + log path.
9. `## Exact Commands and Exit Statuses` — the §B.1/§B.2/§B.3 table (gate + windows build + linux build + ls -l; TODO-faithful order note) + the §E audit command list with observed values.
10. `## Build Results` — both artifact sizes (observed `ls -l`) + the cross-compilation caveat lines.
11. `## Manual Verification Procedure (operator-run)` — the §D step list summary + "the agent did not run, never claims" language + the 2026-10-29 POLLING-build operator confirmation pointer.
12. `## Checks Actually Performed vs NOT Performed` — the honest split (offline gate/audit/edits done; NO live Google/stream Google/handshake validation; latency documented-not-measured on the streaming build).
13. `## Known Limitations` — the offline-untestable TLS/gRPC handshake; the baseline-session loss window (documented API absence — length unbounded); the officially undocumented quota/keepalive semantics; the operator-confirmed latency is on the POLLING build (pending streaming-build confirmation); shutdown-loss window ≤ 64 carried over.
14. `## AC Closing Matrix` — the 13 criteria with per-criterion verdict + evidence pointer + the AC-1 rationale: mechanism-level criterion satisfied offline (retired surfaces + never-periodic + no-added-interval pin) with the live-confirmation annotation "(wired into the production path; live confirmation awaits the operator manual E2E)" — per the Task-3 adherence recommendation PENDING the caller's ratification; the Task-4 flips (AC-7/9/10/11/12/13) per §I.
15. `## Evidence Index` — plan/adherence/TODO/report paths + gate log + repo greps.

---

## §D — THE MANUAL E2E §-update (operator-run; landed via §C.a.14/§C.a.15)

Replace step 6 (exact new text; no keys/URLs — placeholders only):

`6. Actual video arrival and live chat: open YouTube Studio → the broadcast enters live; verify real frames appear (continuous rendered video). Note: re-running \`auth\` is NOT required for chat (no scope change; \`youtube.force-ssl\` already covers the chat calls — run \`auth\` only per the existing recovery paths). Confirm video starts BEFORE any messages arrive (the stream opens black — pre-connection chat is history-discarded by design). Then, from a SEPARATE account, send ordinary text messages AFTER the stream is confirmed active; expect them to render as author/text on the watch page PROMPTLY: pushed messages are mapped the moment their batch arrives and appear on the NEXT frame render (sub-second at 30 FPS) after YouTube-side delivery — record the observed delay honestly; it includes YouTube-side latency the application does not control. Confirm the same messages appear in the chat log (\`logs/chat.log\`, or the configured \`chat.log_file\`). Exercise idle periods: a few minutes with no chat — frames continue, nothing duplicates, no history replays, and no reconnect churn occurs while the connection stays alive. Messages sent before the stream went live are discarded by design (the worker's FIRST baseline session is discarded whole — messages posted during that window are lost, the documented API-absence limitation). Confirm at least ~1–2 minutes of continuous video. Verify the broadcast is on the EXPECTED channel: one OAuth grant binds to one channel identity chosen at consent, and the application neither selects, surfaces, nor verifies which channel ([\`docs/authentication.md\`](authentication.md)) — if the broadcast appeared on a different channel, Ctrl+C-stop, re-run \`auth\` to select the intended channel, and repeat from step 4.`

Insert the NEW optional step 6b directly after step 6:

`6b. OPTIONAL re-establishment check (interpreter-relevant drop discrimination): while the stream is live, briefly interrupt the network path (e.g. disconnect Wi-Fi or block outbound connectivity for roughly 10–15 seconds), then restore it. Expected: the video never stalls and the run keeps running; after restoration, NEW ordinary messages sent from the separate account appear on the watch page and in the chat log WITHOUT duplication (messages sent during the outage may be missing — the link was down; not everything pushed during a drop is recoverable). A prolonged outage can end the run with the single \`live chat transport failed after repeated attempts:\` error line and exit 1 after the bounded budget (10 consecutive transient failures, backoff capped at 60 s) — record whichever outcome occurs. A live re-establishment confirmation is the operator's evidence of the reconnect path the offline tests pin only structurally.`

Steps 1–5, 7–10 stay byte-stable (the Ctrl+C expectations, the auto-complete check, and the failure/evidence steps are already accurate).

Recording rule: the results are recorded ONLY by the operator (its own tracking artifact/issue), never as build evidence; live claims stay operator-run; no secrets pasted anywhere.

---

## §E — BUILD verification + hygiene audit (the exact §F-style command list; all read-only or container-only)

Run AFTER the §C edits (the final docs tree) and RECORD every observed value into the report (§C.i section 9) and the plan §L. Host-side git checks; secret greps with their EXPECTED shapes:

1. `git status --porcelain` → ONLY the task's edited paths (the burst roster + the §C doc/metadata set). Nothing else.
2. `git ls-files dist/ videos/` → EMPTY (both).
3. `git check-ignore -v dist/rust-youtube-streamer-service` → `.gitignore:34:dist/`; `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` → `.gitignore:34:dist/`; `git check-ignore -v logs/chat.log` → `.gitignore:18:logs/*`; `git check-ignore -v videos/` → `.gitignore:55:videos/`.
4. Secret greps (record OBSERVED; expected shapes):
   - `git grep -nE "ya29|AIza" -- src docs README.md .agent` → ZERO hits.
   - `git grep -n "key=" -- src docs README.md` → the pre-existing doc/test-context lines + EXACTLY ONE test negative-assertion line in `stream_client_tests.rs` (the Phase-07 shape; count and list them — no new).
   - `git grep -n "rtmps://" -- src docs` → ONLY the pre-existing `.example.test` fixture shapes + the historical probe-doc mentions (production-run.md Ingest Probe commands); the untouched Ingest Probe section keeps its pre-existing mentions; zero new.
   - `git grep -n "allow(" src/` → 8 `#[allow(dead_code)]` attribute lines + 1 `#[allow(clippy::manual_clamp)]` (at `chat_stream_delivery.rs`) + 1 comment mention (`src/youtube/mod.rs`) — CONSISTENT with the Task-3 observed inventory (the L.8 ledger closed; zero new from Task 4).
5. `git grep -rn "tonic::async_trait" src/app` → ZERO; `git grep -n "tokio::runtime" src` → exactly TWO sites (`chat_worker.rs`, `stream_test_fixtures.rs`).
6. Retired-name greps (Task-3 §C.9 re-run on the final tree) → all zero: `list_messages`, `chat_messages_path`, `ChatListCommand`, `map_page`, `ChatPageOutcome`, `polling_interval_millis`, `DEFAULT_CHAT_POLL_INTERVAL`, `Staged for the Task-3`; `LiveChatMessageList` → ONLY the Task-2-KEEP proto-model substrings (27 hits at Task-3 close — re-record the observed number).
7. `cargo fmt --check` (container) → 0 (docs-only tree must not drift formatting; no Rust change beyond §A.7 which was fmt-run already).

---

## §F — COMMIT LIST for 4.2 + the 4.3/4.4/4.5b/4.6 expectations

4.2 commits (in order; NO push):
1. `chore: track the phase07 task4 verification docs plan` — the plan file only.
2. `feat: pin the chat-worker burst retention at the full channel` — `src/app/run/chat_worker.rs` + NEW `src/app/run/chat_worker_burst_tests.rs` ONLY.
3. `docs: document the stream-backed live chat retrieval for operators` — `docs/production-run.md`, `docs/google-cloud-prerequisites.md`, `README.md` ONLY.
4. `docs: record phase 07 project metadata and completion report` — `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`, the NEW `.agent/reports/20261029-phase-07-streamlist-migration.md`, the amended `.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`, AND this plan's §L addendum (if written post-gate; else patch into commit 4 before 4.3).

- 4.3: code-reviewer reviews the burst roster + doc-set fidelity vs this plan (names, anchors, byte-stable zones); code-simplifier only if genuine duplication is removable (expect a NO-op); fix plans tracked under `.kilo/plans/`.
- 4.4: docs-specialist verifies TOC/anchor consistency in the touched docs (`#manual-verification-procedure-operator-run`, `#live-chat-behavior-diagnostics-recovery-privacy-quota` unchanged; the renamed quota H3 has no inbound anchor inside the touched files except the §C.c pointer, which uses the NEW slug), README links resolve, no stray polling-era claim survives (`grep -n "polls\|polling" docs README.md` re-run at 4.4 — expect only the historical/probe contexts + the Google-cloud-prerequisites §C.c.2 wording that names the retired surface honestly).
- 4.5b: architector adherence vs THIS plan (roster fidelity; doc edit fidelity; §E record completeness).
- 4.6: TODO Task-4 `[DONE]` + the §I flip matrix; nothing else.

---

## §H — Compliance checklist (every TODO Task-4 bullet → the executing step)

- [ ] Coverage-audit the rosters vs the 6 buckets; tests only for genuine gaps — §A (verdicts §A.1–§A.6; ONE new test §A.7; tooling constraints §A.8).
- [ ] Offline tooling only — §A.8 + §E (no Google contact; fakes only).
- [ ] Run the standard gate — §B.1 (observed count recorded; no hardcoding).
- [ ] Rebuild artifacts (Windows first TODO-faithful + Linux), non-empty + `ls -l` cross-checks + recorded sizes — §B.2–§B.4.
- [ ] Doc updates: `production-run.md` stream-backed re-cut incl. quota supersession — §C.a; `configuration.md` drift-check — §C.b; auth/cloud docs scope-conditional check — §C.c; README updates — §C.d; structure map — §C.e; tech.md — §C.f; context.md (+ the operator-confirmation note) — §C.g.
- [ ] Tracked Phase 07 completion report — §C.i.
- [ ] Safe operator-run manual E2E with push-latency + idle + optional drop check + Ctrl+C — §D (+ landed via §C.a.14/15).

---

## §I — NOT-done ledger + the 4.6 flip matrix for the REMAINING criteria

NOT done by Task 4 (owners): the live E2E and ALL live-Google claims = OPERATOR (never agent evidence); the TODO-file `[DONE]`/checkbox flips = workflow 4.6; branch merge/rename/push = workflow step 5 (`origin` ONLY); no release artifact is ever committed; `4.3/4.4/4.5b` = their own workflow steps.

4.6 flip matrix after Task 4's evidence lands (AC numbering = the TODO lines 91–103):
- **AC-7 (runtime confined AND documented) → FLIP `[x]`**: the tech.md confinement bullet (§C.f) + the L.8/greps close the "documented" conjunct (code side was Task-3-provable).
- **AC-9 (typed/actionable/secret-safe + operator-facing actionable docs) → FLIP `[x]`**: the delivery path was Task-3-pinned; the operator-facing failure/recovery documentation lands §C.a (Diagnostics + the Failure Recovery table rows).
- **AC-10 (offline tests pass without real Google access; loopback/fake tooling test-only) → FLIP `[x]` AFTER the fresh gate**: gate exit 0 + the §A audit verdict + the §E greps (no test constructs the concrete transport; no test dials out).
- **AC-11 (artifacts rebuilt + recorded sizes) → FLIP `[x]` AFTER §B.2–§B.3**: both builds exit 0 + `ls -l`-cross-checked observed sizes.
- **AC-12 (docs/structure/context/report accurate incl. adjudications + the operator note) → FLIP `[x]` AFTER §C lands**: includes the C1–C5 adjudications + the 2026-10-29 polling-confirmation note (context.md §C.g + the Phase-06 report addendum §C.h).
- **AC-13 (no secrets/ignored files committed) → FLIP `[x]` AFTER §E**: the audit checklist with the observed zero-hit secret counts and the empty `dist/ videos/` ls-files.
- **AC-1**: ALREADY handled at Task-3's 4.6 per the caller's ratification decision (the annotation "(wired into the production path; live confirmation awaits the operator manual E2E)" is recorded in the report's AC matrix) — if NOT yet flipped, flip now on the same terms; Task 4 adds no new evidence beyond the report's verbatim annotation.
- **AC-2–AC-6, AC-8**: already `[x]` (Tasks 1–3).

---

## §L — Addendum: OBSERVED records (written by the 4.2 implementer)

### L.1 Plan/commit records (observed)

- Commit 1 `f12027a` `chore: track the phase07 task4 verification docs plan` (this plan only).
- Commit 2 `7ab6eee` `feat: pin the chat-worker burst retention at the full channel` (2 files, 80 insertions: `src/app/run/chat_worker.rs` +4 (the registration block), NEW `src/app/run/chat_worker_burst_tests.rs` +76).
- Commit 3 `1e16c3f` `docs: document the stream-backed live chat retrieval for operators` (3 files, +35/−31).
- Commit 4 `1c72eed` `docs: record phase 07 project metadata and completion report` (6 files, +315/−24: the metadata set + the Phase-06 addendum + the NEW completion report + this plan's §L).
- A minimal follow-up corrective docs commit (this one) fixes the commit-2 insertion arithmetic in L.1 (recorded here for traceability; no other content changed).
- Precondition verified: branch `feat/phase07-streamlist-retrieval`; HEAD at entry `ca9c55f`; the only pre-existing dirty path was this plan file (committed as commit 1). No push performed.

### L.2 §A.7 landing (observed)

- `cargo fmt --check` (container): exit 0 BEFORE and AFTER the gate.
- Targeted `cargo test --locked chat_worker_burst`: exit 0, `running 1 test … ok. 1 passed; 0 failed; 0 ignored; 493 filtered out`.
- `src/app/run/chat_worker.rs` observed **187 lines** after the registration block (the plan predicted 186; the inserted block is 4 lines including the separating blank). Hard cap 200 holds. Recorded as a count observation, not a structural deviation.
- The test file landed byte-for-byte as encoded (rustfmt-clean on first write; no mechanical re-shaping needed).

### L.3 §B.1 gate (observed)

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → **exit 0**.
- `test result: ok. 494 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` → **494** (493 baseline + 1 new), matching the 493+1 roster arithmetic.
- Summary `ALL CHECKS PASSED`; log path **`logs/checks/20261029T203729Z.log`** (the fresh log governs; the README count line carries only the observed integer 494).
- No fix cycles were needed (first gate run green).

### L.4 §B.2–B.4 builds (observed, TODO-faithful order)

- `sh scripts/build-windows.sh` (FIRST) → **exit 0**, script-printed size **12,336,636 bytes**.
- `sh scripts/build-linux.sh` (SECOND) → **exit 0**, script-printed size **7,665,712 bytes**.
- `ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` → `rust-youtube-streamer-service` **7,665,712** bytes; `rust-youtube-streamer-service.exe` **12,336,636** bytes. Both match the printed sizes exactly (no build-hygiene stop).
- TODO wording anomaly recorded: TODO line 84's heading says "Rebuild Linux" while the first command is the Windows script; both runs executed in the TODO's exact command order (Windows, then Linux) as §B.2 encodes.

### L.5 §C doc-edit matrix (observed verdicts)

- `docs/production-run.md`: all 15 encoded edits (C.a.1–C.a.15) applied; byte-stable zones untouched; TOC unchanged (H2 set unchanged; the renamed quota H3 has no in-file anchor). **ONE minimal anchor repair**: C.a.3's encoded anchor `(row 7 at the worker's first successful poll;` does not resolve because the actual parenthetical reads `(row 6 after row 5; row 7 at the worker's first successful poll; …`; applied the minimal anchor `row 7 at the worker's first successful poll;` with the encoded replacement text (otherwise byte-identical).
- `docs/configuration.md`: **NO EDIT** — drift-check passed. Hits are exactly the `### chat` section + `log_file` bullet, the validation-rule line, the `run` bullet's "(Phase 06 chat worker wired into `run`; details in production-run.md)" pointer, and the `demo` bullet; none asserts polling or a retrieval method; the re-establishment bounds remain internal constants (no new setting justified).
- `docs/authentication.md`: **NO EDIT** — drift-check passed; the scope-sufficiency sentence ("the minimal YouTube API permission covering the needs of the subsequent broadcast/chat phases") remains true per the §1.5 re-verification; the file asserts no method name.
- `docs/google-cloud-prerequisites.md`: **TWO sanctioned truth-restorations applied** (C.c.1 line 44 row; C.c.2 line 140 bullet) on the recorded basis that the scope/permission picture did NOT change but the "polling is the selected method" statements were false against the landed tree. The line-130 `liveChatMessages.list / streamList | 5` fact row stayed byte-stable. No real drift beyond the two encoded lines was found.
- `README.md`: the 4 encoded edits applied (Current Status sentence, observed test-count integer 494, Tokio bullet, new gRPC/prost bullet) + the Production Run polling sentence; TOC unchanged; no other line touched.
- `.agent/project-structure.md`: full re-cut applied — Cargo.toml v0.13.0 + stream-family deps; `app/` intro; `run/` **31 files: 13 sources + 4 fixtures + 14 test rosters** (OBSERVED by `git ls-files src/app/run` = 31; the plan's 30/13 prediction was stale); the `chat_worker.rs` row rewritten + the two new `chat_stream_worker.rs`/`chat_stream_delivery.rs` rows added; the deleted `chat_worker_delivery.rs` row removed; the roster line re-cut to observed counts (`chat_worker_burst_tests 1`, `chat_worker_flow_tests 6`, `chat_worker_budget_tests 8`, `chat_worker_tests 6`, `gate_chat_tests 2`, `chat_failure_tests 2`, `chat_source_tests 5`; run rosters total **58**); `youtube/` **92 files: 47 sources + 4 fixtures + 41 test rosters** (OBSERVED by `git ls-files src/youtube` = 92; the plan's 82 prediction was stale) + the stream-family sentence; the four Phase-06 chat rows rewritten to the surviving surfaces; the retired `live_chat_models_tests`/`live_chat_mapping_tests` references removed from the roster summary; new sentinel rows added for `stream_proto.rs`, `stream_transport.rs`, `stream_client.rs`, `stream_concrete.rs`, `live_chat_stream_mapping.rs`, `stream_test_fixtures.rs`, `stream_run_fixtures.rs`; the youtube roster summary re-cut to **231** observed tests (per-roster counts from `git grep -c "#\[test\]"`, summing to 231); `live_error` row **17 → 19 variants** + the `grpc_status.rs` description; the `docs/` line gained the renamed-quota-subsection note; the dead-code inventory re-cut: **8 `#[allow(dead_code)]` attribute lines + the `manual_clamp` line at `chat_stream_delivery.rs`** (NO comment-only mention observed — the plan's "1 comment mention" is stale against the tree), zero new `allow(`.
- `.agent/project-info/tech.md`: the 4 encoded edits (Stack bullet, new Phase-07 dependency bullet, retrieval-method replacement, Lifecycle confinement bullet) applied. **ONE minimal accuracy repair beyond the list**: the Lifecycle opening paragraph's "then poll on the server-instructed interval" was corrected to "then establish the server-streamed Live Chat connection", which the plan's item 4 would otherwise contradict.
- `.agent/project-info/context.md`: the encoded Phase-07 dated block inserted after the Task-1 section (placeholders filled with the observed gate/build values); Immediate Next Steps gained the encoded item ABOVE the first item (a duplicate `1.` numbering results by design — the file already carried a duplicate `2.`). Additionally the Current Work Focus header gained the caller-directed Phase-07 state paragraph (kept above the historical Phase-06 paragraph; all dated blocks byte-stable).
- `.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`: the dated operator-confirmation addendum inserted directly after `## Manual Verification Procedure (operator-run)` and before `## Checks Actually Performed vs NOT Performed`; nothing else restructured.
- `.agent/reports/20261029-phase-07-streamlist-migration.md`: NEW completion report written to the §C.i 15-section skeleton with the observed values above.

### L.6 §E hygiene/secret audit (observed)

- `git status --porcelain` (pre-final-commit) → only the task paths (docs/metadata set; no `dist/`, no logs, no credentials).
- `git ls-files dist videos` → EMPTY (both).
- `git check-ignore -v dist/rust-youtube-streamer-service` → `.gitignore:34:dist/`; `… .exe` → `.gitignore:34:dist/`; `logs/chat.log` → `.gitignore:18:logs/*`; `videos/` → **`.gitignore:56:videos/`** (the plan's line-55 note is off by one; the rule resolves at 56).
- `git grep -nE "ya29|AIza" -- src docs README.md .agent` → **15 pre-existing lines, all historical audit-prose in `.agent`** (`context.md` 7, the Phase-04 report 2, the Phase-05 report 5, the Phase-06 report 1) — identical to HEAD; ZERO in `src`/`docs`/`README.md`; ZERO new. The plan's "ZERO hits" expectation is stale for the historical `.agent` prose; the security property (no token literal) holds.
- `git grep -n "key=" -- src docs README.md` → **5 pre-existing lines**: 3 doc-comment "never an API key/no `key=`" mentions (`google_live_api_transport.rs:3`, `live_chat_models.rs:4`, `live_chat_wire_checks_tests.rs:3`) + 2 negative assertions (`live_chat_wire_checks_tests.rs:30`, `stream_client_tests.rs:31`); zero new.
- `git grep -n "rtmps://" -- src docs` → only `.example.test` fixture shapes, the `pipe_drain.rs` redaction comment, the `production-run.md` Ingest-Probe placeholder mention, and secret-absence assertions; zero new.
- `git grep -n "allow(" -- src` → 8 `#[allow(dead_code)]` + 1 `#[allow(clippy::manual_clamp)]` (`chat_stream_delivery.rs:173`); zero new.
- `git grep -n "tonic::async_trait" -- src/app` → ZERO. `git grep -n "tokio::runtime" -- src` → 2 files (`chat_worker.rs`, `stream_test_fixtures.rs`; 3 lines).
- Retired-name greps (`list_messages`, `chat_messages_path`, `ChatListCommand`, `map_page`, `ChatPageOutcome`, `polling_interval_millis`, `DEFAULT_CHAT_POLL_INTERVAL`, `Staged for the Task-3`) → ALL ZERO. `LiveChatMessageList` → **27** hits (the Task-2-KEEP proto-model substrings).
- `git diff --stat dc3312a^..HEAD -- src/app/demo src/streaming` → EMPTY (the preservation ledger's zero-diff re-verified).
- `cargo fmt --check` (container, final docs tree) → exit 0.

### L.7 Deviations and disposition

1. `chat_worker.rs` 187 lines vs the 186 prediction — ACCEPTED (cap holds; recorded).
2. C.a.3 anchor repair — MINIMAL, applied and recorded (L.5).
3. Structure-map observed counts differ from the plan's stale predictions (run 31 vs 30; youtube 92 vs 82; roster totals 58 / 231; no comment-only `allow` mention) — recorded from observation per the plan's own "record OBSERVED, never invent" rule.
4. Structure-map minimal accuracy repairs beyond the enumerated rows (the fixture row, the youtube roster summary's retired references/counts, the `youtube/mod.rs` re-export row) — MINIMAL, observed-driven, no scope change.
5. `tech.md` Lifecycle opening sentence correction ("poll" → "establish the streamed connection") — MINIMAL accuracy repair required by the plan's own new confinement bullet.
6. `context.md` Current Work Focus refresh — caller-directed in the Task-4 prompt; historical blocks untouched; the Immediate-Next-Steps duplicate `1.` numbering follows the plan's explicit insertion rule.
7. §E expectations that proved stale (ya29/AIza zero-hits, `key=` count, `videos/` ignore line 56) — OBSERVED values recorded; no security-relevant deviation.
8. No STOP condition fired: no gate/build failure, no unresolved anchor, no polling-is-selected drift beyond the two sanctioned restorations, no line-count overflow.

### L.8 NOT done by this pass

- 4.3 review/simplification, 4.4 docs pass, 4.5b adherence, 4.6 TODO `[DONE]`/checkbox flips, workflow step 5 (rename/merge/push `origin` ONLY) — all owned by their steps; this pass performed NO push and edited NO TODO file.
- The manual unlisted-broadcast E2E and every live Google/stream claim remain OPERATOR-only.

---

Report/plan: `.kilo/plans/20261029-phase07-task4-verification-docs.md`
Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — Task 4. Produced by the Task-4 4.1b architector pass, 2026-10-29.
