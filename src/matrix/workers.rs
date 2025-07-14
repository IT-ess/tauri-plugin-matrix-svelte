use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::bail;
use futures::{pin_mut, StreamExt};
use matrix_sdk::{
    ruma::{
        api::client::receipt::create_receipt::v3::ReceiptType, matrix_uri::MatrixId, OwnedRoomId,
        RoomOrAliasId,
    },
    Client, RoomMemberships,
};
use tauri::{AppHandle, Listener, Manager, Runtime};
use tokio::{
    runtime::Handle,
    sync::{mpsc::UnboundedReceiver, Mutex},
    task::JoinHandle,
};

use crate::{
    matrix::{
        notifications::enqueue_popup_notification,
        requests::submit_async_request,
        room::rooms_list::{enqueue_rooms_list_update, RoomsCollectionStatus, RoomsListUpdate},
        rooms::UnreadMessageCount,
        singletons::{broadcast_event, UIUpdateMessage, ALL_JOINED_ROOMS, CLIENT},
        sync::sync,
        timeline::{PaginationDirection, TimelineUpdate},
        user_power_level::UserPowerLevels,
        user_profile::{
            enqueue_user_profile_update, process_user_profile_updates, UserProfile,
            UserProfileUpdate,
        },
        utils::current_user_id,
    },
    models::matrix::{MatrixSvelteListenEvent, MatrixUpdateCurrentActiveRoom},
};

use super::{requests::MatrixRequest, room::rooms_list::RoomsList, utils::debounce_broadcast};

/// The main loop that actually uses a Matrix client
pub async fn async_main_loop<R: Runtime>(
    app_handle: AppHandle<R>,
    client: Client,
) -> anyhow::Result<()> {
    let logged_in_user_id = client
        .user_id()
        .expect("BUG: client.user_id() returned None after successful login!");
    let status = RoomsCollectionStatus::Loading(format!(
        "Logged in as {}.\n → Loading rooms...",
        logged_in_user_id
    ));
    // enqueue_popup_notification(status.clone());
    enqueue_rooms_list_update(RoomsListUpdate::Status { status });

    // Listen for updates to the ignored user list.
    // handle_ignore_user_list_subscriber(client.clone());
    let rooms_list_handle = app_handle.app_handle().clone();
    tauri::async_runtime::spawn(ui_worker(rooms_list_handle));

    // call sync
    sync(&app_handle, client).await?;

    bail!("room list service sync loop ended unexpectedly")
}

/// The entry point for an async worker thread that can run async tasks.
///
/// All this thread does is wait for [`MatrixRequests`] from the main UI-driven non-async thread(s)
/// and then executes them within an async runtime context.
pub async fn async_worker(
    mut request_receiver: UnboundedReceiver<MatrixRequest>,
) -> anyhow::Result<()> {
    println!("Started async_worker task.");
    let mut tasks_list: BTreeMap<OwnedRoomId, JoinHandle<()>> = BTreeMap::new();
    while let Some(request) = request_receiver.recv().await {
        match request {
            MatrixRequest::PaginateRoomTimeline {
                room_id,
                num_events,
                direction,
            } => {
                let (timeline, sender) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        println!("Skipping pagination request for not-yet-known room {room_id}");
                        continue;
                    };

                    let timeline_ref = room_info.timeline.clone();
                    let sender = room_info.timeline_update_sender.clone();
                    (timeline_ref, sender)
                };

                // Spawn a new async task that will make the actual pagination request.
                let _paginate_task = Handle::current().spawn(async move {
                    println!("Starting {direction} pagination request for room {room_id}...");
                    sender.send(TimelineUpdate::PaginationRunning(direction)).unwrap();
                    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");

                    let res = if direction == PaginationDirection::Forwards {
                        timeline.paginate_forwards(num_events).await
                    } else {
                        timeline.paginate_backwards(num_events).await
                    };

                    match res {
                        Ok(fully_paginated) => {
                            println!("Completed {direction} pagination request for room {room_id}, hit {} of timeline? {}",
                                if direction == PaginationDirection::Forwards { "end" } else { "start" },
                                if fully_paginated { "yes" } else { "no" },
                            );
                            sender.send(TimelineUpdate::PaginationIdle {
                                fully_paginated,
                                direction,
                            }).unwrap();
                            broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                        }
                        Err(error) => {
                            eprintln!("Error sending {direction} pagination request for room {room_id}: {error:?}");
                            sender.send(TimelineUpdate::PaginationError {
                                error,
                                direction,
                            }).unwrap();
                            broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                        }
                    }
                });
            }

            MatrixRequest::EditMessage {
                room_id,
                timeline_event_item_id: timeline_event_id,
                edited_content,
            } => {
                let (timeline, sender) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        eprintln!("BUG: room info not found for edit request, room {room_id}");
                        continue;
                    };
                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                // Spawn a new async task that will make the actual edit request.
                let _edit_task = Handle::current().spawn(async move {
                    println!("Sending request to edit message {timeline_event_id:?} in room {room_id}...");
                    let result = timeline.edit(&timeline_event_id, edited_content).await;
                    match result {
                        Ok(_) => println!("Successfully edited message {timeline_event_id:?} in room {room_id}."),
                        Err(ref e) => eprintln!("Error editing message {timeline_event_id:?} in room {room_id}: {e:?}"),
                    }
                    sender.send(TimelineUpdate::MessageEdited {
                        timeline_event_id,
                        result,
                    }).unwrap();
                    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                });
            }

            MatrixRequest::FetchDetailsForEvent { room_id, event_id } => {
                let (timeline, sender) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        eprintln!("BUG: room info not found for fetch details for event request {room_id}");
                        continue;
                    };

                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                // Spawn a new async task that will make the actual fetch request.
                let _fetch_task = Handle::current().spawn(async move {
                    // println!("Sending request to fetch details for event {event_id} in room {room_id}...");
                    let result = timeline.fetch_details_for_event(&event_id).await;
                    match result {
                        Ok(_) => {
                            // println!("Successfully fetched details for event {event_id} in room {room_id}.");
                        }
                        Err(ref _e) => {
                            // eprintln!("Error fetching details for event {event_id} in room {room_id}: {e:?}");
                        }
                    }
                    sender
                        .send(TimelineUpdate::EventDetailsFetched { event_id, result })
                        .unwrap();
                    broadcast_event(UIUpdateMessage::RefreshUI)
                        .expect("Couldn't broadcast event to UI");
                });
            }

            MatrixRequest::SyncRoomMemberList { room_id } => {
                let (timeline, sender) = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        eprintln!("BUG: room info not found for fetch members request {room_id}");
                        continue;
                    };

                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                // Spawn a new async task that will make the actual fetch request.
                let _fetch_task = Handle::current().spawn(async move {
                    println!("Sending sync room members request for room {room_id}...");
                    timeline.fetch_members().await;
                    println!("Completed sync room members request for room {room_id}.");
                    sender.send(TimelineUpdate::RoomMembersSynced).unwrap();

                    // Get room members details after the list sync.
                    submit_async_request(MatrixRequest::GetRoomMembers {
                        room_id,
                        memberships: RoomMemberships::all(),
                        local_only: false,
                    });
                    broadcast_event(UIUpdateMessage::RefreshUI)
                        .expect("Couldn't broadcast event to UI");
                });
            }

            // MatrixRequest::JoinRoom { room_id } => {
            //     let Some(client) = CLIENT.get() else { continue };
            //     let _join_room_task = Handle::current().spawn(async move {
            //         println!("Sending request to join room {room_id}...");
            //         let result_action = if let Some(room) = client.get_room(&room_id) {
            //             match room.join().await {
            //                 Ok(()) => {
            //                     println!("Successfully joined room {room_id}.");
            //                     JoinRoomAction::Joined { room_id }
            //                 }
            //                 Err(e) => {
            //                     eprintln!("Error joining room {room_id}: {e:?}");
            //                     JoinRoomAction::Failed { room_id, error: e }
            //                 }
            //             }
            //         } else {
            //             eprintln!("BUG: client could not get room with ID {room_id}");
            //             JoinRoomAction::Failed {
            //                 room_id,
            //                 error: matrix_sdk::Error::UnknownError(
            //                     String::from("Client couldn't locate room to join it.").into(),
            //                 ),
            //             }
            //         };
            //         Cx::post_action(result_action);
            //     });
            // }
            // MatrixRequest::LeaveRoom { room_id } => {
            //     let Some(client) = CLIENT.get() else { continue };
            //     let _leave_room_task = Handle::current().spawn(async move {
            //         println!("Sending request to leave room {room_id}...");
            //         let result_action = if let Some(room) = client.get_room(&room_id) {
            //             match room.leave().await {
            //                 Ok(()) => {
            //                     println!("Successfully left room {room_id}.");
            //                     LeaveRoomAction::Left { room_id }
            //                 }
            //                 Err(e) => {
            //                     eprintln!("Error leaving room {room_id}: {e:?}");
            //                     LeaveRoomAction::Failed {
            //                         room_id,
            //                         error: e.to_string(),
            //                     }
            //                 }
            //             }
            //         } else {
            //             eprintln!("BUG: client could not get room with ID {room_id}");
            //             LeaveRoomAction::Failed {
            //                 room_id,
            //                 error: String::from("Client couldn't locate room to leave it."),
            //             }
            //         };
            //         Cx::post_action(result_action);
            //     });
            // }
            MatrixRequest::GetRoomMembers {
                room_id,
                memberships,
                local_only,
            } => {
                let (timeline, sender) = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found for get room members request {room_id}");
                        continue;
                    };
                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                let _get_members_task = Handle::current().spawn(async move {
                    let room = timeline.room();

                    if local_only {
                        if let Ok(members) = room.members_no_sync(memberships).await {
                            println!(
                                "Got {} members from cache for room {}",
                                members.len(),
                                room_id
                            );
                            sender
                                .send(TimelineUpdate::RoomMembersListFetched { members })
                                .unwrap();
                        }
                    } else {
                        if let Ok(members) = room.members(memberships).await {
                            println!(
                                "Successfully fetched {} members from server for room {}",
                                members.len(),
                                room_id
                            );
                            sender
                                .send(TimelineUpdate::RoomMembersListFetched { members })
                                .unwrap();
                        }
                    }

                    broadcast_event(UIUpdateMessage::RefreshUI)
                        .expect("Couldn't broadcast event to UI");
                });
            }

            MatrixRequest::GetUserProfile {
                user_id,
                room_id,
                local_only,
            } => {
                let Some(client) = CLIENT.get() else { continue };
                let _fetch_task = Handle::current().spawn(async move {
                    // println!("Sending get user profile request: user: {user_id}, \
                    //     room: {room_id:?}, local_only: {local_only}...",
                    // );

                    let mut update = None;

                    if let Some(room_id) = room_id.as_ref() {
                        if let Some(room) = client.get_room(room_id) {
                            let member = if local_only {
                                room.get_member_no_sync(&user_id).await
                            } else {
                                room.get_member(&user_id).await
                            };
                            if let Ok(Some(room_member)) = member {
                                update = Some(UserProfileUpdate::Full {
                                    new_profile: UserProfile {
                                        username: room_member.display_name().map(|u| u.to_owned()),
                                        user_id: user_id.clone(),
                                        avatar_url: room_member.avatar_url().map(|u| u.to_owned()),
                                    },
                                    room_id: room_id.to_owned(),
                                    room_member,
                                });
                            } else {
                                println!("User profile request: user {user_id} was not a member of room {room_id}");
                            }
                        } else {
                            println!("User profile request: client could not get room with ID {room_id}");
                        }
                    }

                    if !local_only {
                        if update.is_none() {
                            if let Ok(response) = client.account().fetch_user_profile_of(&user_id).await {
                                update = Some(UserProfileUpdate::UserProfileOnly(
                                    UserProfile {
                                        username: response.displayname,
                                        user_id: user_id.clone(),
                                        avatar_url: response.avatar_url,
                                    }
                                ));
                            } else {
                                println!("User profile request: client could not get user with ID {user_id}");
                            }
                        }

                        match update.as_mut() {
                            Some(UserProfileUpdate::Full { new_profile: UserProfile { username, .. }, .. }) if username.is_none() => {
                                if let Ok(response) = client.account().fetch_user_profile_of(&user_id).await {
                                    *username = response.displayname;
                                }
                            }
                            _ => { }
                        }
                    }

                    if let Some(upd) = update {
                        // println!("Successfully completed get user profile request: user: {user_id}, room: {room_id:?}, local_only: {local_only}.");
                        enqueue_user_profile_update(upd);
                    } else {
                        println!("Failed to get user profile: user: {user_id}, room: {room_id:?}, local_only: {local_only}.");
                    }
                });
            }
            MatrixRequest::GetNumberUnreadMessages { room_id } => {
                let (timeline, sender) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        println!("Skipping get number of unread messages request for not-yet-known room {room_id}");
                        continue;
                    };

                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };
                let _get_unreads_task = Handle::current().spawn(async move {
                    match sender.send(TimelineUpdate::NewUnreadMessagesCount(
                        UnreadMessageCount::Known(timeline.room().num_unread_messages())
                    )) {
                        Ok(_) => {
                            broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                        },
                        Err(e) => println!("Failed to send timeline update: {e:?} for GetNumberUnreadMessages request for room {room_id}"),
                    }
                    enqueue_rooms_list_update(RoomsListUpdate::UpdateNumUnreadMessages {
                        room_id: room_id.clone(),
                        count: UnreadMessageCount::Known(timeline.room().num_unread_messages()),
                        unread_mentions:timeline.room().num_unread_mentions(),
                    });
                });
            }
            MatrixRequest::IgnoreUser {
                ignore,
                room_member,
                room_id,
            } => {
                let Some(client) = CLIENT.get() else { continue };
                let _ignore_task = Handle::current().spawn(async move {
                    let user_id = room_member.user_id();
                    println!("Sending request to {}ignore user: {user_id}...", if ignore { "" } else { "un" });
                    let ignore_result = if ignore {
                        room_member.ignore().await
                    } else {
                        room_member.unignore().await
                    };

                    println!("{} user {user_id} {}",
                        if ignore { "Ignoring" } else { "Unignoring" },
                        if ignore_result.is_ok() { "succeeded." } else { "failed." },
                    );

                    if ignore_result.is_err() {
                        return;
                    }

                    // We need to re-acquire the `RoomMember` object now that its state
                    // has changed, i.e., the user has been (un)ignored.
                    // We then need to send an update to replace the cached `RoomMember`
                    // with the now-stale ignored state.
                    if let Some(room) = client.get_room(&room_id) {
                        if let Ok(Some(new_room_member)) = room.get_member(user_id).await {
                            println!("Enqueueing user profile update for user {user_id}, who went from {}ignored to {}ignored.",
                                if room_member.is_ignored() { "" } else { "un" },
                                if new_room_member.is_ignored() { "" } else { "un" },
                            );
                            enqueue_user_profile_update(UserProfileUpdate::RoomMemberOnly {
                                room_id: room_id.clone(),
                                room_member: new_room_member,
                            });
                        }
                    }

                    // After successfully (un)ignoring a user, all timelines are fully cleared by the Matrix SDK.
                    // Therefore, we need to re-fetch all timelines for all rooms,
                    // and currently the only way to actually accomplish this is via pagination.
                    // See: <https://github.com/matrix-org/matrix-rust-sdk/issues/1703#issuecomment-2250297923>
                    //
                    // Note that here we only proactively re-paginate the *current* room
                    // (the one being viewed by the user when this ignore request was issued),
                    // and all other rooms will be re-paginated in `handle_ignore_user_list_subscriber()`.`
                    submit_async_request(MatrixRequest::PaginateRoomTimeline {
                        room_id,
                        num_events: 50,
                        direction: PaginationDirection::Backwards,
                    });
                });
            }

            MatrixRequest::SendTypingNotice { room_id, typing } => {
                let Some(room) = CLIENT.get().and_then(|c| c.get_room(&room_id)) else {
                    eprintln!("BUG: client/room not found for typing notice request {room_id}");
                    continue;
                };
                let _typing_task = Handle::current().spawn(async move {
                    if let Err(e) = room.typing_notice(typing).await {
                        eprintln!("Failed to send typing notice to room {room_id}: {e:?}");
                    }
                });
            }

            MatrixRequest::SubscribeToTypingNotices { room_id, subscribe } => {
                let (room, timeline_update_sender, mut typing_notice_receiver) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        println!("BUG: room info not found for subscribe to typing notices request, room {room_id}");
                        continue;
                    };
                    let (room, recv) = if subscribe {
                        if room_info.typing_notice_subscriber.is_some() {
                            println!(
                                "Note: room {room_id} is already subscribed to typing notices."
                            );
                            continue;
                        } else {
                            let Some(room) = CLIENT.get().and_then(|c| c.get_room(&room_id)) else {
                                eprintln!("BUG: client/room not found when subscribing to typing notices request, room: {room_id}");
                                continue;
                            };
                            let (drop_guard, recv) = room.subscribe_to_typing_notifications();
                            room_info.typing_notice_subscriber = Some(drop_guard);
                            (room, recv)
                        }
                    } else {
                        room_info.typing_notice_subscriber.take();
                        continue;
                    };
                    // Here: we don't have an existing subscriber running, so we fall through and start one.
                    (room, room_info.timeline_update_sender.clone(), recv)
                };

                let _typing_notices_task = Handle::current().spawn(async move {
                    while let Ok(user_ids) = typing_notice_receiver.recv().await {
                        // println!("Received typing notifications for room {room_id}: {user_ids:?}");
                        let mut users = Vec::with_capacity(user_ids.len());
                        for user_id in user_ids {
                            users.push(
                                room.get_member_no_sync(&user_id)
                                    .await
                                    .ok()
                                    .flatten()
                                    .and_then(|m| m.display_name().map(|d| d.to_owned()))
                                    .unwrap_or_else(|| user_id.to_string())
                            );
                        }
                        if let Err(e) = timeline_update_sender.send(TimelineUpdate::TypingUsers { users }) {
                            eprintln!("Error: timeline update sender couldn't send the list of typing users: {e:?}");
                        }
                        broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                    }
                    // println!("Note: typing notifications recv loop has ended for room {}", room_id);
                });
            }
            MatrixRequest::SubscribeToOwnUserReadReceiptsChanged { room_id, subscribe } => {
                if !subscribe {
                    if let Some(task_handler) = tasks_list.remove(&room_id) {
                        task_handler.abort();
                    }
                    continue;
                }
                let (timeline, sender) = {
                    let mut all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get_mut(&room_id) else {
                        println!("BUG: room info not found for subscribe to own user read receipts changed request, room {room_id}");
                        continue;
                    };
                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                let subscribe_own_read_receipt_task = Handle::current().spawn(async move {
                    let update_receiver = timeline.subscribe_own_user_read_receipts_changed().await;
                    pin_mut!(update_receiver);
                    if let Some(client_user_id) = current_user_id() {
                        if let Some((event_id, receipt)) =
                            timeline.latest_user_read_receipt(&client_user_id).await
                        {
                            println!("Received own user read receipt: {receipt:?} {event_id:?}");
                            if let Err(e) = sender.send(TimelineUpdate::OwnUserReadReceipt(receipt))
                            {
                                eprintln!("Failed to get own user read receipt: {e:?}");
                            }
                        }

                        while (update_receiver.next().await).is_some() {
                            if let Some((_, receipt)) =
                                timeline.latest_user_read_receipt(&client_user_id).await
                            {
                                if let Err(e) =
                                    sender.send(TimelineUpdate::OwnUserReadReceipt(receipt))
                                {
                                    eprintln!("Failed to get own user read receipt: {e:?}");
                                }
                            }
                        }
                    }
                });
                tasks_list.insert(room_id.clone(), subscribe_own_read_receipt_task);
            }
            // MatrixRequest::SpawnSSOServer {
            //     brand,
            //     homeserver_url,
            //     identity_provider_id,
            // } => {
            //     spawn_sso_server(
            //         brand,
            //         homeserver_url,
            //         identity_provider_id,
            //         login_sender.clone(),
            //     )
            //     .await;
            // }
            MatrixRequest::ResolveRoomAlias(room_alias) => {
                let Some(client) = CLIENT.get() else { continue };
                let _resolve_task = Handle::current().spawn(async move {
                    println!("Sending resolve room alias request for {room_alias}...");
                    let res = client.resolve_room_alias(&room_alias).await;
                    println!("Resolved room alias {room_alias} to: {res:?}");
                    todo!("Send the resolved room alias back to the UI thread somehow.");
                });
            }
            MatrixRequest::FetchMedia {
                media_request,
                content_sender,
            } => {
                let Some(client) = CLIENT.get() else { continue };
                let media = client.media();

                let _fetch_task = Handle::current().spawn(async move {
                    println!("Sending fetch media request for {media_request:?}...");
                    let res = media.get_media_content(&media_request, true).await;
                    content_sender
                        .send(res)
                        .expect("Couldn't send back fetched media")
                });
            }
            MatrixRequest::SendMessage {
                room_id,
                message,
                // replied_to,
            } => {
                let timeline = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found for send message request {room_id}");
                        continue;
                    };
                    room_info.timeline.clone()
                };

                // Spawn a new async task that will send the actual message.
                let _send_message_task = Handle::current().spawn(async move {
                    println!("Sending message to room {room_id}: {message:?}...");
                    // if let Some(replied_to_info) = replied_to {
                    //     match timeline
                    //         .send_reply(message.into(), replied_to_info, ForwardThread::Yes)
                    //         .await
                    //     {
                    //         Ok(_send_handle) => println!("Sent reply message to room {room_id}."),
                    //         Err(_e) => {
                    //             eprintln!("Failed to send reply message to room {room_id}: {_e:?}");
                    //             enqueue_popup_notification(format!("Failed to send reply: {_e}"));
                    //         }
                    //     }
                    // } else {
                    match timeline.send(message.into()).await {
                        Ok(_send_handle) => println!("Sent message to room {room_id}."),
                        Err(_e) => {
                            eprintln!("Failed to send message to room {room_id}: {_e:?}");
                            enqueue_popup_notification(format!("Failed to send message: {_e}"));
                        }
                    }
                    // }
                    broadcast_event(UIUpdateMessage::RefreshUI)
                        .expect("Couldn't broadcast event to UI");
                });
            }

            MatrixRequest::ReadReceipt { room_id, event_id } => {
                let timeline = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found when sending read receipt, room {room_id}, {event_id}");
                        continue;
                    };
                    room_info.timeline.clone()
                };
                let _send_rr_task = Handle::current().spawn(async move {
                    match timeline.send_single_receipt(ReceiptType::Read, event_id.clone()).await {
                        Ok(sent) => println!("{} read receipt to room {room_id} for event {event_id}", if sent { "Sent" } else { "Already sent" }),
                        Err(_e) => eprintln!("Failed to send read receipt to room {room_id} for event {event_id}; error: {_e:?}"),
                    }
                    // Also update the number of unread messages in the room.
                    enqueue_rooms_list_update(RoomsListUpdate::UpdateNumUnreadMessages {
                        room_id: room_id.clone(),
                        count: UnreadMessageCount::Known(timeline.room().num_unread_messages()),
                        unread_mentions: timeline.room().num_unread_mentions()
                    });
                });
            }

            MatrixRequest::FullyReadReceipt {
                room_id, event_id, ..
            } => {
                let timeline = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found when sending fully read receipt, room {room_id}, {event_id}");
                        continue;
                    };
                    room_info.timeline.clone()
                };
                let _send_frr_task = Handle::current().spawn(async move {
                    match timeline.send_single_receipt(ReceiptType::FullyRead, event_id.clone()).await {
                        Ok(sent) => println!("{} fully read receipt to room {room_id} for event {event_id}",
                            if sent { "Sent" } else { "Already sent" }
                        ),
                        Err(_e) => eprintln!("Failed to send fully read receipt to room {room_id} for event {event_id}; error: {_e:?}"),
                    }
                    // Also update the number of unread messages in the room.
                    enqueue_rooms_list_update(RoomsListUpdate::UpdateNumUnreadMessages {
                        room_id: room_id.clone(),
                        count: UnreadMessageCount::Known(timeline.room().num_unread_messages()),
                        unread_mentions: timeline.room().num_unread_mentions()
                    });
                });
            }

            MatrixRequest::GetRoomPowerLevels { room_id } => {
                let (timeline, sender) = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found for fetch members request {room_id}");
                        continue;
                    };

                    (
                        room_info.timeline.clone(),
                        room_info.timeline_update_sender.clone(),
                    )
                };

                let Some(user_id) = current_user_id() else {
                    continue;
                };

                let _power_levels_task = Handle::current().spawn(async move {
                    match timeline.room().power_levels().await {
                        Ok(power_levels) => {
                            println!("Successfully fetched power levels for room {room_id}.");
                            if let Err(e) = sender.send(TimelineUpdate::UserPowerLevels(
                                UserPowerLevels::from(&power_levels, &user_id),
                            )) {
                                eprintln!(
                                    "Failed to send the result of if user can send message: {e}"
                                )
                            }
                            broadcast_event(UIUpdateMessage::RefreshUI)
                                .expect("Couldn't broadcast event to UI");
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch power levels for room {room_id}: {e:?}");
                        }
                    }
                });
            }
            MatrixRequest::ToggleReaction {
                room_id,
                timeline_event_id,
                reaction,
            } => {
                let timeline = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found for send toggle reaction {room_id}");
                        continue;
                    };
                    room_info.timeline.clone()
                };

                let _toggle_reaction_task = Handle::current().spawn(async move {
                    println!("Toggle Reaction to room {room_id}: ...");
                    match timeline.toggle_reaction(&timeline_event_id, &reaction).await {
                        Ok(_send_handle) => {
                            broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                            println!("Sent toggle reaction to room {room_id} {reaction}.")
                        },
                        Err(_e) => eprintln!("Failed to send toggle reaction to room {room_id} {reaction}; error: {_e:?}"),
                    }
                });
            }
            MatrixRequest::RedactMessage {
                room_id,
                timeline_event_id,
                reason,
            } => {
                let timeline = {
                    let all_joined_rooms = ALL_JOINED_ROOMS.lock().unwrap();
                    let Some(room_info) = all_joined_rooms.get(&room_id) else {
                        println!("BUG: room info not found for redact message {room_id}");
                        continue;
                    };
                    room_info.timeline.clone()
                };

                let _redact_task = Handle::current().spawn(async move {
                    match timeline.redact(&timeline_event_id, reason.as_deref()).await {
                        Ok(()) => println!("Successfully redacted message in room {room_id}."),
                        Err(e) => {
                            eprintln!("Failed to redact message in {room_id}; error: {e:?}");
                            enqueue_popup_notification(format!(
                                "Failed to redact message. Error: {e}"
                            ));
                        }
                    }
                });
            }
            MatrixRequest::GetMatrixRoomLinkPillInfo { matrix_id, via } => {
                let Some(client) = CLIENT.get() else { continue };
                let _fetch_matrix_link_pill_info_task = Handle::current().spawn(async move {
                    let room_or_alias_id: Option<&RoomOrAliasId> = match &matrix_id {
                        MatrixId::Room(room_id) => Some((&**room_id).into()),
                        MatrixId::RoomAlias(room_alias_id) => Some((&**room_alias_id).into()),
                        MatrixId::Event(room_or_alias_id, _event_id) => Some(room_or_alias_id),
                        _ => {
                            println!("MatrixLinkRoomPillInfoRequest: Unsupported MatrixId type: {matrix_id:?}");
                            return;
                        }
                    };
                    if let Some(room_or_alias_id) = room_or_alias_id {
                        match client.get_room_preview(room_or_alias_id, via).await {
                            Ok(_preview) => {},
                            // Cx::post_action(MatrixLinkPillState::Loaded {
                            //     matrix_id: matrix_id.clone(),
                            //     name: preview.name.unwrap_or_else(|| room_or_alias_id.to_string()),
                            //     avatar_url: preview.avatar_url
                            // }),
                            Err(_e) => {
                                println!("Failed to get room link pill info for {room_or_alias_id:?}: {_e:?}");
                            }
                        };
                    }
                });
            }
            _ => bail!("Not implemented yet"),
        }
    }

    eprintln!("async_worker task ended unexpectedly");
    bail!("async_worker task ended unexpectedly")
}

/// Worker that loops to update rooms_list updates in queue
/// currently it handles active_room updates outside the other actions,
/// but maybe I should handle this as every other action
pub async fn ui_worker<R: Runtime>(app_handle: AppHandle<R>) -> anyhow::Result<()> {
    let rooms_list = Arc::new(Mutex::new(RoomsList::new()));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // create UI subscriber
    let mut ui_subscriber = debounce_broadcast(
        super::singletons::subscribe_to_events().expect("Couldn't get UI subscriber event"),
        Duration::from_millis(200),
    );

    // Event listener sends to channel instead of directly accessing rooms_list
    let listener_tx = tx.clone();
    app_handle.listen(
        MatrixSvelteListenEvent::MatrixUpdateCurrentActiveRoom.as_str(),
        move |event| {
            println!(
                "Received event to change current active room ! Body: {:?}",
                event.payload()
            );
            let tx_clone = listener_tx.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(payload) =
                    serde_json::from_str::<MatrixUpdateCurrentActiveRoom>(&event.payload())
                {
                    println!("payload: {payload:?}");
                    let (room_id, room_name) = if let Some(id) = payload.room_id {
                        let test = Some(OwnedRoomId::try_from(id).unwrap());
                        println!("test: {test:?}");
                        (test, payload.room_name)
                    } else {
                        (None, None)
                    };
                    println!("room id: {room_id:?}");
                    println!("room_name: {room_name:?}");
                    tx_clone.send((room_id, room_name)).unwrap();
                }
            });
        },
    );

    loop {
        tokio::select! {
            // Handle incoming events from listener
            Some((room_id, room_name)) = rx.recv() => {
                let mut lock = rooms_list.lock().await;
                lock.handle_current_active_room(&app_handle, room_id, room_name)
                    .expect("Couldn't set the room screen");
            }

            // Listen to UI refresh events
            _ = ui_subscriber.recv() => {
                let mut lock = rooms_list.lock().await;
                lock.handle_rooms_list_updates(&app_handle).await;

                process_user_profile_updates(&app_handle).await; // Each time the UI is refreshed we check the profiles update queue.
            }
        }
    }
}
