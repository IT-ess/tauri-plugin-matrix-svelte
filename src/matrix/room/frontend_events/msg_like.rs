use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;
use matrix_sdk::ruma::{
    events::{
        room::message::{
            AudioMessageEventContent, EmoteMessageEventContent, FileMessageEventContent,
            ImageMessageEventContent, KeyVerificationRequestEventContent,
            LocationMessageEventContent, NoticeMessageEventContent,
            ServerNoticeMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
        },
        sticker::{StickerEventContent, StickerMediaSource},
    },
    OwnedEventId, OwnedUserId,
};
use matrix_sdk_ui::timeline::{ReactionInfo, ReactionStatus, ReactionsByKeyBySender};
use serde::{ser::SerializeMap, Serialize, Serializer};

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind",
    content = "body"
)]
pub enum FrontendMsgLikeKind {
    /// An audio message.
    Audio(AudioMessageEventContent),

    /// An emote message.
    Emote(EmoteMessageEventContent),

    /// A file message.
    File(FileMessageEventContent),

    /// An image message.
    Image(ImageMessageEventContent),

    /// A location message.
    Location(LocationMessageEventContent),

    /// A notice message.
    Notice(NoticeMessageEventContent),

    /// A server notice message.
    ServerNotice(ServerNoticeMessageEventContent),

    /// A text message.
    Text(TextMessageEventContent),

    /// A video message.
    Video(VideoMessageEventContent),

    /// A request to initiate a key verification.
    VerificationRequest(KeyVerificationRequestEventContent),

    /// An `m.sticker` event.
    Sticker(FrontendStickerEventContent),

    /// An `m.poll.start` event.
    Poll, //(PollState), // Todo: implement poll state display

    /// A redacted message.
    Redacted,

    /// An `m.room.encrypted` event that could not be decrypted.
    UnableToDecrypt,

    /// An unknown type of message
    Unknown,
}

/// A special kind of [`super::TimelineItemContent`] that groups together
/// different room message types with their respective reactions and thread
/// information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMsgLikeContent<'a> {
    #[serde(flatten)]
    pub kind: FrontendMsgLikeKind,
    /// Map of user reactions to this message
    pub reactions: FrontendReactionsByKeyBySender<'a>,
    /// Event ID of the thread root, if this is a threaded message.
    pub thread_root: Option<OwnedEventId>,
    // The event this message is replying to, if any.
    // pub in_reply_to: Option<InReplyToDetails>,
    /// Wether the event has been edited at least once
    pub edited: bool,
    /// Sender display name (could be none if not resolved yet)
    pub sender: Option<String>,
    /// Sender id of the event
    pub sender_id: String,
}

#[derive(Clone, Debug)]
pub struct FrontendReactionsByKeyBySender<'a>(pub &'a ReactionsByKeyBySender);

impl<'a> Serialize for FrontendReactionsByKeyBySender<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let outer_map = self.0;

        let mut map = serializer.serialize_map(Some(outer_map.len()))?;
        for (key, inner_map) in outer_map.iter() {
            map.serialize_entry(key, &SerializableInnerMap(inner_map))?;
        }
        map.end()
    }
}

struct SerializableInnerMap<'a>(&'a IndexMap<OwnedUserId, ReactionInfo>);

impl<'a> Serialize for SerializableInnerMap<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (user_id, reaction_info) in self.0.iter() {
            map.serialize_entry(user_id, &SerializableReactionInfo(reaction_info))?;
        }
        map.end()
    }
}

struct SerializableReactionInfo<'a>(&'a ReactionInfo);

fn reaction_status_key(status: &ReactionStatus) -> &'static str {
    match status {
        ReactionStatus::LocalToLocal(_) => "LocalToLocal",
        ReactionStatus::LocalToRemote(_) => "LocalToRemote",
        ReactionStatus::RemoteToRemote(_) => "RemoteToRemote",
    }
}

impl<'a> Serialize for SerializableReactionInfo<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("ReactionInfo", 2)?;
        s.serialize_field("timestamp", &self.0.timestamp)?;
        s.serialize_field("status", &reaction_status_key(&self.0.status))?;
        s.end()
    }
}

// New type pattern to add the msgtype field to serialization
#[derive(Debug, Clone)]
pub struct FrontendStickerEventContent(StickerEventContent);

impl Deref for FrontendStickerEventContent {
    type Target = StickerEventContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendStickerEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<StickerEventContent> for FrontendStickerEventContent {
    fn from(content: StickerEventContent) -> Self {
        FrontendStickerEventContent(content)
    }
}

impl Serialize for FrontendStickerEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("StickerEventContent", 4)?;
        state.serialize_field("body", &self.0.body)?;
        state.serialize_field("info", &self.0.info)?;
        match &self.0.source {
            StickerMediaSource::Plain(u) => state.serialize_field("url", u)?,
            StickerMediaSource::Encrypted(e) => state.serialize_field("file", e)?,
            &_ => todo!(),
        };
        state.serialize_field("msgtype", "m.sticker")?;
        state.end()
    }
}
