use std::iter::Peekable;

use anyhow::bail;
use futures::{pin_mut, StreamExt};
use matrix_sdk::{Client, Room};
use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    sync_service::SyncService,
    RoomListService,
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
    let sync_service = SyncService::builder(client)
        .with_offline_mode()
        .build()
        .await?;

    // Start the sync service
    sync_service.start().await;
    let room_list_service = sync_service.room_list_service();
    SYNC_SERVICE
        .set(sync_service)
        .unwrap_or_else(|_| panic!("BUG: SYNC_SERVICE already set!"));

    let all_rooms_list = room_list_service.all_rooms().await?;
    handle_room_list_service_loading_state(all_rooms_list.loading_state());

    // TODO: paginate the rooms instead of getting them all
    let (room_diff_stream, room_list_dynamic_entries_controller) =
        all_rooms_list.entries_with_dynamic_adapters(usize::MAX);

    room_list_dynamic_entries_controller.set_filter(Box::new(|_room| true));

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
                        println!("State of appended room: {:?}", new_room.state());
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
                    println!("State of appended room: {:?}", new_room.state());
                    add_new_room(&new_room, &room_list_service).await?;
                    all_known_rooms.push_front(new_room.into());
                }
                VectorDiff::PushBack { value: new_room } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PushBack");
                    }
                    println!("State of appended room: {:?}", new_room.state());
                    add_new_room(&new_room, &room_list_service).await?;
                    all_known_rooms.push_back(new_room.into());
                }
                remove_diff @ VectorDiff::PopFront => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PopFront");
                    }
                    if let Some(room) = all_known_rooms.pop_front() {
                        optimize_remove_then_add_into_update(
                            remove_diff,
                            &room,
                            &mut peekable_diffs,
                            &mut all_known_rooms,
                            &room_list_service,
                        )
                        .await?;
                    }
                }
                remove_diff @ VectorDiff::PopBack => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff PopBack");
                    }
                    if let Some(room) = all_known_rooms.pop_back() {
                        optimize_remove_then_add_into_update(
                            remove_diff,
                            &room,
                            &mut peekable_diffs,
                            &mut all_known_rooms,
                            &room_list_service,
                        )
                        .await?;
                    }
                }
                VectorDiff::Insert {
                    index,
                    value: new_room,
                } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Insert at {index}");
                    }
                    println!("State of appended room: {:?}", new_room.state());
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
                remove_diff @ VectorDiff::Remove {
                    index: remove_index,
                } => {
                    if LOG_ROOM_LIST_DIFFS {
                        println!("room_list: diff Remove at {remove_index}");
                    }
                    if remove_index < all_known_rooms.len() {
                        let room = all_known_rooms.remove(remove_index);
                        optimize_remove_then_add_into_update(
                            remove_diff,
                            &room,
                            &mut peekable_diffs,
                            &mut all_known_rooms,
                            &room_list_service,
                        )
                        .await?;
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

/// Attempts to optimize a common RoomListService operation of remove + add.
///
/// If a `Remove` diff (or `PopBack` or `PopFront`) is immediately followed by
/// an `Insert` diff (or `PushFront` or `PushBack`) for the same room,
/// we can treat it as a simple `Set` operation, in which we call `update_room()`.
/// This is much more efficient than removing the room and then adding it back.
///
/// This tends to happen frequently in order to change the room's state
/// or to "sort" the room list by changing its positional order.
async fn optimize_remove_then_add_into_update(
    remove_diff: VectorDiff<Room>,
    room: &RoomListServiceRoomInfo,
    peekable_diffs: &mut Peekable<impl Iterator<Item = VectorDiff<matrix_sdk::Room>>>,
    all_known_rooms: &mut Vector<RoomListServiceRoomInfo>,
    room_list_service: &RoomListService,
) -> anyhow::Result<()> {
    let next_diff_was_handled: bool;
    match peekable_diffs.peek() {
        Some(VectorDiff::Insert {
            index: insert_index,
            value: new_room,
        }) if room.room_id == new_room.room_id() => {
            if LOG_ROOM_LIST_DIFFS {
                println!(
                    "Optimizing {remove_diff:?} + Insert({insert_index}) into Update for room {}",
                    room.room_id
                );
            }
            update_room(room, new_room, room_list_service).await?;
            all_known_rooms.insert(*insert_index, new_room.clone().into());
            next_diff_was_handled = true;
        }
        Some(VectorDiff::PushFront { value: new_room }) if room.room_id == new_room.room_id() => {
            if LOG_ROOM_LIST_DIFFS {
                println!(
                    "Optimizing {remove_diff:?} + PushFront into Update for room {}",
                    room.room_id
                );
            }
            update_room(room, new_room, room_list_service).await?;
            all_known_rooms.push_front(new_room.clone().into());
            next_diff_was_handled = true;
        }
        Some(VectorDiff::PushBack { value: new_room }) if room.room_id == new_room.room_id() => {
            if LOG_ROOM_LIST_DIFFS {
                println!(
                    "Optimizing {remove_diff:?} + PushBack into Update for room {}",
                    room.room_id
                );
            }
            update_room(room, new_room, room_list_service).await?;
            all_known_rooms.push_back(new_room.clone().into());
            next_diff_was_handled = true;
        }
        _ => next_diff_was_handled = false,
    }
    if next_diff_was_handled {
        peekable_diffs.next(); // consume the next diff
    } else {
        remove_room(room);
    }
    Ok(())
}
