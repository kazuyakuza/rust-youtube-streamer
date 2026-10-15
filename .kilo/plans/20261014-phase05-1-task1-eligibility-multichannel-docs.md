# Phase 05.1 — Task 1 Implementation Plan — Operator Documentation Updates (eligibility, 24 h wait, multi-channel)

- Date: 2026-10-14
- Sub-step: 4.1b (Analysis & Planning). The implementer executes §6–§8 in 4.2; reviewers use §9–§10 in 4.3–4.5b.
- TODO authority: `.agent/todos/20261014/20261014-todo-1.md`, Task 1 (`### 1. Operator Documentation Updates (eligibility, 24 h wait, multi-channel)`) + the global Scope/Boundaries + Explicitly-Out-of-Scope sections.
- Global plan (binding): `.kilo/plans/20261014-phase05-1-operator-feedback.md` — D-GF1..D-GF6 govern (D-GF2 docs-only surfaces; D-GF3 evidence-backed eligibility; D-GF4 multi-channel framing).
- Task 1 is DOCS-ONLY: no Rust/code changes, no dependency changes, no builds, no test changes, no version change (0.9.0 already set at workflow step 3).
- Branch: `feat/phase05-1-operator-feedback` (checked out; HEAD `cf98aee` "chore: bump version to 0.9.0"; tree clean, verified 2026-10-14).
- Per-task plans are auto-approved (global plan §5): proceed to 4.2 without waiting after this plan is saved.

---

## 1. Research Findings (official Google/YouTube sources, fetched fresh 2026-10-14)

Note on dates: `developers.google.com` pages display a visible "Last updated" line; `support.google.com` (YouTube Help) pages do NOT display a last-update date (only a `©2026 Google` footer) — record citations for those pages without an invented date.

| # | Official page (exact URL) | Visible date | What it states (verbatim/paraphrase) |
|---|---|---|---|
| R1 | https://developers.google.com/youtube/v3/live/authentication — "Implementing OAuth 2.0 Authentication" (YouTube Live Streaming API) | **Last updated 2026-09-14 UTC** | "A channel must be approved to use the YouTube Live feature, which enables the channel owner to stream live content to that channel. If you send API requests on behalf of an authenticated user whose channel is not enabled or eligible to stream live content, the API will return an `insufficientPermissions` error." This is the API-visible evidence for the channel-side requirement. Already the basis of the docs' eligibility failure lines. |
| R2 | https://support.google.com/youtube/answer/2474026 — "Get started with live streaming" (YouTube Help; also reachable with `?hl=en&co=GENIE.Platform%3DDesktop`) | none visible (©2026) | "To live stream, you need to have no live streaming restrictions in the past 90 days and you need to verify your channel." ("verify your channel" links to R3.) Also: minimum age 16 to live stream; a 14-day strike prevents live streaming; a channel restricted from live streaming may not use another channel to stream. |
| R3 | https://support.google.com/youtube/answer/171664 — "Verify your YouTube account" (YouTube Help) | none visible (©2026) | "To verify your channel, you'll be asked to enter a phone number. We'll send a verification code by text or voice call." After verification, unlocked capabilities include "Live stream" (among: uploads >15 min, custom thumbnails, appeal copyright claims). Anti-abuse rule: one phone number can be linked to "no more than 2 channels per year". |
| R4 | https://support.google.com/youtube/answer/9722880 — "See & unlock features with phone verification" (YouTube Help) | none visible (©2026) | WHERE to check: studio.youtube.com → **Settings → Channel → Features**; features screen shows "Features that require phone verification"; once phone-verified, "Live streaming" is unlocked. Access "may change if your channel is restricted because of Community Guidelines or copyright strikes". |
| R5 | https://support.google.com/youtube/answer/2907883 — "Create a YouTube live stream with an encoder" (YouTube Help) | none visible (©2026) | Step "1. Enable live streaming": "Enabling a live stream for the first time may take up to 24 hours. Once enabled, your stream can go live instantly." This is the ENCODER path — the path this service's external-FFmpeg ingestion follows. Primary citation for the ~24 h rule. |
| R6 | https://support.google.com/youtube/answer/9228390 — "Create a live stream on mobile" | none visible (©2026) | Mobile-only requirements (e.g. "at least 50 subscribers"; "You may need to wait 24 hours before you can start your first live stream"). **NOT used in the docs: the 50-subscriber figure is a MOBILE-stream requirement, not an encoder/API requirement — do not import it.** |
| R7 | https://support.google.com/youtube/answer/16404722 — "Stream across platforms" | none visible (©2026) | "Note: Enabling your first live stream will take at least 24 hours after your first request." Phrasing differs from R5 ("may take up to 24 hours"). Research-only nuance (see §2); the docs cite R5 as primary. |
| R8 | `https://www.youtube.com/features` | n/a (product page) | The existing `run` failure lines and docs point operators here. Anonymous fetch presents the Google sign-in wall — the page is per-account and requires signing in as the channel owner. NO doc text claims anonymous accessibility; record "sign in as the channel owner". |
| R9 | https://support.google.com/youtube/community-threads/ (e.g. thread/381471272) | community, NOT official rule text | Community/Top-Contributor replies repeat "typically up to 24 hours… cannot be bypassed". NOT cited in docs (not an official rule source). |

### 1.1 Facts that will be recorded (exact eligibility set)

1. **Distinction:** API enablement in the Google Cloud project (existing three artifacts) and channel-level live-stream access are INDEPENDENT requirements; the API returns `insufficientPermissions` for a not-enabled/ineligible channel (R1).
2. **Channel eligibility rule:** no live-streaming restrictions in the past 90 days + verified channel (phone verification) (R2, R3); unlock location is YouTube Studio Settings → Channel → Features (R4).
3. **Activation window:** first-time live-stream enablement "may take up to 24 hours" (R5, encoder path). The 90-day-restriction rule (R2) is included because R2 states it; the R7 "at least 24 hours" variant is NOT imported (keep one phrasing, from R5).
4. **Minimum age 16** on live streams (R2) — included as a one-clause YouTube-policy note.
5. **Where to check:** `youtube.com/features` (sign-in required, R8) and YouTube Studio → Settings → Channel → Features (R4).
6. **Per-channel phone verification; ≤2 channels per phone number per year** (R3) — dovetails with the multi-channel story.
7. **Operator-supplied ~24 h fact:** the 2026-10-14 user report (channel awaiting live-stream activation, ~24 h wait, real test on hold) is recorded as USER-SUPPLIED, dated, never agent-observed; official text (R5) supplies the general rule. Consistent with the official "up to 24 hours" — flagged in §2, no contradiction.

---

## 2. STOP-Zone / Flag Register (recorded as-is; no interpretation drift)

- **F1 — User report vs official wording:** the user reported (~2026-10-14) "just required live stream access, and need to wait ~24 h". Official encoder-page rule (R5) is "may take up to 24 hours" for FIRST-TIME enablement; R7 says "at least 24 hours". Resolution: official text is recorded as the GENERAL rule with R5's wording; the operator's ~24 h IS the recorded user-supplied instance, dated 2026-10-14, explicitly "never agent-observed". No official fact contradicts the user report.
- **F2 — AC wording anomaly (interpretation ruling):** TODO acceptance-criteria line 47 says production-run.md must state "that live-stream analytics access requirement". The authoritative Task-1 bullet (TODO line 29) says the requirement is **live-stream ACCESS on the channel**. Ruling: implement "channel live-stream ACCESS" (per Task-1 bullet); do NOT invent any "analytics access" requirement.
- **F3 — `youtube.com/features` provenance:** code/docs already direct to `youtube.com/features` (unchanged code — Task 1 may not touch code error lines). The current official Help pages document the Studio path (R4). The docs record BOTH pointers and honestly note the sign-in requirement (R8). No claim that the Help pages still deep-link to that URL.
- **F4 — Support-page dates are invisible:** citations of R2–R7 carry NO dates (do not fabricate any). Only R1 carries "Last updated 2026-09-14 UTC".
- **F5 — Do-not-import list:** 50-subscriber minimum (mobile-only, R6), R7's "at least 24 hours", community-thread claims (R9), "10 active streams/3 per key" limits (not requested). None appear in the docs edits.

---

## 3. Task-1 Decisions (binding for 4.2)

- **D-T1-101 — Section placement (google-cloud-prerequisites.md):** a new top-level section `## Channel Live-Streaming Access (YouTube-Side Requirement)` inserted between `## The Requirements, Term by Term` and `## Why Not an API Key, a Service Account, or Just a Browser?`. Rationale: the existing "Term by Term" section is scoped to the three Google-Cloud artifacts; channel eligibility is a separate axis and follows it.
- **D-T1-102 — citation style:** mirror the file's existing style: inline markdown links, official quotes in *italics*/quotes, a requirements table with a `| Requirement | Official source |` shape, no dates invented for support pages.
- **D-T1-103 — Troubleshooting table gets exactly ONE new row** (bottom) keyed on `insufficientPermissions`, linking to the new section by its internal anchor. No other rows touched.
- **D-T1-104 — Verification Checklist gets exactly ONE new checkbox** (bottom of the `auth` checklist) covering channel live access.
- **D-T1-105 — Operator-fact placement:** the dated operator status lives as ONE paragraph inside the new eligibility section (google-cloud-prerequisites.md) AND as the dated operator-wait note inside production-run.md's Manual Verification Procedure; it is NOT duplicated into authentication.md.
- **D-T1-106 — Multi-channel section placement (authentication.md):** a new top-level section `## One Grant = One Channel Identity` inserted between `## First Authorization (Interactive, Once)` and `## Credential Files` (grant defensible facts belong right after the consent-time narrative). All multi-channel content lives ONLY here; production-run.md carries a one-line reminder, context.md the operator line.
- **D-T1-107 — Production-run operator-wait note placement:** directly after the Manual Verification Procedure's intro paragraph (line 116 area), before step 1, as a bolded dated note. The Prerequisites list gains item 6 pointing at the same facts.
- **D-T1-108 — Multi-channel reminder placement (production-run.md):** ONE sentence appended to manual step 6 ("Actual video arrival"), the step where Studio is opened and the broadcast is observed.
- **D-T1-109 — context.md: ONE new bullet in Immediate Next Steps**, inserted after current item 2 (the E2E-outstanding item), renumbering 3→4, 4→5. No other context.md content is touched by Task 1 (a fresh "Recent Changes" entry, if wanted, belongs to the workflow's closing/docs-alignment step, not 4.2).
- **D-T1-110 — NO README, NO code, NO log-table changes:** the "What the Logs Look Like" table is Task 2's edit zone (watch-link print) — Task 1 must not touch it. `.agent/project-structure.md` and `tech.md` need NO Task-1 update (no new files, no dependency/stack change; flagged for reviewer awareness vs AC line 54, which activates only "where the changes touch them").
- **D-T1-111 — Anchor slugs (exact):** `#channel-live-streaming-access-youtube-side-requirement` and `#one-grant--one-channel-identity` ("=" and parentheses are dropped by the slugifier; the double hyphen matches the repo's existing `#revoked--unrefreshable--insufficient-states` precedent).
- **D-T1-112 — Commit: ONE docs commit** containing exactly the four files (§8).

---

## 4. Permitted Edit Zones (ONLY these)

1. `docs/google-cloud-prerequisites.md` — TOC line insertion; the new section (after "The Requirements, Term by Term"); ONE troubleshooting row; ONE checklist checkbox; TWO See Also bullets.
2. `docs/production-run.md` — Prerequisites item 6; eligibility failure-bullet pointer sentence (line 107 area); the dated operator-wait note in the Manual Verification Procedure; ONE sentence in step 6 of that procedure.
3. `docs/authentication.md` — TOC line insertion; the new section (after First Authorization).
4. `.agent/project-info/context.md` — ONE new numbered bullet in Immediate Next Steps (+ renumber of 3/4).

Everything else in every file stays byte-unchanged. No new files. Max-lines rule: docs files are exempt (rule scopes only to `src/`); keep prose style consistent with hosts (no TOC churn beyond listed insertions).

---

## 5. Exact Edit Specification — `docs/google-cloud-prerequisites.md`

### 5.1 TOC (after the `- [The Requirements, Term by Term](#the-requirements-term-by-term)` line, before the "Why Not…" line)

```
- [Channel Live-Streaming Access (YouTube-Side Requirement)](#channel-live-streaming-access-youtube-side-requirement)
```

### 5.2 New section (copy-ready body; head exactly `## Channel Live-Streaming Access (YouTube-Side Requirement)`)

```markdown
The three artifacts above are the Google Cloud side of the setup. A separate, independent **YouTube-side** requirement also exists: the channel that will carry the broadcast must itself have live-streaming access. Enabling the *YouTube Data API v3* in a Google Cloud project does NOT enable live streaming on the channel — the two are granted independently. Google's Live Streaming API documentation states the API-visible rule: *"A channel must be approved to use the YouTube Live feature"* — a request on behalf of a channel that is not enabled or eligible to stream live *"will return an `insufficientPermissions` error"* ([Implementing OAuth 2.0 Authentication](https://developers.google.com/youtube/v3/live/authentication), Last updated 2026-09-14 UTC). This is the failure the `run` command surfaces as `live streaming is not enabled` / `this channel cannot stream live now` (see [`docs/production-run.md`](production-run.md)).

Official channel-side requirements (YouTube Help):

| Requirement | Official source |
| --- | --- |
| No live-streaming restrictions in the past 90 days, and the channel is verified | ["Get started with live streaming"](https://support.google.com/youtube/answer/2474026) — *"To live stream, you need to have no live streaming restrictions in the past 90 days and you need to verify your channel"* |
| Phone verification is what unlocks the live-streaming feature | ["Verify your YouTube account"](https://support.google.com/youtube/answer/171664) and ["See & unlock features with phone verification"](https://support.google.com/youtube/answer/9722880) — *Live streaming* appears among the features granted by verifying the phone number |
| First-time enablement is not instant: it *"may take up to 24 hours"* | ["Create a YouTube live stream with an encoder"](https://support.google.com/youtube/answer/2907883) — the encoder path this service's external FFmpeg follows |
| Minimum age to live stream is 16 (YouTube policy) | ["Get started with live streaming"](https://support.google.com/youtube/answer/2474026) |

Where to check (both lookups act on the channel and require signing in as the channel owner):

- `https://www.youtube.com/features` — the URL the `run` failure lines point to; it is a per-account page, so it resolves only after sign-in.
- YouTube Studio (`studio.youtube.com`) → **Settings → Channel → Features** — the documented location of the phone-verification feature screen ([See & unlock features with phone verification](https://support.google.com/youtube/answer/9722880)).

Phone verification is per channel; Google's anti-abuse rule allows one phone number to be linked to no more than 2 channels per year ([Verify your YouTube account](https://support.google.com/youtube/answer/171664)).

Operator-supplied status (2026-10-14, user report — never agent-observed): the channel-owning operator enabled live access on the target channel and the channel is awaiting activation, with an expected wait of about 24 hours; the real manual test ([`docs/production-run.md`](production-run.md)) is ON HOLD until live access is active. The official sources above state the general rule (*"may take up to 24 hours"* on first-time enablement); the operator report is that rule's dated, user-supplied instance.
```

### 5.3 Troubleshooting table — ONE row appended at the bottom of the existing table

```
| `insufficientPermissions` (live API error reason) | The channel is not enabled/eligible for live streaming — a YouTube-side channel requirement, separate from this page's Google Cloud setup | Enable live streaming on the channel (see [Channel Live-Streaming Access](#channel-live-streaming-access-youtube-side-requirement)); the `run` failure lines point to `youtube.com/features` |
```

### 5.4 Verification Checklist — ONE checkbox appended at the bottom

```
- [ ] The channel has live-streaming access (channel phone verification done; first-time activation may take up to 24 hours).
```

### 5.5 See Also — TWO bullets appended at the bottom of the list

```
- [Get started with live streaming](https://support.google.com/youtube/answer/2474026) — official YouTube channel-side live-streaming requirements.
- [Create a YouTube live stream with an encoder](https://support.google.com/youtube/answer/2907883) — the encoder path and the first-time enablement window (up to 24 hours).
```

---

## 6. Exact Edit Specification — `docs/production-run.md`

### 6.1 Prerequisites — new item 6 after item 5

```
6. Live-streaming access on the target channel: the channel itself must have live streaming enabled — channel phone verification plus a first-time activation window (official guidance: it may take up to 24 hours). The Google Cloud setup in item 1 does NOT enable live access on the channel; the requirements and where to check are documented in [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) (section "Channel Live-Streaming Access").
```

### 6.2 Broadcast/Stream Fate eligibility failure-bullet (existing bullet starting "- Live-streaming eligibility —") — append one pointer sentence

The bullet ends with "…delete the leftover `ready` resources in YouTube Studio before re-running." → append before the final period:

```
; the channel-side requirements and the first-time activation window (up to 24 hours) are documented in [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) — Channel Live-Streaming Access
```

### 6.3 Manual Verification Procedure — dated operator-wait note, inserted directly after the intro paragraph ("The canonical safe test plan (D-T4-8)…never as build evidence.")

```
**Operator status note (2026-10-14, user-supplied — never agent-observed):** the target channel is awaiting live-stream activation after live access was enabled (~24 h expected wait per the operator report; official guidance is that first-time enablement *"may take up to 24 hours"* — see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md), Channel Live-Streaming Access). The REAL manual test below is ON HOLD until the channel has live-streaming access.
```

### 6.4 Manual step 6 ("Actual video arrival") — ONE appended reminder sentence

Step 6 currently ends "…Confirm at least ~1–2 minutes of continuous video." → append:

```
Verify the broadcast is on the EXPECTED channel: one OAuth grant binds to one channel identity chosen at consent, and the application neither selects, surfaces, nor verifies which channel ([`docs/authentication.md`](authentication.md)) — if the broadcast appeared on a different channel, Ctrl+C-stop, re-run `auth` to select the intended channel, and repeat from step 4.
```

---

## 7. Exact Edit Specification — `docs/authentication.md` + `context.md`

### 7.1 authentication.md TOC (after the `- [First Authorization (Interactive, Once)](#first-authorization-interactive-once)` line)

```
- [One Grant = One Channel Identity](#one-grant--one-channel-identity)
```

### 7.2 New section (head exactly `## One Grant = One Channel Identity`), inserted after the First Authorization section and before `## Credential Files`

```markdown
The OAuth grant binds to exactly ONE YouTube channel identity, and the choice is made by the operator at consent time, never by the application:

- Which channel the grant covers is decided during the Google sign-in / channel-pick page of the consent flow; where one account controls several channels, that page selects the specific channel. The application neither selects, surfaces, nor verifies WHICH channel was granted — the persisted scope evidence records only the granted scopes, never a channel identity.
- The authorization probe that `auth` relies on (`GET /youtube/v3/channels?part=id&mine=true`, see Unattended Operation) reads the authorized channel as an access check; it never prints or validates which channel it is.
- Unattended refresh binds the SAME channel: the stored refresh token can obtain new access tokens for the original grant only; it cannot switch the persisted grant to a different channel.
- To operate a different channel, re-run `auth` and select the intended channel at consent — the persisted credentials are replaced by a fresh grant for that channel.
- During real streaming tests, confirm in YouTube Studio that the broadcast landed on the expected channel (manual-procedure reminder: [`docs/production-run.md`](production-run.md)); phone-verified live access is a per-channel YouTube-side requirement, not a per-application one (see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md), Channel Live-Streaming Access).
```

### 7.3 `.agent/project-info/context.md` — ONE new bullet in "## Immediate Next Steps", inserted after current item 2; old items 3 and 4 renumber to 4 and 5

```
3. OPERATOR FACT (dated 2026-10-14, user-supplied, never agent-observed): the channel-owning operator enabled live access on the target channel and the channel is awaiting live-stream activation (~24 h expected wait per the user report); the operator-run manual E2E is ON HOLD until the channel has live-streaming access (official guidance: first-time enablement may take up to 24 hours — see `docs/google-cloud-prerequisites.md`, "Channel Live-Streaming Access").
```

---

## 8. Execution & Commit Plan (4.2)

1. Verify branch/tree: current branch `feat/phase05-1-operator-feedback`, working tree clean (`git status`).
2. Apply §5–§7 edits with the structured `edit` tool (prefer edits over raw shell; no content with literal `\n` — real newlines only).
3. Run the verification checks of §9.
4. Stage EXACTLY the four files and commit once:
   - `git add docs/google-cloud-prerequisites.md docs/production-run.md docs/authentication.md .agent/project-info/context.md`
   - Read `.gitignore` and re-check `git status --porcelain` before committing (gitignore-compliance rule): nothing else may appear in the staged set.
   - Commit message: `docs: record channel live-streaming eligibility and multi-channel identity`
5. Do NOT push (workflow step 5) and do NOT flip any TODO checkbox (do that in step 5/4.6).

---

## 9. Verification (docs-only; NO dev-checks gate required — the final-tree gate belongs to TODO Task 3)

Self-checks, in order; every failure is fixed by editing only the §4 zones:

1. **Link/anchor existence:** every new markdown link target file exists (`authentication.md`, `production-run.md`, `google-cloud-prerequisites.md` relative links) and every new internal anchor matches its heading slug exactly: `#channel-live-streaming-access-youtube-side-requirement`, `#one-grant--one-channel-identity`.
2. **Newline hygiene:** search the four changed files for the literal two-character sequence `\n` inside content — zero matches (newline-prevention rule).
3. **Invented-numbers grep:** search each §4 zone for digits; every digit in the new content must trace to an official citation or pre-existing text (allowed set in new content: `90` days, `24` hours (×), `16` (age), `2` (channels/year), `2026-10-14`, `2026-09-14`, `2026` in URLs, `feature` path `#...-24-hours` none). Specifically grep for `50`, `10,000`, `120`/`180` appearing OUTSIDE their original lines — must not appear in the new sections.
4. **Operator-fact framing:** the string `never agent-observed` appears in BOTH the eligibility section and the production-run note; the string `user-supplied` too.
5. **Diff scope (`git diff HEAD~1` after commit or `git diff` before):** changed hunks exist ONLY in the four listed files and ONLY in the §4 zones; untouched lines byte-identical.
6. **No TODO-file edits, no YouTube-code edits:** `git status` shows no `src/`, `README.md`, or `.agent/todos/` modifications.

---

## 10. Reviewer / Adherence Expectations (4.3–4.5b)

- Verify citations against this plan's §1 table (URLs, quotes, R1's visible date; absence of invented support-page dates).
- Verify D-GF3/D-GF4 framing: operator facts labeled user-supplied/dated; multi-channel section = one grant/one channel, app-agnostic, `auth` re-runs, Studio verification, refresh binds same channel.
- Verify F2 ruling: NO "analytics access" requirement invented anywhere.
- Verify permitted-zone compliance (§4) and byte-stability outside the zones (including key handling: no keys/tokens/RTMPS text introduced anywhere; docs contain no credential material — hygiene rule).
- Docs-only task ⇒ no comment/section beyond this plan's addenda; any wording tweak MUST keep the §1 facts, headings, and anchors byte-identical.

---

## 11. Final Self-Check vs Original Task-1 Text (TODO lines 28–31)

- ✅ Bullet 1 (eligibility into google-cloud-prerequisites.md, linked coherently with existing failure tables): §5 (new section + troubleshooting row + checklist + TOC/See Also).
- ✅ Bullet 2 (production-run prerequisites note + dated ~24 h user-supplied operator note): §6.1 + §6.3 (+ §6.2 pointer).
- ✅ Bullet 3 (authentication.md multi-channel identity text + production-run one-liner at step 5/6 area): §7.2 + §6.4.
- ✅ Bullet 4 (context.md one dated operator-fact line in Immediate Next Steps): §7.3.

Nothing outside TODO Task 1 scope is planned; no code, no builds, no new files, no pushes.
