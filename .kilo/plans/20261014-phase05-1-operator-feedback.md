# Phase 05.1 — Post-Closure Operator Feedback — Global Plan

- Date: 2026-10-14
- TODO file: `.agent/todos/20261014/20261014-todo-1.md` (Pattern C: 3 tasks under `## Tasks`)
- Status: DRAFT — awaiting user approval
- Base commit: `main` @ `2f15334` (Phase 05 closed, pushed; tree clean)
- Base version: 0.8.0
- Planned feature branch: `feat/phase05-1-operator-feedback`
- Planned version bump: 0.8.0 → 0.9.0 (semver minor — the watch-link print is a new observable feature)

---

## 1. Global Pre-Analysis

### 1.1 Current state (verified 2026-10-14)

- Phase 05 closed: `run` production path merged (`31f6b6e`), closure gate green on `main`
  (355/355, `logs/checks/20261014T225722Z.log`); context.md carries the closure record.
- The Phase-05 completion report exists at `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`
  (18 sections + AC matrix; to be APPENDED, never rewritten).
- User-reported facts feed tasks 1–3: channel needs live-stream access enablement with a ~24 h
  activation wait (real manual test ON HOLD); multi-channel account context.
- Related docs surfaces: `docs/google-cloud-prerequisites.md` (has the quota table + a troubleshooting
  table), `docs/production-run.md` (prerequisites + log table + manual procedure §8),
  `docs/authentication.md` (auth flow + recovery), context.md Immediate Next Steps.
- Code facts for Task 2: `broadcast_id` is already carried from `PreparedBroadcast` and printed in
  the `broadcast prepared for streaming` line; the confirmation-gate success point currently emits
  `stream confirmed active on YouTube` (production-run log table row 4); all event lines are
  `tracing` on stderr; the `WatchLink`-style print must NEVER touch `RtmpsDestination`/stream key
  surfaces.

### 1.2 Task extraction (Pattern C)

1. **Operator Documentation Updates** — eligibility/24 h docs (prerequisites doc + production-run
   note + context operator-fact line) + multi-channel identity docs (authentication doc +
   production-run cross-note).
2. **Console Watch-Link Print** — the one-line public watch URL after the confirmation gate;
   pinned tests; docs log-table/README touch; artifacts rebuilt.
3. **Completion Report Resilience Addendum + Final Verification** — dated future-planning section
   in the Phase 05 report; final gate + hygiene/secret audit; exact evidence.

### 1.3 Front-end determination

No task is front-end related. Sub-steps 4.1a/4.5a are omitted for every task.

---

## 2. Global Technical & Architecture Decisions

- **D-GF1 — Non-secret URL only.** The watch URL `https://www.youtube.com/watch?v=<broadcast_id>`
  is derived from the already-printed non-secret `broadcast_id`; the stream key/RTMPS URL redaction
  guarantees are untouched. Exact line text and print point are Task-2 plan decisions (pinned by
  tests).
- **D-GF2 — Docs-only surfaces for tasks 1/3.** Existing files only; Phase 05 report is appended
  with a dated addendum (never rewritten); plan artifacts stay historical.
- **D-GF3 — Eligibility content is evidence-backed.** Enablement/eligibility details must cite
  current official Google sources (researched fresh in Task 1's 4.1b); the ~24 h wait is recorded
  as the operator's user-supplied report, distinguished from agent-observed facts.
- **D-GF4 — Multi-channel framing.** One channel per grant; the app neither selects, surfaces, nor
  verifies the channel; `auth` re-run switches; Studio verification is the operator check. No code
  change in Task 1.
- **D-GF5 — Version/branch handling.** Branch creation restricted to workflow step 2; version bump
  (0.9.0) restricted to step 3; push restricted to step 5; task 2's artifact rebuilds happen inside
  that task's cycle.
- **D-GF6 — Resilience addendum is record-only.** Facts + ranked hardening candidates
  (bounded FFmpeg exit wait, ureq request timeout, hang watchdog, external supervisor) + the
  cannot-clean-up-after-own-crash reality; no implementation claims; explicitly future-work input.

---

## 3. Workflow Mapping (Critical Workflow steps 2–6)

| Step | Sub-agent | Notes |
|---|---|---|
| 2. Git Feature Branch Setup | implementer | commit pending changes (TODO/plan files); ensure `main`; create `feat/phase05-1-operator-feedback`; switch |
| 3. Version Update | implementer | 0.8.0 → 0.9.0; lockfile refresh via container; `chore: bump version to 0.9.0` |
| Task 1: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | architector / implementer / reviewers / docs-specialist | docs-only task |
| Task 2: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | same cycle | small feature; artifacts rebuilt in-cycle |
| Task 3: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | same cycle | report addendum + final gate/audit |
| 5. TODO File Completion | implementer | rename `-DONE`; tmp cleanup; merge to `main`; push `origin` ONLY |
| 6. Finish | planner | summary + handoff text |

Standard sub-agent preamble, context passing, and failure-escalation rules apply to every
invocation. The global plan is never overwritten; per-task plans land under
`.kilo/plans/20261014-phase05-1-*`.

---

## 4. Per-Task Pre-Analysis

### Task 1 — Operator Documentation Updates

- Research-first: fetch current official Google sources for live-streaming enablement/eligibility
  (phone verification, activation window, Studio features location) and cite them; the content must
  not contradict the existing failure tables in production-run.md (which already point at
  `youtube.com/features`).
- Multi-channel section: one grant = one channel identity; consent-time selection; app-agnostic
  behavior; `auth` re-run to switch; Studio verification during real tests. Place the full text in
  `docs/authentication.md`; cross-note from production-run.md.
- context.md: single dated operator-fact line in Immediate Next Steps (never agent-observed claims).

### Task 2 — Console Watch-Link Print

- 4.1b decides: exact printable line text; the event's stage/point (confirmation-gate success);
  which module emits it (the run orchestration emits events today — plan must name seam and keep
  stdout reserved for the final stop line); test strategy (pinned-line test consistent with the
  console/pinned-line precedents); docs table rows to update.
- Boundaries: redaction guarantees untouched; no `Debug`/`Display` on secret wrappers; no exit-code
  drift; `demo` byte-unchanged; artifacts rebuilt (Linux + Windows) with `ls -l` evidence.

### Task 3 — Completion Report Resilience Addendum + Final Verification

- Record-only addendum (D-GF6); final standard gate on the final tree; hygiene/secret audit
  (23-command style per Task-4 precedent); exact commands/exit statuses/log paths recorded.

---

## 5. Approval & Execution Notes

- Per-task plans are produced in each 4.1 and saved under the naming pattern above; per the
  standing approval mode, per-task plans are auto-approved once this global plan is approved.
- Failures pause and escalate to the user.
