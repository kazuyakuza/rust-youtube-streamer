# Task 4 (4.3) Simplification Plan — Phase 06 Docs/Metadata Landed Set

Plan date: 2026-10-26 · Owner: Code-Simplifier (4.3) → executed by IMPLEMENTER (junior, 50% restriction — execute EXACTLY, no judgment calls) · Type: RECORDS-ONLY fix (docs/metadata; ZERO src, ZERO test, ZERO report-file edits; NO gate run, NO builds).
Binding sources: `.kilo/plans/20261025-phase06-task4-verification-docs.md` (the Task-4 docs plan — its verbatim-mandated wording is respected; see §3 F1).
Branch/HEAD at review: `feat/phase06-youtube-live-chat` @ `ae00f65`, tree clean.
Landed set reviewed: `e22b7ba` (README + `docs/google-cloud-prerequisites.md` + `docs/production-run.md`), `3bc2f1a` (`.agent/project-info/context.md`/`tech.md` + `.agent/project-structure.md`), `ae00f65` (`.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`).

## 1. Review verdicts (what was checked, against code ground truth)

- Redundancy sweep: every fact repeated across the landed set was traced to its TODO-mandated artifact (TODO lines 97–100 demand per-file coverage in docs, metadata, AND the report); no cross-file repetition is removable without breaking a mandate or the pinned-verbatim constraints. The one candidate intra-file duplication (stage-10 vs Live Chat section worker facts) is VERBATIM-MANDATED by the binding plan — LEAVE (adjudication F1). VERDICT: no dedup edits.
- Stale-duplication sweep: the edited Overview/README/prerequisites pointers are summary-level and consistent; the Known Stream-Health "pending" sentence is intentionally kept (report line 106 confirms). VERDICT: no stale-dup edits.
- Factual-consistency sweep (report/docs/metadata vs `src/` code truth): FIVE defects found — S1–S5 below. Three are factual errors introduced by the Task-4 edits; one is a plan-inherited self-contradiction; one is a transcription typo. Everything else verified consistent: row renumbering + referencing sentences (rows 9/10), the three typed error lines vs `chat_failure.rs`, backoff/dedup/capacity/budget numbers vs `chat_worker.rs` constants, the quota table vs report (byte-identical), the 458/0 + artifact-size figures (report ↔ context.md ↔ README), the dead-code ledger arithmetic (8+3+1), the test-roster counts (51/202), the anchors (`#live-chat-behavior-diagnostics-recovery-privacy-quota` matches the H2 slug in both README links and the docs TOC).

## 2. Required steps (each atomic, fully specified; execute S1→S5 in order)

### S1 — `docs/production-run.md`, Live Chat → Prerequisites, 4th bullet: "SERVER-PROVIDED ONLY" contradicts the recorded bounded default

The code has `DEFAULT_CHAT_POLL_INTERVAL_MS = 10_000` (`src/app/run/chat_worker.rs` line 36; absent/zero server field keeps the bounded default per `src/app/run/chat_worker_delivery.rs` lines 47–50). The Task-4 report records it verbatim ("a bounded default (10 s) applies only when the server omits `pollingIntervalMillis`", report line 73) and `docs/google-cloud-prerequisites.md` (also edited by `e22b7ba`) already states "a bounded default applies only when a response omits it". The docs bullet's stronger claim is the outlier factual inconsistency.

FIND (whole line, `docs/production-run.md`, currently line 144 — must match exactly once):

`- There is NO new configuration field for chat: polling pacing is SERVER-PROVIDED ONLY (the API's `pollingIntervalMillis` response field) — nothing to configure, and no operator override exists.`

REPLACE WITH (exactly):

`- There is NO new configuration field for chat: polling pacing is SERVER-DIRECTED (the API's `pollingIntervalMillis` response field; a bounded 10 s default applies only when a response omits it) — nothing to configure, and no operator override exists.`

Invariants: the "NO new configuration field" / "nothing to configure" / "no operator override" claims stay; only the ALL-CAPS emphasis word changes and the default parenthetical is added, mirroring the prerequisites-doc phrasing.

### S2 — `docs/production-run.md`, Live Chat → Expected Behavior, 1st bullet: transcription typo `POLLINGS`

The binding plan (§1.3/line 64) says "the retrieval method: **polling** `liveChatMessages.list`"; the implementer's rendering produced the ungrammatical "the worker POLLINGS". This restores the plan's fact with correct grammar (one word).

FIND (substring of the bullet line, currently line 148 — must match exactly once):

`the worker POLLINGS `liveChatMessages.list``

REPLACE WITH (exactly):

`the worker POLLS `liveChatMessages.list``

### S3 — `docs/production-run.md`, Live Chat → Diagnostics, closing sentence: `liveChatEnded` listed as a fatal-detail example

`ChatEnded` is the GRACEFUL signal and NEVER reaches the fatal terminal mapping: `src/app/run/chat_failure.rs` line 7 ("`ChatEnded` never reaches here"), `src/youtube/live_error/mod.rs` lines 22–23 ("the worker's GRACEFUL end signal"), pinned by `chat_worker_tests.rs` line 59 (`assert!(terminal_of(&channel).is_none(), "ChatEnded is graceful")`). The same doc's Failure Recovery table row (line 166) and the report (lines 64/72) both classify `403 liveChatEnded` as graceful. The docs sentence inherited the defect from the binding plan line 65, which self-contradicts its own line 66 — resolved here in favor of the code + report + same-file table.

FIND (whole line, `docs/production-run.md`, currently line 160 — must match exactly once):

`The `detail` is the redacted Display of the underlying typed YouTube error (e.g. the `liveChatDisabled`/`liveChatEnded`/`liveChatNotFound` reasons) — never a raw response body.`

REPLACE WITH (exactly):

`The `detail` is the redacted Display of the underlying typed YouTube error (e.g. the `liveChatDisabled`/`liveChatNotFound` reasons) — never a raw response body.`

### S4 — `.agent/project-info/tech.md`, Lifecycle paragraph: shutdown order wrong (join placed BEFORE the FFmpeg stdin EOF)

Code truth (Phase-06 §2.3 refinement): `PreparedStreaming::run` lets the pipeline end (loop break + sink EOF + FFmpeg wait happen inside `pipeline.run()`), and only then `gate.finish()` joins the confirmation thread and the recorded chat worker (`src/app/run/streaming.rs` lines 4–7: "joined in `gate.finish()` after the sink EOF"; `src/app/run/chat_worker.rs` lines 4–6; `src/app/run/gate.rs` line 16). The README ("...then the bounded chat-worker join at the gate finish") and the report (line 75) both record the join LAST; the new tech.md sentence contradicts them. The binding plan (line 135) does NOT mandate this sentence — it is an implementer coinage error.

FIND (substring of the Lifecycle paragraph, `.agent/project-info/tech.md` — must match exactly once):

`shutdown order is stop flag (frame loop + chat worker) → chat-worker join → FFmpeg stdin EOF → wait.`

REPLACE WITH (exactly):

`shutdown order is stop flag (frame loop + chat worker) → wait cancellation → FFmpeg stdin EOF → wait → the bounded chat-worker join at the gate finish.`

Invariants: "(frame loop + chat worker)" (shared flag) stays; the rest of the paragraph (stage 7.5 spawn sentence, resource-fate sentence, operator-guide pointer) is untouched.

### S5 — `.agent/project-structure.md`, `run/mod.rs` bullet: handler claimed to flip THREE flags; the code flips TWO

`install_stop_pair` (`src/app/run/mod.rs` lines 108–121) flips exactly the `StopFlag` + the `WaitCanceller`; the chat worker SHARES the frame-loop `StopFlag` (a clone passed at line 151–155; `chat_worker.rs` lines 13–14 "the Ctrl+C handler flag-stops it concurrently"; report line 75 "same single handler"). Listing "the chat-worker stop flag" as a third flipped object is a factual error introduced by `3bc2f1a` and contradicts the report's phrasing.

FIND (substring of the `mod.rs` bullet line, `.agent/project-structure.md` — must match exactly once):

`install_stop_pair` (the app's single Ctrl+C handler flips the frame-loop `StopFlag`, the `WaitCanceller` AND the chat-worker stop flag)

REPLACE WITH (exactly):

`install_stop_pair` (the app's single Ctrl+C handler flips BOTH the frame-loop `StopFlag` — shared with the Phase-06 chat worker — and the `WaitCanceller`)

## 3. NO-action adjudications (recorded; implementer does NOT touch these)

- F1: stage-10 parenthetical vs Live Chat Expected Behavior/Failure Recovery restating the worker start condition, 30 s budget, and ≤1 s/≤30s join bounds — genuine intra-file duplication, BUT the binding plan line 82 VERBATIM mandates the stage-10 facts and lines 64/66 mandate the section facts (precedent: stream-health F1 "both verbatim-mandated ... removal buys only cosmetics and creates an adherence deviation"). LEAVE.
- F2: the quota table appearing in both `docs/production-run.md` and the report — pinned verbatim by caller constraints and TODO-mandated report completeness. LEAVE.
- F3: "re-running `auth` is NOT required for chat" stated in docs Prerequisites + docs step 6 + report scope line + report procedure item (a) — the report's (a)–(g) deliberately mirrors step 6 verbatim (report line 106 records it); removing either desyncs the record or the procedure. LEAVE.
- F4: the "renders author/text when present / frames stay black while idle" summary appearing three times in README (Current Status, run bullet, Production Run section) + docs Overview — section-summary convention, only status clauses were edited; ≤ twice per document. LEAVE.
- F5: retrieval-method + sync_channel(64) bridge facts stated in tech.md, report, project-structure, and docs — each required by its own TODO bullet (97 docs / 98 metadata / 99 report). LEAVE.
- F6 (FLAGGED TO CALLER — not an implementer step, not executed): docs Expected Behavior + manual step 6 phrase the baseline-discard consequence only as "messages sent before the stream went live", while the report Known Limitations (line 145) records the wider window ("messages arriving between stream-active and the worker's first poll are discarded with the history page"). Every doc statement is true (subset); reconciling would ADD content to docs and force a report mirror — outside the simplification categories; caller decides whether to open a separate precision task.
- F7: report file itself — zero edits (audit record; the plan-mandate supersession lives in this plan + the adherence pass).

## 4. Verification (after S1–S5; all single host commands; NO gate, NO builds — nothing under `src/` or tests moves, the 4.2 closing gate stands)

1. `rg -c "SERVER-PROVIDED ONLY|POLLINGS" docs/production-run.md` → no matches (exit 1, zero hits).
2. `rg -n "liveChatEnded" docs/production-run.md` → exactly ONE hit: the Failure Recovery table's GRACEFUL row.
3. `rg -c "chat-worker join → FFmpeg stdin EOF" .agent/project-info/tech.md` → no matches; `rg -c "the bounded chat-worker join at the gate finish" .agent/project-info/tech.md` → 1.
4. `rg -c "AND the chat-worker stop flag" .agent/project-structure.md` → no matches.
5. `rg -c "live-chat-behavior-diagnostics-recovery-privacy-quota" README.md docs/production-run.md` → 3 total hits (README 2 + docs TOC 1) — anchors byte-unchanged.
6. `git status --porcelain` → exactly the three intended files modified (plus this plan file per §5); `git diff --stat` → docs/production-run.md 3 lines changed, tech.md 1, project-structure.md 1.
7. Hygiene: zero `ya29`/`AIza`/`rtmps://` additions in the diff; `logs/`, `dist/`, `config/config.json`, `credentials/*` never staged (gitignore-compliance rule).

## 5. Commits (branch `feat/phase06-youtube-live-chat` only; push is NOT part of this plan)

1. Stage ONLY `.kilo/plans/20261025-phase06-task4-simplification.md` → `chore: track the phase06 task4 simplification plan`.
2. Stage ONLY `docs/production-run.md`, `.agent/project-info/tech.md`, `.agent/project-structure.md` → `docs: correct the phase06 chat pacing, error-detail and shutdown-order records`.

## 6. Stop-and-report (any hit → STOP, report to caller; do not improvise)

- Any FIND string does not match EXACTLY once in its file.
- Working tree not clean at `ae00f65` (or this plan's tracking commit) before starting.
- Any pressure to edit `src/`, tests, the report file, any other `.kilo/plans/` file, README, `docs/google-cloud-prerequisites.md`, the quota table, any Display line, or any constant value.
- Any verification anchor in §4 failing after an edit (revert that edit and report, do not restyle).
