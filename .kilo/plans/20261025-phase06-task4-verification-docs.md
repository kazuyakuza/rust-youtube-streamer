# Phase 06, Task 4 — Offline Tests, Documentation, Builds, and Manual Verification (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — **Task 4 "Offline Tests, Documentation, Builds, and Manual Verification" only.** Do NOT read or expand scope into Tasks 1–3 plans beyond what this file cites.
> Global plan (binding): `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
> Upstream binding records: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (D-1..D-14; §4 lists the Task-4 obligations), `.kilo/plans/20261025-phase06-task2-adherence.md` (§7.2 flags F-1/F-2), `.kilo/plans/20261025-phase06-task3-adherence.md` (§8.1 flags F-1..F-8).
> Report-format precedent: `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`.
> Gate/build conventions precedent: `.kilo/plans/20261024-silent-audio-task2-implementation.md` §6 (verification) and §9 (audit).
> Baseline at planning: branch `feat/phase06-youtube-live-chat`, HEAD `3ae133e`, v0.12.0, tree clean, gate baseline **458 passed / 0 failed** (log `logs/checks/20261026T144323Z.log`).

---

## 0. Scope of this task (TODO Task-4 bullets, verbatim obligations)

1. Deterministic offline tests for the 8 bullet groups in TODO §4's first bullet block — **coverage-audited in §1 below; verdict: ZERO new tests** (every bullet is covered; two adjudications A1/A2 explain why no test is added).
2. Use fake transports/sources and loopback servers only (standing rule; no change needed — no new tests exist).
3. Run the standard gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`.
4. Rebuild Linux: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`.
5. Rebuild Windows: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`.
6. Update operator/user documentation: `docs/production-run.md` (Live Chat prerequisites, expected behavior, diagnostics, failure recovery, privacy), `docs/authentication.md`/`docs/google-cloud-prerequisites.md` only if scopes require, `docs/configuration.md` only if a real new config setting exists, README links.
7. Update `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`.
8. Create the tracked Phase 06 completion report under `.agent/reports/`.
9. Provide the safe operator-run manual E2E procedure (documented, never executed by the agent).

**Hard scope rule for the implementer: ZERO `src/` edits, ZERO `Cargo.toml`/`Cargo.lock` edits, ZERO `config/` edits, ZERO script/Dockerfile edits. This task changes ONLY: `docs/production-run.md`, `README.md`, `docs/google-cloud-prerequisites.md`, the three `.agent/` metadata files, and the new completion report (+ this plan file). If anything seems to demand a src edit: STOP and report (§8).**

---

## 1. Coverage audit (deterministic — TODO bullet → landed test roster)

The Task-2 landings added 38 tests and the Task-3 landings added 29 tests (both re-observed in the 458-gate). Enumerated against TODO §4's 8 bullet groups:

| # | TODO Task-4 bullet group | Covering landed tests (exact names) | Verdict |
|---|---|---|---|
| 1 | Chat ID extraction and missing/disabled chat | `live_chat_resolution_tests` (6): `resolution_returns_the_broadcast_live_chat_id`, `resolution_polls_until_the_chat_id_appears`, `resolution_absent_after_budget_is_typed_not_found`, `resolution_transient_failures_consume_attempts_then_recover`, `resolution_terminal_errors_abort_immediately`, `resolution_wait_cancelled_is_not_translucently_retried`; `live_chat_wire_checks_tests`: `broadcast_chat_lookup_reads_snippet_live_chat_id`, `chat_disabled_error_body_classifies_as_live_chat_disabled`; `chat_worker_tests`: `worker_resolution_absence_after_budget_maps_chat_unavailable`, `worker_chat_disabled_failure_writes_the_chat_unavailable_terminal`; `live_error_tests` +4 (`chat_disabled_reason_maps_typed`, `chat_ended_reason_maps_typed`, `live_chat_not_found_reason_maps_resource_live_chat`, `bare_forbidden_maps_insufficient_permissions`); `live_error_display_tests` +2 | COVERED |
| 2 | Request URLs, query parameters, authorization-header handling, response decoding | `chat_list_sends_bearer_only_with_documented_query` (byte-exact request target `GET /youtube/v3/liveChat/messages?liveChatId=…&maxResults=200&part=snippet,authorDetails&id`, Bearer header present, NO `key=`), `chat_list_appends_page_token_only_when_present`, `chat_list_response_decodes_page_fields`, `chat_list_malformed_body_is_typed_error`; `live_chat_models_tests` (6 decode pins) | COVERED |
| 3 | Provider message → `ChatMessage` mapping, timestamps, normalization, message-type policy, malformed payloads | `live_chat_mapping_tests` (11): full-shape map with id+epoch, displayName→channelId fallback, both-authors-missing skip, non-text/tombstone/unknown-type skip, malformed-payload skip, blank-text skip, API-order preservation, `chatEndedEvent` page marker, `offlineAt` page marker, absent/unparseable timestamp, `rfc3339_parsing_covers_documented_utc_forms`; normalization THROUGH the delivery conversion is pinned by `chat_worker_flow_tests.worker_delivers_mapped_messages_in_page_order_with_metadata` (raw `"line one\nline two"` → rendered `"bob: line one line two"`, id verbatim, epoch timestamp present) | COVERED |
| 4 | Ordering/dedup, empty chat, continuation-token progression, retry/backoff, rate-limit | Ordering: `page_mapping_preserves_api_order` + `worker_delivers_mapped_messages_in_page_order_with_metadata`. Dedup: `live_chat_dedup_tests` (3: unseen-then-seen, evict-oldest-beyond-512, re-seen tolerance) + `worker_dedup_drops_replayed_ids_across_pages`. Empty chat: `empty_items_list_decodes_without_records`, `missing_items_key_decodes_as_empty_items`, `worker_empty_page_advances_the_token_without_inference`, `source_yields_none_while_the_channel_is_open_and_empty`. Continuation: `chat_list_appends_page_token_only_when_present` + the multi-page token-advance flow pins. Retry/backoff: `worker_transient_failures_recover_within_the_budget`, `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal`, `worker_backoff_delay_doubles_and_caps_at_the_maximum`, `resolution_transient_failures_consume_attempts_then_recover`. Rate-limit: `worker_rate_limit_exceeded_is_transient_while_quota_exhaustion_is_terminal` | COVERED |
| 5 | Cancellation during wait/read, bounded queues/backpressure, clean worker shutdown | `resolution_wait_cancelled_is_not_translucently_retried` (cancellation during the O6 wait), `worker_stop_flag_ends_the_loop_before_any_transport_call`, `worker_blocked_list_call_cannot_prevent_bounded_shutdown` (blocked read), `worker_stop_during_delivery_drops_pending_messages_without_blocking` (bounded 64-entry send/backpressure at stop), `confirmed_gate_spawns_and_joins_the_chat_worker` (clean shutdown/join), `cancelled_gate_never_spawns_the_chat_worker` | COVERED |
| 6 | Source → `ChatStore` → renderer integration, log-write failure, continuous frames while chat is idle | Integration + idle-frame continuity: `chat_source_integrates_through_store_and_renderer_in_the_frame_pipeline` (worker-fed messages reach the renderer through the real store; frames render while chat is idle). Log-write failure: store level `append_failure_rejects_message_and_log_stays_consistent` + `log_open_failure_is_typed_and_surfaces_path` (`src/chat/store_tests.rs`); runner level `run_propagates_message_source_failure_without_rendering` (the exact `ChatError::LogAppend` shape → `FramePipelineError::Chat`, no render, no sink finalize) — see Adjudication A1 | COVERED (A1) |
| 7 | Chat failures flowing to the typed `run` error/exit path | `run_maps_retrieval_failure_to_the_chat_retrieval_variant` (`runner_retrieval_tests.rs`), `chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate` (Display line + exit 1 + gate-Fatal precedence + not-a-cancellation), `chat_retrieval_pipeline_error_display_and_source_are_pinned` (pipeline `error_tests.rs`), `source_error_tests` (3: stable secret-free Display lines, delegation, the pure mapper), `run_origin_failures_map_to_the_startup_failure_code` (`RunError::ChatRetrieval` rides the all-variants exit-1 loop) | COVERED |
| 8 | Preservation of `demo` schedule, local MP4 argv, production RTMPS destination redaction, clean Ctrl+C stop | `demo` 14 tests byte-green; `args_tests` locked local vector; `stream_args_tests` 33/42-token CBR vectors; redaction suite (`rtmps_ingestion_tests` 6, `rtmps_destination_tests` 2, `ffmpeg_stream_tests` incl. the `[redacted]` pin); `mod_tests.stop_messages_and_success_exit_are_pinned`; `shutdown_order_cancels_the_wait_before_joining_the_gate`; `ffmpeg_isolation_tests` (2); `streaming_stage_tests` argv pin | COVERED |

### Adjudication A1 — log-write failure via source→store integration (no new test; rationale recorded)

The one literal composition not directly exercised is a REAL `ChatStore` log-append failing inside `pipeline.run()` at the runner's `chat_store.accept(...).map_err(FramePipelineError::Chat)?` call site (`src/app/pipeline/runner.rs`, accept arm). Inducing a post-construction append failure offline requires swapping the store's PRIVATE `log_file: File` field — the technique `src/chat/store_tests.rs` uses, available ONLY to that child module. A runner-level test cannot do this without widening the field's visibility, which is a production-code change — PROHIBITED in this docs/tests task. The coverage that exists is compositionally complete: (a) the real store produces `ChatError::LogAppend` on append failure and keeps queue/log consistent (store-level pins); (b) the runner surfaces that exact error type as `FramePipelineError::Chat` and ends the run without rendering or finalizing (the scripted-source pin uses precisely `ChatError::LogAppend`); (c) the accept-site mapping is the same one-line `ChatError` → `FramePipelineError::Chat` conversion the scripted pin exercises. **Verdict: COVERED-BY-COMPOSITION. Add NO test. Touch NO src file.**

### Adjudication A2 — streaming-reconnect semantics (N/A; documented)

The Task-1 record D-1 SELECTED `liveChatMessages.list` polling and REJECTED `liveChatMessages.streamList`. Polling has no reconnect machinery: the resume-after-gap semantics ARE the `nextPageToken` continuation (pinned by `chat_list_appends_page_token_only_when_present` + the token-advance flow tests), and the brief §8.3 history rule is met by the first-page baseline discard (pinned by `worker_discards_the_entire_baseline_history_page`). A "streaming reconnect" test would test a mechanism that does not exist. **Verdict: N/A — no test; the docs must state the selected method and why no reconnect exists (§2.1 Expected Behavior).**

### Audit verdict

**ZERO new automated tests.** Expected gate count stays **458 passed / 0 failed** (docs-only tree). OBSERVED GOVERNS: record the exact observed count and log path from the fresh run.

---

## 2. Docs deliverables

### 2.1 `docs/production-run.md` — the Live Chat section (the main deliverable)

Add ONE new H2 section **`## Live Chat (Behavior, Diagnostics, Recovery, Privacy, Quota)`** between "Broadcast/Stream Fate (Policy and Honest Limitations)" and "Ingest Probe Procedure (Operator-Run)", with the matching TOC entry inserted in the same position. Six H3 subsections, in this order, carrying ONLY the facts below (all values are binding records D-1..D-14 — do not invent alternatives):

1. `### Prerequisites` — (a) scopes: the already-granted `https://www.googleapis.com/auth/youtube.force-ssl` is documented-sufficient for BOTH chat calls (`liveChatMessages.list` and the `liveBroadcasts.list` chat-id lookup — official scope bindings, Task-1 record §1.4); **re-running `auth` is NOT required for chat** (no scope change since Phase 04); (b) the standing Google Cloud + channel live-access prerequisites (pointers to `docs/google-cloud-prerequisites.md`); (c) live chat is enabled by default on live broadcasts and stays available while the event is active; (d) NO new configuration field exists — polling pacing is SERVER-PROVIDED ONLY (`pollingIntervalMillis`); nothing to configure.
2. `### Expected Behavior` — the retrieval method: polling `liveChatMessages.list` (GET `https://www.googleapis.com/youtube/v3/liveChat/messages`) honoring the server's `pollingIntervalMillis` and `nextPageToken`; `streamList` was evaluated and REJECTED (it requires a gRPC server-streaming client — an async/gRPC dependency stack this synchronous service deliberately does not carry; both methods cost the same 5 quota units per call; cite the Task-1 record §2 D-1). The worker timeline in operator terms (F-3): the worker thread starts ONLY after the confirmation gate observes `streamStatus == active`; its FIRST act resolves the broadcast's `snippet.liveChatId` via `liveBroadcasts.list(part=snippet)` with a bounded budget (6 attempts × 5 s = 30 s) — absence after the budget is a fatal typed error; the worker's first successful page is the BASELINE: ALL of its items are discarded (they are history — never queued, never logged), so messages sent before the stream went live do not appear; on later pages, ordinary text messages (author + text) are accepted at most one per frame, appended to the chat log, and rendered; replayed message IDs are dropped (dedup window); an empty page just advances the token — emptiness NEVER infers the chat ended; while chat is idle, slow, or between messages the frames continue (black when no visible text). Message/event policy: ONLY `textMessageEvent` messages render; Super Chats, Super Stickers, membership/sponsor/gifting events, milestone chats, polls, bans, tombstones, and any unrecognized future event type are SKIPPED — never displayed, never logged.
3. `### Diagnostics` — point at the three new rows in the "What the Logs Look Like" table (below) and enumerate the typed error surfaces (the single redacted `error:` line, exit 1): `live chat is unavailable: <detail>` (chat disabled / not found / permissions / the resolution budget exhausted), `live chat retrieval stopped by YouTube quota: <detail>` (quota exhaustion), `live chat transport failed after repeated attempts: <detail>` (the bounded transient budget exhausted). The `detail` is the redacted Display of the underlying typed YouTube error (e.g. the liveChatDisabled/liveChatEnded/liveChatNotFound reasons) — never a raw body.
4. `### Failure Recovery` — the graceful-vs-fatal table (from the Task-1 record §1.6): broadcast ended/chat closed (`chatEndedEvent`, `offlineAt`, or `403 liveChatEnded`) → GRACEFUL: one info line, the worker stops, video keeps flowing until Ctrl+C; chat disabled (`403 liveChatDisabled`) → fatal; wrong/expired chat id (`404 liveChatNotFound`) → fatal; permissions revoked mid-run (`403 forbidden`/401-class) → fatal, recovery = re-run `auth`; quota exhausted (`quotaExceeded`/`dailyLimitExceeded`) → fatal, recovery = wait for the midnight PT reset or Google's quota-extension form; transient network/backend/malformed/`rateLimitExceeded` failures → bounded retry (backoff = max(server interval, 1 s) × 2^(attempt−1), capped at 60 s; at most 10 consecutive failures), then fatal transport error. Note the honest boundedness facts (F-3): a Ctrl+C stops the worker FIRST (before FFmpeg's stdin EOF); the join is bounded (≤ ~1 s wake; ≤ ~30 s worst case during the one-shot chat-id resolution window); up to 64 queued-but-undelivered messages can be lost at shutdown — they are never logged, so the log and the video never diverge.
5. `### Privacy` — NEVER logged anywhere (stderr events, error lines, or diagnostics): message text, author names/display names, individual users' message IDs, access tokens, authorization headers, raw response bodies, stream keys, RTMPS URLs. The ONLY place message content (author + text) is recorded is the chat log file itself (`chat.log_file`, default `logs/chat.log`, gitignored under the `logs/*` rule) — that is its documented purpose. The `live_chat_id` tracing field on the resolution line is a NON-secret operator diagnostic (same class as `broadcast_id`).
6. `### Quota (What Chat Polling Costs)` — the D-10 table VERBATIM (chat polls cost 5 units/call; the one-time chat-id resolution lookup costs 1 unit; default project quota 10,000 units/day, reset midnight Pacific Time):

   | Chat poll interval (server) | Units/hour | Continuous-session hours before the 10,000/day default is exhausted |
   |---|---|---|
   | 5,000 ms | 3,600 | ≈ 2.8 h |
   | 10,000 ms | 1,800 | ≈ 5.5 h |
   | 13,000 ms | ≈ 1,385 | ≈ 7.2 h |
   | 20,000 ms | 900 | ≈ 11.1 h |

   Operator guidance (D-10): a one-off operator session of a few hours fits comfortably; a multi-hour unattended service can exhaust the $0 default — mid-stream exhaustion is typed-fatal (exit 1 with the honest error); the remedies are Google's documented quota-extension form or shorter sessions. If ANY table figure is questioned, STOP and report — never reinterpret quota math in the field (Task-1 record §5.6 rule).

**Additional edits in the same file:**

- **Log-lines table** ("What the Logs Look Like (Without Secrets)", F-2): append three `info` rows for the chat events in chronological position — after the watch-link row, before the gate-timeout rows: (i) `live chat id resolved` — carries the non-secret `live_chat_id` field; fires after the confirmation gate confirmed the stream active; (ii) `live chat baseline established; the history page was discarded` — carries `discarded_messages` (the count of history items dropped); fires once at the worker's first successful poll; (iii) `live chat ended; the stream continues without new chat messages` — the graceful end-of-chat signal; the video keeps flowing until Ctrl+C. Renumber the table's order column honestly (the existing table already repeats numbers for alternative rows — keep that convention).
- **Startup section, stage 10** (F-3): replace the parenthetical "(a bounded live-chat worker feeds incoming messages into it; the worker's startup/shutdown ordering details are part of the pending Phase 06 documentation pass)" with the real stage-7.5 wording: the chat worker thread starts after the gate confirms `streamStatus == active`, resolves the broadcast's live chat id (bounded 30 s budget), then polls on the server-instructed interval; on Ctrl+C the worker stops FIRST, before FFmpeg's stdin is closed; the join is bounded (≤ ~1 s; ≤ ~30 s during the one-shot resolution window).
- **Log-inspection paragraph** ("Inspecting the logs"): replace "chat-log privacy and handling are covered by the pending Phase 06 documentation pass" with a pointer to the new Live Chat section's Privacy subsection.
- **Manual Verification Procedure, step 6** (F-1): replace the deferral clause with the chat-bearing operator steps (see §2.4).
- **Known Stream-Health Limitation section**: drift-check only — the existing text ("the Phase 06 chat text now wired into `run` adds real — though still modest — content bits when messages arrive (operator Stream-Health confirmation with chat content is pending)") stays TRUTHFUL; the operator Stream-Health confirmation remains pending (never claim it). Edit only if a now-false sentence is found.
- **Overview bullet/paragraph**: refresh the two "pending Phase 06 documentation pass" phrases (line 28 area) to point at the new section.

### 2.2 `README.md` — refresh the pending markers + test-count re-record (F-4)

1. Current Status paragraph (line ~44): replace "the operator-facing chat documentation, the Phase 06 completion report, and the real-broadcast manual verification remain pending" with the final state — the chat documentation and the Phase 06 completion report are landed; the real-broadcast manual verification remains OPERATOR-RUN and was not executed by the agent.
2. Test-count sentence (line ~46): replace the superseded-figure wording with the OBSERVED count from the fresh gate (expected 458; OBSERVED GOVERNS).
3. `run` command bullet (line ~61): drop the "(the Phase 06 chat wiring is landed in code and offline tests; the chat documentation pass and the real-broadcast verification are pending)" parenthetical; point at `docs/production-run.md`'s Live Chat section.
4. Production Run section (line ~120): the sentence "The Phase 06 chat wiring is landed in code and offline tests; the operator-facing chat documentation (prerequisites, behavior, diagnostics, recovery, privacy) and the real-broadcast manual verification are pending." → the documentation is landed (link the Live Chat section); the manual verification stays operator-run.

### 2.3 `docs/google-cloud-prerequisites.md` — two deferral-pointer truth restorations

1. API table row (line ~44): "implemented in code/tests; operator verification pending" → "implemented and documented; see [`docs/production-run.md`](production-run.md)".
2. Quota bullet (line ~140): replace "the detailed quota accounting for a real broadcast belongs to the pending Phase 06 completion report and operator documentation pass" with a pointer to the Live Chat quota subsection now in `docs/production-run.md` (and the completion report). Keep the ≈17,000-units/day illustrative math coherent with the D-10 table.

### 2.4 The operator-run manual E2E (TODO §4 last bullet) — in `docs/production-run.md` step 6 (and summarized in the report)

Rewrite manual-procedure step 6 to include, in order: (a) an explicit note that re-running `auth` is NOT required for chat (no scope change; `youtube.force-ssl` suffices — run `auth` only per the existing recovery paths); (b) confirm video starts BEFORE messages arrive (the stream opens black); (c) send ordinary text messages from a SEPARATE account after the stream is confirmed active; (d) expect them to render as author/text on the watch page within the server's poll interval (allow up to ~30 s for one cycle + render); (e) confirm the same messages appear in the chat log (`logs/chat.log` or the configured `chat.log_file`); (f) test an empty-chat period — frames continue, nothing duplicated, no history replay; (g) note that messages sent before the stream went live are history-discarded by design; then the existing Ctrl+C / exit-0 / auto-complete steps stand. Keep ALL existing privacy/scope-reminder steps intact.

### 2.5 Drift-check-only verdicts (expected: NO edit)

- `docs/configuration.md` — D-12: NO new config field exists (the 21-rule count stands; pacing is server-provided). Drift-check only. If drift is found: STOP and report (out-of-plan edit).
- `docs/authentication.md` — Task-1 verdict: `youtube.force-ssl` suffices; no scope change. Drift-check only. If drift is found: STOP and report.
- `docs/demo.md` — the demo surface is untouched. Drift-check only.
- `docs/build.md` / `docs/build-windows.md` — drift-check only.

---

## 3. Metadata refreshes

### 3.1 `.agent/project-structure.md` (F-6 + Task-2 F-1 variant-count flag)

Re-cut the STALE entries to the final-tree truth. OBSERVE every count fresh (`git ls-files <dir>` host-side, or `ls` on the VM) — never hardcode without observation:

1. `app/` → `run/` header: "19 files: 9 sources + 10 test/fixture files" → the observed count (expected 28 files = 12 sources + 4 fixtures + 12 test rosters).
2. `run/error.rs` bullet: 8 → **9 variants** (the Task-3 `ChatRetrieval` append).
3. `run/source.rs` bullet: `ChatFreeSource` → the `ChatSource` rewrite (worker-fed bounded channel + terminal cell; the Phase-05 chat-free text is stale).
4. NEW `run/` bullets: `chat_failure.rs` (the closed transient class + the three-way terminal mapping), `chat_worker.rs` (the bounded worker: 64-entry channel, stage-7.5 constructor, D-12/D-13 constants, spawn-at-Confirmed), `chat_worker_delivery.rs` (poll cycle: baseline discard, server-interval honor loop, stop-aware try_send delivery, stop-sliced sleeps, backoff cap), and the rosters `chat_source_tests` (5), `chat_worker_tests` (6), `chat_worker_flow_tests` (4), `chat_worker_budget_tests` (6), `gate_chat_tests` (2), `runtime_chat_fakes.rs`.
5. `run/gate.rs` / `seams.rs` / `streaming.rs` / `mod.rs` bullets: the chat-worker spawn/join duty, `BroadcastTransports`, the `StreamingExecution.source` field, stage 7.5.
6. `pipeline/` header: "14 files: 7 sources + 5 test files + 2 helpers" → the observed count (expected 17 = 8 sources + 7 test files + 2 helpers); the `source.rs` bullet "BYTE-unchanged for Phase 06" → the widened `ChatSourceError` contract; NEW bullets `source_error.rs`, `source_error_tests.rs` (3), `runner_retrieval_tests.rs` (1); the pipeline `error_tests.rs` +1 (`ChatRetrieval` Display/source pin).
7. `youtube/` header: "66 files: 35 sources + 31 test files" → the observed count (expected 80 = 42 sources + 38 test files). Update the per-roster test total (164 → observed; expected 202).
8. `live_error/` bullet: "**15 variants**" → **17 variants** (Task-2 `ChatDisabled`/`ChatEnded`) — this is the standing Task-2 F-1 flag; also refresh the classify/display bullets.
9. NEW `youtube/` bullets for the 7 chat production files (`live_chat_models.rs`, `api_chat_transport.rs`, `google_live_chat_transport.rs`, `live_chat_resolution.rs`, `live_chat_mapping.rs`, `live_chat_dedup.rs`, `live_chat_rfc3339.rs`), the fixtures (`live_chat_test_fixtures.rs`), and the rosters (models 6, mapping 11, dedup 3, resolution 6, wire checks 6 + the wire server sibling, live_error +4, display +2); the staging state (the 6 chat module stagings retired by Task 3; the retained re-export allows).
10. Root files: `Cargo.toml` — "v0.11.1" → **v0.12.0**.
11. Dead-code staging inventory section: re-run the fresh `allow(dead_code)`/`allow(unused_imports)` greps and re-cut the composition honestly (expected per Task-3 adherence §9: 9 dead_code/clippy attribute lines + 3 unused_imports braces; the `with_timestamp` and chat-module stagings retired; the V2 stagings stand; record OBSERVED numbers).
12. `docs/` enumeration: `production-run.md`'s description gains "the live chat behavior/diagnostics/recovery/privacy/quota section".

### 3.2 `.agent/project-info/tech.md`

1. Fix the STALE Stack bullet "Tokio remains reserved for later YouTube API/chat streaming phases" — the chat retrieval landed WITHOUT Tokio: a bounded std thread + fixed-capacity `std::sync::mpsc::sync_channel(64)`; Tokio never entered the tree.
2. Add one Phase-06 bullet (Dependencies or Lifecycle): the retrieval method (D-1 polling verdict; O6 `liveBroadcasts.list` chat-id resolution 1 unit; O7 `liveChatMessages.list` 5 units/call honoring `pollingIntervalMillis`/`nextPageToken`; `streamList` rejected — gRPC/async stack unjustified), the std-thread bridge (try_send-only stop-aware sends, 1 s stop-sliced sleeps, shared `StopFlag`, the terminal-cell error funnel), and ZERO new dependencies (`Cargo.toml` changed by the version line only).

### 3.3 `.agent/project-info/context.md`

Add the Task-4 work-focus block (the standing convention): what Task 4 delivered (coverage audit verdict, the docs set, the metadata refreshes, the gate/build evidence with OBSERVED numbers, the report path), what remains (4.3/4.5b/4.6/step 5; the operator-run manual E2E). No secrets, no real-Google claims.

---

## 4. Verification sequence (all via the Alpine VM MCP; `vm_status` first each batch; ONE command per invocation)

| # | Command (verbatim) | Expected |
|---|---|---|
| S1 | `git status --porcelain` → `git branch --show-current` → `git log --oneline -3` | clean; `feat/phase06-youtube-live-chat`; HEAD `3ae133e` before the plan commit |
| S2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | exit 0; four PASS lines; **458 passed / 0 failed** (expected; OBSERVED GOVERNS); `ALL CHECKS PASSED`; record the UTC log path |
| S3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` | exit 0; OBSERVED size printed |
| S4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` | exit 0; OBSERVED size printed |
| S5 | `ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` | both sizes match the script-printed values exactly (never doc-hardcode sizes) |

Run the gate AFTER all docs/metadata edits (the final tree), then the builds, then the hygiene/secret audit (§6 step 5 of the silent-audio precedent):

- `git status --porcelain` → empty (after the docs commits).
- `git ls-files dist/ videos/` → empty; `git ls-files credentials/` → `.gitkeep` only.
- `git check-ignore -v dist/rust-youtube-streamer-service` → a `dist/` rule hit.
- `git grep -n -E "ya29|AIza" -- src/ docs/ README.md` → ZERO hits.
- `git grep -n -F "key=" -- src/ docs/` → the pre-existing doc-context count only.
- `git grep -n -F "allow(dead_code)" -- src/` + `allow(unused_imports)` + `allow(clippy` → the observed pre-existing counts (no NEW lines).
- No `rtmps://` beyond the pre-existing `.example.test` fixtures; no token-shaped strings.

---

## 5. The Phase 06 completion report (F-7)

Create **`.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`** (tracked) with a TOC, mirroring the Phase-05 report's section shapes and carrying EVERY Task-1-record §4 obligation + observed evidence:

1. **Overview** — branch, v0.12.0, the phase summary (chat worker wired into `run`; Task-4 base HEAD `3ae133e`; the report's own commit hash intentionally absent).
2. **Official Sources and Access Dates** — the S1–S11 citation table from the Task-1 record §1 (URLs + "Last updated"/access dates).
3. **Selected and Rejected Retrieval Approach** — D-1 with the grounded rationale.
4. **API Operations and Wire Forms** — O6/O7 exact paths/params (D-2/D-3).
5. **Message and Event Policy** — D-4/D-5 (accept/skip policy, baseline discard, dedup window, ordering).
6. **Error, Retry, Backpressure, and Shutdown Decisions** — D-6/D-7/D-8/D-9 (graceful vs fatal classes, the closed transient list, backoff shape, the widened seam, the funnel + precedence).
7. **Quota Analysis** — the D-10 table + operator implications.
8. **Dependencies** — D-11: zero new; `Cargo.toml` version line only (diffstat-proven).
9. **Files Created/Changed** — per-task commit chain table (Task-2 span, Task-3 span, Task-4 docs/report commits).
10. **Test Coverage Audit** — this plan's §1 table (8 bullets → rosters) + the A1/A2 adjudications + the OBSERVED gate count.
11. **Exact Commands and Exit Statuses** — the §4 runs (gate + builds + ls -l) verbatim.
12. **Build Results** — OBSERVED artifact sizes + the cross-compilation-is-not-runtime-validation caveats.
13. **Manual Verification Procedure (operator-run)** — the §2.4 summary + the explicit header that the agent did NOT execute it.
14. **Checks Actually Performed vs NOT Performed** — the honest manual-verification limitation section: the manual E2E was NOT executed by the agent; native Windows/Linux runtime of the chat path NOT RUN; no live Google call was made in the phase; the operator's channel-activation hold (2026-10-14 operator fact) still applies to the live E2E.
15. **Known Limitations** — the recorded incidentals: the ≤64-message shutdown-loss window; the ≤30 s resolution-window Ctrl+C join delay (raw 5 s slices); the baseline-history discard window (messages between stream-active and the first poll are dropped with history); ureq carries no request-level timeout; the stream-health note stays honest.
16. **Evidence Index** — plans/adherence records, gate logs, `git log --oneline main..HEAD` at report time.

---

## 6. Git handling (implementer, 4.2; the gitignore-compliance rule applies before EVERY commit)

1. S1 state check (§4 row 1). If not clean/on-branch/HEAD-pinned → STOP.
2. Commit THIS plan file ONLY: `git add .kilo/plans/20261025-phase06-task4-verification-docs.md` → `chore: track the phase06 task4 verification docs plan`.
3. Apply §2 docs edits + §3 metadata edits. Read `.gitignore` once before committing (gitignore-compliance rule).
4. Commit the user-facing docs: `git add docs/production-run.md README.md docs/google-cloud-prerequisites.md` → `docs: document the youtube live chat integration for operators`.
5. Commit the metadata: `git add .agent/project-structure.md .agent/project-info/tech.md .agent/project-info/context.md` → `docs: record phase 06 project metadata`.
6. Run the verification sequence (§4) on the final tree; record OBSERVED results.
7. Write the completion report (§5) citing those results → commit: `git add .agent/reports/20261026-phase-06-youtube-live-chat-integration.md` → `docs: add the phase 06 completion report`.
8. Final: `git status --porcelain` → empty; NO push, NO merge, NO TODO `[DONE]` marks, NO acceptance-criteria flips (the caller's 4.6/step-5 own those).

## 7. Stop-guards (any holder; STOP and report — never self-heal)

1. Pre-state mismatch (branch/HEAD/dirty tree with unknown files).
2. The gate exits non-zero or any FAIL line appears.
3. Either build fails, or `ls -l` shows a missing/0-byte artifact, or the `ls -l` size disagrees with the script-printed size.
4. The audit (§4) hits ANY denied condition (staged ignore-hit, secret token count above baseline, a NEW `allow(` attribute line).
5. Any doc sentence would claim real-Google/Studio verification, a Stream-Health confirmation, or an executed manual E2E — such claims are FORBIDDEN; if content seems to need one, STOP.
6. Any quota-table figure is questioned → STOP (never reinterpret D-10 in the field).
7. Drift is found in the §2.5 drift-check-only docs → STOP and report (out-of-plan edit).
8. ANYTHING demands a `src/`, `Cargo.toml`, `config/`, script, or Dockerfile edit → STOP (zero-code-edit task).
9. A `.gitignore`-matching file would be staged → unstage; never force-add.

## 8. Sub-step applicability (codified)

- **4.3 review/simplification: REQUIRED** (docs/report review; the reviewers are bound to this plan's §2/§3/§5 scope; there is no src to simplify — any src-affecting finding is a STOP).
- **4.4 docs pass: subsumed drift-check** — the docs-specialist verifies the REMAINING docs (`docs/build.md`, `docs/build-windows.md`, `docs/demo.md`, `docs/configuration.md`, `docs/authentication.md`) and TOC/anchor conventions; expected NO edit.
- **4.5b adherence: REQUIRED** (architector re-check against this plan + the Task-1 record §4 + the F-1..F-8 flags).
- **4.6 completion: implementer** (TODO Task 4 `[DONE]` + acceptance-criteria flips + the final commit).

## 9. Explicitly NOT done by Task 4 (flagged)

- No src edits, no new tests, no dependency/config changes (§1 verdict).
- No push, no merge, no TODO `[DONE]`/criteria flips (4.6/step 5).
- The manual unlisted-broadcast E2E is DOCUMENTED, never executed by the agent; its results, when the operator records them, are never build evidence.
- No native Windows/Linux runtime validation of the chat path (container-only gate/builds).

## 10. Plan verification against the original TODO task

| TODO Task-4 bullet | Covered by |
|---|---|
| Deterministic offline tests (8 groups) | §1 audit — all COVERED; A1/A2 adjudications; zero new tests |
| Fakes/loopback only | §1 (standing rule; no new tests) |
| dev-checks gate | §4 S2 |
| Linux rebuild | §4 S3/S5 |
| Windows rebuild | §4 S4/S5 |
| Docs: production-run (prereqs/behavior/diagnostics/recovery/privacy), conditional auth/gcp, conditional configuration, README | §2.1–§2.5 (Task-1 verdicts: scopes need NO edit → verify-only; config needs NO edit → drift-check) |
| Metadata: structure/tech/context | §3.1–§3.3 (incl. the Task-2 F-1 "15 variants" flag and Task-3 F-1..F-8) |
| Tracked Phase 06 completion report (URLs/dates, approach, quota, deps, policy, decisions, files, commands, counts, sizes, manual procedure, limitations) | §5 |
| Safe operator-run manual E2E procedure | §2.4 + §5 item 13/14 |

## 11. Footer

Plan: `.kilo/plans/20261025-phase06-task4-verification-docs.md`
Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — Task 4 only. Global plan: `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
Produced 2026-10-26 by the Task-4 4.1b architector pass. NOT implemented, NOT committed by this pass.
