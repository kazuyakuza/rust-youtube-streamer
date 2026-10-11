# Fix Plan — Phase 02, Task 3 Documentation Review

Review scope: `.agent/project-structure.md`, `.agent/project-info/context.md`, `README.md`, `docs/configuration.md`, `.agent/project-info/tech.md` (commits `30dcdc4`, `051966d`, `70bec50`).

## Findings Requiring Fix

### 1. `README.md` line 50 — directional + grammar error

Current text:

> Phase 02 adds two independent src-level components on this foundation: a bounded in-memory chat store (`src/chat/`) that appends accepted messages to an independent append-only chat log and keeps only the currently visible messages in a FIFO queue, and an RGB24 text renderer (`src/renderer/`, ab_glyph-based) that turns complete visible multiline text into raw video frames — both implemented with typed errors and unit tests but not yet wired into the `run` pipeline; neither changes any CLI behavior or the placeholders below. The chat store's chat log (config `chat.log_file`) is created only when the (not yet wired) chat store component is used.

Problems:
- "neither ... or" is grammatically incorrect; should be "neither ... nor".
- "the placeholders below" is directionally wrong: the preserved placeholder paragraph is at line 48, **above** the new Phase 02 paragraph.

Fix: change the clause to "neither changes any CLI behavior nor the placeholders above".

Verbatim preservation: `README.md` line 48 (the auth/run placeholder paragraph) must remain untouched.

### 2. `.agent/project-structure.md` line 34 — inaccurate test-module attribute claim

Current renderer bullet closing line:

>   - `*_tests.rs` / `text_test_helpers.rs` - 19 unit tests (`#[cfg(test)] #[path]` in-tree test modules; font fixture `fonts/Exo2-Light.otf`)

Problem: the renderer test modules (`frame_tests`, `settings_tests`, `text_tests`, `text_test_helpers`) are declared in `src/renderer/mod.rs` with only `#[cfg(test)]`; they do **not** use `#[path]`. The `#[path]` attribute is used only in `src/chat/model.rs` and `src/chat/store.rs` for `model_tests.rs` / `store_tests.rs`.

Fix: remove the `#[path]` claim from the renderer bullet. Suggested wording:

>   - `*_tests.rs` / `text_test_helpers.rs` - 19 unit tests (`#[cfg(test)]` in-tree test modules; font fixture `fonts/Exo2-Light.otf`)

Verbatim preservation: all other sentences in the chat/renderer blocks, the fonts bullet, the Cargo.toml bullet, and the src/main.rs bullet must remain as drafted.

## Suggested Edit Steps

1. Edit `README.md` — replace "or the placeholders below" with "nor the placeholders above" in the Phase 02 Current Status paragraph (line 50). Do not alter line 48.
2. Edit `.agent/project-structure.md` — in the renderer bullet, replace "`#[cfg(test)] #[path]` in-tree test modules" with "`#[cfg(test)]` in-tree test modules". Do not change any other line in the file.
3. Re-verify: ensure line 48 of `README.md` and lines 99–100 of `docs/configuration.md` remain verbatim, and that no code/config files were modified.

## No Other Fixes Required

All other factual claims (16 chat + 19 renderer tests, 81 total, ab_glyph 0.2.28, gate log `20261011T184938Z.log`, epoch-second log-format deviation, right/bottom margin deviation, no wiring into `app::run`, no config/schema expansion) are verified against the tree.
