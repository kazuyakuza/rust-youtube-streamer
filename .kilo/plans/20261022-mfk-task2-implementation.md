# Implementation Plan — Task 2: Per-Stream Made-for-Kids Config Property (CONDITIONAL)

Plan date: 2026-10-22 · Owner: Architector Agent (Critical Workflow §4.1b, Task 2 cycle)
TODO: `.agent/todos/20261022/20261022-todo-2.md` Task 2 · Global plan (BINDING, incl. Amendment 1): `.kilo/plans/20261022-run-streaming-diagnostics-mfk.md` § Encoded Technical Direction (Task 2)
Branch: `feat/run-streaming-diagnostics-mfk` (tree clean at HEAD `26208a3`; Task 1 chain closed through the 4.4 docs pass)
Verdict: **IMPLEMENT** (research gate passed — see §1). NO version change (already 0.10.0 from workflow step 3; no dependency change; `Cargo.lock` untouched).

---

## 1. RESEARCH GATE RECORD (executed 2026-10-23; both pages fetched live, both marked "Last updated 2026-10-15 UTC")

### 1.1 Citation A — the field is a SETTABLE insert property (DECISIVE)

Source: `https://developers.google.com/youtube/v3/live/docs/liveBroadcasts/insert` (LiveBroadcasts: insert reference), "Request body" section, verbatim:

> You can set values for these properties:
> … snippet.title, snippet.description, snippet.categoryId, snippet.scheduledStartTime, snippet.scheduledEndTime, **status.selfDeclaredMadeForKids**, contentDetails.monitorStream.enableMonitorStream, …

and "You must specify a value for these properties: snippet.title, snippet.scheduledStartTime, status.privacyStatus" — all three already sent by the locked O2 body.

Source: `https://developers.google.com/youtube/v3/live/docs/liveBroadcasts` (resource reference), `status.selfDeclaredMadeForKids`, verbatim:

> In a liveBroadcasts.insert request, this property allows the channel owner to designate the broadcast as being child-directed. In a liveBroadcasts.list request, the property value is only returned if the channel owner authorized the API request.

Property type per the table: **boolean**. (`status.madeForKids` verbatim: "This value indicates whether the broadcast is designated as child-directed. This property value is read-only." — the mirror is output-only; never sent by the app.)

Corroboration — `https://developers.google.com/youtube/v3/live/revision_history`, verbatim:

> January 10, 2020: "The selfDeclaredMadeForKids property enables content creators to specify whether a live broadcast is child-directed content. This property can be set when creating a broadcast via the liveBroadcasts.insert method."
> September 21, 2020: "To designate a live broadcast as being child-directed, set the status.selfDeclaredMadeForKids property to true when calling the liveBroadcasts.insert method to create the broadcast."

### 1.2 Citation B — behavior when UNSET (recorded honestly; NOT a blocker)

`liveBroadcasts/insert` "Request body" Note, verbatim:

> Note: The property table documents any default values that are set for the properties listed. The default values will be assigned in either of the following cases: You don't specify values for those properties. The part parameter value in your request doesn't specify the part that contains those properties.

The resource property table documents NO explicit API-level default for `status.selfDeclaredMadeForKids`. The API docs do NOT spell out a "channel-level default fallback" phrase for `liveBroadcasts`; the channel-level made-for-kids audience setting is documented YouTube product behavior (the January 10, 2020 revision links "Learn more about 'made for kids' content in the YouTube Help Center" and "setting the audience for your channel, videos, or broadcasts" — i.e. the YouTube Help Center / Studio channel advanced setting the operator already exercises). Encoded docs contract (used everywhere below): **absent = the field is not sent = YouTube's own default handling, which in practice follows the channel's made-for-kids setting**. This exactly preserves the operator's current channel-default path when the field is omitted.

### 1.3 Citation C — no additional part / scope / setting required

- `liveBroadcasts/insert` Parameters section, verbatim: "The part properties that you can include in the parameter value are id, snippet, contentDetails, and status." The locked O2 query `part=snippet,contentDetails,status` already includes `status` — **no part change**.
- `liveBroadcasts/insert` Authorization section, verbatim: "This request requires authorization with at least one of the following scopes: https://www.googleapis.com/auth/youtube, https://www.googleapis.com/auth/youtube.force-ssl" — the app grants `youtube.force-ssl` (`REQUIRED_SCOPE` unchanged). **No scope change, no additional setting.**

### 1.4 Citation D — no lifecycle/ingestion interaction that breaks simplicity (DECISIVE for "simple")

- The field is an insert-time declaration; the app's lifecycle NEVER calls `liveBroadcasts.update`/`transition`/`delete`, so post-insert mutability restrictions are irrelevant. The resource page attaches "cannot be updated once the broadcast is in the testing or live state" notes to several `contentDetails` fields — **no such note exists for `status.selfDeclaredMadeForKids`**, and no ingest/stream/binding restriction is documented for it.
- The `liveBroadcasts.insert` error table contains **no error specific to selfDeclaredMadeForKids** (no new typed error handling needed).
- Doc-quality note recorded: the resource page's JSON sketch shows `"selfDeclaredMadeForKids": string` — a documentation typo; the property table (boolean), the insert "You can set" list and the revision history govern (boolean).

### 1.5 VERDICT: IMPLEMENT

Both user-ruled conditions hold: (a) documented settable on `liveBroadcasts.insert` (§1.1); (b) the change is exactly one optional field in the existing O2 request body + one optional config field + straight plumbing (§1.2–1.4: no lifecycle, transport, redaction or shutdown restructuring). The global plan's Task-2 direction ("optional config field + one optional field in the existing O2 request body + command plumbing") is satisfied verbatim.

---

## 2. ENCODED DECISIONS (the implementer does NOT re-choose any of these)

- **D1 — config field name and type**: `youtube.broadcast.made_for_kids`, `Option<bool>` on `BroadcastConfig` (`None` = field absent in JSON = not sent on the wire = channel-default behavior). No `#[serde(default)]` needed (serde Options parse absence as `None`); `deny_unknown_fields` stays.
- **D2 — example template value**: `config/config.example.json` GAINS `"made_for_kids": false` (TODO explicitly lists "config example" as an extension point; `false` is the honest value for this app — a text-overlay prototype is not child-directed). Consistency locks: `src/config/test_fixtures.rs` fixture becomes `Some(false)`, and the embedded `example_json()` literal in `loader_tests.rs` gains the same line.
- **D3 — command field**: `CreateBroadcastCommand` gains `pub(crate) made_for_kids: Option<bool>` (no serde derive on commands — unchanged rule).
- **D4 — body field with serde attrs**: `BroadcastCreationStatus` gains `made_for_kids: Option<bool>` annotated `#[serde(rename = "selfDeclaredMadeForKids", skip_serializing_if = "Option::is_none")]`. The explicit rename is REQUIRED (`rename_all = "camelCase"` alone would emit `madeForKids` — the READ-ONLY field's name — which is wrong; do not rely on any camelCase derivation). `skip_serializing_if` is REQUIRED so the absent case keeps the O2 body byte-identical to today.
- **D5 — plumbing path**: `LiveBroadcastPreparer::from_config` maps `broadcast.made_for_kids` verbatim into the command (D-T2-13: no local re-validation; the config loader is the single funnel). No field flows to O1/O3/O4/O5.
- **D6 — no validation rule**: serde's typed `Option<bool>` is the whole contract; `validation.rs` is NOT edited; the documented rule count stays 21.
- **D7 — response models untouched**: nothing is READ back from the insert/list responses (`broadcast_resource.rs` unchanged; the app never reads `selfDeclaredMadeForKids`).
- **D8 — no `#[allow(` additions, no dependency changes, no commented-out code, offline tests only, no lifecycle/transport/redaction/shutdown restructuring, `demo`/`auth` byte-stable.**
- **D9 — new rosters are SPLIT FILES** (roster-split precedent of the 411 fix and Phase-05 rosters A/B), because two existing rosters sit at the 200-line cap edge:
  - body pins → NEW `src/youtube/live_models_made_for_kids_tests.rs` (wired from `live_models/mod.rs`, next to the existing `../live_models_tests.rs` wiring);
  - command-building pins → NEW `src/youtube/live_ingestion_made_for_kids_tests.rs` (wired from `live_ingestion.rs`);
  - config parse pins → NEW `src/config/broadcast_made_for_kids_parse_tests.rs` (wired from `loader.rs`).
  Existing rosters only receive the minimal compile-required literal/assert updates (§5 E/F).

## 3. High-level approach

Config model → example + fixture + parse pins → command type → O2 body field (skip-if-None, explicit rename) → from_config plumbing → existing pin updates (absent-case JSON stays byte-stable) → new split-file pins (absent/true/false body; true/false command building; absent/true/false/wrong-type parse) → fmt → targeted tests → FULL gate → both builds → audit → feature commit → docs + metadata → docs commit. Real-Google confirmation stays the operator's manual E2E (`docs/production-run.md` §8); no verification claim is made anywhere.

## 4. Step S0 — pre-flight

1. `git status` on `feat/run-streaming-diagnostics-mfk`: MUST be clean (only this task's intended edits may appear later). `git log --oneline -10` shows the Task-1 chain ending `26208a3` (+ later Task-1 completion commits if step 4.6/5 landed before this cycle — the caller states the tree is clean).
2. Confirm `Cargo.toml` version is `0.10.0` and `cargo grep`-free of `selfDeclaredMadeForKids`/`made_for_kids` in `src/` (fresh start assumption).

## 5. Steps S1–S9 — code + tests (atomic, in order; commit AFTER S15)

**E-config — S1: `src/config/model.rs`** — add to `BroadcastConfig` after `privacy_status`:

```rust
/// Optional per-stream made-for-kids declaration, sent as
/// `status.selfDeclaredMadeForKids` on liveBroadcasts.insert (documented
/// insert-settable property). Absent means the field is never sent and
/// YouTube applies its default handling (the channel's made-for-kids
/// setting).
pub made_for_kids: Option<bool>,
```

File projection ≈ 146 lines (cap 200, green).

**E-config — S2: `config/config.example.json`** — insert `"made_for_kids": false,` after the `privacy_status` line inside `youtube.broadcast`.

**E-config — S3: `src/config/test_fixtures.rs`** — `BroadcastConfig` literal gains `made_for_kids: Some(false),`.

**E-config — S4: `src/config/loader_tests.rs`** — embedded `example_json()` literal gains the same `"made_for_kids": false,` line (so `loads_valid_example_config`'s equality stays green). Projection 198 → 199 lines. ⚠ STOP condition: if any later rustfmt re-wrap pushes this file past 200 lines, STOP and report (do not split an existing roster on your own authority).

**E-config — S5: NEW `src/config/broadcast_made_for_kids_parse_tests.rs`** + wiring in `loader.rs` (append after the `loader_tests` mod):

```rust
#[cfg(test)]
#[path = "broadcast_made_for_kids_parse_tests.rs"]
mod broadcast_made_for_kids_parse_tests;
```

Test module contents (self-contained; `use crate::config::model::BroadcastConfig;` + `use super::*;`):
- local helper `example_value()` — `std::fs::read_to_string("config/config.example.json")` → `serde_json::from_str::<serde_json::Value>` (reads the committed template; same relative-path precedent as `example_mirrors_oauth_schema`);
- local helper `write_temp_config(dir: &tempfile::TempDir, contents: &str) -> PathBuf` (copy of the `loader_tests` helper shape);
- `committed_example_declares_made_for_kids_false` — `load_from_path` on the real `config/config.example.json`; assert `loaded.youtube.broadcast.made_for_kids == Some(false)` (new `Option<bool>` field; write the assertion with an `is_some_and` or equality form — local detail);
- `made_for_kids_absent_parses_as_none` — take `example_value()`, `remove("made_for_kids")` from `value["youtube"]["broadcast"]` (`.as_object_mut().expect(...)`), write temp, load, assert `is_none()`; ALSO assert the broadcast title parses (guards against overload-shape misreads);
- `made_for_kids_true_parses` — insert `serde_json::json!(true)` into `value["youtube"]["broadcast"]["made_for_kids"]`, load, assert `== Some(true)`;
- `made_for_kids_wrong_type_rejected` — insert `json!("yes")`, expect `ConfigError::Deserialization` via the `expect_load_error`-shaped local helper (`matches!` + message contains the path). Projection ≈ 90 lines.

**E-youtube — S6: `src/youtube/live_models/commands.rs`** — `CreateBroadcastCommand` gains:

```rust
/// Optional made-for-kids declaration (`None` = the field is NOT sent on
/// the O2 insert; YouTube applies its default made-for-kids handling).
pub(crate) made_for_kids: Option<bool>,
```

Projection ≈ 50 lines.

**E-youtube — S7: `src/youtube/live_models/request_bodies.rs`** — `BroadcastCreationStatus` gains the D4 field (attrs exactly: `#[serde(rename = "selfDeclaredMadeForKids", skip_serializing_if = "Option::is_none")]`); `BroadcastRequestBody::from_command` gains the `made_for_kids: command.made_for_kids` mapping line; module doc gains one grounding line citing the insert reference's "You can set values" list (`https://developers.google.com/youtube/v3/live/docs/liveBroadcasts/insert`). Projection ≈ 122 lines.

**E-youtube — S8: existing compile-required updates**:
- `src/youtube/live_models_tests.rs` (line ~155 struct literal): add `made_for_kids: None,` (196 → 197). The existing `create_commands_serialize_documented_shape` JSON pin is UNTOUCHED — its exact expected JSON (`"status": {"privacyStatus": "unlisted"}`) is now the absent-case wire pin by virtue of the serde skip.
- `src/youtube/google_live_api_transport_tests.rs` (line ~162 struct literal): add `made_for_kids: None,` (194 → 195). Its loopback wire pin keeps asserting the absent-shape body.
- `src/youtube/live_ingestion_tests.rs` `prepare_records_exact_commands_from_validated_config`: add `assert!(broadcast.made_for_kids.is_none());` (187 → 188).

**E-youtube — S9: plumbing in `src/youtube/live_ingestion.rs`** — `from_config`'s `broadcast_command` literal gains `made_for_kids: broadcast.made_for_kids,`; module doc gains one sentence ("the optional per-stream made-for-kids declaration rides on the O2 insert"). Append wiring after the `live_ingestion_failure_tests` mod:

```rust
#[cfg(test)]
#[path = "live_ingestion_made_for_kids_tests.rs"]
mod live_ingestion_made_for_kids_tests;
```

Projection ≈ 163 lines.

**E-youtube — S10: NEW `src/youtube/live_models_made_for_kids_tests.rs`** + wiring in `live_models/mod.rs` (after the existing test mod):

```rust
#[cfg(test)]
#[path = "../live_models_made_for_kids_tests.rs"]
mod live_models_made_for_kids_tests;
```

Three tests (`use super::*;` + `serde_json::json`), each building a `CreateBroadcastCommand` (same fixture strings as `create_commands_serialize_documented_shape`) and asserting the full exact `serde_json::to_value(&BroadcastRequestBody::from_command(&command))` JSON:
- `made_for_kids_none_omits_self_declared_field_from_o2_body` — command `made_for_kids: None` → exact body JSON identical to today's pin (status object WITHOUT `selfDeclaredMadeForKids`);
- `made_for_kids_true_appears_in_o2_status_object` — `Some(true)` → status object contains `"selfDeclaredMadeForKids": true` (full-body exact JSON);
- `made_for_kids_false_appears_in_o2_status_object` — `Some(false)` → `"selfDeclaredMadeForKids": false` (full-body exact JSON).
Projection ≈ 85 lines.

**E-youtube — S11: NEW `src/youtube/live_ingestion_made_for_kids_tests.rs`**
Two tests against the shared scripted fake, reusing the public-in-scope helpers of the sibling (`super::live_ingestion_tests::{FIXED_NOW_EPOCH_SECS, queue_preparation}` — both are `pub(super)`, visible to siblings):
- `declared_true_flows_from_config_to_the_broadcast_command` — `let mut config = valid_config();` (crate-level test alias), set `config.youtube.broadcast.made_for_kids = Some(true);`, queue the happy script, `LiveBroadcastPreparer::from_config(&config, FIXED_NOW_EPOCH_SECS).prepare(...)`, then assert the captured O2 command: `transport.captured_create_broadcast.borrow().as_ref().expect("the O2 command was captured").made_for_kids == Some(true)`;
- `declared_false_flows_from_config_to_the_broadcast_command` — same with `Some(false)` → `Some(false)`.
Projection ≈ 75 lines.

## 6. Steps S12–S15 — verification (Alpine-VM MCP; commands single and unchained)

1. **S12 targeted tests**: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked made_for_kids` (exit 0; exercises all new pins + the updated existing pins; `exitCode` authoritative).
2. **S13 full gate**: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — expect exit 0, `ALL CHECKS PASSED`. Baseline 369 passed; expected ≈ 378 (9 new tests: 3 body + 2 command-building + 4 parse); **the OBSERVED count governs** — record the log path under `logs/checks/` and the observed count in the completion summary. The gate's existing pins (`create_commands_serialize_documented_shape`, `stream_insert_posts_documented_path_with_locked_body`, the three bind-wire tests, all O4/readiness rosters) are the byte-stability evidence for O1/O3/O4/O5 forms; demo/auth rear untouched.
   - If `cargo fmt --check` fails with pure re-wrapping of the new lines in NEW files, mechanical rustfmt normalization is allowed; if it re-wraps `loader_tests.rs` over the cap → §5 S4 STOP condition.
3. **S14 builds + sizes**: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`, then `... sh scripts/build-windows.sh` (both exit 0), then `ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` — record BOTH sizes, cross-checked against the build scripts' own printed sizes.
4. **S15 audit** (§5.4 style, host side): `git status --porcelain` shows ONLY the intended paths; `git check-ignore -v dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe` (`.gitignore:34:dist/`); `git ls-files dist/ videos/` empty; secret greps (`ya29`, `AIza`, `key=`) at pre-existing counts only; `git grep -n "allow(" -- src/` unchanged from baseline (no new staging — every added field is consumed). **Never stage `dist/`, `config/config.json`, or any ignored path** (`gitignore-compliance` rule).

**S16 — commit (repo style, single feature commit)**: `feat: add optional per-stream made-for-kids config mapped to the broadcast insert` — paths: the 4 `src/config` files (+ sample JSON), the 5 `src/youtube` files + 2 new test rosters + 1 config parse roster. Nothing else staged.

## 7. Steps S17 — docs + metadata (docs commit `docs: document the made_for_kids broadcast config option`)

- **`docs/configuration.md`** — `### youtube.broadcast` gains the bullet:
  `- made_for_kids (boolean, optional) — when present, the value is sent as status.selfDeclaredMadeForKids on the liveBroadcasts.insert request (a documented insert-settable property in the official liveBroadcasts reference): true declares the broadcast child-directed, false declares it not child-directed. When ABSENT the field is never sent and YouTube applies its own default handling — in practice the channel-level made-for-kids setting (YouTube Studio → Settings → Channel → Advanced section). Any non-boolean value is rejected at deserialize time. Example: false.`
  The Validation Rules count (21) and every other list stay unchanged.
- **`docs/production-run.md`** — (1) Startup item 6 (`liveBroadcasts.insert`…) gains one sentence: "The optional `made_for_kids` config value rides on this same insert as `status.selfDeclaredMadeForKids`; leaving the config field unset keeps YouTube's channel-level made-for-kids default (see `docs/configuration.md`)." (2) The Readiness-timeouts diagnosis procedure item 1 (which already mentions the "made-for-kids channel setting") gains one clause pointing at the per-stream override: "a per-stream declaration is available: `youtube.broadcast.made_for_kids` in `config/config.json` (docs/configuration.md)". No other wording changes; no real-Google verification claim.
- **`README.md` drift check** — read the Configuration section; expected NO edit. If it enumerates broadcast fields and omits this one, add the field name only.
- **Metadata (planner-owned, committed in the docs commit per task-closure conventions)**: `.agent/project-structure.md` (model.rs line — BroadcastConfig `made_for_kids` Option; the 3 new test-roster files and the config/youtube test-file counts; line-count note), `.agent/project-info/tech.md` (one Configuration-section bullet: optional made-for-kids insert field on the O2 body, no dependency change, loopback-pinned offline), `.agent/project-info/context.md` (Task-2 record: verdict + citations summary + gates/builds evidence).
- Recount touched `src/` files against the 200-line cap (projections in §5); report any file ≥ 198.

**S18 — stop-and-report conditions (implementer MUST stop, not improvise)**: full-gate failure that is not a mechanical fmt fix; a new-field compile ripple NOT listed in §5 (e.g. another construction site appears); `loader_tests.rs` projected past 200 lines; any audit mismatch. On stop: report exactly what landed, what did not, and the blocker.

## 8. Encoded constraints (byte-stability ledger)

- O2 body with the field absent = byte-identical to today (serde skip + the untouched existing JSON/wire pins).
- O1/O3/O4/O5 request forms byte-stable (pinned tests re-run in the gate; `api_request.rs`, `google_live_api_transport.rs`, readiness/gate modules untouched).
- `demo`/`auth` and the Task-1 diagnostics surface byte-stable (no edits outside the listed files).
- Single-`error:`-line, exit-code, shutdown-order, redaction contracts untouched.
- Rules: no new `allow`; no dependency/lockfile change; no commented-out code; private-by-default (both new command/body fields stay `pub(crate)`/private exactly like their siblings); single-section booleans (no new hand-written conditionals at all); ≤2-arg functions (no new function signatures except 1-arg test helpers); ≤50-line bodies; ≤200-line files (§5 projections; recount before the commit).

## 9. Planned test roster (new + updated)

| Roster | File | Tests |
| --- | --- | --- |
| Body serialization | `src/youtube/live_models_made_for_kids_tests.rs` (NEW) | absent→omitted, true, false |
| Command building | `src/youtube/live_ingestion_made_for_kids_tests.rs` (NEW) | config Some(true)→command, Some(false)→command |
| Config parse | `src/config/broadcast_made_for_kids_parse_tests.rs` (NEW) | example false, absent→None, true, wrong-type rejected |
| Updated existing | `live_models_tests.rs` (absent literal), `google_live_api_transport_tests.rs` (absent wire pin kept), `live_ingestion_tests.rs` (absent assert) | 3 edits, 0 net new |

Total: 9 new tests; expected gate ≈ 378 (observed governs; log recorded).

## 10. Alignment check vs TODO Task 2 + global plan

TODO bullets: research with citations (§1 ✓, recorded here and mirrored into context.md/docs); optional config property `youtube.broadcast.made_for_kids` absent=channel-default (D1/D2 ✓); mapped into the O2 body via `from_config` (S7/S9 ✓); validation/model/tests/config example/docs extended (D6, S1–S2, S5/S10/S11, S17 ✓); absent/true/false body pins (S10 ✓). Global-plan direction and Amendment 1 honored; no other task's scope touched; version/branch/merge/push NOT part of this plan.
