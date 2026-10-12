# Implementation Plan — Phase 03, TODO Task 4: Production `run` Mode Boundary

- **Source task**: `.agent/todos/20261011/20261011-todo-1.md` — `### 4. Production run Mode Boundary` (Pattern C, tasks 1–5; this plan covers ONLY Task 4)
- **Global plan**: `.kilo/plans/20261011-phase03-ffmpeg-local-pipeline.md` — decision **D8** governs here
- **Workflow sub-step**: 4.1b (Analysis & Planning, architector). Implementer executes §3–§6 of this plan in 4.2; review in 4.3; docs in 4.4; adherence in 4.5b; `[DONE]` in 4.6
- **Branch**: `feat/phase03-ffmpeg-local-pipeline` (already created in step 2 — implementer must NOT create/switch branches)
- **Version**: already `0.6.0` after step 3/en route — no bump in this task
- **Branch tree state verified while planning**: clean (`git status --porcelain` empty)

## 1. Scope and hard guard rails

**In scope (all of it, exactly):**
1. Replace the text of the `run` placeholder stderr message in `src/app/modes.rs` (handler body stays: one `error:` line on stderr, return `NOT_IMPLEMENTED_EXIT`).
2. Extract that message into a private named constant so its exact content is unit-testable.
3. Update the `run` handler's doc comment and the `modes.rs` module header accordingly.
4. Align two doc-only stale references to Task 4/placeholder state: `src/app/mod.rs` module header ("auth stays a placeholder" → "auth and run stay placeholders") and `src/app/pipeline/source.rs` error-channel doc sentence (the "TODO Task 4 may widen this error type" evolution-point sentence).
5. Two new unit tests in `modes.rs` (exact message pin; non-interactive/demo-pointer assertions).
6. One VM smoke run observing the new message + exit code 3 via the Alpine VM MCP (recorded by the implementer in its 4.2 completion summary).
7. Standard dev-checks gate via the Alpine VM MCP.

**Explicitly NOT in Task 4 (any of these is a plan deviation):**
- No real OAuth, broadcast lifecycle, RTMPS/ingestion, or YouTube chat implementation.
- No STD input read, keyboard menu, hotkeys, or typed-message entry anywhere — `run` stays non-interactive and non-demo.
- No demo substitution: `run` must NOT call into `src/app/demo/*` or the pipeline (it returns before any pipeline construction).
- No `src/app/cli.rs` change of any kind (parser, `USAGE`, `cli_tests.rs` untouched).
- No `src/app/logging.rs` change (three-event whitelist: `command`, `config_path`, `exit_code` — respected because `run` adds no tracing events).
- No `dispatch` change (its `run` arm keeps the `modes::run(app_config)` signature).
- No `auth` handler/message change — byte-identical to today.
- No README/docs/`,md` project docs, `.agent/project-structure.md`, or completion report edits (Task 5).
- No new dependency, no `Cargo.toml`/`Cargo.lock` change (no dependency is warranted for a message change).
- No `#[allow(dead_code)]` additions or cfg-test gate changes.
- No git push (restricted to step 5 of the Critical Workflow).

## 2. Verified baseline (live, on `feat/phase03-ffmpeg-local-pipeline`)

- `src/app/modes.rs` (62 lines): `NOT_IMPLEMENTED_EXIT = 3`; `run(_config)` currently prints
  `error: runtime pipeline is not implemented yet: the YouTube/FFmpeg pipeline has not been built; no YouTube resources were created and FFmpeg was not started`
  and returns 3. Tests: `auth_reports_not_implemented_exit_code`, `run_reports_not_implemented_exit_code`, `exit_codes_remain_distinct_and_stable`.
- `src/app/mod.rs` line 9 (module doc): `...mode not implemented yet (`auth` stays a placeholder)...` — stale after this task; run also stays a placeholder.
- `src/app/pipeline/source.rs` lines 14–15, the exact live wording of the reserved evolution point:
  `The only failure class available in this phase is the chat-domain error; TODO Task 4 may widen this error type when a real network listener exists (documented evolution point).`
  Task 4 does NOT widen it: `run` stays an exit-3 placeholder and never reaches the pipeline, so no widening path exists yet.
- `src/app/cli.rs` `USAGE`: `run                  start the streaming runtime (not implemented yet)` — still literally true after this task (run remains unimplemented; only the failure message is clarified). **Decision: the line stays unchanged.**
- `src/app/error.rs`: `NOT_IMPLEMENTED_PLACEHOLDER` contract lives as exit code 3 via `modes::NOT_IMPLEMENTED_EXIT`; no `>not implemented` mapping lives in `StartupError` (placeholder code is produced by the mode handlers, not the error type) — nothing to change.
- Reuse analysis for TODO Task-4 bullet "structure reusable local frame-pipeline orchestration": **already satisfied by Task 2** — `MessageSource` (source.rs) is the chat-listener seam; `runner.rs::prepared_pipeline` treats output via `OutputKind` (`Demo`/`Session` prefixes; future RTMPS = a new `OutputKind`). No restructuring is required in Task 4. Only the doc-comment alignment (§3.2) is needed.
- Test suite baseline: 136 tests green (Task 3); this plan adds 2 → expected 138.
- `docs/configuration.md` line 100 quotes the fragment `error: runtime pipeline is not implemented yet`. The new message (§3.1) **keeps that exact stem**, so the doc stays literally true through Task 4; Task 5 may refine wording.

## 3. Architecture decisions (all locked; implementer does not re-decide)

### D-T4-1 — `run` handler behavior and exit code
- Signature unchanged: `pub(super) fn run(_config: &AppConfig) -> u8`.
- Exit code stays `NOT_IMPLEMENTED_EXIT` (3). Never 0, never 1, never demo behavior. This preserves the Phase-01 exit-code contract (3 = selected mode not implemented yet) and D8.
- The handler remains non-interactive by construction: no stdin, no keyboard, no prompt; it does not call `src/app/demo/*` or the pipeline — `_config` is used only to prove config validation happened before the handler runs (ordering contract, unchanged).

### D-T4-2 — Exact new message (single stderr line; the stem is preserved from the current message for doc consistency)
The full emitted line (exactly one line, including the `error: ` prefix):

```
error: runtime pipeline is not implemented yet: YouTube streaming needs OAuth authorization, broadcast lifecycle, RTMPS ingestion URL acquisition and YouTube live chat integration, none of which is built; the run mode never reads input, never runs the local demo pipeline, and will start the YouTube/FFmpeg pipeline only in a later phase. There is no local fallback: for a local demonstration recording use the demo command
```

Requirement map (the line has no trailing period, matching the existing placeholder-message style of the `auth` line):
- names the four not-yet-built prerequisites: `OAuth authorization` ✔, `broadcast lifecycle` ✔, `RTMPS ingestion URL acquisition` ✔, `YouTube live chat integration` ✔
- states the future intent: `the run mode ... will start the YouTube/FFmpeg pipeline only in a later phase` ✔
- explicitly non-interactive: `never reads input` ✔
- explicitly no demo substitution + no local fallback: `never runs the local demo pipeline` and `There is no local fallback: for a local demonstration recording use the demo command.` ✔

### D-T4-3 — Constant + handler shape in `src/app/modes.rs`
Add directly above the `run` handler (private; prefer-private-members rule):

```rust
/// The single stderr line the `run` placeholder reports: the four
/// not-yet-built YouTube prerequisites, the non-interactive boundary,
/// the later-phase intent, and the explicit no-local-fallback pointer to
/// the `demo` command.
const RUN_NOT_IMPLEMENTED_MESSAGE: &str = "runtime pipeline is not implemented yet: \
YouTube streaming needs OAuth authorization, broadcast lifecycle, RTMPS ingestion \
URL acquisition and YouTube live chat integration, none of which is built; the run \
mode never reads input, never runs the local demo pipeline, and will start the \
YouTube/FFmpeg pipeline only in a later phase. There is no local fallback: for a \
local demonstration recording use the demo command";
```

Continuation rules (Rust strips the newline AND the next line's leading whitespace at `\`): every broken line above ends with exactly one space before the backslash. Verify by reading the test in §5.1 — it asserts the full single-line string, so any splice error fails the test.

New handler body (replaces the current `eprintln!` inline string; doc comment replaced too):

```rust
/// `run` placeholder until a later phase: the configuration is already
/// loaded and validated when this runs (the parameter enforces that
/// ordering), the handler reports the not-yet-built YouTube
/// prerequisites (OAuth, broadcast lifecycle, RTMPS ingestion URL
/// acquisition, YouTube live chat) on standard error, states that the
/// YouTube/FFmpeg pipeline starts in a later phase with no local
/// fallback, and returns `NOT_IMPLEMENTED_EXIT` (3). It never reads
/// input, never runs the local demo pipeline, never creates YouTube
/// resources, and never starts FFmpeg.
pub(super) fn run(_config: &AppConfig) -> u8 {
    eprintln!("error: {RUN_NOT_IMPLEMENTED_MESSAGE}");
    NOT_IMPLEMENTED_EXIT
}
```

### D-T4-4 — Module doc updates (doc-comment only, no behavior)
1. `src/app/modes.rs` first sentence: `Placeholder handlers for the two execution modes.` → `Placeholder handlers for the `auth` and `run` execution modes.` (prevents misreading now that `demo` exists).
2. `src/app/mod.rs` module doc, the parenthetical: `(`auth` stays a placeholder)` → `(`auth` and `run` stay placeholders)`.
3. `src/app/pipeline/source.rs` — replace the three reserved-sentence doc lines of `next_message` with the documented widening decision:

```rust
    /// Returns the next message due now, or `None` when nothing is due.
    /// The only failure class available in this phase is the chat-domain
    /// error; the seam now faces the production run mode, which stays a
    /// not-implemented placeholder in TODO Task 4, so no widening
    /// happens yet; the widening point is reserved for the later phase
    /// that introduces a real YouTube chat listener (documented
    /// evolution point).
    fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError>;
```

(Keeps the `documented evolution point` marker; removes the now-stale attribution of the widening to Task 4 itself. The trait-level sentence above — demo feeds a schedule, a future production run mode feeds live chat without changing this loop — stays verbatim: still accurate.)

### D-T4-5 — `dispatch`, CLI, logging, help: unchanged
- `dispatch`'s run arm stays `cli::Mode::Run => modes::run(app_config)`.
- Logging: `run` adds no tracing events; the three-event whitelist and fields stay intact.
- CLI: `run` was already accepted in Phase 01/Task-3 parser; nothing changes.
- Help line `run                  start the streaming runtime (not implemented yet)` stays — verified still true after this change (run remains an unimplemented placeholder with a clarified failure message).

## 4. Files changed / not changed (atomic list)

Changed (commit 1): `src/app/modes.rs` (constant, handler doc+body, module header first sentence, 2 new tests), `src/app/mod.rs` (one doc phrase), `src/app/pipeline/source.rs` (four doc lines).
Not changed: every other file, including `src/app/cli.rs`, `src/app/cli_tests.rs`, `src/app/logging.rs`, `src/app/error.rs`, `src/app/demo/**`, `src/app/pipeline/{runner,clock,pacing,interrupt,error}.rs`, `src/chat/**`, `src/renderer/**`, `src/streaming/**`, `src/config/**`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `.gitignore`.

Size rules check after change: `modes.rs` ≈ 95 lines (≤200 ✔; handler body 2 lines ✔; tests ≤ 50 ✔; no nesting beyond depth 2 ✔). `source.rs` ≈ 19 lines ✔. `app/mod.rs` doc-only ✔.

## 5. Tests (new; increments 136 → 138)

Add to the existing `#[cfg(test)] mod tests` in `src/app/modes.rs` (keep the three existing tests byte-identical):

```rust
    #[test]
    fn run_placeholder_message_names_the_four_missing_prerequisites() {
        assert_eq!(
            RUN_NOT_IMPLEMENTED_MESSAGE,
            "runtime pipeline is not implemented yet: YouTube streaming needs \
             OAuth authorization, broadcast lifecycle, RTMPS ingestion URL \
             acquisition and YouTube live chat integration, none of which is \
             built; the run mode never reads input, never runs the local demo \
             pipeline, and will start the YouTube/FFmpeg pipeline only in a \
             later phase. There is no local fallback: for a local \
             demonstration recording use the demo command"
        );
    }

    #[test]
    fn run_placeholder_message_denies_interactive_input_and_points_to_demo() {
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("never reads input"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("never runs the local demo pipeline"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("no local fallback"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("use the demo command"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("stdin"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("keyboard"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("press"));
    }
```

Rationale for what is and is not testable: unit tests cannot prove the process does not read stdin at runtime or offer interactivity; the exact-string and contains/negative-contains assertions are the code-testable boundary guarantees (both derive from D-T4-2). The runtime proof is the §6 smoke run, recorded by the implementer as instructed there. No subprocess-based stderr-capture test is added (overkill for a placeholder line; the smoke run covers the real stderr).

## 6. Verification and commands (Alpine VM MCP; implementer records evidence)

Always call `alpine-vm_vm_status` before the first `alpine-vm_vm_run_command`. All verification stays container-only; no FFmpeg needed (run never starts it); no build artifacts in repo root (named-volume redirect).

1. **Standard gate** (after commit 1, and again on the final tree after commit 2):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   EXPECT: exit 0; PASS lines `fmt-check`, `check`, `test`, `clippy`; `running 138 tests` → `138 passed; 0 failed; 0 ignored`; summary `ALL CHECKS PASSED`.
2. **Run-mode smoke** (observes the new message + exit code 3 end-to-end; one docker invocation keeps setup+teardown atomic — `config/config.json` is gitignored, so the tree stays clean):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c 'cp config/config.example.json config/config.json && cargo build --locked && "$CARGO_TARGET_DIR/debug/rust-youtube-streamer-service" run; status=$?; rm -f config/config.json; echo "run_exit_code=$status"'`
   EXPECT:
   - stderr contains the three whitelisted info events (`mode selected`, `configuration loaded and validated`, `mode finished` with `exit_code=3`) and exactly one line starting `error: runtime pipeline is not implemented yet: YouTube streaming needs OAuth authorization, broadcast lifecycle, RTMPS ingestion URL acquisition and YouTube live chat integration,...` — matches §3.2 verbatim;
   - `run_exit_code=3`;
   - no `demo recording finalized` output, no FFmpeg activity.
   Record the observed line and code in the 4.2 completion summary. If the example config unexpectedly fails validation (exit 1) do NOT workaround — stop and report to the caller.
3. **Post-smoke hygiene**: on the host, `git status --porcelain` must be empty after the smoke (`/config/config.json` removed inside the docker call; per Gitignore Compliance Rule verify nothing else appeared).

## 7. Commit plan (locked; exactly two commits for this task)

1. `feat: clarify the run mode production boundary` — `src/app/modes.rs`, `src/app/mod.rs`, `src/app/pipeline/source.rs`.
2. `docs: track task 4 planning documents` — `.kilo/plans/20261011-phase03-task4-run-boundary.md` (this plan; later the 4.5b adherence file joins the same tracked location). **Canonical place decision**: Task 4 follows the Phase-03 precedent (Task 1/2/3) of a dedicated `docs: track task N planning documents` commit rather than folding the plan file into 4.6.
   4.6's own commit (the TODO `[DONE]` marker) is a separate, later commit owned by the Critical Workflow — not part of this plan's commit list.

## 8. Acceptance-criteria mapping (Task-4-relevant items of the TODO)

| TODO acceptance item | Where satisfied |
| --- | --- |
| `run` does not become an interactive local demo | §1 scope ban, §3.1 handler shape (non-interactive by construction), §5.1 negative assertions, §6.2 smoke (no demo output) |
| `run` does not falsely claim YouTube streaming is available | §3.2 message says `not implemented yet` + `only in a later phase` + no local fallback; exit code 3; §5.1 exact-string pin |
| `run` must fail clearly before pretending to stream to YouTube | §3.1/§3.2 — failure with explicit prerequisites and no streaming attempt (no pipeline call), D8 |
| Reusable local frame-pipeline orchestration for a future production mode | Verified already-built in Task 2 (`MessageSource` seam, `prepared_pipeline`, `OutputKind` separates demo- from session-file output; future RTMPS = later/new `OutputKind`); doc-comment alignment in §3.2/§D-T4-4 keeps the seam's contract accurate |

## 9. Review checklist for 4.3 (code-reviewer / code-simplifier)

- Message constant matches §3.2 verbatim (test enforces); stem preserved.
- `auth` handler and message byte-identical to baseline.
- No file outside §4's list changed (`git show --stat` per commit).
- No tracing events added by `run`; no stdin/pipeline/demo call in the handler.
- Doc comments carry no commented-out code; file/method rules hold.
- Gate green with 138 tests; two commits, correct messages, nothing pushed.
