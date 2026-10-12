# Global Plan — Phase 04: OAuth 2.0 and Credential Management

- Date: 2026-10-12
- Origin (TODO file): `.agent/todos/20261012/20261012-todo-1.md` (Pattern C: `## Tasks` → 5 × `###` headings = 5 tasks)
- Request: "full read @AGENTS.md & follow /critical-workflow; do @.agent\todos\20261012\20261012-todo-1.md"
- Workflow: `.kilo/commands/critical-workflow.md` (steps 2–6 below; step 1 done)

## Repository State (verified)

- Branch `main`, clean, up to date with `origin/main` (HEAD `d0d35f5`).
- Package `rust-youtube-streamer-service` v0.6.0 (Cargo.toml), edition 2021, Rust 1.82 pinned image.
- Phase 03 complete (147 tests green, container-only verification); `auth`/`run` are placeholders (exit 3); `demo` is local-only.
- `src/youtube/` is an empty placeholder (tracked `.gitkeep` — the OAuth layer must live there per TODO scope).
- Deps: serde/serde_json, tracing/tracing-subscriber, ab_glyph, ctrlc; dev-dep tempfile. No HTTP client yet, no async runtime (sync-std decision from Phase 03, Tokio reserved).
- `.gitignore` already covers `config/config.json`, `credentials/*` (with `!credentials/.gitkeep`), `videos/`, `dist/`, `logs/*`.
- Build/verify toolchain: Docker Compose `rust` service on the Alpine VM MCP; standard commands recorded in §Verification Commands.

## Front-End Determination

None of the 5 tasks is front-end related (Rust CLI/service, no UI). Sub-steps **4.1a and 4.5a are omitted for every task**; all tasks run 4.1b only.

## Verification Commands (used by sub-agents via Alpine VM MCP)

- Dev checks: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
- Linux artifact: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`
- Windows artifact: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`

## Step 2 — Git Feature Branch Setup → implementer

- `main` is current and clean; nothing to commit.
- Create and switch to feature branch `feat/phase04-oauth-credentials` (feature → `feat/` prefix).
- All work happens on this branch; merge to `main` at step 5.

## Step 3 — Version Update → implementer

- Version lives in `Cargo.toml` (no `package.json`). Feature phase → minor bump: `0.6.0` → `0.7.0`.
- Commit as `chore: bump version to 0.7.0`; refresh `Cargo.lock` via a container `cargo check` when dependencies are added later, keeping `--locked` checks valid.

## Task Cycles (each task = one full 4.1b–4.6 cycle)

### Task 1 — OAuth Design and Dependency Selection

- **4.1b Planning → architector**: research Google's current installed-application OAuth guidance (loopback redirect `http://localhost:<port>` semantics, PKCE requirements, token endpoint behavior, refresh-token rotation/revocation semantics) and the minimal YouTube scope required by later phases (expected candidate: `https://www.googleapis.com/auth/youtube.force-ssl` — MUST be verified against current official docs and justified; do not request broad scopes). Evaluate maintained Rust crates (OAuth crate vs hand-rolled token exchange; HTTP client candidates: sync `ureq` vs async `reqwest`+Tokio — decide and justify against MSRV Rust 1.82 / edition 2021, Windows+Linux, license, maintenance; project policy rejects `google-youtube3`). Decide sync vs Tokio for this phase. Define typed interfaces/errors for: OAuth client config, authorization request, token exchange/refresh, credential persistence, authorization verification — with mockable boundaries (e.g., HTTP transport trait + base-URL override for fakes). Plan file: `.kilo/plans/20261012-phase04-task1-oauth-design.md`.
- **4.2 Implementation → implementer**: implement the planned typed layer under `src/youtube/` (module skeleton, types, errors, selected dependencies in `Cargo.toml` + lockfile refresh via container), no flows wired, no broadcast/chat code.
- **4.3 Review/Simplify → code-reviewer + code-simplifier** (concurrent), then **4.3-fix → implementer** (plans in `.kilo/plans/20261012-phase04-task1-*-fix.md` / `...-simplify.md` when required).
- **4.4 Documentation → docs-specialist**: module docs for the new `src/youtube/` surface; design/dependency rationale recorded (tech.md update deferred to Task 5).
- **4.5b Adherence → architector** (report in `.kilo/plans/20261012-phase04-task1-adherence.md`).
- **4.6 Completion → implementer**: `[DONE]` on Task 1 heading, commit.

### Task 2 — OAuth Configuration and Credential Storage

- **Pre-analysis**: extend `src/config` (serde `deny_unknown_fields` — example file must be regenerated in sync); new OAuth section = client-configuration path + persisted-credential path (defaults consistent with existing conventions; `config/config.example.json` updated; never implicitly loaded). Credential file: JSON token data; atomic persistence (temp file + rename in same directory); restrictive permissions where supported (Unix `0600` via std; Windows ACL limitation documented honestly — no encryption claims). Verify/preserve `.gitignore` `credentials/*` rule. Redaction: errors/logs never contain secrets, tokens, codes, or token-endpoint response bodies.
- **4.1b → architector** (plan: `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md`) → **4.2 implementer** → **4.3 reviewer+simplifier → 4.3-fix implementer** → **4.4 docs-specialist** → **4.5b architector** (`.kilo/plans/20261012-phase04-task2-adherence.md`) → **4.6 implementer**.

### Task 3 — Implement the `auth` Command

- **Pre-analysis**: replace the `auth` placeholder in `src/app/modes.rs`/`src/youtube/`: load+validate OAuth client config before opening the flow; loopback-only local HTTP callback (ephemeral port, `http://localhost:<port>`), CSRF `state` validation, listener closed on success AND failure; console display of the authorization URL; authorization-code exchange with redacted, actionable OAuth error diagnostics; secure credential persistence (Task 2 store); scope/access verification with the Task 1-researched minimal scope; clear non-zero exit on missing/revoked/insufficient authorization; no background listeners/tasks left running; CLI syntax and exit-code contract preserved unless a justified, documented change (decide + document in plan; existing startup-funnel contract: usage=2, config/log=1; the placeholder exit-3 must be retired for `auth` only — `run` keeps exit 3). `run` placeholder and `demo` behavior untouched.
- **4.1b → architector** (plan: `.kilo/plans/20261012-phase04-task3-auth-command.md`) → **4.2 implementer** → **4.3 reviewer+simplifier → 4.3-fix implementer** → **4.4 docs-specialist** → **4.5b architector** (`.kilo/plans/20261012-phase04-task3-adherence.md`) → **4.6 implementer**.

### Task 4 — Credential Loading, Refresh, and Verification

- **Pre-analysis**: reusable credential service for future `run` startup: load stored credentials → determine refresh need → refresh via stored refresh token (network seam mocked in tests) → persist refreshed data atomically; distinct typed errors for absent/malformed/unreadable/expired-unrefreshable/revoked/insufficient-scope; effective-authorization verification via one minimal YouTube API request appropriate to the chosen scope (NO broadcast operations); unit-testable without live Google credentials or network (fake transport/base-URL override); document interactive-first-authorization vs unattended-refresh behavior under Google's current OAuth policies.
- **4.1b → architector** (plan: `.kilo/plans/20261012-phase04-task4-credential-service.md`) → **4.2 implementer** → **4.3 reviewer+simplifier → 4.3-fix implementer** → **4.4 docs-specialist** → **4.5b architector** (`.kilo/plans/20261012-phase04-task4-adherence.md`) → **4.6 implementer**.

### Task 5 — Tests, Documentation, and Completion Evidence

- **Pre-analysis**: fill test-coverage gaps for all acceptance criteria (OAuth config validation, state/callback validation, token exchange success/failure, refresh success/failure, atomic persistence, permission restrictions, redaction); NO test requires a real Google account or network. Then evidence: dev-checks gate + both artifact rebuilds via the exact Verification Commands (record exact exit statuses; cross-compilation ≠ native runtime claims); docs: README, new `docs/authentication.md`, existing docs touch-ups; `.agent/project-structure.md`; `.agent/project-info/context.md`; tracked completion report `.agent/reports/20261012-phase-04-oauth-credentials.md` (dependency rationale, changed files, commands/exit statuses, tests, builds, security review, manual checks, known limitations); final secret-material audit (`git status`, `git ls-files`, `.gitignore` verification — no tokens/secrets/codes/credential files/videos committed).
- **4.1b → architector** (plan: `.kilo/plans/20261012-phase04-task5-tests-docs-evidence.md`) → **4.2 implementer** → **4.3 reviewer+simplifier → 4.3-fix implementer** → **4.4 docs-specialist** → **4.5b architector** (`.kilo/plans/20261012-phase04-task5-adherence.md`) → **4.6 implementer**.

## Step 5 — TODO File Completion → implementer

- Add `[DONE]` to the 5 task headings (Task 4.6 already does each); flip all acceptance-criteria checkboxes to `[x]` at Task 5 completion; rename `.agent/todos/20261012/20261012-todo-1.md` → `20261012-todo-1-DONE.md` (pure rename, content preserved).
- Remove any tmp files/folders created during the process.
- Verify feature branch fully committed; switch to `main`; merge `feat/phase04-oauth-credentials` (fast-forward expected); on success delete the feature branch; on failure notify user.
- Push `main` to `origin` ONLY (origin is set; never push to other remotes).

## Step 6 — Finish (Planner)

- Short work summary + the standard next-chat handoff text.

## Global Constraints (apply to every sub-agent)

- Rust 1.82 / edition 2021; hand-written Display errors, no `thiserror`; no panics in library paths.
- No async runtime unless Task 1 planning justifies and selects one explicitly.
- Never log/commit secrets, tokens, authorization codes, stream keys, or token-endpoint response bodies.
- `demo` unchanged; `run` stays a non-interactive placeholder (exit 3); no broadcast/RTMPS/FFmpeg-production/Live-Chat code.
- Automated tests are deterministic, offline (fakes/mocks only).
- Container-only verification; never claim native Windows/Linux runtime validation.
- Gitignore compliance before every commit; push only to `origin`.
