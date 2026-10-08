# Simplification Plan — Project Bootstrap Task 1 (Docs)

- Origin: STEP 4.3 (Simplification review) of `.agent/todos/20261008/20261008-todo-1.md`, Task 1 "Initialize project info".
- Scope reviewed: `.agent/project-info/product.md`, `context.md`, `architecture.md`, `tech.md` (cross-file redundancy, duplication, verbosity; traceability checked against `brief.md` §§ 2.1–2.3, 3.3–3.5, 4, 5, 6.4, 7.4, 8.4, 9.2, 10.3, 11.2, 13, 14.1–14.2).
- Severity: LOW. Prose/structure tightening only. No content that exists only here is lost. All edits are atomic — executable by the restricted implementer without judgment calls.
- Traceability audit result: every `brief §x.y` citation in the 4 docs was verified accurate. Do NOT change any citation not explicitly listed below.
- Not a candidate for big redesign — 5 small edits total.

## Findings

| ID | Issue | Files | Action |
| --- | --- | --- | --- |
| D1 | Transient status "not yet scaffolded" is triplicated; per `instructions.md`, volatile current state belongs in `context.md` only. The status sections in `architecture.md`/`tech.md` will go stale at first scaffolding commit (drift hazard). | architecture.md `## Status` (l. 78–80), tech.md `## Current Status` (l. 35–37), context.md (l. 5 — correct home, keep) | Edits 1–4 |
| D2 | Brief §2.3 future-evolution statement duplicated with divergent wording: product.md "Future Direction" (lists Chat Processing, Command Processing, Game State) vs architecture.md Data Flow l. 63 (lists only Command Processing and Game State). Canonical home: product.md + brief §2.3. | architecture.md l. 63 | Edit 3 |
| D4 | tech.md Configuration ends with two consecutive version-control-exclusion sentences; the second restates the first at broader scope. | tech.md l. 22 | Edit 5 |

## Edits (execute in order; each is one atomic replacement)

### Edit 1 — architecture.md: add stable pointer under Module Map

Replace (lines 19–21):

```text
## Module Map

| Path | Responsibility |
```

with:

```text
## Module Map

Target layout (brief §4). For current repo state, see `context.md`.

| Path | Responsibility |
```

Rationale: preserves the only non-transient fact from the `## Status` section ("brief §4 tree is the target") without carrying volatile state.

### Edit 2 — architecture.md: remove Status section

Delete the final section, i.e. lines 78–80:

```text
## Status

Not yet scaffolded: `src/` is empty and no `Cargo.toml` exists. The folder layout will be created during bootstrap; the brief §4 tree is the target.
```

After deletion, the file must end with the `## Application Lifecycle` body (old line 76, the "Concurrent tasks coordinated…" paragraph) followed by exactly one newline — no trailing blank line. The fact removed here already exists in `context.md` ("Current Work Focus") and is restated there; nothing is lost.

### Edit 3 — architecture.md: drop duplicated §2.3 evolution sentence

Replace line 63:

```text
Summary: YouTube → Chat → ChatStore → RenderInput → Renderer → raw frames → FFmpeg stdin → YouTube. Per brief §2.3, future evolution may insert Command Processing and Game State between the chat store and the renderer input; those are not implemented in the MVP.
```

with:

```text
Summary: YouTube → Chat → ChatStore → RenderInput → Renderer → raw frames → FFmpeg stdin → YouTube.
```

Rationale: future direction is canonical in product.md "Future Direction" and brief §2.3; keeping one copy removes the divergent module-list wording risk.

### Edit 4 — tech.md: remove Current Status section

Delete the final section, i.e. lines 35–37:

```text
## Current Status

Project is NOT yet scaffolded: no `Cargo.toml`, no `Cargo.lock`, and `src/` is empty (only `.gitkeep`). Scaffolding occurs after bootstrap (TODO items 2–3 and later sessions).
```

After deletion, the file must end with the `run` mode bullet (old line 33) followed by exactly one newline — no trailing blank line. The removed state already lives in `context.md` (Current Work Focus + Notes) and its "Immediate Next Steps" already cover the scheduling clause.

### Edit 5 — tech.md: merge redundant version-control sentences + add citation

Replace line 22:

```text
JSON configuration is loaded at startup and validated before starting the broadcast; invalid configuration must result in a clear error and a non-zero exit code (brief §5). `config/config.example.json` is versioned; the real `config.json` must be excluded from version control. Secrets, OAuth credentials, and tokens are never committed to the repository.
```

with:

```text
JSON configuration is loaded at startup and validated before starting the broadcast; invalid configuration must result in a clear error and a non-zero exit code (brief §5). `config/config.example.json` is versioned; the real `config.json`, which carries secrets and OAuth credentials, must stay out of version control (brief §6.4).
```

Rationale: one sentence instead of two; token-exclusion detail remains traceable via the newly cited brief §6.4 ("Exclude credentials and tokens from version control"). architecture.md Boundary Rules keeps the separate never-logged rule (§6.4, §7.4) untouched — that is logging discipline, a different concern.

## Explicitly reviewed and KEPT as-is (do not touch)

- architecture.md verbatim §2.2 diagram: intentional, self-labeled reproduction; keeps agents from needing the 40 KB brief for the core flow.
- architecture.md Style principles list (from brief §2.1): architecture doctrine, correct home.
- google-youtube3 mention in both product.md Problem (why the gap exists) and tech.md Dependencies (dependency policy, canonical §3.3): one short sentence each, distinct purposes.
- FFmpeg stdin/frames mentions in architecture.md Boundary Rules vs tech.md External Processes: knowledge boundary vs process contract; distinct facets.
- product.md Core Objectives / Non-Goals and the product description close restatement of brief §17: derivative-by-design.
- context.md: correct home of transient state; no edits (it will be updated by the workflow's closing step anyway).

## Verification (implementer must run all)

1. Read both files fully after edits; confirm sections removed and Edit 1/3/5 replacements present verbatim.
2. Search "scaffolded" across `.agent/project-info/*.md` → must match `context.md` ONLY.
3. Search "brief §2.3" across `.agent/project-info/*.md` → must match `product.md` only (architecture.md no longer cites §2.3).
4. Confirm no trailing blank lines at EOF in architecture.md and tech.md.
5. `git status` / `git diff`: only `architecture.md` and `tech.md` modified; `product.md`, `context.md`, all other files untouched.

## Out of scope

- README.md, folder scaffold, `.gitkeep` files (TODO Tasks 2–3 / other plan tasks).
- Any content additions, re-wording beyond the exact replacements above.
