# Implementation Plan — Phase 06, Task 2: YouTube Live Chat API Client and Message Mapping

> TODO (authority): `.agent/todos/20261025/20261025-todo-2.md` — **Task 2 bullets only.**
> Binding research + design record: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (§2 decisions D-1..D-14; §3 worker shape; §4 inherited obligations — this plan TRANSCRIBES them; §1 official citations are inherited verbatim, not re-verified here).
> Global plan (binding context): `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md` (§ Global Pre-Analysis, § Per-Task Pre-Analysis "Task 2").
> Workflow: `.kilo/commands/critical-workflow.md` — this plan covers ONLY step 4.1b analysis for **Task 2** plus the 4.2 execution recipe; 4.3/4.4/4.5b/4.6 applicability is codified in §9.
> Baseline: branch `feat/phase06-youtube-live-chat`, HEAD `95d791d`, v0.12.0, tree clean, test baseline **391 passed / 0 failed**.

## 0. Scope of this task

TODO Task 2 bullets (verbatim obligations, mapped):

| TODO Task-2 bullet | Covered by |
| --- | --- |
| Add YouTube-specific models + mockable API/source interfaces under `src/youtube/`, existing direct-ureq transport + typed-error conventions | S2 (models), S4 (seam + concrete transport), D-3 wire form |
| Reuse authorized token; never log token/auth headers/raw bodies/sensitive data | S4 (Bearer-only engine reuse, D-T2 conventions carried), §5 secret-safety |
| Resolve the broadcast's chat ID from documented fields; absent/disabled chat ⇒ explicit typed error + actionable diagnostic | S5 (O6 resolution), S3 (ChatDisabled + classify), D-2's bounded budget |
| Implement the selected documented retrieval operation with correct request parameters + response decoding | S4 (O7 list), D-3 exact request form |
| Map supported text messages → app model with ID + normalized timestamp; documented tested skip policy; malformed never ordinary chat | S6 (mapping), D-4 |
| Define behavior: empty responses, dup/replayed IDs, ordering, continuation, deleted/hidden, unsupported types, chat ended, permission/quota/transient/malformed, cancellation | S6 + S7 (error classes); the WORKER behavior (baseline discard, scheduled sleeps) is Task 3 per record D-5/D-6 |
| Honor server poll interval + continuation token; no busy-loop/invented interval | S4 decodes `pollingIntervalMillis`/`nextPageToken`; the actual honor-loop is Task 3's worker (D-6; Task 2 only decodes + pins the values) |
| Network waits/retries bounded + cancellable; no unbounded buffers; never block renderer/FFmpeg | Channel/loop machinery is Task 3's worker (D-5/D-6); Task 2 delivers the seam a bounded worker will drive — see §1.5 |
| Types stay in `src/youtube/`; cross-boundary output = app-level values + typed error contract | §1.4 module boundary |

**Binding reconciliation (record D-4 wins over loose phrasing):** the mapping layer returns the youtube-owned carrier `ChatTextMessage`; `src/youtube/` stays free of a `crate::chat` dependency (youtube/mod.rs boundary rule). The `ChatMessage` construction via `ChatMessage::new/with_message_id/with_timestamp` (and the `#[allow(dead_code)]` retirement on `with_timestamp`) is Task 3 (record §4). Task 2 lands the parsing helper `parse_rfc3339_seconds` producing the epoch value the Task-3 consumer will attach.

---

## 1. Binding design transcription (from the Task-1 record; NOT re-adjudicated)

1.1. **D-1/D-3 wire form (O7 `liveChatMessages.list`):** `GET https://www.googleapis.com/youtube/v3/liveChat/messages`; query params in FIXED order `liveChatId=<encoded>` , `maxResults=200`, `part=snippet,authorDetails,id`, and `pageToken=<encoded>` appended LAST only when a token exists; NOT sent: `hl`, `profileImageSize`, no `key=` (Bearer only). GET keeps the no-body `call()` path.

1.2. **D-2/O6 resolution:** `GET https://www.googleapis.com/youtube/v3/liveBroadcasts` with `part=snippet&id=<broadcast_id>`; read `snippet.liveChatId` of the FIRST item; bounded budget 6 attempts × 5 000 ms (`CHAT_ID_RESOLUTION_ATTEMPTS = 6`, `CHAT_ID_RESOLUTION_INTERVAL_MS = 5_000`); transient (Network/5xx/BackendUnavailable/MalformedResponse) consumes an attempt, terminal classes abort immediately; absence after budget ⇒ typed `ApiNotFound { resource: "live chat" }` at this module's boundary (the worker's ChatUnavailable funnel is Task 3); a stop request during the wait ⇒ `LiveApiError::WaitCancelled` (existing variant).

1.3. **D-4 skip/accept policy** (per item, in order):
   1. `snippet.type` != `textMessageEvent` ⇒ SKIP (tombstone, chatEndedEvent, all super/membership/gift/ban/poll types, and ANY unknown string).
   2. missing/blank `snippet.textMessageDetails.messageText` ⇒ SKIP (never ordinary chat).
   3. author = `authorDetails.displayName` if present and non-blank, else `authorDetails.channelId` if present and non-blank, else SKIP.
   4. accepted ⇒ `ChatTextMessage { author, text, message_id: Option<String>, published_epoch: Option<u64> }`; `message_id` verbatim when `items[].id` present; `published_epoch` = `parse_rfc3339_seconds(published_at)` — `None` when absent or unparseable.
   5. `chatEndedEvent` item ⇒ skipped as a message AND marks the page `chat_ended`.

1.4. **D-5 dedup window (Task-2-owned TYPE only):** `ChatDedupWindow` — `VecDeque<String>`, capacity `CHAT_DEDUP_WINDOW = 512`; `record(&mut self, id: &str) -> bool` returns `true` exactly when the id was UNSEEN (then bounds: push_back; if len > capacity pop_front — "last 512"). The worker's per-session ownership is Task 3. Ordering: page-level mapping preserves items[] API order (oldest→newest); cross-page order is the token progression trait of the worker (Task 3).

1.5. **D-6/D-7 error classes:** `classify_api_error` additions ONLY: `403 liveChatDisabled` → NEW `LiveApiError::ChatDisabled`; `403 liveChatEnded` → NEW `LiveApiError::ChatEnded`; `404 liveChatNotFound` → EXISTING `ApiNotFound { resource: "live chat" }` (new constant + arm); `403 forbidden` → EXISTING `InsufficientPermissions` (NEW arm — the current fallback for bare `forbidden` is `ApiRejected`; D-7 explicitly demands the mapping. Guard in §7.2 against existing pins); `403 rateLimitExceeded` → EXISTING `QuotaLike` (already mapped — no code change). Variant count: **15 → 17** (all 15 pre-existing variants verbatim + the 2 additions; nothing removed).

1.6. **D-11 dependencies:** ZERO new. `Cargo.toml`/`Cargo.lock` diff EMPTY (serde/serde_json/ureq/std only).

1.7. **D-12 constants (chat-side only in Task 2):** `MAX_CHAT_RESULTS = 200` (live_chat_models), `CHAT_ID_RESOLUTION_ATTEMPTS = 6`, `CHAT_ID_RESOLUTION_INTERVAL_MS = 5_000` (live_chat_resolution), `CHAT_DEDUP_WINDOW = 512` (live_chat_dedup). DEFAULT_CHAT_POLL_INTERVAL_MS / sleep slices / backoff caps are worker constants — Task 3.

1.8. **D-13 secret-safety:** ZERO tracing/logging calls in ANY Task-2 module. No `Display` on any new type (Debug derive only; nothing prints it in Task 2). The live chat ID is a NON-secret operator diagnostic (record PIN) but still MUST NOT appear in any log line this task (no logging exists at all). Message text/authors/message ids of user messages never enter error payloads: arms added to `classify_api_error` carry only the server `reason` string (the existing redaction contract).

1.9. **D-14 test posture:** all offline; fakes + `127.0.0.1` loopback; zero Google contact; no real credentials/broadcasts; loopback servers confined to the transport test files.

---

## 2. New/edited files (exact inventory)

### 2.1 New production modules (registered in `src/youtube/mod.rs`)

| File | Contents |
| --- | --- |
| `src/youtube/live_chat_models.rs` | Provider serde structs + `ChatListCommand` + `MAX_CHAT_RESULTS` + the two path builders |
| `src/youtube/api_chat_transport.rs` | The `LiveChatTransport` trait (mockable seam) — mirror of `api_transport.rs`'s shape |
| `src/youtube/google_live_chat_transport.rs` | Concrete `impl LiveChatTransport for GoogleLiveApiTransport` (reuses the shared request engine) |
| `src/youtube/live_chat_resolution.rs` | O6 `resolve_live_chat_id` + resolution constants |
| `src/youtube/live_chat_mapping.rs` | `ChatTextMessage`, page mapping, skip policy, `parse_rfc3339_seconds` |
| `src/youtube/live_chat_dedup.rs` | `ChatDedupWindow` + `CHAT_DEDUP_WINDOW` |

### 2.2 Edited production files

| File | Change |
| --- | --- |
| `src/youtube/live_error/mod.rs` | + `ChatDisabled`, `ChatEnded` variants (15 → 17) + module-doc paragraph |
| `src/youtube/live_error/classify.rs` | + 403 arms `liveChatDisabled`/`liveChatEnded`, 403 arm `forbidden`, 404 arm `liveChatNotFound` (with `LIVE_CHAT_RESOURCE` constant) |
| `src/youtube/live_error/display.rs` | + 2 match arms + 2 helpers (exact lines in §3.2) |
| `src/youtube/live_models/broadcast_resource.rs` | `BroadcastSnippet` gains `pub(crate) live_chat_id: Option<String>` (documented field consumed by O6; it also widens `BroadcastSnippet` itself? NO — `LiveBroadcast::snippet` stays `pub(super)`; the field reaches the sibling consumer through... **see §3.5 visibility ruling** |
| `src/youtube/google_live_api_transport.rs` | TWO visibility widenings ONLY: `fn request` → `pub(super)`, `fn parse_resource_body` → `pub(super)` (behavior byte-zero change) |
| `src/youtube/mod.rs` | module doc phase-06 paragraph + 6 `mod` decls (with the staging allows prescribed in §3.8) + re-exports (§3.9) |

### 2.3 New test rosters (exact file names)

| File | Tests |
| --- | --- |
| `src/youtube/live_chat_models_tests.rs` | 6 decode pins |
| `src/youtube/live_chat_mapping_tests.rs` | 10 mapping pins |
| `src/youtube/live_chat_dedup_tests.rs` | 3 pins |
| `src/youtube/live_chat_resolution_tests.rs` | 6 pins |
| `src/youtube/live_chat_wire_tests.rs` | 5 loopback pins |
| `src/youtube/live_error_tests.rs` | +4 classify pins (APPEND to existing file) |
| `src/youtube/live_error_display_tests.rs` | +2 Display pins (APPEND) |
| `src/youtube/live_chat_test_fixtures.rs` | shared `#[cfg(test)]` fixtures (0 tests) |

Projected total: **391 + 36 = 427 tests** — PROJECTED arithmetic only; the OBSERVED gate count governs (Phase-05 arithmetic precedent).

---

## 3. Code specifications (snippets frozen where wire-shapes matter)

### 3.1 `src/youtube/live_chat_models.rs`

```rust
/// Poll budget of ONE list call (documented minimum, S2; per-call 5-unit cost).
pub(crate) const MAX_CHAT_RESULTS: u64 = 200;
const CHAT_MESSAGES_RESOURCE: &str = "liveChatMessageList";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageList {
    #[serde(default)]
    pub(crate) items: Vec<LiveChatMessageRecord>,
    pub(crate) next_page_token: Option<String>,
    pub(crate) polling_interval_millis: Option<u64>,
    pub(crate) offline_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageRecord {
    pub(crate) id: Option<String>,
    pub(crate) snippet: Option<LiveChatMessageSnippet>,
    pub(crate) author_details: Option<LiveChatAuthorDetails>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveChatMessageSnippet {
    pub(crate) message_type: Option<String>,   // #[serde(rename = "type")] applied
    pub(crate) published_at: Option<String>,
    pub(super) text_message_details: Option<TextMessageDetails>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextMessageDetails {
    pub(crate) message_text: Option<String>,
}
...
```

- `LiveChatMessageSnippet` carries `#[serde(rename = "type")] pub(crate) message_type: Option<String>` (the reserved word; documented values per record §1.5; unknown strings tolerated — the skip policy handles them).
- `LiveChatAuthorDetails { pub(crate) display_name: Option<String>, pub(crate) channel_id: Option<String> }` (camelCase; unknown author fields ignored — no `page`/`profile`/`isVerified` fields are mapped).
- Tolerant model (Phase-05 frozen precedent): NO `deny_unknown_fields`; `items` defaults empty; a missing `items` key deserializes to an empty list.
- `ChatListCommand { pub(crate) live_chat_id: String, pub(crate) page_token: Option<String> }` (zero derive needs beyond `Clone` — derive `Clone`).
- Path builders (query order FIXED per D-3; `encode_query_value` reused verbatim from `api_request`):

```rust
pub(super) fn chat_messages_path(command: &ChatListCommand) -> String {
    let mut path = format!(
        "liveChat/messages?liveChatId={}&maxResults={}&part=snippet,authorDetails,id",
        encode_query_value(&command.live_chat_id),
        MAX_CHAT_RESULTS,
    );
    if let Some(token) = command.page_token.as_deref() {
        path.push_str("&pageToken=");
        path.push_str(&encode_query_value(token));
    }
    path
}

pub(super) fn broadcast_chat_lookup_path(broadcast_id: &str) -> String {
    format!(
        "liveBroadcasts?part=snippet&id={}",
        encode_query_value(broadcast_id)
    )
}
```

- `use super::api_request::encode_query_value;` (already `pub(super)`).

### 3.2 Error variant + Display lines (exact)

`live_error/mod.rs` — append to the enum (place AFTER `DestinationMissing`, bottom of the list):

```rust
    /// 403 liveChatDisabled (record D-7): chat is not enabled on the
    /// broadcast — a terminal retrieval failure class.
    ChatDisabled,
    /// 403 liveChatEnded (record D-6/D-7): the chat is no longer live —
    /// the worker's GRACEFUL end-of-chat signal, never a fatal.
    ChatEnded,
```

`live_error/display.rs` — match arms + helpers (exact lines, doctor the same style as neighbors):

```rust
Self::ChatDisabled => Self::display_chat_disabled(formatter),
Self::ChatEnded => Self::display_chat_ended(formatter),
...
fn display_chat_disabled(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        formatter,
        "live chat is disabled for this broadcast; enable chat and reuse its id"
    )
}

fn display_chat_ended(formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "the live chat has ended")
}
```

`live_error/classify.rs` — constants + arms:

```rust
const LIVE_CHAT_RESOURCE: &str = "live chat";
```
- in `classify_forbidden`: arms `"liveChatDisabled" => LiveApiError::ChatDisabled` and `"liveChatEnded" => LiveApiError::ChatEnded` BEFORE the rate-limit arm, plus the NEW arm `"forbidden" => LiveApiError::InsufficientPermissions` (D-7's required addition).
- in `classify_not_found`: arm `"liveChatNotFound" => LiveApiError::ApiNotFound { resource: LIVE_CHAT_RESOURCE.to_string() }`.

### 3.3 `src/youtube/api_chat_transport.rs` — the seam

```rust
/// The chat retrieval seam (record D-7): both chat wire calls (O6 lookup
/// and O7 list) return `Result<.., LiveApiError>` so the app funnels ONE
/// typed source; no streaming/concurrency semantics enter the trait.
pub(crate) trait LiveChatTransport {
    /// O6 documented field read: the FIRST item's `snippet.liveChatId`;
    /// `Ok(None)` only for a verifiably empty list answer.
    fn lookup_broadcast_chat_id(
        &self,
        broadcast_id: &str,
    ) -> Result<Option<String>, LiveApiError>;

    /// O7 `liveChatMessages.list` (D-3 wire form); the typed error covers
    /// every §1.6 lifecycle class.
    fn list_messages(&self, command: &ChatListCommand)
        -> Result<LiveChatMessageList, LiveApiError>;
}
```

Module docs inherit the standing offline-test rule + the record citations (S1/S2/S5 + the research record's own URI).

### 3.4 `src/youtube/google_live_chat_transport.rs` — concrete impl

Implements `LiveChatTransport for GoogleLiveApiTransport` by REUSING the shared engine:

```rust
impl LiveChatTransport for GoogleLiveApiTransport {
    fn lookup_broadcast_chat_id(
        &self,
        broadcast_id: &str,
    ) -> Result<Option<String>, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::getting(
            broadcast_chat_lookup_path(broadcast_id),
        ))?;
        let list = parse_resource_body(&reply, "broadcastChatLookup")?;
        Ok(list
            .items
            .first()
            .and_then(|broadcast| broadcast.snippet.as_ref())
            .and_then(|snippet| snippet.live_chat_id.clone()))
    }

    fn list_messages(
        &self,
        command: &ChatListCommand,
    ) -> Result<LiveChatMessageList, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::getting(chat_messages_path(command)))?;
        parse_resource_body(&reply, CHAT_MESSAGES_RESOURCE)
    }
}
```

(uses `use super::google_live_api_transport::{GoogleLiveApiTransport, parse_resource_body};` — both hardened by the `pub(super)` widenings, §3.5; all other imports follow google_live_api_transport.rs's import block.)

`#[cfg(test)] #[path = "../live_chat_wire_tests.rs"] mod live_chat_wire_tests;` registered at the file foot (the sibling-roster convention; all other Task-2 rosters are registered from their owning files the same way).

### 3.5 Visibility widenings (minimal, zero behavior)

- `google_live_api_transport.rs`: `fn request` → `pub(super) fn request`; free `fn parse_resource_body` → `pub(super) fn parse_resource_body`. Module doc gains ONE sentence: "The engine (+ parse helper) is shared with the concrete chat transport (Phase 06); visibility only."
- `live_models/broadcast_resource.rs`: `BroadcastSnippet { ... pub(crate) live_chat_id: Option<String> }`. The sibling chat consumer reads it via `list.items.first().and_then(...)`: `LiveBroadcast::snippet` is `pub(super)` = visible within `crate::youtube` and its descendants ⇒ `google_live_chat_resolution/transport` (youtube children) can traverse it. If the compiler disallows the traversal at any arm, widen ONLY the exact field there (`snippet` → `pub(crate)`) — compiler-arbitrated prevention (D-T2-14 precedent), record the widening in the cycle summary.
- `live_chat_messages_path`/`broadcast_chat_lookup_path` are `pub(super)` in `live_chat_models.rs` (consumed only by the sibling transport file + its loopback tests).

### 3.6 `src/youtube/live_chat_resolution.rs`

```rust
const CHAT_ID_RESOLUTION_ATTEMPTS: u64 = 6;
const CHAT_ID_RESOLUTION_INTERVAL_MS: u64 = 5_000;

pub(crate) fn resolve_live_chat_id(
    transport: &dyn LiveChatTransport,
    broadcast_id: &str,
) -> Result<String, LiveApiError> {
    resolve_live_chat_id_with_interval(transport, broadcast_id, CHAT_ID_RESOLUTION_INTERVAL_MS)
}
```

- `resolve_live_chat_id_with_interval(transport, broadcast_id, interval_ms)` — the cfg-split core (the D-I zero-sleep re-injection precedent, research record D-14): TEST builds pass a caller-chosen interval; production keeps the 5 000 ms default. Signature = 3 args (exceeds the 2-arg rule; same sanctioned exception family as record D-T3-15's readiness constructors). Loop sleeps in 1 000 ms slices checking the stop flag via... none exists here — Task 3 injects the WaitCanceller? DECISION: the resolution signature takes NO canceller; sleeps use the raw interval (bounded by 6 × 5 s = 30 s worst case; the WAIT itself is short and startup-phase only). Cancellation pin (§7.2 roster) applies to the typed-error classes, NOT to a stop flag (the run-level wiring and its flag handling belong to Task 3's stage 11). If ANY reviewer later demands a canceller parameter, that is a Task-2 amendment requiring a caller ruling — NOT implementer latitude.
- Classification matrix inside the loop (single-section booleans; extract `is_transient` helper):

```rust
fn is_transient(error: &LiveApiError) -> bool {
    matches!(error, LiveApiError::Network { .. } | LiveApiError::BackendUnavailable { .. } | LiveApiError::MalformedResponse { .. })
}
```
Wait disallowed compound: the two boolean-condition rules require single-section conditions for `if/while` STATEMENTS; this is a `matches!` expression bound to a variable inside a match arm — extract to a helper method returning `bool` per arm. Prescribe: each `LiveApiError` class check written as ONE named helper call per condition site (`is_transient_error(&e)` used inside `if is_transient_error(...) { ... }` — single section).
- Loop: for attempt in 1..=attempts { match transport.lookup_broadcast_chat_id(broadcast_id) { Ok(Some(id)) → return Ok(id); Ok(None) → sleep-or-repeat; Err(e) if is_transient_error(&e) → sleep-or-repeat; Err(e) → return Err(e) } } ; after the final attempt => `Err(LiveApiError::ApiNotFound { resource: LIVE_CHAT_RESOURCE.to_string() })`. Reference `CHAT_ID_RESOLUTION_ATTEMPTS` as the budget constant.
- Module doc records D-2's rationale + the "absence after budget is a typed fatal with actionable diagnostic (the resource Display line)" and the D-13 non-secret note.

### 3.7 `src/youtube/live_chat_mapping.rs`

```rust
pub(crate) struct ChatTextMessage {
    pub(crate) author: String,
    pub(crate) text: String,
    pub(crate) message_id: Option<String>,
    pub(crate) published_epoch: Option<u64>,
}

pub(crate) struct ChatPageOutcome {
    pub(crate) messages: Vec<ChatTextMessage>,
    pub(crate) chat_ended: bool,
}

pub(crate) fn map_page(list: &LiveChatMessageList) -> ChatPageOutcome { ... }   // 2-arg-free, order-preserving
fn map_record(record: &LiveChatMessageRecord) -> Option<ChatTextMessage> { ... } // D-4 steps 1..4 in order
fn parse_rfc3339_seconds(value: &str) -> Option<u64> { ... }                     // documented UTC forms
```

- `map_record` reads `snippet.message_type.as_deref()`; constants `TEXT_MESSAGE_TYPE = "textMessageEvent"`, `CHAT_ENDED_TYPE = "chatEndedEvent"` (module-private).
- Blank/missing text and blank/missing author resolution are handled by small named predicates (`has_message_text(snippet)`, `resolve_author(record) -> Option<String>`) — single-section booleans.
- `parse_rfc3339_seconds`: accepts `"YYYY-MM-DDThh:mm:ssZ"` AND `"YYYY-MM-DDThh:mm:ss.fffZ"` (documented ISO 8601 UTC forms, record D-4/D-1.5); rejection cases: no `Z` terminator, wrong component widths/day-month/02-29 non-leap, out-of-range fields, >3 fractional digits ⇒ `None` (never panic; no external time crate per D-11; this is the hand-rolled reverse of the existing civil-from-days formatter family). Use the classic civil-days algorithm (`days_from_civil`-style in pure integer arithmetic with checked/typed-narrow paths — every parse step returns `Option<u64>`; NO `.expect()`).
- Page-level end detection: `chat_ended` = (any item with type `chatEndedEvent`) OR (`list.offline_at.is_some()`); single-section conditions via helpers (`marks_ended(record)` / `list.shows_offline()`).
- Id preserved verbatim: `record.id.clone()` when `Some`.

### 3.8 `src/youtube/live_chat_dedup.rs`

```rust
const CHAT_DEDUP_WINDOW: usize = 512;

/// Provider-ID dedup window (record D-5): the LAST `CHAT_DEDUP_WINDOW`
/// seen ids; the worker owns a per-session instance (Task 3). Tombstones
/// and gift-event re-use never conflict because the skip policy runs
/// BEFORE the window inside the worker flow.
pub(crate) struct ChatDedupWindow {
    seen: VecDeque<String>,
}
impl ChatDedupWindow {
    pub(crate) fn new() -> Self { ... }                 // empty
    pub(crate) fn record(&mut self, id: &str) -> bool { // true = unseen, remembered
        ...
    }
}
```

### 3.9 `src/youtube/mod.rs` registration

- Module doc: append ONE compact paragraph (≤ 10 lines) — Phase-06 chat layer landed per record D-1..D-3/D-4/D-7; the worker consuming it is Task 3; zero new deps (D-11); the api_request wire form for chat is pinned by the chat wire roster.
- Decls (alphabetical placement in the existing block):

```rust
mod api_chat_transport;
mod google_live_chat_transport;
mod live_chat_dedup;
mod live_chat_mapping;
mod live_chat_models;
mod live_chat_resolution;
```

- Staging (D-T2-14 needle: compiler-arbitrated; PRESCRIBED below so the implementer lands the allows on the FIRST pass and only THOSE — retire when Task 3's consumption lands):

```rust
// Wired by Task 3's chat worker (record §4); retire when consumption lands.
#[allow(dead_code)]
mod api_chat_transport;
#[allow(dead_code)]
mod live_chat_mapping;
#[allow(dead_code)]
mod live_chat_models;
#[allow(dead_code)]
mod live_chat_dedup;
#[allow(dead_code)]
mod live_chat_resolution;
// Implemented against the concrete transport; same Task-3 retirement.
#[allow(dead_code)]
mod google_live_chat_transport;
```

(Every Task-2 production module is dead until the Task-3 worker roots the chain; the `ApiNotFound`-resource/`ChatDisabled`/`ChatEnded` classify arms + Display arms ARE consumed by the classify tests, so live_error files stay allow-free.)

- Re-exports (the Task-3 consumption surface):

```rust
pub(crate) use api_chat_transport::LiveChatTransport;
pub(crate) use live_chat_dedup::ChatDedupWindow;
pub(crate) use live_chat_mapping::{map_page, ChatPageOutcome, parse_rfc3339_seconds, ChatTextMessage};
pub(crate) use live_chat_models::{chat_messages_path, ChatListCommand, LiveChatMessageList};
pub(crate) use live_chat_resolution::resolve_live_chat_id;
```

(If rustfmt orders the braces differently, the CONTENT stays as enumerated; the single-line brace per use family is fine. `map_page`/`parse_rfc3339_seconds` naming per §3.7.)

### 3.10 Shared chat fixtures `src/youtube/live_chat_test_fixtures.rs`

`#[cfg(test)]`-gated helper module (registered from `live_chat_resolution.rs` and the wire roster via `#[cfg(test)] #[path]`):

```rust
pub(super) const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";
pub(super) const LIVE_CHAT_ID_FIXTURE: &str = "live-chat-id-fixture";
pub(super) const ACCESS_TOKEN_FIXTURE: &str = "access-token-fixture";

pub(super) struct ScriptedLiveChatTransport {   // consumed ONLY by resolution tests
    pub(super) lookup_results:
        RefCell<VecDeque<Result<Option<String>, LiveApiError>>>,
    pub(super) list_results:
        RefCell<VecDeque<Result<LiveChatMessageList, LiveApiError>>>,
}
impl LiveChatTransport for ScriptedLiveChatTransport { ... }
```

- Queues behind `next_result(&mut vecdeque)` lift (mirror live_test_fixtures.rs, but NO reuse-of-import: the Phase-05 fixtures file stays byte-untouched).
- Canned JSON builders: `live_chat_list_body(&str json-items)` free fns used by the wire roster.
- NO fixture secret shapes (`*-fixture` strings only).

---

## 4. Loopback wire pins (§ roster shape, mirror of the bind wire pattern)

`src/youtube/live_chat_wire_tests.rs` (registered from google_live_chat_transport.rs) hosts a NEW loopback capture server (same head-recording approach as `EnforcingBindServer`, but capturing the FULL head so `Authorization` and query lines are assertable; name it `ChatWireServer`):

wire rosters:

1. `chat_list_sends_bearer_only_with_documented_query` — request line `GET /youtube/v3/liveChat/messages?liveChatId=live-chat-id-fixture&maxResults=200&part=snippet,authorDetails,id` exactly; `Authorization: Bearer access-token-fixture` present; NO `key=` anywhere in the head.
2. `chat_list_appends_page_token_only_when_present` — with `page_token = Some("token-1-fixture")` the request line ends `&pageToken=token-1-fixture`; without → `pageToken=` absent.
3. `chat_list_response_decodes_page_fields` — canned body: items (1 valid text record) + `nextPageToken: "next-token-fixture"` + `pollingIntervalMillis: 13000` + `offlineAt` ABSENT ⇒ parsed fields asserted (interval Some(13000), token Some, offline False). Assert a 200 reply; the Ok(None)/empty path separate.
4. `chat_list_malformed_body_is_typed_error` — non-JSON body ⇒ `MalformedResponse { field: "liveChatMessageList" }` (never panic, never silent).
5. `chat_disabled_body_classifies_typed_and_broadcast_lookup_reads_chat_id` — compound? SPLIT into TWO tests per single-assertion hygiene (6 total; projected delta becomes +37 → 428 — rule: observed governs):
   - `broadcast_chat_lookup_reads_snippet_live_chat_id` (canned `{"items":[{"snippet":{"liveChatId":"live-chat-id-fixture"}}]}` → Some; empty `{"items":[]}` → None through a second server — server must accept TWO sequential requests or the two calls use one server each; prescribe: helper builds per-call server).
   - `chat_disabled_error_body_classifies_as_live_chat_disabled` (status 403 + body `{"error":{"errors":[{"reason":"liveChatDisabled"}]}}` ⇒ `ChatDisabled`).

---

## 5. Test specs (mapping + dedup + resolution, exact names)

### 5.1 `live_chat_mapping_tests.rs`

1. `text_message_maps_full_shape_with_id_and_epoch_timestamp`
2. `missing_display_name_falls_back_to_channel_id`
3. `missing_both_author_fields_is_skipped`
4. `non_text_event_types_are_skipped_including_tombstone_and_unknown` (superChatEvent + tombstone + an invented "futureEvent" string)
5. `malformed_text_message_payload_is_skipped_not_mapped_as_ordinary` (type=textMessageEvent but messageText absent/None)
6. `blank_or_missing_message_text_is_skipped`
7. `page_mapping_preserves_api_order` (items in order [a,b,c] → carriers in the same order)
8. `chat_ended_event_marks_page_ended_without_a_message`
9. `offline_at_marks_page_ended`
10. `absent_or_unparseable_published_at_maps_without_timestamp`
11. `rfc3339_parsing_covers_documented_utc_forms` (both `...ssZ` and `...ss.fffZ`; invalid day/2-29/missing-Z → None)

### 5.2 `live_chat_dedup_tests.rs`

1. `dedup_window_reports_unseen_ids_then_repeated_ids_as_seen`
2. `dedup_window_evicts_the_oldest_id_beyond_512`
3. `dedup_window_tolerates_re_seen_ids_of_skipped_types` (record D-5's re-use tolerance ≙ contract-level: window is pure id-store; skip policy ordering is asserted at mapping level and the worker flow is Task 3 — test: record("x") twice → false second time; window untouched policy documented)

### 5.3 `live_chat_resolution_tests.rs` (fixtures-fed; zero network)

1. `resolution_returns_the_broadcast_live_chat_id`
2. `resolution_polls_until_the_chat_id_appears` (absents first, success last — attempts < budget via test interval)
3. `resolution_absent_after_budget_is_typed_not_found`
4. `resolution_transient_failures_consume_attempts_then_recover` (Network then success)
5. `resolution_terminal_errors_abort_immediately` (InsufficientPermissions NOT retried — operations count == 1)
6. `resolution_wait_cancelled_is_not_translucently_retried` — a scripted `WaitCancelled` result aborts immediately and propagates (single call)

### 5.4 `live_chat_models_tests.rs`

1. `full_list_response_decodes_token_interval_and_items`
2. `empty_items_list_decodes_without_records`
3. `missing_items_key_decodes_as_empty_items` (Phase-05 tolerant-model precedent)
4. `text_message_record_decodes_type_rename_and_text` (`"type": "textMessageEvent"`)
5. `author_details_decodes_display_name_and_channel_id`
6. `unknown_type_strings_decode_verbatim_for_the_skip_policy`

### 5.5 `live_error_tests.rs` (+4, APPEND)

1. `chat_disabled_reason_maps_typed` (403/liveChatDisabled)
2. `chat_ended_reason_maps_typed` (403/liveChatEnded)
3. `live_chat_not_found_reason_maps_resource_live_chat` (404/liveChatNotFound → `ApiNotFound { resource: "live chat" }`)
4. `bare_forbidden_maps_insufficient_permissions` (403/forbidden) — GUARD: if an EXISTING Phase-05 pin asserts 403/`forbidden` → `ApiRejected`, STOP and report to the caller (a pre-frozen pin governs over this plan's transcription; do NOT delete the pin silently).

### 5.6 `live_error_display_tests.rs` (+2)

1. `chat_disabled_display_is_exact_line`
2. `chat_ended_display_is_exact_line`

Rosters are wired ONLY via the `#[cfg(test)] #[path = "..."] mod X;` foot pattern (existing convention). No `tests/` directory files.

---

## 6. Verification (4.2 implementer runs, ALL via Alpine VM MCP)

- Pre: `git status --porcelain` → tree clean; branch + HEAD pinned (`95d791d`, v0.12.0). If mismatched → STOP, report.
- Targeted spot runs (before the gate; any failure → fix → re-run):
  - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked live_chat` (all chat rosters),
  - `... run --rm rust cargo test --locked live_error` (classification + Display pins, plus any pre-existing live_error rosters),
  - `... run --rm rust cargo clippy --locked -- -D warnings` (staging-neutral; the prescribed allows SHOULD hold; if clippy demands MORE allows/modules, apply ONLY the exact allow the compiler names, with a comment naming Task 3, and record it),
  - `... run --rm rust cargo fmt --check` (mechanical rustfmt normalization permitted on the touched files only).
- FULL gate on the final tree: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0 + `ALL CHECKS PASSED` + observed count (projected 427; OBSERVED GOVERNS; report exact number + log path).
- Regression guards (grep + gate): demo/auth/verifier/credential surfaces untouched (`git diff --stat` shows only §2 files); `key=` still only the pre-existing doc-context hits; zero `ya29`/`AIza`; zero `allow(` additions beyond §3.8's prescribed module-staging lines; `rate_limit_exceeded`-style Display needles unchanged.
- Secret audit: `git diff | grep -c "ya29\|AIza\|Bearer access"` → only fixture-strings (`access-token-fixture`) inside tests may appear; no `rtmps://`, no real-shaped tokens.
- NO builds (builds belong to Task 4 per the phase convention/TODO split).

---

## 7. Git handling (4.2)

1. State check (§6 pre) — branch `feat/phase06-youtube-live-chat`, clean, HEAD `95d791d`.
2. Commit the plan file ONLY (`chore: track the phase06 task2 live chat client mapping plan`).
3. Implement §2/§3/§5 exactly.
4. Re-verification chain (§6).
5. Commit code+tests:
   `git add src/youtube/...` (the §2 files, plus mod.rs and the live_error trio) → commit message:
   `feat: add youtube live chat client models mapping and error classes`
6. Final check: `git status --porcelain` → empty; `git show --stat --oneline HEAD` lists exactly the feat files. NO push (step 5 of the workflow owns pushes), no merge, no TODO `[DONE]` marks.

## 8. Byte-stability ledger (regression locks Task 2 must preserve)

- O1–O5 request forms, `bodyless_post_or_get`, `respond`, the verifiers, auth flow, demo source/schedule, streaming vectors: UNTOUCHED.
- Phase-05 `classify_api_error` arms: unchanged except the THREE additions in §3.2; any existing classification pin invalidated (guarded in §5.5.3) → STOP-and-report.
- Exit codes, single-`error:` contract, shutdown ordering — not touched in Task 2.
- `ChatStore`/`ChatMessage`/`renderer`/`streaming` files: zero edits (youtube stays chat-free).

## 9. Later sub-step applicability

- 4.3 review/simplification: REQUIRED (new src files); reviewers bound to §2's inventory; contested structural re-picks → caller.
- 4.4 documentation: NOT REQUIRED in Task 2 (all phase docs + `.agent/project-structure.md`/tech/context/README belong to Task 4 per the global plan; docs-specialist returns "not required", drift-check only).
- 4.5b adherence: architector re-check against THIS plan + the record §4.
- 4.6 completion: implementer marks Task 2 `[DONE]` in the TODO (+ any touched acceptance criteria in a second pass), commits.

## 10. Explicitly NOT done by Task 2 (flagged for caller/later tasks)

- NO worker thread, channel, baseline/discard, sleep-hooked pacing, backoff loops, shared terminal cell, stage-11 wiring, source replacement, trait-widening of `MessageSource`, `ChatMessage::new/with_*` call sites, the with_timestamp allow retirement — ALL Task 3.
- NO user/operator docs, structure-map, context/tech updates, completion report, builds, or manual E2E — ALL Task 4.
- NO Cargo.toml/lockfile edits, no version change beyond the landed 0.12.0, and no new dependencies.

## 11. Footer

Plan: `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`
Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — Task 2 only.
Record: `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md`; global plan `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`.
Produced 2026-10-25 by Task-2 4.1b architector.
