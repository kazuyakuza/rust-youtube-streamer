# Plan — Phase 01, Task 4: Update Documentation and Project Metadata

Source of truth: `.agent/todos/20261009/20261009-todo-3.md`, Task 4 lines only.
Baseline: Tasks 1–3 are DONE (branch `feat/phase01-config-app-skeleton`, clean tree at commit `d163ade`).
Assignee for steps A–E: implementer/docs-specialist (junior, 50 % restriction). Steps F–G are explicit exclusions handled by the Planner.

---

## 0. Verified Code Facts (use verbatim in docs; do not re-derive or alter)

These facts were read from the current HEAD; the docs must state exactly these, nothing more.

### 0.1 CLI contract (`src/app/cli.rs`, `src/app/mod.rs` docs)
- Executable name: `rust-youtube-streamer-service`.
- Exactly two documented commands: `auth` and `run`.
- One option: `--config <path>`, accepted **before or after** the command, **at most once**.
- `--help` is accepted **only before** the command; it prints usage text to **stdout** and exits successfully (exit 0).
- Missing config option → default path `DEFAULT_CONFIG_PATH = "config/config.json"` (`src/app/cli.rs` line 17).
- The example template `config/config.example.json` is **never** referenced, defaulted to, or implicitly loaded as a live configuration.
- Standard-library parser (no CLI crate — dependency policy note).
- Exit codes (stable contract, `src/app/error.rs` + `src/app/modes.rs`):
  - `0` — successful `--help` request (and successful pipeline, though no command currently completes successfully).
  - `1` — configuration load/parse/validation failure or logging-init failure (`STARTUP_FAILURE_EXIT`).
  - `2` — command-line usage error: missing command, unknown command/option, repeated `--config`, missing `--config` value, `--help` after command, non-UTF-8 argument (`USAGE_ERROR_EXIT`).
  - `3` — selected mode is **not implemented yet** (`NOT_IMPLEMENTED_EXIT`). Applies to both `auth` and `run`.
- `auth` placeholder behavior: after configuration loads and validates, prints one stderr line
  `error: authentication is not implemented yet: ...` and exits 3; never contacts YouTube, never requests or stores credentials.
- `run` placeholder behavior: after configuration loads and validates, prints one stderr line
  `error: runtime pipeline is not implemented yet: ...` and exits 3; never creates YouTube resources, never starts FFmpeg.

### 0.2 Logging contract (`src/app/logging.rs` module docs)
- Stack: `tracing` + `tracing-subscriber`, default `fmt` layout, writes to **standard error**, filter via `EnvFilter`.
- Env variable name: exactly `RUST_LOG` — **case-sensitive**.
- Default filter: `DEFAULT_FILTER = "info"` (`src/app/logging.rs` line 36).
- Resolution order:
  1. unset (including a non-UTF-8 value) → default `info`, **silently**;
  2. empty or whitespace-only value → default `info`, **silently**;
  3. value that fails to compile under `EnvFilter` → exactly **one** `warning:` line on stderr
     (`warning: invalid RUST_LOG value '<value>'; using default filter`), then fallback to `info`;
  4. valid directive → used verbatim after trimming surrounding whitespace.
- `init_logging` is called exactly once per process, before any config load or mode handler.
- Secrets rule (do not state in operator docs; internal only): logs never contain configuration field values, secrets, stream keys, or the full config file.
- Module doc says: "Operator guidance for `RUST_LOG` lives in the project README" — this README section makes that statement true.

### 0.3 Configuration schema + validation (`config/config.example.json`, `src/config/model.rs`, `src/config/validation.rs`)
Schema — five required top-level sections; **unknown fields are rejected** (`deny_unknown_fields`):

| Section | Fields | Rules |
|---|---|---|
| `youtube.broadcast` | `title`, `description`, `privacy_status` | title/description non-blank; `privacy_status` ∈ {`private`, `public`, `unlisted`} (case-sensitive) |
| `video` | `width`, `height`, `fps` (u32), `pixel_format` | width/height/fps > 0; `pixel_format` must be exactly `rgb24` (else rejected, no substitution) |
| `renderer` | `font`, `font_size`, `line_height`, `left_margin`, `top_margin`, `right_margin`, `bottom_margin`, `text_color`, `background_color` | font_size > 0; line_height > 0; margins ≥ 0 (u32); `top_margin + bottom_margin + line_height` must not exceed `video.height` (≥ 1 usable line); font/text_color/background_color non-blank |
| `chat` | `log_file` | non-blank |
| `ffmpeg` | `executable`, `video_codec`, `preset`, `bitrate` | all non-blank |

- File existence (`font`, `executable`) and process-start checks are **not** performed at load time; they belong to the consumer components in later phases. Values are checked for non-blankness and path syntax only.
- All validation violations are collected and reported as **one** actionable error naming file/field (no panics).
- Example values (safe, non-secret): title `Rust YouTube Streamer Prototype`, privacy `unlisted`, 1920×1080 @30, `rgb24`, font `fonts/console.ttf`, size 32 / line 40 / margins 20, `#FFFFFF` on `#000000`, chat log `logs/chat.log`, ffmpeg `ffmpeg` + `libx264` + `veryfast` + `6000k`.
- Real config path `config/config.json` is **gitignored** (`/config/config.json` in `.gitignore`); the example stays versioned.

### 0.4 Phase outcome facts (git log, this branch)
- Phase 01 delivered, in order: typed configuration module with validation (commits `a612009`, `09ca897`, `b69f01c`, `4d6d966`, `c62e937` bump to 0.2.0), application skeleton with auth/run commands (`76bfbc6`, `a0a8a93`, `be76529`, `b5c4d63`), structured logging with env filter (`1b49e88`, `88f0a88-family`, `56e1da1`, `b200aed`).
- Test suite went 48 → 46 tests after the Task 3 refactor (`fd00d04` deduped fixtures).
- Dev-checks gates pass: `cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo clippy --locked -- -D warnings` via `scripts/dev-checks.sh` in the Compose `rust` service.
- **Do not claim** native Windows/Linux runtime validation; container checks only.

### 0.5 Discrepancy note (flag to caller, do not act)
The task prompt said `docs/` is empty, but it currently contains two agent-workflow docs:
`how-to-set-up-git.md` and `how-to-write-todo-files.md`. This does not affect the plan (the new
configuration guide is a new, unrelated file at `docs/configuration.md`), but the completion
report must state this observed state.

---

## A. README updates (`README.md` — edits only, NO rewrite)

Current README is 132 lines. It already exceeds 100 lines, so it already needs a TOC; after edits it will be ~165–185 lines. Only the sections listed below change; every other section stays byte-identical.

### A.1 Target section ordering (final)

1. `# Rust YouTube Streamer Service` + existing 3 intro lines (verbatim)
2. `## Table of Contents` (NEW)
3. `## MVP Summary` (verbatim)
4. `## Key Technologies` (verbatim)
5. `## Current Status` (REPLACED — see A.3)
6. `## Commands and Configuration` (NEW — see A.4)
7. `## Logging (RUST_LOG)` (NEW — see A.5)
8. `## Build Checks (Docker via Alpine VM)` (KEEP — see A.6)
9. `## AI Agents` (verbatim)
10. `## How to Start a Task` (verbatim)
11. Footer italic line `*Initial README — ...*` (verbatim — do not touch)

### A.2 TOC (insert immediately after the intro paragraph, before `## MVP Summary`)

Format: flat markdown link list using GitHub-style slugs (same anchor style already used by the existing in-page link `[Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm)`):

```markdown
## Table of Contents

- [MVP Summary](#mvp-summary)
- [Key Technologies](#key-technologies)
- [Current Status](#current-status)
- [Commands and Configuration](#commands-and-configuration)
- [Logging (RUST_LOG)](#logging-rust_log)
- [Build Checks (Docker via Alpine VM)](#build-checks-docker-via-alpine-vm)
- [AI Agents](#ai-agents)
- [How to Start a Task](#how-to-start-a-task)
```

Anchor naming rule for the implementer: lowercase, spaces → `-`, parentheses stripped, `(` `)` removed. `Logging (RUST_LOG)` → `#logging-rust_log` (GitHub keeps the underscore in `RUST_LOG`). Verify each anchor by comparing against the rendered slug of the actual heading; do not guess.

### A.3 `## Current Status` — REPLACE the section body only (keep the `##` heading line)

Replace the current four Paragraph-00 paragraphs (lines 27–32) with new content covering **exactly these facts, in this order**:

1. Phase 00 (repository foundation and build baseline) is complete — keep the existing one-sentence summary of the package/toolchain from the old text (package `rust-youtube-streamer-service`, edition 2021, Docker-based checks; link to Build Checks anchor as today).
2. Phase 01 application foundation is implemented on top of it:
   - typed JSON configuration module (`src/config/`) with load-and-validate semantics and the committed template `config/config.example.json`;
   - two CLI commands `auth` and `run` plus the `--config <path>` option and default path `config/config.json` (see [Commands and Configuration](#commands-and-configuration));
   - structured logging via `tracing`/`tracing-subscriber` to stderr, controlled by `RUST_LOG` (see [Logging (RUST_LOG)](#logging-rust_log)).
3. Placeholder statement (exact wording rule): **both `auth` and `run` are placeholders until later phases** — after configuration is loaded and validated, each prints a "not implemented yet" error and exits with code 3; neither contacts YouTube, requests credentials, creates broadcast resources, or starts FFmpeg.
4. FFmpeg sentence (mandatory, explicit, standalone): FFmpeg remains a **separately provided external prerequisite** — it is **not installed by this phase** and this phase does **not launch it**.
5. Container-only check caveat (carry-forward fact): build checks validate the build inside the Linux container only; there is **no native Windows/Linux runtime validation** yet.

Do not add tasks/swimlanes, dates, commit hashes, or anything not listed above. Do not delete the old "see Build Checks" link line if it fits paragraph (1).

### A.4 `## Commands and Configuration` — NEW section, placed between Current Status and Logging

Content contract (short section, ~15–20 lines):

- Introduce the two commands with one minimal usage block copied from the real usage text in `src/app/cli.rs` (show the `usage:` line only, or a short equivalent — do not paste the full 9-line usage text into README; the full text is available via `--help`):
  `<executable> [--config <path>] auth|run`
- State, as a compact list:
  - `auth` — runs the OAuth authorization flow (**not implemented yet**: prints an error and exits 3 after config validation).
  - `run` — starts the streaming runtime (**not implemented yet**: prints an error and exits 3 after config validation; never starts FFmpeg).
  - `--config <path>` — configuration file to load; accepted before or after the command; default `config/config.json` when absent.
  - `--help` — prints usage to stdout, exit 0.
  - Exit codes: 0 = help/success, 1 = configuration/logging startup failure, 2 = usage error, 3 = mode not implemented yet (present this mapping as a one-line list or inline sentence).
- Default path + setup pointer: the default configuration path is `config/config.json`. To create it, copy `config/config.example.json` (the committed example, safe non-secret values) to `config/config.json` and edit it — `config/config.json` is gitignored and must never be committed. Full field-by-field reference: **`docs/configuration.md`** (link it: [`docs/configuration.md`](docs/configuration.md)).
- FFmpeg reminder (one sentence, consistent with A.3(4)): FFmpeg is a separately provided external prerequisite; this phase never installs or launches it.

### A.5 `## Logging (RUST_LOG)` — NEW section

Short, operator-oriented (~12–18 lines). Exact facts (subsection-free flat list is fine):

- Logs go to **standard error** as structured events via `tracing`/`tracing-subscriber`.
- The level filter comes from the environment variable `RUST_LOG` (exact, case-sensitive name).
- **Default** is `info` when `RUST_LOG` is unset, empty, or whitespace-only — silently (no warning).
- An **invalid non-empty** `RUST_LOG` value additionally prints exactly one `warning:` line to stderr and then still falls back to `info`; a **valid** value (e.g. `debug`, `trace`) is honored verbatim (surrounding whitespace trimmed).
- Example (safe): `RUST_LOG=debug rust-youtube-streamer-service run`.
- This section satisfies the forward-reference in `src/app/logging.rs` ("Operator guidance for RUST_LOG lives in the project README") — do not link to internal sources files, keep it operator-facing only.

### A.6 `## Build Checks (Docker via Alpine VM)` — VERIFY only, do not rewrite

Read the section and confirm it still matches reality:
- The four checks (`cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`) and their order — still correct.
- The dev-checks script description, `FORCE_FAIL` hook, notes (named volume, `rust:1.82`, container-only caveat, MCP capture note) — still accurate.
- If nothing drifted, leave the section byte-identical. If only a wording drift in a fact (not a stylistic preference), stop and report back instead of editing.

---

## B. New documentation file: `docs/configuration.md`

Path is fixed: `docs/configuration.md` (created new; neither existing `docs/` file is touched). Target length **~120–150 lines**; since it will exceed 100 lines, it **must include a TOC** at the top (same flat link-list format and slug rules as A.2).

### B.1 Required section outline and order (exact)

1. `# Configuration` (H1) + one intro sentence: the service loads a JSON configuration file at startup; two live paths exist — the committed example template and the operator-owned real file.
2. `## Table of Contents` — links to all H2s below.
3. `## Configuration File Locations`
   - Default: `config/config.json` (relative to the directory the executable is launched from); overridden by `--config <path>`.
   - `config/config.json` is **gitignored** — never commit it, never put secrets in it expecting commit-review protection.
   - `config/config.example.json` is committed as a safe, non-secret starting template with `privacy_status` defaulting to `unlisted`.
   - The example is **never loaded implicitly**: absence of `--config` always resolves to `config/config.json`.
4. `## Getting Started (Create the Real Configuration)`
   - Steps: copy `config/config.example.json` → `config/config.json`; edit for environment; run the executable.
   - State explicitly: no secrets today, but treat the real file as private; keep it out of version control.
5. `## Field Reference` (one H3 subsection per group; mirror the exact schema of 0.3 — names, types, and per-field rules verbatim from the table; add the example value from `config/config.example.json` per field)
   - `### youtube.broadcast` — `title`, `description`, `privacy_status` (`private`|`public`|`unlisted`, case-sensitive; example uses `unlisted`).
   - `### video` — `width`, `height`, `fps`, `pixel_format` (`rgb24` only).
   - `### renderer` — `font`, `font_size`, `line_height`, the four margins, `text_color`, `background_color`; include the vertical-layout rule `top margin + bottom margin + line_height ≤ video.height`.
   - `### chat` — `log_file`.
   - `### ffmpeg` — `executable`, `video_codec`, `preset`, `bitrate`.
   - Per-field notes only where justified (e.g. "existence of the font/FFmpeg file is checked by the consuming component in a later phase, not at configuration load").
6. `## Validation Rules`
   - Unknown fields are rejected at deserialize time.
   - Wrong value types rejected at deserialize time.
   - Collected semantic validation summarized as one short list (positives, `rgb24`, layout, privacy enum, non-blank strings) — mirror 0.3, no new rules.
   - On failure: one actionable error naming file/field; process exits **1**.
7. `## Command-Line Usage and Exit Codes`
   - `rust-youtube-streamer-service [--config <path>] <command>` with `auth` / `run` placeholder semantics (verbatim facts 0.1, including placeholder exit 3 and the do-not-overstate wording).
   - Exit code mapping 0/1/2/3.
   - This file is configuration-focused; command details stay minimal.
8. `## Logging (RUST_LOG)`
   - Brief repeat of 0.2 facts in 4–6 bullet lines; link back to the README section: `["Logging (RUST_LOG)"](../README.md#logging-rust_log)` for the operator guide (do not duplicate the full explanation).
9. `## See Also`
   - Link back to the README (path `../README.md`, anchor as written in B.8) and to `config/config.example.json` itself.

### B.2 Content rules for the file
- Source facts only from section 0 of this plan and the actual files; no invented values, no future promises beyond "later phases" phrasing.
- No secrets in examples (use only the existing example values).
- Never present `config/config.json` as committable.
- Placeholders wording: "not implemented yet" for both `auth` and `run`; never say "stub", "dummy", or imply streaming/auth works.
- No FFmpeg runtime-prerequisites guide; only the "separately provided external prerequisite, not installed/launched by this phase" sentence(s).
- No tables where a list is shorter than ~4 rows (keep the file near the length target).

---

## C. Exact wording rules (apply to every edit)

1. Both modes are **placeholders until later phases** — say "not implemented yet", exit 3.
2. **Do not claim** native Windows/Linux runtime validation anywhere; container checks only.
3. No secrets in any JSON examples; only values that already exist in `config/config.example.json`.
4. `config/config.json` is never committed and is gitignored; say so wherever the copy step appears.
5. FFmpeg: "separately provided external prerequisite; not installed by this phase; this phase does not launch it" — at least the README sentence (A.3(4)) and a pointer sentence in A.4.
6. Self-documenting; minimal comments/notes; no commented-out markdown; real newlines in file writes.
7. Prefer explicit names over "the tool", "the config file, etc." in fresh prose.

---

## D. Git and commit strategy

- Work happens on the existing branch `feat/phase01-config-app-skeleton`. No branch creation, no version bump (version 0.2.0 was committed in Task 1), no push.
- Before committing: run `git status` and `.gitignore` compliance check (nothing new is ignorable here; `docs/configuration.md` and README edits are trackable).
- Commits (exact messages, in this order):
  1. After README edits only are done: `docs: document phase 01 application foundation`
  2. After `docs/configuration.md` is added: `docs: add configuration guide`
- Stage only the intended file(s) per commit (`git add README.md` / `git add docs/configuration.md`). Never `git add -A` blindly.
- If any commit fails or hooks reject it, fix and create a new commit; do not amend.

## E. Verification steps (in order)

1. **Anchor sanity (manual)**: for every TOC entry in README, confirm the corresponding heading exists and the slug matches GitHub slug style via comparison with the pre-existing anchor `#build-checks-docker-via-alpine-vm` (which is known-good). Repeat for `docs/configuration.md`'s TOC. Fix mismatched slugs before committing.
2. **Line-count checks**: README keeps TOC and adds no more than the two new sections + reordered status (target ≤ ~185 lines); `docs/configuration.md` is within ~120–150 lines and > 100 lines (hence TOC).
3. **Fact audit (mandatory before commits)**: diff docs facts against this plan's section 0 — every stated default path, exit code, env var name (`RUST_LOG` exactly), default filter (`info`), pixel format (`rgb24`), privacy enum, layout rule, and the "example never loaded implicitly" statement must match.
4. **Rules audit**: no commented-out code, no `\n`-literal issues, no >200-line concern (markdown exempt), section ordering matches A.1.
5. **Final dev-checks gate (run once, at the end, after all commits of A/B)**:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — exit code 0 expected (docs do not affect Rust checks; the run is required by the TODO's standard workflow). Run through the Alpine VM MCP (`vm_status` first). Record actual result in the completion report.
6. Report completion signal: files changed, commits made, dev-checks exit code, and the docs/ discrepancy note (0.5).

## F. Explicitly OUT of scope for the implementer steps (do not touch)

1. `.agent/project-structure.md` — Planner-owned update after this task. Content requirements (for the Planner, not the implementer): add/attach `config/config.example.json` under the `config/` entry; ensure `src/app/` (with `cli.rs`, `error.rs`, `logging.rs`, `modes.rs`, tests) and `src/config/` (with `model.rs`, `error.rs`, `loader.rs`, `validation.rs`, `test_fixtures.rs`) plus `docs/configuration.md` are all reflected; keep the map's existing style and bullet widths.
2. `.agent/project-info/context.md` — Planner-owned update. Content requirements: Phase 01 implemented behavior (config module, CLI auth/run placeholders exit 3, tracing/RUST_LOG logging, exit codes 1/2/3, 46 tests), dependency choices (serde/serde_json/tracing/tracing-subscriber/tempfile, std-lib CLI — rationale), tests/dev-checks actual results (48→46 tests, all gates pass, container-only), remaining limitations (no OAuth, no runtime pipeline, no FFmpeg launch, no native runtime validation), next phase.
3. FFmpeg runtime-prerequisites guide — deferred, do not create.
4. Full README rewrite — forbidden; only the deltas in section A.
5. CI changes, `config/config.json` creation, any Rust dependency or code change — forbidden.

---

## G. High-level approach recap (for the Planner)

One pass of edits README-first then docs-file, two separate commits, verification before each commit, one final full dev-checks run. All facts are pre-derived in section 0 so the implementer never needs to re-derive a contract. Ambiguity trap avoided: docs/ is not empty (0.5) — noted, not assumed away.
