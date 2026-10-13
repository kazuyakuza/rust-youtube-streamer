# Task 2 Review Fix Plan — Phase 04: OAuth Configuration and Credential Storage

- Date: 2026-10-12
- Branch: `feat/phase04-oauth-credentials`, HEAD `42d33fd`
- Reviewer finding: one minor test-shape deviation from the sanctioned plan.
- Authority: `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md` §5 + `.kilo/plans/20261012-phase04-task2-oauth-config-storage-amendment-2.md` A1.4.

---

## Finding 1 (low severity) — `oauth_paths_require_non_blank_values` does not verify aggregation

| Field | Value |
| --- | --- |
| Severity | Low |
| File | `src/config/validation_tests.rs` |
| Line | 122–130 |
| Category | Plan adherence / test coverage |

### Description

The sanctioned plan §5 specifies this test as:

> `validation_tests.rs::oauth_paths_require_non_blank_values` — fixture with both blank / whitespace-only paths → `Validation` error containing both field names in one aggregated Display.

The implemented test instead exercises each OAuth path separately:

```rust
#[test]
fn oauth_paths_require_non_blank_values() {
    assert_blank_value_rejected("oauth.client_config_path", |c| {
        c.oauth.client_config_path = "".to_string()
    });
    assert_blank_value_rejected("oauth.credentials_path", |c| {
        c.oauth.credentials_path = "   ".to_string()
    });
}
```

This verifies that each individual path is blank-checked and that the correct field name appears in its error, but it does **not** verify that both blank OAuth paths are reported together in a single aggregated `Validation` error. The generic aggregation mechanism is covered by `multiple_issues_reported_together`, but the plan explicitly required an OAuth-specific aggregation assertion.

### Why it matters

- The validation contract is that `validate()` collects **all** issues before returning.
- A future regression that short-circuits after the first OAuth blank issue would pass the current test but fail the planned assertion.
- The implementer is a junior developer under 50% restriction; deviations from the locked test plan should be corrected rather than silently accepted.

### Required change

Replace the body of `oauth_paths_require_non_blank_values` with a single fixture that blanks both paths and asserts both field names appear in one validation message:

```rust
#[test]
fn oauth_paths_require_non_blank_values() {
    let mut config = valid_config();
    config.oauth.client_config_path = "".to_string();
    config.oauth.credentials_path = "   ".to_string();
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("oauth.client_config_path"));
    assert!(message.contains("oauth.credentials_path"));
    assert!(message.contains("2 issue(s)"));
}
```

### Acceptance check

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked config` exits 0.
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked validation_tests::oauth_paths_require_non_blank_values` exits 0.
- The new assertion fails if either OAuth path is removed from the blank check or if issues are no longer aggregated.
- Total test count remains 195; no other file changes.

---

## Summary

- Total findings: 1 (low severity).
- No correctness, security, redaction, or build failures detected.
- All other Task 2 changes adhere to the plan + amendments and pass the container gate.
