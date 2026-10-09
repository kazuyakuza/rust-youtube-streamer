# Task 4 Implementation Plan — Establish Repository Hygiene (4.1b)

- Source TODO: `.agent/todos/20261008/20261008-todo-2.md`, `### 4. Establish Repository Hygiene`
- Workflow: `.kilo/commands/critical-workflow.md`, Steps 4.2–4.6 for Task 4
- Date: 2026-10-08
- Branch: `feat/phase00-cargo-baseline` (branch setup was done in global Step 2; do NOT create/switch branches here)
- Sub-agent sequence: 4.2 implementer → 4.3 code-reviewer & code-simplifier → 4.4 docs-specialist (NOT required, see §7) → 4.5b architector → 4.6 implementer
- Front-end related: NO (no 4.1a/4.5a)

---

## 1. Verified Tracked-State Facts (input to decisions)

Collected via `git ls-files` and directory inspection on `feat/phase00-cargo-baseline` (research step of this plan — implementer does NOT re-decide, only re-verify with §5 commands):

1. `Cargo.lock` IS tracked (`Cargo.lock` present in `git ls-files` after Tasks 2–3). Must remain tracked.
2. `logs/.gitkeep` IS tracked; `logs/*` is ignored with `!logs/.gitkeep` un-ignoring it. This is the established directory-placeholder pattern in this repo.
3. `.git-credentials` is NOT tracked (the rule is preventive only).
4. `credentials/` contains exactly ONE tracked file: `credentials/.gitkeep` (verified via `git ls-files`). There is NO `credentials/README.md`. Brief §4 sketches a future `credentials/README.md`, but it does not exist today.
5. `config/` contains exactly ONE tracked file: `config/.gitkeep`. `config/config.json` does not exist yet (will appear at runtime in later phases; must be pre-excluded).
6. `.gitignore` is itself tracked (41 lines, content shown in §3).
7. `.ignore` and `.kilocodeignore` are TRACKED files (they are agent-tooling ignore files, not gitignore). Per caller constraint: they already contain `target/`/`Cargo.lock` entries for tooling purposes; they are NOT `.gitignore`. Task 4 must NOT modify them unless something breaks; this is a record-only note.
8. No FFmpeg binaries, no generated binaries, no real secrets exist anywhere in the tracked tree (verified by `git ls-files` above — only docs/config/agent files, Cargo files, Docker files, `src/main.rs`, and `.gitkeep` placeholders).

## 2. Decisions (non-negotiable, encoded)

1. **Change form**: APPEND-ONLY. Every existing byte of `.gitignore` (lines 1–41) stays unchanged. All new rules are appended at the end of the file in new commented sections. (Decision: this guarantees "preserve existing policy lines" verbatim; no regrouping/reordering of existing rules.)
2. **Rust build output**: add root-anchored `/target/`. (Single-crate project; rooted pattern is precise and conventional.)
3. **Cargo.lock guard**: add explicit `!Cargo.lock` line immediately after `/target/`, with a self-documenting comment. Rationale: cheap, self-documenting protection for the TODO requirement "Do not ignore Cargo.lock"; nothing currently ignores it, but the guard makes the policy explicit and future-safe. Cargo.lock stays TRACKED.
4. **Real config**: ignore `/config/config.json` only. The example config (`config/config.example.json` from brief §5.1) does not exist yet; do not add a speculative `!config/config.example.json` line now (minimal-change rule). The `.gitkeep` in `config/` is unaffected by the exact `config.json` pattern.
5. **Credentials directory**: ignore `credentials/*` with an explicit `!credentials/.gitkeep` un-ignore (mirrors the existing `logs/` pattern exactly — same semantics, same repo convention). This pre-excludes all future OAuth token/credential files `<repo>/credentials/*` from version control while keeping the tracked placeholder. NO `!credentials/README.md` line is added (no README exists; scope-expanding — [DONE] separately by whoever adds it, not in Task 4).
6. **Stream-key material**: stream keys/sensitive JSON blobs have no conventional standalone file today. Do NOT invent speculative patterns (`*.pem`, `token*.json`, etc.). The `credentials/*` ignore plus the existing `# Tokens and secrets` section (`.git-credentials`, `.secrets`) is the complete, minimal, consistent policy. (Decision: no additional patterns.)
7. **Scope prohibitions** (do not touch): `.ignore`, `.kilocodeignore`, `README.md`, `CHANGELOG.md`, `.agent/**` (Task 5 owns docs/metadata), `docker-compose.yml`, `Dockerfile`, `Cargo.toml`, `Cargo.lock` content, `src/**`, no new files anywhere.
8. **No ignore-entries for**: FFmpeg binaries (none exist and none must be bundled), machine-specific paths, CI files.

## 3. Exact Final `.gitignore` Layout

Current file (tracked, 41 lines, lines 1–41 unchanged):

```gitignore
# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Temporary files
*.tmp
*.temp
*.swp
*.swo

# Logs
*.log
logs/*

# Keep folder placeholder under version control
!logs/.gitkeep

# Environment variables
.env
.env.local
.env.production

# IDE files
.vscode/
.idea/

# Build artifacts (generic)
build/
dist/

# Tokens and secrets
.git-credentials
.secrets

# Kilo agent manager
.kilo/agent-manager.json
```

Append EXACTLY these lines (with real newlines, one blank line between sections) at the end:

```gitignore

# Rust build output (appended by Phase 00 Task 4)
/target/
!Cargo.lock

# Real application configuration (config.example.json stays versioned; the real file carries secrets)
/config/config.json

# OAuth credentials and tokens (placeholder stays versioned)
credentials/*
!credentials/.gitkeep
```

Rules of engagement for the edit:
- Use the `edit` tool (not shell redirection) to modify `.gitignore`; preserve CRLF/LF style already present in the file.
- No trailing-newline or whitespace "fixes" to existing lines. No reordering.
- Sections may be styled as comment headings consistent with the existing file (first character `#`); the exact comment texts above are the approved texts.

## 4. Step-by-Step Execution (Step 4.2 — implementer)

### 4.0 Pre-checks

1. Run `git status` and confirm: branch `feat/phase00-cargo-baseline`, clean working tree (or only Task-3-committed state; nothing pending from other tasks). If there are unrelated unstaged changes, STOP and report to the caller before touching anything.
2. Run the verification commands of §5 BEFORE editing to record the "before" state (specifically: `git check-ignore -v Cargo.lock` must print nothing — baseline).

### 4.1 Edit `.gitignore`

1. `read` `.gitignore`.
2. Apply the append from §3 using the `edit` tool: match the final two lines (`.kilo/agent-manager.json` + trailing newline) and replace with themselves plus the six appended lines from §3 (blank line + comment, `/target/`, `!Cargo.lock`, blank line + comment, `/config/config.json`, blank line + comment, `credentials/*`, `!credentials/.gitkeep`).
3. `read` the file again and diff mentally against §3: lines 1–41 byte-identical, exactly 6 new lines appended at end. No `\n`-escape literals (newline-prevention rule).

### 4.2 Verify ignore semantics from the Windows host (single, sequential git commands)

Run individually (single commands, no chaining; these are git CLI operations, permitted by tool-selection rules):

1. `git check-ignore -v target/debug/foo` → MUST print a match showing `.gitignore:<line>` rule `/target/`. (Ignore the fact the path does not exist; `git check-ignore` works on hypothetical paths.)
2. `git check-ignore -v Cargo.lock` → MUST print NOTHING (exit code 1 = not ignored, i.e., tracked/stays tracked). If it prints anything, STOP and report — this is a Task-4 acceptance failure (`!Cargo.lock` must win).
3. `git check-ignore -v config/config.json` → MUST match via `/config/config.json`.
4. `git check-ignore -v config/config.example.json` (hypothetical) → MUST print NOTHING. (Documents that example config would stay versioned.)
5. `git check-ignore -v credentials/token.json` → MUST match via `credentials/*`.
6. `git check-ignore -v credentials/.gitkeep` → MUST print NOTHING (negated by `!credentials/.gitkeep`; confirms the tracked placeholder is not accidentally ignored).
7. `git check-ignore -v logs/chat.log` → MUST still match (`*.log` / `logs/*`), i.e., existing policy intact.
8. `git status` → only `.gitignore` modified; no `.gitignore`-matching artifacts staged (gitignore-compliance rule).

### 4.3 Commit

- Message (exact): `chore: ignore Rust build output and local secret artifacts`
- Stage ONLY `.gitignore`: `git add .gitignore`
- Commit on `feat/phase00-cargo-baseline`. No push (push is restricted to global Step 5). No branch operations (restricted to global Step 2).
- After commit: `git status` clean; `git ls-files -- Cargo.lock` still lists `Cargo.lock` (tracked).

### 4.4 What NOT to do (hard list)

- Do not run any Rust/Cargo/Docker commands; Task 3 state is final.
- Do not touch `.ignore` or `.kilocodeignore` (record-only note; they already cover agent-tooling ignores).
- Do not touch README/CHANGELOG/docs/.agent files (Task 5 owns them).
- Do not create `config/config.example.json`, `credentials/README.md`, `fonts/README.md`, or any other file — not part of Task 4.
- Do not add speculative secret-file patterns beyond §2.6.
- Do not commit real credentials, machine-specific paths, or binaries (none exist; keep it that way).

## 5. Verification Commands Reference (copy-paste)

```
git status
git check-ignore -v target/debug/foo
git check-ignore -v Cargo.lock
git check-ignore -v config/config.json
git check-ignore -v config/config.example.json
git check-ignore -v credentials/token.json
git check-ignore -v credentials/.gitkeep
git check-ignore -v logs/chat.log
git add .gitignore
git status
git commit -m "chore: ignore Rust build output and local secret artifacts"
git ls-files -- Cargo.lock
```

Expected matrix:

| Argument | check-ignore result |
|---|---|
| `target/debug/foo` | matched (rule `/target/`) |
| `Cargo.lock` | NOT matched |
| `config/config.json` | matched (rule `/config/config.json`) |
| `config/config.example.json` | NOT matched |
| `credentials/token.json` | matched (rule `credentials/*`) |
| `credentials/.gitkeep` | NOT matched |
| `logs/chat.log` | matched (rule `logs/*` or `*.log`) |

## 6. Step 4.3 Applicability — REQUIRED

Task 4 is a repository-level configuration change and the global plan marks it for review. Both code-reviewer and code-simplifier MUST be invoked on the committed diff (`git show HEAD -- .gitignore` scope):
- code-reviewer: verify diff = exactly the §3 append-only change; verify §5 expected matrix holds; verify no existing rule altered.
- code-simplifier: for a 6-line append-only gitignore change there is nothing to simplify → this is a documented NOT-required finding if no issue appears (emit clear "not required" message per workflow).

## 7. Step 4.4 Applicability — NOT REQUIRED

Documentation updates (README, project-structure, context.md) are exclusively owned by Task 5 per the global plan. No docs-specialist code-comment work applies to `.gitignore`. Record as "not required — docs owned by Task 5".

## 8. Step 4.5b Applicability — REQUIRED

Architector verifies:
1. Planned file content == actual tracked `.gitignore` content (appended block byte-identical to §3).
2. §5 expected matrix passes as reported.
3. Commit exists with the exact message; only `.gitignore` changed in it.
4. `Cargo.lock` still tracked; `logs/.gitkeep` still tracked; `config/.gitkeep` still tracked.
5. Acceptance criterion covered: TODO line 112 — "`target/` and local secrets/runtime artifacts are excluded from version control; `Cargo.lock` remains tracked."
6. No edits to `.ignore`/`.kilocodeignore` or any file outside `.gitignore`.

## 9. Step 4.6 — Task Completion

- Append `[DONE]` to the `### 4. Establish Repository Hygiene` heading in `.agent/todos/20261008/20261008-todo-2.md` (preserve all file content; overwrite-prevention rule). Do NOT touch other tasks' headings or acceptance checkboxes.
- Commit with message: `chore: mark Phase 00 Task 4 repository hygiene done`
- Include in the completion report: files changed (`.gitignore`, TODO), check-ignore matrix results, and confirmation that Cargo.lock/logs/.gitkeep/config/.gitkeep tracking is unchanged.

## 10. Acceptance Criteria Mapping

| TODO Task 4 sub-item | Plan coverage |
|---|---|
| Review `.gitignore`; only required additions | §2.1 append-only, minimal §3 block |
| `target/` ignored | §3 `/target/` + §5 row 1 |
| Preserve config/OAuth/keys/logs policy | §3 `/config/config.json`, `credentials/*`, `!credentials/.gitkeep`; existing `.git-credentials`, `.secrets`, `*.log`, `logs/*`, `.env*` untouched (§4.4) |
| Do not ignore `Cargo.lock` | §2.3 `!Cargo.lock` guard; §5 row 2; commit keeps it tracked |
| No real creds / machine paths / binaries / FFmpeg in repo | §2.7–2.8, §4.4 |
| Docker stays toolchain-only (Task 3 committed state final) | §4.4 "do not run/change Docker files" |
| Acceptance criterion (TODO line 112) | §8.5 |
