//! The concrete `ureq`-backed YouTube Live API transport (plan F-T2-1;
//! D-T2-2..D-T2-7): the five operations O1–O5 with the documented request
//! forms (D-T2-4), OAuth 2.0 Bearer only — NEVER an API key, no `key=`
//! parameter ever (§1) — a redirects-disabled agent, and status-first
//! classification through the ONE staged `classify_api_error` (D-T2-5);
//! bodies surface only the non-secret `reason` and are NEVER embedded.
//! `extract_reason` is the deliberate D-T2-6 local duplicate of the
//! verifier's private extraction. Limits (D-T2-16): no request-level
//! timeout (verifier parity); no transition/delete calls anywhere.
//! Loopback-tested in the sibling tests; no test reaches Google.
//!
//! Grounding: OAuth 2.0 Bearer (never an API key; service accounts are
//! unsupported) per
//! https://developers.google.com/youtube/v3/live/authentication; the five
//! request forms are the O1–O5 operations of the liveStreams /
//! liveBroadcasts references cited in `api_transport`.
//!
//! Plan: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (F-T2-1, D-T2-2..7, D-T2-16); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2.

use super::api_request::{
    bind_path, broadcast_insert_path, broadcast_lookup_path, request_body_value, response_body,
    response_reason_error, stream_insert_path, stream_lookup_path, LiveApiMethod,
    LiveApiRequestSpec,
};
use super::api_transport::LiveApiTransport;
use super::live_error::LiveApiError;
use super::live_models::{
    BindCommand, BroadcastRequestBody, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast,
    LiveBroadcastList, LiveStream, LiveStreamList, StreamRequestBody,
};
use super::token::AccessToken;

/// The documented Live API root; request paths append after the slash.
const LIVE_API_BASE_URL: &str = "https://www.googleapis.com/youtube/v3/";
pub(crate) struct GoogleLiveApiTransport {
    agent: ureq::Agent,
    base_url: String,
    access_token: AccessToken,
}

impl GoogleLiveApiTransport {
    /// Builds the transport against the production endpoint (D-T2-3);
    /// the Bearer value is formatted per request and never stored.
    pub(crate) fn new(access_token: AccessToken) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            base_url: LIVE_API_BASE_URL.to_string(),
            access_token,
        }
    }

    /// Test-only base-URL substitute for loopback servers; production
    /// code never calls it.
    #[cfg(test)]
    pub(crate) fn with_base_url(base_url: String, access_token: AccessToken) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
            base_url,
            access_token,
        }
    }

    /// The ONE request engine (spec-encapsulated, D-T2-4); routes every
    /// reply through the ONE status-first `respond` path (D-T2-5).
    fn request(&self, spec: &LiveApiRequestSpec<'_>) -> Result<String, LiveApiError> {
        let url = format!("{}{}", self.base_url, spec.path_query);
        let authorization = format!("Bearer {}", self.access_token.secret());
        let pending = match spec.method {
            LiveApiMethod::Get => self.agent.get(&url),
            LiveApiMethod::Post => self.agent.post(&url),
        }
        .set("Authorization", &authorization);
        let reply = match spec.body {
            Some(body) => pending
                .set("Content-Type", "application/json")
                .send_string(body.to_string().as_str()),
            None => pending.call(),
        };
        self.respond(reply)
    }

    /// Status-first shared response path (D-T2-5): 2xx → body string;
    /// every other reply (ureq 2 surfaces 3xx as `Ok` with redirects(0))
    /// → reason extraction + the ONE staged classifier, so a 3xx falls
    /// through `classify_api_error`'s final arm into `ApiRejected`;
    /// transport errors → `Network`. Bodies never embedded into errors.
    fn respond(&self, reply: Result<ureq::Response, ureq::Error>) -> Result<String, LiveApiError> {
        match reply {
            Ok(response) if is_success_status(response.status()) => response_body(response),
            Ok(response) => Err(response_reason_error(response.status(), response)),
            Err(ureq::Error::Status(status, response)) => {
                Err(response_reason_error(status, response))
            }
            Err(other) => Err(LiveApiError::Network {
                reason: other.to_string(),
            }),
        }
    }
}

fn is_success_status(status: u16) -> bool {
    (200..300).contains(&status)
}

impl LiveApiTransport for GoogleLiveApiTransport {
    fn create_stream(&self, command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError> {
        let body = request_body_value(StreamRequestBody::from_command(command))?;
        let reply = self.request(&LiveApiRequestSpec::posting_json(
            stream_insert_path(),
            &body,
        ))?;
        parse_resource_body(&reply, "liveStream resource")
    }

    fn create_broadcast(
        &self,
        command: &CreateBroadcastCommand,
    ) -> Result<LiveBroadcast, LiveApiError> {
        let body = request_body_value(BroadcastRequestBody::from_command(command))?;
        let reply = self.request(&LiveApiRequestSpec::posting_json(
            broadcast_insert_path(),
            &body,
        ))?;
        parse_resource_body(&reply, "liveBroadcast resource")
    }

    fn bind(&self, command: &BindCommand) -> Result<LiveBroadcast, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::posting(bind_path(command)))?;
        parse_resource_body(&reply, "liveBroadcast resource")
    }

    fn lookup_stream(&self, id: &str) -> Result<Option<LiveStream>, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::getting(stream_lookup_path(id)))?;
        let list: LiveStreamList = parse_resource_body(&reply, "liveStreamList")?;
        Ok(list.items.into_iter().next())
    }

    fn lookup_broadcast(&self, id: &str) -> Result<Option<LiveBroadcast>, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::getting(broadcast_lookup_path(id)))?;
        let list: LiveBroadcastList = parse_resource_body(&reply, "liveBroadcastList")?;
        Ok(list.items.into_iter().next())
    }
}

fn parse_resource_body<T: serde::de::DeserializeOwned>(
    body: &str,
    field: &str,
) -> Result<T, LiveApiError> {
    serde_json::from_str(body).map_err(|_| LiveApiError::MalformedResponse {
        field: field.to_string(),
    })
}

#[cfg(test)]
#[path = "google_live_api_transport_tests.rs"]
mod google_live_api_transport_tests;

#[cfg(test)]
#[path = "google_live_api_transport_outcomes_tests.rs"]
mod google_live_api_transport_outcomes_tests;
