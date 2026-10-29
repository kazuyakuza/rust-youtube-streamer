//! The concrete chat transport (record D-7, plan §3.4): implements
//! [`LiveChatTransport`] for [`GoogleLiveApiTransport`] by REUSING the
//! Phase-05 shared request engine — same Bearer-only redirects-disabled
//! `ureq` stack, same status-first classification, same never-embed-bodies
//! contract. The O7 `liveChatMessages.list` operation retired with the
//! Phase-07 streamList switch (Task-1 record §12); the O6 lookup stays
//! byte-unchanged.
//!
//! Grounding: S5 (`liveBroadcasts` for the O6 `snippet.liveChatId` read) —
//! citations + access dates in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Loopback-tested in the sibling wire roster; no test reaches Google.

use super::api_chat_transport::LiveChatTransport;
use super::api_request::LiveApiRequestSpec;
use super::google_live_api_transport::{parse_resource_body, GoogleLiveApiTransport};
use super::live_chat_models::broadcast_chat_lookup_path;
use super::live_error::LiveApiError;
use super::live_models::LiveBroadcastList;

impl LiveChatTransport for GoogleLiveApiTransport {
    fn lookup_broadcast_chat_id(&self, broadcast_id: &str) -> Result<Option<String>, LiveApiError> {
        let reply = self.request(&LiveApiRequestSpec::getting(broadcast_chat_lookup_path(
            broadcast_id,
        )))?;
        let list: LiveBroadcastList = parse_resource_body(&reply, "broadcastChatLookup")?;
        Ok(list
            .items
            .first()
            .and_then(|broadcast| broadcast.snippet.as_ref())
            .and_then(|snippet| snippet.live_chat_id.clone()))
    }
}

#[cfg(test)]
#[path = "live_chat_wire_tests.rs"]
mod live_chat_wire_tests;
