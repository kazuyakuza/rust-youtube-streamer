# Plan — Phase 01, Task 4 Simplification Pass (docs + metadata)

Source: 4.3 simplification step over the Task 4 artifacts on branch `feat/phase01-config-app-skeleton`
(HEAD `635a044`, plus unstaged planner edits to `.agent/project-structure.md` and `.agent/project-info/context.md`).
Baseline plan: `.kilo/plans/20261009-task4-docs-metadata.md` (its "verified facts" section 0 remains binding).
Assignee: implementer (junior, 50% restriction). Every step below is atomic and fully specified: copy the exact
before-string, replace with the exact after-string. No judgment calls, no other edits.

SCOPE GUARDRAILS (apply to all steps):
- Only these four files change: `README.md`, `docs/configuration.md`, `.agent/project-structure.md`, `.agent/project-info/context.md`.
- Do NOT touch: `docs/how-to-set-up-git.md`, `docs/how-to-write-todo-files.md`, any Rust/config/scripts file,
  the README footer italic line (`*Initial README — ...*`), the README `## Build Checks (Docker via Alpine VM)` section,
  any heading text (all TOC anchors therefore stay valid), or any baseline-plan-mandated sentence listed under
  "Reviewed, keep as-is".
- No new facts. Every removal below leaves the same fact stated canonically elsewhere in the same file or at its
  planned canonical location. Locate strings by content, not by line number (line numbers are guidance only).

---

## 1. README.md (currently 182 lines)

### R1 — De-duplicate placeholder semantics in the commands list (lines 60–61)
The canonical placeholder statement ("not implemented yet" + exit 3 + after config validation) is already
line 46 (Current Status) and the exit-code map (line 64, same section). The two bullets restate it verbatim.

Before:
```
- `auth` — runs the OAuth authorization flow (**not implemented yet**: prints an error and exits 3 after config validation).
- `run` — starts the streaming runtime (**not implemented yet**: prints an error and exits 3 after config validation; never starts FFmpeg).
```
After:
```
- `auth` — runs the OAuth authorization flow (**not implemented yet**).
- `run` — starts the streaming runtime (**not implemented yet**; never starts FFmpeg).
```
Rationale: single canonical statement (line 46) + exit-code map (line 64); the per-command "never starts FFmpeg"
safety detail stays on `run`. This is the one approved deviation from the baseline plan's A.4 bullet wording;
facts are unchanged, only the triple restatement is reduced.

### R2 — Drop the duplicated default-path sentence (line 66)
The `--config` bullet directly above (line 62) already states the default `config/config.json` canonically.

Before:
```
The default configuration path is `config/config.json`. To create it, copy `config/config.example.json` (the committed example, safe non-secret values) to `config/config.json` and edit it — `config/config.json` is gitignored and must never be committed. Full field-by-field reference: [`docs/configuration.md`](docs/configuration.md).
```
After:
```
Create the default `config/config.json` by copying `config/config.example.json` (the committed example, safe non-secret values) and editing it — `config/config.json` is gitignored and must never be committed. Full field-by-field reference: [`docs/configuration.md`](docs/configuration.md).
```

---

## 2. docs/configuration.md (currently 118 lines; after edits ≈114 — still >100, TOC stays)

### C1 — Trim "at most once" duplication (line 21)
Canonical CLI contract ("before or after the command, at most once") stays in Command-Line Usage (line 102).

Before:
```
- Only one configuration file is used per run: the default path or the single `--config <path>` value, which may be given at most once.
```
After:
```
- Only one configuration file is used per run: the default path or a single `--config <path>` value.
```

### C2 — Trim gitignore/commit-review re-duplication (line 36)
Canonical statement is the Configuration File Locations bullet (line 18). The after-text matches the baseline
plan's required B.4 wording ("no secrets today, treat as private, keep out of version control") exactly.

Before:
```
The example contains no secrets today, but treat the real `config/config.json` as private: it is gitignored, must never be committed, and carries no commit-review protection.
```
After:
```
The example contains no secrets today, but treat the real `config/config.json` as private and keep it out of version control.
```

### C3 — Drop unknown-fields clause duplicated from Validation Rules (line 40)
Canonical rule stays in Validation Rules (line 80: "Unknown fields are rejected at deserialize time.").

Before:
```
Section and field names must match exactly as written; unknown or misspelled fields are rejected.
```
After:
```
Section and field names must match exactly as written.
```

### C4 — Merge the two exit-code-1 bullets in Validation Rules (lines 88–89)
Before (two consecutive bullets):
```
- On failure: one actionable error names the configuration file and field, and the process exits with code 1.
- A missing or unreadable configuration file is also a configuration failure: reported and exited with code 1.
```
After (one bullet):
```
- On failure (including a missing or unreadable configuration file): one actionable error names the configuration file and field, and the process exits with code 1.
```

---

## 3. .agent/project-structure.md (unstaged planner edits — wording fix only)

### S1 — Precision fix in validation.rs bullet (line 16)
The 13 rules cover non-blank strings AND paths (titles, colors are not paths); the current text understates it.

Before:
```
non-blank paths, FFmpeg fields)
```
After (same line, only this fragment changes):
```
non-blank strings/paths, FFmpeg fields)
```

No other structure-map changes: verified against `git ls-files src/ config/ docs/` — every tracked file is
mirrored (`.gitkeep` placeholders are intentionally omitted throughout, consistent style).

---

## 4. .agent/project-info/context.md (unstaged planner edits — order + typo)

Apply X1 → X2 → X3 → X4 in this order. These are pure cut/paste of whole sections; zero content changes
except X1. Goal order (living log: chronological change log, then deviations, then next steps):

`Current Work Focus` → `Recent Changes (2026-10-08)` → `Recent Changes (2026-10-09)` →
`Recent Changes (2026-10-09, continued)` → `Recent Changes (2026-10-09, Phase 01 execution)` →
`Known Deviations and Limitations (Phase 01)` → `Immediate Next Steps` → `Scope Decisions` → `Notes`

### X1 — Typo at start of the Phase 01 execution paragraph (line 46)
Before: `.All Phase 01 work ran on branch`
After:  `All Phase 01 work ran on branch`

### X2 — Move section "Recent Changes (2026-10-09, continued)"
Cut the whole section — heading `## Recent Changes (2026-10-09, continued)` plus its two bullets
(`* Reviewed the updated ...` and `* Created \`.agent/todos/20261009/20261009-todo-2.md\` ...`) — from its
current position (between "Known Deviations" and "Scope Decisions") and paste it immediately AFTER the last
bullet of section "Recent Changes (2026-10-09)" (`* alpine-vm MCP output contract update landed ...`) and
BEFORE `## Immediate Next Steps`.

### X3 — Move section "Immediate Next Steps"
Cut the whole section — heading `## Immediate Next Steps` plus its four numbered items (`1. Execute ...`
through `4. Later phases replace ... stay stable).`) — from its current position and paste it immediately
AFTER the last bullet of `## Known Deviations and Limitations (Phase 01)` (`* The invalid-\`RUST_LOG\` fallback
design ... governs.`) and BEFORE `## Scope Decisions`.

### X4 — Normalize separators
After the moves, ensure exactly ONE blank line between every adjacent section across the file (removes the
pre-existing double blank line that was before the "continued" section, lines 63–64).

---

## 5. Reviewed, keep as-is (no change — do not "fix" these)

- README TOC: flat, one entry per H2, correct GitHub slugs; H3 sub-anchors deliberately excluded. Not over-granular.
- README double FFmpeg prerequisite sentence (lines 48 + 68) and container-only caveat (lines 50 + Build Checks
  Notes): mandated verbatim by the baseline plan (A.3(4)–(5), A.4, C.5, F-notes). Canonical by design.
- README line 46 placeholder statement: baseline-plan exact-wording rule. Keep.
- docs/configuration.md line 76 (font/FFmpeg existence note + external-prerequisite sentence): planned per-field
  note (B.5 example) and B.2 allowance. Keep.
- docs/configuration.md per-command "exits 3" (lines 100–101) + exit-code map (line 104): B.7 mandated verbatim. Keep.
- Heading granularity + TOC depth in both markdown docs: consistent H2 (+H3 field groups); no change.
- `docs/how-to-*.md`: out of scope, untouched.

## 6. Verification

- No per-step code verification (docs-only). Manual self-check after all edits: (a) no heading text changed,
  so all TOC anchors remain valid; (b) every removed fact is still present at the canonical location named in
  the step's rationale; (c) baseline plan section 0 facts (exit codes, `RUST_LOG`/`info`, `rgb24`, privacy enum,
  `config/config.json` default, "example never loaded implicitly", gitignore rule, FFmpeg prerequisite sentence)
  still appear in both docs.
- Phase-evidence gate — run ONCE at the end via the Alpine VM MCP (`vm_status` first), full suite:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  Expected exit 0 / `ALL CHECKS PASSED`. Record actual exit code in the completion report.

## 7. Commits (after gate passes; stage only listed files; never `git add -A`; no push)

1. `git add README.md docs/configuration.md` → commit message: `docs: simplify phase 01 documentation wording`
2. `git add .agent/project-structure.md .agent/project-info/context.md` → commit message: `docs: simplify structure map and context metadata`
- `git status` check before each commit (gitignore compliance; nothing new is ignorable here).
- Remaining workflow steps (mark Task 4 `[DONE]`, TODO rename, merge) belong to the caller — NOT in this plan.
