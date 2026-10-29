# Phase 07 Task 1 — Review Fixes (workflow step 4.3, code-reviewer half)

> Target deliverable: `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` (incl. §14 addendum) and the `.agent/project-info/context.md` Phase-07 Task-1 block.
> Authority: TODO `.agent/todos/20261029/20261029-todo-1.md` Task 1; global plan `.kilo/plans/20261029-phase07-streamlist-migration.md`.
> Task 1 is RESEARCH/DESIGN — no production code. Fixes are content/hygiene only.

## Verdict

- **Defect found:** one corrupted token in the binding plan record (stray CJK character in the §7 P7-11 constraint clause).
- **Git-hygiene item:** the code-simplifier half produced `.kilo/plans/20261029-phase07-task1-simplification.md` but left it untracked; it must be committed with the fix so the 4.3 reasoning is preserved.
- All other checked items (TODO fidelity, citation integrity, internal coherence, §14/context note accuracy, branch/HEAD, commit stats, secret/allow audits, STOP ledger) **PASS**.

---

## D1 — Repair corrupted token in §7 (P7-11 CLOSED-classifier constraint)

- **File:** `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
- **Section:** §7, first bullet under the typed-error mapping table, line 257.
- **Exact text to replace:**

  ```text
  - The app-level CLOSED failure classes are REUSED as-is: `is_transient_failure` STAYS CLOSED (the一身 transient list unchanged — no arm is added); the establishment/mid-session branching for `ChatPrecondition` lives in the strategy-loop policy (a pure match, pinned by test), NOT through the shared classifier.
  ```

- **Replacement text (everything else byte-identical, only `the一身` → `the entire`):**

  ```text
  - The app-level CLOSED failure classes are REUSED as-is: `is_transient_failure` STAYS CLOSED (the entire transient list unchanged — no arm is added); the establishment/mid-session branching for `ChatPrecondition` lives in the strategy-loop policy (a pure match, pinned by test), NOT through the shared classifier.
  ```

- **Why it matters:** the stray CJK token is a transcription error that would propagate into Task-2/3 transcription work and could confuse tooling or future diffs.

---

## D2 — Track the code-simplifier plan already produced

- **File:** `.kilo/plans/20261029-phase07-task1-simplification.md`
- **Issue:** the file exists in the working tree but is untracked (`git status --porcelain` shows `?? .kilo/plans/20261029-phase07-task1-simplification.md`).
- **Fix:** stage and commit it together with the D1 fix. Do **not** edit its contents.
- **Rationale:** it is a code-reviewer/simplifier-owned plan file per `.kilo/rules/markdown-generation-rule.md` and records the 4.3 LEAVE matrix and the S1 finding that the implementer is applying.

---

## Implementer steps (exact, no choices)

1. Verify state: on branch `feat/phase07-streamlist-retrieval`, HEAD `ed583a7`, tree otherwise clean except for the untracked simplification plan.
2. Apply D1 using a single `edit` replacement in `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`.
3. Verify the replacement:
   - `grep -F "the entire transient list unchanged" .kilo/plans/20261029-phase07-task1-streamlist-research-design.md` returns exactly one hit.
   - `grep -P "[\x{4e00}-\x{9fff}]" .kilo/plans/20261029-phase07-task1-streamlist-research-design.md` returns zero hits (no remaining CJK characters).
4. Stage and commit both files:
   - `git add .kilo/plans/20261029-phase07-task1-streamlist-research-design.md`
   - `git add .kilo/plans/20261029-phase07-task1-simplification.md`
   - `git commit -m "docs: fix stray CJK token in Task-1 plan and track the simplification review"`
5. `git status --porcelain` must be EMPTY before proceeding to 4.4.

---

## Verification matrix (reviewed and passed before fix)

| Check | Result | Evidence |
|---|---|---|
| TODO Task-1 bullets mapped to plan sections | PASS | §0 table covers every bullet; §15 checklist has all `[x]` |
| §1 citation internal coherence | PASS | streamList quote, error codes/quota/scope/proto table numbers consistent across sections |
| P7-5 backoff base vs P7-6 constants | PASS | 1-s base via `CHAT_SLEEP_SLICE_MS`, 60-s cap, 10-failure budget |
| §6 seam claims vs P7-9 | PASS | `sync_channel(64)`, `try_recv`, stop-aware send, terminal cell all preserved |
| §7 variant count 17→19 vs table rows | PASS | exactly two new variants (`ChatPrecondition`, `InvalidChatRequest`) |
| §12 retirement list vs §9 test strategy | PASS | REST-list rosters retire, proto-side mapping rosters added, O6 rosters survive |
| §13 steps vs §14 recorded results | PASS | 5 transitive pins + tokio pin, both probes exit 0 |
| §14/context.md note accuracy | PASS | pins, probe verdicts, STOP not fired, retirement, ≤10-line block all match |
| Branch/HEAD | PASS | `feat/phase07-streamlist-retrieval` @ `ed583a7` |
| Commit file stats | PASS | `f8628e2` = only plan file; `ed583a7` = plan + context.md |
| No secrets/tokens in committed docs | PASS | no `ya29`/`AIza`/real tokens; `key=` only in rule-conflict prose |
| No new `#[allow(` in committed docs | PASS | only prose references to the rule |
| STOP-condition ledger | PASS | STOP (i) not fired (probes passed); STOP (ii) not fired (Path A documented) |

---

Plan: `.kilo/plans/20261029-phase07-task1-review-fixes.md`
Produced by: 4.3 code-reviewer half (2026-10-29).
