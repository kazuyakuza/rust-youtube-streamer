# Simplification Plan — Phase 06, Task 3 (step 4.3): Chat-Roster Test-Harness Folds

> Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — Task 3. Binding shapes: `.kilo/plans/20261025-phase06-task3-run-integration.md` (AMENDMENTS 1–2 rulings binding; §4 code specs; §5 rosters; §9 byte-stability ledger).
> Baseline (caller-stated 2026-10-26): branch `feat/phase06-youtube-live-chat`, HEAD `f8bf0ea`, tree clean, gate **458 passed / 0 failed**.
> Landed Task-3 sources reviewed: commit `092f61f` (32 files) + `f8bf0ea` (registration move).
> **Scope of this plan: TEST-ONLY harness folds. ZERO production-file changes.** The implementer applies ONLY these edits — no judgment calls, no extra cleanups.

## 0. Review verdict

- **Production (`chat_worker.rs`, `chat_worker_delivery.rs`, `chat_failure.rs`, `source.rs`, `source_error.rs`, `gate.rs`, `streaming.rs`, `seams.rs`, `mod.rs`, `error.rs`, pipeline files, `youtube/mod.rs`):** plan-conformant, no genuine simplification found — DO NOT TOUCH.
- **Test rosters (`chat_worker_tests` 157, `chat_worker_flow_tests` 124, `chat_worker_budget_tests` 153, `gate_chat_tests` 80, `chat_source_tests` 125, `runtime_chat_fakes` 167):** genuine incidental duplication found — 12 near-identical worker-run blocks, a duplicated `lookup_found` helper, a near-duplicated `drain` helper, 3 hand-inlined terminal-cell reads, 2 magic-string `offlineAt` literals, and 2 bypasses of the file's own `source_of` helper. Folds below.
- **Dead code from the C-5 retirement:** none found (every retired staging has its consumer landed; `failing`/`failing_retrieval` both consumed; all `runtime_chat_fakes` members consumed).

## 1. Binding pins honored (NOT touched by this plan)

1. All §5 test NAMES and assertion MESSAGES stay verbatim (names are plan-pinned; fold edits never add, remove, rename or move a `#[test]`).
2. D-12/D-13 constants, the pinned Display lines, the cfg-split shapes, the `pub(super) const CHAT_CHANNEL_CAPACITY` visibility (plan §4.3.2 literal — kept even though only in-module today), and every registration site (AMENDMENT-1 ruling 1/3, AMENDMENT-2 rulings 4/6/7) are byte-unchanged.
3. The compiler/plan-forced allows recorded in the landed code (`source_error.rs` `#[allow(dead_code)]` on `Chat`, `chat_worker_delivery.rs` `#[allow(clippy::manual_clamp)]`, the 3 `youtube/mod.rs` chat `#[allow(unused_imports)]` braces) — report-only, NOT touched (plan §6(f)/§10 items, D-8 pinned shapes).
4. `runtime_chat_fakes::BROADCAST_ID_FIXTURE` stays local (per-fixture-module constant duplication is the pre-existing repo pattern — `youtube/live_test_fixtures` vs `google_live_api_transport_tests` precedent; folding it would add cross-module coupling: rejected).
5. `runtime_tests.rs` keeps its own `OFFLINE_AT_FIXTURE` const (module privacy: `chat_worker_tests` is unreachable from `run::runtime_tests`; the const is already named, not a literal: rejected).
6. The helper home stays `chat_worker_tests.rs` (`lookup_found`/`drain`/`terminal_of`/`OFFLINE_AT`/`PUBLISHED_AT` are `pub(super)` there; the `chat_worker_flow_tests` import line is the landed precedent) — moving them into `runtime_chat_fakes` was evaluated and rejected (bigger diff, pushes fakes toward the 200 cap).
7. All test counts, the zero-sleep rule, the `-fixture` material shapes and the offline posture are unchanged. Every touched file stays ≤ 200 lines; ZERO new `#[allow(`.

## 2. Step-by-step recipe (execute in order; one commit at the end)

All paths are `src/app/run/`. Steps 2–6 depend on Step 1.

### STEP 1 — `runtime_chat_fakes.rs`: add the shared run helper

Append AFTER `worker_plan` (end of file):

```rust
/// Runs the chat worker to completion against the scripted fake and
/// returns the channel bundle for the caller's drain/terminal asserts.
pub(crate) fn run_worker(
    fake: std::sync::Arc<ScriptedRunChatTransport>,
) -> super::ChatChannel {
    let channel = super::chat_channel();
    super::run_chat_worker(worker_plan(fake, &channel));
    channel
}
```

(`super::chat_channel`, `super::run_chat_worker`, `super::ChatChannel` resolve: this module is a child of `chat_worker`, and a parent's private/`pub(super)` items are visible to its descendants.) Expected result: ~175 lines, ≤ 200.

### STEP 2 — `chat_worker_tests.rs`: fold the six run blocks

2a. Replace the `runtime_chat_fakes` import (lines 10–12) with:

```rust
use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, run_worker, ScriptedRunChatTransport, LIVE_CHAT_ID_FIXTURE,
};
```

2b. Replace line 15 `use super::{chat_channel, run_chat_worker, ChatChannel};` with:

```rust
use super::ChatChannel;
```

2c. The helpers `lookup_found`, `drain`, `terminal_of`, `PUBLISHED_AT`, `OFFLINE_AT` and ALL test names/assertions stay EXACTLY as-is. Replace these six two-line run blocks with one line each (the surrounding `assert!`/script lines stay verbatim):

| Test | BEFORE (2 lines) | AFTER (1 line) |
| --- | --- | --- |
| `worker_graceful_ended_page_disconnects_without_terminal_error` | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(fake, &channel));` | `let channel = run_worker(fake);` |
| `worker_chat_ended_error_class_is_graceful` | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(fake, &channel));` | `let channel = run_worker(fake);` |
| `worker_chat_disabled_failure_writes_the_chat_unavailable_terminal` | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(fake, &channel));` | `let channel = run_worker(fake);` |
| `worker_resolution_absence_after_budget_maps_chat_unavailable` | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(Arc::clone(&fake), &channel));` | `let channel = run_worker(Arc::clone(&fake));` |
| `worker_rate_limit_exceeded_is_transient_while_quota_exhaustion_is_terminal` (first) | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(recovering, &channel));` | `let channel = run_worker(recovering);` |
| same test (second) | `let channel = chat_channel();`<br>`run_chat_worker(worker_plan(exhausted, &channel));` | `let channel = run_worker(exhausted);` |

Expected result: ~151 lines.

### STEP 3 — `chat_worker_flow_tests.rs`: fold the four run blocks

3a. Replace the fakes import (lines 8–10) with:

```rust
use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, run_worker, text_message_item, ScriptedRunChatTransport,
};
```

3b. DELETE line 15 (`use super::{chat_channel, run_chat_worker};`). Keep line 14 (`use super::chat_worker_tests::{drain, lookup_found, terminal_of, OFFLINE_AT, PUBLISHED_AT};`) unchanged.

3c. Replace the run blocks (same pattern as 2c):
- `worker_delivers_mapped_messages_in_page_order_with_metadata`, `worker_discards_the_entire_baseline_history_page`, `worker_dedup_drops_replayed_ids_across_pages`: `let channel = chat_channel();`+`run_chat_worker(worker_plan(fake, &channel));` → `let channel = run_worker(fake);`
- `worker_empty_page_advances_the_token_without_inference`: `let channel = chat_channel();`+`run_chat_worker(worker_plan(Arc::clone(&fake), &channel));` → `let channel = run_worker(Arc::clone(&fake));`

Expected result: ~117 lines.

### STEP 4 — `chat_worker_budget_tests.rs`: drop the duplicated helpers, fold the two run blocks, reuse `terminal_of`/`OFFLINE_AT`

4a. Replace the fakes import (lines 10–12) with:

```rust
use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, run_worker, text_message_item, worker_plan, ScriptedRunChatTransport,
};
```

4b. DELETE the local `lookup_found` fn (lines 21–23) and the trailing local `drain` fn (lines 147–153). ADD after the `use super::{chat_channel, ...}` line:

```rust
use super::chat_worker_tests::{drain, lookup_found, terminal_of, OFFLINE_AT};
```

(Keep lines 10–16 style imports otherwise byte-unchanged: `chat_channel, run_chat_worker, spawn_chat_worker, CHAT_BACKOFF_MAX_MS` stay — tests 4/5/6 still use them; `worker_plan` stays in the fakes list for those tests.)

4c. `worker_transient_failures_exhaust_the_budget_into_a_transport_terminal`:
- `let channel = chat_channel();`+`run_chat_worker(worker_plan(fake, &channel));` → `let channel = run_worker(fake);`
- the five-line `channel.terminal.lock().expect("cell mutex").take().expect(...)` block → 

```rust
    let failure = terminal_of(&channel).expect("the budget exhausts into a terminal");
```

- the `assert!(matches!(failure, crate::app::pipeline::ChatRetrievalError::Transport { ref detail } if detail == "unable to reach the YouTube API: network-fixture-10"));` stays verbatim (rustfmt keeps it multi-line as today).

4d. `worker_transient_failures_recover_within_the_budget`:
- in the third `script_list` page, the literal `Some("2026-10-26T02:00:00Z"),` → `Some(OFFLINE_AT),`
- `let channel = chat_channel();`+`run_chat_worker(worker_plan(fake, &channel));` → `let channel = run_worker(fake);`
- the two lines `let terminal = channel.terminal.lock().expect("cell mutex").take();`+`assert!(terminal.is_none(), "recovery inside the budget is clean");` → 

```rust
    assert!(terminal_of(&channel).is_none(), "recovery inside the budget is clean");
```

- the two lines `let received: Vec<String> = drain(&channel.receiver);`+`assert!(received.is_empty(), "the recovery page is the baseline page: discarded by design");` → 

```rust
    assert!(
        drain(&channel.receiver).is_empty(),
        "the recovery page is the baseline page: discarded by design"
    );
```

4e. `worker_stop_during_delivery_drops_pending_messages_without_blocking`:
- the two lines `let terminal = channel.terminal.lock().expect("cell mutex").take();`+`assert!(terminal.is_none());` → 

```rust
    assert!(terminal_of(&channel).is_none());
```

(The `drain(&channel.receiver).is_empty()` assert keeps its message; `drain` now yields `Vec<ChatMessage>` — emptiness is unchanged.)

4f. Tests `worker_backoff_delay_doubles_and_caps_at_the_maximum`, `worker_stop_flag_ends_the_loop_before_any_transport_call`, `worker_blocked_list_call_cannot_prevent_bounded_shutdown` stay byte-unchanged (they need the explicit `worker_plan`/`spawn_chat_worker` paths).

Expected result: ~137 lines.

### STEP 5 — `gate_chat_tests.rs`: reuse `lookup_found`/`OFFLINE_AT`

5a. In the fakes import (lines 10–12), REMOVE `LIVE_CHAT_ID_FIXTURE` (unused after 5b): keep `chat_page, worker_plan, ScriptedRunChatTransport`.

5b. After `use super::chat_channel;` ADD:

```rust
use super::chat_worker_tests::{lookup_found, OFFLINE_AT};
```

5c. In `confirmed_gate_spawns_and_joins_the_chat_worker`:
- `chat_fake.script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));` → `lookup_found(&chat_fake);`
- the page literal `Some("2026-10-26T02:00:00Z"),` → `Some(OFFLINE_AT),`

`cancelled_gate_never_spawns_the_chat_worker` and all asserts stay verbatim (the `["lookup_broadcast_chat_id", "list_messages"]` operation labels are literals, not the fixture const).

### STEP 6 — `chat_source_tests.rs`: use the file's own `source_of` helper

6a. In `source_surfaces_the_terminal_error_once_then_yields_none`: `let mut source = ChatSource::new(channel.receiver, channel.terminal);` → `let mut source = source_of(channel);`
6b. In `source_disconnected_without_terminal_error_yields_none_forever`: same replacement.
(The `use super::ChatSource;` import stays — `source_of`'s body still names it.)

## 3. Verification (Alpine VM / docker compose per the phase tooling; STOP on any failure)

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt"` then `cargo fmt --check` (touching ONLY the 6 files above — the edits may reflow).
2. Targeted filters (all green, test NAMES identical):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_worker`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_source`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked gate`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked runtime`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → exit 0; ZERO new `#[allow(`.
4. FULL gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0 + `ALL CHECKS PASSED` + observed count **458 passed / 0 failed** (no test added/removed; OBSERVED governs — if the count differs from 458, STOP and report the exact count, never improvise).
5. Line caps: `wc -l` on the 6 touched files → every one ≤ 200 (expected: fakes ~175, worker ~151, flow ~117, budget ~137, gate ~78, source ~124).

## 4. Git handling

- Stage EXACTLY the 6 files: `src/app/run/runtime_chat_fakes.rs`, `src/app/run/chat_worker_tests.rs`, `src/app/run/chat_worker_flow_tests.rs`, `src/app/run/chat_worker_budget_tests.rs`, `src/app/run/gate_chat_tests.rs`, `src/app/run/chat_source_tests.rs`.
- `git status`/staging audit per the gitignore rule (no `logs/`, no `target/`).
- Commit message: `refactor: fold the task-3 chat rosters onto shared harness helpers`.
- NO push.

## 5. STOP-guards (any hit → STOP + report, never improvise)

1. Any production file needs an edit to compile → STOP (this plan is test-only).
2. Any gate failure, clippy demand for a new allow, or observed test count ≠ 458.
3. Any rustfmt result pushing a touched file > 200 lines.
4. Any temptation to rename/move/split tests, touch registration sites, or touch `runtime_tests.rs`, `streaming_stage_tests.rs`, `runner_retrieval_tests.rs`, `source_error_tests.rs`, the pinned Display lines, the constants, or the allow-attributed lines — ALL out of scope.

Plan: `.kilo/plans/20261025-phase06-task3-simplification.md`
Produced 2026-10-26 by the Task-3 step-4.3 code-simplifier pass. Test-only scope; production verdict: as-landed, no change.
