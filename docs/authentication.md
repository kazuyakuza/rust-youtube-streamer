# Authentication (OAuth 2.0)

This guide covers the `auth` command: the Google OAuth 2.0 installed-application authorization flow, the credential files it produces, unattended refresh, failure recovery, and the verification evidence recorded for this phase.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [First Authorization (Interactive, Once)](#first-authorization-interactive-once)
- [Credential Files](#credential-files)
- [Unattended Operation](#unattended-operation)
- [Revoked / Unrefreshable / Insufficient States](#revoked--unrefreshable--insufficient-states)
- [Security Notes](#security-notes)
- [Verification Evidence and Limitations](#verification-evidence-and-limitations)
- [See Also](#see-also)

## Overview

The `auth` command performs the OAuth 2.0 authorization-code flow for a Google-installed application: it prints the authorization URL to the console, waits for the browser callback on a loopback-only listener, exchanges the code (with PKCE S256), validates the exchanged token response offline for the minimum required scope and a refresh token, and persists the credentials for later unattended use (the live network probe belongs to the reusable credential service, below). The scope authority is the project brief; the requested scope is `https://www.googleapis.com/auth/youtube.force-ssl` — the minimal YouTube API permission covering the needs of the subsequent broadcast/chat phases, which reuse the same grant. The research ledger for the scope choice and Google's installed-application requirements lives in [`.kilo/plans/20261012-phase04-task1-oauth-design.md`](../.kilo/plans/20261012-phase04-task1-oauth-design.md) §1.

The reusable credential layer (load → verify → refresh → persist) lives under `src/youtube/`; the interactive flow lives under `src/app/auth/`. `demo` (local-only recording) never contacts OAuth endpoints; `run` consumes the credential service non-interactively (load → verify-first → refresh-once → persist → re-verify) — see [`docs/production-run.md`](production-run.md).

## Prerequisites

1. A Google Cloud project with the YouTube Data API enabled — for why this is required, why API keys/service accounts do not substitute, and the step-by-step setup, see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md).
2. An OAuth 2.0 client of type **Desktop app** (Google's supported flow for installed applications).
3. Download the client JSON and save it at the configured `oauth.client_config_path` (default `credentials/client_secret.json`).
4. Configure `oauth.credentials_path` (default `credentials/tokens.json`) as a **different** path — the issued credentials must never overwrite the client configuration. Configuration validation rejects identical paths, and both paths must be non-blank.

The `credentials/` directory is operator-owned: its only tracked content is `.gitkeep`. Everything you place there (client secrets, tokens) is covered by the `credentials/*` gitignore rule and must never be committed.

## First Authorization (Interactive, Once)

Initial authorization requires user interaction; it cannot be automated:

1. Copy `config/config.example.json` to `config/config.json` if you have not already, and verify the `oauth` paths (see [`docs/configuration.md`](configuration.md#oauth)).
2. Run `rust-youtube-streamer-service auth` (optionally with `--config <path>`).
3. The console prints the Google authorization URL and the loopback redirect URI it is waiting on. Open the URL in a browser, sign in with the Google account that owns the YouTube channel, and approve the requested YouTube permission. The app never launches a browser itself and never reads interactive input.
4. **Unverified-app screen**: on the first authorization with a development client, Google may show `Google hasn't verified this app`. This is expected — the client is not (yet) verified by Google. Choose **Advanced** and continue to the authorization prompt.
5. The callback arrives at a listener bound to `127.0.0.1` only, on an ephemeral port (never a fixed port, never a non-loopback interface). The app validates the OAuth `state` parameter against the generated value (CSRF protection), rejects mismatched state terminally, and closes the listener deterministically on success AND on every failure — no background listener or task survives the mode.
6. The app exchanges the authorization code for tokens (PKCE S256), verifies the granted scope offline, persists the credentials atomically, prints one success line, and exits.

Console output at each stage:

- Success: `authorization succeeded: YouTube credentials saved at <credentials_path>; the required YouTube scope was granted` on stdout, exit 0.
- Any failure: exactly one `error: <redacted reason>` line on stderr, exit 1.

Exit codes for `auth`: `0` = verified success; `1` = configuration error, unreadable/invalid client file, listener-bind failure, CSRF state mismatch, provider decline, noise-budget exhaustion, token-exchange failure, persistence failure, insufficient scope, or missing refresh token; `3` is never returned by `auth` (exit code 3 is no longer used by any command).

## Credential Files

Two distinct files, both operator-configurable:

| File | Config field | Default path | Purpose |
| --- | --- | --- | --- |
| OAuth client configuration | `oauth.client_config_path` | `credentials/client_secret.json` | Google-issued desktop-client identifiers (`client_id`, `client_secret`, `auth_uri`, `token_uri`). Declared endpoints are validated against the accepted Google forms (legacy and v2); requests always use the app's own constants. Never modified by the app. |
| Issued credentials | `oauth.credentials_path` | `credentials/tokens.json` | The tokens this app persists after a successful authorization. |

The persisted credential file uses a frozen four-key JSON schema:

- `access_token` (string) — the current access token.
- `expires_in_seconds` (integer) — advisory TTL metadata for the access token (see Unattended Operation).
- `refresh_token` (string or null) — the refresh credential; `null` when the provider issued none.
- `granted_scopes` (string) — the space-delimited scope list granted by the user.

Persistence is atomic where practical: the file is written to a temporary file in the same directory and renamed over the target, so an interrupted save never leaves a truncated valid credential file; a stale temporary file from a previous interrupted save is cleaned up.

Platform limitations (honest):

- POSIX platforms (Linux): the created file mode is `0600` (owner-only), verified by the test suite.
- Windows: the file inherits ACLs from the parent directory; the Rust standard library has no ACL management, so no Windows-specific permission hardening is applied. The Linux container test suite cannot exercise the Windows rename/permission arms.
- There is **no encryption** of the stored credential file at rest, on any platform. Protect the file with filesystem permissions and disk-level encryption if your threat model requires it.

## Unattended Operation

Future sessions (later production phases) load the stored credentials through the reusable credential service and follow a verify-first design: the effective authorization is verified through the minimal YouTube API probe (`GET /youtube/v3/channels?part=id&mine=true` with the Bearer access token) before any refresh decision — there is no clock-based refresh. If verification rejects the access token and a refresh token is stored, the service refreshes exactly once at Google's token endpoint, persists the refreshed credentials atomically, and re-verifies the fresh token (a provider-issued replacement refresh token is kept; a response without a refresh-token field keeps the stored one).

Unattended longevity is not guaranteed by this application: whether a refresh token remains usable over long periods depends on Google's current OAuth policies (revocation, inactivity, scope changes, security events). A future unattended run that can no longer refresh fails with a distinct redacted error — see the next section — and requires a human to re-run `auth`.

## Revoked / Unrefreshable / Insufficient States

Distinct failure classes, each one redacted stderr line and exit 1:

| State | What happened | Recovery |
| --- | --- | --- |
| Missing credentials | No credential file at `oauth.credentials_path` (first run, or file deleted). | Run `auth`. |
| Unreadable / malformed | The file exists but cannot be read or parsed, or its schema does not match the frozen four-key shape. | Run `auth` (overwrites with a fresh, valid file). |
| Revoked | Google rejected the stored refresh token (`invalid_grant`). | Re-run `auth` — obtain a fresh grant in the browser. |
| Expired, unrefreshable | The access token is expired and no refresh token is stored. | Re-run `auth`. |
| Insufficient scope | The stored/granted scopes do not include `https://www.googleapis.com/auth/youtube.force-ssl`. | Re-run `auth` and approve the requested YouTube permission. |

The single recovery path for revoked or unrefreshable refresh credentials is re-running `auth`; there is no silent re-authorization and no manual token entry. Failures during an unattended production `run` are classified in [`docs/production-run.md`](production-run.md); the recovery path for revoked/unrefreshable credentials remains re-running `auth`.

## Security Notes

- Redaction is enforced at the type level: token and secret wrapper types implement a redacting `Debug` (the secret value never appears) and no `Display`; token-endpoint response bodies deserialize into typed structs, so a stray attempt to format a raw response body is a compile-time error. Errors, logs, and console output never carry access tokens, refresh tokens, authorization codes, client secrets, or PKCE verifiers.
- Gitignore rules protecting the credential material (line numbers from the tracked `.gitignore`):

  | Rule | Line | Effect |
  | --- | --- | --- |
  | `credentials/*` | 52 | ignores all files under `credentials/` |
  | `!credentials/.gitkeep` | 53 | keeps the directory placeholder tracked |
  | `/config/config.json` | 48 | ignores the real configuration file |
  | `/config/config.windows.json` | 49 | ignores the Windows-specific real configuration file |

- What an operator must never commit: `client_secret.json`, `tokens.json`, real `config/config.json`, access/refresh tokens, authorization codes, or any capture of the authorization URL while it carries live parameters. Commit reviews are not a safety net — the files are ignored by default.

## Verification Evidence and Limitations

Verification is offline by design: no automated test uses a real Google account, a live OAuth client secret, or any non-loopback network endpoint. The suite exercises the flow against injected fakes (token-endpoint transport, credential store, verifier) and real loopback HTTP servers (`127.0.0.1` only) standing in for the browser callback and the YouTube API probe. The full suite passed the standard container gate with 249 tests (`ALL CHECKS PASSED`).

Standing limitations (mandatory reading):

- **NO live Google authorization was performed in this phase.** The flow is validated exclusively offline/loopback; no real Google account was used end-to-end.
- **Native Windows/Linux runtime execution was NOT RUN.** All verification is container-only (Linux). The Windows `.exe` is a cross-compilation artifact; cross-compilation is not native runtime validation.
- The unverified-app screen expectation is documented but was not exercised against Google's live pages.
- The Windows rename/permission arms of the credential store cannot be exercised in the Linux container (see Credential Files).

## See Also

- [README](../README.md) — project overview and the Authentication section.
- [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) — Google Cloud requirements (project, API enablement, Desktop-app client), why other credential types do not work, step-by-step setup, and costs/quotas.
- [`docs/configuration.md`](configuration.md) — the `oauth` configuration fields and validation rules.
- [`docs/demo.md`](demo.md) — the local-only demo recording (no OAuth).
