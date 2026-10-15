# Phase 05.1 — Task 1 Code Simplification Plan (docs-only, step 4.3 simplifier side)

- Date: 2026-10-14
- Input diff: `cf98aee..b537acf` — exactly 4 doc files (`docs/google-cloud-prerequisites.md`, `docs/production-run.md`, `docs/authentication.md`, `.agent/project-info/context.md`), +46/−4. No code/Cargo changes (scope check PASSED).
- Task 1 plan: `.kilo/plans/20261014-phase05-1-task1-eligibility-multichannel-docs.md`
- Simplifier mandate: ONLY factual contradictions, duplicated stale claims, or misleading statements justify edits. Prose style/length is NOT a target.

## Findings — verified consistent (NO action)

- `run` failure strings quoted in the new prerequisites section match the code verbatim (`src/youtube/live_error/display.rs` lines 61/68: `live streaming is not enabled; see youtube.com/features`, `this channel cannot stream live now; see youtube.com/features`).
- `insufficientPermissions` attribution matches `src/youtube/live_error/mod.rs` / `classify.rs` (channel-eligibility failure mode); the quoted Google rule and "Last updated 2026-09-14 UTC" were spot-checked live against <https://developers.google.com/youtube/v3/live/authentication> — accurate.
- Multi-channel statements (one grant = one channel; app never selects/surfaces/verifies; refresh keeps the same grant; `auth` re-run replaces the persisted file) match the code: frozen 4-key persisted schema with no channel field (`src/youtube/persisted_credentials.rs`), payload-ignoring probe (`src/youtube/google_authorization_verifier.rs` `verified_scope_evidence`), atomic temp+rename overwrite (`src/app/auth/flow.rs`, recovery table).
- Probe URL quoted in `docs/authentication.md` matches `VERIFICATION_ENDPOINT` exactly; markdown anchors (`#one-grant--one-channel-identity`, `#channel-live-streaming-access-youtube-side-requirement`) resolve correctly; internal "item 1"/"step 4" cross-references are correct.
- The dated operator fact appears in three files (prerequisites, production-run, context.md) — each instance was explicitly required by TODO Task 1 bullets and all three are mutually consistent; not stale duplication.
- context.md Immediate Next Steps renumbering (1–5) is coherent; item 3 does not contradict the closure record in item 1 or the outstanding-evidence item 2.

## Finding F-1 — factual contradiction (the ONE correction)

`docs/authentication.md`, "One Grant = One Channel Identity", 2nd bullet, currently reads:

> The authorization probe that `auth` relies on (`GET /youtube/v3/channels?part=id&mine=true`, see Unattended Operation) reads the authorized channel as an access check; it never prints or validates which channel it is.

Both clauses are factually wrong per this document's own Overview and the code:

1. `auth` does NOT rely on the network probe. The interactive `auth` flow validates the exchanged token response OFFLINE (code: `src/app/auth/flow.rs` header, "validate the granted scope and refresh token OFFLINE (D9 — the network verifier stays Task 4)"; docs: this file's Overview line "the live network probe belongs to the reusable credential service, below"; `docs/google-cloud-prerequisites.md` requirement table attributes `channels.list` to the reusable credential service). The probe runs on the credential service's verify-first load — every `run` startup (`docs/production-run.md` startup step 3) and the Unattended Operation flow.
2. The probe does NOT "read the authorized channel". An HTTP 200 is accepted purely as scope evidence; the response payload is ignored outright (`google_authorization_verifier.rs`: "the payload itself is ignored").

The bullet's conclusion (which-channel is never surfaced/verified) stays true; only the attribution/behavior wording is wrong.

## Simplification step S-1 (single atomic edit — full specification for the implementer)

- File: `docs/authentication.md` ONLY. No other file, heading, TOC entry, or bullet changes.
- Replace this exact line (the whole 2nd bullet of "One Grant = One Channel Identity"):

  ```markdown
  - The authorization probe that `auth` relies on (`GET /youtube/v3/channels?part=id&mine=true`, see Unattended Operation) reads the authorized channel as an access check; it never prints or validates which channel it is.
  ```

- With this exact line:

  ```markdown
  - The network probe (`GET /youtube/v3/channels?part=id&mine=true`) belongs to the reusable credential service's verify-first check — the non-interactive credential load every `run` performs (see Unattended Operation) — never to the interactive `auth` run, which validates the exchanged token response offline. An accepted probe response is treated purely as scope evidence: the response payload is ignored, so the probe never prints or validates which channel it is.
  ```

- No rewording, no splitting into multiple bullets, no additional edits elsewhere.

## Self-checks after S-1

1. `git diff --stat` shows exactly one file changed (`docs/authentication.md`) with a 1-line replacement (1 insertion, 1 deletion).
2. The replacement text character-matches the block above (backticks and em dashes intact); the bullet sits inside the existing 5-bullet list and the section heading/TOC are untouched.
3. `docs/authentication.md` Overview sentence "(the live network probe belongs to the reusable credential service, below)" remains untouched and now agrees with the bullet.
4. Full-range check `git diff cf98aee..HEAD --stat` after the commit still lists ONLY the four docs files — no code/Cargo changes introduced.
5. Markdown link sanity: the two relative links already present in the neighboring bullets are unchanged; S-1 adds no links.
6. No secrets, tokens, stream keys, or RTMPS URLs in the line (it contains none).

## Explicitly NOT part of this plan (do not touch)

- Pre-existing loose wording in `docs/google-cloud-prerequisites.md` "Consequences for this project" bullet ("**`auth` and the startup probe are negligible** (~1 unit per authorization/verification)") predates this task's diff; it is outside the edited content and stays as-is (observation recorded for the caller only — no follow-up task created).
- Prose length/duplication of the operator fact across the three docs (TODO-mandated) — untouched.
- `README.md`, code files, TODO files, the completion report, and the other three doc files — untouched.
