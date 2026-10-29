# Phase 07 Task 2 — Simplification Plan (workflow step 4.3, simplifier half)

> Authority: binding plan `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` (its §C content blocks are DESIGN-staged contracts — NOT simplification scope), TODO `.agent/todos/20261029/20261029-todo-1.md` Task 2, repo rules (`.kilo/rules/*`).
> Reviewed tree: branch `feat/phase07-streamlist-retrieval` @ `35adefa`, gate 507/0.
> Executor: JUNIOR implementer under 50% restriction — execute each edit EXACTLY as written, in order; no judgment calls, no extra cleanup, no commit (the workflow step owns the commit).
> Scale: 2 edits across 2 files, ~14 changed lines total (≪30% of any file). No plan-staged structure is touched (field ORDER, seam trait signatures, the two new variants, Display strings, the 8 staging attributes, the golden byte arrays — all untouched; Edit 1 RESTORES the §C.8 comment+brace unit, it does not restage anything).

## Verdict

Simplification REQUIRED, minimal. Two edits below; then the scan matrix and rejected candidates (for the 4.4 reviewer and the Task-3 ledger).

---

## Edit 1 — `src/youtube/mod.rs`: misplaced Phase-07 staging comment (defect fix)

The 4-line comment describing the Phase-07 staged re-export brace currently sits ABOVE `pub(crate) use live_ingestion::{…}` (line 151–155), detached from the `#[allow(unused_imports)] pub(crate) use stream_transport::{…};` brace (line 158–161) it annotates. Plan §C.8 item 2 stages comment+brace as ONE unit. Fix by moving the comment, content VERBATIM unchanged.

**1a. Anchor** (unique — includes the preceding line):

```rust
pub(crate) use live_error::LiveApiError;
// Phase-07 pinned Task-3 consumption surface (record P7-12; plan
// `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` §C): the
// worker consumes the stream seam; clippy-named staging until the worker
// rewires.
pub(crate) use live_ingestion::{LiveBroadcastPreparer, PreparedBroadcast};
```

**Replacement:**

```rust
pub(crate) use live_error::LiveApiError;
pub(crate) use live_ingestion::{LiveBroadcastPreparer, PreparedBroadcast};
```

**1b. Anchor** (unique — the `readiness_wait` line followed by the allow):

```rust
pub(crate) use readiness_wait::{wait_until_stream_active, StreamObservationRequest};
#[allow(unused_imports)]
pub(crate) use stream_transport::{
```

**Replacement:**

```rust
pub(crate) use readiness_wait::{wait_until_stream_active, StreamObservationRequest};
// Phase-07 pinned Task-3 consumption surface (record P7-12; plan
// `.kilo/plans/20261029-phase07-task2-streaming-client-bridge.md` §C): the
// worker consumes the stream seam; clippy-named staging until the worker
// rewires.
#[allow(unused_imports)]
pub(crate) use stream_transport::{
```

Constraints: do NOT edit the comment text, the `#[allow(unused_imports)]` line, or the brace contents. File stays 199 lines (≤200 ✓). `#[allow(dead_code)]` module stagings and their comments: untouched.

## Edit 2 — `src/youtube/stream_client_tests.rs`: fold the 3×-duplicated bearer construction

`bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE)).expect("the fixture token is header-safe")` repeats in three tests (§E.4 pins names/intent, not bodies). Fold into one roster-local helper. Test names, count (6), assertions and expect-message text stay identical.

**2a. Anchor** (top of file, unique):

```rust
use super::*;
use crate::youtube::ACCESS_TOKEN_FIXTURE;

#[test]
fn bearer_token_metadata_is_bearer_only() {
```

**Replacement:**

```rust
use super::*;
use crate::youtube::ACCESS_TOKEN_FIXTURE;

fn fixture_bearer() -> MetadataValue<Ascii> {
    bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe")
}

#[test]
fn bearer_token_metadata_is_bearer_only() {
```

(`MetadataValue`/`Ascii` are in scope through `use super::*;` — the parent `stream_client.rs` binds them via its own import, the same mechanism the roster already uses for `AccessToken`. If the compiler rejects the path, STOP and report; do NOT add a second import line of your own accord.)

**2b. Anchor** (apply AFTER 2a; unique by the third line):

```rust
    let metadata = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    assert_eq!(
```

**Replacement:**

```rust
    let metadata = fixture_bearer();
    assert_eq!(
```

**2c. Anchor** (now the remaining two-line duplicate sits in `bearer_token_metadata_holds_the_authorization_key_not_an_api_key`; unique with the third line):

```rust
    let metadata = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    let rendered = metadata.to_str().expect("the value is ASCII");
```

**Replacement:**

```rust
    let rendered = fixture_bearer().to_str().expect("the value is ASCII");
```

**2d. Anchor** (unique):

```rust
    let bearer = bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
        .expect("the fixture token is header-safe");
    let request = stream_request(LiveChatMessageListRequest::default(), bearer);
```

**Replacement:**

```rust
    let bearer = fixture_bearer();
    let request = stream_request(LiveChatMessageListRequest::default(), bearer);
```

File lands at 73 lines (≤200 ✓); helper body 2 lines (≤50 ✓, ≤2 args ✓).

---

## Verification (container-only, one command at a time)

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → exit 0 (if it fails, run `cargo fmt` and re-check; fmt-only, no semantic edits).
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked stream_client` → 6 passed / 0 failed.
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → exit 0.
4. FULL GATE `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → `ALL CHECKS PASSED`, 507 passed / 0 failed (Edit 1 removes no code; Edit 2 changes no test semantics — the total must NOT move).
5. `git status --porcelain` → exactly the two modified files (`src/youtube/mod.rs`, `src/youtube/stream_client_tests.rs`).

## DO-NOT-TOUCH list (plan-staged; out of simplification scope)

- `stream_proto.rs` struct field ORDER and every `#[prost(... tag = "n")]`; §C.1 `stream_list_request` body verbatim.
- Golden arrays `ANCHOR_A`–`H`, `TOLERANCE_T1/T2/U1` and the inline `0x08, 0x02` / `0x08`+`U1` fixtures in `stream_proto_tests.rs` (roster pins).
- Seam trait signatures `ChatStreamTransport::open` / `LiveChatStreamSession::next_batch` and the Option semantics comment.
- The two new `LiveApiError` variants + their Display strings + the two §C.6 display helpers.
- All 8 staging `#[allow(dead_code)]` attributes + their one-line comments (L.8 caller ruling) and the §C.8 `#[allow(unused_imports)]` brace contents.
- §C.5 mapper bodies (`map_stream_batch`, `map_item`, `is_text`, `message_text`, `author_name`, `batch_ended`, `is_chat_ended_item`) and the §C.6 `classify_stream_status` body.

---

## Scan matrix (what was reviewed and decided)

| # | Scope question | Result |
|---|---|---|
| 1 | Cross-file genuine duplication | `Network { reason: X }` builders (`endpoint_build_failure`, `stream_connect_timeout_error`, `stream_connect_failure_error`) — §C.2/§C.4 pinned bodies: REJECTED. `author_name` (stream) vs `resolve_author` (`live_chat_mapping.rs`) — identical policy line over DIFFERENT input types (prost vs REST model), §C.5 pinned: REJECT (folding needs a trait/generic = medium redesign + plan deviation). Intra-file 3× bearer recipe — FOLD (Edit 2). |
| 2 | Roster fixture-recipe duplication | Mapping/transport rosters already consume the §C.7 builders (`text_item`, `tombstone_item`, `ended_item`, `response_*`, `yielded`) — verified. Proto roster keeps local builders because the goldens pin literal values (`"Bob"`, `"hello"`, `"msg-1"`, `"chat-id-fixture"`) distinct from the C.7 fixture values: BY DESIGN, not foldable. Local `fn request()` appears in `stream_proto_tests.rs` + `stream_transport_tests.rs` with deliberately different id literals (golden pin vs `LIVE_CHAT_ID_FIXTURE`): REJECTED. `blank_text_item` is a single-use roster-local builder: acceptable. |
| 3 | Allocations/closures/intermediates in non-pinned bodies | §C.1/§C.5/§C.6 bodies match the plan verbatim (fmt wraps only) — no deviation, no simplification allowed there. `stream_endpoint` matches §C.2 EXACT body. The `is_ascii()` guard (L.4) removes an allocation on the failure path — keep. No trimmable conversion found outside pinned bodies. |
| 4 | Dead items masked by the stagings (Task-3 retirement ledger — FLAG ONLY, do not fix) | Masked surfaces, all retiring at the Task-3 rewire per L.8: (1) `mod stream_proto` → `CHAT_STREAM_PARTS`, `stream_list_request`, all structs/enum (prod-dead); (2) `mod stream_transport` → `ChatStreamRequest`, `YieldedChatStreamBatch`, both traits; (3) `mod stream_client` → 3 pub consts, 3 reason consts, `CHAT_STREAM_CONNECT_TIMEOUT_MS`, `bearer_token_metadata`, `stream_request`, `stream_endpoint`, `endpoint_build_failure`, `TonicChatStreamSession`(+`new`); (4) `mod stream_concrete` → `GoogleLiveChatStreamTransport`(+`new`, 2 error fns); (5) `mod live_chat_stream_mapping` → `map_stream_batch` + 6 helpers; (6) `mod grpc_status` → `classify_stream_status`, 6 consts, `stream_code_reason`; (7)(8) item-level → `ChatPrecondition`, `InvalidChatRequest`; plus the `#[allow(unused_imports)]` stream_transport brace. Allow-count verified at 17 = 8 existing + 9 new, matching L.6/L.8. No unmasked dead item exists (clippy `-D warnings` is green). |
| 5 | Line counts / fn bodies | New files (fmt'd, all ≤200): `stream_proto.rs` 163, `stream_client.rs` 96, `stream_transport.rs` 50, `stream_concrete.rs` 72, `live_chat_stream_mapping.rs` 78, `live_error/grpc_status.rs` 61, `stream_test_fixtures.rs` 198, `stream_proto_tests.rs` 190, `live_chat_stream_mapping_tests.rs` 157, `live_error_grpc_status_tests.rs` 151, `stream_client_tests.rs` 72, `stream_transport_tests.rs` 123. Modified: `mod.rs` 199, `live_error/mod.rs` 116, `live_error/display.rs` 187. Largest fn bodies: `classify_stream_status` 26, `open()` 20, `next_batch` 6 — all ≤50. ≤2 args everywhere; depth ≤2 everywhere; no commented-out code (the only `key=` hit is the plan-mandated negative assertion; zero-sleep grep clean, reproduced). |
| — | Other | `stream_transport_tests.rs` embeds the literal `"LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED"` instead of `CHAT_PRECONDITION_DETAIL` — the const is unreachable from that module (live_error re-exports only `classify_stream_status`, per plan §C.6); widening the re-export would add prod surface: REJECTED (Task-3 consumption resolves it naturally). Doc-header `TODO's sanctioned` mention is prose, not a TODO marker. |

## Executability note

Both edits are minor local details (comment relocation + test-local helper extraction) — within the junior's authorized latitude once specified; no medium/big redesign involved, so this plan stands on its own (no architector escalation needed).
