# Task 3 Fix Plan — Game-Foundation Handoff Documentation

Scope: docs-only (the 8 paths already touched). No code/config/dependency/version change.

## Findings

Reviewed against `.kilo/plans/20261101-game-handoff-task3-plan.md` §5 ledger L1–L30 and the Task-3/Global-AC checklists.

**Status: ONE structural deviation from the binding plan.**

The binding plan §2.4 D defines exactly **four** extension points/seams:

1. `src/app/pipeline/source.rs :: MessageSource`
2. `src/renderer/renderer.rs :: Renderer` trait + `RenderInput`
3. The demo timeline (`src/app/demo/schedule.rs` + `source.rs`)
4. Config (`src/config/model.rs` + `validation.rs` precedent)

The delivered `docs/game-foundation.md` adds a **fifth** extension point:

> 5. The diagnostics collector (`src/app/diagnostics/`) — read-only seams already instrument the pipeline; new game metrics can attach at the same sites.

This expands the extension-point map beyond the plan's four-seam design. The diagnostics collector is correctly positioned as a **reused-unchanged** component earlier in the same file and as a module row in `.agent/project-info/architecture.md`; it is not a game-extension seam.

The `.agent/project-info/context.md` Task-3 summary also drifts from the plan by listing a different seam set (`MessageSource`, `Renderer` trait, `FrameSink`, the demo schedule, the diagnostics collector) and omitting `Config`.

## Required Fixes (apply in 4.4)

### Fix 1 — Remove the unplanned fifth extension point

- **File:** `docs/game-foundation.md`
- **Replace:**

```text
5. The diagnostics collector (`src/app/diagnostics/`) — read-only seams already instrument the pipeline; new game metrics can attach at the same sites.
```

- **With:** *(delete the line; the list remains 1–4)*
- **Rationale:** Restores conformance to the binding plan §2.4 D, which enumerates exactly four extension points. The diagnostics collector remains correctly described in the `Reuse Unchanged` section and in the `src/app/diagnostics/` module row of `architecture.md`.

### Fix 2 — Align `context.md` seam summary with the plan

- **File:** `.agent/project-info/context.md`
- **In the Task-3 block (line 35 area), replace:**

```text
the reuse-unchanged vs extension-point map (seams: `MessageSource`, `Renderer` trait, `FrameSink`, the demo schedule, the diagnostics collector; frozen: startup stages, broadcast lifecycle, RTMPS argv + capacity-64 channel, shutdown order, exit codes, redaction)
```

- **With:**

```text
the reuse-unchanged vs extension-point map (seams: `MessageSource`, `Renderer` trait, the demo schedule, `Config`; frozen: startup stages, broadcast lifecycle, RTMPS argv + capacity-64 channel, shutdown order, exit codes, redaction)
```

- **Rationale:** The `context.md` summary must match the binding plan's four-seam list. `FrameSink` is part of the streaming boundary reused unchanged, and the diagnostics collector is read-only; neither is a game-extension seam. `Config` is the fourth planned seam.

## Verification After Fixes

- `docs/game-foundation.md` `### Extension Points` contains exactly the four bullets from §2.4 D.
- `.agent/project-info/context.md` Task-3 block lists the same four seams.
- No other touched file requires change; all ledger claims, cross-links, TOC anchors, and secret-pattern checks remain valid.
