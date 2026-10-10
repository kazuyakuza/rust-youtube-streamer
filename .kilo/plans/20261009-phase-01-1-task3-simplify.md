# Simplification Plan — Phase 01.1, Task 3: Document the Build Workflow

- Critical Workflow step: 4.3 (Code Simplifier review output) for TODO Task 3
- Source TODO: `.agent/todos/20261009/20261009-todo-4.md` → `### 3. Document the Build Workflow`
- Frozen contract (authoritative): `.kilo/plans/20261009-phase-01-1-task3-build-docs.md` (commit `70f377e`) — Frozen Decisions 1–9 and every mandatory-content list in §4 Step 1 are binding; this plan simplifies wording ONLY where it converges toward, not away from, that contract.
- Reviewed artifacts: `docs/build.md` (89 lines, `4cf8679`); `README.md` two new hunks (`7eb2583`); `.agent/project-structure.md` 4 edits + `.agent/project-info/context.md` 3 edit points (`417d6ba`)
- Review date: 2026-10-10 — branch `feat/linux-release-build-artifact`, clean tree at `417d6ba`
- Plan consumer: implementer sub-agent (JUNIOR, under 50% restriction) in sub-step 4.3-fix. Execute exactly the single edit in §2. NO judgment calls, NO other changes.
- Front-end related: **NO** — 4.5a skipped.

## 1. Review verdict

The Task 3 docs are byte-faithful to the frozen plan and internally consistent (TOC anchors, relative links, verbatim command string, verbatim one-line distinction — all re-verified, see §4). Exactly ONE trivial, zero-risk, pure-wording refinement was found (S1): a stale one-word prefix in `.agent/project-structure.md` that the Task 3 edit itself made inaccurate. Everything else on the checklist is rejected because it is frozen plan content or acceptance-critical; no factual statement changes anywhere.

## 2. The single approved edit (S1) — trivial / zero-risk (pure wording)

**Problem:** at `417d6ba` the `scripts/` entry gained the `build-linux.sh` clause but kept the pre-existing prefix `developer check utilities`. `build-linux.sh` is a release builder, not a checker, so the word "check" now mislabels the entry. The frozen Task 3 plan §4 Step 3 edit-1 proposed exactly `developer utilities` as its example wording. Removing the one stale word converges the file to the plan's own suggestion; the `dev-checks.sh` clause below it still fully documents the check role.

**Exact edit** — in `C:\repo\rust-youtube-streamer\.agent\project-structure.md`, replace the single line (currently line 41):

```markdown
- scripts/ - developer check utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory; `build-linux.sh` builds the release with the tracked lockfile in the same service and copies the final Linux executable to gitignored `dist/`
```

with the byte-identical line except ONE removed word (`check `):

```markdown
- scripts/ - developer utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory; `build-linux.sh` builds the release with the tracked lockfile in the same service and copies the final Linux executable to gitignored `dist/`
```

- ONLY the two-word prefix `developer check utilities` → `developer utilities` changes. Both script clauses (everything from `` `dev-checks.sh` `` to end of line) stay byte-identical.
- No other line in the file changes: the `docs/` entry, the `dist/` bullet, and the `Cargo.toml v0.3.0` line are the other three frozen Task 3 edits and must remain untouched.
- Preserve real LF newlines and the file's trailing newline (newline-prevention rule).

## 3. Why content is provably preserved

- No fact removed or added: the entry still names both scripts and states each one's exact purpose; "utilities" is the hypernym that already covers "check utilities".
- TODO Task 3 bullet "Update `.agent/project-structure.md` with the script and documentation paths and the ignored `dist/` output" stays satisfied — all path mentions (`dev-checks.sh`, `build-linux.sh`, `docs/`+`build.md`, `dist/`) are untouched.
- Acceptance row "`.agent/project-structure.md` ... reflect the final state" is strengthened (the prefix stops contradicting the entry's own content).
- Not a Frozen Decision violation: Frozen Decision 6 lists four edit points for the file; S1 only finishes aligning edit point (a) with the plan's own proposed wording. `dev-checks.sh`'s pre-existing descriptive sentence — the part Frozen Decision 6/Step 3 keeps intact — is not touched.
- Docs-only wording change in `.agent/` metadata: no source, script, or config file is involved; no Cargo/dev-checks relevance.

## 4. Rejected candidates (do NOT implement) — per-file evidence

Recorded so the implementer and 4.5b adherence review see they were considered and rejected.

### 4.1 `docs/build.md`

1. **"not a Windows `.exe`" appears twice** (§Expected Output vs §What This Proves and Does Not Prove): both are plan-mandated content of different frozen sections (Step 1 Expected Output bullet 1; Frozen Decision 3 places the caveat canonically in the proves-section). The Expected-Output instance is already minimal and cross-links to the caveat section. Consolidating would delete a mandatory bullet. Rejected.
2. **"Cargo diagnostics never suppressed" twice** (Build Command vs Failure Semantics intro): both are mandatory content of their frozen sections (Step 1 Build Command bullet 3; Step 1 Failure Semantics frozen preamble). Rejected.
3. **"rebuilds are fast" twice** (Build Command cold-run note vs Overwrite incremental note): both are plan-mandated, each anchored to its section's job (runtime expectation vs rebuild semantics). Rejected.
4. **README Build Checks anchor linked twice + `../README.md` once more in See Also**: each instance is mandatory at point of use (Step 1 Prerequisites bullet 1, Dev Checks bullet 1, See Also bullet 1). TOC/anchor redundancy check found no duplicate or dead anchor: all 9 TOC slugs match their headings exactly (`#dev-checks-vs-release-build` from "Dev Checks vs. Release Build" verified), and `../README.md#build-checks-docker-via-alpine-vm`, `#what-this-proves-and-does-not-prove`, `#failure-semantics`, `configuration.md`, `../scripts/dev-checks.sh`, `../README.md` all resolve to real headings/files. Rejected.
5. **"binary" vs "executable" terminology**: `binary` occurs only inside the quoted literal script error `error: expected release binary not found or empty: <path>` (failure-table row 4), which must stay verbatim; all prose consistently uses "executable"/"artifact" per section role. No mixing to fix. Rejected.
6. **Long Prerequisites bullet**: mirrors the plan skeleton's single-bullet structure; the colon list parses clearly. Splitting would alter the frozen mandatory-content bullet structure for cosmetic gain. Rejected.
7. **Success-message quote (§Purpose) vs "prints the final path and size" (§Expected Output)**: different frozen roles (user-perspective success criterion vs the no-size-hardcoding rule, Frozen Decision 4); neither is redundant. Rejected.
8. Standard invocation string verified byte-identical in `docs/build.md` and the new README section, once per file; size `1715296` correctly absent from `build.md`. Nothing to trim.

### 4.2 `README.md`

- The new `## Release Build (Linux Artifact)` section + TOC line are frozen verbatim by Step 2 (exact text, exact position, byte-for-byte command block, Frozen Decision 5: exactly two edits). It is three short paragraphs + the mandatory one-line distinction + the mandatory caveat, ending in the `docs/build.md` link — a genuine "short summary" per the TODO bullet; heavier duplication does not exist. Any trim would break the frozen plan. Rejected — no change.

### 4.3 `.agent/project-info/context.md`

- All three frozen edit points match Step 4 verbatim (Current Work Focus; `## Recent Changes (2026-10-09, Phase 01.1)`; Immediate Next Steps item 1). Voice/grammar consistent with the file's existing terse `*`-bullet living-log style; no first person; no placeholders; commit refs `f9a6b0e`, `92880c7`, `6118322` verified to exist. No other sections changed (`git show 417d6ba` diff confirms). No grammar defect found. Rejected — no change.

### 4.4 `.agent/project-structure.md`

- Frozen edits (b) `docs/` entry, (c) `dist/` bullet, (d) `v0.3.0` verified exact vs Step 3; all other entries byte-identical to the pre-Task-3 state (diff-checked). Only edit (a)'s stale prefix remains addressable → S1.

## 5. Implementer procedure (4.3-fix, in order, atomic)

1. Precheck: `git status --short --branch` → expect `## feat/linux-release-build-artifact` with NO file entries; `git merge-base --is-ancestor 417d6ba HEAD; echo $?` style equivalent — verify HEAD is on `feat/linux-release-build-artifact` and that commits `4cf8679`, `7eb2583`, `417d6ba` are in its history (e.g. `git log --oneline -6`). If anything differs, STOP and ask the caller.
2. Read `.agent/project-structure.md` (tool contract), then apply the §2 edit exactly.
3. Diff self-check: `git diff` must show exactly one modified line, the `- scripts/ - ...` line, with only the word `check ` removed from the prefix; the remaining `check` occurrences (`format/check/test/clippy checks`, `logs/checks/`) still present on that line.
4. Markdown verification (pure self-check subset of the source plan's Step 6; NO VM, NO dev-checks run — Frozen Decision 1):
   - `grep` `.agent/project-structure.md`: `developer utilities` → exactly 1 hit; `developer check utilities` → 0 hits; `` `build.md` (Linux release build guide `` → 1 hit; `- dist/ - gitignored release-build output` → 1 hit; `v0.3.0` → 1 hit (Cargo.toml entry).
   - `git status --porcelain` → expected exactly ` M .agent/project-structure.md` and nothing else (proves `docs/build.md`, `README.md`, `context.md`, and every other file are untouched).
5. Git handling (gitignore-compliance rule): read `.gitignore`; stage ONLY `.agent/project-structure.md` (matches no ignore pattern); commit with message exactly:

   `refactor: tighten build docs wording`

   (caller-frozen for this 4.3-fix cycle; the source plan's three frozen messages belong to Steps 1–3 implementation commits already made).
6. Post-commit: `git status --short --branch` → clean; `git log --oneline -1` shows the frozen message. NO push, NO TODO `[DONE]` flips or checkbox edits, NO `.agent/reports/**` creation (Task 4 owns those), no branch operations.

## 6. Hard boundaries for this cycle

- Only `.agent/project-structure.md` may change; only the single line of §2.
- Forbidden: `docs/build.md`, `README.md`, `.agent/project-info/context.md`, `scripts/**`, `src/**`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `docker-compose.yml`, `.gitignore`, `.agent/todos/**`, `.agent/reports/**`, any other plan file, new/deleted files.
- No `alpine-vm` MCP calls, no Docker, no cargo, no `dev-checks.sh` run, no network fetches (Frozen Decision 1 of the source plan extends to this docs-only cycle).
- Never stage anything matching `.gitignore` (`dist/`, `logs/`, `.kilo/agent-manager.json`); never force-add; never push.

## 7. Return to caller (simplifier summary)

- Approved: 1 trivial zero-risk wording edit (S1: drop stale `check` from the `scripts/` entry prefix; aligns with the frozen plan's own example wording).
- Rejected: 8 candidate classes in `docs/build.md`, 1 in `README.md`, plus full no-change verification passes on `README.md`/`context.md`/remaining `project-structure.md` edits — all documented with evidence in §4.
- Verification contract: §5 steps 3–4 (single-line diff proof + grep self-checks + one-file `git status`), single commit `refactor: tighten build docs wording`, tree clean, no VM.
