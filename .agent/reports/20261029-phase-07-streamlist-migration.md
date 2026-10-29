# Phase 07 — YouTube Live Chat streamList Migration (Real-Time Retrieval): Completion Report

> Branch `feat/phase07-streamlist-retrieval` — version v0.13.0 (pre-merge at the time of writing).
> Authority: TODO `.agent/todos/20261029/20261029-todo-1.md`; global plan `.kilo/plans/20261029-phase07-streamlist-migration.md`; per-task plans under `.kilo/plans/20261029-phase07-task*.md`.
> Every number in this report is an OBSERVED value from a command run inside this pass (or a committed record cited by path); no prediction is restated as a result, and no live Google observation is claimed by the agent.

## Overview

Phase 07 replaces the Phase-06 polling retrieval (`liveChatMessages.list` at the server-directed interval) with the official low-latency server-streaming retrieval (`liveChatMessages.streamList`) implemented as a gRPC-over-HTTP/2+TLS client (Path A: `tonic` + `prost` + a single confined `tokio` current-thread runtime on the dedicated chat worker thread). The bounded worker→frame-loop seam is byte-unchanged (`std::sync::mpsc::sync_channel(64)` + `try_recv` + terminal cell). The establishment is a one-time call; re-establishment happens only after a drop and carries the accumulated `nextPageToken` — never on a fixed/periodic schedule. The Phase-06 `list` production surfaces (trait method, wire implementation, REST list models/mappers, poll-loop wiring, REST-list rosters) were RETIRED; O6 chat-id resolution, the mapping/dedup layer, the failure classifiers and `demo` survive untouched.

Commit chain per task (observed in `git log`):

- Task 1 (research/design): `f8628e2` → `ed583a7` → `aea846c` → `0024642`.
- Task 2 (client + bridge): `dccacbc` → `03e696f` → `229e9cb` → `3f6d1b0` → `75788ae` → `35adefa` → `4229e36`.
- Task 3 (integration switch): `0266eff` → `dc3312a` → `45fe2b9` → `0647860` → `ff05536` → `37ff256` → `ca9c55f`.
- Task 4 (verification/docs): `f12027a` (plan tracked) → `7ab6eee` (burst pin) → `1e16c3f` (operator docs) → the metadata/report commit that contains this file.

## Official Sources and Access Dates

All accessed 2026-10-29 by the Task-1 research pass; citations reproduced from `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` §1.

| # | Source | Kind | URL | Page status |
|---|---|---|---|---|
| S1 | `LiveChatMessages: streamList` method reference | OFFICIAL (Google) | https://developers.google.com/youtube/data/live/reference/rest/v3/liveChatMessages/streamList | "Last updated 2026-10-15 UTC" — unchanged |
| S2 | Guide: "Streaming Live Chat" (gRPC demo + inline `stream_list.proto`) | OFFICIAL (Google) | https://developers.google.com/youtube/data/live/streaming-live-chat | "Last updated 2026-10-15 UTC" — unchanged |
| S3 | YouTube v3 REST discovery document | OFFICIAL (Google, live fetch) | https://www.googleapis.com/discovery/v1/apis/youtube/v3/rest | fetched live 2026-10-29 |
| S4 | Quota Calculator (per-method costs + default allocation) | OFFICIAL (Google) | https://developers.google.com/youtube/data/determine_quota_cost | "Last updated 2026-10-15 UTC" — unchanged |
| S5 | tonic crate registry object | REGISTRY METADATA (crates.io) | https://crates.io/api/v1/crates/tonic | fetched live 2026-10-29 |
| S6 | tonic 0.13.0 / 0.13.1 version objects | REGISTRY METADATA | https://crates.io/api/v1/crates/tonic/0.13.1 | fetched live 2026-10-29 |
| S7 | prost 0.13.5 version object | REGISTRY METADATA | https://crates.io/api/v1/crates/prost/0.13.5 | fetched live 2026-10-29 |
| S8 | tokio 1.54.0 / 1.54.1 / 1.53.0 / 1.53.1 version objects | REGISTRY METADATA | https://crates.io/api/v1/crates/tokio/1.53.1 | fetched live 2026-10-29 |
| S9 | tonic 0.13.1 dependency list | REGISTRY METADATA | https://crates.io/api/v1/crates/tonic/0.13.1/dependencies | fetched live 2026-10-29 |
| S10 | tonic latest docs page (feature-flag surface) | REGISTRY METADATA (docs.rs) | https://docs.rs/tonic/latest/tonic/ | fetched live 2026-10-29 |
| S11 | ureq 2.12.1 version object (locked `tls` feature) | REGISTRY METADATA | https://crates.io/api/v1/crates/ureq/2.12.1 | fetched live 2026-10-29 |
| S12 | tokio latest docs page (platform/features) | REGISTRY METADATA (docs.rs) | https://docs.rs/tokio/latest/tokio/index.html | fetched live 2026-10-29 |

Non-official/community sources: NONE were relied on for any decision. S5–S12 are registry metadata used only as toolchain evidence.

## Third-Party Directive Adjudications

Baseline: the caller pasted third-party AI directives; the Task-1 record §3 adjudicated them claim by claim (verdict + one-line basis):

| # | Pasted directive | Verdict | Basis |
|---|---|---|---|
| C1 | "Reqwest+Tokio with `bytes_stream()` line-by-line reading against `GET .../liveChatMessages/streamList`" | UNDOCUMENTED + RULE-CONFLICT → REJECTED | No official wire format for a plain-HTTP streaming body exists (S1 web-traffic table documents statuses only; S3 carries no stream-body semantics; S2 documents ONLY the gRPC client); `reqwest` was already rejected in Phase 04 for pulling an async runtime through a different HTTP stack. |
| C2 | "async `tokio::sync::mpsc` queue" across the app seam | PARTIALLY CONFIRMED, SHAPE SUPERSEDED | An async runtime IS required by the official protocol, but the crossing boundary stays the existing bounded `sync_channel(64)` + `try_recv`; the async task forwards with stop-aware `try_send`. |
| C3 | "typed serde structs for `textMessageEvent`/`superChatEvent`/`userBannedEvent`/`tombstone`" | CONFIRMED (typed structs), MECHANISM + POLICY SUPERSEDED | The wire is protobuf, not JSON — prost-derived structs from the official proto are the correct form; only `textMessageEvent` maps to visible chat (brief §8.2 policy unchanged). |
| C4 | "'or API key query param'" | RULE-CONFLICT → REJECTED | The project rule is Bearer-only OAuth (brief §6.4); the API-key form is never sent and the token never appears in a URL. |
| C5 | "self-healing backoff loop keyed on `nextPageToken`" | CONFIRMED | Disconnects resume with the last `nextPageToken` (S1); the loop is bounded (cap 60 s, 10-consecutive budget, terminal classes), never unbounded. |

## Selected Path and Toolchain Pins

- Path A (official gRPC server-streaming client) DECIDED; Path B (plain-HTTP REST mirror) REJECTED (no official wire format); Path C moot.
- Pins (registry evidence, accessed 2026-10-29): `tonic` 0.13.1 (`rust_version` 1.75; 0.14.x needs 1.88), `prost` 0.13.5 (1.71.1), `tokio` 1.53.1 (1.71; 1.54.x needs 1.85 > the pinned image).
- Feature set: `tonic` client-only `channel/prost/codegen/tls-ring/tls-webpki-roots`; `tokio` `rt+time+macros`; TLS rustls `ring` provider + bundled `webpki-roots` (matches the locked ureq TLS family, including the Windows-GNU cross-build).
- Codegen: hand-authored prost message types transcribed from the officially published proto; NO `tonic-build`, NO `protoc`.
- Transitive `--precise` pins required by the probe: `idna_adapter` 1.2.0, `indexmap` 2.12.1, `hyper-util` 0.1.17, `zeroize` 1.8.1, `tokio-util` 0.7.19.
- Toolchain probes under the pinned `rust:1.82` image (temporary dependency edits, reverted): Probe 1 `cargo check` (Linux) exit 0; Probe 2 `cargo check --target x86_64-pc-windows-gnu` exit 0 (record §14).

## Quota Semantics and Documented Unknowns

Documented facts (S1/S3/S4):

- `streamList` costs **5 units per call**; `liveBroadcasts.list` (O6 chat-id resolution) costs **1 unit**; default project quota 10,000 units/day combined, reset at midnight Pacific Time; invalid requests still cost ≥ 1 unit.
- Re-establishment is a NEW call (5 units). Bounded worst case inside one sustained outage window: 10 consecutive-failure re-establishments + the initial attempt = **≤ 11 calls ≤ 55 units** before the typed transport error ends the run.

Documented UNKNOWNs (recorded, never invented):

- Whether pushes on an ESTABLISHED connection bill per item/per batch.
- Whether connection lifetime itself bills continuously.
- Any server-side keepalive/idle-drop timing.
- The appearance of `quotaExceeded`/`dailyLimitExceeded` on the gRPC path (code 8 `RESOURCE_EXHAUSTED` carries the documented rate-limit meaning for streamList and is classified into the existing transient arm).

The operator documentation records these unknowns honestly and makes NO quota-neutral promise beyond the documented call costs.

## Buffering, Backpressure, Reconnect, and Shutdown Decisions

- The bounded seam (P7-9): `CHAT_CHANNEL_CAPACITY = 64`, the `ChatSource` non-blocking `try_recv`, one message per frame iteration, stop-aware `try_send` (on `Full`: retain the message and retry once per 1 s slice; on `Disconnected`/stop: return) and the terminal cell — all byte-unchanged.
- Burst backpressure: a burst into a full channel is RETAINED (never silently dropped mid-session) and stalls the consumer side while HTTP/2 flow control stalls reads; no unbounded internal buffer; shutdown loss window = ≤ 64 queued-but-undelivered messages, never logged (log/video never diverge). Pinned by the new `worker_full_channel_retains_the_burst_without_loss_or_reorder`.
- Reconnect (P7-5/P7-7/P7-8): establish (Bearer metadata only) → consume via a select-biased stop tick (`CHAT_STOP_TICK_SLICE_MS = 1_000`) → map/dedup/`try_send` → on drop/clean close, bounded backoff `1 s × 2^(attempt−1)` capped 60 s with a 10-consecutive budget → re-establish with the retained token. The FIRST-established session is the BASELINE session (all items discarded; token retained); fresh token-bearing sessions deliver live. The 512-window dedup survives reconnects. An idle-but-alive stream is never treated as failed.
- Shutdown (P7-10): the Ctrl+C `StopFlag` is read by the select! tick; every async wait is stop-pollable ≤ 1 s EXCEPT the one-shot connect attempt (`CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000`); the gate joins the worker and the worker joins its runtime; gate-Fatal-outranks precedence, the single-`error:`-line contract and exit tables are untouched.

## Files Changed (per task)

- Task 1: research/design records only — `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (and the record commits), `.agent/project-info/context.md`; NO `src/` change, NO version change.
- Task 2 (`75788ae`, 16 files): `Cargo.toml` + `Cargo.lock` (`03e696f`); new `src/youtube/stream_proto.rs`, `stream_transport.rs`, `stream_client.rs`, `stream_concrete.rs`, `live_chat_stream_mapping.rs`, `live_error/grpc_status.rs`, `stream_test_fixtures.rs` and their rosters (12 new `src/youtube/` files in total); `live_error/mod.rs` + `live_error/display.rs` gained the two stream classes; `src/youtube/mod.rs` re-exports.
- Task 3 (`dc3312a`, 29 files, 3 deletions): new `src/app/run/chat_stream_worker.rs` + `chat_stream_delivery.rs` and `chat_failure_tests.rs` + `src/youtube/stream_run_fixtures.rs`; rewired `chat_worker.rs`, `mod.rs`, `seams.rs`, the worker rosters and `runtime_chat_fakes.rs`; the `LiveChatTransport` trait shrank to the O6 lookup; DELETED `src/app/run/chat_worker_delivery.rs`, `src/youtube/live_chat_models_tests.rs`, `src/youtube/live_chat_mapping_tests.rs`; the REST list models/mapper/roster surfaces retired.
- Task 4 (this pass): NEW `src/app/run/chat_worker_burst_tests.rs` + the 4-line `#[cfg(test)]` registration block in `src/app/run/chat_worker.rs`; docs `docs/production-run.md`, `docs/google-cloud-prerequisites.md`, `README.md`; metadata `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`; reports `.agent/reports/20261026-phase-06-youtube-live-chat-integration.md` (dated addendum) + this report.

## Test Coverage Audit

Method: every TODO bucket was verified against the live tree by grep/read before any test was added. Verdicts:

1. **Stream establishment + initial-page/baseline semantics — COVERED, no new test**: `worker_delivers_live_batches_in_api_order_with_metadata`, `worker_discards_the_entire_history_batch_before_the_first_token`, `worker_failed_connect_before_the_first_batch_reestablishes_tokenless_then_discards_history`, `worker_token_progresses_verbatim_without_inference` (flow roster); `worker_transient_failures_recover_within_the_budget` (budget roster); the proto request/token pins (`stream_proto_tests`); the token carry pin (`live_chat_stream_mapping_tests`); the Bearer-only client pins (`stream_client_tests`); `worker_event_constants_are_pinned_and_secret_free`, `worker_offline_ended_batch_disconnects_without_terminal_error` (worker roster).
2. **Reconnect + resume + dedup + bounded backoff / terminal-class mapping — COVERED, no new test**: `worker_dedup_drops_replayed_ids_across_sessions`, `worker_malformed_batch_is_discarded_and_never_ordinary_chat`, `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal`, `worker_backoff_delay_doubles_and_caps_at_the_maximum`, `worker_never_reestablishes_after_a_graceful_end`, the `ChatPrecondition` dual branch, `closed_transient_matrix_is_pinned` / `terminal_mapping_is_three_way_with_the_stream_classes`, and the 12-pin `live_error_grpc_status_tests` matrix.
3. **Push-event latency — COVERED**: `worker_delivery_after_a_stalled_read_adds_no_poll_interval` and `worker_stalled_stream_cannot_prevent_a_bounded_join`.
4. **Runtime-bridge confinement — COVERED (named pins + the compiler gate)**: stop-flip inside the connect window (`flip_stop_on_open_from`), `worker_stop_during_delivery_drops_pending_messages_without_blocking`, `worker_stop_flag_ends_the_loop_before_any_transport_open`, bounded join, `stream_seam_objects_stay_send_bounded`; the 10 s connect window is the documented single non-stop-pollable bound; greps: `tonic::async_trait` in `src/app` = 0, `tokio::runtime` = 2 files.
5. **Burst/backpressure at the bounded channel — GENUINE GAP → ONE new test**: `src/app/run/chat_worker_burst_tests.rs` (`worker_full_channel_retains_the_burst_without_loss_or_reorder`): a 64-message prefill + a 2-message live burst into the full channel, drained by the test's blocking `recv()` loop; asserts the prefill drains first in order, both burst items arrive, no terminal, and nothing extra is queued. Zero sleeps; the test-build `sleep_slice` is a no-op so the retain-retry spins until the reader frees capacity.
6. **Preservation pins — COVERED by the Task-3 zero-diff ledger + the gate**: `demo` schedule/vector pins, RTMPS argv pins, secret-redaction pins, exit-code matrices. Re-verified `git diff dc3312a^..HEAD -- src/app/demo src/streaming` → EMPTY.

Observed gate total: **494 passed / 0 failed** (`ALL CHECKS PASSED`), log `logs/checks/20261029T203729Z.log` (493 baseline + the 1 new burst test).

## Exact Commands and Exit Statuses

| # | Command | Exit | Observed result |
|---|---|---|---|
| 1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | 0 | four checks PASS; `test result: ok. 494 passed; 0 failed; 0 ignored`; `ALL CHECKS PASSED`; log `logs/checks/20261029T203729Z.log` |
| 2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` (FIRST — TODO-faithful order, see note) | 0 | `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (12336636 bytes).` |
| 3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` | 0 | `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (7665712 bytes).` |
| 4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` | 0 | Linux `7665712` bytes; Windows `12336636` bytes — both match the build-script-printed sizes |
| 5 | `docker compose … rust cargo test --locked chat_worker_burst` (before the gate) | 0 | `1 passed; 0 failed; 493 filtered out` |
| 6 | `docker compose … rust cargo fmt --check` (before the gate) | 0 | clean |

**Order anomaly note (recorded, not fixed):** TODO line 84's heading says "Rebuild Linux" but its first command is `sh scripts/build-windows.sh` and its second `sh scripts/build-linux.sh`. Per the binding plan §B.2, the TODO's exact wording was honored: BOTH builds ran, Windows first, then Linux. Both were built from the same tree, so the order does not change either binary's validity.

Hygiene/secret audit (host; observed values):

| Check | Observed |
|---|---|
| `git status --porcelain` (pre-commit) | only the task's intended paths |
| `git ls-files dist videos` | EMPTY (both) |
| `git check-ignore -v dist/rust-youtube-streamer-service` | `.gitignore:34:dist/` |
| `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` | `.gitignore:34:dist/` |
| `git check-ignore -v logs/chat.log` | `.gitignore:18:logs/*` |
| `git check-ignore -v videos/` | `.gitignore:56:videos/` |
| `git grep -nE "ya29\|AIza" -- src docs README.md .agent` | 15 pre-existing lines, all historical audit-prose mentions in `.agent` (unchanged count vs HEAD: 7 + 2 + 5 + 1); ZERO in `src`/`docs`/`README.md`; zero new |
| `git grep -n "key=" -- src docs README.md` | 5 pre-existing lines: 3 doc-comment "never an API key" mentions (`google_live_api_transport.rs`, `live_chat_models.rs`, `live_chat_wire_checks_tests.rs`) + 2 test negative assertions (`live_chat_wire_checks_tests.rs`, `stream_client_tests.rs`); zero new |
| `git grep -n "rtmps://" -- src docs` | only `.example.test` fixtures, doc-context mentions (the Ingest Probe procedure, `pipe_drain.rs` comment) and secret-absence assertions; zero new |
| `git grep -n "allow(" -- src` | exactly 8 `#[allow(dead_code)]` attribute lines + 1 `#[allow(clippy::manual_clamp)]` at `chat_stream_delivery.rs`; no comment-only hit observed; zero new |
| `git grep -n "tonic::async_trait" -- src/app` | ZERO |
| `git grep -n "tokio::runtime" -- src` | 2 files: `src/app/run/chat_worker.rs` (`chat_runtime()`), `src/youtube/stream_test_fixtures.rs` |
| Retired-name greps (`list_messages`, `chat_messages_path`, `ChatListCommand`, `map_page`, `ChatPageOutcome`, `polling_interval_millis`, `DEFAULT_CHAT_POLL_INTERVAL`, `Staged for the Task-3`) | ALL ZERO |
| `git grep -c "LiveChatMessageList" -- src` | 27 hits (the Task-2-KEEP proto-model substrings) |

## Build Results

- Linux `dist/rust-youtube-streamer-service`: **7,665,712 bytes** (observed `ls -l`, cross-checked against the build script's printed size).
- Windows `dist/windows/rust-youtube-streamer-service.exe`: **12,336,636 bytes** (observed `ls -l`, cross-checked against the build script's printed size; GNU runtime statically linked per the script's import audit).
- Caveats: builds are container-only facts; cross-compilation is NOT native Windows runtime validation; building validates no runtime chat behavior (the streaming handshake and push latency remain operator-verified only); no artifact is ever committed (`dist/` is gitignored).

## Manual Verification Procedure (operator-run)

The agent did NOT execute this procedure and makes NO live Google/Studio claim. The canonical steps live in `docs/production-run.md` ("Manual Verification Procedure (Operator-Run)") and now include: video before messages (the stream opens black; the first baseline session is discarded), separate-account text appearing PROMPTLY (push latency; next frame render after YouTube-side delivery), idle periods (no duplication/replay/reconnect churn), the NEW optional step 6b (network-interruption re-establishment check: video never stalls; post-restore messages appear without duplication; a prolonged outage may end the run with the bounded transport error), Ctrl+C (exit 0), and broadcast auto-completion. Operator results are recorded only by the operator — never as build evidence; no secrets are pasted anywhere.

Operator-confirmation pointer: the 2026-10-29 operator report (`logs/run/20261029-153416.log` + screenshot) confirms both live-render criteria ON THE POLLING BUILD v0.12.0 — recorded in `context.md` and as a dated addendum to the Phase-06 report. The STREAMING build's own live confirmation remains the operator E2E above.

## Checks Actually Performed vs NOT Performed

Performed (offline, container-only): the coverage audit over the landed rosters; the ONE new deterministic burst test; the full dev-checks gate (fmt/check/test/clippy) — 494/0; both release builds + `ls -l` cross-checks; the hygiene/secret audit; the docs/metadata edits and their drift-checks.

NOT performed: the manual unlisted-broadcast E2E (operator-run); any live Google call, stream handshake, or push-latency measurement on the streaming build (all tests use in-process scripted fakes; no test dials); native Windows/Linux runtime validation of the streaming path (the `.exe` is a cross-compilation fact); any quota observation against Google.

## Known Limitations

- The real TLS/gRPC handshake against Google is untested offline — it is validated ONLY by the operator E2E; the offline tests pin the proto wire bytes, the Bearer metadata, the status classification and the resumable loop.
- Baseline-session loss window: the worker's FIRST-established session is discarded whole; because the API documents no boundary between pushed history and pushed live traffic, messages posted during that window are lost. The window's length is the session's own lifetime and is not bounded by anything the API documents.
- Quota/keepalive semantics on an established connection are officially undocumented (recorded as unknowns, never invented).
- Re-establishment is event-driven only: a sparse-but-alive stream is never redirected (the official keepalive/idle-drop behavior is undocumented).
- `LIVE_CHAT_DISABLED` vs `LIVE_CHAT_ENDED` is not distinguishable by gRPC code (official footnote); the establishment-fatal / mid-session-graceful split is the deliberate typed decision.
- Up to 64 queued-but-undelivered messages can be lost at shutdown (carried over; never logged).
- The operator-confirmed latency/rendering evidence is on the POLLING build; streaming-build live confirmation is pending the operator E2E.

## AC Closing Matrix

AC numbering = TODO lines 91–103. The 4.6 step owns the checkbox flips; this matrix records the evidence verdict per criterion.

| AC | Criterion (abridged) | Verdict | Evidence pointer |
|---|---|---|---|
| AC-1 | Production `run` receives chat via the official `streamList` mechanism; latency no longer paced by `pollingIntervalMillis` | SATISFIED at mechanism level offline; live confirmation annotation: "(wired into the production path; live confirmation awaits the operator manual E2E)" | Task-3 switch + the never-periodic pins + the no-added-interval pin; report §8 bucket 3 |
| AC-2 | Connection established once, re-established only on drops with the last token; quota semantics recorded as officially stated | SATISFIED (TODO already `[x]`) | Task-1 §11 + report §5 |
| AC-3 | Toolchain officially implementable on the pinned image + Windows-GNU cross-build | SATISFIED (TODO `[x]`) | Task-1 §14 probes (both exit 0) |
| AC-4 | Text-bearing messages map to `ChatMessage` with ID/timestamp/normalization; non-text/malformed never ordinary chat | SATISFIED (TODO `[x]`) | mapping rosters + worker rosters |
| AC-5 | Flow through `MessageSource` → `ChatStore` → renderer → pipeline; frame loop never blocks; demo byte-stable | SATISFIED (TODO `[x]`) | zero-diff demo/streaming ledger + gate |
| AC-6 | Reconnects/backoff/buffers/cancellation/shutdown bounded; no history replay; dedup across reconnects; class verdicts kept | SATISFIED (TODO `[x]`) | budget/flow/failure rosters |
| AC-7 | Async runtime confined AND documented | FLIP `[x]` — the tech.md confinement bullet + the `tokio::runtime`/`tonic::async_trait` greps close the documented conjunct | `tech.md` §Lifecycle bullet; report §9 audit rows |
| AC-8 | YouTube models/seam stay in `src/youtube/`; no API types in renderer/ChatStore/streaming | SATISFIED (TODO `[x]`) | module boundaries unchanged |
| AC-9 | Failures typed/actionable/secret-safe and reach the run error policy; nothing logs secrets | FLIP `[x]` — delivery path Task-3-pinned; operator-facing failure/recovery documentation landed | `docs/production-run.md` Diagnostics + Failure Recovery; report §9 |
| AC-10 | Offline tests pass without real Google access; loopback/fake tooling test-only | FLIP `[x]` after the fresh gate | gate 494/0; report §8; no concretes dial in tests |
| AC-11 | Docker dev checks pass and both artifacts are rebuilt with recorded sizes | FLIP `[x]` after §B | gate log + this report §10 |
| AC-12 | Docs/structure/context/report accurate incl. adjudications + the operator note | FLIP `[x]` after §C | this report, `project-structure.md`, `tech.md`, `context.md`, Phase-06 addendum |
| AC-13 | No credentials/tokens/headers/keys/RTMPS URLs/videos/logs/artifacts committed | FLIP `[x]` after §E | report §9 audit table; `git ls-files dist videos` empty |

## Evidence Index

- TODO: `.agent/todos/20261029/20261029-todo-1.md`
- Global plan: `.kilo/plans/20261029-phase07-streamlist-migration.md`
- Task-1 record: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
- Task-2 plan + adherence: `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md`, `.kilo/plans/20261029-phase07-task2-adherence.md`
- Task-3 plan + adherence: `.kilo/plans/20261029-phase07-task3-integration-switch.md`, `.kilo/plans/20261029-phase07-task3-adherence.md`
- Task-4 plan: `.kilo/plans/20261029-phase07-task4-verification-docs.md`
- Gate log: `logs/checks/20261029T203729Z.log` (gitignored)
- Operator evidence (never agent-observed): `logs/run/20261029-153416.log` (Phase-06 polling-build confirmation)
- Repo greps: `git grep`/`git ls-files` values as listed in report §9.
