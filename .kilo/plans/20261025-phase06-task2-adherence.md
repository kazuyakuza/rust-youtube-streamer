# Phase 06, Task 2 — Overall Plan Adherence Report (step 4.5b, architector)

> Task: **2. YouTube Live Chat API Client and Message Mapping** (TODO `.agent/todos/20261025/20261025-todo-2.md`, Task 2 bullets only).
> Repo state at review: branch `feat/phase06-youtube-live-chat`, HEAD `f369410`, tree clean, version 0.12.0.
> Gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` exit 0, **429 passed / 0 failed**, `ALL CHECKS PASSED` — re-observed first-hand on the VM from `logs/checks/20261026T031702Z.log` (four PASS lines: fmt-check/check/test/clippy).
> Binding documents: global plan `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`; per-task plan `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`; upstream design record `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (D-1..D-14 + §4 inherited obligations); fix plan `.kilo/plans/20261025-phase06-task2-review-fix.md`; simplification plan `.kilo/plans/20261025-phase06-task2-simplification.md`; the 4.2 caller rulings (splits `live_chat_rfc3339`/`live_chat_wire_checks_tests`, `mod.rs` condense to 200, 11 mapping tests kept, `all_error_variants` → 17 amendment, visibility fixes).

---

## 1. Verdict

**ADHERENT WITH ACCEPTABLE DEVIATIONS — zero non-adherent items.**

Every TODO Task-2 bullet is satisfied by the landed code at the boundary the binding record and per-task plan prescribed for Task 2 (the worker/channel/pacing machinery is Task 3 by design — record §4, plan §0 mapping table). All deviations found are compiler-arbitrated or caller-ruled and are enumerated in §6. The single 4.3-fix flag is adjudicated a **plan-text nit** (§5); the landed code is correct.

---

## 2. Landed evidence — commits, diffstat, file inventory

### 2.1 Commit chain (Task-2 span `95d791d..f369410`)

| Commit | Content | Verified |
| --- | --- | --- |
| `baf260e` | `chore: track the phase06 task2 live chat client mapping plan` — the per-task plan file only (539 lines) | ✓ `git show --stat` |
| `ac7385f` | `feat: add youtube live chat client models mapping and error classes` — **22 files**, 1398 insertions / 6 deletions | ✓ `git show --stat` |
| `f369410` | `refactor: apply task2 review fixes and mapping simplification` — **5 files**: the 2 fix/simplification plan files + `live_chat_mapping.rs` (S-1 fold), `live_chat_rfc3339.rs` (review fixes 2–3), `live_error_display_tests.rs` (fix 1) | ✓ `git show --stat` |

### 2.2 Branch-span diffstat vs the plan's file inventory

`git diff --stat 95d791d..f369410 -- src/` = exactly **22 files, all under `src/youtube/`**, 1400 insertions / 6 deletions net (the fix commit's src delta folded in). Composition:

- Plan §2.1 new production modules (6): `live_chat_models.rs`, `api_chat_transport.rs`, `google_live_chat_transport.rs`, `live_chat_resolution.rs`, `live_chat_mapping.rs`, `live_chat_dedup.rs` — all landed. **Plus** `live_chat_rfc3339.rs` (the caller-ruled sibling split of `parse_rfc3339_seconds`, registered from `live_chat_mapping.rs` via `#[path]`).
- Plan §2.2 edited production files (6): `live_error/mod.rs`, `live_error/classify.rs`, `live_error/display.rs`, `live_models/broadcast_resource.rs`, `google_live_api_transport.rs`, `mod.rs` — all landed, diffs additions-only (§4 below).
- Plan §2.3 test rosters (8): `live_chat_models_tests.rs`, `live_chat_mapping_tests.rs`, `live_chat_dedup_tests.rs`, `live_chat_resolution_tests.rs`, `live_chat_wire_tests.rs`, `live_error_tests.rs` (+4 append), `live_error_display_tests.rs` (+2 append), `live_chat_test_fixtures.rs` — all landed. **Plus** `live_chat_wire_checks_tests.rs` (the caller-ruled wire-roster split carrying the six pins).

Branch-level check (`git diff --stat b75d914..f369410`): outside `src/youtube/` the span touches only plan/TODO docs, the step-3 version bump (`Cargo.toml`/`Cargo.lock` version line, commit `ad5986d` — pre-Task-2) and the 3 Task-2 plan-tracking files. **Zero Task-2 edits to `src/app`, `src/chat`, `src/renderer`, `src/streaming`, `config/`, `docs/`, TODO** — the §8 byte-stability ledger holds by direct diffstat evidence.

### 2.3 Dependency lock (D-11)

`git diff --stat 95d791d..f369410 -- Cargo.toml Cargo.lock` → **empty**. Zero new dependencies; serde/serde_json/ureq/std only. ✓

### 2.4 Gate arithmetic

391 baseline + 38 new tests = **429 observed**: models 6 + mapping 11 + dedup 3 + resolution 6 + wire checks 6 + live_error +4 + display +2. The plan's §2.3 projection of +36 (→427) understated two counts the plan itself corrected in its own §5 rosters and §4 split note (11 mapping names listed in §5.1 vs the "10" projection; the wire split's 6th test flagged in §4 as "projected delta becomes +37 → 428") — both were caller-ruled during 4.2; the plan's own "OBSERVED GOVERNS" rule applies. ADHERENT.

---

## 3. TODO Task-2 bullet verification matrix (bullet → landed evidence)

| # | TODO Task-2 bullet | Landed evidence | Verdict |
| --- | --- | --- | --- |
| 1 | Required models + mockable interfaces under `src/youtube/`, direct-ureq transport + typed-error conventions | `live_chat_models.rs` (tolerant serde models, missing-`items` → empty, unknown fields ignored), `api_chat_transport.rs` (`LiveChatTransport` trait, both calls `Result<_, LiveApiError>`), `google_live_chat_transport.rs` (concrete impl REUSING the Phase-05 shared request engine — `request` + `parse_resource_body`, same Bearer-only redirects-disabled ureq stack, status-first classification) | ADHERENT |
| 2 | Reuse the authorized token; never log token/auth headers/raw bodies/sensitive data | The concrete chat transport IS `GoogleLiveApiTransport` — the same instance the run runtime builds from the Phase-04/05 credential service; no second OAuth flow, no new credential store (grep: zero `crate::chat|renderer|streaming|app` imports; zero new auth/store code in the span). **Zero tracing/log macros in every Task-2 module** (grep-verified); no `Display` on any new type (Debug/Deserialize/Clone derives only); error payloads carry only the server `reason` string; wire pins assert `Bearer access-token-fixture` and `!head.contains("key=")` | ADHERENT |
| 3 | Resolve the chat ID from documented fields; absent/disabled chat ⇒ explicit typed error + actionable diagnostic | O6 `GET /youtube/v3/liveBroadcasts?part=snippet&id=<id>` reads the FIRST item's `snippet.liveChatId` (documented field, record D-2, citation S5); bounded budget 6 × 5 000 ms (`CHAT_ID_RESOLUTION_ATTEMPTS`/`CHAT_ID_RESOLUTION_INTERVAL_MS`); transient classes (Network/BackendUnavailable/MalformedResponse) consume an attempt, terminal classes abort immediately; absence after budget ⇒ `ApiNotFound { resource: "live chat" }` (Display: "YouTube live chat not found; deleted or never created"); disabled ⇒ `ChatDisabled` ("live chat is disabled for this broadcast; enable chat and reuse its id"); ended ⇒ `ChatEnded` | ADHERENT |
| 4 | Selected documented retrieval operation with correct params + decoding | O7 `GET /youtube/v3/liveChat/messages`, query order FIXED `liveChatId=<encoded>&maxResults=200&part=snippet,authorDetails&id`, `pageToken` appended LAST only when a token exists; NOT sent: `hl`, `profileImageSize`, `key=` (D-3, §1.1). Decoding pinned by 6 models tests + 6 loopback wire pins (request target byte-exact, Bearer header, no `key=`, pageToken rule, page fields, typed malformed) | ADHERENT |
| 5 | Map text-bearing messages to `ChatMessage` with ID + epoch timestamp; documented tested skip policy; malformed never ordinary chat | Binding reconciliation (plan §0, record §4/D-4): Task 2 maps to the youtube-owned carrier `ChatTextMessage { author, text, message_id, published_epoch }` — the `ChatMessage` construction (`new`/`with_message_id`/`with_timestamp`, where normalization runs) is Task 3's consumption point; `src/youtube/` stays free of `crate::chat`. Skip policy D-4 in order, 11 tested pins: non-`textMessageEvent` (incl. tombstone + unknown strings) skip; missing/blank text skips (`malformed_text_message_payload_is_skipped_not_mapped_as_ordinary`); author = displayName else channelId else skip; id verbatim; `published_epoch` = `parse_rfc3339_seconds` (epoch seconds = the model's unit); `chatEndedEvent` marks the page ended, never a chat line | ADHERENT (carrier conversion = the prescribed Task-3 boundary; recorded as carry-forward C-1) |
| 6 | Behavior defined: empty, dup/replayed IDs, ordering, continuation/stream reconnects, deleted/hidden, unsupported types, chat ended, permission, quota/rate, transient, malformed, cancellation | Empty pages decode as valid pages (models tests 2–3; the never-infer-end-from-emptiness rule is D-6/§1.3, honored by the worker in Task 3); dup/replayed IDs ⇒ `ChatDedupWindow` 512 with gift/tombstone re-use tolerance (3 tests + module docs); page ordering preserves `items[]` API order (test 7; cross-page order = worker trait, Task 3); continuation tokens decoded + wire-pinned (honor-loop = Task 3); tombstones skip; unsupported types skip; chat ended = page marker + `ChatEnded` graceful class; permission = bare `forbidden` → `InsufficientPermissions` + `liveChatDisabled` terminal; quota/rate = `rateLimitExceeded` → `QuotaLike` (pre-existing arm unchanged); transient consumes attempts (resolution tests 4); malformed ⇒ typed `MalformedResponse` (wire test 4, never silent); cancellation ⇒ `WaitCancelled` propagates verbatim, never retried (resolution test 6) | ADHERENT (worker-side behaviors assigned to Task 3 by the binding record — plan §0 rows 6–8) |
| 7 | Honor server-provided poll interval + continuation token; no busy-loop/invented interval | The seam **exposes** the server values: `LiveChatMessageList.polling_interval_millis` + `next_page_token` decoded and pinned (models test 1, wire test 3 at `pollingIntervalMillis: 13000`). The actual honor-loop is Task 3's worker (record D-6; plan §0 bullet-7 row: "Task 2 only decodes + pins the values"). No fixed interval invented anywhere in Task 2; the resolution module's 5 s constant is the D-2-prescribed O6 chat-id resolution budget, not a poll interval | ADHERENT (seam exposure verified; honor-loop = Task 3 carry-forward C-3) |
| 8 | Bounded waits/cancellability; no unbounded buffers; never block renderer/FFmpeg on network I/O | Resolution bounded 6 × 5 s = 30 s worst case, `WaitCancelled` cancellable; dedup window bounded at 512; no channel/worker exists yet so nothing can block the frame loop — zero edits outside `src/youtube/` (diffstat-proven); the wire-roster sleeps are loopback tests only | ADHERENT |
| 9 | YouTube types stay in `src/youtube/`; cross-boundary output = app-level values + typed error contract | All new types `pub(crate)`/`pub(super)` inside the module; re-exports are crate-internal `pub(crate)` (nothing exported beyond the crate); the only cross-boundary output planned is `ChatMessage` via Task 3; grep-verified zero dependencies from `youtube` toward chat/renderer/streaming/app (doc-comment mentions only) | ADHERENT |

---

## 4. Binding-plan conformance details (spot-verified against the landed files)

- **§3.1 models**: `MAX_CHAT_RESULTS: u64 = 200`; `LiveChatMessageList { items(default), next_page_token, polling_interval_millis, offline_at }`; `LiveChatMessageSnippet` with `#[serde(rename = "type")] message_type`; `text_message_details` `pub(super)`; `TextMessageDetails.message_text`; `LiveChatAuthorDetails { display_name, channel_id }`; `ChatListCommand` (Clone); both path builders byte-match the frozen snippets; `encode_query_value` reused. ✓
- **§3.2 error shapes**: `ChatDisabled`/`ChatEnded` appended at the enum bottom (after `DestinationMissing`) with the prescribed doc comments; Display helpers with the EXACT pinned lines; classify arms placed before the rate-limit arm; `LIVE_CHAT_RESOURCE = "live chat"`; 404 `liveChatNotFound` → `ApiNotFound { resource: "live chat" }`; variant count **15 → 17**, all 15 pre-existing variants verbatim. ✓
- **§3.3/§3.4 seam + concrete impl**: trait/method shapes byte-match; the concrete impl reuses `request`/`parse_resource_body`; `CHAT_MESSAGES_RESOURCE = "liveChatMessageList"` feeds the parse label; the Ok(None) contract ("verifiably empty list answer") documented in the trait docs. ✓
- **§3.5 visibility**: `request`/`parse_resource_body` → `pub(super)` + the one-sentence module-doc addition; `live_chat_id: Option<String>` added to `BroadcastSnippet`; the anticipated compiler arbitration fired exactly once — `LiveBroadcast::snippet` widened `pub(super)` → `pub(crate)` with a doc note naming plan §3.5 (the contingency clause's prescribed widening, recorded in the cycle summary). ✓
- **§3.6 resolution**: wrapper + cfg-split core (`resolve_live_chat_id_with_interval`, 3 args — the sanctioned D-T3-15 exception family, documented in the module doc); `is_transient_error` named helper (single-section conditions); no canceller parameter (the plan's own DECISION; run-level flag wiring = Task 3); after-budget `ApiNotFound { resource: LIVE_CHAT_RESOURCE }`. ✓
- **§3.7 mapping (post-S-1)**: `map_page`/`map_record`/`non_blank_message_text`/`resolve_author`/`non_blank_text`/`page_marked_ended`/`list_shows_offline`/`marks_ended`; `TEXT_MESSAGE_TYPE`/`CHAT_ENDED_TYPE` constants; the simplifier's behavior-preservation table holds (untrimmed raw text accepted, identical skip decisions). `ChatTextMessage`/`ChatPageOutcome` derive `Debug` only — never `Display`. ✓
- **§3.8 dedup**: `CHAT_DEDUP_WINDOW = 512`; `record()` returns true exactly for unseen ids; bounded push_back/pop_front eviction. ✓
- **§3.9 mod.rs**: exactly **200 lines** (the caller ruling); the Phase-06 module-doc paragraph ≤ 10 lines; the 6 chat mod decls in alphabetical position with the 6 prescribed `#[allow(dead_code)]` stagings whose comments name Task 3; the §3.9 re-export list landed verbatim (`LiveChatTransport`, `ChatDedupWindow`, `map_page`/`ChatPageOutcome`/`parse_rfc3339_seconds`/`ChatTextMessage`, `chat_messages_path`/`ChatListCommand`/`LiveChatMessageList`, `resolve_live_chat_id`) with 4 compiler-named `#[allow(unused_imports)]` stagings (plan §6's contingency; comments name Task 3). ✓
- **§3.10 fixtures**: `BROADCAST_ID_FIXTURE`/`LIVE_CHAT_ID_FIXTURE`/`ACCESS_TOKEN_FIXTURE` (`*-fixture` shapes only); `ScriptedLiveChatTransport` with per-op queues + an `operations` recorder (minor addition beyond the sketch, required by the §5.3 operation-count assertions); `live_chat_list_body` builder; registered `#[cfg(test)] #[path]` from `live_chat_resolution.rs` and the wire roster (both instantiations crate-rooted). ✓
- **§4 wire pins (6)**: `chat_list_sends_bearer_only_with_documented_query` (byte-exact request target + Bearer + no `key=`), `chat_list_appends_page_token_only_when_present` (both arms), `chat_list_response_decodes_page_fields` (interval 13000/token/offline-absent), `chat_list_malformed_body_is_typed_error` (`MalformedResponse { field: "liveChatMessageList" }`), `broadcast_chat_lookup_reads_snippet_live_chat_id` (Some + empty→None via two per-call servers), `chat_disabled_error_body_classifies_as_live_chat_disabled` (403 body ⇒ `ChatDisabled`). Shared `ChatWireServer` full-head capture in the parent roster. All loopback `127.0.0.1`. ✓
- **§5 rosters**: names and counts match exactly — models 6 (§5.4), mapping 11 (§5.1, caller-ruled kept), dedup 3 (§5.2), resolution 6 (§5.3, zero-interval `resolve_live_chat_id_with_interval`), live_error +4 (§5.5 incl. the `bare_forbidden_maps_insufficient_permissions` guard — no pre-existing pin asserted 403/`forbidden` → `ApiRejected`; the only "forbidden" pin in the tree is the NEW one, so the STOP-guard never fired), display +2 (§5.6). Rosters wired only via the `#[cfg(test)] #[path]` foot pattern. ✓
- **§6 verification posture**: targeted runs + full gate + regression guards all evidenced by the gate log (429/0, `ALL CHECKS PASSED`); secret audit clean (no `ya29`/`AIza`; `rtmps://` hits are the pre-existing `.example.test` fixtures only; the only `Bearer` text is the pre-existing engine + the fixture-shaped test assertion); zero commented-out code; no builds (correctly deferred to Task 4). ✓
- **§7 git handling**: commit messages match the plan verbatim (`baf260e`, `ac7385f`); no push, no merge, no TODO marks in Task 2's span. ✓
- **§10 NOT-done ledger**: no worker/channel/baseline/pacing/backoff/stage-11/source-replacement/trait-widening/`ChatMessage` call sites/with_timestamp retirement; no docs/structure/context/report/builds; no Cargo/lockfile edits. All confirmed by the span diffstat. ✓
- **Line caps**: every new file ≤ 200 lines (`mod.rs` exactly 200; largest others: mapping tests 194, rfc3339 149, mapping 116, resolution tests 106); method bodies far below 50; the one >2-arg signature is the sanctioned 3-arg cfg-split core. ✓

---

## 5. Adjudication of the 4.3-fix flag — "(matching enum order)" parenthetical

**Finding.** The fix plan's required-fix 1 instructs: *"Insert `LiveApiError::WaitCancelled` into `all_error_variants()` between `DestinationMissing` and `ChatDisabled` (matching enum order in `live_error/mod.rs`)."* The two halves of that instruction are internally inconsistent: the ACTUAL enum order in `src/youtube/live_error/mod.rs` is `… ReadinessTimeout → WaitCancelled → DestinationMissing → ChatDisabled → ChatEnded`, so "matching enum order" would place `WaitCancelled` BETWEEN `ReadinessTimeout` and `DestinationMissing`, not after `DestinationMissing`.

**Landed code.** `all_error_variants()` (post-`f369410`) places `WaitCancelled` between `DestinationMissing` and `ChatDisabled` — the plan's literal position, NOT the enum order.

**Classification: PLAN-TEXT NIT.** Not a landed-code issue:

- The roster is a `Vec` zipped against `expected_display_lines()`; the test's invariants are comprehensive coverage (all 17 variants present), one pinned exact Display line per variant, the `[&'static str; 17]` type, and the `rendered.len() == 17` assertion (updated from 16 per the fix). Roster ordering is semantically free — the test's own header says "in the pinned line order", i.e. the roster's own order.
- The `WaitCancelled` Display line in the roster (`"the YouTube readiness wait was cancelled by the stop request"`) is byte-identical to `display.rs`'s `display_wait_cancelled` and is additionally pinned by the separate `wait_cancelled_display_is_stable_and_secret_free` test.
- No behavior, contract, wire shape, or coverage consequence exists. The only inaccuracy is the fix plan's parenthetical misdescribing the enum order; the operative instruction (the position) was followed and is harmless.

**Proposed change: NONE** for the landed code. Optional (caller's discretion, out of this pass's scope): a one-line plan-text correction in the already-committed fix plan would require a new docs commit; the adjudication recorded here is sufficient. The comprehensive-roster invariant and the caller-authorized 17-variant count are both satisfied.

---

## 6. Deviation ledger (all classified; none non-adherent)

| # | Deviation | Class | Rationale |
| --- | --- | --- | --- |
| D-1 | `chat_messages_path` landed `pub(crate)`, not §3.5's `pub(super)` | ADHERENT WITH ACCEPTABLE DEVIATION | Required by the plan's own §3.9 `pub(crate)` re-export (a `pub(super)` item cannot be re-exported at `pub(crate)`); compiler-driven, zero behavior |
| D-2 | `live_error/mod.rs` re-export widened to `pub(crate) use classify::{classify_api_error, LIVE_CHAT_RESOURCE}` | ADHERENT WITH ACCEPTABLE DEVIATION | Needed by §3.6's `use super::live_error::{LiveApiError, LIVE_CHAT_RESOURCE}`; compiler-driven one-line addition consistent with §2.2's intent |
| D-3 | `LiveBroadcast::snippet` widened `pub(super)` → `pub(crate)` | ADHERENT (contingency executed as prescribed) | Plan §3.5 explicitly anticipated exactly this widening ("widen ONLY the exact field there — compiler-arbitrated prevention, record the widening in the cycle summary"); recorded with a doc note in `broadcast_resource.rs` |
| D-4 | Four `#[allow(unused_imports)]` on the chat re-exports | ADHERENT WITH ACCEPTABLE DEVIATION | Plan §6's contingency clause ("apply ONLY the exact allow the compiler names, with a comment naming Task 3, and record it"); every comment names Task 3 |
| D-5 | `ScriptedLiveChatTransport` gained an `operations` recorder beyond the §3.10 sketch | ADHERENT WITH ACCEPTABLE DEVIATION | Required by §5.3's operation-count assertions ("operations count == 1"); mirrors the `live_test_fixtures` precedent; test-only |
| D-6 | Fix plan's "(matching enum order)" parenthetical inaccurate vs the actual enum order | PLAN-TEXT NIT (§5) | Landed code correct; roster order semantically free; no code change proposed |
| D-7 | Plan §2.3 projected +36 tests (427) vs observed +38 (429) | ADHERENT (arithmetic only) | The plan's own "OBSERVED GOVERNS" rule; the two-count delta was §5.1's 11-name roster vs the "10" projection and §4's flagged wire-split 6th test — both caller-ruled during 4.2 |
| D-8 | `is_leap_year`/`split_validated_fraction`/`page_marked_ended` compound-boolean bodies | ADHERENT | Already adjudicated by the 4.3 review as minor observations — function bodies, not `if`/`while` statements; the rule's scope is control-flow statements; the two REQUIRED fixes (2, 3) were applied exactly |

---

## 7. Carry-forwards and flags

### 7.1 Carry-forwards for Task 3 (transcribed; Task 3's 4.1b owns the step list)

- **C-1 — the `ChatTextMessage` → `ChatMessage` conversion point.** Task 3 constructs `ChatMessage::new(author, text)` (normalization runs there — the carrier's text is the untrimmed raw string by design), then `.with_message_id(message_id)` verbatim when `Some`, then `.with_timestamp(published_epoch)` when `Some`. The mapping layer stays chat-free; the `mod.rs` re-exports (`map_page`, `ChatPageOutcome`, `ChatTextMessage`, `parse_rfc3339_seconds`, `ChatDedupWindow`, `LiveChatTransport`, `chat_messages_path`, `ChatListCommand`, `LiveChatMessageList`, `resolve_live_chat_id`) are the PINNED Task-3 consumption surface — do not re-pick or remove without a caller ruling.
- **C-2 — the `with_timestamp` retirement.** `ChatMessage::with_timestamp`'s item-level `#[allow(dead_code)]` (one of the 6 pre-existing reservations) retires when Task 3's consumer attaches `published_epoch` (dead-code retirement ledger precedent; record D-4).
- **C-3 — the worker/channel wiring.** Record §3 + D-5/D-6/D-9: bounded `std::sync::mpsc::sync_channel::<ChatMessage>(64)` (`CHAT_CHANNEL_CAPACITY`), per-session `ChatDedupWindow`, FIRST-page baseline discard (all first-page items dropped; only token + interval retained), server-interval honor-loop with 1 000 ms stop-slices (`CHAT_SLEEP_SLICE_MS`), backoff = max(server interval, 1 s) × 2^(attempt−1) capped 60 000 ms, `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`, shared terminal cell, graceful `ChatEnded`/`offlineAt`/`chatEndedEvent` vs terminal-fatal classes, stage-11 startup after the gate joins, shutdown join ≤ ~1 s, `RunError::ChatRetrieval` funnel where chat failure NEVER outranks `GateFinish::Fatal`. The `polling_interval_millis`/`next_page_token` seam values are already exposed (bullet 7 verified).
- **C-4 — the seam-widening decision.** Record D-8: NEW `src/app/pipeline/source_error.rs` with `ChatSourceError { Chat(ChatError), Retrieval(ChatRetrievalError) }` over `ChatRetrievalError { ChatUnavailable { detail }, Quota { detail }, Transport { detail } }` (detail = redacted `Display` of the inner `LiveApiError` only); `MessageSource::next_message` error type widened; mechanical `DemoMessageSource`/`test_helpers.rs`/`source_helpers.rs` fallout; `ChatSourceError::Chat(e)` → the existing `FramePipelineError::Chat` path (log-first contract intact), `Retrieval(e)` → NEW `FramePipelineError::ChatRetrieval` → `RunError::ChatRetrieval` → exit 1, one redacted `error:` line, no new exit code; demo byte-stable.
- **C-5 — staging retirements.** The 6 `#[allow(dead_code)]` module stagings and 4 `#[allow(unused_imports)]` re-export stagings in `src/youtube/mod.rs` retire when Task 3's worker roots the consumption chain (comments name Task 3; the plan's prescription).

### 7.2 Flags for Task 4 (docs-owned; NOT touched in this pass)

- **F-1 — `.agent/project-structure.md` variant-count refresh.** Line 66 still reads "`LiveApiError` enum (15 variants incl. the Task-2 `WaitCancelled` typed cancellation…)" — stale at 17 (Task 2 added `ChatDisabled`/`ChatEnded`). Task 4's structure-map pass must refresh the count (and the chat-module rosters) to the final-tree truth.
- **F-2 — README test-count re-measure.** README line 46 still claims "391 tests" — the observed count on the Task-2 tree is 429. Task 4's docs pass must re-measure on ITS final tree and fix the count (the standing per-task 4.4 precedent).

---

## 8. Verification evidence index

| Check | Command / observation | Result |
| --- | --- | --- |
| Tree/branch/HEAD | `git status --porcelain` (empty); `git branch --show-current` (`feat/phase06-youtube-live-chat`); `git log --oneline -10` | clean, HEAD `f369410` |
| Commits | `git show --stat --oneline baf260e / ac7385f / f369410` | 1 / 22 / 5 files as §2.1 |
| Span diffstat | `git diff --stat 95d791d..f369410 -- src/`; `git diff --stat b75d914..f369410` | 22 src/youtube files; branch span = plans/TODO/version bump only |
| Dependency lock | `git diff --stat 95d791d..f369410 -- Cargo.toml Cargo.lock` | empty (D-11) |
| Gate | VM `logs/checks/20261026T031702Z.log` tail (re-observed via Alpine VM) | exit 0, `429 passed; 0 failed`, `ALL CHECKS PASSED` |
| Secret audit | greps over the Task-2 files + span: `ya29`/`AIza` = 0 new; `rtmps://` = pre-existing `.example.test` fixtures only; `Bearer` = pre-existing engine + fixture-shaped assertion | clean |
| Logging audit | grep `tracing|println!|eprintln!|info!|warn!|error!|debug!|trace!|log::` over `live_chat_*.rs` | zero hits (D-13) |
| Boundary audit | grep `crate::chat|crate::renderer|crate::streaming|crate::app` over `src/youtube` | doc-comment mentions only (D-4/D-8 boundary holds) |
| Guard §5.5.3 | grep `"forbidden"` over `src/youtube` | only the NEW arm + the NEW test pin; no pre-existing pin invalidated |
| Fix-plan fixes 1–3 | direct file reads: `live_error_display_tests.rs` (17 variants/lines/count), `live_chat_rfc3339.rs` (`is_valid_month`, `february_offset_applies`, single-section day range) | all applied |

---

## 9. Footer

Report: `.kilo/plans/20261025-phase06-task2-adherence.md`
Verdict: **ADHERENT WITH ACCEPTABLE DEVIATIONS — zero non-adherent items.**
Produced 2026-10-26 by the Task-2 step-4.5b architector pass. Not committed (tracking belongs to the caller's next step).
