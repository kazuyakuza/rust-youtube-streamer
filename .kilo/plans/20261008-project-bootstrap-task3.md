# Task 3 Plan — Set Up Initial Structure Folders (.gitkeep placeholders)

- Date: 2026-10-08
- TODO source: `.agent/todos/20261008/20261008-todo-1.md` (Task 3)
- Global plan (binding): `.kilo/plans/20261008-project-bootstrap.md` — "Task 3 (structure)" decisions
- Source of truth for structure: `.agent/project-info/brief.md` §4
- Workflow used for the structure-map update: `.kilo/commands/project-structure.md` (Project Structure Maintenance Workflow)
- Rule: `.kilo/rules/project-structure.md`
- Scope of this plan: Critical Workflow step 4.2 (Implementation). NO commit here — commit happens in 4.6.
- Front-end related: NO (skip 4.1a / 4.5a).

---

## 0. Deviations From Binding Decisions (flagged to caller — verify before executing)

1. **`.gitignore` negation mechanism.** The binding decision says "add a negation line `!logs/.gitkeep` ... adjacent to the existing `logs/` rule". This does not work technically: per git semantics ("It is not possible to re-include a file if a parent directory of that file is excluded"), the pattern `logs/` excludes the `logs/` directory itself, so `!logs/.gitkeep` under it has no effect and `logs/.gitkeep` would remain untracked.
   - **Correction encoded in this plan (preserves the binding intent):** change `logs/` → `logs/*` and add `!logs/.gitkeep` after it with a one-line comment. Outcome matches the intended result: `logs/.gitkeep` is versioned, everything else in `logs/` stays ignored (`logs/*`), and chat logs stay ignored (`*.log`, `logs/*`).
   - If the caller rejects this correction, stop and return the question to the caller. Do NOT silently keep the non-working pattern.
2. **No other deviations.** All other binding decisions are encoded verbatim.

## 1. Out-of-Scope (junior implementer: do NOT do these)

- No commit, no staging, no TODO-file `[DONE]` marking (belongs to 4.6).
- No README.md edits, no `config.example.json`, no `credentials/README.md`, no `fonts/README.md` (user constraint: "no files, just .gitkeep").
- No Cargo scaffolding, no `Cargo.toml`, no `main.rs`, no `mod.rs` files.
- No edits to anything under `.agent/project-info/` (including `architecture.md` Module Map and `CONTEXT.md` — context.md update belongs to docs-specialist in 4.4).
- No future ignore rules for `credentials/*` real credential files (flagged as a follow-up observation for the caller, NOT this task).
- No changes to existing `docs/` files.

## 2. Pre-Verification (implementer re-checks state before touching anything)

Run in `C:\repo\rust-youtube-streamer`:

```powershell
git status
git branch --show-current
```

Expected: branch `feat/project-bootstrap`; clean-ish tree; `src/` contains only `.gitkeep`; no `config/`, `credentials/`, `fonts/`, `logs/` directories at repo root; `.agent/project-structure.md` still shows `# (no folders yet)` under `# Folders in src/`.

If branch is not `feat/project-bootstrap` or the expected state differs → stop, return question to caller. Do not assume anything.

## 3. Step-by-Step Implementation

### Step 3.1 — Create the 4 root folders

Single command (one cmd, no chaining):

```powershell
New-Item -ItemType Directory -Force -Path "config", "credentials", "fonts", "logs"
```

### Step 3.2 — Create the 4 root `.gitkeep` placeholder files (empty)

Single command:

```powershell
New-Item -ItemType File -Force -Path "config\.gitkeep", "credentials\.gitkeep", "fonts\.gitkeep", "logs\.gitkeep"
```

Files must be exactly 0 bytes. Do not add any text.

### Step 3.3 — Create the 6 `src/` subfolders

Single command:

```powershell
New-Item -ItemType Directory -Force -Path "src\app", "src\config", "src\youtube", "src\chat", "src\renderer", "src\streaming"
```

Exact folder names (spelling matters — all lowercase, singular, per brief §4): `app`, `config`, `youtube`, `chat`, `renderer`, `streaming`.

### Step 3.4 — Create the 6 subfolder `.gitkeep` placeholder files (empty)

Single command:

```powershell
New-Item -ItemType File -Force -Path "src\app\.gitkeep", "src\config\.gitkeep", "src\youtube\.gitkeep", "src\chat\.gitkeep", "src\renderer\.gitkeep", "src\streaming\.gitkeep"
```

All empty (0 bytes).

### Step 3.5 — Remove the now-redundant `src/.gitkeep`

This removal is explicitly in scope (global plan: "Remove `src/.gitkeep` (redundant once subfolders exist)").

Single command:

```powershell
Remove-Item -LiteralPath "src\.gitkeep"
```

If the file was already deleted → skip silently, no error handling needed beyond noting it in the summary.

### Step 3.6 — Edit `.gitignore` (the flagged correction; see §0)

Use the `edit` tool (structured file edit, NOT shell redirection). Read `.gitignore` first, then replace exactly this block:

```text
# Logs
*.log
logs/
```

with exactly this block:

```text
# Logs
*.log
logs/*

# Keep folder placeholder under version control
!logs/.gitkeep
```

Rules:
- The blank line between `logs/*` and the negation comment block is deliberate (readability); the negation line itself must be `!logs/.gitkeep` exactly.
- Do not touch any other line of `.gitignore` (including `.secrets`, `.git-credentials`, `*.log`, build artifacts, IDE files, `.kilo/agent-manager.json`).
- Do not add any `credentials/` or `config.json` ignore rules in this task.

### Step 3.7 — Update `.agent/project-structure.md`

Per the Project Structure Maintenance Workflow (Update Workflow + File Format):
- `# Folders in src/` section: one line per folder path **relative to `src`**, bullet format `- folder/path/ - brief comment for AI agent`.
- `# Other folders` section: root-level folders, keep all existing entries, add the 4 new ones in alphabetical order.
- Comments: minimal, AI-agent-focused.
- Only folders are documented (not files).
- The `# (no folders yet)` placeholder must be removed from `# Folders in src/` since it now has folders.

Use the `write` tool to replace the whole file with EXACTLY this content (real newlines, no literal `\n`):

```markdown
# Project Structure

# Folders in src/

- app/ - application orchestration: startup, task coordination, error handling, shutdown
- config/ - loading, validation and exposure of the application configuration (src module)
- youtube/ - YouTube API integration: OAuth auth, broadcast lifecycle, streams, chat transport
- chat/ - chat message model and bounded visible-message queue (ChatStore)
- renderer/ - converts RenderInput to raw RGB24 video frames; no YouTube/FFmpeg dependency
- streaming/ - FFmpeg process supervision and raw-frame input pipeline

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- config/ - runtime JSON configuration files (real config.json excluded from version control later)
- credentials/ - OAuth credentials and token storage location (no files committed here)
- docs/ - Documentation files
- fonts/ - font files used by the renderer (no files yet)
- logs/ - application/chat log output directory (contents ignored; only .gitkeep placeholder versioned)
```

Formatting notes for the implementer:
- Heading style matches the current file (`#` level 1 headings, section order and wording `# Folders in src/` / `# Other folders` exactly as the Workflow file format requires).
- Existing three entries (`.agent/`, `.kilo/`, `.opencode/`) and `docs/` keep their exact comment text — only their ordering may change due to alphabetical placement.
- There is exactly one blank line between the file title, `# Folders in src/` block, and `# Other folders` block (matching the current file's spacing).

### Step 3.8 — Verification (plan-adherence; no build/test suite exists yet, so 4.5b here is verification-only)

Run each command separately (no chained commands):

1. `Get-ChildItem -Directory | Select-Object -ExpandProperty Name` → must list (among existing): `config`, `credentials`, `fonts`, `logs`, `docs`, `src` and NOT lose any pre-existing entry.
2. `Get-ChildItem -Recurse -Force -Path src | Select-Object fullName` → expect exactly:
   - `src\app\.gitkeep`, `src\config\.gitkeep`, `src\youtube\.gitkeep`, `src\chat\.gitkeep`, `src\renderer\.gitkeep`, `src\streaming\.gitkeep`
   - and NO `src\.gitkeep`.
3. `Get-ChildItem -Force -Path config, credentials, fonts, logs | Select-Object fullName` → each directory contains exactly one file: `.gitkeep`.
4. `git check-ignore -v logs\.gitkeep` → expected: exit code 1 / no output (i.e., NOT ignored). If it reports a match, the `.gitignore` edit in 3.6 is wrong — fix it per §0 correction. (Note: PowerShell surfaces git's non-zero exit as an error line mentioning exit code 1 — that non-zero IS the success signal here.)
5. `git check-ignore -v logs\chat.log` → expected: matched by `.gitignore` rule `*.log` (proves chat logs remain ignored).
6. `git check-ignore -v src\app\.gitkeep` → expected: NOT ignored (no output).
7. `git status` → expect untracked: `config/`, `credentials/`, `fonts/`, `logs/.gitkeep` (git collapses `logs/contents`; verify `-uall`), plus the modified `.gitignore` and `.agent/project-structure.md`. Run `git status --porcelain -uall` and confirm exactly these new untracked paths:
   - `.gitignore` (modified, ` M`)
   - `.agent/project-structure.md` (modified, ` M` — note: `.agent/` is a tracked folder already)
   - `config/.gitkeep`, `credentials/.gitkeep`, `fonts/.gitkeep`, `logs/.gitkeep` (added, `??`)
   - `src/app/.gitkeep`, `src/config/.gitkeep`, `src/youtube/.gitkeep`, `src/chat/.gitkeep`, `src/renderer/.gitkeep`, `src/streaming/.gitkeep` (added, `??`)
   - `src/.gitkeep` deletion listed only if it was previously tracked (` D`); if it shows nothing, confirm it was untracked via `git ls-files src/.gitkeep` before assuming a problem.

STOP after verification. Do NOT stage, do NOT commit, do NOT mark the TODO `[DONE]`. Summary goes to 4.3 (code review) next.

## 4. Success Criteria (compare against Task 3 line item)

- [ ] 4 root folders exist: `config/`, `credentials/`, `fonts/`, `logs/`, each containing ONLY an empty `.gitkeep`.
- [ ] `docs/` left untouched (it already has files; no `.gitkeep` added there).
- [ ] 6 `src/` subfolders exist with empty `.gitkeep` each: `src/app/`, `src/config/`, `src/youtube/`, `src/chat/`, `src/renderer/`, `src/streaming/`.
- [ ] `src/.gitkeep` removed.
- [ ] `.gitignore` contains `logs/*` + `!logs/.gitkeep` (flagged §0 correction) with a one-line comment; everything else unchanged.
- [ ] `logs/.gitkeep` verified trackable via `git check-ignore`; `logs/chat.log` still ignored.
- [ ] `.agent/project-structure.md` updated exactly per §3.7 (Workflow file format respected).
- [ ] No other files created or modified anywhere. No commit.

## 5. Journal Entry (append below the line when the step is done)

Record in the step summary returned to the caller: folders created, files created, `src/.gitkeep` removal result, `.gitignore` diff applied, project-structure.md updated, verification outputs (esp. the two `git check-ignore` results), and note that the §0 deviation was applied.
