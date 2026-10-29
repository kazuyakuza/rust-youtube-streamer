# Phase 07 Task 3 — Review Cycle 1 (reviewer half)

**Ruling:** no fixes required. The implementation matches the binding plan `.kilo/plans/20261029-phase07-task3-integration-switch.md` (§A–§L), the TODO Task-3 bullets, and the binding Task-1/Task-2 design records.

---

## Verification matrix

| Plan section | What was verified | Evidence |
|---|---|---|
| §B.1 / §B.4 `chat_worker.rs` | One runtime site (`chat_runtime`), current-thread, built once on the worker thread; `run_chat_worker` stops early, resolves chat id, then `block_on`s the stream loop. | `src/app/run/chat_worker.rs:107-140`; line count 183 ≤ 200. |
| §B.1 / §B.5 `chat_stream_worker.rs` | `StreamWorker` fields/constructor, `ConsumeVerdict`, `run_stream_loop`, `establish`, `consume` with `tokio::select! { biased; … }` and stop-tick sleep. Stop slice is 0 in tests, 1_000 ms in production. | `src/app/run/chat_stream_worker.rs:15-152`; line count 153 ≤ 200. |
| §B.1 / §B.6 / §B.7 `chat_stream_delivery.rs` | Failure policy (`is_mid_session_chat_end`, `handle_failure`, `handle_clean_close`, `await_reestablishment`), batch processing, delivery, backoff, terminal writer. | `src/app/run/chat_stream_delivery.rs:27-185`; line count 193 ≤ 200. |
| §A.3 / §L.2(1) `process_batch` ordering | History-pending batch is discarded whole while still retaining the token and clearing the flag; live path runs only after history is cleared. Matches §A.3 semantics and §E.2 pins. | `src/app/run/chat_stream_delivery.rs:32-55`; tests `worker_discards_the_entire_history_batch_before_the_first_token` and `worker_failed_connect_before_the_first_batch_reestablishes_tokenless_then_discards_history` pass. |
| §B.2 / §B.8 `seams.rs` + `run/mod.rs` | `ChatStageSources`, `ChatStreamHandle`, `BroadcastTransports.stream` added; both constructor sites updated; `build_transport` builds stream from cloned token first, then `Arc` transport from the token; `build_chat_stage` call site uses `ChatStageSources`. | `src/app/run/seams.rs:41-57`; `src/app/run/mod.rs:89-97`, `147-154`. |
| §D.2 gate/source/pipeline/demo/streaming untouched | `git diff b2f7b08..45fe2b9 --stat -- src/app/run/gate.rs src/app/run/source.rs src/app/pipeline src/app/demo src/streaming` is empty. | Confirmed empty diff. |
| §C.1–C.3 / C.6 retirement | `api_chat_transport.rs` shrunk to lookup-only; `google_live_chat_transport.rs` lookup-only; `live_chat_models.rs` survivor-only; `live_chat_mapping.rs` carrier + parser survivors. | Read files; greps for `list_messages`, `chat_messages_path`, `ChatListCommand`, `map_page`, `ChatPageOutcome`, `DEFAULT_CHAT_POLL_INTERVAL`, `polling_interval_millis`, `Staged for the Task-3` all return zero. |
| §C.9 `LiveChatMessageList` grep nuance | 27 hits are all Task-2 proto names (`LiveChatMessageListRequest`/`Response`); zero hits on the retired REST model. (Corrected from "25" per the docs-align pass `ff05536` re-observation, L.6.) | `git grep -n "LiveChatMessageList" -- src` reviewed manually. |
| §C.5 staging-retirement ledger | `src/youtube` `allow(` inventory = exactly the 5 standing lines (`authorization.rs` ×2, `error.rs` ×1, `mod.rs` ×2); zero staging; `#[allow(clippy::manual_clamp)]` moved to `chat_stream_delivery.rs`. | `git grep -n "allow(" src/youtube` = 5. |
| §A.5 failure policy / classifier | `chat_failure.rs` classifier is untouched (only test module registration added); `is_transient_failure` stays closed; `ChatPrecondition` dual-branch uses worker-lifetime `delivered_batches > 0`; clean close maps to transient re-establishment. | `git diff b2f7b08..45fe2b9 -- src/app/run/chat_failure.rs` shows only the `chat_failure_tests` module addition. |
| §E run-roster fidelity | 22 worker tests + 2 failure tests + 2 gate_chat tests + 5 runtime tests; scripted-termination rule (fakes panic on exhaustion); two 250 ms stall pins documented; zero-sleep rule respected elsewhere. | `cargo test --locked chat_worker` → 22 passed; `chat_failure_tests.rs` has 2 tests; `runtime_tests.rs` has 5 tests; `gate_chat_tests.rs` has 2 tests. |
| §G / §L.7 29-file inventory | 30 files changed including the plan file; excluding the plan, 29 files. 5 new files, 3 deleted, 21 modified (including the 3 plan-mandated consequential edits). | `git diff b2f7b08..45fe2b9 --stat`. |
| Open-code sanity | Zero remaining `poll`/`poll_cycle`/`polling_interval` in chat path; `is_graceful_end` gone; `ChatWorkerSession` gone; `tonic::async_trait` absent from `src/app`; exactly two `tokio::runtime` sites. | Greps confirm. |
| Gate | Full `scripts/dev-checks.sh` passes: 493 passed / 0 failed; fmt/check/test/clippy all PASS. | Fresh run `logs/checks/20261029T193048Z.log` and implementer's `logs/checks/20261029T192248Z.log`. |
| Clippy | `cargo clippy --locked -- -D warnings` exit 0. | Recorded in gate log. |
| windows-gnu | Context states exit 0; not re-run in this cycle. | Caller-provided log. |

---

## Adjudicated deviations observed (all consistent with §L)

1. `process_batch` order was corrected so the history-pending discard happens before the live path, preserving §A.3 semantics — verified in code and tests.
2. `pub(super)` visibility on `StreamWorker` fields/methods and on `STREAM_CLEAN_CLOSE_DETAIL` due to the cross-file `impl StreamWorker` split — acceptable mechanical fix-up.
3. `stream_run_fixtures.rs` uses an `Fn()` stop-flip hook and a `stall_next_batch_for(millis)` one-shot real-stall hook, because `src/youtube` cannot name the app-private `StopFlag` and the 250 ms pin must stall the fake `next_batch` — matches §L.2.
4. Roster arithmetic: `live_chat_wire_checks_tests` 6→2 (not 3), `chat_worker_budget_tests` 8 (7 + E.5 latency pin), gate total 493 — observed values govern.
5. Module-doc trims under the §B.10 cap contingency — all source files remain ≤ 200 lines.

---

## Non-blocking note for Task 4 (not a Task-3 defect)

- `.agent/project-structure.md` still describes the retired Phase-06 poll worker (`chat_worker_delivery.rs`, `map_page`, `ChatPageOutcome`, etc.). Plan §I explicitly assigns structure-map/`tech.md`/`context.md`/docs updates to Task 4, so this is expected drift to be closed there.

---

## Reviewer conclusion

No junior-executable fixes are required. Cycle 1 passes; no further review cycles needed for Task 3.
