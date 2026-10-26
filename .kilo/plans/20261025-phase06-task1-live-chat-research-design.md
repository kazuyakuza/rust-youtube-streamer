# Phase 06 Task 1 — Official Live Chat API Research, Verdict and Design (per-task implementation plan)

> Task authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — **Task 1 "Official Live Chat API Research and Design" only.** This file is the Task-1 research + design deliverable (the "implementation plan"). It does NOT implement Tasks 2–4; §4–§6 record the binding decisions those tasks inherit.
> Global plan (binding context): `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
> Baseline: branch `feat/phase06-youtube-live-chat`, v0.12.0 (version bump landed at workflow step 3), `main` ancestor `b75d914`, tree clean, test baseline **391 passed / 0 failed**.
> This plan was produced by the Task-1 4.1b architector pass (2026-10-25). It is saved by that pass; the commit/track act belongs to the 4.2 implementer (§7).

## 0. Scope of this task (TODO Task 1 bullets, verbatim obligations)

1. Read the project context (done by this pass; no findings that alter Task-1 scope).
2. Research current official Google/YouTube documentation — §1 records every official URL + access date + the exact facts used. Non-official/community sources: **NONE were relied on for any decision in this plan** (every decision below cites an official page); should any later 4.3/4.5 pass add community evidence, it must be labeled as such.
3. Record official URLs, access dates, selected method and rejected alternatives, exact API calls/fields, quota calculations, dependencies and transport constraints — §1/§2.
4. Select ONE production retrieval strategy + deterministic fallback/error policy — D-1/D-2/D-6.
5. Define bounded message buffering, ordering/dedup, retry/backoff limits, cancellation semantics, typed error mapping — §3 (D-5, D-6, D-7).
6. Decide the `MessageSource` seam contract — D-8.

Out of scope for Task 1 (per global plan / caller): no `src` edits beyond what this plan itself requires (§7 decides: NONE), no user docs edits, no builds, no gate re-run, no commits by the passer (the 4.2 implementer commits, §7).

---

## 1. Research record — official citations (all accessed 2026-10-25)

| # | Official source | URL | Page "Last updated" |
|---|---|---|---|
| S1 | `liveChatMessage` resource reference | https://developers.google.com/youtube/data/live/reference/rest/v3/liveChatMessages | 2026-10-15 UTC |
| S2 | `liveChatMessages.list` method reference | https://developers.google.com/youtube/data/live/reference/rest/v3/liveChatMessages/list | 2026-10-15 UTC |
| S3 | `liveChatMessages.streamList` method reference | https://developers.google.com/youtube/data/live/reference/rest/v3/liveChatMessages/streamList | 2026-10-15 UTC |
| S4 | Guide: "Streaming Live Chat" (gRPC demo + `stream_list.proto`) | https://developers.google.com/youtube/data/live/streaming-live-chat | 2026-10-15 UTC |
| S5 | `liveBroadcast` resource reference (`snippet.liveChatId`) | https://developers.google.com/youtube/data/live/reference/rest/v3/liveBroadcasts | 2026-10-15 UTC |
| S6 | `video` resource reference (`liveStreamingDetails.activeLiveChatId`) | https://developers.google.com/youtube/data/reference/rest/v3/videos | 2026-10-15 UTC |
| S7 | Official per-method quota-cost table ("Quota Calculator") | https://developers.google.com/youtube/data/determine_quota_cost | 2026-10-15 UTC |
| S8 | YouTube Data API overview (quota model — default allocation) | https://developers.google.com/youtube/data/getting-started | 2026-10-15 UTC |
| S9 | OAuth 2.0 scopes for Google APIs (scope descriptions) | https://developers.google.com/identity/protocols/oauth2/scopes | accessed 2026-10-25 |
| S10 | YouTube v3 REST discovery document (machine-readable, authoritative method/scope bindings) | https://www.googleapis.com/discovery/v1/apis/youtube/v3/rest | fetched live 2026-10-25 |
| S11 | Live Streaming API errors page (pointer; per-method error tables S2/S3 are the operative sources here) | https://developers.google.com/youtube/data/live/reference/rest/v3/errors | not fetched (per-method tables used instead) |

### 1.1 liveChatId resolution (S1, S5, S6, S10, S4)

- The broadcast-carried field is **`snippet.liveChatId`** on the `liveBroadcast` resource: "The ID for the broadcast's YouTube live chat. With this ID, you can use the `liveChatMessage` resource's methods to retrieve, insert, or delete chat messages." (S5). Discovery confirms the field: "The id of the live chat for this broadcast." (S10, `LiveBroadcastSnippet`).
- The same value is mirrored as **`videos.liveStreamingDetails.activeLiveChatId`**: "The ID of the currently active live chat attached to this video. This field is filled only if the video is a currently live broadcast that has live chat. Once the broadcast transitions to complete this field will be removed and the live chat closed down." (S6; the official Streaming Live Chat guide uses this path as the demo's lookup, S4).
- Chat availability semantics (S1, S6): live chat is **enabled by default** for live broadcasts and is available **while the live event is active**; after the event ends, live chat is no longer available for that event. `activeLiveChatId` is removed once the broadcast completes. The docs do NOT spell out the exact moment `snippet.liveChatId` on the broadcast appears/empties — no timing assumption is made anywhere in this design (see D-2: resolution polls the field post-confirmation within a bounded budget; absence after budget is a typed fatal).
- "What indicates chat unavailable/disabled/ineligible": the operative **response-side** indicators are the documented per-method errors (§1.3/S2, S3): `403 liveChatDisabled` ("Live chat is not enabled for the specified broadcast"), `403 liveChatEnded` ("The specified live chat is no longer live"), `404 liveChatNotFound`, `403 forbidden`. The resource-side indicator is the **absence of `snippet.liveChatId`** on our lookup, and (on videos) the absence of `activeLiveChatId`.
- Channel/broadcast ineligibility (channel without live access) fails EARLIER in the Phase-05 path (documented `insufficientPermissions` on insert/list) — recorded, no chat-specific behavior needed.

### 1.2 `liveChatMessages.streamList` — support status, protocol, client requirements (S3, S4)

- streamList IS a currently documented, supported method (S3, last updated 2026-10-15): "This method establishes a server-streaming connection that lets you receive live chat messages for a specific chat with low latency." Quota cost: **5 units per call** (S3; S7 table row `liveChatMessages → streamList → 5`).
- Protocol requirements (S4): gRPC **server-streaming over HTTP/2** (`grpc.secure_channel("dns:///youtube.googleapis.com:443", creds)`); clients must generate language bindings from Google's published `stream_list.proto` (the full proto is on S4). No official HTTP-only REST streaming client is documented; S3's error table has BOTH a gRPC block and a "When using web traffic to connect" block, and the discovery document DOES carry a `youtube.v3.liveChat.messages.stream` REST entry (`GET youtube/v3/liveChat/messages/stream`, same scope trio, quota 5) — but NO official wire/fallback documentation exists for a plain-HTTP streaming client (no chunking/keepalive semantics published). A sync-ureq implementation of streamList is therefore NOT a documented option.
- Rust client requirements for streamList: a generated stub + a gRPC runtime. In practice for Rust that means `tonic`+`prost`+`tokio` (async) or `grpcio` (C++-binding). Both contradict the deliberate sync stack (brief §2.1/§3.3; tech.md S2 "sync-std, Tokio reserved"; pinned `rust:1.82` + Windows GNU cross-build).
- Message payload shape (S3/S4/S10): identical resource shape as `list` (`liveChatMessage` resource; `oneof displayed_content` in proto). Behavior: on connect the server sends recent history, then pushes new messages through the open connection; `nextPageToken` resumes after a disconnect ("provide the last `nextPageToken` you received as the value for the `pageToken` parameter in your new connection request").
- Reconnect behavior: token-resume only (S3); no server-side reconnect guidance beyond that. Errors are gRPC codes (`PERMISSION_DENIED`, `INVALID_ARGUMENT`, `FAILED_PRECONDITION` with a documented UNRESOLVABLE ambiguity: "Due to a gRPC limitation, it is not possible to distinguish based on the error code between a LIVE_CHAT_DISABLED case and a LIVE_CHAT_ENDED case." — starred footnote in S3) or HTTP shapes in web-traffic mode.
- Also documented (S4): high-load channel-pooling guidance (HTTP/2 `SETTINGS_MAX_CONCURRENT_STREAMS`) — irrelevant for our single-stream use, recorded for completeness.

### 1.3 `liveChatMessages.list` — request, response, continuation, empty responses (S2, S10)

- HTTP request (S2): `GET https://www.googleapis.com/youtube/v3/liveChat/messages` (discovery: path `youtube/v3/liveChat/messages`, `flatPath: youtube/v3/liveChat/messages`).
- Required params (S2/S10): `liveChatId` (string; "The live chat ID associated with a broadcast is returned in the `liveBroadcast` resource's `snippet.liveChatId` property"), `part` (supported values: `id`, `snippet`, `authorDetails`).
- Optional params (S2/S10): `hl` (localized currency strings; NOT needed — we accept only `textMessageEvent` and render no currency), `maxResults` (accepts **200–2000**, default 500; "The initial request made without a continuation token will only contain the most recent messages... The API doesn't retrieve messages that are older than those returned by the initial request made without a continuation token."), `pageToken` (from the previous response's `nextPageToken`), `profileImageSize` (16–720, default 88 — we do NOT need profile images).
- Response (S2/S10): `youtube#liveChatMessageListResponse` with **`nextPageToken`**, **`pollingIntervalMillis`** ("The amount of time, in milliseconds, that the client should wait before polling again for new live chat messages"), **`offlineAt`** ("The date and time when the underlying livestream went offline. This property is only present if the stream is already offline."), `pageInfo`, `items[]` (ordered **oldest to newest**), and `activePollItem`.
- Empty responses: the docs phrase continuation as "the API server returns additional chat messages, **if available**" — no explicit statement of an empty-`items` shape. Design consequence: an empty `items` array is treated as an ordinary response (token still advances); the app NEVER infers chat end from emptiness (only from `offlineAt`/`chatEndedEvent`/typed errors, §1.6/D-6).
- Errors (S2): `403 forbidden`; `403 liveChatDisabled`; `403 liveChatEnded`; `404 liveChatNotFound`; `rateLimitExceeded` ("The request was sent too quickly after the previous request... when API requests to retrieve messages are being sent more frequently than YouTube's refresh rates").
- Note displayed atop S2: "To poll for live chat messages, use the `liveChatMessages.streamList` method. The `streamList` method pushes new messages to the client as they become available, which reduces the need for constant polling and helps to avoid exceeding your quota." — this note does NOT deprecate or gate `list`; the method, its parameters, and its full error table remain documented with quota cost 5.

### 1.4 OAuth scopes and quota (S2, S3, S7, S8, S9, S10)

- Scope authority (S10 — authoritative machine-readable bindings): `youtube.liveChatMessages.list` scopes = `{"https://www.googleapis.com/auth/youtube", "https://www.googleapis.com/auth/youtube.force-ssl", "https://www.googleapis.com/auth/youtube.readonly"}`; the discovery's `youtube.v3.liveChat.messages.stream` carries the SAME scope trio. **`https://www.googleapis.com/auth/youtube.force-ssl` (the ONLY scope the app grants, `REQUIRED_SCOPE` in `src/youtube/authorization.rs`) IS sufficient.** The README/description of the scope ("See, edit, and permanently delete your YouTube videos, ratings, comments and captions", S9) is unchanged. **NO re-auth is required.**
- Quota costs (S7 — official table, per call, "All API requests, including invalid requests, incur a quota cost of at least one point"):
  - `liveChatMessages.list` = **5**; `liveChatMessages.streamList` = **5**; `liveChatMessages.insert/delete/transition` = 20 (we never call them); `liveChatBans/Moderators` = 200/50 (never called); `liveBroadcasts.list` = **1** (our chat-id resolution lookup); `liveBroadcasts.*` others = 1; `liveStreams.*` = 1; `videos.list` = **1** (alternative lookup, NOT used by this design).
- Default project quota (S7/S8): **10,000 units/day** combined for all endpoints that don't have their own bucket (`search.list`/`videos.insert` only have separate 100-call buckets); daily quotas **reset at midnight Pacific Time**; quota extensions available via Google's form (S8 link).
- Rate limits: the only chat-specific rate limit is `rateLimitExceeded` (S2) — fires when requests arrive faster than YouTube's refresh rates; honoring `pollingIntervalMillis` is the documented-compliant defense. Generic quota exhaustion surfaces with Google's standard reasons (`quotaExceeded`/`dailyLimitExceeded`, already mapped by the existing `classify_api_error` → `QuotaLike`).
- Retry guidance: official docs publish NO auto-retry guidance for chat polling; the documented posture is honor-the-interval + the per-method error table (this design's bounded retry policy in D-6 is app level and never faster than server guidance).

### 1.5 Message types, author details, publishedAt, text suitability (S1, S10)

- `snippet.type` documented values (S1; proto enum in S4 gives the same set — The REST doc's `snippet.type` value for polls is `pollEvent` (the details object is named `pollDetails`); the proto enum says `POLL_EVENT`. Treat the canonical value as `pollEvent` and match on the documented REST string in code-with-fallback, D-4):
  `textMessageEvent`, `tombstone`, `chatEndedEvent`, `sponsorOnlyModeStartedEvent`, `sponsorOnlyModeEndedEvent`, `newSponsorEvent`, `memberMilestoneChatEvent`, `superChatEvent`, `superStickerEvent`, `userBannedEvent`, `membershipGiftingEvent`, `giftMembershipReceivedEvent`, `pollEvent`, `giftEvent`, plus the legacy/deprecated `fanFundingEvent` details block.
- Value-bearing text per type (S1): only `textMessageEvent` carries an ordinary message (`snippet.textMessageDetails.messageText`). Super Chat (`superChatDetails.userComment`/`amountDisplayString`), Super Sticker, Member Milestone (`userComment`), New Sponsor (no comment field), gifting, gifts (jewels), bans, polls — ALL are events of the paid/membership/moderation family that brief §8.2 explicitly requires the MVP to ignore.
- `snippet.displayMessage` is documented as "Contains a string that is displayed to users" and is "not present if the message type is `chatEndedEvent` or `tombstone`" — a generic rendered string, NOT the user's ordinary-text contract; this design does NOT use it (D-4: the accepted type's own `textMessageDetails.messageText` is the source of truth).
- `snippet.hasDisplayContent` (boolean) and `snippet.publishedAt` (ISO 8601 `YYYY-MM-DDThh:mm:ss.sZ`) documented (S1).
- `tombstone` shape (S1): "a message used to exist with this id and publish time, but it has since been deleted"; "not sent upon deletion of a message, but rather is shown to signify where the message used to be"; **only** `snippet.liveChatId`, `snippet.type`, `snippet.publishedAt` present. Deleted/hidden messages surface as tombstones on re-list — skip policy D-4 handles them.
- `authorDetails` (S1): `channelId`, `channelUrl`, `displayName`, `profileImageUrl`, `isVerified`, `isChatOwner`, `isChatSponsor`, `isChatModerator`. The MVP-visible author = `displayName`, fallback `channelId` (brief §8.2).
- Message IDs (S1): `id` "The ID that YouTube assigns to uniquely identify the message. Note: For giftEvents, the same ID may be reused to update the combo count." — so duplicate-ID tracking (D-5) must tolerate an ID being re-seen later with DIFFERENT type (gift event): the skip-policy runs BEFORE dedup so re-seen IDs of non-text types are skipped anyway; re-seen text IDs are deduped.

### 1.6 Chat lifecycle: broadcast ends / chat disabled / permissions revoked / ineligible (S1, S2, S3, S6)

| Situation | Documented symptom | App-facing outcome (design, D-6/D-9) |
|---|---|---|
| Broadcast ended, chat closed | `chatEndedEvent` message type ("The chat has ended and no more messages can be inserted after this one... a little while after a broadcast ends"; NOT sent for a channel's default broadcast) OR `offlineAt` present in a list response OR `403 liveChatEnded` | Worker stops GRACEFULLY (info log, source yields `Ok(None)` forever, video keeps flowing until Ctrl+C) |
| Chat disabled on the broadcast | `403 liveChatDisabled` at chat-id-resolution/list time | Typed terminal failure (startup: `ChatUnavailable`; fatal exit 1) |
| Wrong/expired liveChatId | `404 liveChatNotFound` | Typed terminal failure (fatal exit 1) |
| Permissions insufficient/revoked mid-run | generic `403 forbidden` (S2) or `401`-class authorization rejection | Typed terminal failure (fatal exit 1); recovery = `auth` re-run (existing docs) |
| Access token expired/stalled | existing credential-service refresh-once has already run at startup; a mid-run 401 persists | Same as permissions revoked (fatal), redacted |
| Quota exhausted mid-stream | Google standard `quotaExceeded`/`dailyLimitExceeded` reasons | Typed terminal failure (fatal exit 1) + operator guidance (D-10) |
| Persisted transient network/backend errors | `Network`/5xx/`BackendUnavailable`/`MalformedResponse`/`rateLimitExceeded` | Bounded retry/backoff (D-6), then typed terminal `Transport` |
| Stream/broadcast ends naturally | `enableAutoStop` auto-completes; chat ends shortly after; operator Ctrl+Cs first in the normal flow | Graceful stop per row 1 |

---

## 2. Design decisions (binding; each picked ONE way — no alternatives left open)

### D-1 — Production retrieval strategy: `liveChatMessages.list` polling. `streamList` REJECTED.

SELECTED: polling via `liveChatMessages.list` (GET `https://www.googleapis.com/youtube/v3/liveChat/messages`) honoring the server's `pollingIntervalMillis` and `nextPageToken` continuation, over the EXISTING ureq transport conventions (Bearer-only, redirects disabled, JSON GET).

REJECTED: `liveChatMessages.streamList`. Grounded rationale (all official-cited):
- Both methods cost **5 units per call** (S7) — polling does NOT spend more quota than streaming; S2's top note ("streamList reduces the need for constant polling and helps to avoid exceeding your quota") is about reducing CALL COUNT, and both equal-cost methods are bounded by the same interval behavior; no official claim that streaming lowers effective quota below equal-cost polling exists in the fetched sources.
- streamList requires a gRPC server-streaming client generated from the published proto (S4). A synchronous Rust service would need `tonic`+`prost`+`tokio` (async runtime forbidden by the deliberate sync-std stack, tech.md S2) or `grpcio` (heavy C++ binding; unevaluated; MSRV/build risk on pinned `rust:1.82` + Windows GNU cross-build). Brief §3.3 REJECTED `google-youtube3` for maintenance reasons; introducing an entire gRPC/async stack for equal-quota-cost chat is exactly the "unnecessary abstractions" the brief forbids (§2.1) and the global plan's R1 escape condition does not trigger (official evidence does NOT make streamList mandatory — `list` is fully documented with a complete error table).
- Latency is irrelevant for this prototype: the frame loop renders at most one message per frame at 30 FPS; chat within ~tens of seconds of posting is product-adequate.
- Polling is reimplementable against the established offline-test pattern (loopback HTTP server), matching every existing seam in `src/youtube/`; a gRPC stub cannot be loopback-tested with the project's fake/loopback conventions without new tooling.

Consequence (global-plan R1 resolution): no dependency decision pauses the phase: **zero new dependencies** (D-11).

### D-2 — liveChatId resolution: ONE documented path, bounded absence budget, typed fatal.

Step 11 of the run startup (after the Phase-05 confirmation gate has joined AND observed `streamStatus == active`):
- New documented operation **O6**: `GET /youtube/v3/liveBroadcasts?part=snippet&id=<broadcast_id>` (quota 1 unit; S7) and read `snippet.liveChatId` (S5/S10).
- Bounded absence budget: **6 attempts × 5 s interval = 30 s** (module constants `CHAT_ID_RESOLUTION_ATTEMPTS = 6`, `CHAT_ID_RESOLUTION_INTERVAL_MS = 5_000`; attempt-failure classification reused from the existing readiness-wait matrix: Network/5xx consume an attempt, other classified errors abort immediately). Rationale: `snippet.liveChatId` is set on the broadcast exactly once — immediately following the live transition — but with the exact timing undocumented; brief §7.2 requires updating the sequence against documented API behavior WITHOUT inventing hardcoded values, so the design POLL-BOUNDS the documented field and fails typed when absent.
- Absence after budget → typed `LiveApiError`-class failure surfaced as the D-8 `ChatUnavailable` retrieval error (fatal exit 1, actionable diagnostic: "the broadcast's live chat id was not returned; confirm the broadcast has live chat enabled").

REJECTED (recorded, not judged): (a) reading `snippet.liveChatId` from the already-captured O2 insert/O3 bind response — no documented guarantee that the field is populated pre-live, and Phase-05 models stopped read-mapping the field; (b) `videos.list → liveStreamingDetails.activeLiveChatId` (S6) — adds a second resource lookup for a field documented against live-chat-enablement, while `liveBroadcasts.list(part=snippet)` is the direct documented owner of the needed field. Resolution reuses the ALREADY-authorized transport and the ALREADY-created broadcast — NO second broadcast, NO lifecycle change, NEVER hardcoded.

### D-3 — Exact polling request form (wire-pinned by Task-2 tests)

- Method/URL: `GET https://www.googleapis.com/youtube/v3/liveChat/messages` (ureq GET, Bearer header only; NO API key, no `key=` — S2/S3 are OAuth-class methods; the app never sends an API key).
- Query params (exact order fixed by the existing path-builder precedent): `liveChatId=<id>` (RFC 3986-encoded via the existing `encode_query_value`), `maxResults=200` (documented minimum 200 — bounds each response; cost is per-call, not per-item), `part=snippet,authorDetails,id` (all three documented supported values; `id` requested explicitly because the plan reads `items[].id` as the dedup/dedup key and does NOT assume it rides on other parts), `pageToken=<nextPageToken>` from the previous response (omitted ONLY on the first call of a worker session).
- NOT sent: `hl` (no localized currency strings rendered), `profileImageSize` (no avatar rendering).
- Body: none (GET; keeps the transport's GET `call()` path — Content-Length behavior unchanged).

### D-4 — Message acceptance policy (mapped + tested; never silently-malformed-as-ordinary)

For each `LiveChatMessageRecord` (provider view of S1):
1. If `snippet.type` != `textMessageEvent` → **SKIP silently per policy** (tested; includes `tombstone`, `chatEndedEvent` handled separately below, all super/membership/gift/ban/poll/gift-event types, and any UNKNOWN/unrecognized type string — future API additions are not ordinary chat).
2. If `snippet.textMessageDetails.messageText` is missing or normalizes to an empty single line → **SKIP** (tested malformed-shape policy; never accepted as ordinary chat).
3. Author: `authorDetails.displayName` if present and non-blank, else `authorDetails.channelId` if present and non-blank, else **SKIP** (no stable identifier exists — tested policy).
4. Map → `crate::chat::ChatMessage::new(author, text)` (the existing normalization flattens line breaks; `Display` stays `author: text`), `.with_message_id(id)` verbatim when `items[].id` present, `.with_timestamp(parse_rfc3339_seconds(published_at))` — a NEW small helper that parses the documented ISO 8601 UTC form `YYYY-MM-DDThh:mm:ss(.fff)Z` to epoch seconds and yields `None` (no timestamp) for anything unparseable — consuming `ChatMessage::with_timestamp` (retires that item-level `#[allow(dead_code)]` at the mapped consumer, per the dead-code retirement ledger precedent).
5. `chatEndedEvent` (or a list response carrying `offlineAt`) → worker terminal state **graceful** (D-6), never a chat line.

Place: the mapping fn lives in `src/youtube/` (provider structs stay there) and returns the youtube-owned normalized carrier `ChatTextMessage { author_display_name, author_channel_id, message_text, message_id, published_epoch: Option<u64> }`; the APP-side constructer of `ChatMessage` performs no other transformation than `ChatMessage::new/with_*`. This keeps YouTube resource structs inside `src/youtube/` AND keeps `youtube` free of a `crate::chat` dependency (both boundary rules hold).

### D-5 — Buffering, ordering, dedup, baseline

- Bounded channel: `std::sync::mpsc::sync_channel::<ChatMessage>(64)`. Constant `CHAT_CHANNEL_CAPACITY = 64` (≥ the 26-line example visible capacity with headroom; bounded, no unbounded buffer anywhere).
- Ordering: accepted items from ONE response are enqueued in API order (documented oldest→newest); token advancement preserves cross-page order.
- Baseline (brief §8.3, TODO Task-2 continuation bullet): the worker's FIRST successful `list` response establishes the baseline — **ALL of its `items` are discarded** (never queued, never logged) and only its `nextPageToken` + `pollingIntervalMillis` are retained. Rationale (S2): "The initial request made without a continuation token will only contain the most recent messages" — everything in that first page is history by definition; discarding it can never discard non-history only under the documented semantics, and satisfies "the stream must start with an empty chat display" + "Do not treat historical messages returned during reconnection as new messages". Known prototype limitation recorded for Task-4 docs: messages that arrive between stream-active and the worker's first poll are discarded with the history page.
- Dedup (provider-ID tracking): every accepted text item's `id` is recorded into a bounded seen-window **`ChatMessage`-independent vector**: last **512** IDs (`CHAT_DEDUP_WINDOW = 512` — `VecDeque<String>` plus `pub(crate)` bounded push; linear `contains`). An item whose `id` is already in the window is skipped; a tombstone can reuse the original ID without conflict because the skip-policy (D-4 step 1) runs before dedup. The window is per-worker-session (in memory only; never persisted — consistent with "the queue is transient").
- The app-side source drains the channel NON-blocking: `Receiver::try_recv()` → message / `Empty` → `Ok(None)` / `Disconnected` → consult the shared terminal cell (D-6) — so the frame loop NEVER blocks on chat and keeps ≥1-message-per-frame pacing.

### D-6 — Retry/backoff, cancellation, terminal classes (deterministic)

Poll loop (worker thread), per full cycle:
1. Check stop flag → break cleanly.
2. `list` call (D-3). Classify:
   - 2xx → reset consecutive-failure counter; process per D-4/D-5; set `polling_interval = response.pollingIntervalMillis if present, else last-known, else DEFAULT_CHAT_POLL_INTERVAL_MS` (default constant **10_000 ms** — used only when the server sends no value; NEVER shorter than server guidance; never an operator override); `offlineAt` present → graceful terminal; sleep `polling_interval` in **1_000 ms stop-slices** (constant `CHAT_SLEEP_SLICE_MS = 1_000`) then continue.
   - Transient (`Network`, 5xx/`BackendUnavailable`, `MalformedResponse`, `rateLimitExceeded`, `quotaExceeded`-in-transient-cases is NOT transient — see terminal) → increment consecutive counter; sleep backoff = max(server interval, 1 s) × 2^(attempt−1) **capped at 60_000 ms** (constants `CHAT_BACKOFF_MAX_MS = 60_000`); continue while consecutive < **10** (`CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`).
   - `LiveApiError::ChatEnded` (from `403 liveChatEnded`) → graceful terminal: worker exits cleanly without writing to the shared terminal cell; source yields `Ok(None)` forever. Do not treat as transient and do not treat as fatal.
   - Terminal (any of): `403 liveChatDisabled`, `404 liveChatNotFound`, `403 forbidden`, 401-class authorization rejection, quota-exhaustion reasons (`quotaExceeded`/`dailyLimitExceeded`) → write the typed failure into the shared cell (D-8) and break.
3. Budget exhausted (10 consecutive transient) → shared cell = `Transport { detail }` → break.

Cancellation: Ctrl+C already flips (a) the frame-loop `StopFlag` and (b) the `WaitCanceller`; Task-3 extends the stop-bundle with (c) the **chat-worker stop flag** (same `Arc<AtomicBool>` shape, flipped by the SAME single handler). All worker sleeps and channel-sends are stop-slice-bounded so a Ctrl+C finalizes the worker within ≤ ~1 s; the worker never blocks shutdown.

Graceful vs fatal (S2's error table drives it):
- **Graceful** (video keeps rolling, worker exits zero-state, source yields `Ok(None)` forever): `offlineAt` observed; `chatEndedEvent` observed.
- **Terminal-fatal** (typed `ChatRetrievalError` in the shared cell): `ChatDisabled`/`NotFound`/permission/authorization/quota/Transport-class.

### D-7 — Typed error mapping (chat side)

- `src/youtube/live_error/` EXTENDED: `classify_api_error` gains the chat-reason arms — `liveChatDisabled` → NEW variant `LiveApiError::ChatDisabled` (15 → 16 variants), `liveChatEnded` → NEW variant `LiveApiError::ChatEnded`, `liveChatNotFound` → existing `ApiNotFound { resource: "live chat" }`, bare `forbidden` → existing `InsufficientPermissions` (verify the current mapping in classify.rs and only ADD where required); hand-written Display lines added for the two new variants (secret-free, no bodies).

  This variant is consumed by the poll loop as a graceful end-of-chat signal (D-6), not as a fatal retrieval error.
- The chat transport returns `Result<…, LiveApiError>` for BOTH wire calls (O6 lookup and O7 list), so the app funnels a single typed source.

### D-8 — `MessageSource` seam decision: minimal WIDENING of the trait's error type. No streaming-mechanism/concurrency semantics enter the trait.

PINNED (Task-2/3 transcribe; do NOT re-adjudicate without caller ruling):
- The existing synchronous trait CANNOT stay as-is: chat retrieval introduces failures that are NOT `ChatError` (Capacity/LogOpen/LogAppend) and collapsing them into chat-log errors is explicitly forbidden (TODO Task-3 bullet). The smallest typed, backward-compatible change is widening the trait's failure type:
  - NEW file `src/app/pipeline/source_error.rs`:
    - `pub(crate) enum ChatSourceError { Chat(ChatError), Retrieval(ChatRetrievalError) }`
    - `pub(crate) enum ChatRetrievalError { ChatUnavailable { detail: String }, Quota { detail: String }, Transport { detail: String } }` — `detail` carries ONLY the redacted `Display` of the inner typed `LiveApiError` (never a body, never a token; the existing redaction contracts hold).
  - `trait MessageSource { fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatSourceError>; }`.
- Mechanical fallout (all backward-compatible, zero behavior change for demo):
  - `ChatFreeSource` (deleted/replaced in Task 3) and `DemoMessageSource` impls: signatures updated to the widened type; demo's outputs/body semantics UNCHANGED (demo can only produce `Ok(...)` or `ChatSourceError::Chat(_)`-class — in practice only `Ok`, since the demo schedule never fails).
  - `src/app/pipeline/test_helpers.rs` + `source_helpers.rs` fakes: mechanical signature updates.
  - The runner's chat-stage error mapping: `ChatSourceError::Chat(e)` → the EXISTING `FramePipelineError::Chat { source: e }` path (log-write failure policy unchanged: `ChatStore::accept` log-first contract intact — a log failure remains fatal per the current contract); `ChatSourceError::Retrieval(e)` → NEW `FramePipelineError::ChatRetrieval(e)` → NEW `RunError::ChatRetrieval(e)` variant → exit 1 with ONE redacted `error:` line (the exit-code table gains NO new code; the single-`error:`-line contract holds).
- Effects on `demo`: signature-only; timeline, schedule, MP4 vector, exit codes byte-stable (regression pins in Task 4).

### D-9 — Chat-id resolution + worker in the run timeline (Task-3 ordering; brief §14.3)

- Startup: stages 1–10 UNCHANGED (Phase-05). NEW stage 11: chat-id resolution (D-2) → construct the chat worker (transport seam injected) → join-spawn AFTER the gate has confirmed `streamStatus == active`, THEN start the frame loop with the real source (the region between stream-active and the worker's first successful poll is the recorded baseline-limit, D-5).
- Shutdown order (extended, keeps every Phase-05 contract): handler flips all stop flags → frame-loop stop (≤ 1 frame) → **chat-worker join (bounded ≤ ~1 s wake + stop-aware sends)** → FFmpeg stdin EOF → wait — i.e., chat stops accepting FIRST (brief §14.3 step 1). The worker's terminal cell is consulted by the source on `Disconnected`.
- Worker failure mapping: any `ChatRetrievalError` fails the run through the D-8 funnel (exit 1) — it does NOT mask a simultaneous FFmpeg/gate primary failure (the existing `primary_shutdown_outcome` selector gains ONE more input: chat failure rank ≤ gate fatal? PIN: chat failure never outranks GateFinish::Fatal — the primary gate/FFmpeg outcome stays the reported one when both occur; encode same precedence philosophy as D-T3-3).

### D-10 — Quota math and operator implications (S7/S8 math; no assumption about the server's exact interval)

Units/day burned by compliant polling = **5 units × (86,400,000 ms ÷ `pollingIntervalMillis`)** entirely IF the operator streams continuously all day; per **hour** of streaming = `5 × 3,600,000 ÷ interval_ms`:

| Chat poll interval (server) | Units/hour | Continuous-session hours before the 10,000/day default is exhausted |
|---|---|---|
| 5,000 ms | 3,600 | ≈ 2.8 h |
| 10,000 ms | 1,800 | ≈ 5.5 h |
| 13,000 ms | ≈ 1,385 | ≈ 7.2 h |
| 20,000 ms | 900 | ≈ 11.1 h |

Operator implications (encoded for Task-4 docs): a ONE-OFF operator session of a few hours fits comfortably; a multi-hour unattended service can exhaust the $0 default — mid-stream exhaustion is typed-fatal (exit 1 with the honest error), the remedy paths are Google's documented quota-extension form (S8 link) or accepting shorter sessions. Note: readiness/confirmation polls (Phase-05, 1 unit each) are negligible (≈ 72 units per failed startup window at 5 s over 180+120 s).

### D-11 — Dependencies: ZERO new. `Cargo.toml` changes: version line ONLY.

 serde + serde_json (models), ureq 2 (transport via the existing engine), std (`sync_channel`, `sync::mpsc`, threads, `VecDeque`) — all already locked. NO Tokio, NO tonic/prost/grpcio (D-1 verdict), NO clock/time crate (RFC3339 parsing is the small hand-rolled reverse of the existing civil-from-days formatter family, `published_epoch` in D-4; no new dependency allowed by precedent).

### D-12 — Worker constants ledger (module-private named constants; no magic numbers, no config fields)

`CHAT_CHANNEL_CAPACITY = 64`; `CHAT_DEDUP_WINDOW = 512`; `DEFAULT_CHAT_POLL_INTERVAL_MS = 10_000`; `CHAT_SLEEP_SLICE_MS = 1_000`; `CHAT_BACKOFF_MAX_MS = 60_000`; `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`; `CHAT_ID_RESOLUTION_ATTEMPTS = 6`; `CHAT_ID_RESOLUTION_INTERVAL_MS = 5_000`; backoff multiplier doc "max(server interval, 1 s) × 2^(attempt−1), capped". NO `config.json` field is added — polling pacing is SERVER-PROVIDED ONLY (never override server guidance; the rule-count stays 21; `docs/configuration.md` needs no new field rows in Task 4 — drift-check only).

### D-13 — Logging/redaction surface for chat (Task-4 docs enumeration input)

NEVER logged: message text, author names, message ids of individual users' messages, access tokens, chat-id value in error payloads? (PIN: the live chat ID is a NON-secret operator diagnostic like `broadcast_id` — it may appear in a tracing field `live_chat_id` for the resolution success line; it is not a Google secret; but message content/authors NEVER reach stderr logs or error lines). New info-level events (constants named in Task 2/3): chat-id resolved; baseline established + discarded-count; chat ended (graceful); each terminal/mapped failure surfaces through the existing single `error:` funnel only.

### D-14 — Test posture carried forward

All offline: fake `LiveChatTransport` (scripted queues per the live_test_fixtures precedent), loopback HTTP server for the concrete ureq chat call pins, fake stop flags/clocks via `#[cfg(test)]` re-injected sleep hooks (the established zero-sleep gate pattern). NO test contacts Google; NO real credentials/broadcasts. Task 4's gates/builds/artifact rebuilds per the standard phase convention.

---

## 3. Bounded worker + channel shape (Task-2/3 assemble exactly this)

```text
[src/youtube/]                                  [src/app/run/ + src/app/pipeline/]
LiveChatTransport (O7 list)                     ChatWorker (std::thread)
GoogleLiveChatTransport ── ureq GET ──►  ──►    loop: stop-check ──► list────────────┐
                                                │ classify (D-6)                     │
                                                │ map/filter/write (D-4)             │
                                                │ dedup ring (D-5)                   │
                                                └── sync_channel::<ChatMessage>(64) ◄─┤
                                                       │ try_recv (non-blocking)      │
                                                ChatSource (impl MessageSource) ──────┘
                                                       + shared Mutex<Option<ChatRetrievalError>> cell + StopFlag
```

- The worker owns: `Sender`, terminal cell (`Arc<Mutex<Option<ChatRetrievalError>>>`), stop flag (`Arc<AtomicBool>`), dedup window, current `nextPageToken`/interval state, the injected `LiveChatTransport` (offline fakes implement it).
- The source owns: `Receiver` + clones of the cell/stop flag. The channel drops its disconnected end when the worker exits.

---

## 4. Tasks 2–4 inherited obligations (decisions they MUST transcribe; details remain their own 4.1b work)

- **Task 2** (`src/youtube/`): D-2 (O6 resolution fn + provider part=snippet lookup), D-3 (wire form + path builder in `api_request.rs`), D-4 (models + `ChatTextMessage` + mapping fn + RFC3339→epoch helper), D-7 (`LiveChatTransport` seam + concrete ureq impl reusing the shared request engine; `classify` chat arms + 2 new `LiveApiError` variants + Display), D-12 constants (chat-side), offline tests (models/body pins via the established `*_tests.rs` sibling pattern).
- **Task 3** (`src/app/`): D-8 (`source_error.rs` + trait widening + mechanical demo/test-helper fallout), D-5/D-6 (`chat_worker`: baseline/dedup/pacing/terminal), D-9 (stage-11 + shutdown joins + `RunError::ChatRetrieval` + precedence), `ChatMessage::with_timestamp` allow-retirement, worker fixtures offline.
- **Task 4**: D-10/D-13 → `docs/production-run.md` chat sections + operator E2E; scope docs: NO change needed (`youtube.force-ssl` sufficiency already documented; verify-drift only); NO `docs/configuration.md` fields; completion report under `.agent/reports/` citing THIS research record.

---

## 5. Task-1 4.2 implementer steps (tiny, ordered; caller-authorized minimal scope)

1. Verify branch state: `git status --porcelain` → empty (tree clean); `git log --oneline -3` shows the step-3 version bump `0.12.0` on `feat/phase06-youtube-live-chat`. If NOT clean/pinned, STOP and ask the caller.
2. Read this plan file; confirm no `.gitignore`-matching file is implicated (the plan lives under tracked `.kilo/plans/`; nothing else is touched).
3. Click commit ONLY this plan file: `git add .kilo/plans/20261025-phase06-task1-live-chat-research-design.md`, `git commit` — message style precedent: `chore: track the phase06 task1 live chat research and design plan`.
4. Verify: `git status --porcelain` → empty again; `git log --oneline -1` shows only the plan file in the commit (`git show --stat --oneline HEAD`).
5. NO `src/`, `docs/`, config, Cargo, script, or `.gitignore` edits. NO dev-checks gate run (no executable surface changed — plan-tracking precedent: the bare plan commits like `67f15c4`/`7c6cd5d` run no gate). NO builds. NO artifacts.
6. Confirm decimal numbers in the record: D-10's table and D-12's constants — if any figure is questioned, STOP and surface to the caller (never reinterpret quota math in the field).

---

## 6. Verification of THIS deliverable (4.5b will re-check)

- [x] Every TODO Task-1 research bullet has official citations + access dates (§1).
- [x] Selected method + rejected alternative recorded with grounded rationale (D-1).
- [x] Deterministic fallback/error policy encoded (D-6/D-7/D-8: retry budget, terminal classes, no silent degradation).
- [x] Buffering/dedup/ordering/cancellation/typed-error decisions encoded (D-5, D-3, D-9, D-8).
- [x] Seam decision + demo effects encoded (D-8: widening with exact enum shapes; demo byte-stable).
- [x] Quota math + operator implications (D-10).
- [x] No scope creep into Tasks 2–4 (§4 is transcription-only; §5 is the implementer's runner).

---

## 7. Explicitly NOT done by this pass (flagged; owned by later workflow steps)

- No commits (the 4.2 implementer commits, §5).
- No `.agent/project-info/context.md` update (owned by the task's 4.4/4.6 documentation passes per the phase conventions — the context-file edit CAN be derived from this plan's §0/§2 summary).
- No src changes, user-document discoveries beyond the fetched official pages, community-source additions, or method re-picks (any of those requires a caller ruling).
- Tasks 2–4 execution and their own 4.1b plans (this file binds their decisions, not their step lists).

## 8. Footer

Plan: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md`
Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — Task 1. Global plan: `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
Ended: 2026-10-25. All official page access dates recorded in §1.
