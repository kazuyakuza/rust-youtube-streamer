//! The published `stream_list.proto` wire types (Task-1 record §A.9/S2,
//! access-dated): hand-authored prost `Message` structs — build-time codegen
//! is rejected (record P7-4: protoc is unavailable on the pinned image;
//! generated files exceed the 200-line cap). Declared-but-unread fields are
//! transcribed exactly as published; unread non-text oneof arms (22–34) and
//! unknown fields/enum values are tolerated-and-skipped by prost — never an
//! error, never ordinary chat. Golden bytes: the sibling roster pins every
//! field number by encoding populated structs against the pre-computed byte
//! sequences in the plan appendix.
//!
//! Streaming tests stay offline: no gRPC dialing anywhere in tests.

use super::stream_transport::ChatStreamRequest;

pub(crate) const CHAT_STREAM_PARTS: [&str; 3] = ["id", "snippet", "authorDetails"];

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageListRequest {
    #[prost(string, optional, tag = "1")]
    pub(crate) live_chat_id: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub(crate) hl: Option<String>, // never sent (record P7 wire-form)
    #[prost(uint32, optional, tag = "3")]
    pub(crate) profile_image_size: Option<u32>, // never sent
    #[prost(uint32, optional, tag = "98")]
    pub(crate) max_results: Option<u32>, // "Not used in the streaming RPC."
    #[prost(string, optional, tag = "99")]
    pub(crate) page_token: Option<String>,
    #[prost(string, repeated, tag = "100")]
    pub(crate) part: Vec<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageListResponse {
    #[prost(string, optional, tag = "200")]
    pub(crate) kind: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "201")]
    pub(crate) etag: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "2")]
    pub(crate) offline_at: Option<String>,
    #[prost(message, optional, tag = "1004")]
    pub(crate) page_info: Option<PageInfo>, // declared-but-unread
    #[prost(string, optional, tag = "100602")]
    pub(crate) next_page_token: Option<String>,
    #[prost(message, repeated, tag = "1007")]
    pub(crate) items: Vec<LiveChatMessage>,
    #[prost(message, optional, tag = "1008")]
    pub(crate) active_poll_item: Option<LiveChatMessage>, // declared-but-unread
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessage {
    #[prost(string, optional, tag = "200")]
    pub(crate) kind: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "201")]
    pub(crate) etag: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "101")]
    pub(crate) id: Option<String>,
    #[prost(message, optional, tag = "2")]
    pub(crate) snippet: Option<LiveChatMessageSnippet>,
    #[prost(message, optional, tag = "3")]
    pub(crate) author_details: Option<LiveChatMessageAuthorDetails>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageSnippet {
    #[prost(enumeration = "SnippetType", optional, tag = "1")]
    pub(crate) message_type: Option<i32>,
    #[prost(string, optional, tag = "201")]
    pub(crate) live_chat_id: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "301")]
    pub(crate) author_channel_id: Option<String>,
    #[prost(string, optional, tag = "4")]
    pub(crate) published_at: Option<String>,
    #[prost(bool, optional, tag = "17")]
    pub(crate) has_display_content: Option<bool>, // declared-but-unread
    #[prost(string, optional, tag = "16")]
    pub(crate) display_message: Option<String>, // declared-but-unread; NEVER promoted
    #[prost(oneof = "DisplayedContent", tags = "19")]
    pub(crate) displayed_content: Option<DisplayedContent>,
}

#[derive(Clone, PartialEq, prost::Oneof)]
pub(crate) enum DisplayedContent {
    #[prost(message, tag = "19")]
    TextMessage(LiveChatTextMessageDetails),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub(crate) enum SnippetType {
    InvalidType = 0,
    TextMessageEvent = 1,
    Tombstone = 2,
    FanFundingEvent = 3,
    ChatEndedEvent = 4,
    SponsorOnlyModeStartedEvent = 5,
    SponsorOnlyModeEndedEvent = 6,
    NewSponsorEvent = 7,
    MemberMilestoneChatEvent = 17,
    MembershipGiftingEvent = 18,
    GiftMembershipReceivedEvent = 19,
    UserBannedEvent = 10,
    SuperChatEvent = 15,
    SuperStickerEvent = 16,
    PollEvent = 20,
    GiftEvent = 21,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatMessageAuthorDetails {
    #[prost(string, optional, tag = "10101")]
    pub(crate) channel_id: Option<String>,
    #[prost(string, optional, tag = "102")]
    pub(crate) channel_url: Option<String>, // declared-but-unread
    #[prost(string, optional, tag = "103")]
    pub(crate) display_name: Option<String>,
    #[prost(string, optional, tag = "104")]
    pub(crate) profile_image_url: Option<String>, // declared-but-unread
    #[prost(bool, optional, tag = "4")]
    pub(crate) is_verified: Option<bool>, // declared-but-unread
    #[prost(bool, optional, tag = "5")]
    pub(crate) is_chat_owner: Option<bool>, // declared-but-unread
    #[prost(bool, optional, tag = "6")]
    pub(crate) is_chat_sponsor: Option<bool>, // declared-but-unread
    #[prost(bool, optional, tag = "7")]
    pub(crate) is_chat_moderator: Option<bool>, // declared-but-unread
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct LiveChatTextMessageDetails {
    #[prost(string, optional, tag = "1")]
    pub(crate) message_text: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct PageInfo {
    #[prost(int32, optional, tag = "1")]
    pub(crate) total_results: Option<i32>, // declared-but-unread
    #[prost(int32, optional, tag = "2")]
    pub(crate) results_per_page: Option<i32>, // declared-but-unread
}

/// The wire request assembly (P7-5 step 1): the chat id ALWAYS; the resume
/// token ONLY when one exists (never fabricated); the documented part set;
/// never `hl`/`profileImageSize`/`maxResults` (record §1.1/P7 wire-form).
pub(crate) fn stream_list_request(command: &ChatStreamRequest) -> LiveChatMessageListRequest {
    LiveChatMessageListRequest {
        live_chat_id: Some(command.live_chat_id.clone()),
        hl: None,
        profile_image_size: None,
        max_results: None,
        page_token: command.page_token.clone(),
        part: CHAT_STREAM_PARTS
            .iter()
            .map(|part| part.to_string())
            .collect(),
    }
}

#[cfg(test)]
#[path = "stream_proto_tests.rs"]
mod stream_proto_tests;
