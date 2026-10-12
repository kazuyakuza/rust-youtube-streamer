# Task 5 Review Fix Plan — Phase 03, FFmpeg Local Pipeline

**Date:** 2026-10-12  
**Reviewer step:** TODO Task 5, sub-step 4.3 (code-reviewer)  
**Branch:** `feat/phase03-ffmpeg-local-pipeline`  
**Target file:** `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`  

## Findings Summary

| # | Severity | Finding | Fix Owner | File(s) |
| --- | --- | --- | --- | --- |
| 1 | **Medium** | Completion report `Files Created/Changed` table still lists the report and task-5 plan file commits as `pending`, but both files are already tracked with resolved hashes (`82ab621` and `c249138`). The plan spec §7 requires "hashes resolved (no `pending` left)". | docs-specialist / 4.4 | `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` |
| 2 | **Low** | Documentation Accuracy Audit claims `docs/demo.md` is 131 lines; the actual file is 87 lines. This is an invented line count that contradicts the live tree. | docs-specialist / 4.4 | `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` |

No source-code, test, build, or production-behavior changes are required. All code-level acceptance criteria remain satisfied.

## Verdict on Code/Tests/Docs (pre-fix)

- **Fake-FFmpeg tests:** PASS. `#[cfg(all(test, unix))]` gating present via two `#[cfg(test)]` + `#[cfg(unix)]` attribute pairs in `src/streaming/ffmpeg.rs`. POSIX `#!/bin/sh` bodies never consult a real `ffmpeg`. Last-arg output-path contract holds. The six test assertions match the plan. `FfmpegProcess::start` appears only inside the `start_fake` retry helper (two call sites) in `ffmpeg_tests.rs`.
- **Real-FFmpeg smoke:** PASS. Loud `assert!` prerequisites for `/usr/bin/ffmpeg` and `/usr/bin/ffprobe`. ffprobe asserts `width=64`, `height=36`, `r_frame_rate=5/1`, `duration` in `[1.5, 2.5]`. Version recorded (`5.1.9-0+deb12u1`) matches observed container evidence.
- **Dockerfile:** PASS. Single added `RUN` line installs `ffmpeg`/`ffprobe` as dev tooling only; comment explicitly states "NOT bundled with either release artifact". Image rebuild recorded.
- **Docs accuracy:** PASS for content (README, `docs/demo.md`, `docs/configuration.md`, `project-structure.md`, `tech.md`, `context.md`). Only the report's own internal line-count claim is wrong.
- **Housekeeping:** PASS. `logs/.gitkeep` tracked; 4 `.gitkeep` files tracked; `videos/` ignored at `.gitignore:55`; `git ls-files videos/ dist/` empty.
- **Gate evidence:** PASS. `logs/checks/20261012T143115Z.log` shows `running 147 tests`, `147 passed; 0 failed`, `ALL CHECKS PASSED`, exit 0.

## Required Edits

### Fix 1 — Resolve `pending` commit hashes in report table

**File:** `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`  
**Location:** `Files Created/Changed` table, rows for the report itself and the task-5 plan file (around lines 45–46).

**Current text:**

```markdown
| `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (this file) | 5 | tracked completion report | `pending` (this commit) |
| `.kilo/plans/20261011-phase03-task5-verification-docs.md` | 5 | task-5 plan tracking | `pending` (plan-tracking commit) |
```

**Replace with:**

```markdown
| `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md` (this file) | 5 | tracked completion report | `82ab621` |
| `.kilo/plans/20261011-phase03-task5-verification-docs.md` | 5 | task-5 plan tracking | `c249138` |
```

Rationale: commit `82ab621` is the report commit and `c249138` is the plan-tracking commit; both are already in `main..HEAD` and listed in the Evidence Index commit list.

### Fix 2 — Correct `docs/demo.md` line count in Documentation Accuracy Audit

**File:** `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`  
**Location:** `Documentation Accuracy Audit` table, `docs/demo.md` row (around line 218).

**Current text:**

```markdown
| §6.2 `docs/demo.md`: all 8 planned sections + TOC + observed version/counts | holds (131 lines) |
```

**Replace with:**

```markdown
| §6.2 `docs/demo.md`: all 8 planned sections + TOC + observed version/counts | holds (87 lines) |
```

Rationale: `docs/demo.md` currently contains 87 lines (verified by direct read); 131 is an invented value.

## Verification After Fix

1. `git diff` should show only the two report lines changed.
2. Re-run the standard gate if any non-report file is touched; because only the report is edited, a docs-only self-check is sufficient.
3. Confirm the report no longer contains the string `pending` (except in historical or limitation contexts).
4. Confirm `docs/demo.md` line count reads 87 in the audit table.

## Notes

- No TODO edits, no source edits, no pushes, no merges are part of this fix plan.
- The junior implementer stayed within the 50% restriction: all production-behavior decisions were pre-locked; the only test-side deviation (ETXTBSY retry helper) is a minor local test robustness detail, and the two sanctioned deviation commits (`ec107ec`, `8d8e739`) are already documented.
