# Phase 05 — Task 3 Plan Adherence Report (Critical Workflow sub-step 4.5b)

- Date: 2026-10-14
- Role: architector (adherence reviewer). Report only — no code fixed, no task marked done, no push.
- Scope: Task 3 only ("### 3. Production `run` Orchestration and FFmpeg Output" of
  `.agent/todos/20261013/20261013-todo-1.md`, plus the TODO's global Scope/Boundaries and
  Explicitly-Out-of-Scope, plus the user ruling that Live Chat is OUT of Phase 05 — global plan
  §1.3).
- Authorities read in full this pass: TODO Task 3 text (ll. 58–68), TODO global scope/limitations
  (ll. 9–34, 81–95), global plan `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md`
  (D-G1..D-G11), Task-3 plan `.kilo/plans/20261013-phase05-task3-run-orchestration.md`
  (D-T3-1..15), simplify plan `.kilo/plans/20261013-phase05-task3-simplify.md`, Task-2 adherence
  `.kilo/plans/20261013-phase05-task2-adherence.md` (E1/E2/E3 carry-forwards), workfows/rules,
  project brief/structure maps.
- Repo state at review: branch `feat/phase05-production-live-runtime`, HEAD `c8db5d5`, clean tree,
  latest green gate `logs/checks/20261014T203340Z.log` — **355 passed / 0 failed /
  ALL CHECKS PASSED** (log verified fresh this pass).

---

## 1. Commit ledger (a37cb4f..c8db5d5) + process note

```
f3afbf5 docs: track the phase 05 task 3 implementation plan         (S1)
9e266b8 feat: extend streaming for the rtmps output target         (S2 / C2, SP1–SP9)
3e09375 feat: carry the stream id through the prepared broadcast   (S3 / C3, Y1+Y2)
23a2ce7 feat: wire the production run orchestration                (S4 / C4; exit-3 deleted —
                                                                   retirement encoded in the body)
d9a4297 fix: restage documented-but-unread live model fields       (S5 transcription)
06cd933 refactor: split the task-3 files breaching the 200-line cap(4.3-fix; D-T3-14)
c53d26d docs: retire the stale exit-3 reference in the auth docs   (4.3-fix doc repair)
20905a3 docs: map the run runtime files in project structure       (S9 / A16)
a2f9f09 refactor: bring the task-3 spawn and gate helpers within the two-args rule
                                                                   (simplify S1–S3 + spawn.rs
                                                                   extraction ruling)
e60267f docs: track the phase 05 task 3 simplification plan        (4.3 tracking)
c8db5d5 docs: verify the task-3 run runtime module docs /repair the structure map (4.4)
```

**Process note (local history reset):** during 4.3-fix the branch history was locally reset ONCE
to split a mis-messaged commit (the reset orphaned the intermediate chain `cb0fe91`/`7cddb5b`).
Nothing was pushed to any remote at any point; the reachable chain above is the canonical Task-3
history and every commit in it remained green-at-each-point (intermediate gates cited in the
commit bodies: `20261014T193230Z.log` at the splits, `20261014T203340Z.log` at the simplify
refactor). No adherence impact — recorded as a process hygiene note only.

## 2. Live-Chat ruling (TOP-priority constraint) — COMPLIANT

- `git diff a37cb4f..HEAD -- src/app/pipeline/source.rs` → **EMPTY** (the Phase-06 entry seam is
  byte-unchanged). Fresh grep `liveChat` (case-insensitive) over `src/` → **zero hits**.
- The frame loop runs chat-free via `src/app/run/source.rs::ChatFreeSource`
  (`MessageSource` → always `Ok(None)`); no chat source is constructed anywhere in the run path.
- The TODO's chat-bearing Acceptance-Criteria lines were never claimed or implemented
  (global plan §1.3 handling observed).

## 3. TODO Task-3 bullet-by-bullet matrix

| # | TODO Task 3 bullet | Verdict | Evidence |
|---|---|---|---|
| 1 | Replace the exit-3 `run` placeholder with production startup loading/refreshing credentials via the Phase-04 credential service; permission verification BEFORE resource creation | MET | `src/app/modes.rs` deleted (−83 lines); dispatch arm `cli::Mode::Run => run::run_command`; `run/credentials.rs` reads+parses the client file FIRST (auth-flow D5 order), constructs the concrete Phase-04 stops and calls `CredentialService::establish_authorization` VERBATIM (no OAuth duplication; D-G4); `authorize_scope_evidence` gates `REQUIRED_SCOPE` BEFORE the transport is used (fresh `VerifiedAuthorization::granted_scopes` evidence, distinct from the service's persisted-scope gate). Test `scope_gate_failure_prevents_resource_creation` proves the transport operation ledger is EMPTY on a scope-gate failure (plan §5). |
| 2 | Explicit startup order: credentials → broadcast/stream resources → readiness + RTMPS destination → FFmpeg start → continuous frame pipeline; refinement documented; no Live Chat source | MET | D-T3-2's ten stages landed in the exact order in `run/mod.rs::run_outcome`/`run_runtime` (Ctrl+C pair → credentials → scope gate → transport → prepare O1..O4+destination → the ONE reveal → FFmpeg start → post-start confirmation gate → chat-free loop → ordered shutdown). The ONE refinement (thread-parallel confirmation gate instead of blocking-before-frame-loop) is documented where the TODO requires: Task-3 plan D-T3-2 stage 8, `run/mod.rs` module docs, and `run/gate.rs` module docs — justified by the pipeline contract (video must flow before `streamStatus` flips to `active`). |
| 3 | Minimal streaming-output extension; `demo` keeps collision-safe MP4; preserve no-shell spawn, drains, frame-write errors, child cleanup | MET | `src/streaming/rtmps_destination.rs` NEW (secret wrapper); `args.rs::build_stream_arguments` output group EXACTLY (i) URL final arg, (ii) `-movflags` removed, (iii) `-f flv`, (iv) `-g <2 × fps>` (constants `STREAM_MUXER="flv"`, `GOP_MULTIPLIER=2`; no magic numbers); the LOCAL `build_arguments` vector is byte-unchanged (verified in-file); `FfmpegProcess::start_stream` shares ONE spawn engine via `spawn.rs` (piped stdio, no shell, drains attached before return; `Drop` kill+reap unchanged); `finish()` = stdin EOF → drain joins → `wait()` → exit check → local-finalize ONLY when a local output exists. Demo: `print_final_path` now guards `CompletedOutput::Recording` and emits the byte-identical success line; all demo timeline/path/exit-code assertions unchanged. |
| 4 | RTMPS URL/key nowhere in command logs, tracing fields, error/panic text, or debug; redaction tests around spawn AND exit failures | MET | D-T3-11's surface enumeration verified (§6 greps + tests): `StreamingError::Spawn` carries executable + io cause only (`stream_spawn_failure_never_carries_the_destination`); exit diagnostics pass `redact_destination` replaces every URL occurrence with `[redacted]` (`REDACTED_TEXT`) BEFORE the payload is built (`stream_exit_failure_diagnostics_redact_the_destination`); no `Display`/`Debug` derive on `RtmpsDestination` (Debug = `RtmpsDestination(<redacted>)`, test-pinned via the recorded execution bundle), no `Display` on `RtmpsIngestion`, NO `Debug` derive on `FfmpegProcess`/`SpawnPlan` (`spawn.rs` doc states `redaction_target` IS the secret); `into_url` has exactly ONE production call site (the reveal expression `run/mod.rs:141`) + test sites; run-error Display lines tested secret-free, with the single permitted public `REQUIRED_SCOPE` in `InsufficientScope`; tracing events field-whitelisted (`broadcast_id` only added) and message-only. |
| 5 | Frame loop continuously producing frames independent of chat activity | MET | `ChatFreeSource` + the untouched `MessageSource` seam; `pipeline/runner.rs` paced loop unchanged for the production path (`prepared_stream_pipeline` differs from `prepared_pipeline` ONLY in the `FfmpegProcess::start_stream` construction, per D-T3-10). |
| 6 | Ctrl+C/fatal shutdown order: stop chat polling, request frame-loop stop, close stdin, wait FFmpeg exit, report cleanup failures WITHOUT masking the primary error; cancel in-progress API waits | MET | One handler flips BOTH `StopFlag` and `WaitCanceller` (`install_stop_pair` → generic `install_stop_request_handler` in `pipeline/interrupt.rs`; the demo handler and module docs stay byte-unchanged). "Stop chat polling" is Step 1's documented structural no-op (never existed in Phase 05). On a pipeline FATAL the run cancels the confirmation wait BEFORE joining the gate (cancel-before-join preserved verbatim through simplify S1's inlining). `sink.finish()` closes stdin and waits; a frame-write fatal falls to `Drop` (kill+reap). Primary-error precedence is the pure, unit-tested `primary_shutdown_outcome` matrix: `GateFinish::Fatal` > pipeline error > `GateThreadStopped` > `Ok`; the causally-superior gate-fatal wins, so downstream finalize noise never MASKS the primary. The Phase-03 unbounded-`finish()` limitation STANDS per plan rule 5 (no timeout field mandated) — documented, not silently hardened. Gate join cost documented (≤ one poll interval + in-flight ureq call). |
| 7 | Broadcast/stream resource fate defined and documented; no auto-delete; avoid leaving a stuck `live` broadcast | MET | D-T3-6 rule 8 encoded in `run/mod.rs` module docs: NEVER delete, NEVER `transition`; `enableAutoStop=true` auto-completes ~1 min after ingestion ends; startup failures AFTER O1/O2 leave the created `ready` resources (repeat-run `userBroadcastsExceedLimit` accumulation = documented limitation; operator guidance = YouTube Studio deletion; automation out-of-scope); pre-streaming Ctrl+C makes NO auto-complete claim (never went live). D-T3-12's voided ready→live log note is recorded in `gate.rs` module docs. Grep: zero `transition(`/`liveBroadcasts.delete`/`liveStreams.delete` call sites. |
| 8 | Preserve `auth`/`demo` behavior and exit contracts; update `run` usage text and tests; exit codes stable | MET (with V-6 doc repair) | `auth`/`demo` behavior, messages, and exit codes byte-preserved (auth diff = one doc-comment sentence only, V-6); `cli.rs` run line now `stream the configured broadcast to YouTube Live` (alignment preserved); +2 cli tests (`usage_text_advertises_the_implemented_run_command`, `usage_text_no_longer_claims_not_implemented`); exit codes: `--help` 0, usage 2, StartupError 1, run clean-stop 0, run fatal 1, pre-streaming `WaitCancelled` 0 with the distinct `RUN_STARTUP_STOP_MESSAGE`; **grep `NOT_IMPLEMENTED_EXIT|exit 3` over `src/` → zero hits**; the retirement rationale ("exit 3 has no remaining truthful referent" incl. Live Chat permanently deferred by the user ruling) is encoded verbatim in commit `23a2ce7`'s body per D-T3-8. |

## 4. Task-3 plan decision compliance (D-T3-1..15)

| Decision | Verdict | Evidence |
|---|---|---|
| D-T3-1 module shape (`src/app/run/`; `modes.rs` deleted; funnels/messages) | MET (files split per D-T3-14) | 17 files (8 sources + 9 test/fixture files) vs the plan's 8-file outline: `seams.rs` and `runtime_credential_fakes.rs` are the pre-authorized line-cap splits (V-4), `spawn.rs` lives in `streaming/` (V-5, different module). Exit funnel arms, both message constants, `report_run_failure` two-liner — landed verbatim (§2 ll. 40–72 of the plan). |
| D-T3-2 ten stages + ONE documented refinement | MET | staged order verified in `run_runtime`; the refinement (gate thread-parallel to the frame loop) is the plan's own stage 8, documented in code AND plan. The credential probes issue no transport ops (ledger proof). |
| D-T3-3 Ctrl+C pairing installer | MET | `install_stop_request_handler(Box<dyn Fn() + Send>)` shape exact; `pipeline/mod.rs` re-export landed consumer-locked; demo's `install_interrupt_handler` byte-unchanged; documented-limitations inheritance stated in the new fn's docs. |
| D-T3-4 `PreparedBroadcast` stream id | MET | private `stream_id` + `pub(crate) stream_id()`; the staging allow retired at the wiring; +1 fixture test (`prepared_broadcast_exposes_bound_stream_and_broadcast_ids`). |
| D-T3-5 streaming extension (F-S1..F-S5) | MET | every item as specified incl. the redaction pass and target-aware `finish`/`CompletedOutput`; `video_output.rs` untouched as planned; `start_stream`'s item-level staging existed and retired at consumption (23a2ce7 body + zero remaining `start_stream` staging). |
| D-T3-6 shutdown rules 1–8 | MET | rule 2's cancel-before-join verbatim (S1 inlined); rule 5's bounded-latency facts in `gate.rs`/`streaming.rs` docs; rule 6 selector; rule 7 exit codes; rule 8 in module docs. |
| D-T3-7 success/stop reporting | MET | both constants pinned by tests; verbatim text equality confirmed; three whitelisted `info!` events; no ingestion material in any field/message. |
| D-T3-8 CLI/usage/exit contract | MET | see matrix bullet 8; the deleted placeholder tests: 4 removed (the non-3 assertions re-homed into `error_tests::run_origin_failures_map_to_the_startup_failure_code`), per plan. |
| D-T3-9 testability seams | MET (V-1/V-3 adaptations) | parameter objects exact; `run_command` never test-called (the whole suite exercises `run_runtime` via `run_with_scripted` — process-global ctrlc respected); fakes fixture-shaped (`ingest.example.test`, `stream-name-fixture`, `stream-key-fixture`); gate tests zero-sleep only. The `Send + Sync` question resolved in the plan's PRIMARY shape (direct `Arc<dyn ...>` — no Mutex wrapper needed; the compiler confirmed the concrete ureq transport satisfies it). |
| D-T3-10 `prepared_stream_pipeline` | MET | `StreamParts` + sibling constructor in `runner.rs` (199/200 lines — at cap, compliant); `prepared_pipeline`/`PipelineConstruction`/`OutputKind` untouched (verified via the streaming `source.rs`/demo-construction diffs). |
| D-T3-11 redaction surface enumeration | MET | all 8 surfaces audited fresh (§6); the argv-capture test pins the production FFmpeg argv inside the test process only (`stream_pipeline_passes_the_destination_to_the_ffmpeg_argv`: last token == fixture URL, `-f flv` immediately before, NO `-movflags`, `-g 60`). |
| D-T3-12 voided ready→live note | MET | gate.rs module docs. |
| D-T3-13 no new dependencies | MET | `git diff a37cb4f..HEAD -- Cargo.toml Cargo.lock` → EMPTY. |
| D-T3-14 code rules | **PARTIAL — one violation (V-10)** | every new file ≤200 (max: `run/mod.rs` 182, `runtime_test_fixtures.rs` 183, `error_tests.rs` 117); `runner.rs` edited at 199 ≤ 200; BUT `pipeline/test_helpers.rs` — EDITED by the SP9 ripple — moved 200 → 202 lines, breaching the hard cap by 2. |
| D-T3-15 E1 exception | MET (re-verified, not expanded) | the four recorded signatures (`ReadinessWait::new` 4; `with_polling` 6; `wait_until_stream_active_with_polling` 4; `confirmation_poll` 4) untouched — diffs touched only the `StreamObservationRequest` widen (`pub(super)`→`pub(crate)`), the `pub(crate) use confirmation_wait::wait_until_stream_active;` line, and one doc line; the ONE production consumption point satisfies ≤2 args via its param object. The three initially-out-of-list NEW production functions were brought INTO compliance by simplify S1–S3 (V-7). |

## 5. Deviation ledger and adjudications

| ID | Deviation | Adjudication |
|---|---|---|
| V-1 | `StreamingRunner::run` takes `&self` instead of the plan's consuming `self` | **ACCEPTABLE.** Compiler-arbitrated adaptation: the plan calls the runner THROUGH `&dyn StreamingRunner`, which requires object safety (`self`-consuming is not callable via `&dyn`). Recorded in the trait's docs (`streaming.rs` ll. 24–29). Every other shape identical. |
| V-2 | `Send + Sync` wrapper shape | **NOT A DEVIATION.** The plan's primary shape (direct `Arc<dyn LiveApiTransport + Send + Sync>`, no Mutex wrapper) compiled — the sanctioned Mutex fallback was never needed; recorded in `seams.rs` docs. |
| V-3 | `run_runtime` landed PRIVATE (not `pub(super)`) and the seam parameter objects live in `seams.rs` rather than inside `mod.rs` | **ACCEPTABLE.** Private-by-default rule (`seams.rs` doc: types are run-module-internal) + D-T3-14-cap split; both callers live in the module subtree, so visibility is strictly tighter than planned. Documented in-code. |
| V-4 | Line-cap splits BEFORE review (commit `06cd933`): `ffmpeg.rs` 262→185 (→ 143 after V-5) via `pipe_drain.rs` + `local_finalize.rs`; `run/mod.rs` 212→182 via `seams.rs`; `runtime_test_fixtures.rs` 280→183 via `runtime_credential_fakes.rs` | **ACCEPTABLE.** Explicitly pre-authorized by D-T3-14 ("split test files or display helpers BEFORE review if required"); zero behavior change (gate green at `20261014T193230Z.log`); structure map updated. |
| V-5 | Spawn plumbing extracted to `src/streaming/spawn.rs` (commit `a2f9f09`: `ffmpeg.rs` 202→143) — beyond the simplify plan's literal text (which kept `spawn_with` inside `ffmpeg.rs` at ≈197 lines) | **ACCEPTABLE (the recorded ruling).** The SpawnPlan refactor (simplify S3) pushed `ffmpeg.rs` to 202 → over the hard cap, so the cohesive no-shell spawning unit moved to its own file with `SpawnPlan` fields `pub(super)` (necessary for sibling use) and the no-`Debug`/no-`Display`-ever contract transcribed into the new file's module docs. Commit body + structure map record it; behavior-identical; test total unchanged at 355. |
| V-6 | `src/app/auth/mod.rs` touched (commit `c53d26d`) although Task-3 plan §3 declared `src/app/auth/*` "Explicitly NOT touched" | **ACCEPTABLE.** The edit is ONE doc-comment sentence: the auth module's "exit 3 stays reserved for the `run` placeholder" clause became FALSE the moment the wiring commit retired exit 3 (D-T3-8 mandates zero exit-3 referents). Behavior, tests, exit codes byte-unchanged. A documentation-drift repair mandated by the plan's own exit-3 retirement, not a scope breach. |
| V-7 | Standalone fmt style ruling (4.3-fix): mechanical rustfmt normalization preserved as its own style act rather than folded into feature commits | **ACCEPTABLE.** The established project precedent (Phase-02 `5e9fea4`, Phase-03 `8d8e739`) recorded by the planner; no restructure while formatting; gates green. |
| V-8 | Simplification results (sub-step 4.3): S1 `join_gate` inlined (cancel-before-join preserved verbatim); S2 `report_exit_failure` → private method reading `redaction_target` through `&self` (same laziness: nothing built on success); S3 `SpawnPlan` param object (same argv/fields/order); S4 mis-owned `#[cfg(test)]` re-export comment re-placed (doc-only) — with `pipe_drain.rs`/`local_finalize.rs` and the redaction order explicitly untouched | **COMPLETE as planned.** Reviewer's follow-up pass: no further fixes required. Adjudicated-not-"fix" items (test-only multi-arg helpers `run_with_scripted`/`service`, sanctioned duplications, `StreamingExecution.stream_id` test-pinned reveal) stand as recorded in simplify §0/§7. |
| V-9 | Test total: plan roster estimated +26 new → ~358; OBSERVED +27 new `#[test]` functions and a final total of **355** | **ACCEPTABLE.** The plan's observed-total rule makes the gate log authoritative. Arithmetic: base 332 + 27 added (app/run 17: error 4 + credentials 3 + gate 3 + runtime 5 + mod_tests 1 + streaming_stage 1 + cli 2; streaming TS 7; TU 1) − 4 deleted `modes.rs` placeholder tests = 355. All roster names/emitters verified present; no test removed except the four mandated placeholder deletions. |
| V-10 | **`src/app/pipeline/test_helpers.rs` = 202 lines (hard-cap violation, +2 over)** | **NOT ACCEPTABLE — sole blocker.** The file sat at exactly 200 at base `a37cb4f`; the SP9 ripple (the two `FrameSink::finish` bodies wrapping their `CompletedVideo` in `Recording`, prescribed by plan §3 SP9/F-S4 without a cap re-check) moved it to 202. D-T3-14 binds "≤200 lines" on EVERY new/edited file; the simplify and review passes audited `ffmpeg.rs`/`streaming.rs` line counts but missed this file. No structural change is required — **proposed mechanical fix (no amendment plan needed): extract the `ScriptedSource` unit (struct + `schedule`/`failing` constructors + the `impl MessageSource` block, current lines ≈160–202) into a sibling `src/app/pipeline/source_helpers.rs` wired identically to the existing `#[cfg(test)]` test-support modules (~45 lines there, leaving `test_helpers.rs` ≈157), and update `.agent/project-structure.md`'s pipeline entry in the same commit.** Gate must re-run green (total unchanged at 355; no test added/removed/renamed). |

## 6. Fresh audit greps (adherence expectations, plan §11, run this pass)

| Grep | Expected | Observed |
|---|---|---|
| `allow(dead_code)` attribute lines in `src/` | ≈ 8 | **8 attribute lines** + 1 comment-text mention (`src/youtube/mod.rs:68`) — matches the structure map (6 pre-existing item reservations + the 2 compiler-arbitrated V2 re-stages `live_lifecycle`/`live_models`, each with the updated V2 contract comment; arithmetic 22 − 6 item − 10 module + 2 re-staged = 8 holds) |
| `allow(unused_imports)` in `src/` | 0 | 0 |
| `liveChat` (case-insensitive) in `src/` | 0 new-code hits | 0 |
| `NOT_IMPLEMENTED_EXIT`, `exit 3` in `src/` | 0 | 0 |
| `transition(`/`liveBroadcasts.delete`/`liveStreams.delete` | 0 | 0 |
| `key=` | 0 (doc-only) | 0 (one doc-comment sentence stating the prohibition) |
| `into_url` production call sites | exactly ONE + tests | `run/mod.rs:141` (the reveal expression) + test/doc sites only |
| `secret()` sites | unchanged from Task 2 + none new | all hits are Phase-04 pre-existing surfaces (verifier/transport/token/persisted/auth flow); zero new |
| Diff surface vs plan §3 | only the planned + pre-authorized deltas | `src/app/pipeline/source.rs`, `src/app/auth/**` (except V-6's doc sentence), `src/app/pipeline/{clock,pacing,error}.rs`, `src/config/**`, `Cargo.toml/lock`, `docs/*`, `README.md`, `config/*`, `scripts/`, `Dockerfile`, `docker-compose.yml` → EMPTY |

## 7. Redaction summary (secret-safe guarantees verified)

- The destination crosses the run boundary ONLY inside `RtmpsDestination` (redacting `Debug`, no
  `Display`, no url accessor on `StreamingExecution`); the ONE plain reveal exists inside the
  stage-6 expression and is immediately re-wrapped.
- FFmpeg's stderr echo of the output URL is neutralized by the mandated redaction pass BEFORE any
  `ExitFailure` payload (test-pinned with a key-shaped sentinel).
- Spawn-argv material exists in memory only; `SpawnPlan`/`FfmpegProcess` derive neither `Debug`
  nor `Display`; gate-thread panics surface as the typed `RunError::GateThreadStopped`, never as
  text; production run code on the destination paths has no `unwrap`/`expect` panics.
- Tracing whitelist: `command`, `config_path`, `exit_code`, `broadcast_id` — plus message-only
  events. The stdout stop messages name no ingestion material.

## 8. Carry-forwards (recorded only; planner/Task-4 owned)

- **E1** (Task-2): RESOLVED as the recorded D-T3-15 exception (verbatim rationale above in §4);
  re-verified at this pass that the four signatures were consumed unchanged, not expanded, and
  that the three NEW initially-out-of-list functions were brought into compliance (V-7/S1–S3).
- **E2** (Task-2, Task-4 report item): record the verified ureq 2 3xx-delivery behavior against
  D-T2-5's wording AND the `send_string`-instead-of-`send_json` (json-feature-off) equivalence —
  both loopback-pinned. No Task-3 action beyond this pointer.
- **E3** (Task-2, Task-4 report item): note D-F (roster-A test-11's absent/non-JSON body arm) and
  D-A/D-B/D-C (the F-T2-1 cap re-cut + roster splits) as as-built facts. No Task-3 action beyond
  this pointer.

## 9. Conclusion

Every TODO Task-3 bullet and every plan decision D-T3-1..15 is implemented as specified, with the
deviations V-1..V-9 all compiler-arbitrated/sanctioned single-local adaptations that are recorded
in code, commit messages, and the structure map. Exactly ONE violation remains: **V-10 —
`src/app/pipeline/test_helpers.rs` at 202 lines, two over the binding 200-line cap**, introduced
by the plan-prescribed `CompletedOutput` ripple and missed by every earlier pass. It requires a
small mechanical extraction as proposed in V-10 (no structural change, no amendment plan), after
which Task 3's surface is fully rule-compliant and the workflow proceeds to 4.6.

**Verdict (initial pass — superseded 2026-10-14; corrected verdict in §11): NOT ADHERENT — sole blocker: `src/app/pipeline/test_helpers.rs` at 202 lines (> 200 hard cap, +2 from the Task-3 `CompletedOutput` ripple); all TODO Task-3 bullets and plan decisions otherwise verified MET with the deviations V-1..V-9 adjudicated ACCEPTABLE; the mechanical 2-block extraction fix is proposed in §5 V-10.**

---

## 10. Re-check (2026-10-14) — V-10 resolution verification

Same session, architector re-check only (no code by this agent). The implementer applied the §5
V-10 proposal in commit `c838793` — `refactor: split the scripted chat source out of the pipeline
test helpers`. Verified fresh this pass:

1. **Extraction faithful to the proposal.** The `ScriptedSource` unit — the struct, the
   `schedule`/`failing` constructors, and the `impl MessageSource` block — moved from
   `src/app/pipeline/test_helpers.rs` into the new `src/app/pipeline/source_helpers.rs` with
   byte-identical bodies and the identical `Message-source fake:` doc comment; only the
   now-unused imports (`VecDeque`, `MessageSource`, `ChatError`) left `test_helpers.rs` with it.
   `#[cfg(test)]` module wiring follows the existing `src/app/pipeline/mod.rs` pattern
   (`#[cfg(test)] mod source_helpers;`); the single cross-file consumer
   (`runner_tests.rs`) gained the `use ...source_helpers::*;` glob mirroring its
   `test_helpers::*` import. Commit scope = exactly the 5 expected files
   (`source_helpers.rs` +35, `test_helpers.rs` −34, `pipeline/mod.rs` +2, `runner_tests.rs` +1,
   `.agent/project-structure.md` pipeline entry 13→14 files) — NO `src/app/run/` or behavioral
   surface touched.
2. **V-10 resolved.** Fresh line counts: `test_helpers.rs` = **168 lines**, `source_helpers.rs`
   = **35 lines** — both ≤ 200 (actuals 168/35 vs the proposal's ≈157/≈45 estimate; both under
   the cap, the delta is blank/import placement). No new `#[allow]` anywhere.
3. **No new violations.** Diff surface confined to the fix; test total UNCHANGED at exactly
   **355** (no test added, removed, renamed, or re-asserted; the glob import adds none);
   `#[allow(dead_code)]` inventory unchanged (8 attribute lines + the 1 comment-text mention at
   `src/youtube/mod.rs:68`); run-path boundaries intact (the §3/§4/§6 evidence of the initial
   pass remains valid — nothing it covered was edited).
4. **Fresh verification (implementer-recorded, gate log verified by this agent):** fmt-check 0;
   clippy 0; `cargo test --locked app::run` 17 passed; full gate exit 0 with
   `ALL CHECKS PASSED` — `logs/checks/20261014T205831Z.log` (355 passed / 0 failed).

## 11. Corrected verdict

**Verdict: ADHERENT WITH ACCEPTABLE DEVIATIONS (re-check 2026-10-14: V-10 resolved by commit c838793; gate green 355 tests).**
