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

// fn to_image_item<'a>(
//     image_info_source: Option<(Option<ImageInfo>, MediaSource)>,
//     body: &str,
//     media_cache: &mut MediaCache,
// ) -> FrontendTimelineItem<'a> {
//     // We don't use thumbnails, as their resolution is too low to be visually useful.
//     // We also don't trust the provided mimetype, as it can be incorrect.
//     let (mimetype, _width, _height) = image_info_source
//         .as_ref()
//         .and_then(|(info, _)| {
//             info.as_ref()
//                 .map(|info| (info.mimetype.as_deref(), info.width, info.height))
//         })
//         .unwrap_or_default();

//     // If we have a known mimetype and it's not a static image,
//     // then show a message about it being unsupported (e.g., for animated gifs).
//     if let Some(mime) = mimetype.as_ref() {
//         if ImageFormat::from_mimetype(mime).is_none() {
//             text_or_image_ref.show_text(format!(
//                 "{body}\n\nImages/Stickers of type {mime:?} are not yet supported."
//             ));
//             return true; // consider this as fully drawn
//         }
//     }

//     let mut fully_drawn = false;

//     // A closure that fetches and shows the image from the given `mxc_uri`,
//     // marking it as fully drawn if the image was available.
//     let mut fetch_and_show_image_uri = |mxc_uri: OwnedMxcUri, image_info: Option<&ImageInfo>| {
//         match media_cache.try_get_media_or_fetch(mxc_uri.clone(), MEDIA_THUMBNAIL_FORMAT.into()) {
//             (MediaCacheEntry::Loaded(data), _media_format) => {
//                 let show_image_result = text_or_image_ref.show_image(cx, |cx, img| {
//                     utils::load_png_or_jpg(&img, cx, &data)
//                         .map(|()| img.size_in_pixels(cx).unwrap_or_default())
//                 });
//                 if let Err(e) = show_image_result {
//                     let err_str = format!("{body}\n\nFailed to display image: {e:?}");
//                     error!("{err_str}");
//                     text_or_image_ref.show_text(cx, &err_str);
//                 }

//                 // We're done drawing the image, so mark it as fully drawn.
//                 fully_drawn = true;
//             }
//             (MediaCacheEntry::Requested, _media_format) => {
//                 if let Some(image_info) = image_info {
//                     if let (Some(ref blurhash), Some(width), Some(height)) = (
//                         image_info.thumbhash.clone(),
//                         image_info.width,
//                         image_info.height,
//                     ) {
//                         let show_image_result = text_or_image_ref.show_image(cx, |cx, img| {
//                             let (Ok(width), Ok(height)) = (width.try_into(), height.try_into())
//                             else {
//                                 return Err(image_cache::ImageError::EmptyData);
//                             };
//                             if let Ok(data) = thumbhash::decode(blurhash, width, height, 1.0) {
//                                 ImageBuffer::new(&data, width as usize, height as usize).map(
//                                     |img_buff| {
//                                         let texture = Some(img_buff.into_new_texture(cx));
//                                         img.set_texture(cx, texture);
//                                         img.size_in_pixels(cx).unwrap_or_default()
//                                     },
//                                 )
//                             } else {
//                                 Err(image_cache::ImageError::EmptyData)
//                             }
//                         });
//                         if let Err(e) = show_image_result {
//                             let err_str = format!("{body}\n\nFailed to display image: {e:?}");
//                             error!("{err_str}");
//                             text_or_image_ref.show_text(cx, &err_str);
//                         }
//                     }
//                 }
//                 fully_drawn = false;
//             }
//             (MediaCacheEntry::Failed, _media_format) => {
//                 text_or_image_ref.show_text(
//                     cx,
//                     format!("{body}\n\nFailed to fetch image from {:?}", mxc_uri),
//                 );
//                 // For now, we consider this as being "complete". In the future, we could support
//                 // retrying to fetch thumbnail of the image on a user click/tap.
//                 fully_drawn = true;
//             }
//         }
//     };

//     let mut fetch_and_show_media_source =
//         |media_source: MediaSource, image_info: Option<&ImageInfo>| {
//             match media_source {
//                 MediaSource::Encrypted(encrypted) => {
//                     // We consider this as "fully drawn" since we don't yet support encryption.
//                     text_or_image_ref.show_text(
//                         cx,
//                         format!(
//                             "{body}\n\n[TODO] fetch encrypted image at {:?}",
//                             encrypted.url
//                         ),
//                     );
//                 }
//                 MediaSource::Plain(mxc_uri) => fetch_and_show_image_uri(mxc_uri, image_info),
//             }
//         };

//     match image_info_source {
//         Some((image_info, original_source)) => {
//             // Use the provided thumbnail URI if it exists; otherwise use the original URI.
//             let media_source = image_info
//                 .clone()
//                 .and_then(|image_info| image_info.thumbnail_source)
//                 .unwrap_or(original_source);
//             fetch_and_show_media_source(media_source, image_info.as_ref());
//         }
//         None => {
//             text_or_image_ref.show_text("{body}\n\nImage message had no source URL.");
//             fully_drawn = true;
//         }
//     }

//     fully_drawn
// }
