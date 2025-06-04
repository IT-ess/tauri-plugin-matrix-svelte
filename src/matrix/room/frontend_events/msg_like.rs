use matrix_sdk::ruma::{
    events::{
        room::message::{
            AudioMessageEventContent, EmoteMessageEventContent, FileMessageEventContent,
            ImageMessageEventContent, KeyVerificationRequestEventContent,
            LocationMessageEventContent, NoticeMessageEventContent,
            ServerNoticeMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
        },
        sticker::StickerEventContent,
    },
    OwnedEventId,
};
use serde::Serialize;

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
    Sticker(StickerEventContent),

    /// An `m.poll.start` event.
    // Poll(PollState),

    /// A redacted message.
    Redacted,
    // An `m.room.encrypted` event that could not be decrypted.
    // UnableToDecrypt(EncryptedMessage),
}

/// A special kind of [`super::TimelineItemContent`] that groups together
/// different room message types with their respective reactions and thread
/// information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMsgLikeContent {
    #[serde(flatten)]
    pub kind: FrontendMsgLikeKind,
    // pub reactions: ReactionsByKeyBySender, // TODO handle serialization for this struct
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
