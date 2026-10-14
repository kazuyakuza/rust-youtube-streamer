# Phase 05 — Task 4 Simplification Plan (Step 4.3, docs-only diff)

- Date: 2026-10-14. Branch `feat/phase05-production-live-runtime`; diff base `1043148`; HEAD `1c22ae4` (7 docs-only commits).
- Scope verified: `git diff 1043148..HEAD --stat` / `--name-status` show ONLY documentation, metadata, plan, and report files — **zero `src/**`, zero Cargo files, zero scripts, zero config examples, zero Docker/compose changes**. No scope violation; this plan is docs-only, and no code simplification is required (no source changed in Task 4).
- Review surface: `docs/production-run.md`, `README.md`, `docs/configuration.md`, `docs/authentication.md`, `docs/google-cloud-prerequisites.md`, `docs/build.md`, `docs/build-windows.md`, `docs/demo.md`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`, `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`.
- Verdict: **3 factual findings (F-1 medium, F-2 minor, F-3 low), 7 exact edits.** No prose-style changes. All other cross-doc claims (exit codes 0/1/2 + exit-3 retirement, wait constants 180 s/5 s and 120 s/5 s, ten startup stages, `enableAutoStart`/`enableAutoStop` fate policy, redaction guarantees, 355-test count, artifact sizes, Live-Chat Phase-06 ruling, `fonts/Exo2-Light.otf` OFL shipping claim) were cross-checked and are mutually consistent.

## Constraints (binding for the implementer — junior, 50% restriction)

- R1. Apply ONLY the edits F1-E1 … F3-E2 below, with the exact old/new strings. Every `OLD:` string must be found verbatim in the named file; if any is not found byte-exact, STOP and return to the caller — never improvise wording.
- R2. Touch ONLY these four files: `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`, `.agent/project-info/context.md`, `.agent/project-info/tech.md`, `docs/production-run.md`. Everything else is off-limits — explicitly OFF-limits: `src/**`, `Cargo.*`, `.gitignore`, `scripts/`, `config/`, Docker files, `docs/google-cloud-prerequisites.md` (its quota table is CORRECT and unchanged), README.md, docs/configuration.md, docs/authentication.md, docs/build.md, docs/build-windows.md, docs/demo.md, `.agent/project-structure.md`, and ALL `.kilo/plans/**` files (task-1/task-4 plans keep the ≈50-unit estimate as the historical research record — do not edit them).
- R3. Each replacement stays on the same single line as its OLD string (these are long one-line bullets; no rewrapping, no section moves, no renumbering, no other sentence changed).
- R4. No new files. No deletions of surrounding content. Real newline characters only.
- R5. Commit is caller-owned: do not commit unless the caller's step instruction says so; if instructed, stage only the four files in R2 and verify `git status` stages no `.gitignore`-matched path.
- R6. These findings were adjudicated by the simplifier against the live official source (see Evidence below); the implementer has NO authority to re-adjudicate or extend the fix.

## Evidence (basis for F-1)

Official YouTube Data API quota-cost table, `https://developers.google.com/youtube/v3/determine_quota_cost`, fetched 2026-10-14 (page footer: "Last updated 2026-10-08 UTC") lists every relevant method at **1 unit per call**: `liveBroadcasts` bind/delete/insert/insertCuepoint/list/transition/update = 1; `liveStreams` delete/insert/list/update = 1. `docs/google-cloud-prerequisites.md` (lines 106/116, "lifecycle = 1") AGREES with the official table. The "insert/bind = 50 units" figure in the Task-1 research record (transcribed into the completion report and context.md) CONTRADICTS both — the wrong surfaces are the 50-unit claims, not the prerequisites doc.

## F-1 (medium) — Quota-cost contradiction across the diff (report vs prerequisites doc vs official table)

The completion report's own Deviations item 3 flagged this tension for a planner decision; this plan is that contained correction, resolving it per the official table.

### Edit F1-E1 — `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`, line 46 (the "Quota-cost line" in §Selected API Operations & Scopes)

OLD (whole line):
Quota-cost line (Task-1 research record): insert/bind ≈ 50 units per call, list ≈ 1 unit per call; the default project quota is 10,000 units/day at $0.

NEW (whole line):
Quota-cost line: the official quota-cost table (`developers.google.com/youtube/v3/determine_quota_cost`, Last updated 2026-10-08 UTC) costs every `liveBroadcasts.*`/`liveStreams.*` method at 1 unit per call — matching the `docs/google-cloud-prerequisites.md` table; the Task-1 research record's ≈50-unit insert/bind estimate is superseded by the official table (see Deviations item 3). The default project quota is 10,000 units/day at $0.

### Edit F1-E2 — same file, Deviations item 3 (line 271): append the resolution sentence

OLD: this tension is flagged for a planner decision (possible docs-specialist correction pass) and is NOT reconciled unilaterally here.
NEW: this tension is flagged for a planner decision (possible docs-specialist correction pass) and is NOT reconciled unilaterally here. RESOLVED post-review: the official quota-cost table (Last updated 2026-10-08 UTC) costs every `liveBroadcasts.*`/`liveStreams.*` method at 1 unit per call, matching `docs/google-cloud-prerequisites.md`; the §Selected API Operations quota-cost line was corrected accordingly, and the Task-1 plan's ≈50-unit estimate stands only as the historical research record.

### Edit F1-E3 — `.agent/project-info/context.md`, line 143 (Task 1 bullet)

OLD: quota model insert/bind 50 units, list ≈ 1, $0/10,000/day
NEW: quota model — the record stated insert/bind 50 units, superseded by the official quota-cost table (Last updated 2026-10-08 UTC): every `liveBroadcasts.*`/`liveStreams.*` method costs 1 unit per call, matching `docs/google-cloud-prerequisites.md`; list ≈ 1; $0/10,000/day

## F-2 (minor) — "Cargo.toml/Cargo.lock unchanged" contradicts the report's own audited fact and self-contradicts in context.md

The report (§Dependencies) records: `git diff main..HEAD -- Cargo.toml` shows exactly one hunk — version `0.7.1` → `0.8.0` — no dependency line changed; `Cargo.lock` untouched. context.md line 142 says both "version 0.7.1 → 0.8.0 (`acfabde`)" AND "Cargo.toml/Cargo.lock unchanged since the Phase-04 bump" — internally contradictory. tech.md line 27 repeats "unchanged since the Phase-04 version bump". The true claim is: no dependency line changed, `Cargo.lock` untouched, the only `Cargo.toml` change is the version line.

### Edit F2-E1 — `.agent/project-info/context.md`, line 142

OLD: NO new dependencies (`Cargo.toml`/`Cargo.lock` unchanged since the Phase-04 bump)
NEW: NO new dependencies (the only `Cargo.toml` change in the phase is that version line; no dependency line changed and `Cargo.lock` is untouched)

### Edit F2-E2 — `.agent/project-info/tech.md`, line 27

OLD: NO new dependencies in this phase — `Cargo.toml`/`Cargo.lock` are unchanged since the Phase-04 version bump;
NEW: NO new dependencies in this phase — no `Cargo.toml` dependency line changed and `Cargo.lock` is untouched (the phase's only `Cargo.toml` change is the version bump to 0.8.0);

## F-3 (low) — `docs/production-run.md` internal "§7" cross-references are unresolvable by a reader (off-by-one against the rendered document)

The "§N" scheme used across the Task-4 artifacts counts the document Title as §1 (D-T4-2's 10-item list), so §7 = "Broadcast/Stream Fate" and §8 = "Manual Verification Procedure". The rendered document has no numbered headings and its unnumbered TOC lists 9 entries — a reader counting content sections lands §7 on "Manual Verification Procedure" (wrong target for the crash-recovery pointer). Name-based references remove the ambiguity without renumbering anything. Only the document's TWO self-references are fixed; the report/plan/context "§7/§8" references stay (they consistently follow the plan's declared D-T4-2 scheme).

### Edit F3-E1 — `docs/production-run.md`, line 95 (§Broadcast/Stream Fate, failure recovery pointers)

OLD: any still-live broadcast after a crash → this document, §7 (there is no separate streaming-recovery document — the recovery content lives here).
NEW: any still-live broadcast after a crash → this document's "Broadcast/Stream Fate" section above (there is no separate streaming-recovery document — the recovery content lives here).

### Edit F3-E2 — `docs/production-run.md`, line 109 (§Manual Verification Procedure, step 9)

OLD: use the failure/recovery tables (this document §7, [`docs/authentication.md`](authentication.md))
NEW: use the failure/recovery tables (this document's "Broadcast/Stream Fate" section, [`docs/authentication.md`](authentication.md))

## Verification checklist (run after the edits; all single commands, no chains)

1. `rg -c "insert/bind 50 units, list ≈ 1, \$0/10,000/day" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md .agent/project-info/context.md` → 0 matches (the old claim lines are gone from both in-diff surfaces).
2. `rg -n "unchanged since the Phase-04" .agent/project-info` → empty.
3. `rg -n "this document, §7|this document §7" docs/production-run.md` → empty.
4. `rg -n "1 unit per call" docs/google-cloud-prerequisites.md` → still matches (file untouched, F-1's premise preserved).
5. `rg -n "Last updated 2026-10-08 UTC" .agent/reports/20261013-phase-05-production-youtube-live-runtime.md .agent/project-info/context.md` → 2 files (the corrected claims present).
6. `git diff --name-only` → exactly the four R2 files (or clean tree if the caller's commit step already ran).

## Considered and deliberately NOT changed (anti-churn record)

- `docs/google-cloud-prerequisites.md` quota table and the "Broadcast/stream lifecycle is cheap (~1 unit)" bullet — correct per the official table; untouched (also honors D-T4-5 item 6's edit scope).
- Task-1 plan line 344 and Task-4 plan line 472 (historical research-record transcriptions) — plan artifacts are historical records, superseded via F1-E2/E3 wording instead of editing them.
- README exit-code line ("0 = ... a clean run stop") omitting the pre-streaming-stop variant — verbatim D-T4-3 item 4 locked wording; the full variant list lives in `docs/configuration.md` and `docs/production-run.md`; changing it would contradict the adherence chain.
- README's unchanged "the `demo` command launches the executable…" FFmpeg paragraph — still-true statement about `demo`; the README run bullet and Production Run section already state `run` launches FFmpeg too.
- The report's own "§7/§8" references to production-run.md — consistent with D-T4-2's declared numbering; renumbering the document would contradict the D-T4-2/D-T4-8 verbatim claims.
- `docs/demo.md` line-80 historical 147-test evidence line, `docs/authentication.md` §Verification Evidence text, and context.md per-phase historical sections — deliberately preserved historical evidence, already adjudicated PASS by the Task-4 docs audit.
- All prose style/length — explicitly out of simplification scope for docs.
