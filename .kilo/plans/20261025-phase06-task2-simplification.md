# Phase 06, Task 2 — Simplification Plan (step 4.3, code-simplifier role)

> Authority: TODO `.agent/todos/20261025/20261025-todo-2.md` — **Task 2 only.**
> Per-task plan (shapes to preserve): `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`.
> Design record (mandated decisions): `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (D-1..D-14).
> Repo state at authoring: branch `feat/phase06-youtube-live-chat`, HEAD `ac7385f`, gate 429 passed / 0 failed.
> Scope of this step: **reduce incidental duplication/complexity in Task 2's landed sources ONLY.** No contract/Display/wire/query-shape/test-name changes; no pre-existing-file edits; no production behavior change.
> Max-3-cycles budget: this plan is **ONE** simplification (S-1). Remaining budget: 2.

## 0. Relationship to the sibling review-fix plan (coordination — NOT a duplicate)

A code-reviewer 4.3 artifact exists at `.kilo/plans/20261025-phase06-task2-review-fix.md`. Its required fixes are **rule-compliance / spec deviations** in two files: `src/youtube/live_error_display_tests.rs` (variant-count 16→17) and `src/youtube/live_chat_rfc3339.rs` (two compound-boolean `if` conditions). Those are **not** simplification work and are **out of this plan's scope**.

- File overlap with this plan: **NONE** (this plan touches `src/youtube/live_chat_mapping.rs` only).
- Ordering: apply the two plans independently; they are disjoint files, so order is irrelevant. Do NOT merge or re-derive the reviewer's fixes here.

## 1. Finding evidence (what was reviewed and the verdict)

Reviewed the full 22-file `ac7385f` surface. Classification:

| Item | Verdict | Reason |
| --- | --- | --- |
| `live_chat_models.rs` query builders, `MAX_CHAT_RESULTS`, serde structs | KEEP | Wire shape pinned by plan §1.1/§3.1 + wire roster |
| `api_chat_transport.rs` trait | KEEP | Seam shape pinned §3.3 |
| `google_live_chat_transport.rs` concrete impl | KEEP | Reuses shared engine; no duplication found |
| `live_chat_resolution.rs` thin `resolve_live_chat_id` wrapper + cfg-split core | KEEP | Mandated D-I zero-sleep precedent (§3.6); wrapper is the production entry |
| `live_chat_rfc3339.rs` helpers | KEEP (simplification-wise) | Mandated D-11 hand-rolled parser; functions tight, depth ≤ 2. Compound-boolean `if`s are the REVIEWER's fix (#2/#3), not simplification |
| `live_chat_dedup.rs` `.iter().any(\|s\| s == id)` | KEEP | Allocation-avoiding; `contains(&id.to_string())` would allocate — current is the simpler/cheaper form |
| `mod.rs` staging allows + re-exports | KEEP | Mandated §3.8/§3.9; `parse_rfc3339_seconds` re-export is Task-3's pinned consumption surface (record §4) — do NOT remove |
| `google_live_api_transport.rs` / `broadcast_resource.rs` visibility widenings | KEEP | Pre-existing files; `pub(super)`/`pub(crate)` widenings are §3.5 compiler-arbitrated, zero behavior |
| Test rosters names/bodies | KEEP | Names + pinned assertions mandated §4/§5; bodies are spec-faithful, not incidental duplication |
| `live_chat_mapping.rs` `has_message_text` + re-traversal in `map_record` | **SIMPLIFY (S-1)** | Redundant double Option-traversal + helper duplicating `non_blank_text` — see §2 |

**Considered-and-rejected (cross-roster fixture fold):** the canned text-record JSON in `live_chat_mapping_tests.rs::text_message_record()` and the inline item in `live_chat_wire_checks_tests.rs::chat_list_response_decodes_page_fields` look similar. Rejected: different rosters, different types (`serde_json::Value` vs raw string body), different purposes (mapping vs wire-decode), and plan §3.10 keeps the mapping roster self-contained. Folding them would couple rosters and touch pinned bodies — a net complexity increase.

## 2. Simplification S-1 — fold the redundant text check + re-clone into one helper

**File:** `src/youtube/live_chat_mapping.rs` (the ONLY file this plan edits).

**Problem (exact, current lines 54–91):** `map_record` runs `if !has_message_text(snippet) { return None; }` (which walks `text_message_details.as_ref()` → `message_text.as_ref()` → `!trim().is_empty()`), and then on the very next statements walks the SAME `Option` chain again to `.clone()` the value. Separately, `has_message_text` re-implements the same "Some non-blank `Option<String>`" extraction that `non_blank_text` already provides for author resolution. Result: one traversal too many and a duplicated predicate.

### Step 1 — rewrite `map_record` (replace the current `fn map_record` body, lines 54–76)

Replace:

```rust
/// D-4 steps 1–4 in order; `None` = skipped by policy (never ordinary chat).
fn map_record(record: &LiveChatMessageRecord) -> Option<ChatTextMessage> {
    let snippet = record.snippet.as_ref()?;
    if !is_text_message(snippet) {
        return None;
    }
    if !has_message_text(snippet) {
        return None;
    }
    let text = snippet
        .text_message_details
        .as_ref()
        .and_then(|details| details.message_text.clone())?;
    let author = resolve_author(record)?;
    Some(ChatTextMessage {
        author,
        text,
        message_id: record.id.clone(),
        published_epoch: snippet
            .published_at
            .as_deref()
            .and_then(parse_rfc3339_seconds),
    })
}
```

with:

```rust
/// D-4 steps 1–4 in order; `None` = skipped by policy (never ordinary chat).
fn map_record(record: &LiveChatMessageRecord) -> Option<ChatTextMessage> {
    let snippet = record.snippet.as_ref()?;
    if !is_text_message(snippet) {
        return None;
    }
    let text = non_blank_message_text(snippet)?;
    let author = resolve_author(record)?;
    Some(ChatTextMessage {
        author,
        text,
        message_id: record.id.clone(),
        published_epoch: snippet
            .published_at
            .as_deref()
            .and_then(parse_rfc3339_seconds),
    })
}
```

### Step 2 — replace the `has_message_text` function (current lines 82–91) in place

Replace:

```rust
/// Blank/missing text is a malformed payload — skipped, never ordinary chat.
fn has_message_text(snippet: &LiveChatMessageSnippet) -> bool {
    let Some(details) = snippet.text_message_details.as_ref() else {
        return false;
    };
    let Some(text) = details.message_text.as_ref() else {
        return false;
    };
    !text.trim().is_empty()
}
```

with:

```rust
/// Blank/missing text is a malformed payload — skipped, never ordinary chat;
/// the accepted text is preserved verbatim (untrimmed).
fn non_blank_message_text(snippet: &LiveChatMessageSnippet) -> Option<String> {
    snippet
        .text_message_details
        .as_ref()
        .and_then(|details| non_blank_text(&details.message_text))
}
```

No other lines change. `is_text_message`, `resolve_author`, and `non_blank_text` (lines 93–105) stay exactly as they are.

### Behavior-preservation proof (every input yields the identical result)

| `snippet` shape | Old result | New result |
| --- | --- | --- |
| `text_message_details == None` | `has_message_text` false → skip | `.as_ref()` → `None` → `?` → skip |
| details present, `message_text == None` | false → skip | `non_blank_text(&None)` → `None` → skip |
| `message_text == Some("")` / `Some("   ")` | `!trim().is_empty()` false → skip | `non_blank_text` trims empty → `None` → skip |
| `message_text == Some("hello")` | passes; `text = "hello".clone()` (raw) | `non_blank_text` → `Some("hello")` (raw, untrimmed) |

The accepted `text` value is the **untrimmed** raw string in both paths (matching the current `message_text.clone()`), so `ChatTextMessage.text` is identical; skip decisions are identical; D-4 step-2 policy is unchanged. Author resolution still runs after text. The mapping tests (`blank_or_missing_message_text_is_skipped`, `malformed_text_message_payload_is_skipped_not_mapped_as_ordinary`, `text_message_maps_full_shape_with_id_and_epoch_timestamp`) still pass unchanged.

### Constraints honored

- No production contract / carrier struct / Display line / wire shape / query order change.
- No test added, renamed, removed, or reworded → **gate count stays 429**.
- No pre-existing file touched; only `live_chat_mapping.rs`.
- Self-documenting-code + single-section-boolean rules preserved (`is_text_message` guard unchanged; new helper returns `Option` via `?`, no compound `if`).
- `src/youtube/` stays free of `crate::chat`.

## 3. Implementer recipe (junior, 50% restriction — no judgment calls)

1. Open `src/youtube/live_chat_mapping.rs`.
2. Apply Step 1 (replace the `map_record` function block verbatim with the "with:" block above).
3. Apply Step 2 (replace the `has_message_text` function block in place with the `non_blank_message_text` block above).
4. Do NOT change any other line in the file or any other file. Do NOT touch the sibling review-fix plan's files.

## 4. Verification (implementer runs, all via the Alpine VM / docker per plan §6)

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked live_chat`
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`
4. Full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect `ALL CHECKS PASSED`, observed count **429 / 0** (unchanged).
5. Regression guard: `git diff --stat` shows `src/youtube/live_chat_mapping.rs` only (plus whatever the sibling review-fix plan changed separately); no `allow(` additions; no Display/wire line edits.

## 5. Not done here (out of the simplifier's remit)

- Applying edits, committing, or pushing — the implementer (next 4.2 cycle) owns those; this step only produced the plan.
- The reviewer's rule fixes (variant-count 17; rfc3339 compound booleans) — owned by `.kilo/plans/20261025-phase06-task2-review-fix.md`.
- Any Task-3 consumption re-pick or removal of the `parse_rfc3339_seconds` re-export (plan-pinned surface).
- Tasks 3–4 scope.

## 6. Footer

Plan: `.kilo/plans/20261025-phase06-task2-simplification.md`
Produced 2026-10-26 by the Task-2 step-4.3 code-simplifier pass.
One simplification (S-1); within max-3-cycles.
