# Task 3 Simplification Plan — Trim Non-Purpose Parentheticals in the Structure Map

- Date: 2026-10-08
- Critical Workflow step: 4.3 output (Simplification review for Task 3)
- TODO source: `.agent/todos/20261008/20261008-todo-1.md` (Task 3) — do NOT edit it here
- Implementation plan (delivered work): `.kilo/plans/20261008-project-bootstrap-task3.md`
- Governing workflow: `.kilo/commands/project-structure.md` → File Format: "Comments should be minimal and focused on the folder's purpose from an AI agent's perspective"
- Junior-implementer target: YES — 3 atomic text edits, fully specified, zero judgment calls.

---

## 1. Review Verdict (what was examined)

| Deliverable | Verdict |
|---|---|
| 4 root folders + 6 `src/` subfolders + 10 `.gitkeep` files, `src/.gitkeep` removed | Structural — out of simplification scope. No action. |
| `.gitignore` Logs block (`logs/*` + one-line comment + `!logs/.gitkeep`) | Compliant: single meaningful comment line, functionally verified (negation match confirmed via `git check-ignore -v logs\.gitkeep`). No action. |
| `.agent/project-structure.md` headings/ordering/format | Workflow-compliant (`# Project Structure` / `# Folders in src/` / `# Other folders`, bullet format, relative paths). No action. |
| `.agent/project-structure.md` comment lines | **3 lines carry redundant or self-staling parentheticals → simplified below.** Remaining lines kept (rationale in §4). |

## 2. Out-of-Scope (junior implementer: do NOT do these)

- No commit, no staging, no TODO-file `[DONE]` marking (belongs to 4.6).
- No edits to `.gitignore`, folders, `.gitkeep` files, or anything else in the repo.
- No edits to the existing entries `.agent/`, `.kilo/`, `.opencode/`, `docs/` (their text predates Task 3 and is pinned by the Task 3 plan §3.7).
- No reordering, no section-heading changes, no other line touched.

## 3. Exact Edits (atomic, in order)

Use the `edit` tool. Each `oldString` below is unique in the file. Do not add trailing whitespace.

### 3.1 — src `config/` line (drop header-redundant parenthetical)

- old: `- config/ - loading, validation and exposure of the application configuration (src module)`
- new: `- config/ - loading, validation and exposure of the application configuration`
- Why: the entry already sits under `# Folders in src/`; "(src module)" repeats the section header.

### 3.2 — root `config/` line (drop future VCS state)

- old: `- config/ - runtime JSON configuration files (real config.json excluded from version control later)`
- new: `- config/ - runtime JSON configuration files`
- Why: "later" describes a not-yet-existing ignore rule; it goes stale the moment such a rule lands and belongs in `.gitignore`, not the structure map.

### 3.3 — `fonts/` line (drop snapshot-of-absence note)

- old: `- fonts/ - font files used by the renderer (no files yet)`
- new: `- fonts/ - font files used by the renderer`
- Why: "(no files yet)" is a transient state, not folder purpose; it becomes false the first time a font is added.

## 4. Comments Deliberately Kept (do NOT trim these)

- `credentials/ ... (no files committed here)` — active security-relevant guardrail for agents.
- `logs/ ... (contents ignored; only .gitkeep placeholder versioned)` — documents the live, already-implemented `.gitignore` contract; prevents agent confusion when files added there appear untracked.
- All `src/` module lines (after 3.1) mirror brief §4 module purposes; each is a purpose statement, kept intact.
- `.gitignore` comment `# Keep folder placeholder under version control` — one line, meaningful, explains the negation; keep verbatim.

## 5. Verification (run each command separately, no chaining)

1. `read` the file → expect exactly 21 lines; the three edited lines match §3 new-text verbatim; everything else byte-identical to the pre-edit file.
2. `grep` tool, path `.agent/project-structure.md`:
   - pattern `\(src module\)` → expect 0 matches
   - pattern `excluded from version control later` → expect 0 matches
   - pattern `\(no files yet\)` → expect 0 matches
3. `git status --porcelain -uall` → same path set as before this step (`.agent/project-structure.md` still ` M`; no new/removed files; `.gitignore` still its prior ` M`, unedited by this step).

## 6. Success Criteria

- [ ] 3 parentheticals removed exactly per §3; no other character changed anywhere.
- [ ] Structure map still satisfies the Workflow File Format (headings, order, bullet format).
- [ ] No `.gitignore`/folder/`.gitkeep` changes. No commit, no staging, no TODO edit.

## 7. Note for Caller (Planner Agent)

This refines the exact §3.7 text of `.kilo/plans/20261008-project-bootstrap-task3.md`: after this step the file intentionally differs from that block by the 3 trims. Record that deviation note in the 4.6 journal so the diff against the implementation plan stays explainable.
