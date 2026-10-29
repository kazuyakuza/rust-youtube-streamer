# Phase 07 Task 1 — Simplification Plan (workflow step 4.3, code-simplifier half)

> Target record under review: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (incl. §14 addendum) and the `.agent/project-info/context.md` block `## Phase 07 Task 1 — streamList client-path research record landed (2026-10-29)`.
> Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` Task 1; global plan `.kilo/plans/20261029-phase07-streamlist-migration.md`.
> Task 1 is a RESEARCH/DESIGN deliverable — no production code exists. This plan fixes ONLY a structural defect in the record; NO research content, citation, or quote wording is touched.
> Executed by the 4.2-adjacent implementer (junior, 50% restriction): every edit below is verbatim-transcription work, zero judgment calls.

## Verdict

- ONE required simplification fix: **S1** (corrupted non-Latin token inside the binding P7-11 constraint clause — transcription-error risk for the Task-2/3 implementers).
- All other reviewed candidates: **LEAVE** (matrix below). Total record change ≈ 1 line (<1% of the record; far under the 30% rewrite threshold).

## S1 — Repair the corrupted token in §7 (P7-11 CLOSED-classifier constraint)

- File: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
- Location: §7, first bullet under the mapping table (the line beginning `- The app-level CLOSED failure classes are REUSED as-is:`)
- Anchor text (exact, unique in the file — contains the stray CJK token `一身`, the only Han-character occurrence in the record):

  `STAYS CLOSED (the一身 transient list unchanged — no arm is added)`

- Replacement text (exact):

  `STAYS CLOSED (the entire transient list unchanged — no arm is added)`

- Constraints: change NOTHING else on the line or in the file; the em-dash, backticks, and rest of the sentence stay byte-identical.
- Meaning preserved: the constraint is that `is_transient_failure`'s existing transient arm list is not extended; every new-class routing decision goes through the strategy-loop match instead (the rest of the bullet already states this).

## Verification (after applying S1)

1. `grep` the plan file for `\p{Han}` (or any non-ASCII CJK): MUST return zero matches.
2. Confirm the corrected phrase exists verbatim: `STAYS CLOSED (the entire transient list unchanged — no arm is added)`.
3. Confirm no other diff hunks exist in the file (`git diff` shows the single-line S1 change only).

## LEAVE matrix (reviewed candidates; no edits)

| # | Candidate | Finding | Verdict |
|---|---|---|---|
| F1 | Backoff base duplication (§5 P7-5 step 3 vs §6 P7-6 ledger) | Both statements agree: 1 s base via `CHAT_SLEEP_SLICE_MS`, 60 s cap via `CHAT_BACKOFF_MAX_MS`, budget 10; cross-checked against the actual `backoff_delay_ms(attempt, base_ms)` floor/cap implementation (`src/app/run/chat_worker_delivery.rs`). §5 carries the formula, P7-6 the constants ledger — complementary locality, identical values. | LEAVE |
| F2 | Dependency lines duplication (P7-2 vs §13.4b) | §13 references P7-2 by pointer ("add the P7-2 dependency lines EXACTLY") — single source of truth, no duplicated literals. §14.2's five observed `--precise` pins are covered by P7-2's general pin-discipline rule; no contradiction. | LEAVE |
| F3 | §7 error-table rows vs §9 worker-roster test names | §9's "FailedPrecondition dual-branch" names the gRPC status family; the variant is `ChatPrecondition` (§7, fully specified dual branch). Behavior is defined once in §7; test names are descriptive. No drifting decision. | LEAVE |
| F4 | Retirement lists (§12 vs §16 vs context.md bullet) | §16 contains no retirement list (it is the not-done ledger); the context.md "RETIRED … O6 … survive untouched" bullet matches §12's retire/survivor lists in substance. | LEAVE |
| F5 | context.md note truthfulness | ≤10 lines (heading + 5 bullets); pins (tonic 0.13.1/1.75, prost 0.13.5/1.71.1, tokio 1.53.1/1.71), TLS story, probe verdicts, five transitive pin versions, §12 retirement, §14 pointer, 2026-10-29 dates — all verified against the record; all cited symbols verified in-tree. | TRUE — LEAVE |
| F6 | Open item in §9 ("name finalized at Task 2 within the module's naming conventions") | Deliberate, bounded delegation of a local naming detail — within the junior's allowed latitude; every structural decision (trait/method signatures, file names, constants) is closed-form. P7-1..P7-12 are transcribable without re-choosing anything. | LEAVE |
| F7 | §7 `LiveApiError` "17 → 19" count claim | Consistent with the Phase-06 record (15→17) and the two NEW variants named once (§7 table + bullet). | LEAVE |

## Out-of-scope flag (for the caller / 4.5b adherence — NOT an edit here)

- TODO "Current Repository Context" line 14 says the operator's 2026-10-29 polling-run confirmation is to be "recorded in `context.md` and the completion report". The Task-1 context note spec (§13 step 5, verbatim content list) does not include it, and TODO Task 4 explicitly owns it ("Update … `context.md` (including the operator's 2026-10-29 polling-run confirmation note)"). Ownership therefore lands on Task 4; the Task-1 note's omission is compliant, but the caller may want the 4.5b re-check to state the allocation once, explicitly. No change made by this plan.

## Footer

Plan: `.kilo/plans/20261029-phase07-task1-simplification.md`
Produced by: 4.3 code-simplifier half (2026-10-29). NOT applied here: the single S1 edit is handed to the implementer; no commit performed by this pass.
