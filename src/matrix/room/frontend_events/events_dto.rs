use std::sync::Arc;

use matrix_sdk::ruma::{events::room::message::MessageType, OwnedRoomId, UInt};
use matrix_sdk_ui::timeline::{
    MsgLikeKind, TimelineItem, TimelineItemContent, TimelineItemKind, VirtualTimelineItem,
};
use serde::Serialize;

use crate::matrix::{
    room::frontend_events::msg_like::FrontendReactionsByKeyBySender,
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
    StateChange, // TODO add methods
    Error,       // TODO add methods
    Call,        // TODO add methods
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
                            MessageType::Text(text_msg) => {
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
                                            kind: FrontendMsgLikeKind::Text(text_msg),
                                        },
                                    ),
                                }
                            }
                            MessageType::Image(img_msg) => {
                                return FrontendTimelineItem {
                                    event_id,
                                    is_own,
                                    is_local,
                                    timestamp,
                                    data: FrontendTimelineItemData::MsgLike(
                                        FrontendMsgLikeContent {
                                            kind: FrontendMsgLikeKind::Image(img_msg),
                                            edited: message.is_edited(),
                                            reactions: FrontendReactionsByKeyBySender(
                                                &msg_like.reactions,
                                            ),
                                            sender_id,
                                            sender,
                                            thread_root: None,
                                        },
                                    ),
                                }
                            }
                            _ => {} // TODO handle other types
                        },
                        _ => {}
                    }

                    // let prev_event = tl_idx.checked_sub(1).and_then(|i| tl_items.get(i));

                    // populate_message_view(
                    //     cx,
                    //     list,
                    //     item_id,
                    //     room_id,
                    //     event_tl_item,
                    //     MessageOrSticker::Message(message),
                    //     prev_event,
                    //     &mut tl_state.media_cache,
                    //     &tl_state.user_power,
                    //     item_drawn_status,
                    //     room_screen_widget_uid,
                    // )
                }
                // TimelineItemContent::Sticker(sticker) => {
                //     let prev_event = tl_idx.checked_sub(1).and_then(|i| tl_items.get(i));
                //     populate_message_view(
                //         cx,
                //         list,
                //         item_id,
                //         room_id,
                //         event_tl_item,
                //         MessageOrSticker::Sticker(sticker.content()),
                //         prev_event,
                //         &mut tl_state.media_cache,
                //         &tl_state.user_power,
                //         item_drawn_status,
                //         room_screen_widget_uid,
                //     )
                // }
                // TimelineItemContent::RedactedMessage => populate_small_state_event(
                //     cx,
                //     list,
                //     item_id,
                //     room_id,
                //     event_tl_item,
                //     &RedactedMessageEventMarker,
                //     item_drawn_status,
                // ),
                // TimelineItemContent::MembershipChange(membership_change) => populate_small_state_event(
                //     cx,
                //     list,
                //     item_id,
                //     room_id,
                //     event_tl_item,
                //     membership_change,
                //     item_drawn_status,
                // ),
                // TimelineItemContent::ProfileChange(profile_change) => populate_small_state_event(
                //     cx,
                //     list,
                //     item_id,
                //     room_id,
                //     event_tl_item,
                //     profile_change,
                //     item_drawn_status,
                // ),
                // TimelineItemContent::OtherState(other) => populate_small_state_event(
                //     cx,
                //     list,
                //     item_id,
                //     room_id,
                //     event_tl_item,
                //     other,
                //     item_drawn_status,
                // ),
                _ => {}
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
    return FrontendTimelineItem {
        event_id: None,
        data: FrontendTimelineItemData::Error,
        is_local: true,
        is_own: true,
        timestamp: None,
    };
}
