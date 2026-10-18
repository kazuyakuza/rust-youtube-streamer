# Phase 05.1 — Task 3 Simplification Plan (Addendum Factual Corrections)

Date: 2026-10-18 · Branch: `feat/phase05-1-operator-feedback` · Base HEAD: `7fd9b00`
Role: simplifier output for critical-workflow step 4.3 (Task 3 — report addendum + verification).

## 1. Verdict

The Task 3 diff (`git diff f783421..HEAD --stat`) is docs/plan-only — zero `src/**`, zero Cargo. No scope violation.
One verification defect found: **no simplification of structure or length is required**; the addendum is otherwise internally consistent and matches the code, docs, and evidence files. **Two contained factual wording corrections ARE required**, both in the same file:

- File: `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`
- No other file changes. The task-3 plan file (`20261014-phase05-1-task3-report-addendum-verification.md`) is a historical record and stays untouched.

## 2. Findings (evidence)

### F1 — Overreaching limitation-attribution claim (report line 372)

Addendum sentence claims the three cited limitations are "all already on the report's Known Limitations list **and** in `docs/production-run.md` §Shutdown Behavior". Fresh verification:

| Cited limitation | Report Known Limitations (a)–(h) | `docs/production-run.md` §Shutdown Behavior |
| --- | --- | --- |
| Unbounded FFmpeg exit wait | ✔ item (c) | ✔ line 97 |
| Absent ureq request-level timeout | ✔ item (d) | ✘ not there (line 97 names only Ctrl+C + FFmpeg wait; file has no `ureq` mention) |
| Repeated-Ctrl+C inertness | ✘ NOT an item (a)–(h) | ✔ line 97 |

The "and all already on the report's Known Limitations list and in ... §Shutdown Behavior" blanket claim is false for two of the three items. Verified fresh: `rg -n -i "timeout|ureq" docs/production-run.md` (only line 97 relevant), report Known Limitations section (items a–h, no Ctrl+C entry).

### F2 — "all exits 0" contradicts its own table (report line 395)

The evidence-table intro sentence says "all exits 0", but the table itself records 7 `git grep` rows at **exit 1** (empty-result rows), and the very next parenthetical explains those exit-1 rows. The claim contradicts its own table and parenthetical. Corrected wording must match the recorded expectation (task-3 plan §7 lists "0 hits" as the expected result for those rows).

### Non-defects (checked, no action)

- Ranked candidates' "today" descriptions match the code: `finish()` uses unbounded `child.wait()` (`src/streaming/ffmpeg.rs:107–114`, no timeout field anywhere in config); no request-level ureq timeout (`gate.rs:12`, `streaming.rs:8`, `google_live_api_transport.rs:9` doc notes); `enable_auto_stop: true` (`src/youtube/live_models/request_bodies.rs:52`, O2/D-T1-7); no watchdog/service-registration code exists.
- Quotes verified verbatim: brief §12.4 (lines 897, 905 — both quotes), `docs/production-run.md` line 104 crash quote, fate-policy rule-8 label in `src/app/run/mod.rs:7` (consistent with the report's own existing attribution).
- Evidence table re-verified fresh at HEAD: `allow(dead_code)` 8 attributes + 1 comment at the exact listed lines; `key=` 1 hit; all "0 hits" rows empty; `watch?v` exactly 2 hits (`broadcast.rs:31`, `mod_tests.rs:28`); `stream-key-fixture` 6 hits in the 3 named test files; println lines `mod.rs:55/59/70`.
- Gate numbers match `logs/checks/20261018T014255Z.log` (357 passed / 0 failed / 1.06 s; PASS timings 3/12/22/1 s). Artifact sizes match Task-2 adherence (5197144 / 8618273). Commit timestamps (Oct 17 22:50 −0300) match the Oct 18 UTC log name.
- `Cargo.toml` version 0.9.0 matches the structure-map line. TOC anchor resolves. D-GF6 exists in the global plan with the record-only framing.
- Hygiene/secret evidence-table duplication vs the report's Security Review section is mandated by TODO Task 3 ("run the repo hygiene/secret audit once more ... report exact commands, exit statuses") — not a redundancy defect.

## 3. Edits (execute exactly, in order; one file)

### Step 1 — fix the attribution sentence (F1)

Exact old text (single sentence, report line 372):

```text
Related documented limitations this reality sits beside (cited by name): the unbounded FFmpeg exit wait in `finish()` (report Known Limitations item (c)), the absent ureq request-level timeout (D-T2-16(a), report Known Limitations item (d)), and repeated-Ctrl+C inertness — all already on the report's Known Limitations list and in `docs/production-run.md` §Shutdown Behavior.
```

Exact replacement:

```text
Related documented limitations this reality sits beside (cited by name): the unbounded FFmpeg exit wait in `finish()` (report Known Limitations item (c) and `docs/production-run.md` §Shutdown Behavior), the absent ureq request-level timeout (D-T2-16(a), report Known Limitations item (d)), and repeated-Ctrl+C inertness (`docs/production-run.md` §Shutdown Behavior) — each already documented where cited; none is introduced by this addendum.
```

### Step 2 — fix the exit-status claim (F2)

Exact old text (report line 395, paragraph intro sentence):

```text
Ran on the final Task-3 tree: the standard dev-checks gate plus the D-T4-10 hygiene/secret audit set adapted to the current tree; all exits 0; nothing was implemented by this addendum, so code/artifact facts are unchanged from Task 2.
```

Exact replacement (fragment swap only; the following parenthetical sentence stays untouched):

```text
Ran on the final Task-3 tree: the standard dev-checks gate plus the D-T4-10 hygiene/secret audit set adapted to the current tree; every command ended with its expected exit status; nothing was implemented by this addendum, so code/artifact facts are unchanged from Task 2.
```

## 4. Constraints

- Touch ONLY `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`. Zero `src/**`, zero Cargo, zero plan/TODO/doc changes.
- Do not alter any table row, quote, number, or other addendum sentence.
- Do not change the task-3 plan file (historical record; its §7 expectations were correct).
- Commit is the caller's workflow step: suggested message `docs: correct the addendum limitation attributions and exit-status wording`.

## 5. Self-checks (run each as a single command from repo root)

1. `git diff --stat` → exactly one file changed: `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` (2 insertions, 2 deletions).
2. `rg -c "all already on the report's Known Limitations list and in" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md` → no match (exit 1).
3. `rg -c "all exits 0" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md` → no match (exit 1).
4. `rg -c "each already documented where cited" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md` → exactly 1.
5. `rg -c "every command ended with its expected exit status" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md` → exactly 1.
6. `rg -c "git grep" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md` → unchanged count vs before the edits (table untouched); if a mismatch is observed, PAUSE and escalate.
