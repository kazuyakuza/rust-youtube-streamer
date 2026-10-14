# Phase 05 — Task 4 Review Fix Plan

- **Review task:** 4.3 (Code Review — reviewer side) for Task 4 of Phase 05.
- **Authority:** `.agent/todos/20261013/20261013-todo-1.md` (Task 4 + global Scope/Boundaries + Explicitly Out of Scope + Acceptance Criteria) and `.kilo/plans/20261013-phase05-task4-verification-docs.md`.
- **Branch:** `feat/phase05-production-live-runtime` (HEAD `1c22ae4`).
- **Review base:** `1043148`.
- **Outcome:** Two documentation/report findings require correction; no code changes, no acceptance-criteria checkbox flips, no merge/push.

---

## Overall Assessment

Task 4 is substantially adherent:

- Diff surface is exactly the D-T4-12 file list; zero `src/**`, Cargo, script, config-example, `.gitignore`, or Docker changes.
- All 13 drift lines from the Task-4 plan §1.1 are corrected.
- `docs/production-run.md` matches D-T4-2/D-T4-8: 10 sections in order, startup sequence correct, shutdown/resource-fate policy accurate, manual verification procedure present and operator-run, no chat claims, no native-runtime claims.
- The completion report contains the required 18 sections, observed-only evidence, honest performed-vs-NOT-performed ledger, correct E2/E3 transcriptions, chat criteria marked ⛔, and no secrets.
- Metadata (`project-structure.md`, `tech.md`, `context.md`) matches the actual tree.
- Verified independently:
  - `git status --porcelain` clean.
  - Gate log `logs/checks/20261014T213126Z.log`: `355 passed; 0 failed; 0 ignored; ALL CHECKS PASSED`.
  - Second gate log `logs/checks/20261014T220133Z.log`: same 355/0/0 result.
  - `ls -l` cross-checks: Linux `5197208` bytes, Windows `8613503` bytes.
  - D-T4-10 audit greps match report: `allow(dead_code)` = 8 attribute lines + 1 comment mention; `allow(unused_imports)` = 0; `liveChat` = 0; `transition(`/`liveBroadcasts.delete`/`liveStreams.delete` = 0; `key=` = 1 doc-comment; `NOT_IMPLEMENTED_EXIT`/`ya29`/`AIza` = 0; `client_secret`/`refresh_token` hits are schema/type paths and test fixtures only.
  - Tree counts: `src/app/run/` = 17, `src/streaming/` = 19, `src/youtube/` = 62, `src/app/pipeline/` = 14.

Two findings require a docs-specialist correction pass (4.4):

1. **Factual quota inconsistency** in `docs/google-cloud-prerequisites.md` (the lifecycle row says 1 unit, but Task-1/Task-2 research and the report state insert/bind = 50 units, list ≈ 1 unit).
2. **Report commit-count arithmetic** in `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` ("59 commits" does not match the listed 61 commits).

A third item is the already-recorded plan-wording slip for `source_helpers.rs`; this fix plan also prescribes the exact correction in the Task-4 plan so the historical document does not keep the misleading parenthetical.

---

## Finding 1 — Quota Table Factual Inconsistency

### File
`docs/google-cloud-prerequisites.md`

### Current text (lines 101–117)
```markdown
| Method | Units per call |
| --- | --- |
| `channels.list` (authorization probe) | 1 |
| `liveBroadcasts.*` / `liveStreams.*` (lifecycle) | 1 |
| `liveChatMessages.list` / `liveChatMessages.streamList` | 5 |
| `liveChatMessages.insert` | 20 |
| `liveChatBans.*` / `liveChatModerators.*` | 200 |
| `search.list` | 100-calls/day bucket |
| `videos.insert` | 100-calls/day bucket |

Consequences for this project:

- **`auth` and the startup probe are negligible** (~1 unit per authorization/verification).
- **Broadcast/stream lifecycle is cheap** (~1 unit per lifecycle call).
- **Live-chat reading is the quota driver.** `liveChatMessages.list` costs 5 units, so a continuous poll every 5 seconds would consume ≈17,000 units/day — over the default ceiling. The future chat phase must therefore pace polling within 10,000 units/day (≈2,000 list calls/day, i.e. no faster than roughly one call per 43 seconds) or use `streamList` (also 5 units per call, but far fewer calls), or request a quota extension.
```

### Problem
The row `liveBroadcasts.*` / `liveStreams.*` (lifecycle) | 1` and the sentence "Broadcast/stream lifecycle is cheap (~1 unit per lifecycle call)" contradict the Task-1/Task-2 research record and the completion report, which state that `liveBroadcasts.insert`, `liveStreams.insert`, and `liveBroadcasts.bind` cost approximately **50 units** each, while `liveBroadcasts.list` / `liveStreams.list` cost approximately **1 unit** each.

### Replacement text
```markdown
| Method | Units per call |
| --- | --- |
| `channels.list` (authorization probe) | 1 |
| `liveBroadcasts.insert` / `liveStreams.insert` / `liveBroadcasts.bind` | 50 |
| `liveBroadcasts.list` / `liveStreams.list` (status/readiness polling) | 1 |
| `liveChatMessages.list` / `liveChatMessages.streamList` | 5 |
| `liveChatMessages.insert` | 20 |
| `liveChatBans.*` / `liveChatModerators.*` | 200 |
| `search.list` | 100-calls/day bucket |
| `videos.insert` | 100-calls/day bucket |

Consequences for this project:

- **`auth` and the startup probe are negligible** (~1 unit per authorization/verification).
- **Broadcast/stream setup is moderate-cost** (~50 units per insert/bind call), while status/list polling is cheap (~1 unit per list call). One successful `run` startup therefore consumes roughly 152 units for the mandatory insert/bind/list calls (plus ~1 unit per poll attempt), well inside the daily budget for normal use.
- **Live-chat reading is the quota driver.** `liveChatMessages.list` costs 5 units, so a continuous poll every 5 seconds would consume ≈17,000 units/day — over the default ceiling. The future chat phase must therefore pace polling within 10,000 units/day (≈2,000 list calls/day, i.e. no faster than roughly one call per 43 seconds) or use `streamList` (also 5 units per call, but far fewer calls), or request a quota extension.
```

### Verification
- Read the edited table and consequence paragraph.
- Confirm it is consistent with:
  - `.kilo/plans/20261013-phase05-task1-official-api-research-design.md` §1 (lines 343–345: "insert/bind = 50 units each").
  - `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md` line 45: "Quota costs (insert/bind = 50 units, list ≈ 1 unit; $0/10,000 per day)".
  - `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` §Selected API Operations & Scopes: "Quota-cost line (Task-1 research record): insert/bind ≈ 50 units per call, list ≈ 1 unit per call".

---

## Finding 2 — Report Commit Count

### File
`.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`

### Current text (line 82)
> 59 commits from the version bump `acfabde` through the metadata commit `f03b999`; the full verbatim list is in [Evidence Index](#evidence-index).

### Problem
The Evidence Index list (lines 294–355) contains 61 commits (`f03b999` through `acfabde` inclusive). The per-task table in the same section sums to 60 commits excluding the version bump, or 61 including it. `git log --oneline acfabde..HEAD` returns 61 commits (excluding `acfabde`, including the report commit `1c22ae4`). Therefore "59" is factually incorrect.

### Replacement text
> 61 commits from the version bump `acfabde` through the metadata commit `f03b999`; the full verbatim list is in [Evidence Index](#evidence-index).

### Verification
- Run `git log --oneline acfabde..HEAD | wc -l` and confirm `61`.
- Count the lines in the Evidence Index code block (`f03b999` through `acfabde`) and confirm 61.
- Confirm the per-task table sums to 61 (Task 1 = 20, Task 2 = 21, Task 3 = 13, Task 4 = 6, version bump = 1).

---

## Finding 3 — Task-4 Plan Parenthetical Slip (Already Recorded as Deviation)

### File
`.kilo/plans/20261013-phase05-task4-verification-docs.md`

### Current text (D-T4-6 item 1, lines 252–253)
> Verify the `src/app/run/` 17-file claim, `src/streaming/` 19-file claim (incl. `spawn.rs`, `source_helpers.rs`), and the `src/youtube/` 62-file claim before writing.

### Problem
`source_helpers.rs` lives in `src/app/pipeline/`, not `src/streaming/`. The project-structure metadata is correct; this is only a plan-wording slip. The report already records this as Task-4-local deviation item 2, but the plan itself should not keep the misleading parenthetical.

### Replacement text
> Verify the `src/app/run/` 17-file claim, `src/streaming/` 19-file claim (incl. `spawn.rs`; `source_helpers.rs` is in `src/app/pipeline/`), and the `src/youtube/` 62-file claim before writing.

### Verification
- Run `find src/app/pipeline -type f | grep source_helpers.rs` and confirm the file is there.
- Run `find src/streaming -type f | grep source_helpers.rs` and confirm no match.

---

## Out of Scope / Not Required

- No `src/**` edits.
- No Cargo.toml/Cargo.lock edits.
- No script, config-example, `.gitignore`, Dockerfile, or compose edits.
- No acceptance-criteria checkbox flips and no TODO `[DONE]` marks (caller-owned 4.6).
- No merge, push, or version bump.
- No re-running of the gate/builds (the existing evidence is valid; the fixes are documentation-only and do not affect compiled artifacts or test counts).

---

## Implementation Order (4.4 docs-specialist)

1. Apply Finding 1 (`docs/google-cloud-prerequisites.md`).
2. Apply Finding 2 (`.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`).
3. Apply Finding 3 (`.kilo/plans/20261013-phase05-task4-verification-docs.md`).
4. Commit as a single docs-fix commit or as separate docs-fix commits (docs-only; either grouping is acceptable).
5. Run `git status --porcelain` and confirm only the intended files are modified.
6. Spot-read each edited file to confirm the new wording renders correctly and no literal `\n` sequences were introduced.
