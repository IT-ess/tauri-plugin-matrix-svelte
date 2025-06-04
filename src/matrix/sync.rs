use anyhow::bail;
use futures::{pin_mut, StreamExt};
use matrix_sdk::{Client, RoomState};
use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    sync_service::SyncService,
};
use tauri::{AppHandle, Runtime};

use crate::matrix::{
    room::rooms_list::{
        enqueue_rooms_list_update, handle_room_list_service_loading_state, RoomsListUpdate,
    },
    rooms::{add_new_room, remove_room, update_room},
    singletons::{ALL_JOINED_ROOMS, LOG_ROOM_LIST_DIFFS},
};

use super::{rooms::RoomListServiceRoomInfo, singletons::SYNC_SERVICE};

pub async fn sync<R: Runtime>(_app_handle: &AppHandle<R>, client: Client) -> anyhow::Result<()> {
    let sync_service = SyncService::builder(client).build().await?;

    // Start the sync service
    sync_service.start().await;
    let room_list_service = sync_service.room_list_service();
    SYNC_SERVICE
        .set(sync_service)
        .unwrap_or_else(|_| panic!("BUG: SYNC_SERVICE already set!"));

    let all_rooms_list = room_list_service.all_rooms().await?;
    handle_room_list_service_loading_state(all_rooms_list.loading_state());

    let (room_diff_stream, room_list_dynamic_entries_controller) =
        all_rooms_list.entries_with_dynamic_adapters(usize::MAX);

    // Handle only joined rooms for the moment TODO: handle all room types
    room_list_dynamic_entries_controller.set_filter(Box::new(|room| match room.state() {
        RoomState::Joined => true,
        _ => false,
    }));

    let mut all_known_rooms: Vector<RoomListServiceRoomInfo> = Vector::new();

    pin_mut!(room_diff_stream);
    while let Some(batch) = room_diff_stream.next().await {
        let mut peekable_diffs = batch.into_iter().peekable();
        while let Some(diff) = peekable_diffs.next() {
            match diff {
                VectorDiff::Append { values: new_rooms } => {
                    let _num_new_rooms = new_rooms.len();
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Append {_num_new_rooms}");
                    }
                    for new_room in new_rooms {
                        add_new_room(&new_room, &room_list_service).await?;
                        all_known_rooms.push_back(new_room.into());
                    }
                }
                VectorDiff::Clear => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Clear");
                    }
                    all_known_rooms.clear();
                    ALL_JOINED_ROOMS.lock().unwrap().clear();
                    enqueue_rooms_list_update(RoomsListUpdate::ClearRooms);
                }
                VectorDiff::PushFront { value: new_room } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PushFront");
                    }
                    add_new_room(&new_room, &room_list_service).await?;
                    all_known_rooms.push_front(new_room.into());
                }
                VectorDiff::PushBack { value: new_room } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PushBack");
                    }
                    add_new_room(&new_room, &room_list_service).await?;
                    all_known_rooms.push_back(new_room.into());
                }
                VectorDiff::PopFront => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PopFront");
                    }
                    if let Some(room) = all_known_rooms.pop_front() {
                        if LOG_ROOM_LIST_DIFFS {
                            println!("PopFront: removing {}", room.room_id);
                        }
                        remove_room(&room);
                    }
                }
                VectorDiff::PopBack => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PopBack");
                    }
                    if let Some(room) = all_known_rooms.pop_back() {
                        if LOG_ROOM_LIST_DIFFS {
                            println!("PopBack: removing {}", room.room_id);
                        }
                        remove_room(&room);
                    }
                }
                VectorDiff::Insert {
                    index,
                    value: new_room,
                } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Insert at {index}");
                    }
                    add_new_room(&new_room, &room_list_service).await?;
                    all_known_rooms.insert(index, new_room.into());
                }
                VectorDiff::Set {
                    index,
                    value: changed_room,
                } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Set at {index}");
                    }
                    if let Some(old_room) = all_known_rooms.get(index) {
                        update_room(old_room, &changed_room, &room_list_service).await?;
                    } else {
                        eprintln!("BUG: room list diff: Set index {index} was out of bounds.");
                    }
                    all_known_rooms.set(index, changed_room.into());
                }
                VectorDiff::Remove {
                    index: remove_index,
                } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Remove at {remove_index}");
                    }
                    if remove_index < all_known_rooms.len() {
                        let room = all_known_rooms.remove(remove_index);
                        // Try to optimize a common operation, in which a `Remove` diff
                        // is immediately followed by an `Insert` diff for the same room,
                        // which happens frequently in order to "sort" the room list
                        // by changing its positional order.
                        // We treat this as a simple `Set` operation (`update_room()`),
                        // which is way more efficient.
                        let mut next_diff_was_handled = false;
                        if let Some(VectorDiff::Insert {
                            index: insert_index,
                            value: new_room,
                        }) = peekable_diffs.peek()
                        {
                            if room.room_id == new_room.room_id() {
                                if LOG_ROOM_LIST_DIFFS {
                                    println!("Optimizing Remove({remove_index}) + Insert({insert_index}) into Set (update) for room {}", room.room_id);
                                }
                                update_room(&room, new_room, &room_list_service).await?;
                                all_known_rooms.insert(*insert_index, new_room.clone().into());
                                next_diff_was_handled = true;
                            }
                        }
                        if next_diff_was_handled {
                            peekable_diffs.next(); // consume the next diff
                        } else {
                            println!("UNTESTED SCENARIO: room_list: diff Remove({remove_index}) was NOT followed by an Insert. Removed room: {}", room.room_id);
                            remove_room(&room);
                        }
                    } else {
                        eprintln!("BUG: room_list: diff Remove index {remove_index} out of bounds, len {}", all_known_rooms.len());
                    }
                }
                VectorDiff::Truncate { length } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Truncate to {length}");
                    }
                    // Iterate manually so we can know which rooms are being removed.
                    while all_known_rooms.len() > length {
                        if let Some(room) = all_known_rooms.pop_back() {
                            remove_room(&room);
                        }
                    }
                    all_known_rooms.truncate(length); // sanity check
                }
                VectorDiff::Reset { values: new_rooms } => {
                    // We implement this by clearing all rooms and then adding back the new values.
                    if LOG_ROOM_LIST_DIFFS {
                        println!(
                            "room_list: diff Reset, old length {}, new length {}",
                            all_known_rooms.len(),
                            new_rooms.len()
                        );
                    }
                    // Iterate manually so we can know which rooms are being removed.
                    while let Some(room) = all_known_rooms.pop_back() {
                        remove_room(&room);
                    }
                    // ALL_JOINED_ROOMS should already be empty due to successive calls to `remove_room()`,
                    // so this is just a sanity check.
                    ALL_JOINED_ROOMS.lock().unwrap().clear();
                    enqueue_rooms_list_update(RoomsListUpdate::ClearRooms);
                    for room in &new_rooms {
                        add_new_room(room, &room_list_service).await?;
                    }
                    all_known_rooms = new_rooms.into_iter().map(|r| r.into()).collect();
                }
            }
        }
    }

    bail!("room list service sync loop ended unexpectedly")
}
