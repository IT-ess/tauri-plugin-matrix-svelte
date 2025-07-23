use std::sync::Arc;

use matrix_sdk::ruma::{events::room::message::MessageType, OwnedRoomId, UInt};
use matrix_sdk_ui::timeline::{
    MsgLikeKind, TimelineItem, TimelineItemContent, TimelineItemKind, VirtualTimelineItem,
};
use serde::Serialize;

use crate::matrix::{
    room::frontend_events::{
        msg_like::{FrontendReactionsByKeyBySender, FrontendStickerEventContent},
        state_event::{
            FrontendAnyOtherFullStateEventContent, FrontendMemberProfileChange,
            FrontendRoomMembershipChange, FrontendStateEvent,
        },
    },
    utils::get_or_fetch_event_sender,
};

use super::{
    msg_like::{FrontendMsgLikeContent, FrontendMsgLikeKind},
    virtual_event::FrontendVirtualTimelineItem,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendTimelineItem<'a> {
    event_id: Option<String>,
    #[serde(flatten)]
    data: FrontendTimelineItemData<'a>,
    timestamp: Option<UInt>, // We keep the timestamp at root to sort events
    is_own: bool,
    is_local: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind",
    content = "data"
)]
pub enum FrontendTimelineItemData<'a> {
    MsgLike(FrontendMsgLikeContent<'a>),
    Virtual(FrontendVirtualTimelineItem),
    StateChange(FrontendStateEvent),
    Error(FrontendTimelineErrorItem),
    Call,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendTimelineErrorItem {
    error: String,
}

pub fn to_frontend_timeline_item<'a>(
    item: &'a Arc<TimelineItem>,
    room_id: Option<&OwnedRoomId>,
) -> FrontendTimelineItem<'a> {
    match item.kind() {
        TimelineItemKind::Event(event_tl_item) => {
            let is_own = event_tl_item.is_own();
            let is_local = event_tl_item.is_local_echo();
            let timestamp = Some(event_tl_item.timestamp().get());
            let sender = Some(get_or_fetch_event_sender(event_tl_item, room_id));
            let sender_id = event_tl_item.sender().to_string();
            let event_id = if let Some(id) = event_tl_item.event_id() {
                Some(id.to_string())
            } else {
                None
            };
            match event_tl_item.content() {
                TimelineItemContent::MsgLike(msg_like) => {
                    // TODO: create a MsgLike mapper to refacto
                    match msg_like.kind.clone() {
                        MsgLikeKind::Message(message) => match message.msgtype().clone() {
                            _ => {
                                return FrontendTimelineItem {
                                    event_id,
                                    is_local,
                                    is_own,
                                    timestamp,
                                    data: FrontendTimelineItemData::MsgLike(
                                        FrontendMsgLikeContent {
                                            edited: message.is_edited(),
                                            reactions: FrontendReactionsByKeyBySender(
                                                &msg_like.reactions,
                                            ),
                                            sender_id,
                                            sender,
                                            thread_root: None,
                                            kind: map_msg_event_content(message.msgtype().clone()),
                                        },
                                    ),
                                }
                            }
                        },
                        MsgLikeKind::Sticker(sticker) => {
                            return FrontendTimelineItem {
                                event_id,
                                is_local,
                                is_own,
                                timestamp,
                                data: FrontendTimelineItemData::MsgLike(FrontendMsgLikeContent {
                                    edited: false,
                                    reactions: FrontendReactionsByKeyBySender(&msg_like.reactions),
                                    sender_id,
                                    sender,
                                    thread_root: None,
                                    kind: FrontendMsgLikeKind::Sticker(
                                        FrontendStickerEventContent::from(
                                            sticker.content().clone(),
                                        ),
                                    ),
                                }),
                            }
                        }
                        MsgLikeKind::Redacted => {
                            return FrontendTimelineItem {
                                event_id,
                                is_local,
                                is_own,
                                timestamp,
                                data: FrontendTimelineItemData::MsgLike(FrontendMsgLikeContent {
                                    edited: true,
                                    reactions: FrontendReactionsByKeyBySender(&msg_like.reactions),
                                    sender_id,
                                    sender,
                                    thread_root: None,
                                    kind: FrontendMsgLikeKind::Redacted,
                                }),
                            }
                        }
                        MsgLikeKind::UnableToDecrypt(_) => {
                            return FrontendTimelineItem {
                                event_id,
                                is_local,
                                is_own,
                                timestamp,
                                data: FrontendTimelineItemData::MsgLike(FrontendMsgLikeContent {
                                    edited: false,
                                    reactions: FrontendReactionsByKeyBySender(&msg_like.reactions),
                                    sender_id,
                                    sender,
                                    thread_root: None,
                                    kind: FrontendMsgLikeKind::UnableToDecrypt,
                                }),
                            }
                        }

                        MsgLikeKind::Poll(_) => {
                            return FrontendTimelineItem {
                                event_id,
                                is_local,
                                is_own,
                                timestamp,
                                data: FrontendTimelineItemData::MsgLike(FrontendMsgLikeContent {
                                    edited: false,
                                    reactions: FrontendReactionsByKeyBySender(&msg_like.reactions),
                                    sender_id,
                                    sender,
                                    thread_root: None,
                                    kind: FrontendMsgLikeKind::Poll,
                                }),
                            }
                        }
                    }
                }
                TimelineItemContent::OtherState(state) => {
                    return FrontendTimelineItem {
                        event_id,
                        is_local,
                        is_own,
                        timestamp,
                        data: FrontendTimelineItemData::StateChange(
                            FrontendStateEvent::OtherState(
                                FrontendAnyOtherFullStateEventContent::from(
                                    state.content().clone(),
                                ),
                            ),
                        ),
                    }
                }
                TimelineItemContent::MembershipChange(change) => {
                    return FrontendTimelineItem {
                        event_id,
                        is_local,
                        is_own,
                        timestamp,
                        data: FrontendTimelineItemData::StateChange(
                            FrontendStateEvent::MembershipChange(
                                FrontendRoomMembershipChange::from(change.clone()),
                            ),
                        ),
                    }
                }

                TimelineItemContent::ProfileChange(change) => {
                    return FrontendTimelineItem {
                        event_id,
                        is_local,
                        is_own,
                        timestamp,
                        data: FrontendTimelineItemData::StateChange(
                            FrontendStateEvent::ProfileChange(FrontendMemberProfileChange::from(
                                change.clone(),
                            )),
                        ),
                    }
                }

                TimelineItemContent::CallNotify | TimelineItemContent::CallInvite => {
                    return FrontendTimelineItem {
                        event_id,
                        is_local,
                        is_own,
                        timestamp,
                        data: FrontendTimelineItemData::Call,
                    }
                }

                TimelineItemContent::FailedToParseMessageLike {
                    event_type: _,
                    error,
                } => {
                    return FrontendTimelineItem {
                        event_id: None,
                        data: FrontendTimelineItemData::Error(FrontendTimelineErrorItem {
                            error: error.to_string(),
                        }),
                        is_local: true,
                        is_own: true,
                        timestamp: None,
                    };
                }

                TimelineItemContent::FailedToParseState {
                    state_key: _,
                    event_type: _,
                    error,
                } => {
                    return FrontendTimelineItem {
                        event_id: None,
                        data: FrontendTimelineItemData::Error(FrontendTimelineErrorItem {
                            error: error.to_string(),
                        }),
                        is_local: true,
                        is_own: true,
                        timestamp: None,
                    };
                }
            }
        }
        TimelineItemKind::Virtual(event) => match event {
            VirtualTimelineItem::DateDivider(timestamp) => {
                return FrontendTimelineItem {
                    event_id: None,
                    data: FrontendTimelineItemData::Virtual(
                        FrontendVirtualTimelineItem::DateDivider,
                    ),
                    is_local: true,
                    is_own: true,
                    timestamp: Some(timestamp.0),
                }
            }
            VirtualTimelineItem::ReadMarker => {
                return FrontendTimelineItem {
                    event_id: None,
                    data: FrontendTimelineItemData::Virtual(
                        FrontendVirtualTimelineItem::ReadMarker,
                    ),
                    is_local: true,
                    is_own: true,
                    timestamp: None,
                }
            }
            VirtualTimelineItem::TimelineStart => {
                return FrontendTimelineItem {
                    event_id: None,
                    data: FrontendTimelineItemData::Virtual(
                        FrontendVirtualTimelineItem::TimelineStart,
                    ),
                    is_local: true,
                    is_own: true,
                    timestamp: None,
                }
            }
        },
    };
}

fn map_msg_event_content(content: MessageType) -> FrontendMsgLikeKind {
    match content {
        MessageType::Audio(c) => FrontendMsgLikeKind::Audio(c),
        MessageType::File(c) => FrontendMsgLikeKind::File(c),
        MessageType::Image(c) => FrontendMsgLikeKind::Image(c),
        MessageType::Text(c) => FrontendMsgLikeKind::Text(c),
        MessageType::Video(c) => FrontendMsgLikeKind::Video(c),
        MessageType::Emote(c) => FrontendMsgLikeKind::Emote(c),
        // Not supported by frontend
        MessageType::Location(c) => FrontendMsgLikeKind::Location(c),
        MessageType::Notice(c) => FrontendMsgLikeKind::Notice(c),
        MessageType::ServerNotice(c) => FrontendMsgLikeKind::ServerNotice(c),
        MessageType::VerificationRequest(c) => FrontendMsgLikeKind::VerificationRequest(c),
        _ => FrontendMsgLikeKind::Unknown,
    }
}
