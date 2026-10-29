# Phase 07 Task 1 — Official streamList Client-Path Research and Design (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — **Task 1 "Official streamList Client-Path Research and Design" only** (the Phase-06 D-1 polling verdict is superseded by an explicit USER RULING of 2026-10-29, not by new official evidence; this record re-verifies official docs and rules the client path).
> Global plan (binding context): `.kilo/plans/20261029-phase07-streamlist-migration.md` (steps/pre-analysis/APPROVED; User Rulings §79–82 honored: NO pre-citation bias on polling fate; task plans AUTO-APPROVED).
> Baseline: `main` @ `346d109` (TODO tracked; `origin/main` @ `e1b8d44`), v0.12.0, working tree clean, test baseline **458 passed / 0 failed**. Branch `feat/phase07-streamlist-retrieval` + version bump 0.12.0 → 0.13.0 are workflow steps 2–3, NOT this pass.
> Precedent: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (S1–S11, D-1..D-14) — this record ADJUDICATES that D-1 record and its starting-point URL set, then re-verifies them with current access dates.
> This plan was produced by the Task-1 4.1b architector pass (2026-10-29). The 4.2 implementer commits it (§13) and executes the toolchain probe (§13.4) whose observed results are added as §14.

---

## 0. Scope of this task (TODO Task-1 bullets, verbatim obligations)

| TODO Task-1 bullet | Where decided |
|---|---|
| Read project context (brief/context/architecture/tech/structure/Phase-06 report/Phase-06 Task-1 record/current implementation) | §0 (done; no finding alters scope) |
| Re-verify CURRENT official documentation (access dates mandatory) | §1–§2 |
| Adjudicate the user-pasted third-party directives claim by claim | §3 |
| Decide the client path (A/B/C) with official citations | §4 |
| Decide polling retire-vs-retain (458-baseline/`demo` implications) | §12 |
| Research + record quota billing semantics as documented (unknowns marked UNKNOWN) | §11 |
| Define reconnect/backoff/resume policy (bounds, budget, token keying, dedup, baseline discard) | §5 |
| Define buffering/backpressure + cancellation/shutdown | §6 |
| Define typed-error mapping (which classes survive, which are new) | §7 |
| Secret-safety surface | §8 |
| Offline test strategy (mockable seam shape; loopback/fake posture) | §9 |
| Confined async-bridge design (if async required — Path A requires it) | §10 |
| Record URLs/access dates/path/fields/dependencies/MSRV/toolchain constraints; STOP conditions | §1, §4, §14 |
| 4.2 implementer steps (plan-tracking + probes; exact commands) | §13 (probe results → §14) |

Out of scope: no production code, no dependency purchase (tree unchanged until 4.2 executes this plan's steps), no user-doc edits (Task 4 owns `docs/`), no Tasks 2–4 execution (this file binds their decisions; their step lists remain their own 4.1b work).

---

## 1. Research record — official citations (ALL accessed 2026-10-29 by this pass)

| # | Source | Kind | URL | Page status |
|---|---|---|---|---|
| S1 | `LiveChatMessages: streamList` method reference | OFFICIAL (Google) | https://developers.google.com/youtube/data/live/reference/rest/v3/liveChatMessages/streamList | "Last updated 2026-10-15 UTC" — UNCHANGED since the 2026-10-15 Phase-06 review (S3 there) |
| S2 | Guide: "Streaming Live Chat" (gRPC demo + full `stream_list.proto` inline + `stream_list_demo.py`) | OFFICIAL (Google) | https://developers.google.com/youtube/data/live/streaming-live-chat | "Last updated 2026-10-15 UTC" — UNCHANGED since the 2026-10-15 review (S4 there) |
| S3 | YouTube v3 REST discovery document (machine-readable method/scope bindings) | OFFICIAL (Google, fetched live) | https://www.googleapis.com/discovery/v1/apis/youtube/v3/rest | fetched live 2026-10-29 |
| S4 | Quota Calculator (per-method costs + default allocation) | OFFICIAL (Google) | https://developers.google.com/youtube/data/determine_quota_cost | "Last updated 2026-10-15 UTC" — UNCHANGED |
| S5 | tonic crate registry object (max/newest version + crate-level `rust_version`) | REGISTRY METADATA (crates.io API, toolchain evidence — NOT a Google page) | https://crates.io/api/v1/crates/tonic | fetched live 2026-10-29 |
| S6 | tonic 0.13.0 / 0.13.1 version objects (`rust_version` per release) | REGISTRY METADATA | https://crates.io/api/v1/crates/tonic/0.13.0 , .../0.13.1 | fetched live 2026-10-29 |
| S7 | prost 0.13.5 version object (`rust_version`) | REGISTRY METADATA | https://crates.io/api/v1/crates/prost/0.13.5 | fetched live 2026-10-29 |
| S8 | tokio 1.54.0 / 1.54.1 / 1.53.0 / 1.53.1 version objects (`rust_version`, yank flags) | REGISTRY METADATA | https://crates.io/api/v1/crates/tokio/{1.54.0,1.54.1,1.53.0,1.53.1} | fetched live 2026-10-29 |
| S9 | tonic 0.13.1 dependency list (transitive floor requirements) | REGISTRY METADATA | https://crates.io/api/v1/crates/tonic/0.13.1/dependencies | fetched live 2026-10-29 |
| S10 | tonic latest docs page (0.14.8; feature-flag surface `tls-ring` / `tls-aws-lc` / `tls-webpki-roots`) | REGISTRY METADATA (docs.rs) | https://docs.rs/tonic/latest/tonic/ | fetched live 2026-10-29 |
| S11 | ureq 2.12.1 version object (the locked `tls` feature = `webpki-roots` + `rustls` + `rustls-pki-types`) | REGISTRY METADATA | https://crates.io/api/v1/crates/ureq/2.12.1 | fetched live 2026-10-29 |
| S12 | tokio latest docs page (1.54.1; platform support incl. Windows; feature list) | REGISTRY METADATA (docs.rs) | https://docs.rs/tokio/latest/tokio/index.html | fetched live 2026-10-29 |

Non-official/community sources: **NONE were relied on for any decision.** S5–S12 are registry metadata used ONLY as toolchain evidence (crate versions/MSRV declaration fields), clearly labeled as such; every protocol/API decision cites S1–S4.

## 1.1 streamList support status, protocol, client requirements (S1, S2)

- streamList IS a currently documented, supported method (S1): "This method establishes a server-streaming connection that lets you to receive live chat messages for a specific chat with low latency. This is the most efficient way to consume live chat messages, as it pushes new messages to your client as soon as they are available, rather than requiring you to poll for updates."
- Delivery semantics (S1): "When you first connect, the API sends a series of messages containing recent chat history. As new messages are posted, the server will continue to send them through the open connection. Messages in each server response are ordered from oldest to newest. Each response also includes a `nextPageToken`. If your client disconnects, you can use this token to resume the stream. To do so, provide the last `nextPageToken` you received as the value for the `pageToken` parameter in your new connection request. The API will then resume sending messages from the point where you left off."
- Protocol (S2): gRPC server-streaming over HTTP/2 with TLS; the demo client targets `grpc.secure_channel("dns:///youtube.googleapis.com:443", creds)`; the client MUST be generated from the published `stream_list.proto` — the complete proto is published INLINE on S2 (package `youtube.api.v3`, service `V3DataLiveChatMessageService`, rpc `StreamList(LiveChatMessageListRequest) returns (stream LiveChatMessageListResponse)`; proto2 syntax). There is NO separately hosted .proto file URL in the pages; the guide page IS the file location.
- Authentication forms documented on S2 rest on the demo client metadata: `x-goog-api-key` (API key) OR `authorization: "Bearer " + <token>` (OAuth 2.0 access token). **The project's Bearer-only rule (brief §6.4; established transport conventions) selects the Bearer form; the API-key form is never sent** (§3 adjudication).
- The transmission request/response rules on S1: required params `liveChatId` + `part` (supported values `id`, `snippet`, `authorDetails`); optional `hl`, `maxResults` (documented 200–2000 with the same "initial request without continuation token only contains the most recent events" clause), `pageToken`, `profileImageSize`. The S2 proto additionally marks `max_results`: **"Not used in the streaming RPC."** — so the app never sends `maxResults`/`hl`/`profileImageSize` (the P7 wire-form decision; request-field parity with the Phase-06 record's D-3).
- Initial-focus rule: `pageToken`-less establishment receives recent history first; `pageToken`-resume establishment sends no replayed messages "from the point where you left off" (S1, quoted above). No official page marks a boundary between history and live deliveries on the SAME connection (recorded as documented-absent; drives P7-8).
- Idle/keepalive/drop: **NO official page documents any keepalive, ping/idle timing, or server drop behavior for the streaming connection** — every reconnect-timing constant in this design is project-side (§5), and the absence is recorded honestly (§11, §10 limitations).

## 1.2 The REST mirror entry and the Path-B question (S3, S1)

- The discovery document (S3) carries the REST mirror method: `id: "youtube.youtube.v3.liveChat.messages.stream"`, `httpMethod: "GET"`, `flatPath: "youtube/v3/liveChat/messages/stream"`, `path: "youtube/v3/liveChat/messages/stream"`, parameters `liveChatId` (query, string), `pageToken` (query), `maxResults` (query, uint32, min 200 max 2000, default 500 — "Not used in the streaming RPC."), `part` (repeated), `hl`, `profileImageSize`; description: "Allows a user to load live chat through a server-streamed RPC. A call to this method has a quota cost of 5 units."
- **NO official wire format for a plain-HTTP streaming response body is documented anywhere** (neither in S1's "When using web traffic to connect:" error table, nor in S2, nor in S3's parameter metadata): no documented chunking/delimiter/JSON-per-event framing, no documented keepalive/comment-heartbeat semantics, no documented mid-stream error propagation for web-traffic mode (the web-traffic table governs CALL failures, and its error rows are identical in meaning to the polling table). The mirror's `alt=proto` parameter hint in S3's global parameters (media/proto formats) is generic and undocumented for THIS method's streaming body.
- Path-B consequence: `ureq` can issue the GET, but the project would be inventing an UNdocumented parser for an undocumented wire (brief §16.9: never claim success without evidence; §7.3: do not hardcode undocumented API behavior). **Path B is REJECTED** (grounding: P7-1's first bullet; evidence quoted verbatim above).

## 1.3 Error tables (S1) — verbatim operative facts

When using gRPC to connect (transport-relevant for Path A):

| gRPC code | Error detail | Description |
|---|---|---|
| `PERMISSION_DENIED (7)` | The caller does not have permission | no permission for the specified chat |
| `INVALID_ARGUMENT (3)` | Request contains an invalid argument | params fail to parse |
| `FAILED_PRECONDITION (9)\*` | Precondition check failed | `LIVE_CHAT_DISABLED` — the chat is disabled |
| `FAILED_PRECONDITION (9)\*` | Precondition check failed | `LIVE_CHAT_ENDED` — chat no longer live |
| `NOT_FOUND (5)` | Requested entity was not found | wrong `liveChatId` |
| `RESOURCE_EXHAUSTED (8)` | Resource has been exhausted (e.g. check quota) | "The request was sent too quickly after the previous request. This error occurs when API requests to retrieve messages are being sent more frequently than YouTube's refresh rates, which unnecessarily wastes bandwidth." |

Starred official footnote (S1): "Due to a gRPC limitation, it is not possible to distinguish based on the error code between a `LIVE_CHAT_DISABLED` case and a `LIVE_CHAT_ENDED` case. YouTube is actively working on a solution to address this issue." — carried verbatim into the P7-11 `FAILED_PRECONDITION` mapping decision (the establishment/mid-session branch).

When using web traffic (S1): `403 forbidden`, `400` (e.g. `pageTokenInvalid`), `403 liveChatDisabled`, `403 liveChatEnded`, `404 liveChatNotFound`, `403 rateLimitExceeded` — the SAME classes as the polling method; identical semantic classes to the app's existing `LiveApiError` variants.

No other mid-stream error documentation exists (what arrives after items begin, and how, is documented only through gRPC mechanics).

## 1.4 Quota (S1, S3, S4)

- streamList quota cost is **5 units per call** (S1 "Quota impact: A call to this method has a quota cost of 5 units."; S3 method description "A call to this method has a quota cost of 5 units."; S4 table row `liveChatMessages → streamList → 5`; same table: `list` 5, `liveBroadcasts.list` 1, `liveChatMessages.insert/delete/transition` 20).
- Default allocation (S4): "Projects that enable the YouTube Data API have a default quota allocation of 100 `search.list` calls, 100 `videos.insert` calls, and 10,000 units per day combined for all other endpoints"; "Daily quotas reset at midnight Pacific Time"; invalid requests still cost ≥ 1 unit.
- Billing semantics for the LONG-LIVED connection are NOT documented (nothing states per-event billing, per-byte billing, or zero billing for pushes after establishment). Re-establishment is a NEW call → 5 units per re-establishment. Full accounting + the documented-unknown list = §11.

## 1.5 Scope sufficiency (S3)

- Discovery (S3) binds `youtube.youtube.v3.liveChat.messages.stream` to the SAME scope trio as `list`: `https://www.googleapis.com/auth/youtube`, `...youtube.force-ssl`, `...youtube.readonly`. **`youtube.force-ssl` (the app's only granted scope, `REQUIRED_SCOPE` in `src/youtube/authorization.rs`) IS sufficient. NO `auth` re-run is required.** (Same verdict as the Phase-06 record §1.4; unchanged.)

## 1.6 Proto wire fields (S2) — the exact table Task 2 transcribes

Request `LiveChatMessageListRequest`: `live_chat_id=1`, `hl=2`, `profile_image_size=3`, `max_results=98` ("Not used in the streaming RPC." — never sent), `page_token=99`, `part=100` (repeated).
Response `LiveChatMessageListResponse`: `kind=200`, `etag=201`, `offline_at=2`, `page_info=1004` (`PageInfo{total_results=1, results_per_page=2}`), `next_page_token=100602`, `items=1007` (repeated), `active_poll_item=1008`.
`LiveChatMessage`: `kind=200`, `etag=201`, `id=101`, `snippet=2`, `author_details=3`.
`LiveChatMessageSnippet`: `type=1` (nested enum), `live_chat_id=201`, `author_channel_id=301`, `published_at=4`, `has_display_content=17`, `display_message=16`, `oneof displayed_content { text_message_details=19, user_banned_details=22, super_chat_details=27, super_sticker_details=28, new_sponsor_details=29, member_milestone_chat_details=30, membership_gifting_details=31, gift_membership_received_details=32, poll_details=33, gift_details=34 }`.
Snippet type enum: `INVALID_TYPE=0, TEXT_MESSAGE_EVENT=1, TOMBSTONE=2, FAN_FUNDING_EVENT=3, CHAT_ENDED_EVENT=4, SPONSOR_ONLY_MODE_STARTED_EVENT=5, SPONSOR_ONLY_MODE_ENDED_EVENT=6, NEW_SPONSOR_EVENT=7, MEMBER_MILESTONE_CHAT_EVENT=17, MEMBERSHIP_GIFTING_EVENT=18, GIFT_MEMBERSHIP_RECEIVED_EVENT=19, USER_BANNED_EVENT=10, SUPER_CHAT_EVENT=15, SUPER_STICKER_EVENT=16, POLL_EVENT=20, GIFT_EVENT=21`.
`LiveChatMessageAuthorDetails`: `channel_id=10101`, `channel_url=102`, `display_name=103`, `profile_image_url=104`, `is_verified=4`, `is_chat_owner=5`, `is_chat_sponsor=6`, `is_chat_moderator=7`.
`LiveChatTextMessageDetails`: `message_text=1`.
`LiveChatMessage.id` has the same giftEvent-ID-reuse note as the REST resource (S1/S2) — the existing skip-before-dedup ordering keeps tombstone/gift reuse safe (§7 reuse note).

## 1.7 What CHANGED vs. the 2026-10-15 review (official pages)

- NOTHING on S1/S2/S4 changed ("Last updated 2026-10-15 UTC" on all three; contents match the Phase-06 S3/S4/S7 quotes). The discovery document (S3) re-fetched live: the mirror entry, scope trio, and quota sentence are unchanged.
- The only NEW evidence is toolchain-side (S5–S12): crate versions and MSRVs advanced since any prior evaluation; the newest tonic (0.14.8) and newest tokio (1.54.x) each declare `rust_version` **1.85 / 1.85→1.88-era > the pinned 1.82** — driving the pinned-selection matrix in §4.

---

## 2. High-level approach (Path A anatomy — decided ONCE)

Establish an OFFICIAL gRPC server-streaming client (tonic) confined to the existing bounded worker thread family, consuming the published proto wire, forwarding mapped text events into the UNCHANGED bounded screen channel; a sync strategy loop reuses the existing failure/budget/dedup machinery verbatim:

1. Worker thread (spawned at gate `Confirmed`, joined after sink EOF — topology byte-unchanged) resolves `snippet.liveChatId` (O6, unchanged) then spawns ONE dedicated runtime thread (the async bridge, §10) that owns a Tokio **current-thread** runtime and the resumable stream loop.
2. The loop: establish (Bearer metadata only) → consume (`select!` over `session.next_batch()` and a 1-s interval tick checking the shared `StopFlag` `Arc<AtomicBool>`) → map/normalize (`textMessageEvent`-only) → dedup (512 window) → `try_send` into the EXISTING `sync_channel(64)` → on drop/error → bounded backoff → re-establish with last `nextPageToken`.
3. Terminal/graceful classes funnel through the EXISTING `chat_failure` CLOSED lists + the terminal cell; the frame loop's `try_recv` source and one-message-per-frame pacing stay byte-untouched.

---

## 3. Claim-by-claim adjudication of the user-pasted third-party directives

(the TODO §Scope bullet: record confirmed facts, undocumented assumptions, rule conflicts, each with citations)

| # | Pasted directive | Verdict | Grounded basis |
|---|---|---|---|
| C1 | "Reqwest+Tokio with `bytes_stream()` line-by-line reading against `GET .../liveChatMessages/streamList`" | **UNDOCUMENTED + RULE-CONFLICT → REJECTED** | There is NO official wire format for a plain-HTTP streaming body (S1 web-traffic table documents statuses, not body framing; S3 carries the mirror entry with NO stream-body semantics; S2 documents ONLY the gRPC client). "Line-by-line reading" implies NDJSON/chunk framing nobody has published. Additionally `reqwest` was rejected in Phase 04 precisely because it pulls an async runtime through a different HTTP stack than the tree's locked `ureq`+rustls engine (tech.md Phase-04 bullet). A gRPC client (Path A) is the only officially documented client form. |
| C2 | "async `tokio::sync::mpsc` queue" between the working component and the frame loop | **PARTIALLY CONFIRMED, SHAPE SUPERSEDED by project rule** | An async runtime IS genuinely required by the official protocol (S2 ⇒ Path A/tonic ⇒ tokio). But the CROSSING boundary must remain the project's existing bounded `std::sync::mpsc::sync_channel(64)` + `try_recv` source seam (global plan pre-analysis 4; TODO §Preserve). The pasted async channel is replaced by the confined-bridge design (§10): the async task forwards INTO the existing channel with stop-aware `try_send` — the SAME non-blocking delivery the existing worker uses (`chat_worker_delivery::send_stop_aware`). No `tokio::sync::mpsc` ships across the app seam. |
| C3 | "typed serde structs for `textMessageEvent`/`superChatEvent`/`userBannedEvent`/`tombstone`" | **CONFIRMED (typed structs), MECHANISM + POLICY SUPERSEDED** | The wire is protobuf, not JSON (S2 publishes a proto2 message schema; serde structs cannot decode it) — typed structs via `prost` derive transcribed from the OFFICIAL proto are the correct form. BUT the visible-message policy is unchanged: ONLY `textMessageEvent` maps to `ChatMessage` (brief §8.2; the documented tested skip policy; TODO §Explicitly-Out-of-Scope: "Support for non-text event types in the visible queue … remains skipped"). Super Chat/ban/tombstone type-enum values are DEFINED (for the skip policy + tests) but their detail structs are NOT carried across the mapping (only the type value is inspected; prost tolerates unknown fields by skipping). |
| C4 | "'or API key query param'" | **RULE-CONFLICT → REJECTED** | Officially the gRPC demo accepts `x-goog-api-key` metadata OR a Bearer token (S2) — but the project rule is Bearer-ONLY OAuth (brief §6.4: "A static API key is not a substitute for OAuth authorization"; the established wire conventions never send `key=` or API-key headers; the O6/O7 loopback pins enforce it). The stream request carries `authorization: Bearer <token>` metadata ONLY (§2, §6). The token must NEVER appear in a URL/query — the API-key form would put a credential in an addressable location anyway. |
| C5 | "self-healing backoff loop keyed on `nextPageToken`" | **CONFIRMED** | Officially: disconnects resume with the last `nextPageToken` as `pageToken` (S1 quoted in §1.1). The bounded shape (cap 60 s, 10-consecutive budget, terminal classes) is project-defined per the existing worker constants and the global plan (5); the pasted "self-healing" framing is encoded as the bounded re-establishment loop, never an unbounded retry. |

Confirmed-with-no-conflict directives are transcribed into §5–§7; conflicting ones are REJECTED with the rule citations — none is silently adopted or silently dropped.

---

## 4. Client-path decision, toolchain verdict, TLS story, and codegen strategy

### P7-1 — Client path: **PATH A** (official gRPC server-streaming client).

- Grounding: Path B requires an officially documented REST-mirror wire format, which does not exist (§1.2 verbatim). Path C is moot because Path A IS officially implementable with maintained Rust libraries (S1/S2 protocol + proto; S5–S10 crates).
- `grpcio` (the C++-binding alternative named in the Phase-06 record) is REJECTED for the same Phase-06 reasons (heavy C++ build, MSRV/build risk on the pinned image + Windows-GNU), plus tonic/prost/tokio are the maintained pure-Rust family explicitly named by the global plan (brief §3.3 sanctions "evaluate maintained Rust libraries … gRPC and Protocol Buffers").

### P7-2 — Dependency set and exact pins (the compat matrix; registry evidence S5–S9)

| Crate | Pinned version | Declared `rust_version` | Verdict vs pinned rust 1.82 |
|---|---|---|---|
| `tonic` | **0.13.1** (latest 0.13.x; newest-any = 0.14.8) | 0.13.1 = **1.75** (S6); 0.14.x = **1.88** (S5) | COMPATIBLE (0.13.1); 0.14.x is NOT and must not enter the lock |
| `prost` | **0.13.5** (newest 0.13.x; required `^0.13` by tonic 0.13.1) | **1.71.1** (S7) | COMPATIBLE |
| `tokio` | **1.53.1** (newest with declared MSRV ≤ 1.82; 1.54.0 yanked AND `rust_version` 1.85; 1.54.1 `rust_version` 1.85; S8) | 1.53.1 = **1.71** | COMPATIBLE (pin required — cargo's default resolution on rust 1.82 is not MSRV-aware, so fresh `cargo update` would pull 1.54.x and fail the gate) |
| `ureq` | 2.12.1 (already locked; unchanged) | 1.71 | — |
| transitive floors (S9) | tonic 0.13.1 requires: `prost ^0.13` (std), `tokio ^1`, `tokio-rustls ^0.26.1`, `webpki-roots ^0.26`, `h2 ^0.4`, `hyper ^1`, `hyper-util ^0.1.4`, `tower ^0.5`, `http ^1`, `tokio-stream ^0.1.16` | — | resolved via the tracked lockfile; the §13.4 probe verifies the FULL set compiles on 1.82 and, if a single transitive patch declares `rust_version` > 1.82, `cargo update -p <crate> --precise <newest-≤1.82-patch>` pins it; if the chain becomes unsolvable (required floor has no ≤1.82-compatible release) → **STOP condition (i)** fires with evidence |

`Cargo.toml` dependency lines (Task 2 transcribes exactly; features chosen client-side only — no server, no axum/router, no generated-code build step). The `Cargo.toml` carries CARET minimums; the PRECISE versions live in the tracked lockfile (the tempfile `--precise` precedent):
`tonic = { version = "0.13", default-features = false, features = ["channel", "prost", "codegen", "tls-ring", "tls-webpki-roots"] }`
`prost = { version = "0.13", default-features = false, features = ["derive", "std"] }`
`tokio = { version = "1", default-features = false, features = ["rt", "time", "macros"] }`
Precise lockfile pins (§13.4 probe commands): `tokio 1.53.1` (`cargo update -p tokio --precise 1.53.1` — REQUIRED; the resolver alone pulls 1.54.x); tonic settles at 0.13.1 (newest ^0.13 today), prost at 0.13.5 — both verified during the probe; if tonic/prost gain a newer patch with `rust_version` > 1.82 before Task 2 lands, the same `--precise` discipline pins them to 0.13.1/0.13.5 respectively.
Feature rationale: `channel` = client `Channel`/`Endpoint` only; `prost` = tonic↔prost codec bridge; `codegen` = the `tonic::async_trait` re-export used by the seam traits (the ONLY codegen-surface API used; no generated code shipped); `tls-ring` + `tls-webpki-roots` = rustls provider + bundled roots (§P7-3); tokio needs `rt` (current-thread scheduler), `time` (stop ticker + connect timeout), `macros` (`select!`).

### P7-3 — TLS choice: **rustls via the `ring` provider + `webpki-roots` trust store**; native-roots and `aws-lc` REJECTED.

- Evidence: tonic 0.13.1's TLS surface is `tls-ring` (rustls/ring) or `tls-aws-lc` (aws-lc-rs) plus `tls-native-roots`/`tls-webpki-roots` root-mode options (S10). aws-lc-rs is a C build (cmake-family complexity) on `x86_64-pc-windows-gnu` — REJECTED. `tls-native-roots` reads OS cert stores via `rustls-native-certs` — unnecessary (the client target is fixed `*.googleapis.com`, already covered by webpki roots) and adds a store dependency.
- KEY in-tree evidence: the locked `ureq 2.12.1` `tls` feature is exactly `webpki-roots + rustls + rustls-pki-types` (S11) — the tree ALREADY locks rustls-family + webpki-roots compiled for `x86_64-pc-windows-gnu` (every Phase-05/06 Windows cross-build succeeded with it, e.g. 8,764,036-byte exe, Phase-06 report "Build Results"). tonic's `tls-ring`+`tls-webpki-roots` resolves through the SAME rustls 0.23/ring/webpki-roots family, minimizing the new transitive surface. Final proof = the §13.4 probe.

### P7-4 — Codegen strategy: **NO tonic-build; NO protoc; hand-authored prost message types transcribed from the OFFICIAL published proto** + tonic's documented generic client.

- Decision: `src/youtube/` owns prost-`Message` structs transcribing EXACTLY the §1.6 field table from the published proto (S2), checked against golden-bytes test pins. The gRPC call is made through tonic's documented generic client API (`tonic::client::Grpc` + `ProstCodec` + `server_streaming::<Req, Resp>` with the service path constant `"/youtube.api.v3.V3DataLiveChatMessageService/StreamList"` derived from S2's service/rpc names).
- Rationale (each pinned, no alternatives left open):
  1. Build-time codegen needs `protoc` at compile time: `tonic-build`→`prost-build` requires a `protoc` binary in `PATH`/`PROTOC` — the pinned `rust:1.82` Docker image does NOT ship protoc, and the Dockerfile may not change without an explicit caller decision (global constraint). Checked-in generated code would also collide with the project's 200-line file cap (the generated package file is a single multi-hundred-line unit that cannot be hand-split).
  2. The projected structure is SMALL: one request message, ~6 response-path messages the app reads (`LiveChatMessageListResponse`, `LiveChatMessage`, `LiveChatMessageSnippet`, `LiveChatMessageAuthorDetails`, `LiveChatTextMessageDetails`, `PageInfo`) plus the snippet type enum. Unknown fields, non-text `displayed_content` arms (22/27/28/29/30/31/32/33/34), and unknown enum values are TOLERATED-AND-SKIPPED by prost — defined but unread fields are not needed for correctness, only the type value and text path are (decode tolerance documented by prost).
  3. It preserves the project's EXISTING modeling precedent: the REST chat client already hand-authored typed models for the documented resource fields (task decision record from Phase 06 Task 2); this is that same discipline applied to the proto encoding.
  4. brief §3.3 ("Do not implement the entire Google API protocol manually if suitable maintained libraries exist") still holds: tonic+prost implement the ENTIRE protocol; the structs are configuration of maintained tools, not a protocol implementation.
- Accuracy safeguards (binding): the §1.6 field table is transcribed verbatim from S2 into a golden test file listing each struct's fields+numbers with the official source line quoted in a comment; the gRPC path constant is pinned by test; and the operator manual E2E (Task 4) is the ONLY real-Google validation of the wire completeness — if a field the app READS is wrong, the typed decode/malformed handling (§7) surfaces it, never silently.
- Prohibited: hand-computing wire bytes, hand-implemented HTTP/2 or framing, or any "fallback" plain-HTTP parser (Path B stays rejected — §1.2).

### STOP-condition ledger for this plan

- STOP (i) toolchain: fires at §13.4 ONLY if the probe cannot produce a compiling set under the pinned image (the crate matrix above already predicts COMPATIBLE-with-pins; the probe is the observed verdict).
- STOP (ii) neither path officially implementable: DOES NOT FIRE (Path A is fully officially documented; §1).
- Neither STOP fired at plan time; the plan completes with Path A and the §13.4 probe as the final observed verdict. If the probe fails, the implementer STOPS ≤ hours into 4.2 and records options for the caller — never silently bumps `rust:1.82`, never edits the Dockerfile.

---

## 5. Reconnect / backoff / resume policy (binding; Task 2/3 transcribe)

### P7-5 — Connection lifecycle state machine (deterministic)

The strategy loop is the SAME worker-thread shape as today, re-keyed from the poll interval to connection events:

1. `establishment(token: Option<String>)`: open a fresh stream session, Bearer metadata only, `part = {"id","snippet","authorDetails"}`, `pageToken` sent ONLY when a token exists (never fabricated). Connect is bounded by `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000` (a `tokio::time::timeout` around the connect future; connect failure = one failure against the budget).
2. `consume`: for each delivered batch (a decoded `LiveChatMessageListResponse`): advance `next_page_token` whenever present; process items per §7; honor `offline_at` presence as graceful end (same class as the polling `offlineAt`).
3. `on session end (drop/clean EOF/error)`:
   - graceful-end classes (§7 verdicts) → exit cleanly;
   - transient/transport classes → `consecutive_failures += 1`; sleep `backoff_delay_ms(consecutive_failures, CHAT_SLEEP_SLICE_MS)` — the base is 1 s (the slice constant reuses `backoff_delay_ms` semantics with a 1-s base; the poll-interval concept has no streaming counterpart — the server governs push cadence): 1 s × 2^(attempt−1), capped at 60 s; budget `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10` → then typed fatal `Transport`; then re-establish with the last token.
   - terminal classes → write terminal cell, exit.
4. NEVER call the streaming call on a fixed/periodic schedule: the only re-calls are (a) the first establishment, (b) re-establishments after a session end. An idle-but-alive session is left untouched (no artificial read timeouts that would convert "sparse chat" into re-connect churn — the 1-s stop-ticker reads the SAME pending future, it does not restart it, so silence never errors).

### P7-6 — Bounds and constants ledger (stream family; module-private, no config fields, rule count stays 21)

Consuming: `CHAT_CHANNEL_CAPACITY = 64` (unchanged), `CHAT_DEDUP_WINDOW = 512` (unchanged), `CHAT_SLEEP_SLICE_MS = 1_000` (unchanged; reuse), `CHAT_BACKOFF_MAX_MS = 60_000` (unchanged), `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10` (unchanged), `CHAT_ID_RESOLUTION_ATTEMPTS = 6` / `CHAT_ID_RESOLUTION_INTERVAL_MS = 5_000` (unchanged O6 budget), NEW: `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000`, `CHAT_STOP_TICK_SLICE_MS = 1_000` (select! ticker period). NO new config field (docs/configuration.md only gets a Task-4 drift-check).

### P7-7 — Resume keying and dedup across reconnects

- `nextPageToken` is updated whenever a batch delivers one (S1: each response includes it) and is the ONLY resume key. The token survives re-establishments (worker-owned state; lost only on worker death — never persisted, consistent with "the queue is transient").
- The 512-window dedup (existing `ChatDedupWindow`) runs across ALL sessions of one worker (never reset across reconnects; item `id` key; skip-policy runs FIRST — re-ordered IDs from tombstones/gift events cannot enter the window).
- Re-deliveries of still-new items after a reconnect are deduped; processed messages are never duplicated into log or queue.

### P7-8 — Baseline/history discard: first-session rule (the officially-indeterminate boundary, decided deterministically)

- Official fact (S1): a token-less establishment "sends a series of messages containing recent chat history. As new messages are posted, the server will continue to send them through the open connection" — with NO documented marker separating history from live on the SAME connection. There is NO documented way to detect "history ended".
- Decision: **the worker's FIRST-established session (token-less by construction) is a BASELINE session: EVERY item it delivers is discarded (never queued, never logged; `published_at`/author metadata never surfaced); its `nextPageToken` is retained.** From the SECOND-established session onward (token resume), delivery is LIVE: items flow under per-item dedup. Re-establishments always carry the token, so officially "resume sending messages from the point where you left off" carries no history replay; item-level dedup is the safety net for server-side re-delivery at the credential/wall boundary.
- Honest limitation recorded for Task-4 docs (operator-facing): messages posted during the operator's first baseline session do NOT reach display or log — the display stays empty during that window (brief §8.3 satisfied), and the recorded cost replaces the Phase-06 baseline cost (which was "the window between stream-active and the first poll"); the window's length is NOT documented (no history/live boundary exists) and is not bounded by anything except the connection lifetime; documented, never claimed fixable.
- The app NEVER infers chat or connection state from item emptiness (only `offline_at`/graceful classes/typed errors; same policy as D-6's "no emptiness-inferred-end").

---

## 6. Buffering / backpressure / cancellation / shutdown (binding seam shapes)

### P7-9 — The `sync_channel(64)` seam is BYTE-UNCHANGED

- The bounded worker→frame-loop channel (`CHAT_CHANNEL_CAPACITY = 64`, `chat_channel()` in `src/app/run/chat_worker.rs`), the `ChatSource` `try_recv` (non-blocking, empty→`Ok(None)`), one message per frame iteration, the stop-aware `send_stop_aware` delivery shape (try_send; on `Full` retain the message and retry in 1-s stop-slices; on `Disconnected`/stop return), and the `Arc<Mutex<Option<ChatRetrievalError>>>` terminal cell are all PRESERVED verbatim. The pasted `tokio::sync::mpsc` directive never crosses this seam (§3/C2).
- Burst backpressure: since the async loop forwards with `try_send`-only + 1-s retain-retry, a burst during a full channel stalls the CONSUMER side (not the video): the stream's HTTP/2 read stalls while the loop waits to forward, exercising the server-side flow-control window until capacity frees. No unbounded internal buffer exists; ITEMS ARE RETAINED (never silently dropped mid-session) while the shutdown contract (≤1 s-examined slices → drop pending and return) bounds shutdown. Shutdown-loss window = same as today: queued-but-undelivered messages (≤64) are never logged (log/video never diverge).

### P7-10 — Cancellation / shutdown semantics (exact)

- The Ctrl+C handler keeps flipping the ONE `StopFlag` (`Arc<AtomicBool>`); the async loop's `select!` includes a `tokio::time::interval(CHAT_STOP_TICK_SLICE_MS)` branch that checks `stop_flag.is_requested()` and `return`s → the runtime task ends → the runtime thread joins ≤ 1 s beyond the last non-stop wait (connect attempts ≤ 10 s; re-establishment backoffs are select!-armed too, so a stop ends them ≤ 1 s).
- Worker topology byte-compatible: spawn at `GateFinish::Confirmed`; join after sink EOF inside `gate.finish()`; the worker joins its runtime thread before returning; gate-Fatal-outranks precedence untouched; the single-`error:`-line contract and exit tables untouched.
- A stalled/dropped connection can not hang shutdown: every async wait is stop-pollable ≤ 1 s except the ≤ 10 s connect attempt (documented).

---

## 7. Typed-error mapping (which classes survive; what's new)

### P7-11 — The funnel stays: `LiveApiError` → `ChatRetrievalError` → `ChatSourceError::Retrieval` → `RunError::ChatRetrieval` (variants and Display lines UNCHANGED).

Mapping additions in `src/youtube/` (new pure module `live_error/grpc_status.rs` — `classify_stream_status`):

| tonic/gRPC received | Mapped `LiveApiError` (existing unless noted) | App verdict (worker policy) |
|---|---|---|
| `PERMISSION_DENIED (7)` | existing `InsufficientPermissions` | terminal → `ChatUnavailable` (fatal exit 1; existing display) |
| `INVALID_ARGUMENT (3)` | NEW `LiveApiError::InvalidChatRequest { detail }` (a request-shape failure is neither a network nor a decode class; never retried) | terminal (a request-shape bug never retried — matches the "do not retry invalid requests indefinitely" posture of brief §12.2) |
| `FAILED_PRECONDITION (9)` | NEW `LiveApiError::ChatPrecondition { detail }` (the OFFICIAL DISABLED/ENDED ambiguity preserved verbatim in the detail) | establishment-time (0 delivered batches): terminal → `ChatUnavailable` (fatal; chat disabled matches existing fatal families). mid-session (≥1 batch delivered): **graceful end** (chat-ended dominance; the broadcast is terminating anyway — never regress the ChatEnded graceful contract to fatal) |
| `NOT_FOUND (5)` | existing `ApiNotFound { resource: "live chat" }` | terminal → `ChatUnavailable` |
| `RESOURCE_EXHAUSTED (8)` | existing `QuotaLike { reason: "rateLimitExceeded" }` semantics — the S1 table defines streamList `RESOURCE_EXHAUSTED` as "sent too quickly after the previous request" → **transient** (matches the existing `is_transient_failure` rate-limit arm) | transient → bounded backoff re-establishment |
| `UNAUTHENTICATED (16)` / Bearer rejected | existing `AuthorizationRejected` | terminal (same as 401-class) |
| `UNAVAILABLE`, `CANCELLED`, `DEADLINE_EXCEEDED`, `INTERNAL`, `UNKNOWN`, gRPC transport/connection errors | existing `Network { … }` / `BackendUnavailable { … }` per shape | transient (bounded, budgeted) |
| clean connection close (stream `None` without status) | existing `Network { … }`-class (transient) | transient re-establishment (budgeted) |
| item batch DECODE failure (prost error on an item) | existing `MalformedResponse { … }` | transient (batch discarded; NEVER treated as ordinary chat — the whole batch is dropped and the token state stays at last-delivered; dedup prevents re-entrance on the next establish) |

- `LiveApiError` variant count: 17 → **19** (the NEW `ChatPrecondition` and `InvalidChatRequest`; decode failure reuses the existing `MalformedResponse`, so no separate deserialize variant is added). Hand-written secret-free Display lines added; the structure map/`tech.md` counts are updated in Task-4 metadata.
- The app-level CLOSED failure classes are REUSED as-is: `is_transient_failure` STAYS CLOSED (the一身 transient list unchanged — no arm is added); the establishment/mid-session branching for `ChatPrecondition` lives in the strategy-loop policy (a pure match, pinned by test), NOT through the shared classifier.
- New pure helpers are ≤ 2-arg, body ≤ 50 lines, file ≤ 200 lines, nesting ≤ 2 (§15).

---

## 8. Secret-safety surface (preserved verbatim; stream additions)

- NEVER logged, same redaction rule set as today: message text, author names/display names, individual user-message ids, access tokens, authorization headers, raw request/response bodies, decode payloads, stream keys, RTMPS URLs.
- The stream requests add TWO non-secret diagnosable constants when logging failures: the `live_chat_id` (already the sanctioned non-secret per Phase-06 D-13) and the gRPC status code number (Google's own code identity). The status MESSAGE is NOT logged verbatim: the mapping `detail` for the new `ChatPrecondition` carries ONLY the documented `LIVE_CHAT_DISABLED`/`LIVE_CHAT_ENDED` reason labels (official tokens from S1); arbitrary server status text is logged as the redacted classification shape, e.g. "stream request rejected (code 9)".
- The existing loopback wire-pin tests continue to cover Bearer-only on the REST side (the O6 lookup loopback pins still enforce Bearer-only, never `key=`), and the streaming-side Bearer metadata construction is unit-pinned offline (no `key=` anywhere on the stream request surface).
- Nothing new is logged per-event (message arrival is NOT logged; the D-13 events stay: chat-id resolved, baseline established + discarded-count, chat ended, terminal/error funnel lines).

---

## 9. Offline test strategy (mockable seam shape; no test reaches Google)

### P7-12 — Seam: traits over the runtime-confined wire (async via `tonic::async_trait` re-export)

Shapes (Task 2 transcribes; names fixed):
- `src/youtube/stream_proto.rs` — prost message structs from the §1.6 table + request/service-path constants + the snippet type enum; `stream_proto_tests.rs` — golden-bytes encode/decode pins (each field number asserted by encoding a populated struct and matching the pinned byte sequence) + tolerance pin (unknown fields/values decode-and-skip).
- `src/youtube/stream_client.rs` — the concrete tonic adapter: endpoint constant `https://youtube.googleapis.com` (S2's `dns:///youtube.googleapis.com:443` mapping; pinned by test), a pure Bearer-metadata builder (name finalized at Task 2 within the module's naming conventions; unit-pinned offline), session opening with the tokio timeout, and the `LiveChatStreamSession` adapter wrapping `tonic::Streaming<LiveChatMessageListResponse>`.
- The MOCK SEAM (sync-worker-reusable): `pub(crate) trait ChatStreamTransport: Send` with one async method `open(&mut self, request: ChatStreamRequest) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError>` where `LiveChatStreamSession: Send` exposes `async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>>`. Fakes implement both with scripted `VecDeque` sequences (wire-status → typed-error classification pins re-use the same shapes the worker consumes). This replaces ureq-loopback wire testing for the stream side ONLY — documented limitation: the tonic `Channel`/hyper/rustls handshake itself stays untested offline (covered by the operator manual E2E; recorded in Task-4 docs).
- `live_error/grpc_status_tests.rs` — pure status→error-classification pins for every §7 table row (constructed in-process; no network).
- Worker rosters (`src/app/run/`): baseline-discard, resume-to-keyed re-establishment + dedup across reconnect, establishment-budget exhaustion → `Transport` terminal, `FailedPrecondition` dual-branch (establishment fatal vs mid-session graceful), burst `Full` retain-policy, disconnect-then-resume without replay, stop-ticker stop ≤ 1 slice, offline_at graceful end, malformed-never-as-chat. All offline (`#[cfg(test)]` fakes; no sleeps per the zero-sleep test rule; every async piece runs either inside the confined threads or under a test-constructed runtime — the TODO permits test runtime use).
- Per-roster file plan respect: each new file ≤ 200 lines; sibling `*_tests.rs` wired via the established `#[path]` modules pattern; fixture reuse from `live_chat_test_fixtures` where shapes overlap.

---

## 10. Async-bridge confinement design (spec for Task 3; Task 4 documents in `tech.md`)

- ONE runtime instance for the app: `tokio::runtime::Builder::new_current_thread().enable_all().build()` constructed ONCE on the dedicated chat worker's runtime thread; never in the frame loop, never in `auth`, never a second instance.
- The runtime thread runs the complete consume loop described in §5–§6; state (token, dedup window, baseline flag, consecutive failures, in-retention message) lives INSIDE the async task; cross-thread outputs are ONLY: `try_send(values)` into the existing channel + the terminal cell + the `StopFlag` read.
- Bounded join: the worker join chain = gate joins worker → worker joins runtime thread; every await is stop-pollable ≤ 1 s (ticker/seed backoff) other than the ≤ 10 s connect attempt (constant) — matching the existing bounded-join argument verbatim in structure.
- Documented limitations for `tech.md` (Task 4 writes them): a current-thread runtime confined to one thread with no nested `block_on` calls anywhere else; no work-stealing threads; HTTP/2 flow control is genuine backpressure (a full screen channel stops stream reads); connect window 10 s; the official idle-drop behavior is undocumented so re-establishment is event-driven only; per the S1 footnote, live chat DISABLED vs ENDED is not distinguishable by gRPC code (the mid-session graceful choice is a deliberate, typed, documented decision).

---

## 11. Quota semantics RECORDED AS DOCUMENTED (never invented math)

Documented facts (S1/S3/S4):
- streamList costs **5 units per call**. `liveBroadcasts.list` (O6 chat-id resolution) costs **1 unit** (unchanged from Phase 06). Invalid requests still cost ≥ 1 unit. Default 10,000 units/day combined; reset midnight Pacific.
- Re-establishment = a new call = 5 units. The app's documented-max churn under the bounded budget = 10 consecutive failure-budget re-establishments before typed fatal, plus the initial establishment = **≤ 11 calls (≤ 55 units) per sustained outage window** (this IS a documented-cost-upper-bound derived from project constants, not invented per-event math).

Documented UNKNOWNs (recorded, never filled in):
- Whether pushes on an ESTABLISHED long-lived connection bill again per item/per batch — undocumented.
- Whether connection lifetime itself bills continuously — undocumented.
- Any keepalive/idle-drop timing on the server side — undocumented.
- `quotaExceeded`/`dailyLimitExceeded` appearance on the gRPC path is undocumented: code 8 (RESOURCE_EXHAUSTED) carries the documented rate-limit meaning for streamList (sent-too-quickly), and the web-traffic table shows only the `rateLimitExceeded` row. Mid-run quota-shaped exhaustion on the streaming path is therefore classified into the existing rate-limit/transient arm under the bounded budget — honestly recorded as an unknown in the Task-4 docs; with token resume, a transient re-establishment loses no messages.
- Operator implication guidance (Task-4 docs): streaming REPLACES the per-interval burn (supersedes the polling D-10 table); the residual documented cost is per-connection establishment; the unknowns above mean the docs must NOT promise quota-neutral behavior beyond the documented call-cost facts.

Replacement-for-D-10: `docs/production-run.md` (Task 4) states the documented 5-units-per-connection-call floor, ≤ 55 units per bounded adjacent-failure window, the resolution lookup 1 unit, and the unknowns verbatim — no table inventing per-event quota.

---

## 12. Polling fate: **RETIRED from the production path** (ruling; official evidence only)

- Officially (S1): streamList is "the most efficient way to consume live chat messages", pushing messages "instead of requiring you to poll for updates"; the `list` reference note (Phase-06 S2) recommends streamList for reducing constant polling while `list` REMAINS a fully documented supported method (not deprecated). Per-call cost is equal (5 units); an established idle stream makes NO additional documented calls while polling burns 5 units per server-directed interval — at the documented CALL level the streaming profile strictly dominates (per-push billing is undocumented — §11 recorded as UNKNOWN).
- USER RULING (2026-10-29): streamList is the production retrieval strategy; polling loops must not remain the primary pacing mechanism (TODO Objective). Official evidence does NOT contradict the ruling; no official fact creates a first-class fallback-need (list's documentation continues to exist, but carrying a cataloged-never-wired fallback adds dead code the project's policies forbid).
- RULING: **RETIRE the `O7 list` production surfaces when the streaming client lands** (Task 2/3 execute the exact retirement): remove the `list_messages` trait method and its ureq wire implementation, the REST list models `LiveChatMessageList`/`ChatListCommand` and the `chat_messages_path` builder's list-only portions, and the poll-loop wiring in the worker; the `LiveChatTransport` trait SHRINKS to the O6 lookup (`lookup_broadcast_chat_id`).
- SURVIVORS (explicitly retained, reused — never forked): O6 resolution (`live_chat_resolution.rs` + budget + fakes), the mapping policy layer (the RFC3339 parser `live_chat_rfc3339.rs`, the `ChatDedupWindow`, the `ChatTextMessage` carrier, and a NEW proto-side page mapper in the `live_chat_mapping.rs` family operating on the proto types with the SAME rules — Task 2's mapping work), the terminal cell + failure classifiers (`chat_failure.rs`), the ChatSource/MessageSource seam, ChatStore log-first, the frame-loop consumer, the `demo` command (UNTOUCHED — byte-stable schedule/vector).
- 458-test-baseline implications: the REST-list-only rosters retire (decode pins for the `youtube#liveChatMessageListResponse`, the O7 wire-check pins, poll-cycle worker rosters); the mapping policy rosters gain proto-side twins; O6 rosters stay green. The EXACT surviving count is observed-at-gate per task (no doc hardcodes a predicted total; the Task-4 gate re-count governs); the retirement ledger in `.agent/project-structure.md` and the dead-code inventory are updated with the deletions (zero-new-`#[allow(` rule holds — retired items are DELETED, not allowed).
- Reversion path: if Task 2/3 implementation surfaces an official/operational blocker in the streaming path, retirement is REVERTED only by an explicit caller ruling (the poll code would be restored from history) — never silently re-added.

---

## 13. 4.2 implementer steps (exact, ordered)

1. Verify state: `git status --porcelain` → EMPTY (tree clean); `git log --oneline -3` shows the step-2/3 branch+version work (`feat/phase07-streamlist-retrieval`, bump `0.13.0`) — Branch creation/switch is step-2 only; do NOT create the branch here. If the tree/branch state is not as presumed, STOP and ask the caller.
2. Read this plan; confirm the probe steps' preconditions (nothing staged).
3. Commit ONLY this plan file:
   - `git add .kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
   - `git commit -m "chore: track the phase07 task1 streamList research and design plan"`
4. TOOLCHAIN PROBE (observed MSRV/windows-GNU verdict; the tree returns to clean after):
   a. `git status --porcelain` → confirm clean.
   b. Open `Cargo.toml` and add the P7-2 dependency lines EXACTLY (the three caret lines with the listed features).
   c. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update` (updates `Cargo.lock`; NOT `--locked` — this probe intentionally resolves).
   d. Pin tokio: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update -p tokio --precise 1.53.1` (REQUIRED — the fresh resolver pulls 1.54.x whose MSRV is 1.85). Verify the lockfile inside the container (`grep -A2 'name = "tokio"' Cargo.lock` shows `1.53.1`; `grep -A2 'name = "tonic"' Cargo.lock` shows `0.13.1`; prost shows `0.13.5`).
   e. Probe 1: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check` (host Linux target; the lockfile entry tokens are the responsibility of the update step; expect DONE with zero errors — this also compiles tonic/prost/tokio for the first time).
   f. Probe 2: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --target x86_64-pc-windows-gnu` (Windows GNU compile-only verdict; the image carries the mingw toolchain and the target's rust-std since Phase 01.2; ring/rustls compile here exactly as the existing ureq TLS stack does).
   g. On a cargo rust-version error naming a transitive crate at version V: look up that crate's NEWEST version whose `rust_version` ≤ 1.82 on crates.io, run `cargo update -p <crate> --precise <ver>`, then retry (e) and (f). Bound: at most 6 pin iterations; beyond that stop and record.
   h. Record BOTH probe results verbatim (exit codes + any version-pinning actions) into §14 of this file.
   i. RESTORE the tree: `git checkout -- Cargo.toml Cargo.lock`; verify `git status --porcelain` → EMPTY. NO probe artifacts are committed.
   j. IF any probe is fundamentally UNFIXABLE under the pinned image/table → STOP condition (i): record the evidence + the exact options for the caller in §14 (options: (α) keep pins at the failing-point and STOP the phase; (β) caller-authorized image-pin bump `rust:1.82` → newer (a whole-tree decision, NEVER silent); (γ) caller-authorized alternative crate set). Do NOT choose among them; STOP and ask.
5. Probe-success path only: SECOND commit — append the §14 probe record to this plan file AND insert the Task-1 note into `.agent/project-info/context.md` (a NEW dated "## Phase 07 Task 1 — streamList client-path research record landed (2026-10-29)" block, ≤ 10 lines, verbatim: path A decided, key pins, probe verdicts, polling-fate record-ruling; plan path named). Commit BOTH files:
   - `git add .kilo/plans/20261029-phase07-task1-streamlist-research-design.md .agent/project-info/context.md`
   - `git commit -m "docs: record the phase07 task1 toolchain probe verdict in the task plan and context"`
   - `git status --porcelain` → EMPTY.
6. NO `src/` edits, NO builds (artifacts unchanged; the plan-tracking bare-commit precedent applies), NO docs/ production edits, NO version-file changes beyond what step 2–3 already own (steps 2–3 own the bump; this pass changes NO version).
7. `demo`, `RTMPS argv`, redaction, exit codes, and every regression lock are untouched by definition (no code edits).

### 4.3 / 4.4 / 4.5b / 4.6 expectations (carried by the caller's workflow)

- 4.3 code-reviewer + code-simplifier review THIS record (fidelity of the citations, the decision table, the retirement ruling, the check-size inclusion) and may require content fixes ONLY (no code exists to simplify).
- 4.4 docs-specialist: deliverables are the plan record + `context.md` note ONLY (already committed by 4.2 if step 5 landed) — NO operator docs edits (they are Task 4's), NO structure-map/tech.md edits yet (they record the IMPLEMENTATION, not this research ruling; structure/tech updates happen in Tasks 2–4).
- 4.5b architector adherence re-check (evidence into an adherence file under `.kilo/plans/`).
- 4.6 implementer marks TODO Task 1 `[DONE]` + criteria flips only where already provable at this point (research completed, path decided with citations, quota semantics recorded, STOP conditions adjudicated: none fired pre-probe — the probe verdict governs criterion 3; leave unchecked until §14 lands the verdict if the probe state is unknown).

---

## 14. Task-1 Addendum: observed toolchain probe verdicts (written by 4.2 §13.4; TO BE FILLED)

*(pending the 4.2 probe run — the implementer records here: command list executed, exit codes, exact resolved versions pinning, any `cargo update -p --precise` actions taken, both target verdicts, and the STOP ledger state: fired / not fired.)*

---

## 15. Compliance checklist (every TODO Task-1 bullet → plan section)

- [x] Read project context per the bullet | §0 row 1.
- [x] Re-verify CURRENT official docs with access dates | §1 (§1.7 change-detection note included).
- [x] Adjudicate user-pasted directives claim-by-claim (confirmed/undocumented/rule-conflict) | §3 table.
- [x] Decide client path A/B/C with official citations; toolchain verdict against `rust:1.82` and Windows-GNU cross-build | §4 (P7-1/P7-2/P7-3/P7-4 + §13.4 probe; STOP ledger §4-last).
- [x] Decide polling-fate retire-vs-retain with citations; 458-test-baseline + `demo` implications | §12.
- [x] Quota billing semantics AS DOCUMENTED (unknowns marked UNKNOWN; no invented math) | §11 (+ operative per-call code table §1.4).
- [x] Define reconnect/backoff/resume policy (bounds, consecutive-failure budget, `nextPageToken` keying, dedup, baseline-discard interaction) | §5 (P7-5..P7-8).
- [x] Define bounded buffering/backpressure + cancellation/shutdown semantics | §6 (P7-9/P7-10).
- [x] Define typed-error mapping (survivors + NEW stream classes) | §7 (P7-11).
- [x] Define secret-safety surface | §8.
- [x] Define the offline test strategy (mockable seam shape; fake stream transport; loopback posture) | §9 (P7-12).
- [x] Specify the exact confined-bridge design if async required | §10 (+ global-plan pre-analysis 4 satisfied).
- [x] Record official URLs + access dates + selected/rejected path + exact API calls/fields + dependencies + MSRV constraints | §1, §4, §1.6.
- [x] STOP conditions enumerated with caller-facing options | §4 (two STOPs), §13.4(j).

### Regression-lock ledger (this package's invariants; Tasks 2–4 re-pin per their own 4.1b)

- `demo` byte-stable (schedule/vector/exit contracts) — untouched by this decision; Task-4 pins re-verify.
- RTMPS argv vectors, secret redaction, single-`error:`-line contract, exit-code tables, clean Ctrl+C stop (exit 0) — untouched; the stream path ADDs no log lines beyond the existing six lines + the gRPC status-classified failure funnel.
- zero new `#[allow(` beyond compiler-named singletons (existing 8 dead-code + 1 clippy + comment-mention inventory unchanged); IF any clippy lint lands on the prescribed pure mappers, prefer a bounded rule-following re-shape, and only as a last resort ask the caller (the HTTP-411 `Box<ureq::Error>` precedent documents the established shape of such compiler-forced adaptations).
- no test contacts Google; no secrets in any committed file; the 200-line / 50-line / 2-arg / depth≤2 code rules govern anything this plan prescribes for Tasks 2–3.
- scope stays EXACTLY the current production wire + this narrow shift; the omitted-from-scope items (OAuth flows, broadcast lifecycle, renderer architecture, non-text events, config fields) are NOT silently changed by this decision.

---

## 16. Explicitly NOT done by this pass (flagged; owned by later steps)

- No commits (the 4.2 implementer commits per §13), and NO `Cargo.toml`/`Cargo.lock` probe change survives the §13.4(i) revert (the probe edits are temporary; the REAL dependency addition is Task 2's feature commit).
- No Tasks 2–4 plans; their per-task 4.1b records start from THIS record's decisions and must transcribe them (P7-1..P7-12), then decide their own step lists.
- No operator-doc updates (Task 4); no structure-map/tech.md updates (Tasks 2–4); no version-line change (step 3 already bumped).
- No live verification of the wire PATH is claimed anywhere: the loopback/fake tooling + golden pins constitute the offline evidence; the real-Google E2E stays the OPERATOR-RUN manual procedure.

---

## 17. Footer

Plan: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` — Task 1. Global plan: `.kilo/plans/20261029-phase07-streamlist-migration.md`.
Supersedes-for-production: Phase-06 record `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` D-1 (polling selection) — by USER RULING 2026-10-29 re-affirmed by official evidence (no official fact contradicts it); the Phase-06 record's other decisions survive (O6 resolution, mapping policy, families of terminal classes, dedup shape, boundedness constants).
Ended: 2026-10-29 (produced by the Task-1 4.1b architector pass). All official/registry source access dates in §1.
