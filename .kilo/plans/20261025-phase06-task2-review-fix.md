# Phase 06, Task 2 — Code Review Fix Plan

> Review step: 4.3 (code-reviewer role)  
> Branch: `feat/phase06-youtube-live-chat`  
> HEAD: `ac7385f`  
> Gate: 429 passed / 0 failed

## Scope

This plan records required fixes for the Task 2 landed implementation in `ac7385f`. It does **not** cover Tasks 3–4. Findings are limited to deviations from:

- `.kilo/plans/20261025-phase06-task2-live-chat-client-mapping.md`
- `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` (D-1..D-14)
- The caller rulings applied during 4.2 (splits, `mod.rs` 200-line cap, mapping tests 11 kept, `all_error_variants()` amended to 17, visibility fixes).

## Verified (no fixes required)

1. **Plan shapes landed:** module names, struct/variant definitions, `MAX_CHAT_RESULTS = 200`, `ChatTextMessage` carrier, query-builder order, `parse_rfc3339_seconds`, `ChatDedupWindow` 512-window, error variants `ChatDisabled`/`ChatEnded`, classify arms including `forbidden → InsufficientPermissions`, `liveChatNotFound → ApiNotFound { resource: "live chat" }`, exact pinned Display lines.
2. **Secret-safety:** no token/header/raw-body/raw-response logging; Bearer-only wire; no `key=` parameter.
3. **Module boundary:** `src/youtube/` contains no `crate::chat`, `crate::renderer`, `crate::streaming`, or `crate::app` dependencies.
4. **Test rosters:** names and counts match the spec (models 6, mapping 11, dedup 3, resolution 6, wire checks 6, live_error +4, display +2).
5. **Offline-only:** tests use fakes and `127.0.0.1` loopback; no Google contact.
6. **Regression locks:** pre-existing `classify_api_error` arms untouched; demo/auth/streaming surfaces untouched (diff confined to `src/youtube/`).
7. **No unprescribed `#[allow(...)]` or commented-out code.**
8. **File/method line caps and max-args respected** except as noted below.

## Required fixes

### 1. `all_error_variants()` must cover all 17 `LiveApiError` variants

**File:** `src/youtube/live_error_display_tests.rs`

**Deviation:** `all_error_variants()` returns 16 variants and `expected_display_lines()` returns `[&'static str; 16]`. The enum has 17 variants; `WaitCancelled` is missing from the comprehensive roster. This contradicts the caller ruling that `all_error_variants()` be amended to 17.

**Fix:**

- Insert `LiveApiError::WaitCancelled` into `all_error_variants()` between `DestinationMissing` and `ChatDisabled` (matching enum order in `live_error/mod.rs`).
- Insert the pinned Display line `"the YouTube readiness wait was cancelled by the stop request"` at the same position in `expected_display_lines()`.
- Change the array type to `[&'static str; 17]`.
- Update the assertion in `every_variant_display_stable_and_secret_free` from `16` to `17`.

**Rationale:** The `WaitCancelled` Display contract is already pinned by the separate `wait_cancelled_display_is_stable_and_secret_free` test, but the comprehensive variant roster must include it to satisfy the "all variants" invariant and the caller-authorized 17-variant count.

### 2. Compound boolean condition in `validated_epoch_days`

**File:** `src/youtube/live_chat_rfc3339.rs`

**Deviation:** `validated_epoch_days` contains `if day < 1 || day > days_in_month(year, month)`, a two-section boolean condition in an `if` statement, violating `.kilo/rules/single-section-boolean-conditions.md`.

**Fix:** Replace the compound condition with a single-section expression. Acceptable options:

- Inline: `if !(1..=days_in_month(year, month)).contains(&day) { return None; }`
- Extract helper `is_valid_day(year, month, day) -> bool` returning the range-contains result, then `if !is_valid_day(year, month, day) { return None; }`.

Also extract `is_valid_month(month) -> bool` for `if !(1..=12).contains(&month)` to keep the function body composed of single-section calls.

### 3. Compound boolean condition in `month_start_day`

**File:** `src/youtube/live_chat_rfc3339.rs`

**Deviation:** `month_start_day` contains `if month > 2 && is_leap_year(year)`, a two-section boolean condition in an `if` statement.

**Fix:** Extract a helper:

```rust
fn february_offset_applies(month: u32, year: u32) -> bool {
    month > 2 && is_leap_year(year)
}
```

Then replace the `if` condition with `if february_offset_applies(month, year)`. This keeps the `if` statement's condition a single-section call.

## Minor observations (not required fixes)

- `split_validated_fraction` builds a compound boolean in a `let` binding; it is not in an `if`/`while` and is acceptably descriptive.
- `is_leap_year` returns a compound boolean as a function body; not in a control-flow statement.
- `page_marked_ended` returns a compound boolean as a function body; the function name describes the condition.

## Verification after fixes

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked live_chat`
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked live_error`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`
5. Full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`

## Ownership

- Fixes: implementer (next 4.2 cycle).
- Re-review: code-reviewer after fixes land.
