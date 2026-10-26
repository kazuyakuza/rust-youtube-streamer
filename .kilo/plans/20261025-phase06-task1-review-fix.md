# Phase 06 Task 1 — Review Fix Plan (code-reviewer pass)

**Artifact reviewed:** `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` @ `c7250c1`
**Binding context:** `.kilo/plans/20261025-phase06-youtube-live-chat-integration.md`, TODO Task 1 in `.agent/todos/20261025/20261025-todo-2.md`
**Verdict:** Fixes required — only the plan file itself needs correction; no `src/`, docs, config, or Cargo changes.

## Verified (no fixes needed)

- Commit scope: `git show --stat c7250c1` changed only `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (+282 lines). No `src/`, `docs/`, config, script, or `.gitignore` edits. `git status --porcelain` is empty.
- Every TODO Task 1 bullet is covered by sections 0–2 of the plan.
- Official URLs S1–S4, S7, and S10 were fetched and are current; "Last updated 2026-10-15 UTC" matches the live pages. S10 discovery document confirms scopes and paths cited in the plan.
- Community sources: none relied on for decisions; S11 is explicitly noted as not fetched.
- Selected method (`liveChatMessages.list` polling) + rejected alternative (`streamList`) recorded in D-1 with grounded, official-cited rationale.
- Quota math in D-10 recomputed and correct:
  - 5 s interval: 3,600 units/hour, ~2.8 h to 10,000/day
  - 10 s interval: 1,800 units/hour, ~5.5 h
  - 13 s interval: ~1,385 units/hour, ~7.2 h
  - 20 s interval: 900 units/hour, ~11.1 h
- Buffering (bounded `sync_channel` capacity 64), ordering (API order oldest→newest), deduplication (512-id window), retry/backoff/cancellation, and typed error mapping are defined in D-5, D-6, D-7, and D-8.
- `MessageSource` seam decision (widen to `ChatSourceError`) and `demo` byte-stable effects are stated in D-8.

## Required fixes

### F-1 — Explicitly handle `403 liveChatEnded` as graceful in D-6

**Issue:** Section 1.6 and D-7 correctly identify `403 liveChatEnded` as mapping to a new `LiveApiError::ChatEnded` variant, and the lifecycle table says this results in **graceful** worker termination. However, D-6's poll-loop classification only lists three outcomes (2xx, transient, terminal) and the terminal branch enumerates `liveChatDisabled`, `liveChatNotFound`, `forbidden`, authorization, and quota errors — it does **not** state what to do with `LiveApiError::ChatEnded`. A Task 2/3 implementer could fall through to a default terminal path, violating the lifecycle decision.

**Fix in D-6:** Add an explicit classification branch before the terminal branch for `LiveApiError::ChatEnded`:

```text
- `LiveApiError::ChatEnded` (from `403 liveChatEnded`) → graceful terminal: worker exits cleanly
  without writing to the shared terminal cell; source yields `Ok(None)` forever. Do not treat as
  transient and do not treat as fatal.
```

**Fix in D-7:** Add a clarifying sentence after introducing `LiveApiError::ChatEnded`:

```text
This variant is consumed by the poll loop as a graceful end-of-chat signal (D-6), not as a fatal
retrieval error.
```

### F-2 — Correct poll event type note in section 1.5

**Issue:** The research note says "the REST doc's list item for polls is written `pollDetails`, the proto enum says `POLL_EVENT`; treat the canonical value as `pollEvent`". In the REST resource reference, the `snippet.type` value is `pollEvent`; `pollDetails` is the name of the details object, not the type string. The design conclusion (`pollEvent`) is correct, but the research record is misleading.

**Fix in section 1.5:** Replace the sentence with:

```text
The REST doc's `snippet.type` value for polls is `pollEvent` (the details object is named
`pollDetails`); the proto enum says `POLL_EVENT`. Treat the canonical value as `pollEvent` and match
on the documented REST string in code-with-fallback, D-4.
```

### F-3 — Typo in D-10

**Issue:** "logs_MESSAGE-budget note" appears to be a typo.

**Fix in D-10:** Replace with "logs/message-budget note" or rephrase for clarity, e.g.:

```text
Operator implications (encoded for Task-4 docs): a ONE-OFF operator session of a few hours fits
comfortably; a multi-hour unattended service can exhaust the $0 default — mid-stream exhaustion is
typed-fatal (exit 1 with the honest error), the remedy paths are Google's documented quota-extension
form (S8 link) or accepting shorter sessions. Note: readiness/confirmation polls (Phase-05, 1 unit
each) are negligible (≈ 72 units per failed startup window at 5 s over 180+120 s).
```

## Not in scope / no action

- No `src/`, `docs/`, config, Cargo, script, or `.gitignore` edits are required.
- No builds, tests, or commits by this review pass.
- Re-picking the retrieval method, adding dependencies, or expanding into Tasks 2–4 is out of scope.
