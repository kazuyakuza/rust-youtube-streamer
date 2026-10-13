# Task 3 Review Fix Plan

**Authority:** `.kilo/plans/20261012-phase04-task3-auth-command.md` §2 D6/D7  
**Scope:** exactly one rule-cap finding in `src/app/auth/listener.rs`. No behavior change; no other files touched.

## Finding: `write_callback_response` exceeds the max-arguments limit

| Item | Value |
| --- | --- |
| Severity | fix required |
| Rule | `.kilo/rules/max-arguments-per-method.md` |
| File | `src/app/auth/listener.rs` |
| Line | 190 |
| Current signature | `fn write_callback_response(stream: &mut TcpStream, status_line: &str, body: &str)` |
| Problem | 3 non-receiver parameters; the rule cap is 2. The plan D6 miscounted this as “2 params” (adaptation #7). |

## Required change

Encapsulate `status_line` and `body` in a private struct so the writer function takes exactly two parameters.

1. Add a private response descriptor struct near `CallbackOutcome` in `src/app/auth/listener.rs`:

```rust
struct CallbackResponse {
    status_line: &'static str,
    body: &'static str,
}
```

2. Change `outcome_response_parts` to return the struct:

```rust
fn outcome_response_parts(outcome: &CallbackOutcome) -> CallbackResponse {
    match outcome {
        CallbackOutcome::Authorized(_) => CallbackResponse {
            status_line: CALLBACK_ACCEPTED_STATUS,
            body: CALLBACK_ACCEPTED_BODY,
        },
        _ => CallbackResponse {
            status_line: CALLBACK_REJECTED_STATUS,
            body: CALLBACK_REJECTED_BODY,
        },
    }
}
```

3. Change `write_callback_response` to two parameters:

```rust
fn write_callback_response(stream: &mut TcpStream, response: CallbackResponse) {
    let content_length = response.body.len();
    let response_text = format!(
        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_line, content_length, response.body
    );
    let _ = stream.write_all(response_text.as_bytes());
}
```

4. Update the call site in `handle_next_connection`:

```rust
let response = outcome_response_parts(&outcome);
write_callback_response(&mut stream, response);
```

## Constraints preserved

- Response bodies remain the fixed constants `CALLBACK_ACCEPTED_BODY` / `CALLBACK_REJECTED_BODY`; nothing is echoed from the request.
- `write_callback_response` still ignores write failures (outcome already decided).
- No new public API, no new dependencies, no module-level `#[allow]`.

## Acceptance check

1. `git grep -n "fn write_callback_response" src/app/auth/listener.rs` shows exactly two parameters after the `&mut TcpStream`.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` exits 0.
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` exits 0.
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` exits 0 with 218 tests passed.
5. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` exits 0.
