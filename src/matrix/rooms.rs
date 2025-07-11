use std::sync::Arc;

use crate::matrix::{
    events::get_latest_event_details,
    room::rooms_list::{enqueue_rooms_list_update, JoinedRoomInfo, RoomsListUpdate},
    singletons::{ALL_JOINED_ROOMS, TOMBSTONED_ROOMS},
    timeline::timeline_subscriber_handler,
};
use anyhow::bail;
use matrix_sdk::{event_handler::EventHandlerDropGuard, ruma::OwnedRoomId, RoomState};
use matrix_sdk_ui::{timeline::RoomExt, RoomListService, Timeline};
use tokio::{runtime::Handle, sync::watch, task::JoinHandle};

use super::{
    singletons::LOG_ROOM_LIST_DIFFS,
    timeline::{TimelineRequestSender, TimelineUpdate},
};

/// Backend-specific details about a joined room that our client currently knows about.
pub struct JoinedRoomDetails {
    #[allow(unused)]
    room_id: OwnedRoomId,
    /// A reference to this room's timeline of events.
    pub timeline: Arc<Timeline>,
    /// An instance of the clone-able sender that can be used to send updates to this room's timeline.
    pub timeline_update_sender: crossbeam_channel::Sender<TimelineUpdate>,
    /// A tuple of two separate channel endpoints that can only be taken *once* by the main UI thread.
    ///
    /// 1. The single receiver that can receive updates to this room's timeline.
    ///    * When a new room is joined, an unbounded crossbeam channel will be created
    ///      and its sender given to a background task (the `timeline_subscriber_handler()`)
    ///      that enqueues timeline updates as it receives timeline vector diffs from the server.
    ///    * The UI thread can take ownership of this update receiver in order to receive updates
    ///      to this room's timeline, but only one receiver can exist at a time.
    /// 2. The sender that can send requests to the background timeline subscriber handler,
    ///    e.g., to watch for a specific event to be prepended to the timeline (via back pagination).
    pub timeline_singleton_endpoints: Option<(
        crossbeam_channel::Receiver<TimelineUpdate>,
        TimelineRequestSender,
    )>,
    /// The async task that listens for timeline updates for this room and sends them to the UI thread.
    timeline_subscriber_handler_task: JoinHandle<()>,
    /// A drop guard for the event handler that represents a subscription to typing notices for this room.
    pub typing_notice_subscriber: Option<EventHandlerDropGuard>,
    /// The ID of the old tombstoned room that this room has replaced, if any.
    replaces_tombstoned_room: Option<OwnedRoomId>,
}
impl Drop for JoinedRoomDetails {
    // TODO: implement dropping the Svelte store as well
    fn drop(&mut self) {
        println!("Dropping RoomInfo for room {}", self.room_id);
        self.timeline_subscriber_handler_task.abort();
        drop(self.typing_notice_subscriber.take());
        if let Some(replaces_tombstoned_room) = self.replaces_tombstoned_room.take() {
            TOMBSTONED_ROOMS
                .lock()
                .unwrap()
                .insert(self.room_id.clone(), replaces_tombstoned_room);
        }
    }
}

/// Info we store about a room received by the room list service.
///
/// This struct is necessary in order for us to track the previous state
/// of a room received from the room list service, so that we can
/// determine if the room has changed state.
/// We can't just store the `matrix_sdk::Room` object itself,
/// because that is a shallow reference to an inner room object within
/// the room list service
#[derive(Clone)]
pub struct RoomListServiceRoomInfo {
    room: matrix_sdk::Room,
    pub room_id: OwnedRoomId,
    room_state: RoomState,
}
impl From<&matrix_sdk::Room> for RoomListServiceRoomInfo {
    fn from(room: &matrix_sdk::Room) -> Self {
        room.clone().into()
    }
}
impl From<matrix_sdk::Room> for RoomListServiceRoomInfo {
    fn from(room: matrix_sdk::Room) -> Self {
        Self {
            room_id: room.room_id().to_owned(),
            room_state: room.state(),
            room,
        }
    }
}

/// The number of unread messages in a room.
#[derive(Clone, Debug)]
pub enum UnreadMessageCount {
    /// There are unread messages, but we do not know how many.
    Unknown,
    /// There are unread messages, and we know exactly how many.
    Known(u64),
}

pub async fn add_new_room(
    room: &matrix_sdk::Room,
    room_list_service: &RoomListService,
) -> anyhow::Result<()> {
    let room_id = room.room_id().to_owned();
    // We must call `display_name()` here to calculate and cache the room's name.
    let room_name = room.display_name().await.map(|n| n.to_string()).ok();

    match room.state() {
        RoomState::Knocked => {
            // TODO: handle Knocked rooms (e.g., can you re-knock? or cancel a prior knock?)
            return Ok(());
        }
        RoomState::Banned => {
            println!("Got new Banned room: {room_name:?} ({room_id})");
            // TODO: handle rooms that this user has been banned from.
            return Ok(());
        }
        RoomState::Left => {
            println!("Got new Left room: {room_name:?} ({room_id})");
            // TODO: add this to the list of left rooms,
            //       which is collapsed by default.
            //       Upon clicking a left room, we can show a splash page
            //       that prompts the user to rejoin the room or forget it.

            // TODO: this may also be called when a user rejects an invite, not sure.
            //       So we might also need to make a new RoomsListUpdate::RoomLeft variant.
            return Ok(());
        }
        // RoomState::Invited => {
        //     let invite_details = room.invite_details().await.ok();
        //     let latest = room
        //         .latest_event()
        //         .await
        //         .as_ref()
        //         .map(|ev| get_latest_event_details(ev, &room_id));
        //     let room_avatar = room_avatar(room, room_name.as_deref()).await;

        //     let inviter_info = if let Some(inviter) = invite_details.and_then(|d| d.inviter) {
        //         Some(InviterInfo {
        //             user_id: inviter.user_id().to_owned(),
        //             display_name: inviter.display_name().map(|n| n.to_string()),
        //             avatar: inviter
        //                 .avatar(AVATAR_THUMBNAIL_FORMAT.into())
        //                 .await
        //                 .ok()
        //                 .flatten()
        //                 .map(Into::into),
        //         })
        //     } else {
        //         None
        //     };

        //     rooms_list::enqueue_rooms_list_update(RoomsListUpdate::AddInvitedRoom(
        //         InvitedRoomInfo {
        //             room_id,
        //             room_name,
        //             inviter_info,
        //             room_avatar,
        //             canonical_alias: room.canonical_alias(),
        //             alt_aliases: room.alt_aliases(),
        //             latest,
        //             invite_state: Default::default(),
        //             is_selected: false,
        //         },
        //     ));
        //     return Ok(());
        // }
        RoomState::Joined => {} // Fall through to adding the joined room below.
        _ => bail!("We do not handle invited rooms yet"),
    }

    // Subscribe to all updates for this room in order to properly receive all of its states.
    room_list_service.subscribe_to_rooms(&[&room_id]).await;

    // Do not add tombstoned rooms to the rooms list; they require special handling.
    if let Some(tombstoned_info) = room.tombstone_content() {
        println!("Room {room_id} has been tombstoned: {tombstoned_info:#?}");
        // Since we don't know the order in which we'll learn about new rooms,
        // we need to first check to see if the replacement for this tombstoned room
        // refers to an already-known room as its replacement.
        // If so, we can immediately update the replacement room's room info
        // to indicate that it replaces this tombstoned room.
        let replacement_room_id = tombstoned_info.replacement_room;
        if let Some(room_info) = ALL_JOINED_ROOMS
            .lock()
            .unwrap()
            .get_mut(&replacement_room_id)
        {
            room_info.replaces_tombstoned_room = Some(replacement_room_id.clone());
        }
        // But if we don't know about the replacement room yet, we need to save this tombstoned room
        // in a separate list so that the replacement room we will discover in the future
        // can know which old tombstoned room it replaces (see the bottom of this function).
        else {
            TOMBSTONED_ROOMS
                .lock()
                .unwrap()
                .insert(replacement_room_id, room_id.clone());
        }
        return Ok(());
    }

    let timeline = if let Ok(tl) = room.timeline().await {
        Arc::new(tl)
    } else {
        Arc::new(
            room.timeline_builder()
                .track_read_marker_and_receipts()
                .build()
                .await?,
        )
    };
    let latest_event = timeline.latest_event().await;
    let (timeline_update_sender, timeline_update_receiver) = crossbeam_channel::unbounded();

    let (request_sender, request_receiver) = watch::channel(Vec::new());
    let timeline_subscriber_handler_task = Handle::current().spawn(timeline_subscriber_handler(
        room.clone(),
        timeline.clone(),
        timeline_update_sender.clone(),
        request_receiver,
    ));

    let latest = latest_event
        .as_ref()
        .map(|ev| get_latest_event_details(ev, &room_id));

    let tombstoned_room_replaced_by_this_room = TOMBSTONED_ROOMS.lock().unwrap().remove(&room_id);

    println!("Adding new joined room {room_id}. Replaces tombstoned room: {tombstoned_room_replaced_by_this_room:?}");
    ALL_JOINED_ROOMS.lock().unwrap().insert(
        room_id.clone(),
        JoinedRoomDetails {
            room_id: room_id.clone(),
            timeline,
            timeline_singleton_endpoints: Some((timeline_update_receiver, request_sender)),
            timeline_update_sender,
            timeline_subscriber_handler_task,
            typing_notice_subscriber: None,
            replaces_tombstoned_room: tombstoned_room_replaced_by_this_room,
        },
    );

    // We need to add the room to the `ALL_JOINED_ROOMS` list before we can
    // send the `AddJoinedRoom` update to the UI, because the UI might immediately
    // issue a `MatrixRequest` that relies on that room being in `ALL_JOINED_ROOMS`.
    enqueue_rooms_list_update(RoomsListUpdate::AddJoinedRoom(JoinedRoomInfo {
        room_id,
        latest,
        tags: room.tags().await.ok().flatten().unwrap_or_default(),
        num_unread_messages: room.num_unread_messages(),
        num_unread_mentions: room.num_unread_mentions(),
        // start with a basic text avatar; the avatar image will be fetched asynchronously below.
        avatar: room.avatar_url(),
        room_name,
        canonical_alias: room.canonical_alias(),
        alt_aliases: room.alt_aliases(),
        has_been_paginated: false,
        is_selected: false,
    }));

    Ok(())
}

/// Invoked when the room list service has received an update that changes an existing room.
pub async fn update_room(
    old_room: &RoomListServiceRoomInfo,
    new_room: &matrix_sdk::Room,
    room_list_service: &RoomListService,
) -> anyhow::Result<()> {
    let new_room_id = new_room.room_id().to_owned();
    if old_room.room_id == new_room_id {
        let new_room_name = new_room.display_name().await.ok();
        let room_avatar_changed = false;

        // Handle state transitions for a room.
        let old_room_state = old_room.room_state;
        let new_room_state = new_room.state();
        if old_room_state != new_room_state {
            if LOG_ROOM_LIST_DIFFS {
                println!("Room {new_room_name:?} ({new_room_id}) changed from {old_room_state:?} to {new_room_state:?}");
            }
            match new_room_state {
                RoomState::Banned => {
                    // TODO: handle rooms that this user has been banned from.
                    println!("Removing Banned room: {new_room_name:?} ({new_room_id})");
                    remove_room(&new_room.into());
                    return Ok(());
                }
                RoomState::Left => {
                    println!("Removing Left room: {new_room_name:?} ({new_room_id})");
                    remove_room(&new_room.into());
                    // TODO: we could add this to the list of left rooms,
                    //       which is collapsed by default.
                    //       Upon clicking a left room, we can show a splash page
                    //       that prompts the user to rejoin the room or forget it.

                    // TODO: this may also be called when a user rejects an invite, not sure.
                    //       So we might also need to make a new RoomsListUpdate::RoomLeft variant.
                    return Ok(());
                }
                RoomState::Joined => {
                    println!(
                        "update_room(): adding new Joined room: {new_room_name:?} ({new_room_id})"
                    );
                    return add_new_room(new_room, room_list_service).await;
                }
                RoomState::Invited => {
                    println!(
                        "update_room(): adding new Invited room: {new_room_name:?} ({new_room_id})"
                    );
                    return add_new_room(new_room, room_list_service).await;
                }
                RoomState::Knocked => {
                    // TODO: handle Knocked rooms (e.g., can you re-knock? or cancel a prior knock?)
                    return Ok(());
                }
            }
        }

        // if let Some(new_latest_event) = new_room.latest_event() { TODO: check if this is still necessary
        //     if let Some(old_latest_event) = old_room.room.latest_event() {
        //         if new_latest_event. > old_latest_event.timestamp() {
        //             println!("Updating latest event for room {}", new_room_id);
        //             room_avatar_changed =
        //                 update_latest_event(new_room_id.clone(), &new_latest_event, None);
        //         }
        //     }
        // }

        if room_avatar_changed || (old_room.room.avatar_url() != new_room.avatar_url()) {
            println!("Updating avatar for room {}", new_room_id);
            // spawn_fetch_room_avatar(new_room.inner_room().clone());
        }

        if let Some(new_room_name) = new_room_name {
            if old_room.room.cached_display_name().as_ref() != Some(&new_room_name) {
                println!(
                    "Updating room name for room {} to {}",
                    new_room_id, new_room_name
                );
                enqueue_rooms_list_update(RoomsListUpdate::UpdateRoomName {
                    room_id: new_room_id.clone(),
                    new_room_name,
                });
            }
        }

        if let Ok(new_tags) = new_room.tags().await {
            enqueue_rooms_list_update(RoomsListUpdate::Tags {
                room_id: new_room_id.clone(),
                new_tags: new_tags.unwrap_or_default(),
            });
        }

        enqueue_rooms_list_update(RoomsListUpdate::UpdateNumUnreadMessages {
            room_id: new_room_id.clone(),
            count: UnreadMessageCount::Known(new_room.num_unread_messages()),
            unread_mentions: new_room.num_unread_mentions(),
        });

        Ok(())
    } else {
        println!(
            "UNTESTED SCENARIO: update_room(): removing old room {}, replacing with new room {}",
            old_room.room_id, new_room_id,
        );
        remove_room(old_room);
        add_new_room(new_room, room_list_service).await
    }
}

/// Invoked when the room list service has received an update to remove an existing room.
pub fn remove_room(room: &RoomListServiceRoomInfo) {
    ALL_JOINED_ROOMS.lock().unwrap().remove(&room.room_id);
    enqueue_rooms_list_update(RoomsListUpdate::RemoveRoom {
        room_id: room.room_id.clone(),
        new_state: room.room_state,
    });
}
