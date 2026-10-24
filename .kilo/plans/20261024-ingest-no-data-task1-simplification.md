# Simplification Plan — Ingest "No data" Task 1 (docs, step 4.3 code-simplifier half)

Date: 2026-10-24 · Branch `fix/ingest-no-data` · Reviewed tree: HEAD `55ec596` (clean).
Reviewed scope (Task-1-changed docs ONLY): `docs/production-run.md` (TOC entry, new H2 `## Ingest Probe Procedure (Operator-Run)`, diagnosis step-2 pointer sentence) and `.agent/project-info/context.md` (new `## Recent Changes (2026-10-24, ingest "No data" — Task 1 research + probe procedure)` block).
Binding specs the docs were checked against: `.kilo/plans/20261024-ingest-no-data-task1-implementation.md` §2 (structure/commands/quotes/constraints), §3 (record spec), §5.1 (4.3 authority: "simplifier edits are prose-only, never command text"); global plan `.kilo/plans/20261024-ingest-no-data-audio-signal.md` (probe intent, environment/connection discrimination).

Verdict: exactly ONE actionable prose edit (S1). Everything else is either plan-locked content (untouchable by the 4.3 simplifier: the four mandated word-exact quotes, the four byte-exact command strings, the mandated intro facts, the mapping-table wording) or already verified clean. Two mapping-table accuracy issues are FLAGGED for the caller (F1/F2) — they are plan-level content, NOT executable simplifications; the implementer must not act on them without a caller/architector ruling.

---

## S1 — Fix the dangling "below" cross-reference in the context record (ONE edit, atomic)

File: `.agent/project-info/context.md`, line 197, last bullet of the 2026-10-24 Recent Changes block.

Problem: the record says `gitignore/hygiene checks (§4.1 below) CLEAN`. This phrase was copied verbatim from the implementation plan's §3.2 skeleton, where "§4.1 below" meant the PLAN's §4.1. Inside `context.md` the reference dangles — `context.md` has no §4.1 and nothing relevant is "below" in that file. The block already names the per-task plan path two bullets above ("in the per-task plan §1", line 189/194 pattern; full path at line 188), so the qualified form matches the file's own convention.

Edit (exact; use the structured edit tool, nothing else in the line changes):

- Find (unique in the file — verified): `gitignore/hygiene checks (§4.1 below) CLEAN;`
- Replace with: `gitignore/hygiene checks (§4.1 of the per-task plan) CLEAN;`

Constraints: do NOT touch anything else on the line or in the block; no src/config/plan-file edits; do NOT alter the gate-log path, counts, or commit hashes (all independently re-verified TRUE — see checklist); commit timing is the caller's decision (this tree's 4.3 precedents committed the fix, e.g. `b76a0c8`/`051a787` — do not commit unless the caller says so).

## F1 — FLAGGED, not executable: mapping-table row 3 cause label (plan-level content)

`docs/production-run.md` line 184, outcome-table row 3: "probe iii (i and ii inactive) ⇒ **environment-level** | encoder/network/firewall/build capability: no app change warranted". This wording is byte-faithful to the BINDING plan §2.3 table, but it inverts the global plan's model: global-plan line 23/49 assign the environment/connection bucket to probe iii FAILING ("if even (iii) fails: connection/URL-level"; "If the cause turns out environment-level (probe iii fails)"). If probe iii SUCCEEDS while i and ii fail, the environment demonstrably works (a push landed and locked) — the observation points at the signal side (Google's documented "audio AND movement" tested-signal expectation = both C1+C2 jointly), not "no app change". Any wording change here is a plan-content change: route to the caller/architector (4.5b or a plan amendment), then to the implementer. DO NOT edit under this plan. The same "only iii ⇒ environment" wording is mirrored in `context.md` line 194 and would inherit the ruling.

## F2 — FLAGGED, not executable: "each probe's result maps 1:1" is overstated

`docs/production-run.md` line 178 header: "(each probe's result maps 1:1)". The table covers 4 outcome patterns; combinations like "ii AND iii active, i inactive" — which is exactly the expected pure-C1 observation under the plan's own research (rich+audio locks too, since it adds motion to a locking vector) — match no row literally (row 2 says "probe ii only"). Plan-mandated wording again (same skeleton); flag for the caller/architector adjudication together with F1. DO NOT edit under this plan.

## NOT actionable / NO-change checklist (what was verified)

1. Redundancy vs existing `production-run.md` sections: cleanup bullet REFERENCES the fate policy instead of restating it (no duplication); the section's secret-safety rules are the operator-shell angle (distinct from the app log-guarantees paragraph and the manual-procedure step-2 reminder); "never as build evidence" and the OPERATOR-RUN facts are plan-mandated intro content — nothing removable without violating the binding skeleton. 60 s / `-t 60` mentions are each in distinct operative contexts (secret-safety kill-free guarantee, per-probe bound, observation window).
2. Markdown/TOC hygiene: TOC entry position (after Broadcast/Stream Fate, before Manual Verification) matches document order; heading `## Ingest Probe Procedure (Operator-Run)` ↔ anchor `#ingest-probe-procedure-operator-run` verified; H2-only + bold lead-ins + bullet style matches the file's established structure; table well-formed; section 63 lines, inside the plan's ~60–75 budget; ```bash fences per the plan skeleton.
3. Command readability: all four commands are byte-identical to the binding plan §2.3 (verified by line-exact comparison of docs lines 150/155/163/171 against plan lines 192/199/209/221) and MUST stay single-line (cmd.exe paste target named in the doc itself; `^`-continuations would break copy-paste, `\`-continuations would mislead PowerShell users) — simplifier has zero authority over command text per plan §5.1.
4. Wording accuracy of the four mandated quotes: all present word-exact (`noAudioStream` sentence; `STREAM_URL/STREAM_NAME`; "audio and movement in the video"; "Finish encoding when the shortest output stream ends."); only plan-allowed URLs cited; the "Options mirror the application's streaming vector exactly" claim checks against the documented startup vector (libx264/veryfast/6000k/GOP 60 = 2 s at 30 fps/yuv420p/flv) — accurate. Remaining overstatements are F1/F2 (plan-level).
5. Secrets: no `rtmps://<host>`, no 32+ char alnum strings, no key-shaped examples in either changed file (only the bracketed placeholders and the pre-existing doc-context audit tokens).
6. Record truthfulness of the context block: gate log `logs/checks/20261024T170918Z.log` exists and shows `running 378 tests` / `378 passed; 0 failed` / `ALL CHECKS PASSED`; README "378 tests" claim matches; commits `07f48c5`/`983293e` and the docs diff contents verified as described; `git status` clean. Only defect found: S1's dangling reference.
