# Plan — Task 4 Sub-step 4.3: Report Wording Simplification (Phase 02)

## Scope

- Single file in scope: `.agent/reports/20261010-phase-02-renderer-chat-store.md` (307 lines).
- Wording-level deduplication ONLY. Two edits, each a single-line replacement. Zero structural changes.
- Out of scope (do NOT touch): any code, `Cargo.*`, the TODO file, `context.md`, `project-structure.md`, Tasks 1–3 deliverables, the two other Task 4 commits, all other report sections.

## Hard Must-Not-Change List

The following are evidence and must remain byte-identical:

1. Every number: byte sizes (`1715264`, `3191353`), test counts (`16`, `19`, `81`), all exit codes (`exit=0`/`exit=1`/`exit=3`), all durations, all timestamps and log-file names, all 40/24 commit hashes, line numbers (`33/49`, `62/86`, `loader_tests.rs:86`), DLL lists, file/line counts (`307`, `118`).
2. The verbatim code blocks F1–F4 and the Host-side tracking checks block, including the closing cross-check sentence (line 144).
3. All NOT-RUN statements: the Manual Platform Tests block (lines 192–197) and limitations bullets (lines 297–300, 306–307).
4. The full Acceptance Criteria Verification table (lines 205–219), its flip rule (203) and closing line (221).
5. All Evidence Index entries (246–293): planning paths, prior reports, gate logs, observed commit list, and the Pending rows table contents.
6. All reported deviations: limitations bullets 301–305.
7. The Table of Contents (lines 5–17) and every heading text/level/anchor.
8. The Files Created/Changed table rows (29–51) and the `pending` convention note (52) — pending markers stay as-is (intentional no-back-patch convention; back-patching hashes would be a FACT change, forbidden in this step).

## Edit 1 — Overview: trim triple-stated container-only disclaimer

- File: `.agent/reports/20261010-phase-02-renderer-chat-store.md`
- Location: line 25, first sentence (Overview section, third paragraph).
- Reason: the container-only disclaimer appears 3 times in the document (Overview line 25, Environment Notes line 238, Limitations line 307). The Environment Notes sentence is a strict superset of the Overview sentence (it names fmt/check/test/clippy, both release builds AND artifact listings). Removing the duplicated middle clause loses no fact document-wide; the disclaimer still appears 3x via Environment Notes + Limitations + the retained Overview clause.
- Exact replacement (one whole line):

OLD line 25 (verbatim):

```text
Verification is container-only: the fresh dev-checks gate and both release builds below ran inside the pinned `rust:1.82` container on the Alpine VM; no native Windows/Linux runtime validation was performed. The tracked report itself is `.agent/reports/20261010-phase-02-renderer-chat-store.md`.
```

NEW line 25 (verbatim):

```text
Verification is container-only (see Environment Notes below); no native Windows/Linux runtime validation was performed. The tracked report itself is `.agent/reports/20261010-phase-02-renderer-chat-store.md`.
```

- Notes: keep the second sentence of the line unchanged; keep the semicolon clause `no native Windows/Linux runtime validation was performed` verbatim; the deleted clause's facts live in Environment Notes line 238 (unchanged).

## Edit 2 — Evidence Index: dedupe the pending-convention explanation

- File: `.agent/reports/20261010-phase-02-renderer-chat-store.md`
- Location: line 287 (intro sentence above the Pending rows table).
- Reason: the sentence "commits created after this report was written; hashes ... not back-patched" is already stated verbatim-in-substance at line 52 (Files Created/Changed note). The Evidence Index intro keeps a cross-reference instead of restating the explanation. The table below (lines 289–293) is NOT touched.
- Exact replacement (one whole line):

OLD line 287 (verbatim):

```text
Pending rows (commits created after this report was written; hashes recorded in the implementer summary, not back-patched):
```

NEW line 287 (verbatim):

```text
Pending rows (see the `pending` convention stated under Files Created/Changed):
```

## Implementation Guide (junior implementer — no judgment calls)

1. Open the report file. Make exactly TWO single-line replacements: the lines quoted as OLD 25 and OLD 287, each replaced by the corresponding NEW block verbatim, including backticks, parentheses and the trailing period/colon shown.
2. Do not re-wrap, re-flow, reorder or edit any other line. Do not touch tables or code blocks.
3. Verify after editing:
   - `git diff --stat` → exactly `1 file changed, 2 insertions(+), 2 deletions(-)`.
   - File still has 307 lines.
   - `git diff` shows ONLY the two OLD→NEW lines; no numeric character changes anywhere.
   - Sanity greps (each must still return ≥1 match): `1715264`, `3191353`, `81 passed`, `NOT RUN`, `pending`, `Container-only verification scope`.
4. Commit message (repo conventional style, markdown-only change): `docs: dedupe container-only and pending-convention statements in phase 02 report`.
5. No push unless instructed; push only to `origin`.

## Explicit Non-Edits (rejected candidates, recorded so they are not "rediscovered")

- "no native Windows runtime..." phrasing in F3 and the Windows Build Result verbatim caveat quotes: script output — untouchable evidence.
- "Build success is compilation evidence ONLY" (line 306): TODO §4 mandates this statement — keep.
- `#[allow(dead_code)]` Phase-03 retirement mentions (lines 21 and 301): 2x occurrences, one is a limitations bullet — keep both.
- MCP stderr caveat (lines 89 and 240): 2x below the 3x threshold; line 89 sits adjacent to the command evidence blocks — keep both.
- Overview self-identification sentence "The tracked report itself is ..." (line 25 tail): kept (deliverable-identifying context sentence).

## Flagged For Caller (NOT simplifications, no action in this task)

- O1: Report `pending` markers are now stale in fact (report commit `9f7a089`, TODO flip `19ec42c`, context refresh `ad8d176` all exist), but the report's own line-52 convention deliberately does not back-patch. Any change here = fact change → separate decision.
- O2: `context.md` "Immediate Next Steps" item 1 still reads "Phase 02 Tasks 1–2 are complete; Tasks 3–4 … close the phase" — stale post-Task 4, but it is Task 3 deliverable content and fixing it is a fact update, outside this simplification's scope.
