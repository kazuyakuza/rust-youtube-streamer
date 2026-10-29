# Phase 07 Task 2 — Streaming Client and Async Bridge Implementation (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — **Task 2 "Streaming Client and Async Bridge Implementation" only** (bullets transcribed §H).
> Binding design record: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (decisions P7-1..P7-12, §1.6 field table, §5–§10 bridge/loop semantics, §12 retirement ruling). Per its §16, this plan TRANSCRIBES those decisions and adds Task-2's own closed-form choices; it re-decides nothing already ruled.
> Global plan: `.kilo/plans/20261029-phase07-streamlist-migration.md` (Task-2 entry = "New `src/youtube/` modules … Bearer-only; secret-safety identical; bridge in `src/app/run/` only if needed").
> Baseline: branch `feat/phase07-streamlist-retrieval` @ `3cc119f` (Task 1 closed, `[DONE]`), package v0.13.0, working tree clean, test baseline **458 passed / 0 failed**. Branch/step ownership: branch creation + version bump are DONE; `git push` stays workflow step 5.
> This plan was produced by the Task-2 4.1b architector pass (2026-10-29). It commits nothing; the 4.2 implementer commits and executes its steps (§K), writing observed results into the §L addendum.
> Container-only verification: every cargo command runs on the Alpine VM as `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust …`. The Windows host has NO rust toolchain.
> S2 official page re-verified verbatim by this pass on 2026-10-29: the full `stream_list.proto` inline page (https://developers.google.com/youtube/data/live/streaming-live-chat, "Last updated 2026-10-15 UTC") — §C transcribes from the page text (the `part` field is `repeated string part = 100`; §A.6). Task type: Rust backend module (`src/youtube/`), NOT front-end ⇒ workflow 4.1b/4.5b only.

---

## 0. Scope of this task (TODO Task-2 bullets, verbatim obligations → owning section)

| TODO Task-2 bullet | Where decided |
|---|---|
| Add the required YouTube-specific models + the mockable stream seam under `src/youtube/`, per typed-error/secret-safety/module-boundary conventions; the only cross-boundary outputs stay `ChatMessage`-carriers via the existing conversion point plus the typed error contract | §C (files), §A.3 (carrier boundary), §A.4 (typed errors) |
| Implement the selected stream retrieval operation: Bearer-only (never an API key); correct request constants (chat id, part set, resume token); response/event decoding per documented framing; typed decoding failures for malformed payloads (never silently treated as ordinary chat) | §C.2 (bearer metadata + endpoint + path), §C.1 (proto models + golden pins), §A.5 (classification), §E.3 |
| Implement the resumable streaming loop exactly per the Task-1 design (establish → consume → drop/timeout/failure → bounded backoff → re-establish with last `nextPageToken`) | `open()`/`next_batch()` seam in Task 2 (§C.2/C.4); the LOOP itself is the confined runtime thread — Task 3's worker owns the loop body; §C.6 records the seam contract Task 3 consumes (P7-5/P7-7/P7-8 unchanged) |
| Keep the worker-to-frame-loop bridge bounded/cancellable per Task-1 design | Task 3 (worker rewrite). Task 2 lands ONLY the youtube-side modules + seam + fakes (§I). The bridge design is rehearsed in §A.7 so Task 3 transcribes it |
| Reuse (not fork) the existing mapping layer (textMessageEvent → carrier, author fallback, provider id, RFC3339→epoch) and the 512 dedup across reconnects; skip-policy unchanged and tested; never log payloads | §C.5 (the proto-side mapper REUSES `ChatTextMessage`, `parse_rfc3339_seconds`, `non_blank_text` — no fork), §E.2 |
| If research rules `list_messages` retired, remove it plus its now-unused surfaces with the plan's exact retirement steps | RETIRED — but the removal is Task 3 (§D: zero deletions in Task 2) |

Explicitly NOT in Task 2 (owned later): worker rewire/loop (Task 3), retirement deletions (Task 3), docs/structure-map/tech.md updates + builds + report + operator E2E (Task 4). §I ledger.

---

## §A — Decisions transcribed QUOTE-STYLE from the Task-1 record (the implementer treats THIS section + §C/§D as the one self-contained reference; each item cites the record)

### A.1 Client path (record P7-1, §4)

- **PATH A — official gRPC server-streaming client.** Grounding quoted in the record: "Path B requires an officially documented REST-mirror wire format, which does not exist (§1.2 verbatim)". `grpcio` stays rejected (Phase-06 reasons + build risk). tonic+prost+tokio are the maintained pure-Rust family (brief §3.3 sanctions the evaluation).
- Path B stays REJECTED — no HTTP-streaming parser may be written ("Prohibited: hand-computing wire bytes, hand-implemented HTTP/2 or framing, or any 'fallback' plain-HTTP parser" — record §4).
- STOP conditions (i)/(ii): already adjudicated — the Task-1 probe (record §14) PASSED both targets (Linux host `cargo check` exit 0; `--target x86_64-pc-windows-gnu` exit 0). If Task 2's own container gate fails on a NEW transitive crate, §B.3's ≤6-pin discipline governs; STOP to the caller only beyond it.

### A.2 Dependencies + feature lines (record P7-2) — exact transcription

- `tonic` **0.13.1** (`rust_version` 1.75; newest 0.14.x needs 1.88 — never enters the lock), `prost` **0.13.5** (1.71.1), `tokio` **1.53.1** (1.71; 1.54.x needs 1.85 and MUST be pinned down — record: "cargo's default resolution on rust 1.82 is not MSRV-aware, so fresh `cargo update` would pull 1.54.x and fail the gate").
- The `Cargo.toml` carries CARET minimums; the PRECISE versions live in the tracked lockfile (tempfile `--precise` precedent). Exact lines in §B.2.
- Feature rationale (record, verbatim sense): `channel` = client `Channel`/`Endpoint` only; `prost` = tonic↔prost codec; `codegen` = the `tonic::async_trait` re-export used by the seam traits (the ONLY codegen-surface API used); `tls-ring` + `tls-webpki-roots` = rustls provider + bundled roots (A.3); tokio needs `rt` (current-thread scheduler), `time` (stop ticker + connect timeout), `macros` (`select!` — Task 3).
- TLS (record P7-3): **rustls via the `ring` provider + `webpki-roots` trust store; native-roots and `aws-lc` REJECTED**. aws-lc-rs is a C build on `x86_64-pc-windows-gnu` — rejected; `tls-native-roots` unnecessary (fixed `*.googleapis.com` target, webpki covers it). In-tree evidence: locked `ureq 2.12.1` already compiles the same rustls/ring/webpki-roots family for Windows-GNU (every Phase-05/06 cross-build succeeded). Final proof = the Task-1 §14 probe (PASS) — repeated as Task-2's check-only sanity (§F.7).
- Codegen (record P7-4): **NO tonic-build; NO protoc; NO checked-in generated code; hand-authored prost message types transcribed from the published proto** + tonic's documented generic client. Grounding: `tonic-build` needs `protoc` at compile time (the pinned image does NOT ship it; the Dockerfile may not change); generated package files collide with the 200-line cap; the projected surface is small (one request, ~6 response-path messages + the snippet type enum). "brief §3.3 … still holds: tonic+prost implement the ENTIRE protocol; the structs are configuration of maintained tools, not a protocol implementation."
- Accuracy safeguards (record, binding): the §1.6 field table transcribed verbatim and PINNED by golden-bytes tests; the gRPC path constant pinned by test; the operator manual E2E (Task 4) is the ONLY real-Google validation.
- **A.2 refinement recorded by this plan (observed 2026-10-29):** the full proto is on the S2 page (re-verified "Last updated 2026-10-15 UTC"); the transcription below (§C.1/appendix) copies the published field TYPES verbatim (`repeated string part = 100`; `optional uint32 profile_image_size = 3`; `PageInfo` = `optional int32 total_results = 1; optional int32 results_per_page = 2`; `active_poll_item` = `optional LiveChatMessage active_poll_item = 1008`). No ambiguity remains.

### A.3 TLS + connection establishment (record P7-3 + P7-5 step 1 + §10)

- Endpoint: `https://youtube.googleapis.com` (mapped from S2's demo `grpc.secure_channel("dns:///youtube.googleapis.com:443", creds)`); pinned by test as `STREAM_ENDPOINT_URL`.
- Bearer metadata ONLY: header **`authorization`** with value **`Bearer <access-token>`** — the API-key form (`x-goog-api-key`) is never sent (record §1.1/§3-C4; brief §6.4).
- Connect is bounded: record P7-5 — "Connect is bounded by `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000` (a `tokio::time::timeout` around the connect future; connect failure = one failure against the budget)". Constant lands in Task 2 (§C.2).
- Runtime confinement (record §10, for Task 3's consumption — Task 2 lands nothing of it except the seam that fits it): "ONE runtime instance … `tokio::runtime::Builder::new_current_thread().enable_all().build()` constructed ONCE on the dedicated chat worker's runtime thread … cross-thread outputs are ONLY: `try_send(values)` into the existing channel + the terminal cell + the `StopFlag` read."

### A.4 Typed errors (record P7-11) — the mutation set

- The funnel stays: `LiveApiError` → `ChatRetrievalError` → `ChatSourceError::Retrieval` → `RunError::ChatRetrieval` (variants and Display lines UNCHANGED).
- Variant count 17 → **19**: NEW `ChatPrecondition { detail: String }` + `InvalidChatRequest { detail: String }`. "Decode failure reuses the existing `MalformedResponse`, so no separate deserialize variant is added."
- Mapping table (record §7 — the full table reproduced in §A.5 with the one documented interpretation refinement at the end of this subsection).
- "`is_transient_failure` STAYS CLOSED (the entire transient list unchanged — no arm is added); the establishment/mid-session branching for `ChatPrecondition` lives in the strategy-loop policy (a pure match, pinned by test), NOT through the shared classifier." — the branching itself is Task 3; the two variants + their classification land in Task 2.
- **Task-2 interpretation pin for the `INTERNAL` code:** a batch/frame decode failure under tonic surfaces as a gRPC status with code `INTERNAL`; the two §7 rows (`INTERNAL`-class vs "item batch DECODE failure → `MalformedResponse`") are INDISTINGUISHABLE at the code level (the status message is never read — §8). Decision: `Code::Internal` maps to **`LiveApiError::MalformedResponse { field: "stream response" }`** — an existing, transient (CLOSED-list) class, so the §7 "batch discarded; NEVER treated as ordinary chat; token state stays at last-delivered" worker policy is keyed on a typed class Task 3 can match. Honest deviation recorded: a mid-stream server `<INTERNAL>` also lands there; the §7 "Network/BackendUnavailable per shape" row is realized by `CANCELLED`/`DEADLINE_EXCEEDED`/`UNKNOWN` → `Network` and `UNAVAILABLE` → `BackendUnavailable`. Both classes are inside `is_transient_failure`, so behavior (bounded backoff re-establishment) is identical; only the redacted Display wording differs. Pinned by test (§E.3).

### A.5 The §7 mapping table (record verbatim, extended with the fixed Task-2 reason constants)

| tonic/gRPC received (`tonic::Code`) | Mapped `LiveApiError` | App verdict / worker policy (Task 3) |
|---|---|---|
| `PermissionDenied (7)` | existing `InsufficientPermissions` | terminal → `ChatUnavailable` |
| `InvalidArgument (3)` | NEW `InvalidChatRequest { detail: "invalid argument" }` | terminal (never retried) |
| `FailedPrecondition (9)` | NEW `ChatPrecondition { detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED" }` | establishment-time (0 batches): terminal → `ChatUnavailable`; mid-session (≥1 batch): graceful end (Task 3 policy) |
| `NotFound (5)` | existing `ApiNotFound { resource: "live chat" }` (the existing `LIVE_CHAT_RESOURCE` constant — reused, not forked) | terminal → `ChatUnavailable` |
| `ResourceExhausted (8)` | existing `QuotaLike { reason: "rateLimitExceeded" }` (the loader-level same literal matches the CLOSED `is_transient_failure` rate-limit arm — chat_failure.rs keeps its own private local duplicate, the D-T2-6 precedent) | transient → bounded backoff re-establishment |
| `Unauthenticated (16)` | existing `AuthorizationRejected` | terminal |
| `Unavailable (14)` | existing `BackendUnavailable { reason: "unavailable" }` | transient |
| `Cancelled (1)`, `DeadlineExceeded (4)`, `Unknown (2)` | existing `Network { reason: "gRPC code <n>" }` (numbered reason — `format!("gRPC code {}", i32::from(code))`) | transient |
| `Internal (13)` | existing `MalformedResponse { field: "stream response" }` (per A.4 refinement) | transient |
| any other code (`Aborted`, `AlreadyExists`, `UNCHARTED…`, `Ok`, `OutOfRange`, `Unimplemented`) | `Network { reason: "gRPC code <n>" }` (catch-all; Closed classifier) | transient |
| clean connection close (`Streaming::message()` → `Ok(None)`) | seam returns **`None`** (see A.6); the worker's loop maps clean-close to the transient Network-class branch per §7 | transient re-establishment (budgeted) |
| batch decode failure (surfaces as `Err(status)`, code `Internal`) | `MalformedResponse { field: "stream response" }` | transient; batch discarded; token state stays at last-delivered |

Secret-safety (record §8, binding): "The status MESSAGE is NOT logged verbatim: the mapping `detail` for the new `ChatPrecondition` carries ONLY the documented `LIVE_CHAT_DISABLED`/`LIVE_CHAT_ENDED` reason labels (official tokens from S1); arbitrary server status text is logged as the redacted classification shape". Conclusion encoded in Task 2: `classify_stream_status` reads ONLY `status.code()` — NEVER `status.message()`; the two new variants and the Network/BackendUnavailable reasons carry fixed constants (or the code NUMBER — "the gRPC status code number (Google's own code identity)" is the sanctioned non-secret diagnostic). Establishment connect failures map transport errors to FIXED reason strings (no error text embedded). Pinned by the never-embed test (§E.3).

### A.6 Seam (record P7-12) — trait shapes, names FIXED

- `stream_proto.rs` — prost message structs from the §1.6 table + request/service-path constants + the snippet type enum; `stream_proto_tests.rs` — "golden-bytes encode/decode pins (each field number asserted by encoding a populated struct and matching the pinned byte sequence) + tolerance pin (unknown fields/values decode-and-skip)".
- `stream_client.rs` — "the concrete tonic adapter: endpoint constant `https://youtube.googleapis.com` (S2's `dns:///youtube.googleapis.com:443` mapping; pinned by test), a pure Bearer-metadata builder … unit-pinned offline, session opening with the tokio timeout, and the `LiveChatStreamSession` adapter wrapping `tonic::Streaming<LiveChatMessageListResponse>`".
- Mock seam: "`pub(crate) trait ChatStreamTransport: Send` with one async method `open(&mut self, request: ChatStreamRequest) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>` where `LiveChatStreamSession: Send` exposes `async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>>`. Fakes implement both with scripted `VecDeque` sequences (wire-status → typed-error classification pins re-use the same shapes the worker consumes). This replaces ureq-loopback wire testing for the stream side ONLY — documented limitation: the tonic `Channel`/hyper/rustls handshake itself stays untested offline (covered by the operator manual E2E; recorded in Task-4 docs)."
- **Task-2 closure of the Option (so Task 3 never guesses):** the session's `next_batch` returns `Some(...)` for every delivered state — batches (`Ok`) and classified failures (`Ok(Err)`) — and returns `None` EXACTLY for the clean server close (`tonic::Streaming::message()` → `Ok(None)`). Per the §7 table, clean close is the transient Network-class session end; the worker loop (Task 3) maps `None` to the bounded re-establishment branch with the last `nextPageToken`. This gives the `Option` real semantics and keeps the Record's clean-close row as a LOOP policy rather than an impossible adapter error (the clean close carries no status to classify).
- Async-mechanism choice (Task-2 closure within P7-2's sanctioned surface): the two seam traits use the tonic re-export `tonic::async_trait` (verified against tonic 0.13.1 source: `#[cfg(feature = "codegen")] pub use async_trait::async_trait;` — docs.rs fetched 2026-10-29). This introduces NO extra direct dependency (the `async-trait` crate stays transitive), and the desugared `Pin<Box<dyn Future + Send>>` methods remain dyn-safe for `Box<dyn ChatStreamTransport>` / `Box<dyn LiveChatStreamSession>` with the `Send` bounds P7-12 requests.
- Implementation-layer names fixed by Task 2 (the record's `LiveChatStreamSession` is BOTH referenced as the trait object name — the concrete adapter therefore cannot share that EXACT name; recorded, reviewer-facing): trait `LiveChatStreamSession` + concrete session adapter `TonicChatStreamSession` in `stream_client.rs`; concrete transport `GoogleLiveChatStreamTransport` in `stream_concrete.rs` (google_-prefix type naming follows the `google_live_chat_transport.rs` precedent even though the file name is caller-specified `stream_concrete.rs`).

### A.7 Loop semantics rehearsed for Task 3 (NOT landed in Task 2 — transcribed so the seam consumer is fully specified; record §5–§6, P7-5..P7-10, §10)

- Strategy loop: `establishment(token: Option<String>)` → `consume` (advance `next_page_token` whenever a batch delivers one; items per the mapping policy; `offline_at` presence = graceful end) → `on session end`: graceful-end classes exit cleanly; transient classes count a failure behind `CHAT_SLEEP_SLICE_MS`-based backoff (1 s × 2^(attempt−1), cap 60 s, budget `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`, base 1 s since the streaming call has no server-directed interval) then re-establish WITH the last token; terminal classes write the terminal cell and exit. Establishment and re-establishment are the ONLY re-calls — NEVER a periodic schedule.
- First-established session = BASELINE: EVERY delivered item discarded (never queued/logged); its `nextPageToken` retained (P7-8). From the second session on: live delivery + per-item dedup (`ChatDedupWindow`, 512, never reset across reconnects; skip-policy FIRST — record: "skip-before-dedup ordering keeps tombstone/gift reuse safe").
- Channel: the EXISTING `sync_channel(64)` + `try_send`-only stop-aware delivery + the `Arc<Mutex<Option<ChatRetrievalError>>>` terminal cell + the `StopFlag` read — byte-UNCHANGED (P7-9; pasted async-mpsc directive superseded per §3/C2 of the record). Stop ticker `CHAT_STOP_TICK_SLICE_MS = 1_000` (P7-10); every async wait stop-pollable ≤ 1 s except the ≤ 10 s connect (documented).
- Constants ledger (P7-6): consuming constants all unchanged (`CHAT_CHANNEL_CAPACITY 64`, `CHAT_DEDUP_WINDOW 512`, `CHAT_SLEEP_SLICE_MS 1_000`, `CHAT_BACKOFF_MAX_MS 60_000`, `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES 10`, resolution 6×5 s); NEW stream-family constants land with Task 2: `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000` and `CHAT_STOP_TICK_SLICE_MS = 1_000` (the stop-ticker constant is consumed only by Task 3's loop; it lands in Task 2 as a module-private constant so Task 3 does not invent one, with an explicit "reserved" note — see §C.2 note).

### A.8 Quota semantics (record §11 — unchanged posture; nothing new for Task 2 code)

5 units per call; re-establishment = new call; documented-cost ceiling ≤ 55 units per bounded outage window; per-push/connection-lifetime billing UNKNOWN (never filled in). No code consumes this beyond the existing constants; Task 4 documents it.

### A.9 The §1.6 field table (record §1.6 — REPRODUCED COMPLETE; now type-resolved against the re-verified S2 proto)

Request `LiveChatMessageListRequest`: `live_chat_id=1` (string), `hl=2` (string, never sent), `profile_image_size=3` (uint32, never sent), `max_results=98` (uint32, "Not used in the streaming RPC." — never sent), `page_token=99` (string), `part=100` (**repeated string** — verified on S2 2026-10-29).
Response `LiveChatMessageListResponse`: `kind=200` (string), `etag=201` (string), `offline_at=2` (string), `page_info=1004` (`PageInfo{total_results=1 int32, results_per_page=2 int32}`), `next_page_token=100602` (string), `items=1007` (repeated `LiveChatMessage`), `active_poll_item=1008` (optional `LiveChatMessage`).
`LiveChatMessage`: `kind=200` (string), `etag=201` (string), `id=101` (string), `snippet=2`, `author_details=3`.
`LiveChatMessageSnippet`: `type=1` (nested enum), `live_chat_id=201` (string), `author_channel_id=301` (string), `published_at=4` (string), `has_display_content=17` (bool), `display_message=16` (string), oneof `displayed_content` = { `text_message_details=19`, `user_banned_details=22`, `super_chat_details=27`, `super_sticker_details=28`, `new_sponsor_details=29`, `member_milestone_chat_details=30`, `membership_gifting_details=31`, `gift_membership_received_details=32`, `poll_details=33`, `gift_details=34` }.
Snippet type enum: `INVALID_TYPE=0, TEXT_MESSAGE_EVENT=1, TOMBSTONE=2, FAN_FUNDING_EVENT=3, CHAT_ENDED_EVENT=4, SPONSOR_ONLY_MODE_STARTED_EVENT=5, SPONSOR_ONLY_MODE_ENDED_EVENT=6, NEW_SPONSOR_EVENT=7, MEMBER_MILESTONE_CHAT_EVENT=17, MEMBERSHIP_GIFTING_EVENT=18, GIFT_MEMBERSHIP_RECEIVED_EVENT=19, USER_BANNED_EVENT=10, SUPER_CHAT_EVENT=15, SUPER_STICKER_EVENT=16, POLL_EVENT=20, GIFT_EVENT=21`.
`LiveChatMessageAuthorDetails`: `channel_id=10101`, `channel_url=102`, `display_name=103`, `profile_image_url=104`, `is_verified=4`, `is_chat_owner=5`, `is_chat_sponsor=6`, `is_chat_moderator=7` (all per the published proto; the odd `10101` number is verbatim on S2).
`LiveChatTextMessageDetails`: `message_text=1`.
Plus the S2 giftEvent-ID-reuse note (re-use on gift events) — handled by skip-before-dedup (record §1.6 last line; D-4 ordering unchanged).

Transcription SCOPE (record P7-4 rationale 2, verbatim sense): "Unknown fields, non-text `displayed_content` arms (22/27/28/29/30/31/32/33/34), and unknown enum values are TOLERATED-AND-SKIPPED by prost — defined but unread fields are not needed for correctness, only the type value and text path". ⇒ the oneof is declared with ONLY the text arm (tag 19); arms 22–34 stay undeclared (they decode as unknown fields and are skipped — verified prost behavior, and the wire keys for all nine arms pre-computed in the appendix for the tolerance pins). Everything the table names IS declared (including the unread `hl`/`profile_image_size`/`max_results`/`kind`/`etag`/`display_message`/`page_info`/`active_poll_item` — read by the prost encode impl, so no dead-code findings). `active_poll_item` is declarable at zero cost because its type is the already-transcribed `LiveChatMessage`.

---

## §B — Dependency changes (exact edits + exact commands)

### B.1 State precondition (first step)

`git status --porcelain` → EMPTY; `git branch --show-current` → `feat/phase07-streamlist-retrieval`; `git log --oneline -3` ends at the Task-1 close (`3cc119f … adherence report`). Any other state ⇒ STOP and ask the caller.

### B.2 `Cargo.toml` edit (the ONLY `Cargo.toml` changes in this task: these three lines; NO other line touched; NO `google-youtube3` ever)

Append exactly after the `ureq = "2"` line (alphabetical placement is irrelevant; the caret line texts are binding):

```toml
tokio = { version = "1", default-features = false, features = ["rt", "time", "macros"] }
tonic = { version = "0.13", default-features = false, features = ["channel", "prost", "codegen", "tls-ring", "tls-webpki-roots"] }
prost = { version = "0.13", default-features = false, features = ["derive", "std"] }
```

(record §P7-2 lines verbatim; tokio/prost/tonic each pinned by the tracked lockfile to 1.53.1 / 0.13.5 / 0.13.1 through B.3.)

### B.3 Lockfile procedure (in-container; the pinned `rust:1.82` image; the observed Task-1 §14 expected set)

Run in order; LE container aborts or a check fails ⇒ follow the ≤6-pin discipline at the end.

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update -p tokio --precise 1.53.1
```

REQUIRED (record §14: the fresh resolver pulls tokio 1.54.x ⇒ MSRV 1.85). Then verify the observed triad in-container BEFORE fixing transitive pins:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "grep -A1 'name = \"tokio\"' Cargo.lock | head -2; grep -A1 'name = \"tonic\"' Cargo.lock | head -2; grep -A1 'name = \"prost\"' Cargo.lock | head -2"
```

Expect: tokio 1.53.1, tonic 0.13.1, prost 0.13.5. If tonic/prost drifted above 0.13.1/0.13.5 (a patch may have appeared after 2026-10-29), run `cargo update -p tonic --precise 0.13.1` / `-p prost --precise 0.13.5` respectively (count each toward the pin budget).

THE FIVE OBSERVED TRANSITIVE PINS (Task-1 §14 — a fresh 2026-10-29 `cargo update` resolved these ABOVE their rust-1.82-compatible releases): `idna_adapter 1.2.0`, `indexmap 2.12.1`, `hyper-util 0.1.17`, `zeroize 1.8.1`, `tokio-util 0.7.19`. The implementer MUST re-verify whether the fresh resolution still needs them (crates advance daily): for each, grep `Cargo.lock` in-container; pin ONLY if the resolved version differs from the expected one AND any subsequent check fails with an `edition2024`/rust-version diagnostic naming it:

- failure diagnostics name them in the classic order (§14): after `cargo update`, run check first; on `feature 'edition2024' is required` or `requires rustc > 1.82` naming crate X at version V, look up X's newest release with declared `rust_version` ≤ 1.82 (crates.io API, access-date it) and run `cargo update -p X --precise <that-version>`; re-run check; repeat.
- The expected map (apply WITHOUT the failure hop if the fresh `cargo update` already re-bumped them — identical end state either way): `idna_adapter=1.2.0`, `indexmap=2.12.1`, `hyper-util=0.1.17`, `zeroize=1.8.1`, `tokio-util=0.7.19`.
- Budget: ≤ 6 pin iterations total (tokio included) — beyond it STOP, record the evidence + options (§13.4(j) style) and ask the caller. Note: Task-1's probe consumed 5 by the failure-driven path; the failure sequence may differ today; the end-state (any compiling resolution under 1.82) is the goal, not the path.

Then the two sanity checks (compile-only, NO artifacts):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --target x86_64-pc-windows-gnu
```

Both exit 0 expected (§14 verdict). Commit the dependency step:

```
git add Cargo.toml Cargo.lock
git commit -m "feat: add the phase07 streamList dependency set and its tracked pins"
```

`Cargo.lock` diff expectation: version/feature entries from the pins ONLY (see §G). Fast compile sanity BEFORE the code commit is covered by the checks above and again in §F.

---

## §C — File plan under `src/youtube/` (exact names; ≤200 lines per file; sibling `*_tests.rs` wired via the established pattern)

Conventions enforced (existing precedent): every new source file ≤ 200 lines (count excludes nothing — the cap is the fmt'd file); every test sibling is registered from its OWNER file via `#[cfg(test)] #[path = "<sibling>.rs"] mod <sibling>;` (the `live_error_tests.rs` precedent; roster files live at `src/youtube/<name>.rs`); functions ≤ 50 body lines; ≤ 2 params (fixtures/fakes: param objects or 1–2-arg builders only); nesting ≤ 2; no commented-out code; no secret shapes in fixtures (`*-fixture` suffix strings only); **no test contacts Google; no async test may sleep** (zero-sleep rule). Structure map + tech.md updates are Task 4's (NOT this task).

New source files (7): `stream_proto.rs`, `stream_client.rs`, `stream_transport.rs`, `stream_concrete.rs`, `live_chat_stream_mapping.rs`, `live_error/grpc_status.rs`, `stream_test_fixtures.rs` (`#[cfg(test)]`).
Modified source files (3): `src/youtube/mod.rs` (module decls + re-exports + a doc-header trim preserving the cap), `src/youtube/live_error/mod.rs` (+2 variants), `src/youtube/live_error/display.rs` (+2 arms + 2 display helpers). `classify.rs` UNTOUCHED (is_transient/`classify_api_error` stay closed — A.4).
New test siblings (5): `stream_proto_tests.rs`, `live_chat_stream_mapping_tests.rs`, `live_error_grpc_status_tests.rs`, `stream_client_tests.rs`, `stream_transport_tests.rs`.

### C.1 `src/youtube/stream_proto.rs` (+ sibling `stream_proto_tests.rs`)

Owns: the prost message structs + the snippet type enum + the request assembly + the part constant + optional-field tolerance notes. Estimated ~165 lines. Exact content (the implementer transcribes mechanically; field ORDER inside each struct = ascending field number, so encode order matches the anchors):

```rust
//! The published `stream_list.proto` wire types (Task-1 record §A.9/S2,
//! access-dated): hand-authored prost `Message` structs — build-time codegen
//! is rejected (record P7-4: protoc is unavailable on the pinned image;
//! generated files exceed the 200-line cap). Declared-but-unread fields are
//! transcribed exactly as published; unread non-text oneof arms (22–34) and
//! unknown fields/enum values are tolerated-and-skipped by prost — never an
//! error, never ordinary chat. Golden bytes: the sibling roster pins every
//! field number by encoding populated structs against the pre-computed byte
//! sequences in the plan appendix.
//!
//! Streaming tests stay offline: no gRPC dialing anywhere in tests.

pub(crate) const CHAT_STREAM_PARTS: [&str; 3] = ["id", "snippet", "authorDetails"];

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageListRequest {
    #[prost(string, optional, tag = "1")]
    pub(crate) live_chat_id: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub(crate) hl: Option<String>,            // never sent (record P7 wire-form)
    #[prost(uint32, optional, tag = "3")]
    pub(crate) profile_image_size: Option<u32>, // never sent
    #[prost(uint32, optional, tag = "98")]
    pub(crate) max_results: Option<u32>,       // "Not used in the streaming RPC."
    #[prost(string, optional, tag = "99")]
    pub(crate) page_token: Option<String>,
    #[prost(string, repeated, tag = "100")]
    pub(crate) part: Vec<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageListResponse {
    #[prost(string, optional, tag = "200")]
    pub(crate) kind: Option<String>,           // declared-but-unread
    #[prost(string, optional, tag = "201")]
    pub(crate) etag: Option<String>,           // declared-but-unread
    #[prost(string, optional, tag = "2")]
    pub(crate) offline_at: Option<String>,
    #[prost(message, optional, tag = "1004")]
    pub(crate) page_info: Option<PageInfo>,    // declared-but-unread
    #[prost(string, optional, tag = "100602")]
    pub(crate) next_page_token: Option<String>,
    #[prost(message, repeated, tag = "1007")]
    pub(crate) items: Vec<LiveChatMessage>,
    #[prost(message, optional, tag = "1008")]
    pub(crate) active_poll_item: Option<LiveChatMessage>, // declared-but-unread
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessage {
    #[prost(string, optional, tag = "200")]
    pub(crate) kind: Option<String>,           // declared-but-unread
    #[prost(string, optional, tag = "201")]
    pub(crate) etag: Option<String>,           // declared-but-unread
    #[prost(string, optional, tag = "101")]
    pub(crate) id: Option<String>,
    #[prost(message, optional, tag = "2")]
    pub(crate) snippet: Option<LiveChatMessageSnippet>,
    #[prost(message, optional, tag = "3")]
    pub(crate) author_details: Option<LiveChatMessageAuthorDetails>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageSnippet {
    #[prost(enumeration = "SnippetType", optional, tag = "1")]
    pub(crate) message_type: Option<i32>,
    #[prost(string, optional, tag = "201")]
    pub(crate) live_chat_id: Option<String>,   // declared-but-unread
    #[prost(string, optional, tag = "301")]
    pub(crate) author_channel_id: Option<String>,
    #[prost(string, optional, tag = "4")]
    pub(crate) published_at: Option<String>,
    #[prost(bool, optional, tag = "17")]
    pub(crate) has_display_content: Option<bool>, // declared-but-unread
    #[prost(string, optional, tag = "16")]
    pub(crate) display_message: Option<String>,   // declared-but-unread; NEVER promoted (record §C.5 note)
    #[prost(oneof = "DisplayedContent", tags = "19")]
    pub(crate) displayed_content: Option<DisplayedContent>,
}

#[derive(Clone, PartialEq, prost::Oneof)]
pub(crate) enum DisplayedContent {
    #[prost(message, tag = "19")]
    TextMessage(LiveChatTextMessageDetails),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub(crate) enum SnippetType {
    InvalidType = 0,
    TextMessageEvent = 1,
    Tombstone = 2,
    FanFundingEvent = 3,
    ChatEndedEvent = 4,
    SponsorOnlyModeStartedEvent = 5,
    SponsorOnlyModeEndedEvent = 6,
    NewSponsorEvent = 7,
    MemberMilestoneChatEvent = 17,
    MembershipGiftingEvent = 18,
    GiftMembershipReceivedEvent = 19,
    UserBannedEvent = 10,
    SuperChatEvent = 15,
    SuperStickerEvent = 16,
    PollEvent = 20,
    GiftEvent = 21,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageAuthorDetails {
    #[prost(string, optional, tag = "10101")]
    pub(crate) channel_id: Option<String>,
    #[prost(string, optional, tag = "102")]
    pub(crate) channel_url: Option<String>,      // declared-but-unread
    #[prost(string, optional, tag = "103")]
    pub(crate) display_name: Option<String>,
    #[prost(string, optional, tag = "104")]
    pub(crate) profile_image_url: Option<String>, // declared-but-unread
    #[prost(bool, optional, tag = "4")]
    pub(crate) is_verified: Option<bool>,         // declared-but-unread
    #[prost(bool, optional, tag = "5")]
    pub(crate) is_chat_owner: Option<bool>,       // declared-but-unread
    #[prost(bool, optional, tag = "6")]
    pub(crate) is_chat_sponsor: Option<bool>,     // declared-but-unread
    #[prost(bool, optional, tag = "7")]
    pub(crate) is_chat_moderator: Option<bool>,   // declared-but-unread
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatTextMessageDetails {
    #[prost(string, optional, tag = "1")]
    pub(crate) message_text: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct PageInfo {
    #[prost(int32, optional, tag = "1")]
    pub(crate) total_results: Option<i32>,     // declared-but-unread
    #[prost(int32, optional, tag = "2")]
    pub(crate) results_per_page: Option<i32>,  // declared-but-unread
}

/// The wire request assembly (P7-5 step 1): the chat id ALWAYS; the resume
/// token ONLY when one exists (never fabricated); the documented part set;
/// never `hl`/`profileImageSize`/`maxResults` (record §1.1/P7 wire-form).
pub(crate) fn stream_list_request(command: &ChatStreamRequest) -> LiveChatMessageListRequest {
    LiveChatMessageListRequest {
        live_chat_id: Some(command.live_chat_id.clone()),
        hl: None,
        profile_image_size: None,
        max_results: None,
        page_token: command.page_token.clone(),
        part: CHAT_STREAM_PARTS.iter().map(|part| part.to_string()).collect(),
    }
}
```

Imports: `use super::stream_transport::ChatStreamRequest;`. If rustc rejects a prost derive form (e.g., the `optional` token spelling differs on prost 0.13), the ONLY permitted correction is the minimal derive-spelling fix — field numbers/scripts unchanged; the golden pins will catch any drift of the FIELD MAPPING (they are the real lock). If the cap (200) fires because of the derive expansions, the pre-authorized contingency: move `PageInfo` + the tolerance-notes doc lines out to a second sibling roster file — NO: contingency for THIS file only = trim the header doc lines (be terse); any deeper split ⇒ STOP and ask.

Sibling roster `src/youtube/stream_proto_tests.rs` (wired from this file): 14 tests; imports `use prost::Message;` and the appendix pinned arrays; struct values built via `Default::default()` + field assignments (prost derive emits `Default` for its message structs; if the compiler rejects it on any struct, fall back to full struct literals — every field named once in one shared `fn default_response()` style helper is FORBIDDEN for golden arrays: build the EXACT values from the appendix). Names:

1. `stream_list_request_encodes_the_documented_wire_form` (anchor `A`, no token)
2. `stream_list_request_with_resume_token_appends_field_99` (anchor `B`)
3. `unset_request_encodes_zero_bytes` (`Default::default()` → empty slice)
4. `text_message_details_encode_pin` (anchor `D`)
5. `snippet_text_message_oneof_encode_pin` (anchor `E`)
6. `author_details_display_name_and_owner_flag_encode_pin` (anchor `C`)
7. `live_chat_message_encode_pin` (anchor `F`)
8. `response_encode_pin` (anchor `G`)
9. `page_info_encode_pin` (anchor `H`)
10. `response_decodes_the_pinned_wire_bytes` (decode(`G`) == the struct)
11. `unknown_response_fields_are_tolerated_and_skipped` (append `T1` to `G`; decode succeeds, fields equal)
12. `unknown_snippet_oneof_arm_is_tolerated_and_skipped` (build snippet variant with `T2` appended after the type varint; decode succeeds, `displayed_content` = None, `message_type` = Some(2))
13. `unknown_snippet_type_value_decodes_without_failure` (type varint `U1` = raw 99; decode succeeds, mapper-treatable value is NOT `TextMessageEvent`)
14. `full_response_round_trips` (encode(decode(`G`)) == `G`)

Golden-byte pin mechanics (CRITICAL): the implementer writes every pinned array exactly as in the appendix table; after the FIRST successful compile, run the roster in-container; if ANY pin fails, re-derive that array from `field_number << 3 | wire_type` + the record §A.9 numbers, fix the PLAN-SIDE array or the transcription accordingly, and record BOTH the mismatch and the resolved value in the §L addendum marked OBSERVED. Never loosen an assertion to plain round-trips.

### C.2 `src/youtube/stream_client.rs` (+ sibling `stream_client_tests.rs`)

Owns: the endpoint constant, the service path constant, the pure Bearer-metadata builder, the one-arg request wrapper, the offline-instantiable endpoint builder, the concrete tonic session adapter, and the connect-timeout constant. Estimated ~130 lines. Exact content skeleton:

```rust
//! The concrete tonic adapter for the streamList gRPC call (record P7-12):
//! endpoint + service-path constants pinned by test; the Bearer metadata is
//! the ONLY authentication form (never an API key — record §3-C4); the
//! endpoint construction is offline-instantiable (pinned without dialing);
//! the REAL `open()` path dials Google only in production runs (the operator
//! manual E2E tests it; the offline rosters stop at the seam).
//!
//! tonic surfaces verified 2026-10-29 (docs.rs/tonic/0.13.1): `Grpc::
//! server_streaming(request, path, codec) -> Result<Response<Streaming<M2>>,
//! Status>`; `Streaming::message() -> Result<Option<T>, Status>`; `Endpoint::
//! from_shared/tls_config/connect_timeout/connect`; `ClientTlsConfig::
//! with_enabled_roots`; the codegen feature re-exports `tonic::async_trait`
//! and `tonic::codegen::http`.

pub(crate) const STREAM_ENDPOINT_URL: &str = "https://youtube.googleapis.com";
pub(crate) const STREAM_LIST_PATH: &str =
    "/youtube.api.v3.V3DataLiveChatMessageService/StreamList";
pub(crate) const AUTHORIZATION_METADATA_KEY: &str = "authorization";
/// Fixed stream family failure reasons (secret-safe: no server text).
const STREAM_CONNECT_TIMEOUT_REASON: &str = "the stream endpoint connect attempt timed out";
const STREAM_CONNECT_FAILURE_REASON: &str = "the stream endpoint connection failed";
const STREAM_ENDPOINT_BUILD_REASON: &str = "the stream endpoint could not be built";
```

Functions (all closed-form):

1. `pub(crate) fn bearer_token_metadata(access_token: &AccessToken) -> Result<tonic::metadata::MetadataValue<tonic::metadata::Ascii>, LiveApiError>` — body: `tonic::metadata::MetadataValue::try_from(format!("Bearer {}", access_token.secret())).map_err(|_| LiveApiError::AuthorizationRejected)` (token.rs's `secret()` is the sanctioned request-build accessor). Linked into `tonic::Request` via (2).
2. `pub(crate) fn stream_request(request_proto: LiveChatMessageListRequest, bearer: MetadataValue<Ascii>) -> tonic::Request<LiveChatMessageListRequest>` — body: `let mut request = tonic::Request::new(request_proto);` then `request.metadata_mut().insert(AUTHORIZATION_METADATA_KEY, bearer);` then `request`. (2 args ✓.)
3. `pub(super) fn stream_endpoint() -> Result<tonic::transport::Endpoint, LiveApiError>` — EXACT body:

```rust
pub(super) fn stream_endpoint() -> Result<tonic::transport::Endpoint, LiveApiError> {
    let endpoint = tonic::transport::Endpoint::from_shared(STREAM_ENDPOINT_URL.to_string())
        .map_err(|_| endpoint_build_failure())?;
    let endpoint = endpoint
        .connect_timeout(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS));
    endpoint
        .tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .map_err(|_| endpoint_build_failure())
}

fn endpoint_build_failure() -> LiveApiError {
    LiveApiError::Network { reason: STREAM_ENDPOINT_BUILD_REASON.to_string() }
}
```

(`with_enabled_roots()` resolves to webpki-roots under the feature set — verified: the 0.13.1 cfg gates are `#[cfg(feature = "tls-webpki-roots")]`, and its native sibling stays off.) The `connect_timeout` constant:

```rust
/// P7-6 stream family connect bound (Task 1 record §A.3): 10 s.
pub(super) const CHAT_STREAM_CONNECT_TIMEOUT_MS: u64 = 10_000;
```

4. `pub(crate) struct TonicChatStreamSession { inner: tonic::Streaming<LiveChatMessageListResponse> }` + `pub(crate) fn new(inner: tonic::Streaming<LiveChatMessageListResponse>) -> Self`, implementing the seam trait (definitions live in `stream_transport`):

```rust
#[tonic::async_trait]
impl LiveChatStreamSession for TonicChatStreamSession {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>> {
        match self.inner.message().await {
            Ok(Some(response)) => Some(Ok(map_stream_batch(&response))),
            Err(status) => Some(Err(classify_stream_status(&status))),
            Ok(None) => None, // clean server close = the seam's None (record §A.6)
        }
    }
}
```

Imports: `use std::time::Duration;`, `use std::sync::mpsc?` none, `use super::live_error::{classify_stream_status? via live_error re-export}, Super::stream_proto reprs, use super::stream_transport::{LiveChatStreamSession, YieldedChatStreamBatch}`, `use super::token::AccessToken`, `use super::live_chat_stream_mapping::map_stream_batch;`.

Sibling roster `stream_client_tests.rs` (6 tests — all offline, no dialing):
1. `bearer_token_metadata_is_bearer_only` — `bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))` → Ok; `value.to_str()` equals `Ok("Bearer access-token-fixture")` (the fixture constant lives in `stream_test_fixtures.rs`).
2. `bearer_token_metadata_holds_the_authorization_key_not_an_api_key` — `AUTHORIZATION_METADATA_KEY == "authorization"`; the metadata VALUE starts with `"Bearer "`; and the assertion that NO constant in this module spells `x-goog-api-key` is expressed structurally: assert `!auth_metadata.to_str().unwrap().contains("key=")` plus the key-constant pin (the two together pin Bearer-only with no API-key form).
3. `non_ascii_access_token_fails_as_authorization_rejected` — `AccessToken::new("non-ascii-token-Θ")?` → Err(AuthorizationRejected) (offline-demonstrable failure arm).
4. `stream_endpoint_url_is_the_official_google_host` — `STREAM_ENDPOINT_URL == "https://youtube.googleapis.com"` and `Stream_LIST_PATH == "/youtube.api.v3.V3DataLiveChatMessageService/StreamList"` plus `tonic::codegen::http::uri::PathAndQuery::from_static(STREAM_LIST_PATH)` constructible without panic. (Name it `stream_wire_constants_are_the_official_host_and_path` and split into ONE test with 3 asserts — 1 test.)
5. `stream_endpoint_builds_offline_with_connect_timeout_and_tls_shape` — `stream_endpoint()` → Ok; `endpoint.uri().to_string() == STREAM_ENDPOINT_URL`; `endpoint.get_connect_timeout() == Some(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS))` — pure construction, no IO.
6. `stream_request_carries_the_bearer_metadata_only` — build via (1)+(2) over a default request proto and assert `request.metadata().get("authorization").unwrap().to_str()` startswith "Bearer " and `request.metadata().len() == 1` (no other metadata exists — no API-key header can be smuggled).

So final names (5 tests): `bearer_token_metadata_is_bearer_only`, `bearer_token_metadata_holds_the_authorization_key_not_an_api_key`, `non_ascii_access_token_fails_as_authorization_rejected`, `stream_wire_constants_are_the_official_host_and_path`, `stream_endpoint_builds_offline_with_connect_timeout_and_tls_shape`, `stream_request_carries_the_bearer_metadata_only` → 6 names; count = 6.

### C.3 `src/youtube/stream_transport.rs`

The mockable seam (P7-12) + the two carrier types. Estimated ~75 lines. Exact content:

```rust
//! The mockable stream seam (record P7-12, names fixed): one async open and
//! one async batch tick, `Send`-bounded for the confined runtime thread
//! (Task 3 owns the thread; the seam never kisses std::thread). The real
//! implementer is `stream_concrete`; the scripted fakes live in the
//! `#[cfg(test)] stream_test_fixtures` sibling. The Option on `next_batch`
//! is EXACTLY the clean server close (record §A.6) — the worker loop maps
//! it to the bounded transient re-establishment branch.

use tonic::async_trait;

use super::live_error::LiveApiError;
use super::live_chat_mapping::ChatTextMessage;

/// The establishment request (record §A.6): the chat id + the resume token
/// when any; the part set is the module constant in `stream_proto`.
#[derive(Clone, Debug)]
pub(crate) struct ChatStreamRequest {
    pub(crate) live_chat_id: String,
    pub(crate) page_token: Option<String>,
}

/// One delivered batch (record §A.6/P7-7): the mapped text carriers in API
/// order + the resume key + the graceful `offlineAt`/`chatEndedEvent`
/// marker. The mapping produced these through the REUSED policy layer.
#[derive(Debug)]
pub(crate) struct YieldedChatStreamBatch {
    pub(crate) messages: Vec<ChatTextMessage>,
    pub(crate) next_page_token: Option<String>,
    pub(crate) offline_ended: bool,
}

#[tonic::async_trait]
pub(crate) trait ChatStreamTransport: Send {
    /// Opens (or re-opens) the stream; ONE quota-billed call per invocation
    /// (record P7-5: never called periodically — only establishment and
    /// re-establishment).
    async fn open(&mut self, request: ChatStreamRequest)
        -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>;
}

#[tonic::async_trait]
pub(crate) trait LiveChatStreamSession: Send {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>>;
}
```

Import of `tonic::async_trait`: `use tonic::async_trait;`.

### C.4 `src/youtube/stream_concrete.rs`

The ONE production implementer (called `GoogleLiveChatStreamTransport`; file name per the caller-specified plan). Estimated ~95 lines. Exact content skeleton:

```rust
//! The concrete `ChatStreamTransport` over the tonic adapter (record
//! P7-12/A.3): a FRESH `Endpoint` per open (the recorded channel-reuse
//! alternative rejected — each open is a new quota-billed call and a shared
//! lazy channel would void the bounded 10 s establishment), the tokio
//! timeout around the connect future, the generic-client server-streaming
//! call, and per-status classification. Errors are exactly the §A.5 table.
//! The handshake stays untested offline; the operator manual E2E is the
//! only real-Google validation (record P7-12 limitation).

use tonic::codegen::http::uri::PathAndQuery;

use super::live_error::{classify_stream_status? path, LiveApiError};
use super::stream_client::{bearer_token_metadata, stream_endpoint, stream_request, STREAM_LIST_PATH, STREAM_CONNECT_TIMEOUT_REASON, STREAM_CONNECT_FAILURE_REASON};
use super::stream_proto::{stream_list_request, LiveChatMessageListResponse, ... no, request type};
use super::stream_transport::{ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession};
use super::token::AccessToken;

pub(crate) struct GoogleLiveChatStreamTransport {
    access_token: AccessToken,
}

impl GoogleLiveChatStreamTransport {
    pub(crate) fn new(access_token: AccessToken) -> Self {
        Self { access_token }
    }
}

fn stream_connect_timeout_error() -> LiveApiError {
    LiveApiError::Network { reason: STREAM_CONNECT_TIMEOUT_REASON.to_string() }
}

fn stream_connect_failure_error() -> LiveApiError {
    LiveApiError::Network { reason: STREAM_CONNECT_FAILURE_REASON.to_string() }
}

#[tonic::async_trait]
impl ChatStreamTransport for GoogleLiveChatStreamTransport {
    async fn open(&mut self, command: ChatStreamRequest)
        -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>
    {
        let endpoint = stream_endpoint()?;
        let bearer = bearer_token_metadata(&self.access_token)?;
        let channel = tokio::time::timeout(
            Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
            endpoint.connect(),
        )
        .await
        .map_err(|_| stream_connect_timeout_error())?
        .map_err(|_| stream_connect_failure_error())?;
        let mut grpc = tonic::client::Grpc::new(channel);
        let response = grpc
            .server_streaming(
                stream_request(stream_list_request(&command), bearer),
                PathAndQuery::from_static(STREAM_LIST_PATH),
                tonic::codec::ProstCodec::default(),
            )
            .await
            .map_err(|status| classify_stream_status(&status))?;
        Ok(Box::new(TonicChatStreamSession::new(response.into_inner())))
    }
}
```

Imports consolidated: `use std::time::Duration;`, the filtered constants from `stream_client` (`CHAT_STREAM_CONNECT_TIMEOUT_MS` is `pub(super)` — visible to the sibling ✓), `super::live_error::classify_stream_status` (re-exported via live_error/mod.rs), `super::stream_proto::{stream_list_request}` + ` super::stream_client::{bearer_token_metadata, stream_endpoint, stream_request, STREAM_LIST_PATH}` + `TonicChatStreamSession`.

(Names/compilability are the 4.2 implementer's mechanical fix-ups — path spelling `tonic::client::Grpc`, `tonic::codec::ProstCodec`, `PathAndQuery` via `tonic::codegen::http` all verified against 0.13.1 sources fetched 2026-10-29.)

Tests: NO tests in this roster (the concrete path dials only in production); the structural one-implementer claim is verified by the hygiene grep in §F.8.

### C.5 `src/youtube/live_chat_stream_mapping.rs` (+ sibling `live_chat_stream_mapping_tests.rs`)

The proto-side page mapper (record §12 survivor: "a NEW proto-side page mapper in the `live_chat_mapping` family operating on the proto types with the SAME rules — Task 2's mapping work"). REUSE, not fork (TODO bullet): the carrier type `ChatTextMessage`, the RFC3339 parser `parse_rfc3339_seconds`, and the blank-text predicate `non_blank_text` are consumed from the existing `live_chat_mapping` module (its `non_blank_text` is widened THIS task from `fn` to `pub(super) fn` — a one-token change; its own tests unaffected). Estimated ~120 lines. Exact content:

```rust
//! The proto-side page mapping of Phase 07 (record §12; the D-4 policy over
//! the prost types): for every delivered item, in API order — the snippet
//! type must be TEXT (the resumable enum value), the text arm must carry a
//! non-blank `messageText`, the author is `displayName` else `channelId`
//! else skip; the carrier keeps the verbatim id and the RFC3339→epoch
//! second via the REUSED sibling parser. A `chatEndedEvent` item or a
//! present `offline_at` marks the batch as gracefully ended. Provider
//! payloads are never logged; `display_message` is NEVER promoted to text
//! (the text arm is the only sanctioned text source for textMessageEvent).

use super::live_chat_mapping::{non_blank_text, parse_rfc3339_seconds, ChatTextMessage};
use super::stream_proto::{
    DisplayedContent, LiveChatMessage, LiveChatMessageListResponse, LiveChatMessageSnippet,
    SnippetType,
};
use super::stream_transport::YieldedChatStreamBatch;

pub(crate) fn map_stream_batch(response: &LiveChatMessageListResponse) -> YieldedChatStreamBatch {
    YieldedChatStreamBatch {
        messages: response.items.iter().filter_map(map_item).collect(),
        next_page_token: response.next_page_token.clone(),
        offline_ended: batch_ended(response),
    }
}

/// The D-4 accept/skip policy in order; `None` = skipped (never ordinary
/// chat); API order preserved.
fn map_item(item: &LiveChatMessage) -> Option<ChatTextMessage> {
    let snippet = item.snippet.as_ref()?;
    if !is_text(snippet) {
        return None;
    }
    let text = message_text(snippet)?;
    let author = author_name(item)?;
    Some(ChatTextMessage {
        author,
        text,
        message_id: item.id.clone(),
        published_epoch: snippet.published_at.as_deref().and_then(parse_rfc3339_seconds),
    })
}

fn is_text(snippet: &LiveChatMessageSnippet) -> bool {
    snippet.message_type == Some(SnippetType::TextMessageEvent as i32)
}

/// Text comes ONLY from the oneof text arm; blank/missing text skips.
fn message_text(snippet: &LiveChatMessageSnippet) -> Option<String> {
    snippet
        .displayed_content
        .as_ref()
        .and_then(|content| match content {
            DisplayedContent::TextMessage(details) => non_blank_text(&details.message_text),
        })
}

/// `displayName` else `channelId` else skip (no stable id) — the D-4 rule.
fn author_name(item: &LiveChatMessage) -> Option<String> {
    let author = item.author_details.as_ref()?;
    non_blank_text(&author.display_name).or_else(|| non_blank_text(&author.channel_id))
}

fn batch_ended(response: &LiveChatMessageListResponse) -> bool {
    response.offline_at.is_some()
        || response.items.iter().any(is_chat_ended_item)
}

fn is_chat_ended_item(item: &LiveChatMessage) -> bool {
    item.snippet.as_ref().is_some_and(|snippet| {
        snippet.message_type == Some(SnippetType::ChatEndedEvent as i32)
    })
}
```

(`is_some_and` is available on 1.82 ✓.) The skipped-after-chat-ended rule: the ending marker item itself is skipped (it is not an ordinary message) — handled because its type is not TEXT.

Sibling roster `live_chat_stream_mapping_tests.rs` (12 tests; items built from struct recipes mirroring `live_chat_mapping_tests.rs` style; use the fixtures builders from C.7): the exact names listed in §E.2.

### C.6 `src/youtube/live_error/grpc_status.rs` (+ sibling `live_error_grpc_status_tests.rs`)

Pure status mapper (§A.4/§A.5) + constants. Estimated ~95 lines. Exact content:

```rust
//! The pure gRPC-status → typed-LiveApiError mapping for the streaming call
//! (Task-1 record §A.4/§A.5): status-FIRST (only `status.code()` is read —
//! NEVER the server message, record §8); existing `LiveApiError` classes
//! are reused; the two NEW classes carry fixed documented detail labels.
//! Establishment/mid-session branching for `ChatPrecondition` is a worker
//! policy (Task 3), not this classifier (record: `is_transient_failure`
//! stays CLOSED).

use tonic::Code;

use super::classify::LIVE_CHAT_RESOURCE;
use super::LiveApiError;

/// The officially documented DISABLED/ENDED ambiguity (the S1 footnote),
/// carried verbatim as the ONLY detail text (record §8).
pub(crate) const CHAT_PRECONDITION_DETAIL: &str = "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED";
/// The official web-traffic rate-limit reason (record §1.3): the SAME loader
/// literal that keeps the CLOSED transient matcher in `chat_failure`.
pub(crate) const STREAM_RATE_LIMIT_REASON: &str = "rateLimitExceeded";
pub(crate) const INVALID_CHAT_ARGUMENT_DETAIL: &str = "invalid argument";
pub(crate) const STREAM_UNAVAILABLE_REASON: &str = "unavailable";
pub(crate) const STREAM_INTERNAL_FIELD: &str = "stream response";
pub(crate) const STREAM_CODE_PREFIX: &str = "gRPC code";

pub(crate) fn classify_stream_status(status: &tonic::Status) -> LiveApiError {
    match status.code() {
        Code::PermissionDenied => LiveApiError::InsufficientPermissions,
        Code::InvalidArgument => LiveApiError::InvalidChatRequest {
            detail: INVALID_CHAT_ARGUMENT_DETAIL.to_string(),
        },
        Code::FailedPrecondition => LiveApiError::ChatPrecondition {
            detail: CHAT_PRECONDITION_DETAIL.to_string(),
        },
        Code::NotFound => LiveApiError::ApiNotFound {
            resource: LIVE_CHAT_RESOURCE.to_string(),
        },
        Code::ResourceExhausted => LiveApiError::QuotaLike {
            reason: STREAM_RATE_LIMIT_REASON.to_string(),
        },
        Code::Unauthenticated => LiveApiError::AuthorizationRejected,
        Code::Unavailable => LiveApiError::BackendUnavailable {
            reason: STREAM_UNAVAILABLE_REASON.to_string(),
        },
        Code::Internal => LiveApiError::MalformedResponse {
            field: STREAM_INTERNAL_FIELD.to_string(),
        },
        _ => LiveApiError::Network { reason: stream_code_reason(status.code()) },
    }
}

/// The sanitized class shape for any remaining code: the sanctioned gRPC
/// code NUMBER only (record §8 — "the gRPC status code number").
fn stream_code_reason(code: Code) -> String {
    format!("{STREAM_CODE_PREFIX} {}", i32::from(code))
}
```

Sibling roster wired from `grpc_status.rs` as `#[cfg(test)] #[path = "../live_error_grpc_status_tests.rs"] mod live_error_grpc_status_tests;` (the `live_error_tests.rs` path precedent). 12 tests (§E.3).

`src/youtube/live_error/mod.rs` edits: add `mod grpc_status;` + `pub(crate) use grpc_status::classify_stream_status;` + register the sibling: extending the existing tests-footer — no wait, the sibling is registered from grpc_status.rs, not mod.rs. Add the 2 variants directly after `ChatEnded`:

```rust
    /// gRPC FAILED_PRECONDITION (record §A.4): the chat is DISABLED or
    /// ENDED — the officially documented code-level ambiguity carried in
    /// `detail`; the establishment vs mid-session worker branch (Task 3)
    /// decides fatal vs graceful.
    ChatPrecondition {
        detail: String,
    },
    /// gRPC INVALID_ARGUMENT (record §A.4): a request-shape failure —
    /// terminal (never retried; brief §12.2 posture).
    InvalidChatRequest {
        detail: String,
    },
```

Display arms (exact strings — pinned by §E.3 tests):

```rust
fn display_chat_precondition(formatter: &mut fmt::Formatter<'_>, detail: &str) -> fmt::Result {
    write!(formatter, "live chat precondition failed (reason '{detail}')")
}

fn display_invalid_chat_request(formatter: &mut fmt::Formatter<'_>, detail: &str) -> fmt::Result {
    write!(formatter, "invalid stream chat request parameters (reason '{detail}')")
}
```

With the match arms `Self::ChatPrecondition { detail } => Self::display_chat_precondition(formatter, detail),` / `Self::InvalidChatRequest { detail } => Self::display_invalid_chat_request(formatter, detail),` inserted after the `ChatEnded` arm + its helper. Full rendered lines (assert targets):
- `live chat precondition failed (reason 'LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED')`
- `invalid stream chat request parameters (reason 'invalid argument')`

### C.7 `src/youtube/stream_test_fixtures.rs` (registered `#[cfg(test)] mod stream_test_fixtures;` in `src/youtube/mod.rs`)

Shared offline doubles for the Task-2 rosters AND the later Task-3 worker rosters. Estimated ~170 lines. Exact content decisions (all ≤2-arg builders; zero sleeps; `*-fixture` values only):

```rust
//! Shared offline stream fixtures (the `live_chat_test_fixtures` precedent,
//! record D-14 style): the scripted ChatStream-transport fake with per-
//! session `VecDeque` scripting, the response/carrier builders, and the
//! tiny current-thread test runtime helper (the TODO's sanctioned
//! test-constructed runtime — zero sleeps; fakes return immediately).
//! Never built in production; fixture strings are `*-fixture` shaped only;
//! no test reaches Google.

pub(crate) const ACCESS_TOKEN_FIXTURE: &str = "access-token-fixture";
pub(crate) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
pub(crate) const RESUME_TOKEN_FIXTURE: &str = "token-fixture";

pub(crate) fn stream_test_block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the stream test runtime builds")
        .block_on(future)
}
```

The scripted fake (`Send` to satisfy the trait superbound; interior mutability via `RefCell` is NOT Send-safe for real threads — the FAKE is used only on the test's current-thread runtime, so it must satisfy the STATIC bounds: `ScriptedChatStreamTransport` holds `RefCell`s ⇒ NOT Send ⇒ the supertrait `Send` breaks. RESOLUTION (closed-form): the fake holds its queues behind `Mutex` instead of `RefCell` (`std::sync::Mutex`), matching `runtime_chat_fakes.rs`'s simpler Queue pattern? (live_chat fixtures use RefCell because LiveChatTransport takes `&self` synchronously without Send). For the STREAM seam (trait: Send), the fake MUST be Send ⇒ use `Mutex<VecDeque<…>>`. Exact shape:

```rust
use std::sync::Mutex;
use std::sync::mpsc? no.

pub(crate) struct ScriptedChatStreamTransport {
    pub(crate) sessions: Mutex<VecDeque<Result<ScriptedSessionSpec, LiveApiError>>>,
    pub(crate) operations: Mutex<Vec<&'static str>>,
    pub(crate) captured_requests: Mutex<Vec<ChatStreamRequest>>,
}

/// One scripted session: a queue of delivered states; popping past the end
/// = the clean close `None`.
pub(crate) struct ScriptedSessionSpec(VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>);

impl ScriptedChatStreamTransport {
    pub(crate) fn new() -> Self { ... empty ... }
    pub(crate) fn enqueue_session(&self, spec: Result<ScriptedSessionSpec, LiveApiError>) { ... }
    pub(crate) fn invokes(&self) -> Vec<&'static str> { ... drained copy ... }
    pub(crate) fn captured(&self) -> Vec<ChatStreamRequest> { ... }
}

#[tonic::async_trait]
impl ChatStreamTransport for ScriptedChatStreamTransport {
    async fn open(&mut self, request: ChatStreamRequest)
        -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>
    {
        self.captured_requests.lock().expect("...").push(request);
        self.operations.lock().expect("...").push("open");
        match next from sessions ... {
            Ok(spec) => Ok(Box::new(ScriptedChatStreamSession { deliveries: Mutex::new(spec.0) })),
            Err(error) => Err(error),
        }
    }
}

pub(crate) struct ScriptedChatStreamSession {
    deliveries: Mutex<VecDeque<Option<Result<YieldedChatStreamBatch, LiveApiError>>>>,
}

#[tonic::async_trait]
impl LiveChatStreamSession for ScriptedChatStreamSession {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>> {
        self.deliveries.lock().expect("...").pop_front()
        // empty queue => None = the clean close, deterministic
    }
}
```

(`expect` in `cfg(test)` fixtures follows the live_chat fixture precedent.) `open(&mut self)` — for the scripted fake the queue/state lives in Mutexes so `&mut self` needs no interior alias.

Response builders (proto struct recipes; ≤2 args each):

```rust
pub(crate) fn text_item(id: &str) -> LiveChatMessage          // type=TEXT, published_at=FIXTURE rfc3339, text "hello", author BOTH display+channel
pub(crate) fn tombstone_item(id: &str) -> LiveChatMessage     // type=TOMBSTONE, no text arm
pub(crate) fn ended_item() -> LiveChatMessage                 // type=CHAT_ENDED_EVENT
pub(crate) fn response_plain(items: Vec<LiveChatMessage>) -> LiveChatMessageListResponse   // token None, offline None
pub(crate) fn response_with_token(items: Vec<LiveChatMessage>, next_page_token: &str) -> LiveChatMessageListResponse
pub(crate) fn response_offline(items: Vec<LiveChatMessage>) -> LiveChatMessageListResponse // offline_at = FIXTURE_RFC3339, token None
pub(crate) fn yielded(response: &LiveChatMessageListResponse) -> YieldedChatStreamBatch { map_stream_batch(response) }
```

Shared fixture constants: `PUBLISHED_AT_FIXTURE: &str = "2026-10-29T10:00:00Z"` (parses by the reused parser to a KNOWN epoch; the mapping roster asserts the exact epoch value computed by hand: 2026-10-29T10:00:00Z epoch seconds — the implementer computes it in-test via the EXISTING parser result pinned in the sibling `live_chat_mapping_tests` (it already pins parser outputs); simplest: assert `message.published_epoch == expected` with the hand-computed value `1_792_632_400`?? NO hand-math: assert the value EQUALS the parser output recorded for the same literals in `live_chat_mapping_tests.rs` (the parser rosters pin those exact strings; the implementer LOOKS the value up in the existing parser test constants and uses THAT literal — no new invented math). Encode: reuse the same RFC3339 fixture literal the existing mapping tests already pin (grep `live_chat_mapping_tests.rs` for the pinned epoch literal and use the SAME string+expected pair).

Default-item recipe details (for reproducible tests): `text_item` sets `author_details.display_name = Some("author-fixture")`, `channel_id = Some("channel-id-fixture")` (mapping roster's fallback test then UNSETs display_name), text = Some("text-fixture"), published_at = Some(PUBLISHED_AT_FIXTURE), id = Some(id).

### C.8 `src/youtube/mod.rs` wiring (exact edits; file ≤ 200 lines — requires the doc trim below)

1. Module declarations added in the existing alphabetical position:
```rust
mod live_chat_stream_mapping;
mod stream_client;
mod stream_concrete;
mod stream_proto;
mod stream_test_fixtures;   // #[cfg(test)] attribute; as live_test_fixtures
mod stream_transport;
```
(with `#[cfg(test)]` on `stream_test_fixtures`.)
2. Production re-export brace — the Task-3 consumption surfaces, staged exactly like the C-1 precedent braces:
```rust
// Phase-07 pinned Task-3 consumption surface (record P7-12; plan
// .kilo/plans/20261029-phase07-task2-streaming-client-bridge.md §C): the
// worker consumes the stream seam; clippy-named staging until the worker
// rewires.
#[allow(unused_imports)]
pub(crate) use stream_transport::{
    ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession, YieldedChatStreamBatch,
};
```
(= 1 NEW `#[allow(` line — a compiler-named singleton naming its consumer; TODO bullet "no new `#[allow(` beyond compiler-named singletons" satisfied.)
3. Test-only fixture re-export brace (consumed by the new rosters via the crate root):
```rust
#[cfg(test)]
pub(crate) use stream_test_fixtures::{
    stream_test_block_on, text_item, tombstone_item, ended_item, response_plain,
    response_with_token, response_offline, yielded, ScriptedChatStreamTransport,
    ScriptedSessionSpec, ACCESS_TOKEN_FIXTURE, LIVE_CHAT_ID_FIXTURE,
    RESUME_TOKEN_FIXTURE, PUBLISHED_AT_FIXTURE,
};
```
(used by the Task-2 rosters ⇒ no allow needed; Task-3's worker rosters import through the same brace.)
4. Doc-header trim (PRE-AUTHORIZED, cap-driven): the history paragraphs (Phase-04/05/06 landed-vs-deferred narration inside `//! Guidance for future agents`) are condensed to the standing rules (boundary rules, seam list one-line each, the standing no-Google-contact rule) plus ONE new Phase-07 line:
```rust
//! Phase-07 streaming layer (Task 2, plan ...task2...md): the streamList
//! gRPC seam + prost wire models + Bearer-only client + grpc-status mapper;
//! the resumable loop and the confined runtime thread are Task 3's.
```
Target: the file lands ≤ 200 lines (was 199; the trim must free ≥ 15 doc lines). Code behavior untouched.

---

## §D — Retirement step-set (record §12) — the TASK-2/TASK-3 split (ruling)

**Task 2 deletes NOTHING.** The retirement of `list_messages` is EXECUTED IN TASK 3, entirely. Rationale (explicit ruling for the record): the §12 retirement set = (a) the `list_messages` trait method on `LiveChatTransport` (api_chat_transport.rs), (b) its ureq wire implementation (google_live_chat_transport.rs), (c) the REST list models `LiveChatMessageList`/`ChatListCommand` + `chat_messages_path` list-only portions (live_chat_models.rs), (d) the poll-loop wiring (chat_worker.rs/chat_worker_delivery.rs). Items (a)–(c) are ALL consumed by (d) — while the polling worker remains the active production path (it must remain byte-functional through Task 2 per TODO "the worker/polling path REMAINS ACTIVE"), every one of them is STILL USED; deleting any of them in Task 2 would break the active worker or orphan its fixtures. A junior cannot execute (d) without touching the worker. Therefore: zero deletions in Task 2; no new dead code appears (everything stays consumed); the "no dead code" bullet's enforcement instant belongs to Task 3 where (a)–(d) go out together. Task-2's only duty: ensure its OWN new module work creates no unused surface (§C.8 staging braces, hygiene §F.8).

---

## §E — Test rosters (new files; offline-only; zero sleeps; no Google contact anywhere)

### E.1 `stream_proto_tests.rs` — 14 tests (names in §C.1)
Golden pins (anchors A–H), decode pins, unknown-field / unknown-oneof-arm / unknown-enum-value tolerance, round-trip. Byte arrays come from the appendix; field-number shifting is the thing the goldens LOCK.

### E.2 `live_chat_stream_mapping_tests.rs` — 12 tests
1. `text_message_event_maps_to_a_carrier` (author from displayName, id + epoch carried, text verbatim unnormalized)
2. `display_name_missing_falls_back_to_channel_id`
3. `both_author_sources_missing_skips_the_item`
4. `blank_message_text_is_skipped`
5. `tombstone_items_are_skipped`
6. `chat_ended_event_marks_the_batch_ended_and_skips_its_item`
7. `offline_at_marks_the_batch_ended`
8. `unknown_type_values_are_skipped` (message_type raw 99 per record's PROST-tolerance guarantee: prost maps unknown ENUM values to the DEFAULT fallback variant through the typed accessor, and the raw field stays 99 — either way not TEXT ⇒ skipped; the test asserts map-outcome skip, never the prost internals)
9. `items_keep_the_api_order`
10. `the_carrier_records_ids_and_epochs` (id preserved; epoch via the REUSED parser — assert equal to the parser value the existing `live_chat_mapping_tests` roster already pins for the same literal)
11. `the_next_page_token_carries_forward_when_present_and_stays_none_when_absent`
12. `missing_id_maps_to_none_message_id` (worker-fatal contract: deliver() treats None-id as always-unseen — Task 3 consumes; carrier test only asserts `message_id.is_none()`)

### E.3 `live_error_grpc_status_tests.rs` — 12 tests
1. `permission_denied_becomes_insufficient_permissions`
2. `invalid_argument_becomes_invalid_chat_request` (exact detail constant)
3. `failed_precondition_carries_the_documented_disabled_or_ended_detail`
4. `not_found_resolves_to_the_live_chat_resource`
5. `resource_exhausted_carries_the_rate_limit_reason`
6. `unauthenticated_becomes_authorization_rejected`
7. `unavailable_becomes_backend_unavailable`
8. `internal_becomes_malformed_stream_response`
9. `cancelled_deadline_exceeded_and_unknown_become_network`
10. `unmapped_codes_become_network` (DataLoss representative; reason `gRPC code 15`)
11. `chat_precondition_and_invalid_request_display_lines_are_secret_free` (both exact rendered lines from §C.6)
12. `mapped_stream_statuses_never_embed_the_server_message` (marker message "stream-message-marker-MUST-NOT-LEAK" into EVERY mapped status; assert no mapped variant's Display or Debug contains the marker)

Statuses constructed in-process offline: `tonic::Status::new(Code::X, "stream-message-marker-MUST-NOT-LEAK")` — NO network, no dialing.

### E.4 `stream_client_tests.rs` — 6 tests (names in §C.2)
Bearer-only pins, endpoint/path constants, offline endpoint construction metadata (no dialing), rejected-non-ASCII token class.

### E.5 `stream_transport_tests.rs` — 5 tests (all async pieces run via `stream_test_block_on`; zero sleeps)
1. `stream_seam_objects_stay_send_bounded` — compile pin: `let transport: Box<dyn ChatStreamTransport> = Box::new(ScriptedChatStreamTransport::new());` and `let session: Box<dyn LiveChatStreamSession> = Box::new(ScriptedChatStreamSession::new_empty());` (trait Send superbound ⇒ the boxes are Send; this compiles only when the bounds hold)
2. `fake_transport_opens_scripted_sessions_in_order` (two enqueued sessions open in sequence; request capture equals the passed `ChatStreamRequest` incl. token presence/absence)
3. `fake_transport_yields_scripted_batches_then_clean_close` (a session yields 2 mapped carriers then `None`)
4. `fake_transport_reports_typed_errors_from_the_queue` (enqueued `Err(InsufficientPermissions)` → open passthrough; and a delivery `Err(ChatPrecondition)` surfaces verbatim)
5. `fake_session_carriers_match_the_real_mapper` (`yielded()`-built carriers deepen the mapper seam contract for the worker)

(Adds to ~49 new tests across the five rosters; the caller's "rough +12–20" estimate is superseded by this encoded roster — §F reports the OBSERVED gate count, no doc hardcoding.)

### E.6 Structural assertions not expressed as tests (hygiene-time, §F.8)

- One-production-implementer: `grep -rn "impl ChatStreamTransport for" src/youtube` (non-`cfg(test)`) = exactly 1 (`stream_concrete.rs`).
- The async seam lives ONLY in `src/youtube/` (`tonic::async_trait` hits: stream_transport.rs/stream_client.rs/stream_concrete.rs/stream_test_fixtures.rs — zero in `src/app/`).

---

## §F — Verify and sanity (exact commands; container-only)

1. After code lands: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → fix mechanically (fmt only, no semantic edits) until 0.
2. Targeted rosters (fast feedback):
   - `cargo test --locked stream_proto`
   - `cargo test --locked stream_mapping`
   - `cargo test --locked grpc_status`
   - `cargo test --locked stream_client`
   - `cargo test --locked stream_transport`
   (all under the Compose prefix; expect all-green; counts reported observed).
3. Worker/polling regression spot-run (protects §G): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_worker` → all PASS (worker untouched; rosters must stay green).
4. Clippy: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → 0. Known-tight spots: staging braces (allowed), `is_some_and`/`manual_clamp`-style rewrites (map to a rule-following re-shape, never a new allow).
5. FULL GATE: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, four PASS lines, `ALL CHECKS PASSED`; record the log path + the OBSERVED total (baseline 458 + the roster above ≈ +49).
6. Windows/GNU factor sanity (check-only, NO artifact builds): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --target x86_64-pc-windows-gnu` → 0 (async_trait boxes + prost structs compile for the cross target).
7. NO release builds in Task 2 (`build-linux.sh`/`build-windows.sh` belong to Task 4).
8. Hygiene/structural checks:
   - `git status --porcelain` = exactly the task file set (the 3 modified + 12 created files + plan).
   - `git grep -n "allow(" src/youtube` ⇒ count = existing (8 dead-code + 1 clippy + comment mention in mod.rs) + the ONE new staged `#[allow(unused_imports)]` brace (§C.8).
   - `git grep -in "x-goog-api-key\|key=" src/youtube/stream*" src/youtube/live_error` ⇒ ZERO hits.
   - `git grep -rn "tonic::async_trait" src/app` ⇒ zero.
   - secret grep across the diff: `git diff HEAD -- src | grep -E "ya29|AIza|rtmps://"` ⇒ zero (fixture strings only).
   - the one-implementer grep (E.6 first bullet) = 1.
9. ADDENDUM duties (§L): OBSERVED gate count/log path; OBSERVED golden verification (any hand-math correction); observed transitive pin map (final versions); any deviation encountered with its caller-facing options.

---

## §G — Regression ledger (Task 2 invariants)

- `demo` byte-untouched; its roster untouched. RTMPS argv vectors byte-untouched (`stream_args.rs`/`args.rs` zero edits). Secret-redaction contracts untouched. Single-`error:`-line contract untouched (no new error RENDERING path exists yet). Exit-code tables untouched.
- The worker/polling path REMAINS ACTIVE and byte-functional (Task 2 edits nothing in `src/app/`): `chat_worker_tests`/`chat_worker_flow_tests`/`chat_worker_budget_tests`/`gate_chat_tests`/`chat_source_tests` all stay green (§F.3).
- `src/youtube` REGRESSION: O6 resolution + REST list/mapping/dedup rosters ALL unchanged and green (the retirement untouched per §D). `live_error` existing rosters (`live_error_tests` 15 test-layer incl. variant arms, `live_error_display_tests`) remain green — the new variants are ADDITIVE (matches-based tests unaffected).
- `Cargo.lock` diff: ONLY dependency version/feature entries from §B.3 pins; NO unrelated reformat/churn (diff review step: `git diff Cargo.lock` before the deps commit; if unrelated entries move beyond the expected five + tokio/tonic/prost families, STOP and review).
- Dependency inventory: exactly +tokio/+tonic/+prost direct lines; zero `google-youtube3`; no dev-dependency change.
- Commit list (this task, in order): (1) `chore: track the phase07 task2 streaming client bridge implementation plan` (2) `feat: add the phase07 streamList dependency set and its tracked pins` (3) `feat: land the phase07 streamList client, seam and typed stream classes`. Push? NO (step 5). `git push` forbidden in 4.2.

---

## §H — Compliance checklist (every TODO Task-2 bullet → owning section)

- [ ] Models + mockable stream seam under `src/youtube/`, typed-error/secret-safety/module-boundary conventions, cross-boundary output = carriers + typed error contract | §C, §A.4, §A.6
- [ ] Stream retrieval op: Bearer-only (never `key=`), correct request constants, documented framing decode, typed malformed handling never silently ordinary chat | §C.1–C.4, §A.5, §E.1
- [ ] Resumable streaming loop per Task-1 design | seam + adapter in §C.2–C.4; loop itself Task 3 (§A.7, §I)
- [ ] Bounded/cancellable bridge per Task-1 design | Task 3; design transcribed §A.3/A.7
- [ ] Reuse (not fork) the mapping layer + 512 dedup + skip policy unchanged/tested; never log payloads | §C.5, §E.2
- [ ] Retirement execution with exact steps | §D (Task 2 = none; Task 3 owns the deletions)
- Code rules: files ≤200 (§C budgets), fns ≤50 body, ≤2-arg (fixtures use builders/param objects), depth ≤2, no commented-out code, self-documenting names, clippy `-D warnings` clean (§F.4), zero-sleep offline tests (§E), no secrets (§F.8), no Google contact (§E).

## §I — Explicitly NOT done by this pass (owned later) + expected 4.6 criteria flips

NOT done (flagged; each owning step named):
- The confined runtime thread + the resumable consume loop + stop-ticker `select!` + worker rewire: **Task 3** (§A.7 rehearsed).
- The §12 retirement deletions: **Task 3** (§D).
- Docs (`production-run.md`, structure map, tech.md, context.md, README): **Task 4**.
- Release artifact builds + `ls -l` cross-checks + the Phase-07 completion report + operator manual E2E: **Task 4**.
- The real-Google/TLS handshake validation (offline limitation): **operator manual E2E** (record P7-12 limitation; Task 4 documents).
- `docs/configuration.md` — no new config field exists (`CHAT_STREAM_CONNECT_TIMEOUT_MS` is a module-private constant, record P7-6); Task 4 performs the drift check only.

Expected 4.6 marks for Task 2 (exact):
- TODO Task-2 heading → `[DONE]` (with the 4.6 commit; absent fixes from 4.3).
- **AC-8 flips to `[x]`**: "YouTube-specific models and the stream seam remain in `src/youtube/`; the renderer, `ChatStore`, and streaming modules do not depend on YouTube API types" — fully provable at Task-2 end (§C placement + zero app-side edits; the gate + structure inspection are the evidence).
- **AC-3 is already `[x]`** (Task 1). Nothing else flips:
  - AC-1/AC-2 (production mechanism + never-periodic schedule): Task 3/4.
  - AC-4/AC-5 (message mapping into the pipeline + demo stability): Task 3/4 (the mapper is landed but the pipeline is not switched).
  - AC-6 (reconnect/backoff/boundedness runtime semantics): Task 3 (the loop).
  - AC-7 (runtime confinement): Task 3 (the thread must exist to prove it; Task 2 lands only seam-side machinery).
  - AC-9 (failures reach the production run error policy): Task 3 — partial typing only; do NOT flip.
  - AC-10 (offline tests incl. loopback posture): Task 4's gate re-verdict (Task-2 evidence contributes).
  - AC-11 (artifacts): Task 4. AC-12 (docs/report): Task 4. AC-13 (secrets): Task 4's final audit (Task-2's §F.8 is a step toward it).

---

## §K — 4.2 ordered steps (the implementer's execution list)

1. §B.1 preconditions (clean tree, correct branch) — otherwise STOP.
2. Commit the plan file (§G commit 1).
3. Re-verify S2 minutes-fresh (webfetch the streaming-live-chat page; confirm `repeated string part = 100` + the §1.6 numbers unchanged; access-date it). Any drift from §A.9 ⇒ STOP with evidence (the transcription would be re-audited by the planner, never silently landed).
4. §B dependency purchase + probes + lockfile pins + the two `cargo check` verdicts; commit (§G commit 2).
5. Land the code per §C in this order: `stream_transport.rs` + `stream_proto.rs` (+ both siblings) → `live_error/grpc_status.rs` + `live_error/mod.rs` display edits (+ sibling) → `live_chat_stream_mapping.rs` (+ sibling) → `stream_client.rs` (+ sibling) → `stream_concrete.rs` → `stream_test_fixtures.rs` → `mod.rs` wiring (+ doc trim). fmt-normalize between each landing batch.
6. §F targeted runs → fixes → clippy → zero-sleep audit (grep each new roster for `sleep`/`Instant`/thread::park ⇒ zero).
7. FULL GATE (§F.5) + windows-gnu sanity (§F.6) + hygiene/structural (§F.8).
8. Commit (§G commit 3), with `git add` limited to the 15 task files (3 modified + 12 new).
9. Write the §L addendum into this plan file (OBSERVED records) and commit it together with (8) or as its own `docs:` commit if landed after review.
10. Report: exact file set, gate log path, observed test count delta, addendum pointer.

## §L — Addendum: OBSERVED records (written by 4.2; STOP state — see L.7)

### L.1 S2 re-verification (step 3, 2026-10-29) — PASS

Live fetch of `https://developers.google.com/youtube/data/live/streaming-live-chat` on
2026-10-29 (page footer: "Last updated 2026-10-15 UTC"). Confirmed verbatim: `repeated
string part = 100`; the full §A.9 number set (`live_chat_id=1`, `hl=2`,
`profile_image_size=3`, `max_results=98`, `page_token=99`; response `kind=200`,
`etag=201`, `offline_at=2`, `page_info=1004`, `next_page_token=100602`, `items=1007`,
`active_poll_item=1008`; `LiveChatMessage` `id=101`, `snippet=2`, `author_details=3`;
snippet `type=1`, `live_chat_id=201`, `author_channel_id=301`, `published_at=4`,
`has_display_content=17`, `display_message=16`, oneof arms 19/22/27/28/29/30/31/32/33/34;
author details 10101/102/103/104/4/5/6/7; `PageInfo` 1/2; the enum value table; the
`giftEvent` ID-reuse note). Zero drift ⇒ the §C.1 transcription landed unchanged.

### L.2 Dependency purchase (step 4) — OBSERVED (end state differs from the prescribed path)

- The prescribed `cargo update` re-resolved the Task-1 §14.1 collateral set AND additionally
  bumped the dev-dependency `tempfile 3.14.0 -> 3.27.0`, which pulls `getrandom 0.4.3`
  (edition2024). `cargo check --tests` then fails at manifest parse under Cargo 1.82
  (`feature 'edition2024' is required`) — §G's "unrelated churn" condition fired and the
  §B.3 six-pin budget would have been exceeded (7th pin) to fix it.
- Remedy (end-state goal per §B.3/§G, not path): the lockfile was restored to the pre-task
  state (`git checkout dccacbc -- Cargo.toml Cargo.lock`), the three §B.2 lines re-applied,
  and the lock re-resolved MINIMALLY (`cargo check --tests` adds only the tonic family),
  then four `--precise` pins. Commit 2 was re-created in place (the first attempt was
  unpushed and reset via `git reset --soft`); no unrelated version bump survives.
- Final pin map (in-container, tracked lock): `tokio 1.53.1`, `tonic 0.13.1`,
  `prost 0.13.5`, `indexmap 2.12.1`, `hyper-util 0.1.17`, `tokio-util 0.7.19`;
  `idna_adapter 1.2.0` and `zeroize 1.8.1` needed NO pin (the minimal resolution kept the
  pre-task versions), `hashbrown 0.16.1`, `tempfile 3.14.0`, `ab_glyph 0.2.28`,
  `rustix 0.38.44`, `icu* 1.5.x`.
- `Cargo.lock` diff = additions only (394 lines) + two `windows-sys` dependency-edge lines
  on pre-existing packages (`errno` 0.61.2 -> 0.52.0, `nu-ansi-term` 0.61.2 -> 0.59.0)
  forced by the added `windows-sys` versions; NO pre-existing package version changed.
- Verdicts on the final tree: `cargo check` exit 0; `cargo check --target
  x86_64-pc-windows-gnu` exit 0; `cargo fmt --check` exit 0. Commit 2 = `03e696f`.

### L.3 Golden-byte pins (step 5/6) — OBSERVED corrections

First roster run: 10 passed / 4 failed. Cause: **prost 0.13.5 encodes message fields in
ASCENDING FIELD-NUMBER order, not declaration order** — plan anchors C/F/G assumed
declaration order. Corrected arrays (OBSERVED, now in `stream_proto_tests.rs`):

- C = `28 01 BA 06 03 42 6F 62` (owner flag first, then display_name)
- F = `12 0C 08 01 9A 01 07 0A 05 68 65 6C 6C 6F 1A 06 BA 06 03 42 6F 62 AA 06 05 6D 73 67 2D 31`
- G = `FA 3E 1E <F> D2 8F 31 0D 74 6F 6B 65 6E 2D 66 69 78 74 75 72 65`

Anchors A/B/D/E/H matched the plan exactly. Also observed: the plan's A/B byte-count
annotations are wrong (A = 48 bytes not 54; B = 64 not 72); the arrays themselves matched.
After correction: 14/14 pass, including the round-trip and both tolerance pins; no
assertion was loosened.

### L.4 Mechanical fix-ups (allowed class; each recorded)

- `stream_client`: `STREAM_CONNECT_TIMEOUT_REASON` / `STREAM_CONNECT_FAILURE_REASON`
  widened `const` -> `pub(super) const` (§C.4 imports them from the sibling).
- Endpoint pin: `endpoint.uri().to_string()` renders the canonical URI WITH a trailing
  slash (`https://youtube.googleapis.com/`); the assertion compares
  `trim_end_matches('/')` to the unchanged constant (the constant is separately pinned by
  the host/path test).
- `bearer_token_metadata` gained a 3-line `is_ascii()` guard so the planned
  `AuthorizationRejected` failure arm actually fires: `http`'s `HeaderValue` accepts
  obs-text bytes, so `MetadataValue::try_from` does NOT reject `"Bearer non-ascii-token-Θ"`.
  Roster name/count/intent preserved.
- `ScriptedChatStreamSession::next_batch` uses `.pop_front().flatten()` (the scripted queue
  stores `Option<Result<..>>`; the plan's sketch omitted the flatten).
- `ScriptedChatStreamSession` is imported directly from `stream_test_fixtures` in the
  transport roster (it is not in the §C.8 brace name list).

### L.5 File set (deviation from the "15 files" count)

The §C.5-mandated one-token widening (`non_blank_text`: `fn` -> `pub(super) fn` in
`src/youtube/live_chat_mapping.rs`) makes the task set **4 modified + 12 new = 16 files**;
the plan's "3 modified + 12 new" count predates that requirement. No other file is touched
(`src/app/`, `demo`, docs, scripts: zero edits).

### L.6 Targeted rosters (step 6) — ALL GREEN

`stream_proto` 14 pass, `stream_mapping` 12 pass, `grpc_status` 12 pass, `stream_client`
6 pass, `stream_transport` 5 pass ⇒ **49 new tests, 0 failed**. Zero-sleep audit (grep
`sleep|Instant|park` in the five new roster files) ⇒ zero matches. No test dials Google.

Full-suite evidence (same tree, `cargo test --locked`): **507 passed / 0 failed**
(458 baseline + 49 new), so the regression ledger (§G) holds for every existing roster.

Hygiene matrix (§F.8) observed: one production `impl ChatStreamTransport` =
`stream_concrete.rs` (the fake lives in `#[cfg(test)] stream_test_fixtures`); `allow(`
count in `src/youtube` = 9 = the 8 existing + the ONE new staged brace; `tonic::async_trait`
in `src/app` = 0; secret grep (`ya29|AIza|rtmps://`) across the new files = 0;
`x-goog-api-key` = 0. `key=` = 1 hit — the plan-mandated negative assertion
`assert!(!rendered.contains("key="))` in `stream_client_tests.rs` (not an API-key form).
All 12 new files ≤ 200 lines (largest: `stream_test_fixtures.rs` 198).

### L.7 STOP — dead-code lint vs. the plan's allow budget (step 7 blocked)

**Evidence (verbatim verdict):** `cargo clippy --locked -- -D warnings` exits 101 with
`error: could not compile 'rust-youtube-streamer-service' (bin) due to 39 previous errors`
— 39 `dead_code` findings naming the ENTIRE Task-2 staged surface: `live_chat_stream_mapping`
(7 items), the 2 new `LiveApiError` variants, `live_error/grpc_status` (6 consts + 2 fns),
`stream_client` (3 public consts + 2 reason consts + 2 items + `TonicChatStreamSession`),
`stream_concrete` (struct + `new` + 2 error fns), `stream_proto` (`CHAT_STREAM_PARTS`,
`stream_list_request`), `stream_transport` (2 structs + 2 traits). `cargo check` (both
targets) compiles with the same 39 WARNINGS and exit 0.

**Cause:** Task 2 lands the surface but Task 3's consumer does not exist yet, so every item
is dead in the production bin. §C.8's premise ("everything stays consumed"; the single
`#[allow(unused_imports)]` brace covers the staging) is false: `unused_imports` allows do
not affect `dead_code`, and a re-export does not count as a use for the dead-code pass
(verified experimentally: adding `GoogleLiveChatStreamTransport` to the brace changed
nothing; it made the transport items report as dead as well).

**Options (STOP — caller decides; no code-only fix preserves the §F.8 allow budget):**

1. AUTHORIZE the project's established staging shape: `#[allow(dead_code)]` on the module
   declarations `stream_client`, `stream_concrete`, `stream_proto`,
   `live_chat_stream_mapping`, `live_error` (covers `grpc_status` + the 2 variants) and
   `stream_transport`, each comment naming Task 3 as the consumer/retirer (the exact
   `live_lifecycle`/`live_models` precedent). Effect: gate green; hygiene allow count
   becomes existing + 6 instead of existing + 1; Task 3 retires all six.
2. Move the Task-3 consumer forward (worker rewire now) — Task-3 scope; forbidden here.
3. Accept the red clippy gate — not viable (the gate must be green).
4. One blunt staging attribute `#[allow(dead_code)] mod youtube;` at the crate root —
   keeps the count at +1 but silences dead-code detection for the whole subtree.

**Tree state at this STOP:** commit 1 `dccacbc` (plan tracked), commit 2 `03e696f`
(dependencies + pins). The 16 code files are verified (`cargo fmt --check` 0, both `cargo
check` targets 0, 49/49 new tests green) but **UNCOMMITTED** (step 8 not executed; steps
9–10 pending the decision). No push, no branch change.

---

## Appendix — Computed golden byte arrays (NOT-observed: hand-derived from `key = (field_number << 3) | wire_type`, wire_type 0 = varint, 2 = length-delimited; verified by the §C.1 roster at 4.2 and recorded OBSERVED in §L)

### Key-byte table (varint of the tag, hex)

| Message | field | wire type | tag bytes |
|---|---|---|---|
| LiveChatMessageListRequest | live_chat_id 1 | 2 | `0A` |
| | hl 2 | 2 | `12` |
| | profile_image_size 3 | 0 | `18` |
| | max_results 98 | 0 | `90 06` |
| | page_token 99 | 2 | `9A 06` |
| | part 100 (each) | 2 | `A2 06` |
| LiveChatMessageListResponse | offline_at 2 | 2 | `12` |
| | kind 200 | 2 | `C2 0C` |
| | etag 201 | 2 | `CA 0C` |
| | page_info 1004 | 2 | `E2 3E` |
| | next_page_token 100602 | 2 | `D2 8F 31` |
| | items 1007 (each) | 2 | `FA 3E` |
| | active_poll_item 1008 | 2 | `82 3F` |
| LiveChatMessage | snippet 2 | 2 | `12` |
| | author_details 3 | 2 | `1A` |
| | id 101 | 2 | `AA 06` |
| | kind 200 | 2 | `C2 0C` |
| | etag 201 | 2 | `CA 0C` |
| LiveChatMessageSnippet | type 1 | 0 | `08` |
| | published_at 4 | 2 | `22` |
| | display_message 16 | 2 | `82 01` |
| | has_display_content 17 | 0 | `88 01` |
| | live_chat_id 201 | 2 | `CA 0C` |
| | author_channel_id 301 | 2 | `EA 12` |
| | oneof text_message_details 19 | 2 | `9A 01` |
| | undeclared arms (skipped) 22/27/28/29/30/31/32/33/34 | 2 | `B2 01` `DA 01` `E2 01` `EA 01` `F2 01` `FA 01` `82 02` `8A 02` `92 02` |
| LiveChatMessageAuthorDetails | channel_id 10101 | 2 | `AA F7 04` |
| | channel_url 102 | 2 | `B2 06` |
| | display_name 103 | 2 | `BA 06` |
| | profile_image_url 104 | 2 | `C2 06` |
| | is_verified 4 / is_chat_owner 5 / is_chat_sponsor 6 / is_chat_moderator 7 | 0 | `20` `28` `30` `38` |
| LiveChatTextMessageDetails | message_text 1 | 2 | `0A` |
| PageInfo | total_results 1 | 0 | `08` |
| | results_per_page 2 | 0 | `10` |

### Fixture strings (ASCII hex)

- `"chat-id-fixture"` (15) = `63 68 61 74 2D 69 64 2D 66 69 78 74 75 72 65`
- `"token-fixture"` (13) = `74 6F 6B 65 6E 2D 66 69 78 74 75 72 65`
- `"msg-1"` (5) = `6D 73 67 2D 31`
- `"hello"` (5) = `68 65 6C 6C 6F`
- `"Bob"` (3) = `42 6F 62`
- `"id"` (2) = `69 64`; `"snippet"` (7) = `73 6E 69 70 70 65 74`; `"authorDetails"` (13) = `61 75 74 68 6F 72 44 65 74 61 69 6C 73`

### Anchors

- **A** — `LiveChatMessageListRequest { live_chat_id: Some("chat-id-fixture"), page_token: None, part: ["id","snippet","authorDetails"] }` (54 B):
  `0A 0F 63 68 61 74 2D 69 64 2D 66 69 78 74 75 72 65 A2 06 02 69 64 A2 06 07 73 6E 69 70 70 65 74 A2 06 0D 61 75 74 68 6F 72 44 65 74 61 69 6C 73`
- **B** — as A but `page_token: Some("token-fixture")` (72 B; token block sits between id and parts):
  `0A 0F 63 68 61 74 2D 69 64 2D 66 69 78 74 75 72 65 9A 06 0D 74 6F 6B 65 6E 2D 66 69 78 74 75 72 65 A2 06 02 69 64 A2 06 07 73 6E 69 70 70 65 74 A2 06 0D 61 75 74 68 6F 72 44 65 74 61 69 6C 73`
- **C** — `LiveChatMessageAuthorDetails { channel_id: None, display_name: Some("Bob"), is_chat_owner: Some(true) }` (8 B):
  `BA 06 03 42 6F 62 28 01`
- **D** — `LiveChatTextMessageDetails { message_text: Some("hello") }` (7 B):
  `0A 05 68 65 6C 6C 6F`
- **E** — `LiveChatMessageSnippet { message_type: Some(1), published_at: None, author_channel_id: None, live_chat_id: None, has_display_content: None, display_message: None, displayed_content: Some(TextMessage(D)) }` (12 B):
  `08 01 9A 01 07 0A 05 68 65 6C 6C 6F`
- **F** — `LiveChatMessage { id: Some("msg-1"), snippet: Some(E), author_details: Some(display_name-only C)` (30 B):
  `AA 06 05 6D 73 67 2D 31 12 0C 08 01 9A 01 07 0A 05 68 65 6C 6C 6F 1A 06 BA 06 03 42 6F 62`
  (author block = `1A 06 BA 06 03 42 6F 62`: field-3 wrapper around author_details with ONLY display_name set)
- **G** — `LiveChatMessageListResponse { next_page_token: Some("token-fixture"), items: [F] }` (50 B):
  `D2 8F 31 0D 74 6F 6B 65 6E 2D 66 69 78 74 75 72 65 FA 3E 1E AA 06 05 6D 73 67 2D 31 12 0C 08 01 9A 01 07 0A 05 68 65 6C 6C 6F 1A 06 BA 06 03 42 6F 62`
- **H** — `PageInfo { total_results: Some(1), results_per_page: Some(2) }` (4 B): `08 01 10 02`

### Tolerance fixtures

- **T1** — unknown response field (555, wire 2, 1-byte payload) tail: `DA 22 01 01`
- **T2** — unknown snippet oneof arm (22) after the type varint `08 02`: `B2 01 03 0A 01 01` (tag+length+payload skipped; decode succeeds; `displayed_content` = None; `message_type` stays Some(2))
- **U1** — unknown enum value 99 as the snippet type payload bytes: `E3 01`

---

## Footer

Plan: `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` (this file).
Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — Task 2. Design record: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (P7-1..P7-12, §5–§10, §12). Global plan: `.kilo/plans/20261029-phase07-streamlist-migration.md`.
Precedents followed: sibling-test `#[path]` wiring, `#[cfg(test)]` fixtures, staged re-export braces naming their consumer, the "first-principles S2 re-verification" of Task 1, and pinned golden arrays with an observe-then-verify ADDENDUM.
Ended: 2026-10-29 (produced by the Task-2 4.1b architector pass).
