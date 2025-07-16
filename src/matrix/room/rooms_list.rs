use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
};

use anyhow::{anyhow, bail, Ok};
use crossbeam_queue::SegQueue;
use eyeball::Subscriber;
use matrix_sdk::{
    ruma::{
        events::tag::Tags, MilliSecondsSinceUnixEpoch, OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId,
    },
    RoomDisplayName, RoomState,
};
use matrix_sdk_ui::room_list_service::RoomListLoadingState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_svelte::{ManagerExt, StoreState};
use tokio::{
    runtime::Handle,
    sync::oneshot::{self, Sender},
};

use crate::matrix::{
    invited_room::InvitedRoomInfo,
    room::room_filter::{FilterableRoom, RoomDisplayFilterBuilder, RoomFilterCriteria, SortFn},
    rooms::UnreadMessageCount,
    singletons::{broadcast_event, UIUpdateMessage},
    stores::{
        room_store::send_room_creation_request_and_await_response,
        rooms_collection::ROOMS_COLLECTION_STORE_ID,
    },
};

use super::{room_filter::RoomDisplayFilter, room_screen::RoomScreen};

/// The possible updates that should be displayed by the single list of all rooms.
///
/// These updates are enqueued by the `enqueue_rooms_list_update` function
/// (which is called from background async tasks that receive updates from the matrix server),
/// and then dequeued by the `RoomsList` widget's `handle_event` function.
#[derive(Debug)]
pub enum RoomsListUpdate {
    /// No rooms have been loaded yet.
    NotLoaded,
    /// Some rooms were loaded, and the server optionally told us
    /// the max number of rooms that will ever be loaded.
    LoadedRooms { max_rooms: Option<u32> },
    /// Add a new room to the list of rooms the user has been invited to.
    /// This will be maintained and displayed separately from joined rooms.
    AddInvitedRoom(InvitedRoomInfo),
    /// Add a new room to the list of all rooms that the user has joined.
    AddJoinedRoom(JoinedRoomInfo),
    /// Clear all rooms in the list of all rooms.
    ClearRooms,
    /// Update the latest event content and timestamp for the given room.
    UpdateLatestEvent {
        room_id: OwnedRoomId,
        timestamp: MilliSecondsSinceUnixEpoch,
        /// The Html-formatted text preview of the latest message.
        latest_message_text: String,
    },
    /// Update the number of unread messages for the given room.
    UpdateNumUnreadMessages {
        room_id: OwnedRoomId,
        count: UnreadMessageCount,
        unread_mentions: u64,
    },
    /// Update the displayable name for the given room.
    UpdateRoomName {
        room_id: OwnedRoomId,
        new_room_name: RoomDisplayName,
    },
    /// Update the avatar (image) for the given room.
    // UpdateRoomAvatar {
    //     room_id: OwnedRoomId,
    //     avatar: RoomPreviewAvatar,
    // },
    /// Remove the given room from the rooms list
    RemoveRoom {
        room_id: OwnedRoomId,
        /// The new state of the room (which caused its removal).
        new_state: RoomState,
    },
    /// Update the tags for the given room.
    Tags {
        room_id: OwnedRoomId,
        new_tags: Tags,
    },
    /// Update the status label at the bottom of the list of all rooms.
    Status { status: RoomsCollectionStatus },
}

static PENDING_ROOM_UPDATES: SegQueue<RoomsListUpdate> = SegQueue::new();

/// Enqueue a new room update for the list of all rooms
/// and signals the UI that a new update is available to be handled.
pub fn enqueue_rooms_list_update(update: RoomsListUpdate) {
    PENDING_ROOM_UPDATES.push(update);
    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
}

/// UI-related info about a joined room.
///
/// This includes info needed display a preview of that room in the RoomsList
/// and to filter the list of rooms based on the current search filter.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinedRoomInfo {
    /// The matrix ID of this room.
    pub(crate) room_id: OwnedRoomId,
    /// The displayable name of this room, if known.
    pub(crate) room_name: Option<String>,
    /// The number of unread messages in this room.
    pub(crate) num_unread_messages: u64,
    /// The number of unread mentions in this room.
    pub(crate) num_unread_mentions: u64,
    /// The canonical alias for this room, if any.
    pub(crate) canonical_alias: Option<OwnedRoomAliasId>,
    /// The alternative aliases for this room, if any.
    pub(crate) alt_aliases: Vec<OwnedRoomAliasId>,
    /// The tags associated with this room, if any.
    /// This includes things like is_favourite, is_low_priority,
    /// whether the room is a server notice room, etc.
    #[serde(skip)]
    pub(crate) tags: Tags,
    /// The timestamp and Html text content of the latest message in this room.
    pub(crate) latest: Option<(MilliSecondsSinceUnixEpoch, String)>,
    /// The avatar for this room
    pub avatar: Option<OwnedMxcUri>,
    /// Whether this room has been paginated at least once.
    /// We pre-paginate visible rooms at least once in order to
    /// be able to display the latest message in the room preview,
    /// and to have something to immediately show when a user first opens a room.
    pub(crate) has_been_paginated: bool,
    /// Whether this room is currently selected in the UI.
    pub(crate) is_selected: bool,
    /// Whether this a direct room.
    pub(crate) is_direct: bool,
}

pub fn handle_room_list_service_loading_state(mut loading_state: Subscriber<RoomListLoadingState>) {
    println!(
        "Initial room list loading state is {:?}",
        loading_state.get()
    );
    Handle::current().spawn(async move {
        while let Some(state) = loading_state.next().await {
            println!("Received a room list loading state update: {state:?}");
            match state {
                RoomListLoadingState::NotLoaded => {
                    enqueue_rooms_list_update(RoomsListUpdate::NotLoaded);
                }
                RoomListLoadingState::Loaded {
                    maximum_number_of_rooms,
                } => {
                    enqueue_rooms_list_update(RoomsListUpdate::LoadedRooms {
                        max_rooms: maximum_number_of_rooms,
                    });
                }
            }
        }
    });
}

// Frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomsList {
    /// The list of all rooms that the user has been invited to.
    invited_rooms: HashMap<OwnedRoomId, InvitedRoomInfo>,

    /// The set of all joined rooms and their cached preview info.
    /// Frontend Svelte : `allJoinedRooms` Record param of the rooms-collection store
    all_joined_rooms: HashMap<OwnedRoomId, JoinedRoomInfo>,

    /// The currently-active filter function for the list of rooms.
    ///
    /// Note: for performance reasons, this does not get automatically applied
    /// when its value changes. Instead, you must manually invoke it on the set of `all_joined_rooms`
    /// in order to update the set of `displayed_rooms` accordingly.
    /// Frontend Svelte : not implemented yet
    #[serde(skip)]
    display_filter: RoomDisplayFilter,

    /// The list of invited rooms currently displayed in the UI, in order from top to bottom.
    /// This is a strict subset of the rooms present in `all_invited_rooms`, and should be determined
    /// by applying the `display_filter` to the set of `all_invited_rooms`.
    displayed_invited_rooms: Vec<OwnedRoomId>,

    /// The list of joined rooms currently displayed in the UI, in order from top to bottom.
    /// This is a strict subset of the rooms present in `all_joined_rooms`, and should be determined
    /// by applying the `display_filter` to the set of `all_joined_rooms`.
    /// Frontend Svelte : `displayedJoinedRooms` array of the rooms-collection store
    displayed_joined_rooms: Vec<OwnedRoomId>,

    /// The latest status message that should be displayed in the bottom status label.
    /// Frontend Svelte : `status` param of the rooms-collection store
    status: RoomsCollectionStatus,
    /// The ID of the currently-selected room.
    /// Frontend Svelte : `currentActiveRoom` param of the rooms-collection store
    current_active_room: Option<OwnedRoomId>,
    /// The current active room sender to interrupt the task when room is closed.
    /// Backend only
    #[serde(skip)]
    current_active_room_killer: Option<Sender<()>>,
    /// The maximum number of rooms that will ever be loaded.
    /// Frontend Svelte : `maxKnownRooms` param of the rooms-collection store
    max_known_rooms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "status",
    content = "message"
)]
pub enum RoomsCollectionStatus {
    NotLoaded(String),
    Loading(String),
    Loaded(String),
    Error(String),
}

impl RoomsList {
    pub fn new() -> Self {
        Self {
            invited_rooms: HashMap::default(),
            all_joined_rooms: HashMap::default(),
            display_filter: RoomDisplayFilter::default(),
            displayed_joined_rooms: Vec::new(),
            displayed_invited_rooms: Vec::new(),
            status: RoomsCollectionStatus::NotLoaded("Initiating".to_string()),
            current_active_room: None,
            current_active_room_killer: None,
            max_known_rooms: None,
        }
    }

    pub fn patch_frontend_store_with_current_state<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        let json = serde_json::to_value(self).expect("Couldn't serialize Rooms List");
        let mut empty_state = StoreState::new();
        let state = match json {
            Value::Object(map) => {
                let hashmap: HashMap<String, Value> = map.into_iter().collect();
                empty_state.patch(hashmap);
                Ok(empty_state)
            }
            _ => Err(anyhow!(
                "Unexpected JSON object received during serialization"
            )),
        };
        app_handle
            .svelte()
            .patch(
                ROOMS_COLLECTION_STORE_ID,
                state.expect("Wrong state sent to frontend"),
            )
            .expect("Couldn't patch the frontend state");
    }

    /// Handle all pending updates to the list of all rooms.
    pub async fn handle_rooms_list_updates<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        let mut num_updates: usize = 0;
        while let Some(update) = PENDING_ROOM_UPDATES.pop() {
            num_updates += 1;

            #[cfg(debug_assertions)]
            println!("Processing update type: {update:?}");

            match update {
                RoomsListUpdate::AddInvitedRoom(invited_room) => {
                    let room_id = invited_room.room_id.clone();
                    let should_display = (self.display_filter)(&invited_room);
                    let _replaced = self
                        .invited_rooms
                        .borrow_mut()
                        .insert(room_id.clone(), invited_room);
                    if let Some(_old_room) = _replaced {
                        eprintln!("BUG: Added invited room {room_id} that already existed");
                    } else {
                        if should_display {
                            self.displayed_invited_rooms.push(room_id);
                        }
                    }
                    self.update_status_rooms_count();
                }
                RoomsListUpdate::AddJoinedRoom(joined_room) => {
                    let room_id = joined_room.room_id.clone();
                    let should_display = (self.display_filter)(&joined_room);
                    let _replaced = self.all_joined_rooms.insert(room_id.clone(), joined_room);
                    if let Some(_old_room) = _replaced {
                        eprintln!("BUG: Added joined room {room_id} that already existed");
                    } else {
                        if should_display {
                            // Create Svelte store
                            send_room_creation_request_and_await_response(
                                &app_handle,
                                room_id.as_str(),
                            )
                            .await
                            .expect("Couldn't create svelte store");

                            self.displayed_joined_rooms.push(room_id.clone());
                        }
                    }
                    // If this room was added as a result of accepting an invite, we must:
                    // 1. Remove the room from the list of invited rooms.
                    // 2. Update the displayed invited rooms list to remove this room.
                    if let Some(_accepted_invite) = self.invited_rooms.borrow_mut().remove(&room_id)
                    {
                        println!("Removed room {room_id} from the list of invited rooms");
                        self.displayed_invited_rooms
                            .iter()
                            .position(|r| r == &room_id)
                            .map(|index| self.displayed_invited_rooms.remove(index));
                    }
                    self.update_status_rooms_count();
                }
                // RoomsListUpdate::UpdateRoomAvatar { room_id, avatar } => {
                //     if let Some(room) = self.all_joined_rooms.get_mut(&room_id) {
                //         room.avatar = avatar;
                //     } else {
                //         eprintln!("Error: couldn't find room {room_id} to update avatar");
                //     }
                // }
                RoomsListUpdate::UpdateLatestEvent {
                    room_id,
                    timestamp,
                    latest_message_text,
                } => {
                    if let Some(room) = self.all_joined_rooms.get_mut(&room_id) {
                        room.latest = Some((timestamp, latest_message_text.clone()));
                    } else {
                        eprintln!("Error: couldn't find room {room_id} to update latest event");
                    }
                }
                RoomsListUpdate::UpdateNumUnreadMessages {
                    room_id,
                    count,
                    unread_mentions,
                } => {
                    if let Some(room) = self.all_joined_rooms.get_mut(&room_id) {
                        (room.num_unread_messages, room.num_unread_mentions) = match count {
                            UnreadMessageCount::Unknown => (0, 0),
                            UnreadMessageCount::Known(count) => (count, unread_mentions),
                        };
                    } else {
                        eprintln!(
                            "Error: couldn't find room {} to update unread messages count",
                            room_id
                        );
                    }
                }
                RoomsListUpdate::UpdateRoomName {
                    room_id,
                    new_room_name,
                } => {
                    if let Some(room) = self.all_joined_rooms.get_mut(&room_id) {
                        let was_displayed = (self.display_filter)(room);
                        room.room_name = Some(new_room_name.to_string());
                        let should_display = (self.display_filter)(room);
                        match (was_displayed, should_display) {
                            (true, true) | (false, false) => {
                                // No need to update the displayed rooms list.
                            }
                            (true, false) => {
                                // Room was displayed but should no longer be displayed.
                                self.displayed_joined_rooms
                                    .iter()
                                    .position(|r| r == &room_id)
                                    .map(|index| self.displayed_joined_rooms.remove(index));
                            }
                            (false, true) => {
                                // Room was not displayed but should now be displayed.
                                self.displayed_joined_rooms.push(room_id);
                            }
                        }
                    } else {
                        eprintln!("Error: couldn't find room {room_id} to update room name");
                    }
                }
                RoomsListUpdate::RemoveRoom {
                    room_id,
                    new_state: _,
                } => {
                    if let Some(_removed) = self.all_joined_rooms.remove(&room_id) {
                        println!("Removed room {room_id} from the list of all joined rooms");
                        if let Some(_removed) = self.invited_rooms.borrow_mut().remove(&room_id) {
                            println!("Removed room {room_id} from the list of all invited rooms");
                            self.displayed_invited_rooms
                                .iter()
                                .position(|r| r == &room_id)
                                .map(|index| self.displayed_invited_rooms.remove(index));
                        } else {
                            self.displayed_joined_rooms
                                .iter()
                                .position(|r| r == &room_id)
                                .map(|index| self.displayed_joined_rooms.remove(index));
                        };
                    }
                    self.update_status_rooms_count();
                }
                RoomsListUpdate::ClearRooms => {
                    self.all_joined_rooms.clear();
                    self.displayed_joined_rooms.clear();
                    self.invited_rooms.borrow_mut().clear();
                    self.displayed_invited_rooms.clear();
                    self.update_status_rooms_count();
                }
                RoomsListUpdate::NotLoaded => {
                    self.status = RoomsCollectionStatus::Loading(
                        "Loading rooms (waiting for homeserver)...".to_string(),
                    );
                }
                RoomsListUpdate::LoadedRooms { max_rooms } => {
                    self.max_known_rooms = max_rooms;
                    self.update_status_rooms_count();
                }
                RoomsListUpdate::Tags { room_id, new_tags } => {
                    if let Some(room) = self.all_joined_rooms.get_mut(&room_id) {
                        room.tags = new_tags;
                    } else if let Some(_room) = self.invited_rooms.borrow().get(&room_id) {
                        println!("Ignoring updated tags update for invited room {room_id}");
                    } else {
                        eprintln!("Error: skipping updated Tags for unknown room {room_id}.");
                    }
                }
                RoomsListUpdate::Status { status } => {
                    self.status = status;
                }
            }
        }
        if num_updates > 0 {
            println!(
                "RoomsList: processed {} updates to the list of all rooms",
                num_updates
            );
            // updates the frontend
            self.patch_frontend_store_with_current_state(&app_handle);
        }
    }

    pub fn handle_current_active_room<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        updated_current_active_room: Option<OwnedRoomId>,
        mut room_name: Option<String>,
    ) -> anyhow::Result<()> {
        println!("{updated_current_active_room:?}");
        self.current_active_room = updated_current_active_room;
        match self.current_active_room.clone() {
            Some(id) => {
                let mut ui_subscriber = crate::matrix::singletons::subscribe_to_events()
                    .expect("Couldn't get UI subscriber event");
                let (tx, mut rx) = oneshot::channel::<()>();
                self.current_active_room_killer = Some(tx);
                let room_screen_app_handle = app_handle.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut room_screen = RoomScreen::new(
                        id,
                        room_name
                            .take()
                            .expect("Room name should be defined if room_id is"),
                    );
                    room_screen.show_timeline(&room_screen_app_handle);
                    // TODO handle message actions here or within room_screen ?

                    loop {
                        tokio::select! {
                            _ = ui_subscriber.recv() => {
                                room_screen.process_timeline_updates(&room_screen_app_handle);
                            }
                            _ = &mut rx => {
                                break;
                            }
                        }
                    }
                    // as soon as this task is done,
                    // the room_screen will be dropped,
                    // and hide_timeline() will be called on drop
                });
                Ok(())
            }
            None => {
                if let Some(sender) = self.current_active_room_killer.take() {
                    sender.send(()).map_err(|e| {
                        anyhow!("Error while sending message to terminate RoomScreen thread {e:?}")
                    })
                } else {
                    bail!("Sender hasn't been set properly !");
                }
            }
        }
        // match the option
        // if none, then send a message to the running thread to end task
        // if yes then try to spawn the corresponding RoomScreen with a receiver and store the sender somewhere in a map
        // this thread will also handle room actions such as sending messages
    }

    /// Updates the status message to show how many rooms have been loaded.
    fn update_status_rooms_count(&mut self) {
        let num_rooms = self.all_joined_rooms.len() + self.invited_rooms.borrow().len();
        self.status = if let Some(max_rooms) = self.max_known_rooms {
            let message = format!("Loaded {num_rooms} of {max_rooms} total rooms.");
            if num_rooms as u32 == max_rooms {
                RoomsCollectionStatus::Loaded(message)
            } else {
                RoomsCollectionStatus::Loading(message)
            }
        } else {
            RoomsCollectionStatus::Loaded(format!("Loaded {num_rooms} rooms."))
        };
    }

    /// Updates the status message to show how many rooms are currently displayed
    /// that match the current search filter.
    fn _update_status_matching_rooms(&mut self) {
        let num_rooms = self.displayed_joined_rooms.len() + self.displayed_invited_rooms.len();
        self.status = match num_rooms {
            0 => RoomsCollectionStatus::Loaded("No matching rooms found.".to_string()),
            1 => RoomsCollectionStatus::Loaded("Found 1 matching room.".to_string()),
            n => RoomsCollectionStatus::Loaded(format!("Found {} matching rooms.", n)),
        }
    }

    /// Returns true if the given room is contained in any of the displayed room sets,
    /// i.e., either the invited rooms or the joined rooms.
    fn _is_room_displayable(&self, room: &OwnedRoomId) -> bool {
        self.displayed_invited_rooms.contains(room) || self.displayed_joined_rooms.contains(room)
    }

    /// Updates the lists of displayed rooms based on the current search filter
    /// and redraws the RoomsList.
    fn _update_displayed_rooms(&mut self, keywords: &str) {
        // let portal_list = self.view.portal_list(id!(list));
        if keywords.is_empty() {
            // Reset the displayed rooms list to show all rooms.
            self.display_filter = RoomDisplayFilter::default();
            self.displayed_joined_rooms = self.all_joined_rooms.keys().cloned().collect();
            self.displayed_invited_rooms = self.invited_rooms.borrow().keys().cloned().collect();
            self.update_status_rooms_count();
            // portal_list.set_first_id_and_scroll(0, 0.0);
            // self.redraw(cx);
            return;
        }

        // Create a new filter function based on the given keywords
        // and store it in this RoomsList such that we can apply it to newly-added rooms.
        let (filter, sort_fn) = RoomDisplayFilterBuilder::new()
            .set_keywords(keywords.to_owned())
            .set_filter_criteria(RoomFilterCriteria::All)
            .build();
        self.display_filter = filter;

        /// An inner function that generates a sorted, filtered list of rooms to display.
        fn generate_displayed_rooms<FR: FilterableRoom>(
            rooms_map: &HashMap<OwnedRoomId, FR>,
            display_filter: &RoomDisplayFilter,
            sort_fn: Option<&SortFn>,
        ) -> Vec<OwnedRoomId>
        where
            FR: FilterableRoom + Send + Sync + 'static,
        {
            if let Some(sort_fn) = sort_fn {
                let mut filtered_rooms: Vec<_> = rooms_map
                    .iter()
                    .filter(|(_, room)| {
                        let room_trait: &(dyn FilterableRoom + Send + Sync) = *room;
                        display_filter(room_trait)
                    })
                    .collect();
                filtered_rooms.sort_by(|(_, room_a), (_, room_b)| {
                    let room_a_trait: &(dyn FilterableRoom + Send + Sync) = *room_a;
                    let room_b_trait: &(dyn FilterableRoom + Send + Sync) = *room_b;
                    sort_fn(room_a_trait, room_b_trait)
                });
                filtered_rooms
                    .into_iter()
                    .map(|(room_id, _)| room_id.clone())
                    .collect()
            } else {
                rooms_map
                    .iter()
                    .filter(|(_, room)| display_filter(*room))
                    .map(|(room_id, _)| room_id.clone())
                    .collect()
            }
        }

        // Update the displayed rooms list and redraw it.
        self.displayed_joined_rooms = generate_displayed_rooms(
            &self.all_joined_rooms,
            &self.display_filter,
            sort_fn.as_deref(),
        );
        self.displayed_invited_rooms = generate_displayed_rooms(
            &self.invited_rooms.borrow(),
            &self.display_filter,
            sort_fn.as_deref(),
        );
        self._update_status_matching_rooms();
        // portal_list.set_first_id_and_scroll(0, 0.0);
        // self.redraw(cx);
    }
}
