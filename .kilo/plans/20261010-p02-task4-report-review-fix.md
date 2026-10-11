# Fix Plan — Task 4 Report Review: Acceptance-Criteria Count

## Trigger

Code review of `.agent/reports/20261010-phase-02-renderer-chat-store.md` (commit `9f7a089`) and `.agent/project-info/context.md` (commit `ad8d176`) found a factual count mismatch against the authoritative TODO.

## Finding

- `.agent/todos/20261010/20261010-todo-2.md` contains **14** acceptance-criteria checkboxes.
- Commit `19ec42c` legitimately flipped **all 14** to `[x]`.
- The 14th criterion — `No OAuth, YouTube API, FFmpeg process, audio or live streaming behavior is implemented` — is backed by:
  - box-5 independence greps (`crate::youtube`, `crate::streaming`, `std::process`) → no matches in `src/chat`/`src/renderer`;
  - docs-accuracy audit item (c) → no claims of OAuth/YouTube/FFmpeg/audio/live streaming in README.md, docs/configuration.md, .agent/project-structure.md, or .agent/project-info/context.md;
  - `auth`/`run` placeholder tests still exit 3 (not implemented);
  - report sections "Known Limitations & Not-Run Checks" and "Manual Platform Tests Actually Performed".
- The report, however, states "13-row matrix", "All 13 boxes verified", and "all 13 acceptance criteria".
- `.agent/project-info/context.md` (Current Work Focus) also states "all 13 acceptance criteria ... are verified `[x]`".

This is a documentation accuracy defect, not an evidence fabrication. All evidence is real and already recorded; only the count and the matrix row are missing.

## Fixes Required (evidence corrections only)

1. **`.agent/reports/20261010-phase-02-renderer-chat-store.md`**
   1. In the "Acceptance Criteria Verification" intro, change "13-row matrix" to "14-row matrix".
   2. Add a 14th row to the acceptance-criteria table:
      - `#`: 14
      - `Acceptance criterion (short)`: No OAuth, YouTube API, FFmpeg process, audio or live streaming behavior is implemented
      - `Evidence (observed this task unless noted)`: box-5 greps (`crate::youtube`, `crate::streaming`, `std::process`) → no matches in `src/chat`/`src/renderer`; docs-accuracy audit item (c) → no implementation claims in README.md/docs/configuration.md/.agent/project-structure.md/.agent/project-info/context.md; `auth`/`run` placeholder tests still exit 3; limitations/NOT-RUN sections
      - `Decision`: [x]
   3. Change "All 13 boxes verified → all flip `[ ] → [x]` in the TODO" to "All 14 boxes verified → all flip `[ ] → [x]` in the TODO".
   4. Scan the report for any remaining "13" references to the acceptance-criteria count and update to "14".

2. **`.agent/project-info/context.md`**
   1. In the `## Current Work Focus` paragraph, change "all 13 acceptance criteria in `.agent/todos/20261010/20261010-todo-2.md` are verified `[x]`" to "all 14 acceptance criteria in `.agent/todos/20261010/20261010-todo-2.md` are verified `[x]`".

3. **Do NOT modify** `.kilo/plans/20261010-p02-task4-tests-builds-report.md`; its "13 boxes" wording is a historical planning mismatch and is noted in this review finding. It is not a runtime deliverable.

## Out of Scope / Must Not Change

- Do NOT re-run F1–F4 gates or builds.
- Do NOT modify any `src/`, `Cargo.toml`, `Cargo.lock`, `config/`, `fonts/`, `scripts/`, `Dockerfile`, or `docker-compose.yml` file.
- Do NOT flip any TODO checkbox state (all 14 are already correctly `[x]`).
- Do NOT invent new evidence, byte sizes, test counts, or commit hashes.

## Verification After Fix

- `grep -n "13" .agent/reports/20261010-phase-02-renderer-chat-store.md` should show only incidental hits (e.g., year 2026, file sizes, unrelated counts), not "13 acceptance criteria/boxes".
- The acceptance-criteria table in the report must contain exactly 14 rows, numbered 1–14.
- `.agent/project-info/context.md` must state "14 acceptance criteria".
- `git diff --stat` after the fix commit(s) must show only the two markdown files changed.

## Suggested Commit Message

`docs: correct acceptance-criteria count from 13 to 14 in Task 4 report and context`
