# Task-2 Implementation Plan — Documentation, Builds, and Tracked Fix Record

> Binding inputs: TODO `.agent/todos/20261030/20261030-todo-1.md` (Task 2 bullets; the Pre-Diagnosis binds the
> incident facts) + global plan `.kilo/plans/20261030-establishment-readiness-fix.md` (incl. User Rulings:
> per-task plans auto-approved; containment = TYPED FATAL exit 1) + Task-1 records:
> `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` (§L carries the OBSERVED values) +
> `.kilo/plans/20261030-fix-task1-adherence.md` (verdict ADHERENT; the 9-allow canonical figure;
> the windows-gnu final-tree re-run duty; the AC flip list 4 HOLD → Task 2).
> Baseline at plan time: branch `fix/stream-establishment-readiness`, v0.13.1 (Task 1 + its 4.6 closed:
> TODO Task 1 `[DONE]`, AC-1–AC-4 `[x]`), post-fold gate **497 passed / 0 failed** (log
> `logs/checks/20261030T222042Z.log`), tree expected clean except the untracked plan file.
> This plan is for the 4.2 implementer (junior-50%): every decision is closed-form. The implementer creates
> NO code, NO tests, NO version changes; `git push` stays workflow step 5; branch switch stays step 2.

---

## §0 TODO Task-2 bullet ↔ owning-section map

| TODO Task-2 bullet (`.agent/todos/20261030/20261030-todo-1.md` § Tasks / Task 2) | Owning section |
|---|---|
| Bullet 1 — coverage audit vs this TODO; add tests only for genuine gaps (expected: none beyond Task 1's pins) | §A (verdict + audit note) |
| Bullet 2 — docs: `docs/production-run.md` (new diagnostic behavior + recovery pointer + limitation-wording revision), README drift, `.agent/project-structure.md` re-cut, `tech.md` notes, `context.md` incident record | §C.1 (production-run), §C.2 (README), §C.3 (structure map), §C.4 (tech.md), §C.5 (context.md) |
| Bullet 3 — tracked completion report in `.agent/reports/` (incident facts + log citation + root-cause citations + fix + offline-pin evidence + commands/statuses + counts + artifact sizes) | §C.6 |
| Bullet 4 — gate fresh on the final tree; rebuild BOTH artifacts (`build-windows.sh` then `build-linux.sh`), `ls -l` cross-check with recorded sizes; hygiene/secret audit | §B |

AC flip duty carried into 4.6: AC-5 (line 60), AC-6 (line 61), AC-7 (line 62) — see §D.8.

## §A COVERAGE AUDIT (verdict — record in the completion report §Test Coverage Audit)

**Verdict: ZERO new automated tests.** The TODO Task-2 bullets define no test expectations: the only
coverage-bearing words in Task 2 are the audit bullet itself. The three offline pins that the fix-phase
needed were landed and gated by Task 1 (`stream_concrete_tests.rs` 2 + `chat_worker_panic_tests.rs` 1;
grep-verified rosters exist and are registered via the `#[cfg(test)] #[path]` pattern). The run-side funnel
(rendering of the single redacted `error:` line + exit 1) is composition-proven by the pre-existing pin
`error_tests.rs::chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate` (zero diff across
the whole fix chain, per Task-1 adherence §7/AC-3) — COVERED-BY-COMPOSITION, no widening performed.

Audit note to carry into the report (recorded, not argued): the 4.2 verification for this task is the FULL
gate on the final tree, not new tests; the honest coverage statement is "the 3-pin offline proof from Task 1
+ the pre-existing funnel/display pins close every AC claim Task 2 must flip (AC-5/AC-6/AC-7)".

## §B GATE + BUILDS + AUDIT (4.2 implementer; container-only via the Alpine VM MCP; `vm_status` FIRST, then `vm_run_command`)

Pre-flight (record outputs in §L): `git status` (clean except the untracked plan), `git log --oneline -8`
(branch `fix/stream-establishment-readiness`, v0.13.1 lineage), allow snapshot
`grep -rn "#\[allow(" src --include="*.rs"` (expected: the 9-entry attribute-exact inventory of Task-1
adherence §6 — 8 × `dead_code` + 1 × `clippy::manual_clamp`; Task 2 adds NOTHING).

Numbered duties (each its own `vm_run_command` invocation):

1. **Doc edits FIRST** (§C) — the gate runs on the FINAL tree only after every §C edit exists.
2. **Plan-tracking commit** (§D.1), then edits, then:
3. **FULL gate fresh on the final tree:**
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   → expected exit 0, four PASS lines (`fmt-check`/`check`/`test`/`clippy`), `ALL CHECKS PASSED`. Record the
   log path and the OBSERVED test total (expected **497** = 494 baseline + Task-1's 3 pins; **the OBSERVED
   number governs — the docs/report never claim 497 until the fresh log says it**). The log dir is
   gitignored and VM-side; record the path from the script print.
4. **Windows-gnu check re-run on the final tree** (Task-1 adherence record-quality note (b)):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked --target x86_64-pc-windows-gnu`
   → expected exit 0; record it (NO release relationship — the .exe build below is the artifact duty).
5. **Both release artifacts — Windows FIRST, then Linux** (the Phase-07 TODO wording precedent the global
   plan sanctions; order recorded, not silently swapped):
   1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`
      → exit 0; `dist/windows/rust-youtube-streamer-service.exe`; record the script-printed size.
   2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`
      → exit 0; `dist/rust-youtube-streamer-service`; record the script-printed size.
   3. **`ls -l` cross-checks:** `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows`
      → BOTH files non-empty; the `ls -l` sizes MUST match the script-printed sizes exactly (record all four).
      NO size is ever invented or copied from Phase-07 — the v0.13.1 builds' OBSERVED sizes govern every
      record in §C.6/§C.5. (Phase-07 historical sizes 12,336,636 / 7,665,712 bytes are citations only, never
      the fix-phase truth.)
6. **Hygiene/secret audit (host-side `bash` tool + one container call):**
   - `git status --porcelain` → ONLY the intended paths of §D.2/§D.3 (+ the plan file / report file).
   - `git ls-files dist videos` → EMPTY both; `git ls-files credentials` → `.gitkeep` only.
   - `git check-ignore -v dist/rust-youtube-streamer-service` → `.gitignore:34:dist/`;
     `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` → same rule; the cited operator
     log path `logs/run/20261030-211225.log` → `.gitignore:18:logs/*` (it is cited as a PLAIN PATH in docs —
     never staged, never read into any doc beyond the path citation).
   - Token greps across `src docs .agent README.md`: `ya29`/`AIza` → ZERO in src/docs/README (the
     `.agent` historical audit-prose mentions are pre-existing, count unchanged); `key=` → the pre-existing
     "never an API key" doc/line set, count unchanged; `rtmps://` → pre-existing `.example.test` fixtures and
     doc-context mentions only, count unchanged.
   - `grep -rn "#\[allow(" src --include="*.rs"` → EXACTLY 9 attribute lines, unchanged.
   - Report self-check: the new report file contains NO secret material (no tokens/keys/URIs; the panic
     quotes are stack text only), NO literal `\n` escape sequences (real newlines per the newline rule), and
     every numeric claim is an OBSERVED value from this pass or a path-cited record.
7. **§L addendum** on this plan: the fresh gate log path + OBSERVED total, the 4 build outputs + sizes, the
   audit results, and any deviating observation. NO other section is modified.

## §C DOC EDITS (exact anchors; the implementer applies the prescribed texts; `cargo fmt` never touches docs)

### §C.1 `docs/production-run.md` (the live operator manual)

Four edits, ALL inside the `## Live Chat (Behavior, Diagnostics, Recovery, Privacy, Quota)` H2. TOC keeps
its existing anchor (`#live-chat-behavior-diagnostics-recovery-privacy-quota` — the H2 text is NOT changed,
so no TOC edit). Everything OUTSIDE the Live Chat H2 is drift-verified NO-EDIT (see the list below).

**Edit C.1.1 — the typed-error bullet amendment** (`### Diagnostics`, the third bullet, currently
`- \`live chat transport failed after repeated attempts: <detail>\` — the bounded transient retry budget exhausted.`).
Replace with:

```
- `live chat transport failed after repeated attempts: <detail>` — the bounded transient retry budget exhausted, or (since v0.13.1) a contained retrieval-worker panic: the fixed detail names the panicked worker and points at the stderr backtrace (see the contained-panic paragraph below).
```

**Edit C.1.2 — the limitation-sentence revision** (`### Diagnostics`, closing sentence of the
"On a connection drop…" paragraph). REPLACE exactly the sentence

```
Honest limitation: the real TLS/gRPC handshake against Google is validated ONLY by the operator E2E below — the offline tests pin everything around it (proto wire bytes, Bearer metadata, status classification, the resumable loop) with scripted offline doubles.
```

with:

```
Honest limitation (wording revised 2026-10-30): the real TLS/gRPC handshake against Google and the cloud-side behavior remain validated ONLY by the operator E2E — the offline tests pin the proto wire bytes, the Bearer metadata, the status classification and the resumable loop as before, and, since the v0.13.1 establishment fix, ALSO the call-shape contract itself: the reserve-less shape is pinned to panic locally and the fixed reserve-then-stream shape is pinned to establish without a panic, against a dead plaintext loopback endpoint (a real Channel object, loopback analog — no test ever contacts Google).
```

Scope ruling for this edit (the TODO's "revise ONLY where true"): the "handshake itself stays untested
offline" claim is revised ONLY for the readiness-reserve contract the two `stream_concrete_tests` pins now
cover on a real (loopback-analog) endpoint object. The real TLS/gRPC handshake and cloud-side behavior
claims STAND (operator E2E). No other sentence in the paragraph is touched ("the operator CANNOT observe
… the handshake itself" remains true and stays).

**Edit C.1.3 — the contained-panic narrative** (`### Diagnostics`, a NEW paragraph appended immediately
AFTER the "On a connection drop…" paragraph, i.e. the last paragraph of the Diagnostics H3):

```
Contained retrieval-worker panic (v0.13.1): a panic inside the chat worker thread no longer ends in a mute failure. On v0.13.0 (operator log `logs/run/20261030-211225.log`, both of its runs — attribution operator-supplied, never agent-observed) the worker's panic print appeared but the run kept streaming, NO `error:` line ever printed, and the run exited 0 with message delivery dead. Since v0.13.1 the worker's panic is contained: the panic's own print — including the backtrace when `RUST_BACKTRACE=1` is set — still appears on stderr from the worker thread, and the run then ends through the normal chat failure funnel with exactly ONE redacted `error:` line naming the panicked worker —
`error: live chat transport failed after repeated attempts: the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1`
— and exit code 1. Retrieval stays final: the fatal chat error ends the run (video stops with it — the incident's mute shape, video alive while delivery is dead, is exactly what this containment removes). Operator recovery: there is no retry-level remedy — a contained panic can only be a defect of the running build; on a current build, re-run reproducing the shape means capture the full stderr panic print plus backtrace text (`RUST_BACKTRACE=1`) and include it in the report you file (never into the repository and never counted as agent-observed evidence).
```

The single terminal-detail line above is VERBATIM from the Task-1 record (§B.1) — recognition text, not
invention. The `RUST_BACKTRACE` hint is sanctioned by the global plan (§7: "Operators may keep using
`RUST_BACKTRACE` for diagnostics").

**Edit C.1.4 — the Failure Recovery table row** (`### Failure Recovery`; insert AFTER the
"Dropped/stalled stream link" row, BEFORE the "Malformed/partial batch" row):

```
| Retrieval-worker panic (contained, v0.13.1) | the panic print on stderr, then the single `live chat transport failed after repeated attempts:` error line naming the panicked worker, then exit 1 (never a silent exit 0 with dead retrieval — the v0.13.0 mute shape is fixed) | fatal (exit 1) | update to the current build and re-run; if the shape repeats on a current build, capture the stderr panic + backtrace (`RUST_BACKTRACE=1`) and report it |
```

**Drift-verified NO-EDIT list (record in §L):** Overview; Prerequisites; the ordered startup section
(stage 7.5 wording stands); the log-lines table (the panic print is NOT a `tracing` event and adds NO row —
the funnel's `error:` line is documented in the Live Chat Diagnostics + Shutdown Behavior surfaces, as
before); Shutdown Behavior; Broadcast/Stream Fate; Privacy; Quota ("What Chat Streaming Costs"); the Ingest
Probe Procedure; the Known Stream-Health note; the Manual Verification Procedure (its chat-bearing steps are
already exact — the incident changes which outcome the operator sees mid-E2E on a fixed build, which the new
Diagnostics + recovery rows now document); Out of Scope; See Also.

### §C.2 `README.md`

Two edits; the Key Technologies dependency bullets STAY VERBATIM (tonic 0.13.1 / prost 0.13.5 / tokio 1.53.1
pins are unchanged by the fix). Decision recorded: **NO separate fix/incident note is added** — README
carries version-agnostic facts + the observed test count only; incident narratives live in
`docs/production-run.md`, `context.md`, and the tracked report.

**Edit C.2.1 — Current Status, Phase-07 sentence** (line 45). Within the sentence that begins `Phase 06
(Live Chat integration) landed, and Phase 07 (streamList migration, v0.13.0) swapped its retrieval to the
official server-streaming \`liveChatMessages.streamList\` gRPC client:`, insert after `gRPC client` (before
the `:`):

```
, and the v0.13.1 fix added the establishment's endpoint readiness reserve and contained retrieval-worker panics (a contained panic now ends the run with its single redacted `error:` line and exit 1 — never silently)
```

The remainder of the sentence (`: the connection is established once per session …`) is unchanged — the
streamList machinery wording stays.

**Edit C.2.2 — the observed test count** (line 47). Replace `494 tests, re-recorded by the Phase 07
verification pass` with `<OBS_GATE_TOTAL> tests, re-recorded by the 2026-10-30 establishment-readiness-fix
pass` — where `<OBS_GATE_TOTAL>` is the FRESH gate total from §B.3 (expected 497; the observed number
governs).

### §C.3 `.agent/project-structure.md` re-cut (conservative, section-scoped)

Itemized edits (verify each count with `git ls-files` before recording; "observed at this pass" replaces
the 2026-10-29 frame on the two touched aggregate lines ONLY):

1. `run/` aggregate line (currently `31 files: 13 sources + 4 fixtures + 14 test rosters`) →
   `32 files: 13 sources + 4 fixtures + 15 test rosters`.
2. `chat_worker.rs` row → append the containment clause:
   `; the contained-panic catch boundary (2026-10-30 fix): std::panic::catch_unwind(AssertUnwindSafe(...)) over the block_on call — a caught panic records the typed terminal via the shared factory and the thread returns while the default hook keeps the backtrace on stderr` — the existing row content is unchanged otherwise.
3. `chat_stream_delivery.rs` row → append:
   `; the 2026-10-30 containment items beside record_terminal: the fixed CHAT_WORKER_PANIC_DETAIL constant + the transport_terminal factory (pub(super), consumed by chat_worker.rs)`.
4. The `run/` roster summary line → the roster list gains `chat_worker_panic_tests 1 (the contained-panic funnel pin)` and the counts become `15 rosters, 59 unit tests (observed at this pass: …)` (fresh-count governs).
5. `youtube/` aggregate (currently `92 files, observed 2026-10-29: 47 sources + 4 fixtures + 41 test rosters`) → `93 files, observed at this pass: 47 sources + 4 fixtures + 42 test rosters` (the new roster `stream_concrete_tests.rs`).
6. `stream_client.rs` row → append `; the fixed readiness-failure reason constant STREAM_READY_FAILURE_REASON (the 2026-10-30 fix)` (inside the fixed-reason constants group).
7. `stream_concrete.rs` row → re-cut to:
   `stream_concrete.rs - the concrete ChatStreamTransport over the tonic adapter (a FRESH Endpoint per open; the composed establishment open_with_endpoint: bearer metadata → bounded connect → Grpc::new → the readiness reserve grpc.ready().await mapped to the fixed non-secret reason → the server-streaming call with per-status classification via classify_stream_status; open() keeps the production surface and delegates; the stream_concrete_tests roster re-produces the incident shape and pins the fixed composition offline)`.
8. The `youtube/` roster summary → the list gains `stream_concrete_tests 2 (the incident anchor + the fixed-shape establishment pins)` next to the Phase-07 entries and the aggregate becomes `233 unit tests (observed at this pass; …)` (fresh count governs).
9. Root files `Cargo.toml` line → `v0.13.0` → `v0.13.1`.
10. The dead-code staging inventory paragraph → append, dated:
    `2026-10-30 fix pass: the canonical attribute-exact figure stays 9 (8 dead_code + 1 clippy::manual_clamp, now at chat_stream_delivery.rs:168 after the ruled §OBS fold); the fix chain added ZERO allows (re-derived fresh by the Task-2 audit).`
11. **NO line-count rows are added** (decision): the structure map records content, not line counts, for
    these files; the `wc -l` observations (`chat_worker.rs` 199, `chat_stream_delivery.rs` 195,
    `stream_concrete.rs` 95, `stream_client.rs` 97, `stream_concrete_tests.rs` 107,
    `chat_worker_panic_tests.rs` 76) live in the Task-1 plan §L.5 and in §C.6 (§Test Results), and the
    4.5b pass may re-derive from the tree if it needs them. The only line-count annotation style the map
    carries (the `loader_tests.rs sits at the cap` note) is NOT extended.

### §C.4 `.agent/project-info/tech.md` (concise, precise)

**Edit C.4.1 — the `## Stack` runtime bullet** (the Phase-07 chat-worker sentence): append one sentence:

```
The establishment call reserves the endpoint's readiness (tonic `Grpc::ready()`) before the server-streaming call — the tower `Service` contract the `tower::buffer::Buffer`-backed transport `Channel` requires (a reserve-less `call` panics locally; the 2026-10-30 incident + the fix's offline anchor/fix-shape loopback pins).
```

**Edit C.4.2 — the `## Lifecycle` containment bullet**: insert directly AFTER the "Streaming-bridge
confinement (Phase 07)" bullet:

```
* Chat-worker panic containment (2026-10-30 fix, v0.13.1): the worker's `block_on` body runs inside `std::panic::catch_unwind(AssertUnwindSafe(...))`; a caught panic writes the EXISTING typed terminal (`record_terminal` → `ChatRetrievalError::Transport` with the FIXED detail string — NEVER the panic payload text) and the thread returns; the default panic hook keeps the backtrace on stderr; the run then surfaces the funnel's single redacted `error:` line and exits 1 — a retrieval-thread panic is TYPED FATAL, never a silent stream with dead retrieval; the ChatEnded/`offlineAt` graceful paths are untouched.
```

### §C.5 `.agent/project-info/context.md` (truthful, dated, section-scoped)

Three edits; NO existing content is deleted or reworded (append/prepend only — the overwrite-prevention
rule and the file's own "historical record follows" convention).

**Edit C.5.1 — a NEW first paragraph inside `## Current Work Focus`** (PREPENDED above the Phase-07 head
paragraph, which becomes history):

```
Stream establishment readiness fix (2026-10-30) is closed on branch `fix/stream-establishment-readiness` (v0.13.1; NOT merged — workflow step 5 owns the merge/push). Trigger (OPERATOR-SUPPLIED FACTS, `logs/run/20261030-211225.log`, never agent-observed): two v0.13.0 `run` sessions both started, confirmed active, resolved the live chat id — then the worker thread panicked at the `.open()` call (`send_item called without first calling poll_reserve`, tower 0.5.3); the stream kept working, NO chat message ever rendered, and the run exited 0 without any `error:` line. Root cause pinned from the locked source (Task-1 record §A.0): tonic 0.13.1's `Grpc::streaming` calls the tower-Buffer-backed transport `Channel` WITHOUT the required `ready()` reserve — our Phase-07 `open()` skeleton omitted it (a plan defect, NOT a crate defect; tonic issue #2555 + community client docs corroborate; no version reroute exists). The remedies (Task 1, plan `.kilo/plans/20261030-fix-task1-remedy-and-pins.md`, adherence `.kilo/plans/20261030-fix-task1-adherence.md` ADHERENT with 2 caller-ruled deviations): Fix 1 — `grpc.ready().await` before `server_streaming`, failure mapped to the FIXED non-secret reason `the stream endpoint readiness reserve failed` (Network/transient class); Fix 2 — the worker's `block_on` body contained by `std::panic::catch_unwind(AssertUnwindSafe(...))`, a caught panic writing the existing typed terminal with the FIXED detail (`the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1`) → the single redacted `error:` line + exit 1; the mute exit-0 shape is impossible on the fixed tree. Three offline pins landed (`stream_concrete_tests.rs` 2 + `chat_worker_panic_tests.rs` 1; dead plaintext loopback `http://127.0.0.1:1` only; zero Google contact; the observed fix-shape reason recorded as `the stream endpoint connection failed`). Task 2 (THIS pass): docs `docs/production-run.md` (the contained-panic diagnostics/recovery wording + the revised offline-coverage limitation), README (v0.13.1 sentence + OBSERVED gate count), structure map/tech/context re-cut, tracked report `.agent/reports/20261030-establishment-readiness-fix.md`; verification — FULL gate OBSERVED ⟨OBS_GATE_TOTAL⟩ / 0 on the final tree (`⟨OBS_GATE_LOG_PATH⟩`), windows-gnu check re-run exit 0, artifacts rebuilt Windows-first-then-Linux (observed sizes ⟨OBS_WIN_SIZE⟩ / ⟨OBS_LINUX_SIZE⟩ bytes, `ls -l`-cross-checked), hygiene/secret audit clean. Acceptance criteria: 1–4 flipped at Task-1 4.6; AC-5/6/7 flip as Task 2 lands (the 4→7 state). OPERATOR NEXT STEPS: rebuild/re-deploy on v0.13.1 from the freshly built `dist/` artifacts and run the streaming-build manual E2E's chat-bearing steps (`docs/production-run.md` §Manual Verification) — the v0.13.0 dead-delivery shape is EXPECTED FIXED: messages from the separate account must render; live claims stay operator-run. The 2026-10-30 incident broadcasts auto-completed via `enableAutoStop` (no live orphan remains) — no Studio action beyond the standing fate-policy glance.
```

**Edit C.5.2 — a NEW dated H2 block** placed after the Current Work Focus paragraphs, BEFORE
`## Phase 07 Task 1 — streamList client-path research record landed (2026-10-29)` (mirroring the Phase-07
dated-block style; the content densifies the fix record — commit hashes are read from `git log` at edit
time, never invented):

```
## Stream Establishment Readiness Fix — remedy, pins, docs, builds (2026-10-30)

* Incident (operator-supplied, `logs/run/20261030-211225.log`, v0.13.0, both runs exited 0 mute): worker-thread panic at `.open()` — `thread '<unnamed>' panicked at .../tower-0.5.3/src/buffer/service.rs:122:23: `send_item` called without first calling `poll_reserve``; run 2's `RUST_BACKTRACE=1` stack: `PollSender::send_item` → `tower::buffer::Buffer::call` → `<tonic Channel as tower_service::Service<...>>::call` → `<GoogleLiveChatStreamTransport as ChatStreamTransport>::open` → `run_stream_loop`. Secondary facts from the same log: `enableAutoStop` auto-completed both broadcasts (no live orphan; the standing fate-policy glance stands); both video encodes healthy (CBR ~5989/5965 kbit/s, clean exits).
* Root cause (pinned from the locked source, Task-1 record §A.0): tonic 0.13.1 `Grpc::streaming` (`src/client/grpc.rs:282–319`) calls `self.inner.call(request)` WITHOUT `Grpc::ready()` (`grpc.rs:199–204`); the transport `Channel` is tower-`Buffer`-backed and its `call` MUST follow `poll_ready` on the same handle. Independent corroboration: tonic issue #2555 (identical panic before server contact) + real-world client client docs ("reserve with client.ready().await before call()"). Attribution: our Phase-07 `open()` skeleton omitted the reserve — a PLAN DEFECT, not a crate defect (tonic 0.13.1 requires `tower ^0.5`; no 0.5.4+ exists to route around).
* Fixes (Task 1): the readiness reserve in `open_with_endpoint` (fixed reason `STREAM_READY_FAILURE_REASON` = "the stream endpoint readiness reserve failed", Network/transient class; error value discarded, secret-safety unchanged) + the worker containment (catch_unwind → the typed Transport terminal with the FIXED `CHAT_WORKER_PANIC_DETAIL` detail — payload text never read; `gate.rs` join arm untouched; ChatEnded graceful paths untouched). The landed chain is recorded in the Task-1 plan §L (plan `cf258f2`, code `5190145`, fold `2873692`, alignment `fee9d44`; the 4.6 marks follow on the same lineage — read exact hashes from `git log` here).
* Offline pins: (a) the incident anchor — the PRE-FIX reserve-less shape (via `connect_lazy`, zero I/O) panics locally against a dead plaintext loopback endpoint; (b) the fixed shape — the EXACT production composition (through `open_with_endpoint`) yields NO panic and a typed Network-class error (observed reason `the stream endpoint connection failed` — a member of the pinned 3-reason set); (c) the funnel pin — a faked panicking transport produces the typed terminal with the exact fixed detail and `ChatSource` surfaces `Retrieval(Transport)` once then `Ok(None)`.
* Verification: gate OBSERVED ⟨OBS_GATE_TOTAL⟩ / 0 (`⟨OBS_GATE_LOG_PATH⟩`; Task-1's own gates `20261030T220937Z` pre-fold + `20261030T222042Z` post-fold first proved 497/0); windows-gnu check exit 0 re-run on the final tree; artifacts rebuilt Windows-first-then-Linux — ⟨OBS_WIN_SIZE⟩ B Windows / ⟨OBS_LINUX_SIZE⟩ B Linux, `ls -l`-cross-checked.
* Docs/records (Task 2): `docs/production-run.md` Diagnostics/Failure-Recovery contained-panic wording + the revised "handshake untested offline" limitation (the loopback pins now cover the readiness contract; the real-Google TLS/handshake/cloud-side stay operator-run); README v0.13.1 + observed count; structure map/tech/context re-cut; report `.agent/reports/20261030-establishment-readiness-fix.md` (the full incident/root-cause/fix record).
* AC state: TODO AC-1..AC-4 `[x]` (Task-1 4.6); AC-5/6/7 `[x]` when Task 2's 4.6 lands (regression locks/audit; gate+builds on the final tree; docs/records/report).
* OPERATOR NEXT STEPS: run the streaming build's manual E2E on v0.13.1 (the chat-bearing steps — video before messages, separate-account text renders PROMPTLY, idle periods, optional network-interruption check, clean Ctrl+C exit 0, auto-complete); the 2026-10-30 incident broadcasts auto-completed — no Studio action beyond the standing fate-policy glance.
```

**Edit C.5.3 — `## Immediate Next Steps`**: PREPEND one new item at the top of the list (the file already
carries literal duplicate numbering; the `1.` prefix matches that reality — do NOT renumber anything):

```
1. Stream establishment readiness fix (2026-10-30) is closed on `fix/stream-establishment-readiness` (v0.13.1, TODO `.agent/todos/20261030/20261030-todo-1.md`, report `.agent/reports/20261030-establishment-readiness-fix.md`; adherence `.kilo/plans/20261030-fix-task1-adherence.md` ADHERENT): the v0.13.0 mute-delivery panic is fixed (readiness reserve + contained worker panic → the single `error:` line + exit 1) and pinned offline. OPERATOR NEXT STEPS: rebuild/re-deploy from the freshly built `dist/` artifacts and run the streaming-build manual E2E's chat-bearing steps (`docs/production-run.md` §Manual Verification) — messages from the separate account must now render; live claims stay operator-run; no Studio action beyond the standing fate-policy glance (the incident broadcasts auto-completed).
```

### §C.6 The tracked completion report — NEW file `.agent/reports/20261030-establishment-readiness-fix.md`

Format follows the recent exemplars (`.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`,
`...20261029-phase-07-streamlist-migration.md`): TOC, section-per-concern, every number OBSERVED-at-gate or
path-cited, NO invented size, `⟨OBS_*⟩` placeholders filled ONLY from this pass's command output. Required
sections in order:

1. `## Overview` — branch `fix/stream-establishment-readiness`; version v0.13.1; the observed-only rule
   statement; Task-1/Task-2 split of the fix chain; the report's own hash is unknowable from inside the
   record (predecessor pass convention).
2. `## Incident (operator-supplied facts)` — the TODO Pre-Diagnosis facts restated: the two sessions on
   v0.13.0 (broadcasts `TUcJ71xiBqM`, `WebHLRIOlvo`), the log path `logs/run/20261030-211225.log` (plain
   citation, gitignored, never staged), the VERBATIM panic line (`thread '<unnamed>' panicked at ...
   tower-0.5.3/src/buffer/service.rs:122:23: `send_item` called without first calling `poll_reserve``),
   run 2's `RUST_BACKTRACE=1` frame chain (the four-hop quoting from the TODO), the mute outcome (stream
   alive, zero messages rendered, exit 0, NO `error:` line), the "panicked" quotes being the operator log's
   only terminal evidence, and the secondary shaping facts (`enableAutoStop` auto-completed both broadcasts;
   healthy CBR encodes ~5989/5965 kbit/s).
3. `## Root Cause (pinned from the locked source)` — tonic 0.13.1 `Grpc::streaming`
   (`src/client/grpc.rs:282–319`) calls `inner.call` without `Grpc::ready()` (`grpc.rs:199–204`); tower
   0.5.3 Buffer contract (`service.rs:122:23` = the exact panic site; `poll_ready` → `poll_reserve`); tonic
   issue #2555; community-doc concordance; the **plan-defect admission** (the Phase-07 Task-2 plan §C.4
   `open()` skeleton omitted the reserve — our call shape, never a crate defect; no tonic/tower patch
   reroute exists as of 2026-10-30); the eager-`connect` mechanics correction (Task-1 §A.0 item 4) recorded
   as the research record it was.
4. `## The Two Fixes` — Fix 1 (the reserve in `open_with_endpoint`; the exact fixed reason string; the
   `|_|` discard; Network/transient class) and Fix 2 (the catch boundary; the typed terminal path; the
   moved constants home in `chat_stream_delivery.rs` per the Caller's Option-A ruling; `gate.rs` untouched;
   ChatEnded/offlineAt untouched). Include the operator-visible composed error line verbatim.
5. `## The Three Offline Pins (evidence)` — names + homes (`stream_concrete_tests.rs`
   `establishment_anchor_unread_shape_panics_locally` /
   `establishment_fixed_shape_reserves_readiness_and_yields_typed_error`; `chat_worker_panic_tests.rs`
   `worker_panicking_transport_writes_the_terminal`); the offline posture (dead plaintext
   `http://127.0.0.1:1`, `connect_lazy` zero-I-O anchor, no listener, no Google); the OBSERVED fix-shape
   reason `the stream endpoint connection failed` and the membership rationale; the Task-1 targeted-run
   counts (2/24/12 filters) as recorded in the Task-1 plan §L.4.
6. `## Files Changed / Commits` — the chain table (Task-1: `6151fa0` bump → `cf258f2` plan → `5190145`
   code → `2e89cc3` addendum → `2873692` fold → `fee9d44` alignment + the observed 4.6 marks hashes +
   Task-2's own commits as landed before this file's final write; exact hashes read from `git log`).
7. `## Test Coverage Audit` — the §A verdict verbatim; the roster arithmetic (494 + 3 = the fresh total).
8. `## Exact Commands and Exit Statuses` — the table of §B duties 3–6 + this pass's verification runs,
   each with exit code + observed result.
9. `## Build Results` — Windows exe path + OBSERVED size; Linux binary path + OBSERVED size; both
   `ls -l`-cross-checked, both non-empty; the Windows-then-Linux order note; the caveats (cross-compilation
   ≠ native runtime validation; building validates no live chat behavior — the real handshake stays the
   operator E2E).
10. `## Hygiene/Secret Audit` — the §B.6 observed table.
11. `## Manual Verification Pointer (operator-run)` — the E2E pointer; the incident's expected-fixed note;
    no agent-observed live claim.
12. `## Checks Actually Performed vs NOT Performed` — per performed §B duty; NOT: live Google
    verification, native runtime execution, any Studio observation.
13. `## Known Limitations` — (a) the real-Google TLS/handshake + cloud-side establishment behavior remain
    operator-verified ONLY (the offline pins cover the call-shape contract locally); (b) the E2E (push
    latency, re-establishment) remains pending on the streaming build; (c) carried Phase-07 facts
    (baseline-session loss window, quota unknowns, the `LIVE_CHAT_DISABLED/ENDED` ambiguity, ≤64 message
    shutdown loss) unchanged.
14. `## AC Closing Matrix` — the 7 TODO ACs (lines 56–62): AC-1..AC-4 with the Task-1 flip annotations
    carried from `.kilo/plans/20261030-fix-task1-adherence.md` §7 (rationale sentences preserved);
    AC-5/AC-6/AC-7 with the Task-2 verdicts + evidence pointers (audit/gate log + build sizes/docs set).
15. `## Evidence Index` — the TODO, both fix plans, the Task-1 adherence record, the gate log path (VM-side,
    gitignored), the build command list, the log path citations, and the prior report precedents.

## §D COMMIT LIST for 4.2 and the 4.3–4.6 expectations

1. **Plan-tracking commit BEFORE any edit:**
   `git add .kilo/plans/20261030-fix-task2-docs-builds-report.md && git commit -m "chore: track stream establishment readiness fix Task 2 plan"`
2. **User-facing docs commit** (after §B.3–§B.5 verification is recorded so the docs carry observed numbers):
   `git add docs/production-run.md README.md && git commit -m "docs: record the stream establishment readiness fix for operators"` — message may be tuned to the branch convention; no amend.
3. **Metadata + report commit:**
   `git add .agent/project-structure.md .agent/project-info/tech.md .agent/project-info/context.md .agent/reports/20261030-establishment-readiness-fix.md && git commit -m "docs: record the readiness-fix metadata and completion report"`.
4. **§L addendum commit ONLY if it gained observations:**
   `docs: record the readiness-fix task-2 plan addendum` (this plan file only).
5. **4.3 review/simplification expectations:** docs-only pass — reviewer verifies the §C anchors/facts
   against the gate log, the `ls -l` outputs, and the binding records (a numeric/wording drift is a fix
   candidate); simplifier — expected "not required" (no code exists to simplify; wording tightening beyond
   the prescribed edits is OUT of scope unless a factual contradiction needs repair). Any fix commit stays
   docs-only; gate/builds re-run ONLY if a code-named claim changed materially (code is untouched).
6. **4.4 docs-specialist:** the Task-2 §C set IS the docs deliverable; 4.4 verifies alignment (anchors, TOC
   resolution, no literal `\n`, the manual-E2E NO-EDIT verdict) and records rather than rewrites.
7. **4.5b (architector):** fresh-evidence audit — the gate log exists with the recorded total; sizes recorded
   + cross-checked; the report's `⟨OBS_*⟩` values all replaced with observed ones; target-file edits match
   §C; the constants/wording match the Task-1 records verbatim.
8. **4.6 (implementer):** flip TODO lines 60 (AC-5), 61 (AC-6), 62 (AC-7) to `[x]` with the recorded
   rationale carried from this plan (§B/§C evidence), mark the Task-2 heading `[DONE]`, and commit the TODO
   change; NOTHING else in the TODO file is modified (overwrite-prevention rule). `git push` stays step 5.

## §E STOP PROCEDURE (to the caller, with evidence + options)

Stop ONLY on: (1) the FULL gate fails or the observed total deviates from 497 without an explanation
traceable to a roster change; (2) a build script exits non-zero or a `ls -l` size misses/non-empty check
fails; (3) a recorded doc fact contradicts the tree or the Task-1 records (e.g. the fixed reason string
mismatches `STREAM_READY_FAILURE_REASON`'s value); (4) the hygiene/secret audit finds a staged
gitignored file or a token/secret hit that is not the documented pre-existing set. Docs-only adjustments
(word wrap, anchor reflow within the same sentence semantics) do NOT stop — apply and record in §L.

## §H Compliance mapping (Task-2 bullets → sections; §0 is the short form)

| TODO bullet / AC | Covered by |
|---|---|
| Bullet 1 — coverage audit; no new tests beyond Task-1 pins | §A |
| Bullet 2 — production-run diagnostics/recovery + limitation revision | §C.1 (C.1.1–C.1.4) |
| Bullet 2 — README drift (version + observed count) | §C.2 |
| Bullet 2 — structure-map re-cut | §C.3 |
| Bullet 2 — tech.md notes | §C.4 |
| Bullet 2 — context.md incident/fix record with operator attribution | §C.5 |
| Bullet 3 — tracked completion report | §C.6 |
| Bullet 4 — final-tree gate; BOTH builds + sizes; audit | §B.3–§B.6 |
| AC-5 flip (regression locks + final commit-set hygiene) | §B.6 audit + §D.5/§D.2/§D.3 scopes |
| AC-6 flip (gate on final tree + both artifacts with sizes) | §B.3–§B.5 |
| AC-7 flip (docs/records/report; operator attribution) | §C.1–§C.6 |
| Rules: no secrets; gitignore compliance; no invented numbers; no commented code; single-section boolean/limits not applicable (docs-only) | §B.6, §C.6 rule-1/§B, global ruleset |

## §I NOT-DONE LEDGER (Task 2 leaves for the workflow — never claimed here)

- 4.3 review/simplification sub-steps, 4.4 docs-alignment adjudication, 4.5b adherence, 4.6 completion
  (AC-5/6/7 flips + the `[DONE]` mark) — the caller owns the step sequencing.
- Workflow step 5: TODO rename `-DONE`, tmp cleanup, merge to `main`, push `origin` ONLY.
- The live streaming-build operator E2E (push latency, re-establishment, rendering) — operator-run, never
  agent evidence; the real-Google TLS/handshake/cloud-side establishment behavior stays operator-verified.
- Any code/test/lockfile change (Task 1 owns all three; Task 2 is docs/builds/report ONLY).

## §J DEPTH RULING (encoded)

`docs/production-run.md` is the live operator manual and MAY carry the content-heavy contained-panic
narrative (its precedent is the v0.11.1 product-level status paragraphs), but the edits stay inside the
Live Chat H2 and add ≤ ~10 net lines total. README stays fact-level (2 edits, no incident section).
The metadata files (structure map / tech / context) are updated conservatively: truthful, section-scoped,
no rewrites of historical blocks, no invented line-count rows. The completion report follows the
phase-exemplar format (TOC + concern sections) and carries the full incident/root-cause record BECAUSE the
repo convention is that the tracked report is the fix-phase's record of truth — but every number in it is
recalled from a command or a path-cited record, never asserted.

## §L ADDENDUM (initially empty — §B duties append here)

<!-- IMPLEMENTER: append dated observations — the fresh gate log path + OBSERVED total, the 4 build outputs + sizes,
the audit results, the drift-verified NO-EDIT confirmations, and any correction found while applying §C.
NO other section may be modified; the pre-analysis above is the binding record. -->

### L.1 Pre-flight and plan tracking (2026-10-30, implementer)

- Branch `fix/stream-establishment-readiness`; HEAD `a4eb832` (Task-1 4.6 marks; v0.13.1 lineage); tree clean except the untracked plan file.
- Plan-tracking commit: `a7db8dc chore: track stream establishment readiness fix Task 2 plan`.
- `#[allow(` snapshot: 9 attribute lines (8 `dead_code` + 1 `clippy::manual_clamp` now at `chat_stream_delivery.rs:168`); unchanged by Task 2.

### L.2 §B observed matrix (container 2026-10-30)

- FULL gate `sh scripts/dev-checks.sh` → exit 0; four PASS lines (`fmt-check`/`check`/`test`/`clippy`);
  `ALL CHECKS PASSED`; `test result: ok. 497 passed; 0 failed`; log `logs/checks/20261030T224512Z.log`.
  OBSERVED total **497** (494 baseline + Task-1's 3 pins) — the plan's expected 497 held.
- windows-gnu check `cargo check --locked --target x86_64-pc-windows-gnu` → exit 0 (`Finished dev profile`).
- Windows build (`build-windows.sh`) → exit 0; script print 12334184 bytes.
- Linux build (`build-linux.sh`) → exit 0; script print 7654688 bytes.
- `ls -l` cross-check → Linux 7654688 and Windows 12334184, both non-empty and MATCHING the script prints exactly.
- Audit: `git status --porcelain` = only the intended docs/metadata/report paths as each commit was staged;
  `git ls-files dist videos` EMPTY; `git ls-files credentials` = `.gitkeep` only;
  `check-ignore` → `.gitignore:34:dist/` (both artifacts) and `.gitignore:18:logs/*` (the cited operator log);
  `ya29`/`AIza` = ZERO in `src docs README.md`; `key=` = the pre-existing doc/test-context set, count unchanged;
  `rtmps://` = pre-existing `.example.test` fixtures/doc-context only, count unchanged; `#[allow(` = 9 attribute lines, unchanged.
- Fresh structure counts (from `git ls-files` + `rg -c "#\[test\]"`): run/ **32 files = 13 sources + 4 fixtures + 15 rosters**
  carrying **59 unit tests**; youtube/ **93 files = 47 sources + 4 fixtures + 42 rosters** carrying **233 unit tests**;
  `Cargo.toml` v0.13.1. All §C.3 predicted counts matched the observed tree.
- Drift-verified NO-EDIT (all confirmed untouched): `docs/production-run.md` sections Overview; Prerequisites;
  What Happens at Startup; What the Logs Look Like; Shutdown Behavior; Broadcast/Stream Fate; Privacy;
  Quota; Ingest Probe Procedure; Known Stream-Health Limitation; Manual Verification Procedure; Out of Scope;
  See Also. README Key Technologies bullets stay verbatim. No correction to §C text was needed while applying it.
- Report placeholders `⟨OBS_GATE_TOTAL⟩`/`⟨OBS_GATE_LOG_PATH⟩`/`⟨OBS_WIN_SIZE⟩`/`⟨OBS_LINUX_SIZE⟩` were replaced with
  the observed values above in `README.md`, `.agent/project-info/context.md`, and the new report.

### L.3 Commits landed by this pass

- `a7db8dc` plan tracked; `9604bbd` user docs (`docs: record the stream establishment readiness fix for operators`);
  `369937d` metadata + report (`docs: record the readiness-fix metadata and completion report`); the §L addendum commit follows.
- Task-2 changed docs only — no code, test, lockfile, or version change. The report's own hash is unknowable from inside the record (predecessor convention).
