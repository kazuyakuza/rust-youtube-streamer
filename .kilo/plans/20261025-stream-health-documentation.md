# Global Plan — Document the Stream-Health Warning as a Known MVP Limitation

Plan date: 2026-10-25 · Owner: Planner Agent (Critical Workflow)
TODO: `.agent/todos/20261025/20261025-todo-1.md` · Type: DOCS-ONLY (NO version bump — the executable is unchanged) · Front-end: NO

## Global Pre-Analysis

Facts (attributed honestly):

- AGENT-VERIFIED (from the operator-supplied v0.11.1 log `logs/run/20261025-200332.log`): confirmed-active broadcast; CBR video ~5993.65 kbit/s (x264 summary) / 6132.3 kbit/s (progress line); MP3 silent track ~124 kbit/s; mux overhead 0.17%; clean Ctrl+C stop (`exit_code=Some(0)`, run exit 0) — the v0.11.1 stop-path + health remedies all shipped and work.
- OPERATOR-OBSERVED: YouTube Studio's Stream Health tab (current stream) still flags "not receiving enough video to maintain smooth streaming"; the operator's Studio view shows no bitrate graph.
- DOCUMENTED ASSESSMENT (the recorded root cause): the MVP's pure-black 1080p content carries ~zero real entropy; the `nal-hrd=cbr` padding maintains the wire bitrate but is padding, not content; YouTube's health measurement registers the signal poverty regardless of the wire rate. For a static black screen the warning is spurious for playback (nothing moves, nothing to stutter). The warning is expected to persist until real content arrives (Phase 06 chat text adds real — though still modest — content bits).

Operator ruling (2026-10-25): ACCEPT and DOCUMENT (the invisible-dither Option A was offered and declined).

## Encoded Technical Direction

- `docs/production-run.md`: one clearly-titled known-limitation note in the operator diagnosis area — title, root cause (content entropy, with the filler-vs-content distinction), the verified wire facts (v0.11.1 log citation), the operator guidance (expect the warning during pure-black streaming; verify playback on the watch page; do NOT chase encoder/network causes for this specific flag while the content is pure black; it clears when real content arrives), and the honest attribution split (agent-verified wire facts / operator-observed health flag / documented assessment — never claim agent-observed Studio behavior).
- `.agent/project-info/context.md`: the dated record + Immediate Next Steps truthful refresh.
- `.agent/project-structure.md`: the production-run.md summary line updated ONLY if it enumerates contents.
- README.md: drift-check only (expected no edit).
- NO code/config/dependency/version change; `demo`/`auth` byte-untouched; verification = docs hygiene only (gitignore compliance, markdown/TOC consistency, secret scan) — no container gate needed for a docs-only change, but ONE final-tree dev-checks gate run is still recorded as the closing evidence per repo practice (docs never break it).

## Execution Map (steps 2–6)

- Step 2: Git Feature Branch Setup => implementer — branch `docs/stream-health-warning-note` from `main` (clean, at `f244dd1`, synced with origin)
- Step 3: Version Update => implementer — SKIPPED BY DECISION (docs-only; recorded here; the approval of this plan constitutes the user ruling)
- Task 1: 4.1b Analysis & Implementation Plan => architector
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Code Review & Simplification => code-reviewer + code-simplifier (docs-accuracy scope); 4.3-fix => implementer if plans require
- Task 1: 4.4 Documentation => docs-specialist (light pass: context fold + drift checks)
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion => implementer
- Step 5: TODO File Completion => implementer — `-DONE` rename, tmp hygiene, merge to `main`, push `origin` ONLY

## Risks and Mitigations

- Documentation drift risk: the note must not contradict the existing v0.11.1 remedy guidance (CBR/MP3 stay correct — they keep the CONNECTION healthy and the audio warning cleared; only the video-content flag remains, rooted in content entropy).
- Attribution risk: never blur agent-verified wire facts into claimed agent-observed Studio behavior; the split is explicit in the note's wording.
