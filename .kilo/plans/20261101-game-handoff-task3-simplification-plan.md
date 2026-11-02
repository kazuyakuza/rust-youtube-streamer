# Task 3 Simplification Plan — Game-Foundation Handoff (step 4.3, docs-only)

Inputs reviewed: `docs/game-foundation.md` (working tree, 193 lines), all seven cross-link diffs vs HEAD `27586ba`, `docs/latency-test-map.md`, `docs/production-run.md` (collision scan), binding plan `20261101-game-handoff-task3-plan.md` (§2.3–§2.6, §3, §5, §9 coverage floor).
Executor: docs-specialist (4.4 alignment pass). The junior implementer only commits at 4.6. No edits are made by this plan's author.

Verdict: the guide is already plan-faithful and near-final. TWO word-level edits are approved (§1). Everything else considered is rejected with reasons (§3) — mostly because the binding plan §2.4 fixes the exact wording/structure of the candidate passages, or because each restatement serves a distinct reader and the coverage floor (§9 traceability, §3 ledger) requires it.

## 0. Line budget and coverage

- Guide: 193 lines, cap 230 — PASS; edits below are line-neutral (intra-line inserts).
- Coverage floor: no TODO-mandated item is added, removed, or folded. 10 content H2 / 17 H3 / 8-bullet checklist / 10-item frozen list / 10-item guidance all intact.
- The 7 cross-link files: each one-liner matches the plan §2.6/§3 prescribed text and placement EXACTLY (README AI-Agents position after the `architecture.md` bullet; three See-Also rows appended; architecture three deltas; tech single bullet; context Task-3 block). NO drift, nothing to change there.

## 1. Approved edits (docs-specialist, `docs/game-foundation.md` only)

### E1 — gloss the undefined "O6" tag in the Module Responsibility Map (L47)

`O6` is a code/design-record tag; the guide never expands it, but the code the reader opens uses it (`src/youtube/live_chat_resolution.rs:1` "O6 live-chat-id resolution"). Keep the tag (code-traceable), add the gloss.

- Exact before (unique in file): `Live Chat retrieval (the O6 lookup + the `streamList` seam family)`
- Exact after: `Live Chat retrieval (the O6 live-chat-id lookup + the `streamList` seam family)`

### E2 — gloss the undefined "D-13" tag in frozen list item 6 (L178)

Same class: `src/app/run/chat_worker.rs:18` comments "D-13 redaction: ...". Keep the tag, mirror the code comment verbatim.

- Exact before (unique in file): `worker D-13 (message text/authors never logged)`
- Exact after: `worker D-13 redaction (message text/authors never logged)`

Both edits deviate by insertion ONLY from the plan §2.4(C)/(I) verbatim wording; they add zero claims, change zero facts, and strictly improve self-documentation (self-documenting-code rule applied to prose). Flag both insertions to 4.5b as sanctioned-by-this-plan micro-deviations.

## 2. Post-edit verification (docs-specialist, extends D9)

1. `docs/game-foundation.md` line count stays 193; `git status --porcelain` still shows exactly the 8 expected paths.
2. Sweeps re-run, all must stay 0 matches in the guide: `ya29|AIza|key=|rtmps://`, literal `\n`, emoji ranges. (Pre-edit state already verified clean; the single `rtmps://` hit repo-wide is pre-existing `docs/production-run.md` line 214, not a Task-3 edit.)
3. TOC slugs re-verified (all 10 H2 anchors resolve; the in-guide `#what-not-to-change` link resolves). Unchanged by E1/E2 — confirm only.

## 3. Candidates considered and REJECTED (with governing reason)

- R1 — credential chain + scope-gate restated in both the `run` H3 (L30) and the 10-stage funnel (L85): both passages are plan-mandated (§2.4(B) verbatim key sentence; §2.4(C) funnel list), and they serve different readers (mode responsibility vs stage-order reference). A pointer would break one of the plan's verbatim key sentences. Keep as-is.
- R2 — one-per-frame / capacity-64 appears 4x (L45 table, L67, L137, L174): every occurrence is plan-mandated (§2.4(C) table + body, §2.4(F) Viewers, §2.4(I) 2) and each is the citation anchor for a ledger row (L16–L17, L28). Removing any weakens per-list completeness ("economize with pointers, not omissions" — the lists already ARE complete per reader).
- R3 — ~35 s caveat (L133): the guide states it EXACTLY once, TODO pp. 140–146 + global AC p. 166 mandate its presence in the guide, and the stage-by-stage detail is already delegated to `latency-test-map.md` by link. Compliant.
- R4 — double `latency-test-map.md` link inside The Latency Warning (L133, twice): each link anchors a DIFFERENT ledger citation (boundary map vs procedure/template, plan §2.4(F) sentences 1 and 3, rows L27). Dropping one removes a required claim→source pointer. Keep.
- R5 — single-sentence H3 "Resolution Does Not Change Timing Semantics" (fold candidate): the four Timing H3s are fixed by the plan §2.3 section tree and traceability row "pp. 140–146"; folding would re-litigate the binding plan. Keep (line budget allows).
- R6 — guidance item 3 vs frozen items 3/4/8 overlap: item 3's enumeration is plan §2.4(G) verbatim (TODO-list reword); the checklist's "→ see What Not to Change" pointer already exists where the plan placed it. No sanctioned room.
- R7 — module-table boundary column vs Independence Rule vs frozen item 6: three views of the redaction/boundary theme, all plan-mandated, each phrased for its reader (scan map / game-module rule / change-freeze). Not verbatim duplicates.
- R8 — Message Path diagram vs module table row-by-row: the diagrams carry per-stage responsibilities the table lacks; merge would lose diagram information (constraint forbids).
- R9 — "terminal cell" (L62) gloss: sits inside a plan-verbatim code-block diagram; glossing would re-write diagram inner structure beyond tightening, and the cell does keep flow. Note only for the reviewer, no edit.
- R10 — `latency-test-map.md` See Also links `production-run.md` twice (L140/L142): pre-existing HEAD content with distinct anchors; Task-3 edit was insertion-only (D4 boundary). Out of scope.

## 4. Observations for other lanes (NOT edits)

- For 4.5b/caller: plan §2.3 header says "H3 count = 15" but its own tree and the guide both have 17 H3s — plan-side miscount erratum; guide matches the tree.
- Accepted tightenings already present (no action): the two code-block diagrams use ASCII `->` where plan §2.4 shows `→` (internally consistent with the future-flow diagram); frame-path diagram omits `128k` from the libmp3lame line (bitrate detail stays code-side, pinned by frozen item 1).
- Factual suspects: NONE identified in the simplification pass; no correction is encoded here (reviewer's lane).

## 5. Return

2 approved intra-line edits (E1, E2); 10 rejected candidates; projected guide length 193 lines (≤ 230 cap, unchanged). Apply E1/E2 in the 4.4 pass; re-run §2.1–§2.3 verification; commit set remains exactly the 8 markdown paths at 4.6.
