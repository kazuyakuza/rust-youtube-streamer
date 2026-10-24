# Per-Task Implementation Plan — Ingest "No data" Task 1 (4.1b → 4.6)

Plan date: 2026-10-24 · Owner: Architector Agent (Critical Workflow sub-step 4.1b)
TODO: `.agent/todos/20261024/20261024-todo-1.md` · Global plan (BINDING): `.kilo/plans/20261024-ingest-no-data-audio-signal.md`
Task: **Task 1 — Ingest-Requirement Research + Operator Probe Procedure (docs/research, no behavior change)**
Branch: `fix/ingest-no-data` at HEAD `983293e` (`chore: bump version to 0.11.0`), tree clean. NOT front-end → no 4.1a. No branch creation (step 2 already done); no push (step 5).

---

## 0. Scope Contract (READ FIRST)

- **DOCS + RESEARCH ONLY.** Zero changes to `src/`, `config/`, `scripts/`, `Dockerfile`, `Cargo.toml`, `Cargo.lock`. The app's streaming vectors, diagnostics surface, demo command, and exit contracts stay byte-stable.
- The deliverables are exactly: (1) a new section in `docs/production-run.md`, (2) a new Recent-Changes block in `.agent/project-info/context.md`, (3) one commit in the repo `docs:` style, (4) light verification evidence (gitignore/hygiene + ONE dev-checks gate run; NO builds).
- The implementer is a JUNIOR developer: every structural decision is encoded below. Do not restructure, retitle, or reorder anything not specified here.

---

## 1. RESEARCH RECORD (the verdict — carried into every downstream artifact)

All facts below were gathered 2026-10-24 from the cited live pages. Every citation is exact URL + exact quoted text. Separation: §1.1–§1.3 are OFFICIAL (Google/YouTube + FFmpeg project); §1.4 is COMMUNITY (labeled as such); anything unresolved is listed in §1.6.

### 1.1 (i) Does YouTube's ingest accept a VIDEO-ONLY RTMP/RTMPS stream? — OFFICIAL ANSWER: the documented ingest expectation is "exactly one audio stream"; video-only is NOT a compliant configuration

1. **YouTube Live Streaming API reference, `liveStreams` resource** — https://developers.google.com/youtube/v3/live/docs/liveStreams (page footer: "Last updated 2026-10-15 UTC"), section `status.healthStatus.configurationIssues[].type`:

   > **`noAudioStream`** — Reason: "No audio" — "The ingestion stream does not contain any audio streams, but it must contain exactly one audio stream."

   > **`noVideoStream`** — Reason: "No video" — "The ingestion stream does not contain any video streams, but it **must contain exactly one video stream**."

   > **`multipleAudioStreams`** — Reason: "Check audio settings" — "The ingestion stream contains multiple audio streams, but it must only contain one audio stream."

   These are the documented ingest expectations: EXACTLY ONE audio stream + EXACTLY ONE video stream. Our app's current wire form (video-only, `audio:0KiB` in the 20261024-160137 incident) contradicts the audio half of this documented expectation — a direct, official hit for candidate cause **C1**. The incident's mirror observation (`ready` → `inactive`, Studio "No data") matches the documented semantics: "`inactive` – The stream is in inactive state which means the user is not receiving data via the stream."

2. **Severity caveat (honesty rule for all records)**: the same page defines the severity enumeration (`info` – broadcast with no adverse effect; `warning` – performance not optimal; `error` – "**The video cannot be broadcast to viewers**") but the final per-issue assignment ("severity of `noAudioStream` = error") is NOT published as a static table on any fetched official page (the companion page §1.1b lists type/reason/description only). Therefore: the OFFICIAL record is "the ingest must contain exactly one audio stream (documented configuration expectation)"; whether YouTube provably REFUSES to lock ingest on an audio-less stream is the probe's job (§2). No docs page may claim the severity verdict as an official fact.

1b. **Companion page, `Configuration Issues for LiveStream Resources`** — https://developers.google.com/youtube/v3/live/docs/liveStreams/health_status_messages (footer "Last updated 2026-10-15 UTC") lists the same `noAudioStream`/`noVideoStream`/`audioCodec`/`audioSampleRate`/`bitrateLow`/`videoIngestionStarved` entries (type + reason + description; no severity column). Duplicates the quotes above; cited for completeness because the app's operator docs link this page's subject matter.

### 1.2 (ii) Is an audio track REQUIRED/recommended? — OFFICIAL ANSWER: required at the ingest-expectation level; recommended parameters are AAC, 44.1/48 kHz, 128 Kbps stereo

1. **YouTube Help, "Choose live encoder settings, bitrates, and resolutions"** — https://support.google.com/youtube/answer/2853702 ("Encoder settings" table):

   > "Audio codec: **AAC or MP3** (5.1 surround sound audio is only supported for AAC in RTMP/RTMPS)"
   > "Audio sample rate: **44.1 KHz for stereo audio, 48 KHz for 5.1 surround sound**"
   > "Audio bitrate: **128-Kbps for stereo** or 384 Kbps for 5.1 surround sound"

2. Same page, "Recommended advanced settings" — the bitrate table row relevant to our locked vector:

   > "Recommended bitrate setting ranges are based on video ingestion codec, video ingestion resolution and frame rate." → "1080p @30fps | Minimum Bitrate Setting (Mbps) H.264: **5 Mbps** | Recommended Bitrate Setting (Mbps) H.264: **14 Mbps**"

3. Same page's explicit test guidance (corroborates BOTH causes):

   > "Make sure to test before you start your live stream. Tests should include **audio and movement in the video** similar to what you'll be doing in the stream. During the event, monitor the stream health and review messages."

   Google's own tested-signal expectation is "audio AND movement" — our incident stream had NEITHER (video-only all-black at ~18.55 kbit/s effective vs the configured 6000k; x264 QP floor `Avg QP: 0.08`).

4. Reconciliation note for the record: the encoder-settings page's audio-sample-rate line reads "44.1 KHz for stereo audio" while the `liveStreams` health issue `audioSampleRate` states — "The current sample rate is `%(actual)`. The recommended sample rates are **44.1kHz and 48kHz**." Both 44.1 and 48 kHz sit inside the officially recommended set. All probe/audio-remedy constants in this plan use 48 kHz stereo 128k (the officially recommended values AND the candidate remedy's documented constants) — no conflict.

### 1.3 (iii) Documented minimum-bitrate / signal-complexity expectations — OFFICIAL: minimum setting 5 Mbps (video, 1080p30 H.264); keyframes ≤ 4 s; CBR; health issues name low-output/starvation explicitly — but NO documented per-pixel-content minimum

1. **Encoder settings page** (https://support.google.com/youtube/answer/2853702):
   > "Keyframe frequency: **Recommended 2 seconds, Do not exceed 4 seconds**"
   > "Bitrate encoding: **CBR**"
   > Protocol: "RTMP/RTMPS Streaming"; Video codec "H.264"; 1080p30 H.264 minimum **5 Mbps** / recommended **14 Mbps**.

   Our app's locked vector (libx264, 6000k, g 60 = 2 s at 30 fps, yuv420p, flv, preset veryfast) satisfies the SETTING-level requirements. The incident's ~18.55 kbit/s EFFECTIVE output is the encoder hitting its QP floor on a zero-information source — that is `bitrateLow`-shaped but is a CONTENT property, not a settings violation.

2. **`liveStreams` health issues** (https://developers.google.com/youtube/v3/live/docs/liveStreams):
   > **`bitrateLow`** — Reason: "Video output low" — "The stream's current bitrate `(%(actual)s)` is lower than the recommended bitrate. We recommend that you use a stream bitrate of `%(expected)s`."
   > **`videoIngestionStarved`** — Reason: "Video output low" — "YouTube is not receiving enough video to maintain smooth streaming. As such, viewers will experience buffering."

   These document that output-bitrate expectations exist in the health system; they do NOT document a content-complexity minimum for the ingest signal lock. C2 therefore remains PROBE-GATED (cannot be confirmed or excluded from official text).

3. **Concatenation rule (validates the probe destination shape)** — same page, `cdn.ingestionInfo.rtmpsIngestionAddress`:
   > "Depending on which application or tool you use to encode your video stream, you may need to enter the stream URL and stream name separately or you may need to concatenate them in the following format: **STREAM_URL/STREAM_NAME**"

   Matches the app's `rtmpsIngestionAddress/streamName` shape exactly and the probe commands' destination form below.

### 1.4 Community corroboration (CLEARLY LABELED COMMUNITY — never cited as official)

1. **Stack Overflow 38944747** ("YouTube isn't receiving any data from my FFmpeg stream to the RTMP server", 2020-11-09) — https://stackoverflow.com/questions/64745099/ — the incident der Arst: video-only FFmpeg → RTMP stream; answer verdict "**YouTube needs audio.**" with the fix adding `-f lavfi -i anullsrc -c:a aac`. Community: matches the full incident shape (`ready`-ish → not receiving data).
2. **Stack Overflow 38944747's near-twin SO 38944747-era classic** ("YouTube live says not receiving data", 2016-08-14) — https://stackoverflow.com/questions/38944747/ — operator symptom: "In the live dashboard on YouTube I get a green *"Starting"* briefly but then it goes grey to say it's not receiving data" for video-only ingest; answer (llogan): "You can add silent audio with the anullsrc audio filter" (`anullsrc=r=16000:cl=mono` example). Community: same discriminator, silent-audio as the working pattern.
3. **Stack Overflow 39660547** ("How to Live Stream YouTube without Audio", 2016-09-23) — https://stackoverflow.com/questions/39660547/ — video-only feeds "can't stream to YouTube"; "From what I understand, YouTube define stream as audio-video content". Community.
4. **Stack Overflow 12368151** ("adding silent audio in ffmpeg", 2012-09-11) — https://stackoverflow.com/questions/12368151/ — the canonical silent-audio command shape (validates our probe ii syntax form):
   > "ffmpeg -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=44100 -i video.mov -c:v copy -c:a aac -shortest output.mov"
5. **GitHub gist stiltr/`bd8ace06b86ad1bec18de2718d3abba0`** — "Streaming to the new YouTube Studio with ffmpeg and nginx-rtmp" — environment-exception evidence (older Studio flow): dashboard went "No data" → "Excellent connection" but the stream never went live until switching ingest flows. Community: supports the "environment/flow-level" bucket; dated, Studio changed since — weight accordingly.

### 1.5 FFmpeg-side verification (for the probe commands' correctness)

1. **`anullsrc` filter source** (FFmpeg master, `libavfilter/asrc_anullsrc.c`) — https://github.com/FFmpeg/FFmpeg/blob/master/libavfilter/asrc_anullsrc.c:
   - Option names (aliases in parens): `channel_layout` (`cl`, default `"stereo"`), `sample_rate` (`r`, default `44100`), `nb_samples` (`n`, default 1024), `duration` (`d`, default `-1` = unlimited).
   - Filter description: `"Null audio source, return empty audio frames."` — an INFINITE silence generator unless `duration` is set.
   - The long option names `channel_layout=stereo:sample_rate=...` are the portable form (the filters doc's own examples use the short aliases `anullsrc=r=48000:cl=4`, `anullsrc=r=48000:cl=mono` — https://ffmpeg.org/ffmpeg-filters.html#anullsrc, §9.6). **Locked convention: all probe/remedy commands use the LONG names** (`channel_layout`, `sample_rate`) for clarity; both forms are documented-valid.
2. **`-shortest` semantics** — FFmpeg CLI manual (https://ffmpeg.org/ffmpeg.html, §5.11 Advanced options):
   > "`-shortest` (*output*) — Finish encoding when the shortest output stream ends."
   > "Note that this option may require buffering frames, which introduces extra latency. The maximum amount of this latency may be controlled with the `-shortest_buf_duration` option."

   Applied to our remedy/probe shape (infinite `anullsrc` audio + FINITE video): when the video stream ends it is the shortest stream, so `-shortest` terminates the whole output — the infinite audio is cut. That is the documented mechanism making the stdin-EOF shutdown bounded (Task 2's shipping contract). Officially documented caveat is only the buffering/latency note; NO FLV/RTMP-specific incompatibility is documented. Task 2 must still pin the with-audio vector offline (loopback) per the global plan before shipping; nothing in the research licenses skipping the pin.
3. **`lavfi` input device availability in common Windows builds** — gyan.dev builds page (https://www.gyan.dev/ffmpeg/builds/):
   > "All variants contain **all internal components available for Windows**."

   `lavfi` is an internal libavformat/libavfilter input device (NOT in the page's "external libraries" lists, which enumerate only third-party libs), so it is present in both `essentials` and `full` gyan.dev variants. Corroboration: probe ii/iii's anullsrc+AAC path uses only internal components (native `aac` encoder, `libx264` is present in all gyan.dev variants per the essentials libraries list "…libx264 libx265…"). **Operator-side availability gate (mandatory preflight, §2.4): `ffmpeg -devices` must list `lavfi` and the local no-network smoke must exit 0.** If an operator runs an exotic/trimmed FFmpeg build where this fails, the probe is aborted at preflight and the observation recorded (environment bucket).

### 1.6 Research verdict for Task 2's gate (+ what was NOT researched)

- **Verdict:** C1 (audio-track requirement) is SUPPORTED by official documentation ("must contain exactly one audio stream" — `noAudioStream`) and corroborated by every community case found; C2 (degenerate-signal rejection) is PROBE-GATED (the official pages document bitrate/health expectations but no content-complexity lock rule; the "audio **and movement**" test guidance is consistent with it). The probe procedure (§2) is therefore REQUIRED equipment regardless: it separates C1 from C2 from environment from connection cheaply and authoritatively, with the operator's own hands and eyes.
- **Gate posture for Task 2 (preliminary, re-adjudicated at Task-2 4.1b):** official research alone leans CONFIRM for the silent-audio remedy (the documented ingest expectation is exactly-one-audio-stream; the remedy constants 48 kHz stereo 128k sit inside the official recommended set). Per the global plan's pause rule, Task 2's formal gate still consumes the OPERATOR's probe results before any implementation decision is finalized — the workflow pauses at Task-2 4.1b and asks the user to run §2's procedure and report the outcome. SKIP remains a first-class outcome if the probe contradicts the research (e.g. probe i active ⇒ C2, audio unnecessary).
- **NOT researched / consciously out of reach:** (a) the per-issue severity assignment for `noAudioStream` (not published as a static table on any fetched official page — probe answers it empirically); (b) whether YouTube's CURRENT Studio-era RTMP accept path can promote a video-only stream to `active` against the documented expectation (all community evidence is 2016–2020; the probe is the current-era test); (c) YouTube's internal stream-health algorithm/thresholds beyond the documented issue list; (d) the `liveBroadcasts` resource reference was not directly consulted for ingest requirements — the audio/bitrate/stream-health material lives in the `liveStreams` resource (`status.healthStatus.configurationIssues`) and the help-center encoder-settings page, not in the lifecycle-focused `liveBroadcasts` fields; if a reviewer demands it, fetch https://developers.google.com/youtube/v3/live/docs/liveBroadcasts and verify before recording; (e) gyan.dev's per-build README filter roster beyond the "all internal components" page statement (the preflight command is the binding availability check).

---

## 2. Deliverable 1 — `docs/production-run.md` edit spec (the probe procedure)

### 2.1 Structural edits (exact anchors against the current 161-line file)

The doc is 161 lines with a TOC (lines 5–15) in the established anchor style. Three edits, in this order:

**Edit A — TOC entry.** After the line:

    - [Broadcast/Stream Fate (Policy and Honest Limitations)](#broadcaststream-fate-policy-and-honest-limitations)

insert the new TOC line (anchor derived exactly like the existing `(...)→#...-operator-run` precedent of "Manual Verification Procedure (Operator-Run)" → `#manual-verification-procedure-operator-run`):

    - [Ingest Probe Procedure (Operator-Run)](#ingest-probe-procedure-operator-run)

**Edit B — new H2 section.** Insert the entire new section (§2.3) immediately AFTER the last common-ingestion/API-failures bullet (the transient backend/network bullet ending `... then re-run.`) and BEFORE the existing line:

    ## Manual Verification Procedure (Operator-Run)

Exact anchor: the inserted section starts with `## Ingest Probe Procedure (Operator-Run)` on its own line, separated from the previous bullet list by one blank line, and ends with one blank line before `## Manual Verification Procedure (Operator-Run)`. This places the procedure directly under the operator-diagnosis area (the readiness-timeout diagnosis step list that precedes it in the "Broadcast/Stream Fate" section is the diagnostic area the caller referenced).

**Edit C — pointer line in the existing diagnosis procedure.** In the readiness-timeouts diagnosis step list, the step currently reading (bottom of the file's diagnosis area):

    2. `…last observed streamStatus` naming another label (`inactive`, `error`, `created`) — treat it per the documented `liveStreams` status semantics; the `error` label points at the stream's own diagnostics.

is extended to (single line, only the sentence appended; nothing else in the item changes):

    2. `…last observed streamStatus` naming another label (`inactive`, `error`, `created`) — treat it per the documented `liveStreams` status semantics; the `error` label points at the stream's own diagnostics; for the publish-succeeded-but-inactive/`No data` shape (FFmpeg exited cleanly with the full frame budget), run the Ingest Probe Procedure below before re-running.

### 2.2 Structural conventions for the new section (mandatory)

- H2 heading only; NO H3 subsections (the document currently uses H2 sections + bullets/bold lead-ins exclusively — match that style).
- The three commands in fenced code blocks (the doc already uses fenced blocks, e.g. the shutdown message) WITH the `<RTMPS_INGEST_URL>/<STREAM_KEY>` placeholder pair — NEVER an example-looking real URL or key-shaped string (no `rtmps://a.rtmp.youtube.com/live2/xxxx` anywhere).
- Keep total added length modest (~60–75 lines); the doc stays well under the file-size guidance that applies only to `src/` anyway.

### 2.3 New-section content spec (the implementer composes the prose from this skeleton)

```
## Ingest Probe Procedure (Operator-Run)

Intro paragraph (must state ALL four of these facts, in any sensible order, ~3–5 sentences):
- What it discriminates: WHY a run whose FFmpeg publish succeeded at the protocol level
  (clean exit status 0 with the full frame budget) still leaves the stream inactive and
  YouTube Studio showing "No data": C1 the ingest expects an audio track; C2 the streamed
  content itself fails the ingest signal lock (e.g. a near-zero-bitrate all-black source);
  environment levels (FFmpeg build capability, local network/firewall); connection/URL level.
- This procedure is OPERATOR-RUN: the agent did not execute it, never will, and probe
  results are recorded only by the operator — never as build evidence.
- The probe results feed the implement-or-skip decision of the ingest-fix task's research
  gate (recorded in this work's TODO; an operator report back is the expected next step).
- Background fact, cited, kept word-exact: YouTube's Live Streaming API documents the
  configuration issue `noAudioStream` — "The ingestion stream does not contain any audio
  streams, but it must contain exactly one audio stream."
  (https://developers.google.com/youtube/v3/live/docs/liveStreams) — and the encoder-settings
  page instructs that pre-stream tests "should include audio and movement in the video"
  (https://support.google.com/youtube/answer/2853702).

**Secret-safety rules (read before touching the shell):**
- The stream key is pasted ONLY into the local command line of a terminal you control.
  It must NEVER appear in any log file, documentation file, issue, screenshot, or commit.
  The placeholders below (`<RTMPS_INGEST_URL>` and `<STREAM_KEY>`) are replaced by you at
  run time; the command with the real key is never copied into any persistent artifact.
- Prefer a plain `cmd.exe` window (no persistent history); if your shell persists history
  (e.g. PowerShell's PSReadLine), clear or start a fresh session afterwards.
- Each probe ends BY ITSELF after 60 s (`-t 60`): no manual kill needed, no orphan process.

**Get a probe destination (your secret):**
- In YouTube Studio (CREATE → Go live), open the Stream tab, create fresh stream settings
  (or reuse an existing one you are willing to delete afterwards), and copy BOTH the stream
  URL (an `rtmps://…` server URL) and the stream key. The FFmpeg destination is the
  concatenation `stream URL + "/" + stream key` — the same shape the application assembles
  from the API's `rtmpsIngestionAddress` and `streamName` fields, per the official form
  "STREAM_URL/STREAM_NAME" (https://developers.google.com/youtube/v3/live/docs/liveStreams,
  `cdn.ingestionInfo.rtmpsIngestionAddress`).
- Keep Studio's stream tab OPEN on a visible surface: the probe outcomes are read there.

**Preflight (local, no network, no secrets):**
- `ffmpeg -devices` — confirm the device list contains `lavfi` (the lavfi input device is an
  internal FFmpeg component present in standard Windows builds; gyan.dev's builds include
  "all internal components available for Windows" per https://www.gyan.dev/ffmpeg/builds/).
  If `lavfi` is absent, stop: your FFmpeg build cannot run this procedure; record that fact
  (environment-level evidence) instead of improvising.
- `ffmpeg -f lavfi -i "testsrc2=size=160x120:rate=30:duration=3" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:a aac -shortest -f null -` —
  expect a clean exit 0 with no output file. This validates the exact filter/lavfi/`-shortest`
  syntax shape below against YOUR installed FFmpeg build before any network push.

**Probe i — rich source, video-only (`testsrc2` at the app's locked streaming-vector parameters):**

```bash
ffmpeg -re -f lavfi -i "testsrc2=size=1920x1080:rate=30" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Options mirror the application's streaming vector exactly (libx264, veryfast, 6000k, GOP 60
frames = the 2-second-target keyframe at 30 fps, yuv420p, FLV muxer) at 1080p30; `-re` paces
the generated video at native rate; `-t 60` bounds the run. The video content moves continuously.

**Probe ii — black source + silent AAC audio (the candidate remedy's wire shape):**

```bash
ffmpeg -re -f lavfi -i "color=c=black:size=1920x1080:rate=30" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -c:a aac -b:a 128k -shortest -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Same video vector (all-black content = the application's frame content); adds a second lavfi
input — infinite silence at 48 kHz stereo (`anullsrc`, long option names
`channel_layout`/`sample_rate` per FFmpeg's `asrc_anullsrc` source) — encoded AAC at 128 kbit/s
(the officially recommended stereo audio bitrate/codec), cut by `-shortest` when the bounded
video input ends (FFmpeg: "Finish encoding when the shortest output stream ends.").

**Probe iii — rich source + silent AAC audio (control: separates environment/connection from signal):**

```bash
ffmpeg -re -f lavfi -i "testsrc2=size=1920x1080:rate=30" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -c:a aac -b:a 128k -shortest -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Same shape as probe ii with the moving `testsrc2` content — a positive signal in both halves.

**How to observe (during each 60 s run):** watch the Studio stream page: whether the ingest
indicator/preview shows incoming data (the exact wording YouTube shows varies — record what
you see, with the timestamp). A probe is ACTIVE when the preview/health indicator shows the
stream receiving data; a probe is INACTIVE when Studio keeps showing "No data".

**Outcome → cause mapping (each probe's result maps 1:1):**

| When... | Active probe(s) | Cause indicated | What it means for the fix task |
| --- | --- | --- | --- |
| i active | probe i only | **C2 — content/degenerate-signal** | video-only ingest of rich content locks; the all-black signal is the problem |
| ii active | probe ii only (i inactive) | **C1 — audio-track** | the ingest locks once a (silent) audio track exists, even on black video — the silent-audio remedy is indicated |
| iii active | probe iii (i and ii inactive) | **environment-level** | encoder/network/firewall/build capability: no app change warranted |
| none active | no probe | **connection/URL-level** | key/URL assembly, Studio stream state, or account/channel-level: re-check the key and the stream's readiness before deeper diagnosis |

Record each outcome with the timestamp and the exact Studio wording. If ALL probes are
inactive, also re-run probe i once with a freshly copied URL+key before concluding
connection/URL-level.

**Cleanup after the probes:**
- Delete the probe's stream/broadcast pair in YouTube Studio (the same hygiene the production
  fate policy requires of failed `run` attempts) — do not let probe resources accumulate.
- If any probe accidentally reached the live state, let it end/stop it, and verify the
  broadcast completed in Studio.
- Remember: the key was only ever on your local command line — nothing to purge from this
  repository, and nothing to paste anywhere.
```

### 2.4 Implementation constraints on the section (mandatory)

- Every URL cited in the section is one of the three in §1.1–§1.5 (liveStreams API doc, encoder-settings help page, gyan.dev builds). No other links.
- The four quotes that MUST appear word-exact (they are the evidence-traceable anchors): the `noAudioStream` sentence; the "STREAM_URL/STREAM_NAME" phrase; the "audio and movement in the video" test phrase; the `-shortest` "Finish encoding when the shortest output stream ends." phrase.
- The three commands MUST be the exact strings from §2.3 (option order included). The only placeholder-replaceable part is the destination tail.
- No stream-key-shaped example text anywhere (the `<RTMPS_INGEST_URL>/<STREAM_KEY>` pair is the ONLY destination spelling in the doc).

---

## 3. Deliverable 2 — `.agent/project-info/context.md` record spec (truthful, secret-free)

### 3.1 Placement & shape

- Add ONE new block with the heading `## Recent Changes (2026-10-24, ingest "No data" — Task 1 research + probe procedure)` immediately after the LAST existing "## Recent Changes" block in the file (i.e. after the 2026-10-23 made-for-kids Task-2 block ends and before the next heading — keep every existing block byte-untouched).
- Follow the file's established block style: one bulleted paragraph set (bullets, dense factual sentences, `code` spans for paths/commands/labels, no comments, no TODOs).
- Do NOT touch "Current Work Focus", "Immediate Next Steps", "Known Deviations", "Scope Decisions", or any other block (the caller's planner refreshes those in later steps).

### 3.2 Block skeleton (the implementer fills the placeholder slots with the ACTUAL outcomes of §4; never claim unrun steps)

```
## Recent Changes (2026-10-24, ingest "No data" — Task 1 research + probe procedure)

* Trigger (operator log fact, `logs/run/20261024-160137.log`, never agent-observed): the third native Windows
  `run` attempt (v0.10.0) streamed the full confirmation budget (3801 frames ≈ 126 s at 30 fps, speed=0.999x)
  and FFmpeg exited 0 — the RTMPS publish succeeded at the protocol level — while YouTube's stream went
  `ready` → `inactive` and Studio showed "No data"; the encoded signal was DEGENERATE: video-only
  (`audio:0KiB`) at ~18.55 kbit/s against the configured 6000k (x264 QP floor `Avg QP: 0.08`). Two ranked
  candidates: C1 audio-track requirement; C2 degenerate-signal rejection. Branch `fix/ingest-no-data` (v0.11.0
  bump at HEAD `983293e`); global plan `.kilo/plans/20261024-ingest-no-data-audio-signal.md`; TODO
  `.agent/todos/20261024/20261024-todo-1.md` Task 1; per-task plan `.kilo/plans/20261024-ingest-no-data-task1-implementation.md`.
* Task 1 is DOCS + RESEARCH ONLY (no behavior change; no src/config/dependency edits). Research verdict recorded
  with official citations in the per-task plan §1:
  OFFICIAL — the Live Streaming API's `liveStreams` health-status issue list documents `noAudioStream`
  ("The ingestion stream does not contain any audio streams, but it must contain exactly one audio stream.") and
  `noVideoStream` ("…must contain exactly one video stream.") per
  https://developers.google.com/youtube/v3/live/docs/liveStreams ("Last updated 2026-10-15 UTC") — the documented
  ingest expectation is EXACTLY ONE audio stream + one video stream, so a video-only RTMPS ingest is NOT a
  documented-compliant configuration (C1 supported at the documentation level); the companion page
  https://developers.google.com/youtube/v3/live/docs/liveStreams/health_status_messages lists the same issues'
  reasons/descriptions; no official page publishes a per-issue severity table, so whether YouTube provably refuses
  to lock an audio-less ingest stays the operator probe's empirical question.
  OFFICIAL — the encoder-settings page https://support.google.com/youtube/answer/2853702 recommends audio codec
  "AAC or MP3", audio sample rate 44.1 KHz (stereo) / 48 KHz (5.1), audio bitrate "128-Kbps for stereo", bitrate
  encoding CBR, keyframe frequency "Recommended 2 seconds, Do not exceed 4 seconds", and 1080p30 H.264 with a
  5 Mbps minimum / 14 Mbps recommended bitrate SETTING; its test guidance requires tests "should include audio and
  movement in the video" — both half-causes of the incident (no audio, no movement) sit in Google's own tested-signal
  expectation. Health issues `bitrateLow` ("Video output low") and `videoIngestionStarved` document low-output
  expectations; NO content-complexity ingest-lock rule is documented — C2 remains PROBE-GATED.
  COMMUNITY (labeled, never official): video-only FFmpeg→YouTube RTMP ingest showing "No data"/not-receiving-data
  with silent audio (`anullsrc`) as the working pattern — stackoverflow.com/questions/64745099 ("YouTube needs
  audio."), 38944747, 39660547, and the canonical silent-audio command form in 12368151. No positive community
  evidence of a video-only ingest succeeding was found in the sample.
* FFmpeg verification (official sources): `anullsrc` options `channel_layout` (alias `cl`, default `stereo`) and
  `sample_rate` (alias `r`, default 44100) per libavfilter/asrc_anullsrc.c; the filters doc examples use the short
  aliases (`anullsrc=r=48000:cl=mono`); `-shortest` = "Finish encoding when the shortest output stream ends."
  (ffmpeg.html §5.11; the documented caveat is buffering/latency via `-shortest_buf_duration`; no FLV/RTMP-specific
  incompatibility documented); the `lavfi` input device is an internal FFmpeg component present in standard
  gyan.dev Windows builds ("all internal components available for Windows", gyan.dev builds page) — the operator
  preflight (`ffmpeg -devices` listing `lavfi` + a local `-f null` smoke) is the binding availability check.
* Deliverable 1: `docs/production-run.md` gained the operator probe procedure as a new H2 section
  `## Ingest Probe Procedure (Operator-Run)` (inserted between the common-ingestion-failures list and the Manual
  Verification Procedure section, with the matching TOC entry) — three exact manual FFmpeg commands with the
  locked streaming-vector parameters (1080p30 testsrc2/color libx264 veryfast 6000k g 60 yuv420p flv; probe i
  rich video-only; probe ii black + silent 48 kHz stereo AAC 128k via `anullsrc` + `-shortest`, bounded `-t 60`;
  probe iii rich + audio control) against the operator-supplied `rtmpsIngestionAddress/streamName` destination
  (placeholders only, key NEVER in any doc/log/secret surface); the preflight (`ffmpeg -devices` + local
  `-f null` smoke); the outcome→cause mapping table (i active ⇒ C2 content; ii active ⇒ C1 audio; only iii
  ⇒ environment; none ⇒ connection/URL-level with the fresh-key re-check); secret-safety rules; cleanup (delete
  the probe stream/broadcast pair per the fate-policy hygiene); the pointer sentence appended to the readiness-
  time-out diagnosis step 2. Also verified: TOC/anchor consistency (`#ingest-probe-procedure-operator-run`).
* Deliverable 2: this context block records the verdict truthfully; secret-free (`ya29`/`AIza`/`key=` zero hits apart
  from pre-existing doc-context counts).
* Task-2 gate posture: official research leans CONFIRM for the silent-audio remedy (documented exactly-one-audio-stream
  ingest expectation; remedy constants 48 kHz stereo 128k inside the recommended set), but per the global plan's
  pause rule Task-2 4.1b still consumes the OPERATOR's probe results (the workflow pauses there and asks the user to
  run the new procedure and report outcomes; SKIP stays a first-class outcome if the probe contradicts the research,
  e.g. probe i active ⇒ C2).
* Verification (this task, 4.2): gitignore/hygiene checks (§4.1 below) CLEAN; ONE standard dev-checks gate run on the
  final docs tree — [GATE LOG PATH] exit 0, [TEST COUNT] passed / 0 failed, `ALL CHECKS PASSED`; NO builds in Task 1
  (docs-only, binary unchanged) — both release-artifact rebuilds + `ls -l` cross-check are deferred to Task-2's cycle
  end (TODO criterion 5 spans the whole TODO). Commit(s): [COMMIT HASH LIST].
```

### 3.3 Record honesty rules

- All `[PLACEHOLDER]` slots get filled from actually-executed steps only; a step not executed leaves its slot as an explicit `NOT RUN` marker (never invented).
- No stream keys, no RTMPS URLs, no tokens, no machine-specific paths beyond the repo-root-relative ones the file convention already uses.
- The block states the probe results as RECORDED-BY-OPERATOR-LATER — the agent never claims them.

---

## 4. Verification spec (4.2 implementer — cheap and exact)

### 4.1 Git hygiene + gitignore compliance (MANDATORY, in this order, before the commit)

1. `git status --porcelain` → expect EXACTLY: ` M docs/production-run.md`, ` M .agent/project-info/context.md`, `?? .kilo/plans/20261024-ingest-no-data-task1-implementation.md` (the plan file; the planner tracks it per repo practice `.../plans/ (per-task implementation plans tracked)` in `.agent/project-structure.md` — leave it UNTRACKED for the implementer; the planner commits it). If anything else appears, STOP and report (§6).
2. `git check-ignore -v logs/chat.log` → expect a `.gitignore` hit (confirming nothing related leaked); `git ls-files videos/ dist/ credentials/ config/config.json` → expect empty/`.gitkeep`-only (no new ignored-path content ever staged).
3. Confirm NO `.gitignore`-matching path is staged: `git diff --cached --name-only` after staging shows only the two docs files.
4. Read `.gitignore` once before committing (rule compliance) and verify the two edited paths are tracked files (both are).

### 4.2 Markdown consistency check (light, no tooling beyond grep/read)

1. TOC: the new entry `- [Ingest Probe Procedure (Operator-Run)](#ingest-probe-procedure-operator-run)` sits directly after the Broadcast/Stream Fate entry and BEFORE the Manual Verification entry; heading text and anchor match exactly (lowercase, spaces→hyphens, parentheses dropped — mirroring the existing `manual-verification-procedure-operator-run` precedent).
2. Secret scan over the changed files: grep the diff for `rtmps://` (expect only inside the `<RTMPS_INGEST_URL>/<STREAM_KEY>` discussion or none at all), and confirm `<STREAM_KEY>` is the only key-adjacent token; assert no 32-or-longer alnum strings that could pass for a key (visual scan of the diff suffices — the placeholders are bracketed tokens).
3. Verify the three exact command strings against §2.3 verbatim (option order, quoting, placeholder pair).
4. File line counts untouched: `docs/production-run.md` grows to ~220–235 lines (TOC keeps >100-line docs compliant); `context.md` grows by one block. No other file changes.

### 4.3 Standard gate (ONE run on the final tree — division encoded per the caller)

- The Phase-05 Task-4 docs pass precedent ran the standard gate on a docs-only tree for recorded evidence (355 passed on a docs tree, `logs/checks/20261014T213126Z.log`) — FOLLOW THE PRECEDENT.
- Run via the Alpine VM MCP, standard invocation: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — expect exit 0, four PASS lines (`fmt-check`, `check`, `test`, `clippy`), summary `ALL CHECKS PASSED`, and the observed test count (current expected neighborhood: 378; record the OBSERVED count).
- Record: the UTC log path under gitignored `logs/checks/` + the observed test count → goes into the context block's verification slot.
- **NO builds in Task 1** — division: TODO criterion 5 ("Both release artifacts rebuilt with `ls -l`-cross-checked sizes; version bumped…") spans the WHOLE TODO and is assigned to Task-2's cycle end (or workflow step 5 if Task 2 skips). The docs-only change cannot move the binary; do not run `build-linux.sh`/`build-windows.sh`, do not `ls -l dist/`.

---

## 5. Commits (4.2 implementer)

1. Stage exactly: `docs/production-run.md` + `.agent/project-info/context.md`.
2. ONE commit, repo style (`docs:` prefix, concise):
   - `docs: add ingest probe procedure and record ingest-requirement research`
3. Do NOT commit the plan file (planner tracks it), do NOT mark the TODO `[DONE]` (4.6 owner), do NOT push (step 5 owner), do NOT touch `.agent/project-structure.md` in 4.2 (4.4 owner — see below).

### 5.1 Downstream ownership map (for the caller — do not let any owner merge into another's step)

| Step | Owner | Content |
| --- | --- | --- |
| 4.2 | implementer | §2 (production-run.md + §2.1 A/B/C edits), §3 (context.md block), §4 (hygiene + ONE gate), §5 commit |
| 4.3 | code-reviewer + code-simplifier | review the docs diff against §2/§3 specs (quotes word-exact, commands verbatim, anchors consistent, no secrets); simplifier edits are prose-only, never command text; both save fix/simplification plans to `.kilo/plans/` when fixes are required |
| 4.4 | docs-specialist | drift-check `README.md`, `docs/configuration.md`, `docs/authentication.md`, `docs/google-cloud-prerequisites.md`, `docs/demo.md`, `docs/build.md`, `docs/build-windows.md` against the new section (expected: no drift except possibly README's test-count line AFTER the 4.2 gate run — if the gate observed count differs from README's number, update it; that's the only README touch permitted); refresh `.agent/project-structure.md`'s `docs/` paragraph entry for `production-run.md` to include "the operator ingest probe procedure (three FFmpeg probes + outcome→cause mapping; C1/C2/environment/connection discrimination)" |
| 4.5b | architector | adherence report vs THIS plan |
| 4.6 | implementer | add `[DONE]` to Task 1 heading in the TODO; flip criteria 1 and 2 to `[x]`, and criterion 4 `[x]` (gate green on the final tree with recorded log path + observed test count; demo/auth and the Task-1 diagnostics byte-stable — the docs-only change guarantees the byte-stability clause trivially); annotate criterion 3 as Task-2-governed (research gate posture recorded in context.md, decision lands in Task 2); annotate criterion 5 as deferred to Task-2's cycle end per §4.3; commit `docs: mark ingest no-data task 1 complete` or per repo style |

---

## 6. Stop-and-report conditions (every holder of this plan)

Stop and return the question to the caller (never improvise) when ANY of:

1. §4.1's `git status` shows paths beyond the three expected entries.
2. The production-run.md anchors from §2.1 no longer match (the file changed since this plan).
3. A mandated quote from §2.4 fails to reproduce from the live cited pages (do not paraphrase — ask).
4. The standard gate (§4.3) exits non-zero on a docs-only tree (a docs-only change never breaks the gate — treat any failure as environment trouble to report).
5. Any scenario requires touching `src/`, `config/`, `scripts/`, `Cargo.*`, or the demo/auth byte-stability guarantees — all out of scope for Task 1.
6. The implementer is tempted to RUN the probes (they are operator-run only; the agent has no stream key and must never acquire one).

---

## 7. Traceability

- TODO Task-1 bullets covered: research bullets → §1 (recorded in plan + context block); probe-procedure bullet → §2/§3; context-recording bullet → §3.
- TODO acceptance criteria coverage: criterion 1 (research recorded, official vs community separated) → §1 + §3; criterion 2 (production-run.md carries the procedure; no key anywhere) → §2 + §4.2; criterion 3 → Task-2-governed (§1.6 posture + §5.1 annotation rule); criterion 4 → §4.3 gate division (the byte-stable clause is trivially satisfied; recorded evidence at 4.2/4.6); criterion 5 → deferred to Task-2's cycle end per the caller's explicit division (§4.3); criterion 6 → §3 (structure map refresh owned by 4.4) + no real-Google claims anywhere (§2.3 intro + §3.3).
- Global-plan alignment: the three probes, the outcome mapping, the destination shape, the anullsrc/-shortest mechanism, the secret-safety rules, the probe-is-research-gate-input note, and the skip-is-first-class posture all carry over verbatim in intent (global plan "Encoded Technical Direction" + risk items).
- Honesty ledger: severity of `noAudioStream` unpublished (probe-answered); community evidence dated (probe is the current-era test); `-shortest` mechanism documented, FLV-compat unknown-but-unflagged (Task 2 still loopback-pins the vector); NO real-Google verification is performed or claimed by anyone in this task.
