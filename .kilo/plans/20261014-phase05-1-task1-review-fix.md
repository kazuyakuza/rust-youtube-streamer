# Phase 05.1 — Task 1 Review Fix Plan

- Review date: 2026-10-14
- Commit range reviewed: `cf98aee..b537acf` on `feat/phase05-1-operator-feedback`
- Files reviewed: `docs/google-cloud-prerequisites.md`, `docs/production-run.md`, `docs/authentication.md`, `.agent/project-info/context.md`
- Authority: `.agent/todos/20261014/20261014-todo-1.md` Task 1 + `.kilo/plans/20261014-phase05-1-task1-eligibility-multichannel-docs.md`

## Verdict

**One fix required** — Check 2 fails on a factual inaccuracy in `docs/authentication.md`. All other checks pass.

## Findings per requested check

1. **Plan copy-faithfulness — PASS.** Every edited hunk matches the copy-ready text in the Task-1 plan §5–§7 byte-for-byte (TOC lines, section bodies, table row, checklist item, See-Also bullets, context.md bullet, production-run.md insertions).
2. **Factual accuracy of the multi-channel bullet — FAIL.** `docs/authentication.md` new section "One Grant = One Channel Identity", 2nd bullet, states:

   > The authorization probe that `auth` relies on (`GET /youtube/v3/channels?part=id&mine=true`, see Unattended Operation) reads the authorized channel as an access check; it never prints or validates which channel it is.

   This is wrong on two independent grounds:
   - **Attribution:** `auth` does **not** rely on the network probe. `src/app/auth/flow.rs` validates the exchanged token response **offline** (`storeable_credentials` checks scope + refresh token only); the live probe belongs to the reusable credential service used on `run` startup.
   - **Payload handling:** The probe does **not** "read the authorized channel". `src/youtube/google_authorization_verifier.rs` maps HTTP status only; on `200` it calls `verified_scope_evidence`, whose comment explicitly says *"the payload itself is ignored"*. The probe is status-only scope evidence, not a channel read.

   > **Overlap with simplifier:** This finding independently confirms the simplifier plan's F-1 (`20261014-phase05-1-task1-simplify.md`). The fix below adopts the simplifier's exact S-1 replacement wording, since it corrects both attribution and payload-handling errors while preserving the bullet's true conclusion (which-channel is never surfaced/verified).

3. **Official citations — PASS.** URLs and dates match the Task-1 plan research table; no invented support-page dates; only R1 carries the "Last updated 2026-09-14 UTC" date.
4. **Operator-wait framing — PASS.** The 2026-10-14 operator fact is labeled "user-supplied — never agent-observed" in all three doc locations and in `context.md`; the manual E2E is stated as ON HOLD; no analytics/stream claims appear.
5. **Scope — PASS.** Exactly the four authorized files changed; no edits to `README.md`, log tables, `.agent/project-structure.md`, `.agent/project-info/tech.md`, or `src/`.
6. **Links/anchors — PASS.** New internal anchors (`#channel-live-streaming-access-youtube-side-requirement`, `#one-grant--one-channel-identity`) match their heading slugs; all relative links resolve within `docs/`.

## Required fix — F-1: authentication.md multi-channel probe wording

### File

`docs/authentication.md` ONLY.

### Exact replacement

Replace this single bullet line (the 2nd bullet under `## One Grant = One Channel Identity`):

```markdown
- The authorization probe that `auth` relies on (`GET /youtube/v3/channels?part=id&mine=true`, see Unattended Operation) reads the authorized channel as an access check; it never prints or validates which channel it is.
```

With:

```markdown
- The network probe (`GET /youtube/v3/channels?part=id&mine=true`) belongs to the reusable credential service's verify-first check — the non-interactive credential load every `run` performs (see Unattended Operation) — never to the interactive `auth` run, which validates the exchanged token response offline. An accepted probe response is treated purely as scope evidence: the response payload is ignored, so the probe never prints or validates which channel it is.
```

No heading, TOC, or other bullet changes.

## Self-checks after applying the fix

1. `git diff -- docs/authentication.md` shows exactly one changed line (1 insertion, 1 deletion) inside `## One Grant = One Channel Identity`.
2. The replacement text character-matches the block above (backticks, em dashes, and the parenthetical `GET /youtube/v3/channels?part=id&mine=true` intact).
3. The section still contains exactly five bullets; the heading and TOC entry `One Grant = One Channel Identity` are untouched.
4. `git diff cf98aee..HEAD --stat` continues to list ONLY the four Task-1 doc files — no code/Cargo changes introduced.
5. No literal `\n` escape sequences appear in the new line.
6. No credentials, tokens, stream keys, or RTMPS URLs are introduced by the replacement.
