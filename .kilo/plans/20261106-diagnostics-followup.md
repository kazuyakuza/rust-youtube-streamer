# Global Plan — Diagnostics Findings Follow-up (provider timestamp + reconnect reason)

Source TODO: `.agent/todos/20261106/20261106-todo-1.md` (Pattern C: 2 tasks under `## Tasks`).
Base: `main` at `7437c62` (v0.14.0), working tree clean. The TODO does NOT contain the auto-approve string → user approval required.

## Workflow Step Mapping (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer (branch `fix/diagnostics-followup`)
- Step 3: Version Update => implementer (**0.14.0 → 0.14.1**, patch: fixes to the shipped diagnostics surface; commit `chore: bump version to 0.14.1`)
- Task 1: 4.1b => architector → 4.2 => implementer → 4.3 => code-reviewer + code-simplifier (fixes => implementer) → 4.4 => docs-specialist → 4.5b => architector → 4.6 => implementer
- Task 2: 4.1b => architector → 4.2 => implementer → 4.3 => code-reviewer + code-simplifier → 4.4 => docs-specialist → 4.5b => architector → 4.6 => implementer
- Step 5: TODO File Completion => implementer (rename `-DONE`; release-artifact rebuild for BOTH platforms with `ls -l` cross-check — REQUIRED by the TODO's own global criterion since production code changes; merge to `main`; push `origin` ONLY)

Front-end determination: NOT front-end (no 4.1a).

## Global Pre-Analysis (technical & architecture decisions)

- Findings provenance (operator-supplied, `logs/run/20261106-021348.log` + `logs/diagnostics.log`): per-message rows all carry `provider_epoch_seconds:null`; reconnect rows carried no cause (`reconnect_count` 1→17 over ~4 min, flat 1000 ms backoff, zero message loss).
- Schema strategy: `TimingEvent` message rows ALREADY serialize the `provider_epoch_seconds` key (always null) ⇒ Task 1 populates it — zero JSON-shape change. Reconnect rows gain the new `reason` key ONLY on reconnect rows ⇒ the `QueueEvent` skeleton must serialize `reason` with `skip_serializing_if = "Option::is_none"` so every OTHER queue row stays byte-identical (schema-compatible; Task-2 plan decision).
- Value routing (Task 1, to be verified at 4.1b against real code): provider epoch flows provider `publishedAt` → `live_chat_mapping`/`rfc3339` parser → carrier timestamp → (C-1 conversion) `ChatMessage.timestamp` → frame loop's `note_message_dequeued(&ChatMessage)` can read it directly (dequeued row); `report_enqueue_success`/`InFlightEntry` must begin carrying it so `rendered`/`written` rows can (receipt/enqueue report from `send_stop_aware` — the full `ChatMessage` IS available at the try_send site). One extra `Option<u64>` on the cap-128 in-flight entry; no new timers, no cross-domain math.
- Reason routing (Task 2, to be verified at 4.1b): the re-establishment trigger already differentiates a server clean close from a transient failure inside `chat_stream_delivery.rs`; the collector's `report_*` for reconnect/backoff gains one non-secret lowercase-underscore label (exact words chosen at 4.1b, e.g. `clean_close` / `transient_failure`); the declaration-shape change must not disturb other report sites.
- Disabled-by-default guarantee: both fixes live inside collector internals — `from_config → None` gating untouched; zero cost when diagnostics off.
- No dependencies, no `#[allow(` additions, ≤200-line caps with established `#[path]` roster pattern, ≤2-args rule (parameter objects where needed).
- Jobs owned by this TODO only: the two fixes + truthful docs row/field mentions + tests; unrelated refactors prohibited.

## Per-Task Pre-Analysis

### Task 1 — Provider publication timestamp in per-message diagnostic rows (non-front-end)

- Verify the actual loss point first (mapping → conversion → collector): plan must name the exact functions where the timestamp exists and where it is dropped (4.1b with file/function anchors).
- Populate the three stage rows from one source of truth (the message's `timestamp` at dequeue; the in-flight entry stamped at enqueue/receipt for rendered/written).
- Absent timestamps stay null (row shape already tolerates it).
- Tests: populated pin, absent pin, in-flight-eviction/bounds guard, summary unchanged; re-pin any historically null-row expectations truthfully.
- Docs: one truthful sentence in the observability section (provider-supplied wall-clock carried per message when available); no real-Google claims.

### Task 2 — Reconnect cause label (non-front-end)

- Pick the fixed label words at 4.1b from the classifier's clean-close vs transient-failure distinction; never raw provider internals; document the labels where reconnect rows are described.
- Additive serialization with `skip_serializing_if` so other queue rows stay byte-identical; pins: both cause labels, absent-on-other-rows shape.
- Tests: delivery-classification pins for both cases + row-shape pins; all offline.

## Risks

- R1: Historical test expectations pin `provider_epoch_seconds:null` on enabled rows — 4.2 must re-pin ONLY the rows whose truth changes (populated cases) and keep genuinely-absent cases null.
- R2: `QueueEvent` skeleton edit touches the shared constructor path — 4.2 must verify every existing queue-row pin stays byte-identical (the skip-attribute is the mechanism, not null emission).
- R3: Line caps may fire (collector/stages/files near known tight marks) → STOP protocol + caller adjudication (established precedent).
- R4: The 2-args rule on modified `report_enqueue_*` signatures — prefer the parameter-object/entry extension, plan encodes it.
- R5: Version bump 0.14.1 + artifact rebuild at step 5 (both platforms, `ls -l` cross-check) since production code changes — record in close-out.

## Approval

Presented via `question` tool: (a) Approve Global and Tasks Plans (auto-approve per-task plans), or (b) Approve Global Plan (per-task plans presented for approval).
