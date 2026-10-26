# Task 1 (4.3) Simplification Plan — Stream-Health Known-Limitation Docs

Plan date: 2026-10-25 · Owner: Code-Simplifier (4.3 docs-accuracy half) → executed by IMPLEMENTER (4.3-fix, junior, 50% restriction — execute EXACTLY, no judgment calls) · Type: DOCS-ONLY fix (one metadata edit)
Binding sources: `.kilo/plans/20261025-stream-health-task1-implementation.md` (Task-1 plan — its §1.3 production-run text is VERBATIM-mandated and was applied byte-identically: NO edit there) + global plan `.kilo/plans/20261025-stream-health-documentation.md` (line 19 requires the context.md "Immediate Next Steps truthful refresh" — the per-task plan §2 omitted it; this plan closes that omission).
Branch/HEAD at review: `docs/stream-health-warning-note` @ `4547d23`, tree clean.

## 1. Review verdicts (what was checked)

- `docs/production-run.md` (286 lines, the Task-1 insertions): TOC anchor `#known-stream-health-limitation-pure-black-mvp-content` matches the H2 slug; section placement matches binding plan §1.1/§1.2; the new section is verbatim vs plan §1.3; attribution split complete (agent-verified wire facts / operator-observed flag / documented assessment); every operator-guidance bullet present; "no bitrate graph shown in that view" traced to the operator-supplied fact (TODO §Scope line "no graph shown in the operator's view"); the quoted warning wording traces to the TODO's operator report; no literal `\n`; diff insertions-only for the file (remedy/probe text byte-identical). VERDICT: NO EDIT (see §3 flags F1–F3).
- `.agent/project-structure.md` line 123: docs enumeration lists the note in true section order between probe procedure and manual verification. VERDICT: NO EDIT.
- `.agent/project-info/context.md`: the new dated record block (line 7) is truthful and complete; README drift verdict re-confirmed (no health-warning claim anywhere; line 126's section summary was never exhaustive — same precedent as the probe section). BUT: `## Immediate Next Steps` item 1 (line 302) still claims "both Studio health warnings (low video bitrate; audio bitrate 0) should clear (CBR video + MP3-CBR silence)" — this operator-facing expectation is contradicted by the very work this branch documents (audio warning cleared; video-content warning expected to PERSIST until Phase 06). This is the one accuracy defect. VERDICT: FIX S1 below.
- Secret/hygiene: `git diff HEAD~1 HEAD` → zero `ya29`/`AIza`/`rtmps://` matches (count 0); the run log appears only as a plain citation path.

## 2. S1 — the single required edit (atomic, fully specified)

File: `.agent/project-info/context.md` — ONLY line 302 (Immediate Next Steps item 1). One-line in-line replacement; the item stays a single line; no other line, no other section, no other file is touched.

FIND (exactly, must match exactly once — uniqueness pre-verified 2026-10-25, this is the only `should clear` hit in the file):

`a Ctrl+C stop should now exit 0 with the stop message, and both Studio health warnings (low video bitrate; audio bitrate 0) should clear (CBR video + MP3-CBR silence); the real-Studio confirmation is the operator's E2E observation;`

REPLACE WITH (exactly):

`a Ctrl+C stop should now exit 0 with the stop message; the operator's fifth v0.11.1 run reported the audio-bitrate warning cleared by the MP3-CBR remedy while the video-content Stream-Health warning persists — documented as a known limitation of the pure-black MVP content, expected to remain until Phase 06 real content arrives (`docs/production-run.md`, "Known Stream-Health Limitation (Pure-Black MVP Content)"); the real-Studio confirmation is the operator's E2E observation;`

Wording invariants (do not deviate): "the operator's fifth v0.11.1 run reported" keeps the Studio observation attributed to the operator (never agent-observed); "expected to remain until Phase 06 real content arrives" mirrors the docs bullet; the surviving clauses (exit-0 stop, real-Studio E2E, libmp3lame prerequisite, overflow-hardening TODO) stay byte-identical; the dated CLOSED record blocks and `## Recent Changes` are HISTORY — never edit them.

## 3. NO-action adjudications (recorded; implementer does NOT touch these)

- F1: intro clause "it is recorded here so operators do not chase encoder/network causes" vs guidance bullet 1 "do not treat it as an encoder, network, or configuration fault" — mild intra-section overlap, both verbatim-mandated by binding plan §1.3; removal buys only cosmetics and creates an adherence deviation. LEAVE.
- F2: "(verified ~6.1 Mbit/s below)" — "below" means "in the attribution bullet below"; figures truthful vs the log (5993.65 kb/s x264 / 6132.3 kbits/s progress). Binding wording. LEAVE.
- F3: warning quote "YouTube is not receiving enough video…" (§1.3, TODO operator-report verbatim) vs the older remedy-note quote without the "YouTube is" prefix — each faithful to its own source; the remedy note is READ-ONLY for this task (binding plan §1.4). LEAVE.
- F4: the 2026-10-25 stop-path record block's historical "expect both Studio health warnings to clear" line — dated CLOSED record; supersession lives in the newest record + refreshed next steps. LEAVE.
- F5: PRE-EXISTING markdown wart (NOT Task-1-caused, NOT edited): Immediate Next Steps has two items numbered `2.` (context.md lines 303–304); renderers renumber automatically. Reported to the caller for a future decision only.

## 4. Verification (after S1; all on the Windows host, single commands)

1. `Select-String -Path .agent\project-info\context.md -Pattern 'should clear'` → ZERO hits.
2. `git status --porcelain` → exactly the intended paths (context.md modified; the plan file untracked until §5 commit 1); `docs/production-run.md` and `.agent/project-structure.md` show NO change.
3. `git diff -- .agent/project-info/context.md` → exactly 1 changed line, and zero `ya29`/`AIza`/`rtmps://` matches in the diff.
4. NO gate run, NO builds (metadata docs edit only; nothing code-adjacent moves — the 4.2 closing gate stands).

## 5. Commits (branch `docs/stream-health-warning-note` only; push NOT done here)

1. Stage ONLY `.kilo/plans/20261025-stream-health-task1-simplification.md` → `chore: track stream-health task1 simplification plan` (precedent: `ed3e8ca`/`07f48c5`).
2. Stage ONLY `.agent/project-info/context.md` → `docs: refresh the stream-health operator expectation in the context next steps`.
3. Gitignore compliance before each commit: read `.gitignore`, run `git status`, never stage `logs/`, `dist/`, `config/config.json`, `credentials/*`.

## 6. Stop-and-report (any → STOP, report to caller; do not improvise)

- The FIND string does not match EXACTLY once in context.md.
- Working tree not clean at branch `docs/stream-health-warning-note` HEAD `4547d23` (or its 4.3-plan-tracking successor) before starting.
- Any pressure to edit `docs/production-run.md`'s §1.3 verbatim text, the remedy note, the probe procedure, the historical record blocks, `src/`, `config/`, or any plan file.
