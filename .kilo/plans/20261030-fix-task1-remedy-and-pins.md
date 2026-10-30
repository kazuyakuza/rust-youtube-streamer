# Task-1 Implementation Plan — Readiness-Reserve Fix, Worker Panic Funnel, and Offline Contract Pins

> Binding inputs: TODO `.agent/todos/20261030/20261030-todo-1.md` (Pre-Diagnosis is binding) + global plan
> `.kilo/plans/20261030-establishment-readiness-fix.md` (incl. User Rulings: per-task plans auto-approved;
> containment = TYPED FATAL exit 1 via the existing terminal).
> Baseline: branch `fix/stream-establishment-readiness` already carries steps 2–3 (`Cargo.toml` = 0.13.1, observed
> at plan time); tree expected clean before this task's commit except the untracked plan file.
> Test baseline: 494 passed / 0 failed (closing gate `logs/checks/20261029T222610Z.log`).
> This plan is for the 4.2 implementer (junior-50%): every decision is closed-form. NO release builds here (Task 2).

## §0 TODO Task-1 bullet ↔ owning-section map

| TODO Task-1 bullet (`.agent/todos/20261030/20261030-todo-1.md` § Tasks / Task 1) | Owning section |
|---|---|
| Bullet 1 — Fix 1 (`ready()` before the streaming call, fixed-reason mapping), Fix 2 (worker panic containment → existing terminal/fatal funnel) exactly per the per-task plan; `open()` production shape otherwise byte-stable; endpoint parameterized ONLY for the loopback/test pins | §A (Fix 1), §B (Fix 2), §A.4 (parameterization) |
| Bullet 2 — the three offline pins exactly per the plan: incident anchor / fix shape / panic→terminal funnel | §C.1 (pin a), §C.2 (pin b), §C.3 (pin c), §C.4 (roster homes + registration) |
| Bullet 3 — standard gate + windows-gnu check; observed test count recorded; NO release builds | §E |

Evidence record for the tonic/tower research (all access-dated) is §A.0/§C.0. Compliance matrix: §H.

## §A FIX 1 — the readiness reserve in the establishment composition

### §A.0 Verified tonic/tower evidence (access date 2026-10-30)

Primary sources are the in-tree lockfile (tonic 0.13.1 / tower 0.5.3 / tokio-util 0.7.19, `Cargo.lock`) plus:

1. `https://docs.rs/tonic/0.13.1/tonic/client/struct.Grpc.html` (accessed 2026-10-30): `Grpc::ready(&mut self) -> Result<(), T::Error>` exists ("call GrpcService::poll_ready until it returns ready or an error"); `Grpc::server_streaming` exists with the currently used signature. Note: `server_streaming`'s docs impose NO readiness contract — which is exactly why the missing reserve stayed silent until tower's Buffer panicked.
2. tonic v0.13.1 source `tonic/src/transport/channel/mod.rs` (raw.githubusercontent.com, tag `v0.13.1`, accessed 2026-10-30): `Channel` is backed by `tower::buffer::Buffer` and **its `Service::call` delegates DIRECTLY to the Buffer without any internal `poll_ready`** — the reserve is the caller's duty.
3. tower 0.5.3 source `tower/src/buffer/service.rs` (raw.githubusercontent.com, tag `tower-0.5.3`, accessed 2026-10-30): `Buffer::call` → `self.tx.send_item(Message { .. })` on a `tokio_util::sync::PollSender`; `Buffer::poll_ready` → `self.tx.poll_reserve(cx)`. Calling `call` without a prior `poll_reserve` on the SAME handle is exactly the operator-recorded panic (`send_item called without first calling poll_reserve`, tower-0.5.3 `buffer/service.rs:122` — `logs/run/20261030-211225.log`, operator-supplied).
4. **Mechanics correction over the caller's prompt assumption (verified, does NOT contradict the bound diagnosis):** tonic v0.13.1 `tonic/src/transport/channel/endpoint.rs` (accessed 2026-10-30) shows `Endpoint::connect()` is EAGER (`Channel::connect(connector, endpoint).await` → awaits `Connection::connect`); the documented LAZY variant is `connect_lazy()` ("The channel returned by this method does not attempt to connect to the endpoint until first use"). The incident itself is consistent: production connected to the REAL Google endpoint (connect succeeded), and the panic came from the reserve-less call path. Consequence for pin (a): the dead-port anchor must use `connect_lazy()` so the call path (and its local panic) is reached with zero network I/O — §C.1.
5. `Endpoint::from_shared(s: impl Into<Bytes>) -> Result<Self, Error>` never inspects TLS (docs.rs page above + source: TLS is opt-in via `tls_config`, and the implicit-HTTPS default lives in the doc-hidden `Endpoint::new`, not `from_shared`). An `http://`-scheme endpoint needs NO TLS configuration — the exact production `stream_endpoint()` adds `tls_config` because `STREAM_ENDPOINT_URL` is `https://`. The test's `http://127.0.0.1:1` endpoint therefore builds and connects without any TLS configuration (§C).

### §A.1 New reason constant — `src/youtube/stream_client.rs`

Insert into the existing fixed-reason constants group (after `STREAM_CONNECT_FAILURE_REASON`, line 31; keeping `STREAM_ENDPOINT_BUILD_REASON` last):

```rust
pub(super) const STREAM_READY_FAILURE_REASON: &str =
    "the stream endpoint readiness reserve failed";
```

- Group doc comment ("Fixed stream family failure reasons (secret-safe: no server text).") already covers it; no new doc lines needed.
- Exact text pinned above: secret-safe, no dependency wording (no "tonic"/"tower"/"buffer"), actionable class ("readiness reserve failed").
- Visibility `pub(super)` mirrors the two sibling connect constants (needed by the child roster §C).

### §A.2 Mapping helper — `src/youtube/stream_concrete.rs`

Add next to the existing two helpers (after `stream_connect_failure_error`, exact mirror shape):

```rust
fn stream_ready_failure_error() -> LiveApiError {
    LiveApiError::Network {
        reason: STREAM_READY_FAILURE_REASON.to_string(),
    }
}
```

Extend the `use super::stream_client::{ ... }` import braces with `STREAM_READY_FAILURE_REASON` (one token; keep the existing brace order).

### §A.3 The exact `open()` refactor — `src/youtube/stream_concrete.rs`

DECIDED shape: extract a private inherent async method `open_with_endpoint(&mut self, endpoint, command)` (2 params beyond the receiver ⇒ ≤2-args rule compliant; the receiver IS the object, the same counting the sanctioned seam `open(&mut self, request)` uses). `open()` keeps its production shape identical except delegating endpoint construction. The child test roster (§C) reaches `open_with_endpoint` through the same-module private visibility (the `#[path]` child module's rights — the `stream_client_tests.rs` `use super::*` precedent).

After-code (replace the current `open` body; keep the `#[tonic::async_trait] impl ChatStreamTransport` block):

```rust
#[tonic::async_trait]
impl ChatStreamTransport for GoogleLiveChatStreamTransport {
    async fn open(
        &mut self,
        command: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        let endpoint = stream_endpoint()?;
        self.open_with_endpoint(endpoint, command).await
    }
}
```

Inside the EXISTING `impl GoogleLiveChatStreamTransport { pub(crate) fn new(..) .. }` block, append:

```rust
    /// The one establishment composition (the test-constructed endpoint
    /// pins it offline): bounded connect → readiness reserve → the
    /// server-streaming call.
    async fn open_with_endpoint(
        &mut self,
        endpoint: tonic::transport::Endpoint,
        command: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        let bearer = bearer_token_metadata(&self.access_token)?;
        let channel = tokio::time::timeout(
            Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
            endpoint.connect(),
        )
        .await
        .map_err(|_| stream_connect_timeout_error())?
        .map_err(|_| stream_connect_failure_error())?;
        let mut grpc = tonic::client::Grpc::new(channel);
        grpc.ready()
            .await
            .map_err(|_| stream_ready_failure_error())?;
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
```

Hard rules encoded:

- The failure mapping arm is EXACTLY `.map_err(|_| stream_ready_failure_error())?` — the channel's own error value is DISCARDED (`|_|`), its text is NEVER appended (secret-safety identical to the `map_err(|_| stream_connect_*)` arms).
- The insertion point of the reserve is EXACTLY between `tonic::client::Grpc::new(channel)` and the `server_streaming` call. Nothing else changes: the fresh-Endpoint-per-open decision, the tokio connect-timeout wrapper, the Bearer metadata, the classification, and the session construction are byte-stable.
- `grpc` is bound `let mut grpc` (ready takes `&mut self`).
- Module doc header amendment (honesty of the edited file's own doc; Task 2 owns the docs/ and tech.md records): replace the closing sentence of the header ("The handshake stays untested offline; the operator manual E2E is the only real-Google validation (record P7-12 limitation).") with:

  `//! The establishment shape is pinned offline against a dead plaintext loopback endpoint (the readiness reserve anchor and fix-shape pins); the real-Google TLS/handshake behavior remains the operator manual E2E (record P7-12 limitation, amended 2026-10-30).`

- Expected post-edit file size: ≈ 95–100 lines (cap 200 holds with wide margin).

## §B FIX 2 — the contained panic funnel in `run_chat_worker`

File: `src/app/run/chat_worker.rs`. Three edits, exact after-code:

### §B.1 The new fixed detail constant

Placed directly next to `CHAT_RUNTIME_START_FAILURE` (exact text; secret-safe, no dependency wording, no payload echo):

```rust
/// The contained-worker-panic terminal detail (2026-10-30 fix); the panic
/// payload is never read — the default hook keeps its backtrace on stderr.
const CHAT_WORKER_PANIC_DETAIL: &str =
    "the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1";
```

The composed error line the operator will see: `error: live chat transport failed after repeated attempts: the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1` (the `Transport` Display prefix is the existing untouched template in `src/app/pipeline/source_error.rs:68-73`).

### §B.2 The Transport-detail factory

Add the tiny factory directly after the constant (single definition — it replaces the duplicated Transport literal at BOTH terminal arms; no doc comment: the name is self-documenting):

```rust
fn transport_terminal(detail: String) -> ChatRetrievalError {
    ChatRetrievalError::Transport { detail }
}
```

- The catch shape is encoded in §B.3 directly (NO extracted helper; there is no `loop_unwound` fn in this plan).
- The runtime keeps building OUTSIDE the catch (the `chat_runtime()` result arm is a typed terminal, not a panic path).

### §B.3 The `run_chat_worker` tail (exact replacement)

Replace the existing single line `runtime.block_on(run_stream_loop(StreamWorker::new(plan, live_chat_id)));` (line 130) with:

```rust
    let terminal = Arc::clone(&plan.terminal);
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(run_stream_loop(StreamWorker::new(plan, live_chat_id)));
    }))
    .is_err();
    if unwound {
        record_terminal(&terminal, transport_terminal(CHAT_WORKER_PANIC_DETAIL.to_string()));
    }
```

And replace the runtime-start arm's record block (`record_terminal(&plan.terminal, ChatRetrievalError::Transport { detail: CHAT_RUNTIME_START_FAILURE.to_string() });` multi-line structure, lines 122-127) with the single line:

```rust
        record_terminal(&plan.terminal, transport_terminal(CHAT_RUNTIME_START_FAILURE.to_string()));
```

Hard rules encoded:

- The catch shape: `catch_unwind` wraps the `block_on` CALL (the whole loop future, driven by the confined runtime inside the inline closure) — NOT inside any loop body. The payload is never bound or read: `.is_err()` discards it cleanly (decided catch-shape note; a `let payload = _` discard is NOT used).
- `AssertUnwindSafe` is fully-qualified inline (`std::panic::AssertUnwindSafe`) — NO new `use` lines anywhere in `chat_worker.rs` (`ChatRetrievalError`, `record_terminal`, `Arc`, `StreamWorker`, and `tokio` are already in scope via the existing imports; `std::panic` is fully-qualified).
- The join arm in `src/app/run/gate.rs` (line 188, `let _ = worker.join()`) is UNTOUCHED. A contained panic resumes normal thread exit; the JoinError channel remains for gate semantics; the funnel (terminal cell → `ChatSource` → `ChatSourceError::Retrieval` → `RunError::ChatRetrieval` → one redacted `error:` line + exit 1) now covers every panic.
- Semantics per the User Ruling: TYPED FATAL — the panic writes the terminal and the run exits 1; the "keep streaming + warn" alternative is rejected; the panic payload text never reaches the terminal (fixed constant only).
- `StreamWorker::new(plan, live_chat_id)` is constructed at the catch point (after the `terminal` Arc is cloned out — the clone happens BEFORE `plan` is moved).
- ChatEnded/offlineAt graceful paths, backoff/budget logic, and every pre-existing line outside §B.1-§B.3 are byte-unchanged.

### §B.4 Line-cap ledger and condensation ladder

Current `chat_worker.rs` = 187 lines; expected post-edit ≈ 199-201. Opening ledger: constant +4 (2 doc + 2 decl), factory +3 (+1 blank), tail +8, runtime-start arm −5, registration +3 (+1 blank) ⇒ ≈ 201.
If rustfmt lands the file OVER 200: apply the ladder IN ORDER, stopping as soon as `wc -l` shows ≤ 200 (each step is mechanical, semantics-free):
- L1 (−1): the constant's 2 doc lines → the ONE line: `/// The contained-worker-panic terminal detail (2026-10-30 fix); the payload is never read.`
- L2 (−1): reflow the tail from the `let unwound … .is_err(); if unwound { … }` form (9 lines) to the equivalent 7-line `if let Err(_) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { runtime.block_on(run_stream_loop(StreamWorker::new(plan, live_chat_id))); })) { … }` form — same payload-free discard via the `Err(_)` pattern.
The 200-line cap MUST hold; if the ladder ends and the file is still over, STOP with the exact counts (no unilateral reshapes). `cargo fmt --check` is the formatting authority after each ladder step.

### §B.5 The panic-roster registration

Append at the end of `chat_worker.rs` (after the `gate_chat_tests` registration, mirroring the established block shape):

```rust
#[cfg(test)]
#[path = "chat_worker_panic_tests.rs"]
mod chat_worker_panic_tests;
```

## §C THE THREE OFFLINE PINS

### §C.0 Evidence recorded for the pin mechanics (access dates 2026-10-30)

- `Endpoint::from_shared` accepts an `http://` scheme WITHOUT any TLS configuration (docs.rs + v0.13.1 source per §A.0 item 5); `tls_config` is never called on the test endpoint.
- `connect()` is EAGER; `connect_lazy()` is documented lazy ("does not attempt to connect to the endpoint until first use", tonic v0.13.1 endpoint.rs). On a dead loopback port:
  - `connect_lazy()` returns a `Channel` immediately (no I/O); the first poll of the server-streaming call future invokes `Buffer::call` → `PollSender::send_item` → the deterministic LOCAL panic, before any connection write. No `tokio::time` timeout is needed in the anchor — the panic path is synchronous on a current-thread runtime (decided: deterministic as-is).
  - `connect()` to `127.0.0.1:1` is refused quickly by the loopback stack (no route escalation); the 10 s bound (`CHAT_STREAM_CONNECT_TIMEOUT_MS`) is kept on the test endpoint for faithfulness AND the fix-shape test additionally wraps the whole helper future in a TEST-ONLY `tokio::time::timeout(CHAT_STREAM_CONNECT_TIMEOUT_MS)` guard (decided unblocking shape; if the establishment were ever pending past the bound the test fails with the guard's message rather than hanging).
- No listener, no Google contact, no external address anywhere (the raw TCP would only ever target the loopback's port 1; with `connect_lazy` nothing is even attempted).

### §C.1 Pin (a) — `establishment_anchor_unread_shape_panics_locally`

Home: NEW file `src/youtube/stream_concrete_tests.rs` (see §C.4). The pre-fix shape is reproduced in-test (the 411 incident-shape anchor precedent, `google_live_api_transport_bind_checks_tests.rs`): after the fix NO production path runs reserve-less, so the in-test reproduction is the only honest wire-negative pairing.

Exact test body:

```rust
/// The incident anchor (TODO Task-1 bullet (a), offline): the PRE-FIX call
/// shape — `Grpc::new(channel)` + `server_streaming(..)` WITHOUT the
/// readiness reserve — against a dead plaintext loopback endpoint. The
/// channel is built with the documented-lazy `connect_lazy` so the call
/// path (and its local panic) is reached with zero network I/O; the
/// reserve-less Buffer call panics locally exactly as the incident logged
/// (`send_item called without first calling poll_reserve`, tower-0.5.3
/// buffer/service.rs, operator-supplied `logs/run/20261030-211225.log`).
/// The literal panic text is deliberately NOT asserted (brittle across
/// tower patches); the anchor pins the unwinding itself, and the exact
/// text lives in the incident record / the plan's evidence block.
#[test]
fn establishment_anchor_unread_shape_panics_locally() {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        stream_test_block_on(async {
            let endpoint = dead_loopback_endpoint();
            let channel = endpoint.connect_lazy();
            let mut grpc = tonic::client::Grpc::new(channel);
            let _ = grpc
                .server_streaming(
                    stream_request(
                        stream_list_request(&fixture_command()),
                        bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
                            .expect("the fixture token is header-safe"),
                    ),
                    PathAndQuery::from_static(STREAM_LIST_PATH),
                    tonic::codec::ProstCodec::default(),
                )
                .await;
        });
    }));
    assert!(
        outcome.is_err(),
        "the pre-fix shape must unwind locally without the readiness reserve"
    );
}
```

- Assertion decision (recorded): `is_err`-only — matching the literal panic message is REJECTED as brittle across tower patches; the exact incident text stays in §C.0/§A.0 and the incident record (Task 2 owns the tracked report).
- `let _ =` on the awaited result: the expected value is the local unwind (the future body always panics before producing a value); the discard keeps the anchor free of type gymnastics.

### §C.2 Pin (b) — `establishment_fixed_shape_reserves_readiness_and_yields_typed_error`

Same file. This drives the EXACT production composition through `open_with_endpoint` with the test-constructed dead endpoint (the sanctioned parameterization).

```rust
/// The fix shape: the EXACT production establishment composition
/// (bearer → bounded connect → `Grpc::new` → the readiness reserve →
/// `server_streaming`) driven through the private `open_with_endpoint`
/// helper with a dead plaintext loopback endpoint. Assert: NO unwinding
/// (the reserve honors the tower Buffer contract; the incident panic
/// cannot recur on this shape) and the outcome is a typed
/// `LiveApiError::Network` under the dead-endpoint stream reason set.
#[test]
fn establishment_fixed_shape_reserves_readiness_and_yields_typed_error() {
    let mut transport = GoogleLiveChatStreamTransport::new(AccessToken::new(ACCESS_TOKEN_FIXTURE));
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        stream_test_block_on(tokio::time::timeout(
            Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
            transport.open_with_endpoint(dead_loopback_endpoint(), fixture_command()),
        ))
    }));
    let attempt = outcome.expect("no panic occurs on the fixed shape");
    let attempt = attempt
        .expect("the dead loopback establishment completes within the pinned connect bound");
    let reason = match attempt {
        Err(LiveApiError::Network { reason }) => reason,
        Err(other) => panic!("a dead endpoint yields the Network class, got {other:?}"),
        Ok(_) => panic!("a dead endpoint cannot establish a stream session"),
    };
    assert!(
        DEAD_ENDPOINT_STREAM_REASONS.contains(&reason.as_str()),
        "the observed reason '{reason}' is a pinned dead-endpoint stream reason"
    );
}
```

With the file-local constant:

```rust
const DEAD_ENDPOINT_STREAM_REASONS: [&str; 3] = [
    STREAM_CONNECT_TIMEOUT_REASON,
    STREAM_CONNECT_FAILURE_REASON,
    STREAM_READY_FAILURE_REASON,
];
```

- Acceptance semantics (recorded): the observed reason is NEVER hardcoded as an expectation beyond membership; expected observation on this test = `STREAM_CONNECT_FAILURE_REASON` (the dead port's eager connect is refused before the reserve arm could fire); `STREAM_READY_FAILURE_REASON` is a member because the helper can lawfully surface it on an established-but-unready endpoint. The gate §E records the observed value in the §L addendum.
- Note recorded for the adherence pass: with a dead endpoint the eager `connect` fails first, so the reserve ARM is exercised by composition rather than by observation; the reserve's contract (reserve-less call panics locally) is pinned by pin (a) itself — together the pair proves the incident shape is gone from the production composition.

### §C.3 Pin (c) — `worker_panicking_transport_writes_the_terminal`

Home: NEW file `src/app/run/chat_worker_panic_tests.rs` (see §C.4). Harness decision (leanest correct form): the worker runs via the confined runtime entry `run_chat_worker(plan)` DIRECTLY on the test thread — the exact `runtime_chat_fakes::run_worker` harness pattern (no spawned thread; the containment makes the thread exit normal, and the funnel-equivalent assertions are the terminal cell + `ChatSource`).

```rust
//! The worker panic funnel roster (2026-10-30 muteness fix): a stream
//! transport whose `open` panics is contained by the worker's catch
//! boundary — the typed terminal is written with the FIXED detail, the
//! calling thread resumes (the process does not abort), the sender drops,
//! and `ChatSource` surfaces the typed retrieval error to the frame
//! funnel (once, then `Ok(None)` forever). Offline only; fixture-shaped.

use std::sync::Arc;

use tonic::async_trait;

use crate::app::pipeline::{ChatRetrievalError, ChatSourceError, MessageSource, StopFlag};
use crate::youtube::{ChatStreamRequest, ChatStreamTransport, LiveApiError, LiveChatStreamSession};

use super::{chat_channel, run_chat_worker, ChatChannel, ChatWorkerPlan, CHAT_WORKER_PANIC_DETAIL};
use super::runtime_chat_fakes::{LIVE_CHAT_ID_FIXTURE, ScriptedRunChatTransport};
use super::source::ChatSource;

const BROADCAST_ID_FIXTURE: &str = "broadcast-id-fixture";

#[derive(Default)]
struct PanickingStreamTransport;

#[async_trait]
impl ChatStreamTransport for PanickingStreamTransport {
    async fn open(
        &mut self,
        _request: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        panic!("panic-fixture: the scripted stream open panics");
    }
}
```

Type-visibility notes (already resolved by the imports above, kept as the implementer's check list):
- The roster is a CHILD of `chat_worker`, so `super::` IS `chat_worker`: `runtime_chat_fakes` is reached via `super::runtime_chat_fakes` (NOT `super::chat_worker::…`); `ChatChannel`/`ChatWorkerPlan`/`chat_channel`/`run_chat_worker`/the constant are direct `super::` items.
- `ChatSource` lives in `super::source`; `ChatStreamTransport`/`LiveChatStreamSession` are `pub(crate)` re-exports at `crate::youtube` (project-structure map); `StopFlag`/`MessageSource` likewise (`crate::app::pipeline`).

Plan factory + helpers + test (all ≤2 params; helpers private to the roster):

```rust
fn panicking_stream_plan(channel: &ChatChannel) -> ChatWorkerPlan {
    let mut lookup = ScriptedRunChatTransport::default();
    lookup.script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
    ChatWorkerPlan {
        lookup: Arc::new(lookup),
        stream: Box::new(PanickingStreamTransport),
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        sender: channel.sender.clone(),
        terminal: Arc::clone(&channel.terminal),
        stop_flag: StopFlag::new(),
    }
}

fn assert_terminal_carries_the_panic_detail(channel: &ChatChannel) {
    let stored = channel.terminal.lock().expect("cell mutex").take();
    let ChatRetrievalError::Transport { detail } =
        stored.expect("a contained worker panic writes a terminal")
    else {
        panic!("the contained panic writes the Transport terminal class");
    };
    assert_eq!(detail, CHAT_WORKER_PANIC_DETAIL);
}

#[test]
fn worker_panicking_transport_writes_the_terminal() {
    let channel = chat_channel();
    run_chat_worker(panicking_stream_plan(&channel));
    assert_terminal_carries_the_panic_detail(&channel);
    drop(channel.sender);

    let ChatChannel {
        receiver, terminal, ..
    } = channel;
    let mut source = ChatSource::new(receiver, Arc::clone(&terminal));
    assert!(matches!(
        source.next_message(),
        Err(ChatSourceError::Retrieval(ChatRetrievalError::Transport { .. }))
    ));
    assert!(matches!(source.next_message(), Ok(None)));
}
```

- Letter assertions encoded: the terminal cell holds `ChatRetrievalError::Transport` with the EXACT `CHAT_WORKER_PANIC_DETAIL` string; the test process does not abort (test returns = contained); the sender drops to `Disconnected`; `ChatSource::next_message` (the CRATE surface) surfaces `ChatSourceError::Retrieval(ChatRetrievalError::Transport{..})` once, then `Ok(None)`.
- Discrimination guarantee: with this plan the ONLY Transport-class detail reachable is this fixed string — a resolution failure would be `ChatUnavailable` and a runtime-start failure would carry `CHAT_RUNTIME_START_FAILURE`, so the exact-detail assert binds the roster to the panic funnel deterministically.
- The panic print inside the test reruns the default hook (stderr noise is accepted; no `set_hook` precedent exists in the repo and none is introduced).
- Roster drift note: rosters that panic INSIDE the confined loop would now surface as the wrong-state terminal instead of a direct panic; the existing rosters' asserts are the failure detector — expected zero behavior change for non-panicking paths.

### §C.4 Roster homes and registration (decided)

- Pins (a)+(b): NEW sibling file `src/youtube/stream_concrete_tests.rs` registered from `stream_concrete.rs` via the established `#[cfg(test)] #[path]` pattern appended at the end of that file:

  ```rust
  #[cfg(test)]
  #[path = "stream_concrete_tests.rs"]
  mod stream_concrete_tests;
  ```

  Imports at its top: `use std::panic::{catch_unwind, AssertUnwindSafe};` + `use super::*;` (the `stream_client_tests.rs` precedent — it exposes the parent's items and its imports: transport, `AccessToken`, `bearer_token_metadata`, `stream_request`, `stream_list_request`, `PathAndQuery`, `LiveApiError`, `CHAT_STREAM_CONNECT_TIMEOUT_MS`, `STREAM_CONNECT_TIMEOUT_REASON`, `STREAM_CONNECT_FAILURE_REASON`, `STREAM_READY_FAILURE_REASON` (after §A.2's import extension), `STREAM_LIST_PATH`, `Duration`) + `use crate::youtube::stream_test_block_on;` + `use crate::youtube::ACCESS_TOKEN_FIXTURE;`.
  File-local helpers (private, ≤0/1 args): `dead_loopback_endpoint()` and `fixture_command()` EXACTLY as in §C.5 below.
  Expected size ≈ 130-150 lines (cap 200 holds).

- Pin (c): NEW sibling file `src/app/run/chat_worker_panic_tests.rs` registered from `chat_worker.rs` (§B.5). Expected size ≈ 95-110 lines (cap 200 holds). NOT placed in any existing roster (the roster stays panic-focused; the re-shape/simplifier may only touch it under its own 4.3 cycle).

### §C.5 The stream-roster file-local helpers (exact)

```rust
const LOOPBACK_DEAD_URI: &str = "http://127.0.0.1:1";

/// The dead plaintext loopback endpoint: `http://` scheme (no TLS
/// configuration), the pinned 10 s connect bound for faithfulness, and a
/// dead port so no network I/O can complete anywhere in the roster.
fn dead_loopback_endpoint() -> tonic::transport::Endpoint {
    tonic::transport::Endpoint::from_shared(LOOPBACK_DEAD_URI.to_string())
        .expect("the plaintext dead loopback endpoint parses")
        .connect_timeout(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS))
}

fn fixture_command() -> ChatStreamRequest {
    ChatStreamRequest {
        live_chat_id: crate::youtube::LIVE_CHAT_ID_FIXTURE.to_string(),
        page_token: None,
    }
}
```

(The `crate::youtube::LIVE_CHAT_ID_FIXTURE` exact path governs even where the `use super::*` glob also names it — the explicit path wins and the glob stays untouched.)

## §D REGRESSION LEDGER (verify at the gate; annotate in §L any drift)

- `demo`: untouched (`src/app/demo/*`, streaming local vectors) — roster set re-pinned green by the gate.
- RTMPS argv vectors / redaction / exit tables: untouched (`RunError` mapping, `report_failure`, `RUN_STOP_MESSAGE` paths unchanged).
- The single-`error:`-line contract: COMPLEMENTED by this fix (the mute case gains the guaranteed single line); no second error surface added.
- `open()` production shape: unchanged except the inserted `ready()` reserve; the fresh-Endpoint-per-open decision, the connect timeout, the Bearer form, and classification byte-stable.
- `gate.rs` untouched; `chat_stream_worker.rs` / `chat_stream_delivery.rs` untouched; `chat_failure.rs`'s CLOSED transient list untouched (Network is already transient — no arm added).
- ChatEnded/`offlineAt` graceful paths untouched (worker flow/budget rosters re-verified green).
- Zero NEW `#[allow(` (expected ZERO additions; see §F for the ladder if a lint fires).
- No test contacts Google or any external address (loopback `127.0.0.1:1` only, lazy no-I/O); no secrets (only `ACCESS_TOKEN_FIXTURE`).
- New files: exactly 2 test rosters (`stream_concrete_tests.rs`, `chat_worker_panic_tests.rs`) + 3 changed sources (`stream_concrete.rs`, `stream_client.rs`, `chat_worker.rs`). NO other path changes.

## §E COMMANDS, VERIFICATIONS, COMMITS (implementer; container-only via the Alpine VM MCP)

Pre-flight (record output in §L): `git status` (clean except the untracked plan), `git log --oneline -2` (branch `fix/stream-establishment-readiness`, v0.13.1 bump present), allow-count snapshot `grep -rn "#\[allow(" src --include="*.rs"` (expected: the pre-existing inventory, unchanged after this task).

1. Plan-tracking commit (BEFORE code): `git add .kilo/plans/20261030-fix-task1-remedy-and-pins.md && git commit -m "chore: track stream establishment readiness fix Task 1 plan"` (per the User Ruling the plan is auto-approved).
2. Apply §A/§B/§C edits exactly.
3. Targeted filters (each a single compose run):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked stream_concrete` → expected: ONLY the 2 new `stream_concrete_tests` pins pass (the name filter matches the new roster's path; the neighboring `stream_client` roster does not contain the substring) — record the OBSERVED output.
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_worker` → the existing chat-worker family (chat_worker_tests 6 + chat_worker_flow_tests 6 + chat_worker_budget_tests 8 + chat_worker_burst_tests 1 + gate_chat_tests 2 = 23) + the NEW funnel pin (1) = 24 expected; the OBSERVED count governs and is recorded.
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` (fix any rustfmt rewrap ONLY mechanically; re-run).
5. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → expected exit 0 with ZERO warnings (see §F if not).
6. FULL gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, four PASS lines, `ALL CHECKS PASSED`; record the log path and the OBSERVED test total (expected ≈ 494 + 3 = 497; the OBSERVED number governs — never claimed as 497 before observation).
7. Windows-gnu check: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked --target x86_64-pc-windows-gnu` → exit 0 (NO release builds — Task 2).
8. Hygiene: `git status --porcelain` must list ONLY the intended paths (3 sources + 2 rosters + the plan file); `grep -rn "#\[allow(" src --include="*.rs"` unchanged from the pre-flight snapshot.
9. Code+tests commit: `git add src/youtube/stream_concrete.rs src/youtube/stream_concrete_tests.rs src/youtube/stream_client.rs src/app/run/chat_worker.rs src/app/run/chat_worker_panic_tests.rs && git commit -m "feat: reserve stream endpoint readiness and contain chat worker panics"` (message may be adjusted to the branch convention; do NOT amend).
10. Optional addendum commit (ONLY if §L gained observations): `docs: record the readiness-fix task-1 plan addendum`.
    Task-2 scope stays untouched (docs/, builds, report, project-structure re-cut, tech.md/context.md — in the §I ledger).

## §F CLIPPY POSTURE (decided)

- Expected lint set: ZERO clippy findings. `catch_unwind`/`AssertUnwindSafe`/`panic!`-in-tests are NOT in any default-enabled clippy group (`panic_in_result_fn` is pedantic/off-by-default; `panic` is restriction; the gate runs default lints). The fake's `panic!` sits in a `Result`-returning async fn inside `#[cfg(test)]` — still not default-enabled.
- If the gate DOES fire a lint: preferred handling is a rule-following re-shape (e.g., if `clippy::panic_in_result_fn` fires on the roster fake, move the panic into a `#[cold]` private helper returning `Box<dyn LiveChatStreamSession>` and call it). If NO rule-abiding shape exists, the single compiler-forced adaptation precedent (`Box<ureq::Error>` under the `-D warnings` gate, HTTP-411 fix) applies: ONE allow with the exact lint name in a one-line justification comment, recorded in §L with the lint name. Zero unconditional furthermore allow additions.

## §G STOP PROCEDURE

STOP with evidence + options (to the caller) ONLY on: (1) the anchor pin fails against a current tower/tonic patch (the reserve-less call no longer unwinds — a contract shift) — options: extend the accepted tower range with the caller's ruling, or re-pin with an error-return contract; (2) the fix-shape pin observes a reason OUTSIDE the membership set; (3) a cap/lint condensation reaches the ladder end and still violates a rule; (4) any gate failure not attributable to a mechanical rewrap. The verified mechanics in §A.0/§C.0 do NOT block: the eager-`connect` discovery REDIRECTS pin (a) to `connect_lazy` and does not contradict the bound diagnosis.

## §H Compliance mapping (Task-1 bullets → sections; §0 table is the short form)

| TODO Task-1 bullet / criterion | Covered by |
|---|---|
| Fix 1: reserve step + fixed non-secret reason + Network/transient class | §A.1 (constant), §A.2 (helper), §A.3 (reserve + `map_err(|_| …)`) |
| Fix 2: containment, existing terminal, fixed detail, no payload text, fatal exit 1 | §B.1-§B.3 |
| `open()` shape byte-stable; endpoint parameterized only for pins | §A.3 (helper split), §C.5 (test endpoint) |
| Pin (a): pre-fix shape panics locally (no listener, no Google) | §C.1 (+ §C.0 evidence) |
| Pin (b): fixed shape, no panic, typed Network on a dead endpoint | §C.2 |
| Pin (c): faked panic → terminal written → ChatSource surfaces | §C.3 |
| Gate + windows-gnu check; observed counts recorded; no builds | §E.6-§E.7, §L duties |
| Regression locks (demo/RTMPS/redaction/exit/ChatEnded/`allow(`-count) | §D |
| No test contacts Google; no secrets | §D, §C.0 |

## §I NOT-DONE LEDGER (Task 1 leaves for TODO Task 2 — never claimed here)

- Docs record: `docs/production-run.md` diagnostics/recovery wording; limitation-revision scope (the offline-coverage note); README test count (497 → the OBSERVED gate number, refreshed at Task 2); `docs/build*` untouched.
- Metadata: `.agent/project-structure.md` re-cut (the 2 new rosters + changed sources); `tech.md` (the `ready()` contract + containment note); `context.md` (incident + fix record with operator attribution).
- Tracked completion report `.agent/reports/` (incident facts + log citation + root-cause citations + offline-pin evidence + the §A.0/§C.0 access-dated research).
- Gate on the FINAL tree (Task 1's gate is on its own tree; Task 2 re-runs); BOTH release builds + `ls -l` sizes; hygiene/secret audit across the full TODO.
- The live observation note: the ready-failure arm's real-Google behavior remains the operator manual E2E.

## §L ADDENDUM (initially empty — §E duties append here)

<!-- IMPLEMENTER: append dated observations — targeted-run outputs (observed counts/reasons), the gate log path,
the OBSERVED 494+n total, any lint ladder usage, and corrections to this plan's snippets discovered while editing.
NO other section may be modified; the pre-analysis above is the binding record. -->

### L.1 Pre-flight (2026-10-30, implementer)

- Branch `fix/stream-establishment-readiness`; HEAD `6151fa0` (v0.13.1 bump lineage); tree clean except the untracked plan file.
- `#[allow(` inventory snapshot: 9 entries (`app/auth/mod.rs:86`, `app/pipeline/source_error.rs:23`,
  `app/run/chat_stream_delivery.rs:173`, `chat/store.rs:123`, `youtube/authorization.rs:35` + `:93`,
  `youtube/error.rs:24`, `youtube/mod.rs:76` + `:81`). Unchanged by this task (no additions).
- Plan-tracking commit: `cf258f2 chore: track stream establishment readiness fix Task 1 plan`.

### L.2 Line-cap STOP and the caller ruling (Option A)

- First landing of §A/§B/§C: `chat_worker.rs` = 208 after rustfmt (two §B snippets rewrap at the 100-column
  limit: the runtime-start arm 1→4 lines, the tail `record_terminal` 1→4 lines). Ladder L1 (−1) → 207,
  L2 (−2) → 205 ⇒ cap still violated ⇒ STOP per §B.4/§G(3) (reported to the caller, no commit).
- Caller ruling (Option A, with a bounded release valve), applied:
  - `CHAT_WORKER_PANIC_DETAIL` + `transport_terminal` MOVED from `chat_worker.rs` into
    `chat_stream_delivery.rs` beside `record_terminal` (`pub(super)` both; `chat_worker.rs` imports them in the
    existing `use super::chat_stream_delivery::{...}` line — no new line).
  - `chat_worker.rs` 205 → 197; `chat_stream_delivery.rs` 193 → 201; the moved block's single doc line trimmed
    (the ruling's authorized lever) → 200. Code never compressed beyond the plan's ladder.
  - Clippy then fired `clippy::redundant_pattern_matching` on the L2 `if let Err(_) = …` form; handled by a
    rule-following re-shape (NOT an allow): L2 reverted to the plan's canonical §B.3 shape
    (`let unwound = std::panic::catch_unwind(…).is_err(); if unwound { … }`) → `chat_worker.rs` 199.
  - Final: `chat_worker.rs` = 199, `chat_stream_delivery.rs` = 200 (module-doc release valve NOT used).
- `gate.rs` untouched (`let _ = worker.join()` intact).

### L.3 Plan-snippet corrections discovered while editing (mechanical)

1. §C.2: `tokio::time::timeout` must be CONSTRUCTED inside the runtime (eager construction panics
   "there is no reactor running"); the fix-shape pin wraps it in `async { … }.await` inside
   `stream_test_block_on`.
2. §C.1/§C.2: `let _ = grpc.server_streaming(..)` is ambiguous (E0282); the anchor pins `M2` through
   `.map(|response| TonicChatStreamSession::new(response.into_inner()))` (the production constructor).
3. §C.3: `use super::source::ChatSource` is invalid from the roster (its `super` IS `chat_worker`); corrected to
   `use super::super::source::ChatSource`.
4. §C.3: `let mut lookup` → `let lookup` (`ScriptedRunChatTransport::script_lookup` takes `&self`; avoids
   `unused_mut`).
5. §C.3: the helper's `.take()` consumed the terminal before the `ChatSource` assertion; changed to a
   non-consuming `as_ref()` peek with `detail.as_str()` so the cell survives for the source assertion.

### L.4 Observed verification (container, 2026-10-30)

- `cargo fmt --check` → exit 0.
- `cargo test --locked stream_concrete` → **2 passed / 0 failed** (495 filtered ⇒ 497 total); both pins
  (`establishment_anchor_unread_shape_panics_locally`,
  `establishment_fixed_shape_reserves_readiness_and_yields_typed_error`) ok.
- Pin (b) OBSERVED reason member: **`the stream endpoint connection failed`** (`STREAM_CONNECT_FAILURE_REASON`) —
  inside the pinned 3-member set (the eager connect is refused before the reserve arm can fire, as §C.2 records).
- `cargo test --locked chat_worker` → **24 passed / 0 failed** (23 pre-existing + the new
  `worker_panicking_transport_writes_the_terminal`).
- `cargo test --locked chat_stream` (delivery/mapping family filter) → **12 passed / 0 failed**.
- `cargo clippy --locked -- -D warnings` → exit 0, zero warnings, zero new `#[allow(`.
- FULL GATE `sh scripts/dev-checks.sh` → exit 0; `PASS fmt-check/check/test/clippy`; **ALL CHECKS PASSED**;
  log `logs/checks/20261030T220937Z.log`; OBSERVED **497 passed / 0 failed** (= 494 baseline + 3 new pins).
- `cargo check --locked --target x86_64-pc-windows-gnu` → exit 0. NO release builds (Task 2).

### L.5 Commits and final line counts

- `cf258f2` plan tracked; `5190145 feat: reserve stream endpoint readiness and contain chat worker panics`
  (6 files: 3 changed sources + the ruled `chat_stream_delivery.rs` relocation + 2 new rosters).
- Final `wc -l`: `chat_worker.rs` 199, `chat_stream_delivery.rs` 200, `stream_concrete.rs` 95,
  `stream_concrete_tests.rs` 107, `chat_worker_panic_tests.rs` 76, `stream_client.rs` 97 (all ≤ 200).
- Post-fold (`2873692`, the §OBS fold ruled in by the caller): `chat_stream_delivery.rs` 200 → **195**
  (rustfmt kept the folded `record_terminal(&self.plan.terminal, transport_terminal(last_detail));` on ONE
  line — the §OBS 200→198 projection did not bind); every other count above unchanged; the governing
  post-fold gate is **497 passed / 0 failed** (`logs/checks/20261030T222042Z.log`, four PASS lines).
- Task-2 scope untouched (docs/, builds, report, project-structure, tech.md, context.md).

(End of plan — §L addendum entries follow this line when recorded by the implementer.)
