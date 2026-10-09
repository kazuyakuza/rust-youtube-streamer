# Task 2 Fix Plan — Documentation Factual Accuracy

- Source TODO: `.agent/todos/20261009/20261009-todo-2.md` → "### 2. Update Documentation and Project Metadata" (step 4.3 review)
- Parent plan: `.kilo/plans/20261009-phase00-1-task2-docs-metadata.md`
- Branch: `feat/dev-checks-script`
- Date: 2026-10-09

---

## Defect found

The documented per-check result line format does not match the actual output produced by `scripts/dev-checks.sh`.

- Script actual output (line 35):
  `run_check_result="$run_check_status $run_check_name exit=$run_check_exit ($run_check_duration s)"`
  Example: `PASS fmt-check exit=0 (2 s)`

- README currently documents (line 62):
  `` `PASS|FAIL <name> exit=<code> (<duration>s)` ``

- `context.md` currently documents (line 32):
  `` `PASS|FAIL <name> exit=<code> (<duration>s)` ``

Both docs are missing the space between the duration value and the `s` unit that the script emits.

## Required fixes

Update only the two documentation files below. Do **not** change `scripts/dev-checks.sh` (Task 2 scope forbids touching it).

### Fix 1 — `README.md`

- Location: `## Build Checks (Docker via Alpine VM)` → `### Commands`, behavior paragraph.
- Change `(<duration>s)` to `(<duration> s)` in the per-check result line description.

### Fix 2 — `.agent/project-info/context.md`

- Location: `## Recent Changes (2026-10-09)`, first bullet describing the script.
- Change `(<duration>s)` to `(<duration> s)` in the per-check result line description.

## Scope guard

- Allowed changes: `README.md`, `.agent/project-info/context.md`.
- Forbidden: `scripts/dev-checks.sh`, `.agent/project-structure.md`, `.agent/project-info/brief.md`, `architecture.md`, `tech.md`, `product.md`, `.gitignore`, `Dockerfile`, `docker-compose.yml`, `Cargo.toml`, `Cargo.lock`, `src/**`, TODO file edits, plan file edits.
- No container runs, no check re-runs, no new commits beyond the fix commit(s), no push.

## Verification after fix

1. `grep` README and context.md for `(<duration> s)`; ensure no remaining `(<duration>s)` occurrences.
2. Confirm `scripts/dev-checks.sh` still produces `(<duration> s)` format (already established; no re-run needed).
3. `git diff` shows only the two target files changed.

## Out of scope

No functional changes, no CI, no dependency changes, no application code changes.
