# Task 1 (4.1b) Implementation Plan — Document the Stream-Health Warning as a Known MVP Limitation

Plan date: 2026-10-25 · Owner: Architector (task 4.1b) → executed by IMPLEMENTER (4.2) · Type: DOCS-ONLY
Binding sources: `.agent/todos/20261025/20261025-todo-1.md` (Task 1 — the ONLY task) and the global plan `.kilo/plans/20261025-stream-health-documentation.md` (full fact set + attribution rules). Front-end: NO (no 4.1a). NO version bump (recorded skip, global plan step 3). NO code/config/dependency change of any kind.

## 0. Preconditions (verify first; stop-and-report if violated)

- [ ] Branch is `docs/stream-health-warning-note`, HEAD `d5a5aa7`, tree clean: `git status --porcelain` → empty, `git branch --show-current` → `docs/stream-health-warning-note`, `git rev-parse --short HEAD` → `d5a5aa7`.
- [ ] The citation log exists on the host working tree: `logs/run/20261025-200332.log` (`Test-Path logs/run/20261025-200332.log` → True). This file is GITIGNORED (`logs/*`, `.gitignore:18`-area) — it is never staged/committed; it is only the attribution citation target.
- If any precondition fails: STOP and report to the caller. Do NOT create branches (git/branch setup is not this step's duty), do NOT assume.

## 1. File A — `docs/production-run.md`: the known-limitation note

### 1.1 Anchor (exact)

Insert ONE new H2 section between the end of the v0.11.1 "Product-level status" block (the paragraph ending "...report the recorded results to the planner rather than re-judging in the field)." — currently the last line of `## Ingest Probe Procedure (Operator-Run)`) and the existing heading `## Manual Verification Procedure (Operator-Run)`. One blank line after the status block's final line, then the new section, then one blank line before the existing H2.

Rationale (why this anchor, do not move it): the new note is the CONTENT-rooted residual item; it sits directly after the v0.11.1 remedy note whose CBR/MP3 guidance it complements — CBR/MP3 keep the CONNECTION healthy (and the audio-bitrate warning cleared); the zero-entropy VIDEO content flag is what remains. Separating it as its own H2 keeps the probe machinery and the remedy status intact and un-contradicted.

### 1.2 TOC edit (exact)

In `## Table of Contents`, insert this bullet BETWEEN the `Ingest Probe Procedure` line and the `Manual Verification Procedure` line:

```
- [Known Stream-Health Limitation (Pure-Black MVP Content)](#known-stream-health-limitation-pure-black-mvp-content)
```

### 1.3 New section content (verbatim — implementer copies the structure; the wording below is the mandated guidance)

```markdown
## Known Stream-Health Limitation (Pure-Black MVP Content)

Operator verdict (2026-10-25): the fifth native Windows `run` (v0.11.1) is healthy on the wire,
while YouTube Studio's **Stream Health** tab still flags "YouTube is not receiving enough video to
maintain smooth streaming". The verdict is ACCEPT and DOCUMENT: for this MVP's pure-black content
the flag is expected, content-rooted, and spurious for playback — it is recorded here so operators
do not chase encoder/network causes for it.

- Root cause (documented technical assessment): a 1080p static black frame carries ~zero real
  entropy. The v0.11.1 true-CBR video block (`nal-hrd=cbr`) keeps the WIRE at the target bitrate
  (verified ~6.1 Mbit/s below), but the filler is padding, not content — YouTube's Stream-Health
  measurement still registers the content-level signal poverty.
- Honest attribution: the wire facts are AGENT-VERIFIED from the operator-supplied v0.11.1 log
  (`logs/run/20261025-200332.log`: x264 summary `kb/s:5993.65`, progress `bitrate=6132.3kbits/s`,
  MP3 silent track ~124 kbit/s, `exit_code=Some(0)`, run exit 0). The Stream-Health flag is
  OPERATOR-OBSERVED (Studio Stream Health tab; no bitrate graph shown in that view). The
  zero-entropy root cause is a DOCUMENTED TECHNICAL ASSESSMENT — it is NEVER claimed as
  agent-observed Studio behavior (the agent did not run the manual procedure and never will).
- Operator guidance while the MVP streams a static black screen:
  - EXPECT this warning for as long as the content is pure black; do not treat it as an encoder,
    network, or configuration fault.
  - It is spurious for playback: static content cannot visibly stutter. Verify smoothness on the
    watch page (`run` prints the non-secret watch URL) instead of chasing this specific flag.
  - The separate audio-bitrate warning ("audio bitrate (0) lower than the recommended 128 Kbps")
    was CLEARED by the v0.11.1 MP3-CBR remedy above.
  - This video-content warning is expected to persist until real content arrives (Phase 06 chat
    text adds real — though still modest — content bits).
- The v0.11.1 CBR/MP3 remedy note and the Ingest Probe Procedure above remain fully valid for
  CONNECTION-level issues (publish-succeeded-but-inactive/"No data" shapes and the probe
  discrimination path); this note covers only the content-level Stream-Health flag.
```

Copy rules: preserve exact newline characters (never literal `\n`), keep the bullet list shape, keep `logs/run/20261025-200332.log` inline as a plain path reference (no markdown link — the file is gitignored host data, and the doc's evidence conventions reference log paths as plain code spans).

### 1.4 No other `production-run.md` edits

- The v0.11.1 remedy note, the probe procedure, its commands, its outcome table, the log-lines table, the shutdown section and the failure lists are READ-ONLY for this task (zero edits). Only the two insertions in §1.1/§1.2 happen.

## 2. File B — `.agent/project-info/context.md`: the dated record

Under `## Current Work Focus`, insert ONE new top record block (the newest-record-first convention, mirroring the existing block shape):

```markdown
Stream-health known-limitation note (2026-10-25) is recorded on `main`-bound branch `docs/stream-health-warning-note` (DOCS-ONLY; no version bump — the executable is unchanged). Trigger: the operator's fifth native Windows `run` (`logs/run/20261025-200332.log`, v0.11.1, operator-supplied) is CONFIRMED healthy on the wire — `stream confirmed active on YouTube`, CBR video ~5993.65 kbit/s (x264) / 6132.3 kbits/s (progress), MP3 silent track ~124 kbit/s, mux overhead 0.17%, clean Ctrl+C stop (`exit_code=Some(0)`, run exit 0) — while Studio's Stream Health tab still flags "not receiving enough video to maintain smooth streaming". Operator ruling (2026-10-25): ACCEPT and DOCUMENT. Root cause (documented assessment, never agent-observed Studio behavior): the pure-black 1080p content carries ~zero real entropy; the v0.11.1 `nal-hrd=cbr` padding keeps the wire at target but is padding, not content. Documentation: `docs/production-run.md` gained the `## Known Stream-Health Limitation (Pure-Black MVP Content)` section (attribution split: agent-verified wire facts / operator-observed health flag / documented assessment; operator guidance — expect the warning during pure-black streaming, spurious for playback, verify on the watch page, audio warning cleared by the MP3-CBR remedy, video-content warning persists until Phase 06 real content; probe/v0.11.1 remedy remain valid for CONNECTION-level issues). Structure map's docs line refreshed truthfully; README drift-checked (no edit expected).
```

Insert it as the FIRST record in `## Current Work Focus`, above the "Stop-path fix + stream-health remedies (2026-10-25) are CLOSED..." block. Note: `context.md` has TWO name variants on disk (`context.md` is the one with the record blocks; the read returned the full record file). Confirm by reading the file's first lines before editing.

## 3. File C — `.agent/project-structure.md`: conditional docs-line edit

Line ~123's docs entry DOES enumerate `production-run.md` contents ("...the operator-run ingest probe procedure, the canonical manual verification procedure, out-of-scope list"). Conditional edit APPLIES: insert `the known stream-health limitation note,` between `the operator-run ingest probe procedure,` and `the canonical manual verification procedure,`. NO other structure-file change (no version label change — NO bump; no roster touched).

## 4. File D — `README.md`: drift-check (expected NO edit)

- Read README's production-run references (lines ~16 `#production-run-youtube-live` TOC anchor, ~61 run bullet, ~126 operator-guide pointer). The README does NOT enumerate production-run.md's section list and makes no Stream-Health claim — VERDICT: no drift, NO edit. If an edit turns out necessary, it must be minimal and flagged in the task report as a deviation with the drift quote.

## 5. Verification (docs hygiene only + ONE gate run)

1. Working-tree audit: `git status --porcelain` shows ONLY the four doc/metadata paths (docs/production-run.md, .agent/project-info/context.md, .agent/project-structure.md, README.md only if drifted) + the 4.4/step behavior applies (this 4.2's docs edits are committed per §6).
2. Secret scan on the diff: `git diff -- <paths>` contains zero stream keys, zero `rtmps://`, zero tokens (`ya29`/`AIza` counts THAT EXIST must equal HEAD's pre-existing doc-context counts); `logs/run/20261025-200332.log` appears only as a plain citation path and is never staged (`git check-ignore -v logs/run/20261025-200332.log` → `logs/*` rule hits).
3. Markdown hygiene in `docs/production-run.md`: TOC anchor `#known-stream-health-limitation-pure-black-mvp-content` matches the H2 slug; file uses real newlines (`rg -n '\\\\n' docs/production-run.md` → no hits); the v0.11.1 remedy/probe text is byte-identical (`git diff` touch count = insertions only).
4. ONE closing dev-checks gate run on the final tree (repo practice; docs never break it, still recorded as evidence): via the Alpine VM MCP — `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — REQUIRE exit 0, four PASS lines (fmt-check/check/test/clippy), `ALL CHECKS PASSED`, 391 passed / 0 failed (docs-only change ⇒ gate count unchanged); record the fresh `logs/checks/<UTC>.log` filename in the task's record.
5. NO builds of any kind — the artifacts are unchanged (docs only).

## 6. Commits (branch `docs/stream-health-warning-note` only; push NOT done here)

1. First: `chore: track stream-health task1 implementation plan` — stages ONLY `.kilo/plans/20261025-stream-health-task1-implementation.md` (the plan file this document — created by 4.1b, tracked by the implementer; the caller-authorized precedent: the 2026-10-24 record's plan-tracking commit `07f48c5`).
2. Then: `docs: document stream-health known limitation (pure-black MVP content)` — stages ONLY files A/B/C (+D if and only if §4 forced an edit). NO `\n`-literal/line-ending surprises: verify `git diff --stat` before commit.
3. Before each commit: read `.gitignore` and run `git status` — allow staging ONLY the intended tracked paths; NEVER stage `logs/`, `dist/`, `config/config.json`, `credentials/*`.
4. If either commit is rejected by tooling (crash claims, gitignore mismatch): STOP and report (failure path, per the failure-accounting rule).

## 7. Stop-and-report conditions (any of these → STOP, report to caller)

- Any pre-condition in §0 fails (wrong branch/HEAD, dirty tree, missing log file).
- The insertion surface (the v0.11.1 status block end / Manual Verification heading) is not found exactly as described (file drifted on this branch).
- The gate run is not 391/0 or fails on the docs tree.
- The `context.md` record-block convention differs materially from §2's mirror (e.g. a newer record block already exists for this task).

## 8. Alignment check (4.1b self-verify, this plan)

- TODO Task 1 acceptance criteria 1→§1, 2→§3/§4, 3→§5; out-of-scope lines honored (no content change, no troubleshooting beyond the note, no code/bump). Global-plan Encoded-Technical-Direction bullets: production-run note ✓§1, context refresh ✓§2, structure-map conditional ✓§3, README drift-check ✓§4, gate-as-closing-evidence ✓§5 item 4. Attribution split ✓§1.3 verbatim. No clash with the v0.11.1 guidance ✓ (note's cross-links bullet).
