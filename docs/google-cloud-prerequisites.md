# Google Cloud Prerequisites (OAuth 2.0)

This document explains the Google Cloud setup that [`docs/authentication.md`](authentication.md) lists as prerequisites for the `auth` command: **why** each item exists, **which alternatives do not work**, the **step-by-step setup**, and the **cost/quota model**. It is a prerequisite companion to the authentication guide — the flow itself, credential files, and failure semantics live there.

## Table of Contents

- [Overview](#overview)
- [The Requirements, Term by Term](#the-requirements-term-by-term)
- [Why Not an API Key, a Service Account, or Just a Browser?](#why-not-an-api-key-a-service-account-or-just-a-browser)
- [Step-by-Step Setup](#step-by-step-setup)
- [Costs and Quotas](#costs-and-quotas)
- [Troubleshooting](#troubleshooting)
- [Handling the Credentials Securely](#handling-the-credentials-securely)
- [Verification Checklist](#verification-checklist)
- [See Also](#see-also)

## Overview

The service authenticates as **you** (the YouTube channel owner) to operate on your own channel. Google grants that access through OAuth 2.0, and Google only issues OAuth credentials to **applications registered in a Google Cloud project**. There is no path that avoids this registration — it is a platform requirement, not an implementation choice.

Three artifacts must exist before `auth` can run:

1. a Google Cloud **project**,
2. the **YouTube Data API v3** enabled in that project,
3. a **Desktop-app OAuth client** whose downloaded JSON is placed at the configured `oauth.client_config_path`.

## The Requirements, Term by Term

### Google Cloud project

Google issues API access and OAuth clients **per project**. A project is the container that binds together the enabled APIs, the OAuth consent screen, and the OAuth client. The client and the enabled API must live in the **same** project: a request made with an OAuth client whose project does not have the API enabled fails with `403 accessNotConfigured` ("YouTube Data API has not been used in project ... or it is disabled").

Creating the project itself is free and requires no billing account.

### YouTube Data API v3 enabled

Every operation this service performs is a **YouTube Data API v3** operation:

| Operation | Used by | Phase |
| --- | --- | --- |
| `channels.list` (`part=id&mine=true`) — authorization probe | reusable credential service | Phase 04 (implemented) |
| `liveBroadcasts` / `liveStreams` lifecycle | future production `run` backend | later phases |
| `liveChatMessages` / `liveChatMessages.streamList` | future live-chat integration | later phases |

Google's rule is explicit: *"Any application that calls Google APIs needs to enable those APIs in the API Console."* If the API is not enabled in the project that owns your OAuth client, the first API call fails.

### OAuth 2.0 client of type **Desktop app**

The OAuth client is the **application identity** — a `client_id` and `client_secret` pair that Google issues after registration. The browser is **not** a substitute: the browser is where *you* sign in and approve the permission, but Google confers that permission *onto the registered application*, not onto the browser session.

The **client type** determines which redirect mechanism is permitted:

| Client type | Redirect mechanism | Fits this service? |
| --- | --- | --- |
| **Desktop app** | Installed-application flow: loopback redirect `http://127.0.0.1:<random-port>` handled by a local listener; PKCE supported | **Yes** — this is what the service implements |
| Web application | Fixed, pre-registered HTTPS redirect URI on a public domain | No — the service has no public callback endpoint |
| Android / iOS / TV & Limited Input | Mobile/TV SDK flows | No |
| Service account | Machine identity; only for YouTube **content owners** via `onBehalfOfContentOwner` | No — see the alternatives section |

The service's `auth` command binds a listener on `127.0.0.1` (loopback only, ephemeral port), prints the authorization URL, and waits for Google to redirect the browser back to that loopback address. The downloaded JSON for a Desktop-app client carries `client_id`, `client_secret`, `auth_uri`, and `token_uri`; the app validates those endpoints against Google's fixed constants at startup.

## Why Not an API Key, a Service Account, or Just a Browser?

| Alternative | Why it cannot work here |
| --- | --- |
| **API key** | Google's own rule: an API key is for *"requests that do not require user authorization"* (public data, e.g. listing public videos). Every operation this service needs touches **private user data** and must carry an OAuth 2.0 token. The project brief also explicitly rejects an API key as a substitute for OAuth. |
| **Service account** | Supported only for **YouTube content owners** (CMS/MCN accounts) using the `onBehalfOfContentOwner` parameter. A normal channel owner authorizing a desktop application is not that case. |
| **Browser only** | The browser session proves *you* consented; it cannot identify the application or receive the authorization code. The authorization URL is built from the registered `client_id`, and exchanging the code requires the client secret plus the PKCE verifier that only the application holds. |
| **Reusing another tool's OAuth client** (e.g. a streaming application's) | Not possible — a client's credentials belong to the app registered against them; a different application cannot legitimately use them. |

## Step-by-Step Setup

Sign in to the [Google Cloud Console](https://console.cloud.google.com) with the Google account that owns the target YouTube channel.

1. **Create or select a project.** Project selector (top bar) → *New project*. Give it a recognizable name. No billing account is needed.
2. **Enable the YouTube Data API v3.** *APIs & Services → Library* → search `YouTube Data API v3` → *Enable*. After enabling, the API appears under *APIs & Services → Enabled APIs*.
3. **Configure the OAuth consent screen.** *APIs & Services → OAuth consent screen* now opens the **Google Auth Platform** dashboard (Branding / Audience / Data Access / Clients). If it is not configured yet, click *Get started* and complete **Branding** (app name, user support email) and **Audience**.
   - **Audience** → User type: **External**; then add the channel-owning account under **Test users**. While the app stays in **Testing** status only listed test users can authorize (up to 100 — far more than this single-operator service needs; publishing/verification is not required for private use).
   - **Where the scope goes — Data Access** (sidebar): `https://console.cloud.google.com/auth/scopes?project=<YOUR_PROJECT_ID>` → **Add or remove scopes**. In the *Update selected scopes* panel, either search and check the scope or paste `https://www.googleapis.com/auth/youtube.force-ssl` into the **Manually add scopes** box → **Add to table** → **Update** → **Save**. `youtube.force-ssl` is a *sensitive* scope, so it is normally NOT in the quick-select list — use the manual box. It is the minimum scope covering the broadcast/chat operations of later phases (see [`docs/authentication.md`](authentication.md)).
   - Note: scopes listed on the consent screen are the consent/verification record; the application still requests the scope at runtime.
4. **Create the OAuth client.** *APIs & Services → Credentials → Create credentials → OAuth client ID* → Application type **Desktop app** → Create. (Do **not** choose *Web application*.)
5. **Download the client JSON.** Use the download action for the new client. The file is named like `client_secret_<client-id>.json`.
6. **Place and configure the files.**
   - Save the downloaded JSON at the configured `oauth.client_config_path` (default `credentials/client_secret.json`).
   - Ensure `oauth.credentials_path` (default `credentials/tokens.json`) is a **different** path — configuration validation rejects identical or blank paths, because the issued tokens must never overwrite the client configuration.
   - Both files live under the gitignored `credentials/` directory (see [Handling the Credentials Securely](#handling-the-credentials-securely)).
7. **Run the authorization.** `rust-youtube-streamer-service auth`. Expected behavior:
   - The console prints the authorization URL and the loopback redirect it is waiting on.
   - Open the URL in a browser and sign in as the channel owner.
   - Google may show `Google hasn't verified this app` (normal while the consent screen is in Testing). Choose *Advanced* → *Go to \<app name\> (unsafe)* → approve the requested YouTube permission.
   - The app catches the loopback callback, exchanges the code (PKCE S256), validates the granted scope, saves the credentials atomically, prints one success line, and exits `0`.

Detailed console output, exit codes, and failure reasons are documented in [`docs/authentication.md`](authentication.md).

## Costs and Quotas

**Money: $0.** Creating the project, enabling the API, creating the OAuth client, and using the flow cost nothing. No billing account or credit card is required.

The real limit is the **quota**: each Data API request consumes quota *units*, and every project gets a default allocation of **10,000 units per day** (resetting at midnight Pacific Time). `search.list` and `videos.insert` have separate 100-calls-per-day buckets; this service uses neither.

Relevant per-method costs (from Google's Quota Calculator):

| Method | Units per call |
| --- | --- |
| `channels.list` (authorization probe) | 1 |
| `liveBroadcasts.*` / `liveStreams.*` (lifecycle) | 1 |
| `liveChatMessages.list` / `liveChatMessages.streamList` | 5 |
| `liveChatMessages.insert` | 20 |
| `liveChatBans.*` / `liveChatModerators.*` | 200 |
| `search.list` | 100-calls/day bucket |
| `videos.insert` | 100-calls/day bucket |

Consequences for this project:

- **`auth` and the startup probe are negligible** (~1 unit per authorization/verification).
- **Broadcast/stream lifecycle is cheap** (~1 unit per lifecycle call).
- **Live-chat reading is the quota driver.** `liveChatMessages.list` costs 5 units, so a continuous poll every 5 seconds would consume ≈17,000 units/day — over the default ceiling. The future chat phase must therefore pace polling within 10,000 units/day (≈2,000 list calls/day, i.e. no faster than roughly one call per 43 seconds) or use `streamList` (also 5 units per call, but far fewer calls), or request a quota extension.
- **More quota cannot be bought.** The only path above the default is the free, manually reviewed *YouTube API Services — Audit and Quota Extension* form; approval and timing are not guaranteed.
- Exhausting the daily quota returns `403 quotaExceeded` until the reset.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `403 accessNotConfigured` — "YouTube Data API has not been used in project ... or it is disabled" | The API is not enabled in the project that owns the OAuth client | Enable *YouTube Data API v3* in that exact project (step 2) |
| `Google hasn't verified this app` | The consent screen is in Testing status and the requested scope is sensitive | Expected for private use: *Advanced* → continue. Publishing/verification is only needed for public multi-user distribution |
| `redirect_uri_mismatch` | The client is not a Desktop-app client (e.g. a Web application client with no matching redirect URI) | Create a **Desktop app** client and use its JSON |
| `unauthorized_client` | The client is not permitted to use this authorization flow (typically a client-type mismatch) | Recreate the client as **Desktop app** |
| `invalid_grant` during refresh | The refresh token was revoked or expired (e.g. access revoked, password change, long inactivity) | Re-run `auth` to authorize again; the persisted token data is replaced |
| `403 quotaExceeded` | The project's daily quota is exhausted | Wait for the midnight PT reset, reduce usage, or apply for a quota extension |
| `No such file` / unreadable client file at `auth` startup | `oauth.client_config_path` points at a missing or unreadable file | Verify the path and place the downloaded JSON there |
| Configuration rejected before `auth` starts | Blank or identical `oauth.client_config_path` / `oauth.credentials_path` | Use two distinct, non-blank paths |

## Handling the Credentials Securely

- The client JSON contains a `client_secret`. Treat it as sensitive even though Google notes that installed applications cannot truly keep secrets: do not commit it, do not paste it into issues/logs, and keep it only under the gitignored `credentials/` path.
- The issued `tokens.json` contains an access token and (crucially) a refresh token that grants long-lived access. The same rules apply — never commit it, never log it.
- Confirm the ignore rules before adding files: `credentials/*` is ignored while `credentials/.gitkeep` keeps the directory tracked; `/config/config.json` is ignored. See the gitignore table in [`docs/authentication.md`](authentication.md).
- On shared machines, restrict the credential file to the operating user. The service applies `0600` permissions on Unix-like systems; Windows uses inherited ACLs (no ACL management and no encryption at rest — documented limitations).

## Verification Checklist

Before running `auth`, confirm:

- [ ] Project created and selected in the Google Cloud Console.
- [ ] *YouTube Data API v3* listed as **Enabled** in that project.
- [ ] OAuth consent screen configured (External, required scope added, channel-owning account listed as a test user).
- [ ] OAuth client of type **Desktop app** created and its JSON downloaded.
- [ ] JSON saved at `oauth.client_config_path` (default `credentials/client_secret.json`).
- [ ] `oauth.credentials_path` set to a different path (default `credentials/tokens.json`).
- [ ] Both paths under the gitignored `credentials/` directory.

## See Also

- [`docs/authentication.md`](authentication.md) — the `auth` flow, credential files, unattended refresh, exit codes, and verification evidence.
- [`docs/configuration.md`](configuration.md) — the `oauth` configuration fields and validation rules.
- [YouTube Data API v3 overview](https://developers.google.com/youtube/v3/getting-started)
- [Obtaining authorization credentials](https://developers.google.com/youtube/registering_an_application)
- [OAuth 2.0 for installed applications](https://developers.google.com/youtube/v3/guides/auth/installed-apps)
- [Quota costs for API requests](https://developers.google.com/youtube/v3/determine_quota_cost)
- [Quota and compliance audits](https://developers.google.com/youtube/v3/guides/quota_and_compliance_audits)
