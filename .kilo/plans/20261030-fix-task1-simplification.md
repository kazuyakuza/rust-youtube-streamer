# Task-1 Simplification Review (workflow step 4.3, simplifier half)

> Binding inputs: plan `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` (§A/§B/§C staged contracts),
> TODO 20261030 Task 1, `.kilo/rules/` code rules. Reviewed tree: branch
> `fix/stream-establishment-readiness`, HEAD `2e89cc3`, gate 497/0, clippy 0.
> Read-only review; NO edits made; NO commit.

## VERDICT: SIMPLIFICATION NOT REQUIRED — zero mandated edits

Every fold candidate found either (a) breaks a staged plan contract (§A/§B/§C bodies and hard rules),
(b) touches §D-locked pre-existing production lines beyond Task-1 scope, or (c) would obscure a
contract pin. One adjudication-optional observation is recorded in §OBS (NOT ordered).

## SCAN MATRIX

| # | Candidate | Location | Fold shape considered | Decision | Contract-safety rationale |
|---|---|---|---|---|---|
| 1 | `stream_ready_failure_error` + 2 `stream_connect_*_error` siblings → one 1-arg reason builder | `stream_concrete.rs:64-80` | `fn stream_network_failure_error(reason: &'static str)`; 3 call sites rewrap | **NOT COMPLIANT — rejected** | §A.2 stages the new helper with "exact mirror shape" (zero-arg body); §A.3 hard rule stages the reserve arm EXACTLY `.map_err(\|_\| stream_ready_failure_error())?` — a builder call changes that staged text; §D pins `open()` production shape "unchanged except the inserted ready() reserve" — the two connect arms (`stream_concrete.rs:46-47`) are byte-stable locks. Folding breaks §A.2, §A.3 and §D simultaneously. |
| 2a | `transport_terminal` × `record_terminal` merge (e.g. `record_transport_terminal(cell, detail)`) | `chat_stream_delivery.rs:181,190`; callers `chat_worker.rs:122-125,134-137` | single store-and-build fn | **NOT COMPLIANT — rejected** | No redundancy in the relationship: one BUILDS (return value), the other STORES (`Mutex` write). Their composition at both arms is the §B.3 exact staged line text; merging changes §B.2's staged factory and both §B.3 staged arms. Zero-semantic-change is impossible to prove without violating staged bodies. |
| 2b | `transport_terminal` vs the inline `ChatRetrievalError::Transport { detail }` literal in `await_reestablishment` | `chat_stream_delivery.rs:128-133` | `record_terminal(&self.plan.terminal, transport_terminal(last_detail));` (rustfmt rewraps ~6→4 lines; 200→~198) | **ADJUDICATION-ONLY — see §OBS; NOT ordered** | Semantically identical construction (same variant, same `detail`, single call; budget test `chat_worker_budget_tests.rs:28-39` asserts the produced value, unaffected). BUT the literal predates Task-1 (AMENDMENT-2 split lineage) and §D locks "chat_stream_delivery.rs untouched"; §B.2 staged the factory's scope as "replaces the duplicated Transport literal at BOTH terminal arms" — i.e. the two `chat_worker.rs` arms only. Folding here expands Task-1's diff beyond its locks → caller ruling required, junior-implementer has no authority for it. |
| 3 | `stream_concrete_tests.rs` roster fold (anchor + fixed-shape shared boilerplate) | `stream_concrete_tests.rs:50-74 / 84-107` | shared `catch_unwind` wrapper helper / shared establishment block | **NOT COMPLIANT — rejected** | Sharing is already maximized within the plan: both tests reuse the §C.5 helpers `dead_loopback_endpoint()` and `fixture_command()`; the remaining shared text is only the `catch_unwind(AssertUnwindSafe(\|\| stream_test_block_on(async …)))` scaffolding, whose inner shapes MUST stay DISTINCT and INLINE — §C.1/§C.2 stage the exact bodies (plus the L.3.1/L.3.2 corrections). FLAG (per scope): any fold moving the anchor's `connect_lazy → Grpc::new → server_streaming` sequence behind a helper obscures the visibly reserve-less pre-fix shape — the anchor's whole evidentiary value is that a reader sees NO `ready()` call in it. Rejected outright. |
| 4 | `chat_worker.rs` residual duplication (`record_terminal` 3 sites; Arc clone) | `chat_worker.rs:117,122,134` | partial-application helper for the cell | **REJECTED** | All three arms are §B.3/§B.2 staged exact text (or pre-existing §D-locked line 117); the `terminal` clone is required by the plan's move ordering (§B.3 hard rule "the clone happens BEFORE `plan` is moved"). Nothing foldable. |
| 5 | `stream_client.rs` / `chat_worker_panic_tests.rs` | whole files | misc | **NOTHING FOUND** | `stream_client.rs`: the task added only the §A.1 staged constant (group ordering honored, `pub(super)` mirroring siblings — §A.1 explicit). Panic roster: exact §C.3 body as corrected by L.3.3–L.3.5; helpers ≤1 arg; no duplication with existing rosters (its `PanickingStreamTransport` fake is deliberately distinct from `ScriptedRunChatTransport`). |

## LINE-COUNT & RULE VERIFICATION (actuals, working tree)

| File | Lines (≤200) | Max fn body (≤50) | Args (≤2) | Block depth (≤2) | Notes |
|---|---|---|---|---|---|
| `src/youtube/stream_client.rs` | 97 ✓ | ≤8 ✓ | ≤2 ✓ | ≤2 ✓ | only +1 const this task |
| `src/youtube/stream_concrete.rs` | 95 ✓ | `open_with_endpoint` 22 ✓ | 2 (endpoint, command; receiver excluded per §A.3 counting) ✓ | ≤2 ✓ (all `?`/`map_err` chains, no nesting) | doc sentence = §A.3 amended text ✓ |
| `src/app/run/chat_worker.rs` | 199 ✓ | `run_chat_worker` 31 ✓ | `build_chat_stage` = 3 — PRE-EXISTING sanctioned exception (record D-T3-15, lines 85-87 doc, untouched by Task 1); all task-touched fns ≤1 ✓ | ≤2 ✓ | catch shape = §B.3 canonical (`is_err()` form, L.2 re-shape) ✓ |
| `src/app/run/chat_stream_delivery.rs` | 200 ✓ AT CAP — zero headroom (see §OBS note) | ≤14 ✓ | ≤2 ✓ | ≤2 for all Task-1-touched fns ✓ | `send_stop_aware` depth 3 (loop→match→arm, lines 78-90) — pre-existing D-9 shape §D-locked byte-unchanged; not Task-1 content |
| `src/youtube/stream_concrete_tests.rs` | 107 ✓ | anchor 25 / fix-pin 23 ✓ | helpers 0 args ✓ | async scaffolding reaches indent level 3 (lines 52-68, 87-92) | prescribed: the §C.1/§C.2 exact bodies (incl. L.3 corrections) mandate the closure→async-block shape; rule-exempt by staged contract — NOT reducible without candidate-3's rejected fold |
| `src/app/run/chat_worker_panic_tests.rs` | 76 ✓ | ≤15 ✓ | ≤1 ✓ | ≤2 ✓ | — |

Contracts re-verified intact (untouched by any considered fold): reserve insertion point
(`stream_concrete.rs:48-51`), catch shape (`chat_worker.rs:129-134`), fixed detail/reason strings
(`stream_client.rs:29-32`, `chat_stream_delivery.rs:187-188`), `gate.rs` untouched
(`let _ = worker.join()` — scope confirmed unchanged).

## §OBS — ADJUDICATION-OPTIONAL OBSERVATION (NOT an ordered edit)

`chat_stream_delivery.rs` `await_reestablishment` (lines 128-133) builds the Transport variant inline
while `transport_terminal` — the plan's "single definition" factory — sits 55 lines below in the SAME
file. Fold spec if the caller rules it in (mechanical, zero semantic change):

- Replace lines 128-133
  ```rust
            record_terminal(
                &self.plan.terminal,
                ChatRetrievalError::Transport {
                    detail: last_detail,
                },
            );
  ```
  with the single statement `record_terminal(&self.plan.terminal, transport_terminal(last_detail));`
  (rustfmt authority: `cargo fmt` rewrap at fn_call_width 60 expected ≈4 lines; net −2, file 200→198).
- Gate duty after the change: `cargo test --locked chat_worker` (expect 24), full gate.
- Why not ordered: it edits a §D-locked pre-existing production line (outside Task-1's landed scope);
  §B.2 staged the factory's replacement scope to the two `chat_worker.rs` arms. Caller ruling required
  (same channel as the L.2 Option-A ruling). Alternative: leave for a dedicated cleanup cycle.

Secondary note: the 200-line AT-CAP status of `chat_stream_delivery.rs` is exactly the §L.2 landing;
no Task-1-cycle-internal lever gives headroom without the §OBS fold (module-doc valve already declined
per L.2; further compression forbidden by "Code never compressed beyond the plan's ladder").

## DISPOSITION

No file in Task-1's landed scope carries a compliant simplification. The 4.3 simplifier half closes
with zero edits; the caller may adjudicate §OBS independently. Gate/clippy state is preserved as-is.

(End of simplification review.)
