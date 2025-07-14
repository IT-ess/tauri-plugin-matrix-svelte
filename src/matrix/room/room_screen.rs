use std::{collections::BTreeMap, sync::Arc};

use matrix_sdk::{
    ruma::{OwnedEventId, OwnedRoomId, OwnedUserId},
    RoomMemberships,
};
use matrix_sdk_ui::{
    eyeball_im::Vector,
    timeline::{EventTimelineItem, TimelineItem},
};
use rangemap::RangeSet;
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_svelte::{ManagerExt, StoreState};

use crate::matrix::{
    requests::{submit_async_request, MatrixRequest},
    timeline::{
        take_timeline_endpoints, PaginationDirection, TimelineUiState, TimelineUpdate,
        TIMELINE_STATES,
    },
    user_power_level::UserPowerLevels,
    utils::room_name_or_id,
};

/// The main widget that displays a single Matrix room.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomScreen {
    /// The room ID of the currently-shown room.
    room_id: OwnedRoomId,
    /// The display name of the currently-shown room.
    room_name: String,
    /// The persistent UI-relevant states for the room that this widget is currently displaying.
    tl_state: Option<TimelineUiState>,
    /// Known members of this room
    members: BTreeMap<OwnedUserId, FrontendRoomMember>,
}
impl Drop for RoomScreen {
    fn drop(&mut self) {
        // This ensures that the `TimelineUiState` instance owned by this room is *always* returned
        // back to to `TIMELINE_STATES`, which ensures that its UI state(s) are not lost
        // and that other RoomScreen instances can show this room in the future.
        // RoomScreen will be dropped whenever its widget instance is destroyed, e.g.,
        // when a Tab is closed or the app is resized to a different AdaptiveView layout.
        self.hide_timeline();
    }
}

impl RoomScreen {
    pub fn new(room_id: OwnedRoomId, room_name: String) -> Self {
        Self {
            room_id,
            room_name,
            tl_state: None,
            members: BTreeMap::new(),
        }
    }

    /// Processes all pending background updates to the currently-shown timeline.
    pub fn process_timeline_updates<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        // let top_space = self.view(id!(top_space));
        // let jump_to_bottom = self.jump_to_bottom_button(id!(jump_to_bottom));
        // let curr_first_id = portal_list.first_id();
        let curr_first_id: usize = 0; // TODO: replace this dummy value

        // let ui = self.widget_uid();
        let Some(tl) = self.tl_state.as_mut() else {
            return;
        };

        let mut done_loading = false;
        let mut should_continue_backwards_pagination = false;
        let mut num_updates = 0;
        let mut typing_users = Vec::new();
        while let Ok(update) = tl.update_receiver.try_recv() {
            num_updates += 1;
            match update {
                TimelineUpdate::FirstUpdate { initial_items } => {
                    tl.content_drawn_since_last_update.clear();
                    tl.profile_drawn_since_last_update.clear();
                    tl.fully_paginated = false;
                    // Set the portal list to the very bottom of the timeline.
                    // portal_list.set_first_id_and_scroll(initial_items.len().saturating_sub(1), 0.0);
                    // portal_list.set_tail_range(true);
                    // jump_to_bottom.update_visibility(cx, true);

                    // update
                    tl.items = initial_items;
                    done_loading = true;
                }
                TimelineUpdate::NewItems {
                    new_items,
                    changed_indices,
                    is_append: _,
                    clear_cache,
                } => {
                    if new_items.is_empty() {
                        if !tl.items.is_empty() {
                            println!("Timeline::handle_event(): timeline (had {} items) was cleared for room {}", tl.items.len(), tl.room_id);
                            // For now, we paginate a cleared timeline in order to be able to show something at least.
                            // A proper solution would be what's described below, which would be to save a few event IDs
                            // and then either focus on them (if we're not close to the end of the timeline)
                            // or paginate backwards until we find them (only if we are close the end of the timeline).
                            should_continue_backwards_pagination = true;
                        }

                        // If the bottom of the timeline (the last event) is visible, then we should
                        // set the timeline to live mode.
                        // If the bottom of the timeline is *not* visible, then we should
                        // set the timeline to Focused mode.

                        // TODO: Save the event IDs of the top 3 items before we apply this update,
                        //       which indicates this timeline is in the process of being restored,
                        //       such that we can jump back to that position later after applying this update.

                        // TODO: here we need to re-build the timeline via TimelineBuilder
                        //       and set the TimelineFocus to one of the above-saved event IDs.

                        // TODO: the docs for `TimelineBuilder::with_focus()` claim that the timeline's focus mode
                        //       can be changed after creation, but I do not see any methods to actually do that.
                        //       <https://matrix-org.github.io/matrix-rust-sdk/matrix_sdk_ui/timeline/struct.TimelineBuilder.html#method.with_focus>
                        //
                        //       As such, we probably need to create a new async request enum variant
                        //       that tells the background async task to build a new timeline
                        //       (either in live mode or focused mode around one or more events)
                        //       and then replaces the existing timeline in ALL_ROOMS_INFO with the new one.
                    }

                    // Maybe todo?: we can often avoid the following loops that iterate over the `items` list
                    //       by only doing that if `clear_cache` is true, or if `changed_indices` range includes
                    //       any index that comes before (is less than) the above `curr_first_id`.

                    if new_items.len() == tl.items.len() {
                        // println!("Timeline::handle_event(): no jump necessary for updated timeline of same length: {}", items.len());
                    } else if curr_first_id > new_items.len() {
                        println!("Timeline::handle_event(): jumping to bottom: curr_first_id {} is out of bounds for {} new items", curr_first_id, new_items.len());
                        // portal_list.set_first_id_and_scroll(new_items.len().saturating_sub(1), 0.0);
                        // portal_list.set_tail_range(true);
                        // jump_to_bottom.update_visibility(cx, true);
                    } else if let Some((curr_item_idx, new_item_idx, new_item_scroll, _event_id)) =
                        find_new_item_matching_current_item(
                            0,
                            Some(0.0), // TODO replace
                            curr_first_id,
                            &tl.items,
                            &new_items,
                        )
                    {
                        if curr_item_idx != new_item_idx {
                            println!("Timeline::handle_event(): jumping view from event index {curr_item_idx} to new index {new_item_idx}, scroll {new_item_scroll}, event ID {_event_id}");
                            // portal_list.set_first_id_and_scroll(new_item_idx, new_item_scroll);
                            tl.prev_first_index = Some(new_item_idx);
                            // Set scrolled_past_read_marker false when we jump to a new event
                            tl.scrolled_past_read_marker = false;
                            // When the tooltip is up, the timeline may jump. This may take away the hover out event to required to clear the tooltip
                            // cx.widget_action(
                            //     ui,
                            //     &Scope::empty().path,
                            //     RoomScreenTooltipActions::HoverOut,
                            // );
                            // notify frontend ?
                        }
                    }
                    //
                    // TODO: after an (un)ignore user event, all timelines are cleared. Handle that here.
                    //
                    else {
                        // eprintln!("!!! Couldn't find new event with matching ID for ANY event currently visible in the portal list");
                    }

                    // If new items were appended to the end of the timeline, show an unread messages badge on the jump to bottom button.
                    // if is_append && !portal_list.is_at_end() {
                    //     if let Some(room_id) = &self.room_id {
                    //         // Immediately show the unread badge with no count while we fetch the actual count in the background.
                    //         jump_to_bottom
                    //             .show_unread_message_badge(cx, UnreadMessageCount::Unknown);
                    //         submit_async_request(MatrixRequest::GetNumberUnreadMessages {
                    //             room_id: room_id.clone(),
                    //         });
                    //     }
                    // }

                    if clear_cache {
                        tl.content_drawn_since_last_update.clear();
                        tl.profile_drawn_since_last_update.clear();
                        tl.fully_paginated = false;

                        // If this RoomScreen is showing the loading pane and has an ongoing backwards pagination request,
                        // then we should update the status message in that loading pane
                        // and then continue paginating backwards until we find the target event.
                        // Note that we do this here because `clear_cache` will always be true if backwards pagination occurred.
                        // let loading_pane = self.view.loading_pane(id!(loading_pane));
                        // let mut loading_pane_state = loading_pane.take_state();
                        // if let LoadingPaneState::BackwardsPaginateUntilEvent {
                        //     ref mut events_paginated,
                        //     target_event_id,
                        //     ..
                        // } = &mut loading_pane_state
                        // {
                        //     *events_paginated += new_items.len().saturating_sub(tl.items.len());
                        //     println!("While finding target event {target_event_id}, loaded {events_paginated} messages...");
                        //     // Here, we assume that we have not yet found the target event,
                        //     // so we need to continue paginating backwards.
                        //     // If the target event has already been found, it will be handled
                        //     // in the `TargetEventFound` match arm below, which will set
                        //     // `should_continue_backwards_pagination` to `false`.
                        //     // So either way, it's okay to set this to `true` here.
                        //     should_continue_backwards_pagination = true;
                        // }
                        // loading_pane.set_state(cx, loading_pane_state);
                    } else {
                        tl.content_drawn_since_last_update
                            .remove(changed_indices.clone());
                        tl.profile_drawn_since_last_update
                            .remove(changed_indices.clone());
                        // println!("Timeline::handle_event(): changed_indices: {changed_indices:?}, items len: {}\ncontent drawn: {:#?}\nprofile drawn: {:#?}", items.len(), tl.content_drawn_since_last_update, tl.profile_drawn_since_last_update);
                    }
                    tl.items = new_items;
                    done_loading = true;
                }
                TimelineUpdate::NewUnreadMessagesCount(_unread_messages_count) => {
                    // jump_to_bottom.show_unread_message_badge(unread_messages_count);
                }
                TimelineUpdate::TargetEventFound {
                    target_event_id,
                    index,
                } => {
                    // println!("Target event found in room {}: {target_event_id}, index: {index}", tl.room_id);
                    tl.request_sender.send_if_modified(|requests| {
                        requests.retain(|r| r.room_id != tl.room_id);
                        // no need to notify/wake-up all receivers for a completed request
                        false
                    });

                    // sanity check: ensure the target event is in the timeline at the given `index`.
                    let item = tl.items.get(index);
                    let is_valid = item.is_some_and(|item| {
                        item.as_event()
                            .is_some_and(|ev| ev.event_id() == Some(&target_event_id))
                    });
                    // let loading_pane = self.view.loading_pane(id!(loading_pane));

                    // println!("TargetEventFound: is_valid? {is_valid}. room {}, event {target_event_id}, index {index} of {}\n  --> item: {item:?}", tl.room_id, tl.items.len());
                    if is_valid {
                        // We successfully found the target event, so we can close the loading pane,
                        // reset the loading panestate to `None`, and stop issuing backwards pagination requests.
                        // loading_pane.set_status(cx, "Successfully found replied-to message!");
                        // loading_pane.set_state(cx, LoadingPaneState::None);

                        // NOTE: this code was copied from the `MessageAction::JumpToRelated` handler;
                        //       we should deduplicate them at some point.
                        // let speed = 50.0;
                        // Scroll to the message right above the replied-to message.
                        // FIXME: `smooth_scroll_to` should accept a scroll offset parameter too,
                        //       so that we can scroll to the replied-to message and have it
                        //       appear beneath the top of the viewport.
                        // portal_list.smooth_scroll_to(cx, index.saturating_sub(1), speed, None);
                        // // start highlight animation.
                        // tl.message_highlight_animation_state =
                        //     MessageHighlightAnimationState::Pending { item_id: index };
                    } else {
                        // Here, the target event was not found in the current timeline,
                        // or we found it previously but it is no longer in the timeline (or has moved),
                        // which means we encountered an error and are unable to jump to the target event.
                        eprintln!(
                            "Target event index {index} of {} is out of bounds for room {}",
                            tl.items.len(),
                            tl.room_id
                        );
                        // Show this error in the loading pane, which should already be open.
                        // loading_pane.set_state(LoadingPaneState::Error(String::from(
                        //     "Unable to find related message; it may have been deleted.",
                        // )));
                    }

                    should_continue_backwards_pagination = false;

                    // redraw now before any other items get added to the timeline list.
                    // self.view.redraw(cx);
                }
                TimelineUpdate::PaginationRunning(direction) => {
                    if direction == PaginationDirection::Backwards {
                        // top_space.set_visible(cx, true);
                        done_loading = false;
                    } else {
                        eprintln!("Unexpected PaginationRunning update in the Forwards direction");
                    }
                }
                TimelineUpdate::PaginationError { error, direction } => {
                    eprintln!(
                        "Pagination error ({direction}) in room {}: {error:?}",
                        tl.room_id
                    );
                    done_loading = true;
                }
                TimelineUpdate::PaginationIdle {
                    fully_paginated,
                    direction,
                } => {
                    if direction == PaginationDirection::Backwards {
                        // Don't set `done_loading` to `true`` here, because we want to keep the top space visible
                        // (with the "loading" message) until the corresponding `NewItems` update is received.
                        tl.fully_paginated = fully_paginated;
                        if fully_paginated {
                            done_loading = true;
                        }
                    } else {
                        eprintln!("Unexpected PaginationIdle update in the Forwards direction");
                    }
                }
                TimelineUpdate::EventDetailsFetched { event_id, result } => {
                    if let Err(_e) = result {
                        eprintln!("Failed to fetch details fetched for event {event_id} in room {}. Error: {_e:?}", tl.room_id);
                    }
                    // Here, to be most efficient, we could redraw only the updated event,
                    // but for now we just fall through and let the final `redraw()` call re-draw the whole timeline view.
                }
                TimelineUpdate::RoomMembersSynced => {
                    // println!("Timeline::handle_event(): room members fetched for room {}", tl.room_id);
                    // Here, to be most efficient, we could redraw only the user avatars and names in the timeline,
                    // but for now we just fall through and let the final `redraw()` call re-draw the whole timeline view.
                }
                TimelineUpdate::RoomMembersListFetched { members } => {
                    println!("RoomMembers list fetched !");
                    members.iter().for_each(|member| {
                        self.members.insert(
                            member.user_id().to_owned(),
                            FrontendRoomMember {
                                name: member.name().to_string(),
                                display_name_ambiguous: member.name_ambiguous(),
                                is_ignored: member.is_ignored(),
                                max_power_level: member.normalized_power_level(),
                            },
                        );
                    });
                    println!("{:?}", self.members);
                }
                TimelineUpdate::MediaFetched => {
                    println!(
                        "Timeline::handle_event(): media fetched for room {}",
                        tl.room_id
                    );
                    // Here, to be most efficient, we could redraw only the media items in the timeline,
                    // but for now we just fall through and let the final `redraw()` call re-draw the whole timeline view.
                }
                TimelineUpdate::MessageEdited {
                    timeline_event_id: _,
                    result: _,
                } => {
                    // self.view
                    //     .editing_pane(id!(editing_pane))
                    //     .handle_edit_result(cx, timeline_event_id, result);
                }
                TimelineUpdate::TypingUsers { users } => {
                    // This update loop should be kept tight & fast, so all we do here is
                    // save the list of typing users for future use after the loop exits.
                    // Then, we "process" it later (by turning it into a string) after the
                    // update loop has completed, which avoids unnecessary expensive work
                    // if the list of typing users gets updated many times in a row.
                    typing_users = users;
                }

                TimelineUpdate::UserPowerLevels(user_power_level) => {
                    tl.user_power = user_power_level;

                    // Update the visibility of the message input bar based on the new power levels.
                    let _can_send_message = user_power_level.can_send_message();
                    // self.view
                    //     .view(id!(input_bar))
                    //     .set_visible(cx, can_send_message);
                    // self.view
                    //     .view(id!(can_not_send_message_notice))
                    //     .set_visible(cx, !can_send_message);
                }

                TimelineUpdate::OwnUserReadReceipt(receipt) => {
                    tl.latest_own_user_receipt = Some(receipt);
                }
            }
        }

        if should_continue_backwards_pagination {
            submit_async_request(MatrixRequest::PaginateRoomTimeline {
                room_id: tl.room_id.clone(),
                num_events: 50,
                direction: PaginationDirection::Backwards,
            });
        }

        if done_loading {
            // top_space.set_visible(cx, false);
        }

        if !typing_users.is_empty() {
            let _typing_notice_text = match typing_users.as_slice() {
                [] => String::new(),
                [user] => format!("{user} is typing "),
                [user1, user2] => format!("{user1} and {user2} are typing "),
                [user1, user2, others @ ..] => {
                    if others.len() > 1 {
                        format!("{user1}, {user2}, and {} are typing ", &others[0])
                    } else {
                        format!("{user1}, {user2}, and {} others are typing ", others.len())
                    }
                }
            };
            // Set the typing notice text and make its view visible.
            // self.view
            //     .label(id!(typing_label))
            //     .set_text(cx, &typing_notice_text);
            // self.view.view(id!(typing_notice)).set_visible(cx, true);
            // // Animate in the typing notice view (sliding it up from the bottom).
            // self.animator_play(cx, id!(typing_notice_animator.show));
            // // Start the typing notice text animation of bouncing dots.
            // self.view
            //     .typing_animation(id!(typing_animation))
            //     .start_animation(cx);
        } else {
            // Animate out the typing notice view (sliding it out towards the bottom).
            // self.animator_play(cx, id!(typing_notice_animator.hide));
            // self.view
            //     .typing_animation(id!(typing_animation))
            //     .stop_animation(cx);
        }

        if num_updates > 0 {
            // println!("Applied {} timeline updates for room {}, redrawing with {} items...", num_updates, tl.room_id, tl.items.len());
            self.patch_frontend_store_with_current_state(&app_handle);
        }
    }

    fn patch_frontend_store_with_current_state<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        let mut state = StoreState::new();
        let json_room_id = serde_json::to_value(&self.room_id).expect("Couldn't serialize room_id");
        state.set("roomId", json_room_id);
        let json_room_name =
            serde_json::to_value(&self.room_name).expect("Couldn't serialize room_name");
        state.set("roomName", json_room_name);
        let json_tl_state =
            serde_json::to_value(&self.tl_state).expect("Couldn't serialize tl_state");
        state.set("tlState", json_tl_state);
        let json_tl_state =
            serde_json::to_value(&self.members).expect("Couldn't serialize members");
        state.set("members", json_tl_state);

        app_handle
            .svelte()
            .patch(&self.room_id, state)
            .expect("Couldn't patch the frontend state");
    }

    /// Handles any [`MessageAction`]s received by this RoomScreen.
    /// TODO: add an action queue fed by the frontend and treated here
    // fn handle_message_actions(&mut self, loading_pane: &LoadingPaneRef) {
    // ...
    // }

    // /// Shows the user profile sliding pane with the given avatar info.
    // fn show_user_profile(
    //     ...
    // }

    /// Shows the editing pane to allow the user to edit the given event.
    // fn show_editing_pane(
    //     &mut self,
    //     cx: &mut Cx,
    //     event_tl_item: EventTimelineItem,
    //     room_id: OwnedRoomId,
    // ) {
    //     ...
    // }

    /// Handles the EditingPane in this RoomScreen being fully hidden.
    // fn on_hide_editing_pane(&mut self) {
    //     // In `show_editing_pane()` above, we hid the input_bar while the editing pane
    //     // is being shown, so here we need to make it visible again.
    //     ...
    // }

    /// Shows a preview of the given event that the user is currently replying to
    /// above the message input bar.
    // fn show_replying_to(&mut self, cx: &mut Cx, replying_to: (EventTimelineItem, RepliedToInfo)) {
    //    ...
    // }

    /// Clears (and makes invisible) the preview of the message
    // /// that the user is currently replying to.
    // fn clear_replying_to(&mut self, cx: &mut Cx) {
    //     ...
    // }

    // fn show_location_preview(&mut self, cx: &mut Cx) {
    //     ...
    // }

    /// Invoke this when this timeline is being shown,
    /// e.g., when the user navigates to this timeline.
    pub fn show_timeline<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        let room_id = self.room_id.clone();
        // just an optional sanity check
        assert!(
            self.tl_state.is_none(),
            "BUG: tried to show_timeline() into a timeline with existing state. \
            Did you forget to save the timeline state back to the global map of states?",
        );

        // Obtain the current user's power levels for this room.
        submit_async_request(MatrixRequest::GetRoomPowerLevels {
            room_id: room_id.clone(),
        });

        let state_opt = TIMELINE_STATES.lock().unwrap().remove(&room_id);
        let (mut tl_state, first_time_showing_room) = if let Some(existing) = state_opt {
            (existing, false)
        } else {
            let (_update_sender, update_receiver, request_sender) =
                take_timeline_endpoints(&room_id)
                    .expect("BUG: couldn't get timeline state for first-viewed room.");
            let new_tl_state = TimelineUiState {
                room_id: room_id.clone(),
                // We assume the user has all power levels by default, just to avoid
                // unexpectedly hiding any UI elements that should be visible to the user.
                // This doesn't mean that the user can actually perform all actions.
                user_power: UserPowerLevels::all(),
                // We assume timelines being viewed for the first time haven't been fully paginated.
                fully_paginated: false,
                items: Vector::new(),
                content_drawn_since_last_update: RangeSet::new(),
                profile_drawn_since_last_update: RangeSet::new(),
                update_receiver,
                request_sender,
                // replying_to: None,
                saved_state: SavedState::default(),
                last_scrolled_index: usize::MAX,
                prev_first_index: None,
                scrolled_past_read_marker: false,
                latest_own_user_receipt: None,
            };
            (new_tl_state, true)
        };

        // Subscribe to typing notices, but hide the typing notice view initially.
        // self.view(id!(typing_notice)).set_visible(cx, false);
        submit_async_request(MatrixRequest::SubscribeToTypingNotices {
            room_id: room_id.clone(),
            subscribe: true,
        });

        submit_async_request(MatrixRequest::SubscribeToOwnUserReadReceiptsChanged {
            room_id: room_id.clone(),
            subscribe: true,
        });
        // Kick off a back pagination request for this room. This is "urgent",
        // because we want to show the user some messages as soon as possible
        // when they first open the room, and there might not be any messages yet.
        if first_time_showing_room && !tl_state.fully_paginated {
            println!(
                "Sending a first-time backwards pagination request for room {}",
                room_id
            );
            submit_async_request(MatrixRequest::PaginateRoomTimeline {
                room_id: room_id.clone(),
                num_events: 50,
                direction: PaginationDirection::Backwards,
            });
        }

        // This fetches the room members of the displayed timeline.
        submit_async_request(MatrixRequest::SyncRoomMemberList {
            room_id: room_id.clone(),
        });

        // Now, restore the visual state of this timeline from its previously-saved state.
        self.restore_state(&mut tl_state);

        // As the final step, store the tl_state for this room into this RoomScreen widget,
        // such that it can be accessed in future event/draw handlers.
        self.tl_state = Some(tl_state);

        // Now that we have restored the TimelineUiState into this RoomScreen widget,
        // we can proceed to processing pending background updates, and if any were processed,
        // the timeline will also be redrawn.
        if first_time_showing_room {
            // let portal_list = self.portal_list(id!(list));
            self.process_timeline_updates(app_handle);
        }

        self.patch_frontend_store_with_current_state(app_handle);
    }

    /// Invoke this when this RoomScreen/timeline is being hidden or no longer being shown.
    fn hide_timeline(&mut self) {
        self.save_state();

        // When closing a room view, we do the following with non-persistent states:
        // * Unsubscribe from typing notices, since we don't care about them
        //   when a given room isn't visible.
        // * Clear the location preview. We don't save this to the TimelineUiState
        //   because the location might change by the next time the user opens this same room.
        // self.location_preview(id!(location_preview)).clear();
        submit_async_request(MatrixRequest::SubscribeToTypingNotices {
            room_id: self.room_id.clone(),
            subscribe: false,
        });
        submit_async_request(MatrixRequest::SubscribeToOwnUserReadReceiptsChanged {
            room_id: self.room_id.clone(),
            subscribe: false,
        });
    }

    /// Removes the current room's visual UI state from this widget
    /// and saves it to the map of `TIMELINE_STATES` such that it can be restored later.
    ///
    /// Note: after calling this function, the widget's `tl_state` will be `None`.
    fn save_state(&mut self) {
        let Some(mut tl) = self.tl_state.take() else {
            eprintln!(
                "Timeline::save_state(): skipping due to missing state, room {:?}",
                self.room_id
            );
            return;
        };

        // let portal_list = self.portal_list(id!(list));
        // let message_input_box = self.text_input(id!(input_bar.message_input.text_input));
        // let editing_event = self
        //     .editing_pane(id!(editing_pane))
        //     .get_event_being_edited();
        let state = SavedState {
            first_index_and_scroll: None,
            editing_event: None,
            // first_index_and_scroll: Some((portal_list.first_id(), portal_list.scroll_position())),
            // message_input_state: message_input_box.save_state(),
            // replying_to: tl.replying_to.clone(),
            // editing_event,
        };
        println!(
            "Saving TimelineUiState for room {}: {:?}",
            tl.room_id, state
        );
        tl.saved_state = state;
        // Store this Timeline's `TimelineUiState` in the global map of states.
        TIMELINE_STATES
            .lock()
            .unwrap()
            .insert(tl.room_id.clone(), tl);
    }

    /// Restores the previously-saved visual UI state of this room.
    ///
    /// Note: this accepts a direct reference to the timeline's UI state,
    /// so this function must not try to re-obtain it by accessing `self.tl_state`.
    fn restore_state(&mut self, tl_state: &mut TimelineUiState) {
        let SavedState {
            first_index_and_scroll: _,
            // message_input_state,
            // replying_to,
            editing_event: _,
        } = &mut tl_state.saved_state;
        // 1. Restore the position of the timeline.
        // if let Some((first_index, scroll_from_first_id)) = first_index_and_scroll {
        //     self.portal_list(id!(timeline.list))
        //         .set_first_id_and_scroll(*first_index, *scroll_from_first_id);
        // } else {
        //     // If the first index is not set, then the timeline has not yet been scrolled by the user,
        //     // so we set the portal list to "tail" (track) the bottom of the list.
        //     self.portal_list(id!(timeline.list)).set_tail_range(true);
        // }

        // 2. Restore the state of the message input box.
        // let saved_message_input_state = std::mem::take(message_input_state);
        // self.text_input(id!(input_bar.message_input.text_input))
        //     .restore_state(cx, saved_message_input_state);

        // 3. Restore the state of the replying-to preview.
        // if let Some(replying_to_event) = replying_to.take() {
        //     self.show_replying_to(cx, replying_to_event);
        // } else {
        //     self.clear_replying_to(cx);
        // }

        // 4. Restore the state of the editing pane.
        // if let Some(editing_event) = editing_event.take() {
        //     self.show_editing_pane(cx, editing_event, tl_state.room_id.clone());
        // } else {
        //     self.editing_pane(id!(editing_pane)).force_hide(cx);
        //     self.on_hide_editing_pane(cx);
        // }
    }

    /// Sets this `RoomScreen` widget to display the timeline for the given room.
    pub fn set_displayed_room<S: Into<Option<String>>, R: Runtime>(
        &mut self,
        room_id: OwnedRoomId,
        room_name: S,
        app_handle: &AppHandle<R>,
    ) {
        // If the room is already being displayed, then do nothing.
        // if self.room_id.as_ref().is_some_and(|id| id == &room_id) {
        //     return;
        // }

        self.hide_timeline();
        // Reset the the state of the inner loading pane.
        // self.loading_pane(id!(loading_pane)).take_state();
        self.room_name = room_name_or_id(room_name.into(), &room_id);
        self.room_id = room_id.clone();

        // Clear any mention input state
        // let input_bar = self.view.room_input_bar(id!(input_bar));
        // let message_input = input_bar.mentionable_text_input(id!(message_input));
        // message_input.set_room_id(room_id);

        self.show_timeline(app_handle);
    }

    /// Sends read receipts based on the current scroll position of the timeline.
    fn _send_user_read_receipts_based_on_scroll_pos(
        &mut self,
        _scrolled: bool,
        _first_id: usize,
        _visible_items: usize,
    ) {
        // TODO: leave this to frontend
    }

    /// Sends a backwards pagination request if the user is scrolling up
    /// and is approaching the top of the timeline.
    fn _send_pagination_request_based_on_scroll_pos(&mut self, _scrolled: bool, _first_id: usize) {
        // TODO: leave this to frontend
    }
}

/// States that are necessary to save in order to maintain a consistent UI display for a timeline.
///
/// These are saved when navigating away from a timeline (upon `Hide`)
/// and restored when navigating back to a timeline (upon `Show`).
#[derive(Default, Debug)]
pub struct SavedState {
    /// The index of the first item in the timeline's PortalList that is currently visible,
    /// and the scroll offset from the top of the list's viewport to the beginning of that item.
    /// If this is `None`, then the timeline has not yet been scrolled by the user
    /// and the portal list will be set to "tail" (track) the bottom of the list.
    first_index_and_scroll: Option<(usize, f64)>,
    /// The content of the message input box.
    // message_input_state: TextInputState, // TODO: replace this by a string ?
    /// The event that the user is currently replying to, if any.
    // replying_to: Option<(EventTimelineItem, RepliedToInfo)>, // TODO: adapt with new type from sdk
    /// The event that the user is currently editing, if any.
    editing_event: Option<EventTimelineItem>,
}

/// Returns info about the item in the list of `new_items` that matches the event ID
/// of a visible item in the given `curr_items` list.
///
/// This info includes a tuple of:
/// 1. the index of the item in the current items list,
/// 2. the index of the item in the new items list,
/// 3. the positional "scroll" offset of the corresponding current item in the portal list,
/// 4. the unique event ID of the item.
fn find_new_item_matching_current_item(
    visible_items: usize,          // DUMMY PARAM TODO CHANGE THIS
    position_of_item: Option<f64>, // DUMMY PARAM TODO CHANGE THIS
    starting_at_curr_idx: usize,
    curr_items: &Vector<Arc<TimelineItem>>,
    new_items: &Vector<Arc<TimelineItem>>,
) -> Option<(usize, usize, f64, OwnedEventId)> {
    let mut curr_item_focus = curr_items.focus();
    let mut idx_curr = starting_at_curr_idx;
    let mut curr_items_with_ids: Vec<(usize, OwnedEventId)> = Vec::with_capacity(visible_items);

    // Find all items with real event IDs that are currently visible in the portal list.
    // TODO: if this is slow, we could limit it to 3-5 events at the most.
    if curr_items_with_ids.len() <= visible_items {
        while let Some(curr_item) = curr_item_focus.get(idx_curr) {
            if let Some(event_id) = curr_item.as_event().and_then(|ev| ev.event_id()) {
                curr_items_with_ids.push((idx_curr, event_id.to_owned()));
            }
            if curr_items_with_ids.len() >= visible_items {
                break;
            }
            idx_curr += 1;
        }
    }

    // Find a new item that has the same real event ID as any of the current items.
    for (idx_new, new_item) in new_items.iter().enumerate() {
        let Some(event_id) = new_item.as_event().and_then(|ev| ev.event_id()) else {
            continue;
        };
        if let Some((idx_curr, _)) = curr_items_with_ids
            .iter()
            .find(|(_, ev_id)| ev_id == event_id)
        {
            // Not all items in the portal list are guaranteed to have a position offset,
            // some may be zeroed-out, so we need to account for that possibility by only
            // using events that have a real non-zero area
            if let Some(pos_offset) = position_of_item {
                println!("Found matching event ID {event_id} at index {idx_new} in new items list, corresponding to current item index {idx_curr} at pos offset {pos_offset}");
                return Some((*idx_curr, idx_new, pos_offset, event_id.to_owned()));
            }
        }
    }

    None
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendRoomMember {
    name: String,
    max_power_level: i64,
    display_name_ambiguous: bool,
    is_ignored: bool,
}
