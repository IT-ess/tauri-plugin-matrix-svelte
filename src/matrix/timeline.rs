use std::{
    cmp::{max, min},
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, Mutex},
};

use futures::StreamExt;
use matrix_sdk::{
    room::RoomMember,
    ruma::{
        events::{receipt::Receipt, FullStateEventContent},
        OwnedEventId, OwnedRoomId,
    },
    Room,
};
use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    timeline::{
        self, AnyOtherFullStateEventContent, EventTimelineItem, MembershipChange,
        TimelineEventItemId, TimelineItem, TimelineItemContent,
    },
    Timeline,
};
use rangemap::RangeSet;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use tokio::sync::watch;

use crate::matrix::singletons::{broadcast_event, UIUpdateMessage, LOG_TIMELINE_DIFFS};

use super::{
    events::get_latest_event_details,
    requests::{submit_async_request, MatrixRequest},
    room::{
        frontend_events::events_dto::{to_frontend_timeline_item, FrontendTimelineItem},
        room_screen::SavedState,
        rooms_list::{enqueue_rooms_list_update, RoomsListUpdate},
    },
    rooms::UnreadMessageCount,
    singletons::ALL_JOINED_ROOMS,
    user_power_level::UserPowerLevels,
    utils::current_user_id,
};

/// Which direction to paginate in.
///
/// * `Forwards` will retrieve later events (towards the end of the timeline),
///   which only works if the timeline is *focused* on a specific event.
/// * `Backwards`: the more typical choice, in which earlier events are retrieved
///   (towards the start of the timeline), which works in  both live mode and focused mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PaginationDirection {
    Forwards,
    Backwards,
}
impl std::fmt::Display for PaginationDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forwards => write!(f, "forwards"),
            Self::Backwards => write!(f, "backwards"),
        }
    }
}

/// A tokio::watch channel sender for sending requests from the RoomScreen UI widget
/// to the corresponding background async task for that room (its `timeline_subscriber_handler`).
pub type TimelineRequestSender = watch::Sender<Vec<BackwardsPaginateUntilEventRequest>>;

/// A request to search backwards for a specific event in a room's timeline.
#[derive(Debug)]
pub struct BackwardsPaginateUntilEventRequest {
    pub room_id: OwnedRoomId,
    pub target_event_id: OwnedEventId,
    /// The index in the timeline where a backwards search should begin.
    pub starting_index: usize,
    /// The number of items in the timeline at the time of the request,
    /// which is used to detect if the timeline has changed since the request was made,
    /// meaning that the `starting_index` can no longer be relied upon.
    pub current_tl_len: usize,
}

/// A message that is sent from a background async task to a room's timeline view
/// for the purpose of update the Timeline UI contents or metadata.
pub enum TimelineUpdate {
    /// The very first update a given room's timeline receives.
    FirstUpdate {
        /// The initial list of timeline items (events) for a room.
        initial_items: Vector<Arc<TimelineItem>>,
    },
    /// The content of a room's timeline was updated in the background.
    NewItems {
        /// The entire list of timeline items (events) for a room.
        new_items: Vector<Arc<TimelineItem>>,
        /// The range of indices in the `items` list that have been changed in this update
        /// and thus must be removed from any caches of drawn items in the timeline.
        /// Any items outside of this range are assumed to be unchanged and need not be redrawn.
        changed_indices: Range<usize>,
        /// An optimization that informs the UI whether the changes to the timeline
        /// resulted in new items being *appended to the end* of the timeline.
        is_append: bool,
        /// Whether to clear the entire cache of drawn items in the timeline.
        /// This supersedes `index_of_first_change` and is used when the entire timeline is being redrawn.
        clear_cache: bool,
    },
    /// The updated number of unread messages in the room.
    NewUnreadMessagesCount(UnreadMessageCount),
    /// The target event ID was found at the given `index` in the timeline items vector.
    ///
    /// This means that the RoomScreen widget can scroll the timeline up to this event,
    /// and the background `timeline_subscriber_handler` async task can stop looking for this event.
    TargetEventFound {
        target_event_id: OwnedEventId,
        index: usize,
    },
    /// A notice that the background task doing pagination for this room is currently running
    /// a pagination request in the given direction, and is waiting for that request to complete.
    PaginationRunning(PaginationDirection),
    /// An error occurred while paginating the timeline for this room.
    PaginationError {
        error: timeline::Error,
        direction: PaginationDirection,
    },
    /// A notice that the background task doing pagination for this room has become idle,
    /// meaning that it has completed its recent pagination request(s).
    PaginationIdle {
        /// If `true`, the start of the timeline has been reached, meaning that
        /// there is no need to send further pagination requests.
        fully_paginated: bool,
        direction: PaginationDirection,
    },
    /// A notice that event details have been fetched from the server,
    /// including a `result` that indicates whether the request was successful.
    EventDetailsFetched {
        event_id: OwnedEventId,
        result: Result<(), matrix_sdk_ui::timeline::Error>,
    },
    /// The result of a request to edit a message in this timeline.
    MessageEdited {
        timeline_event_id: TimelineEventItemId,
        result: Result<(), matrix_sdk_ui::timeline::Error>,
    },
    /// A notice that the room's members have been fetched from the server,
    /// though the success or failure of the request is not yet known until the client
    /// requests the member info via a timeline event's `sender_profile()` method.
    RoomMembersSynced,
    /// A notice that the room's full member list has been fetched from the server,
    /// includes a complete list of room members that can be shared across components.
    /// This is different from RoomMembersSynced which only indicates members were fetched
    /// but doesn't provide the actual data.
    RoomMembersListFetched { members: Vec<RoomMember> },
    /// A notice that one or more requested media items (images, videos, etc.)
    /// that should be displayed in this timeline have now been fetched and are available.
    MediaFetched,
    /// A notice that one or more members of a this room are currently typing.
    TypingUsers {
        /// The list of users (their displayable name) who are currently typing in this room.
        users: Vec<String>,
    },
    /// An update containing the currently logged-in user's power levels for this room.
    UserPowerLevels(UserPowerLevels),
    /// An update to the currently logged-in user's own read receipt for this room.
    OwnUserReadReceipt(Receipt),
}

/// The global set of all timeline states, one entry per room.
pub static TIMELINE_STATES: Mutex<BTreeMap<OwnedRoomId, TimelineUiState>> =
    Mutex::new(BTreeMap::new());

/// The UI-side state of a single room's timeline, which is only accessed/updated by the UI thread.
///
/// This struct should only include states that need to be persisted for a given room
/// across multiple `Hide`/`Show` cycles of that room's timeline within a RoomScreen.
/// If a state is more temporary and shouldn't be persisted when the timeline is hidden,
/// then it should be stored in the RoomScreen widget itself, not in this struct.
#[derive(Debug)]
pub struct TimelineUiState {
    /// The ID of the room that this timeline is for.
    pub(crate) room_id: OwnedRoomId,

    /// The power levels of the currently logged-in user in this room.
    pub(crate) user_power: UserPowerLevels,

    /// Whether this room's timeline has been fully paginated, which means
    /// that the oldest (first) event in the timeline is locally synced and available.
    /// When `true`, further backwards pagination requests will not be sent.
    ///
    /// This must be reset to `false` whenever the timeline is fully cleared.
    pub(crate) fully_paginated: bool,

    /// The list of items (events) in this room's timeline that our client currently knows about.
    pub(crate) items: Vector<Arc<TimelineItem>>,

    /// The range of items (indices in the above `items` list) whose event **contents** have been drawn
    /// since the last update and thus do not need to be re-populated on future draw events.
    ///
    /// This range is partially cleared on each background update (see below) to ensure that
    /// items modified during the update are properly redrawn. Thus, it is a conservative
    /// "cache tracker" that may not include all items that have already been drawn,
    /// but that's okay because big updates that clear out large parts of the rangeset
    /// only occur during back pagination, which is both rare and slow in and of itself.
    /// During typical usage, new events are appended to the end of the timeline,
    /// meaning that the range of already-drawn items doesn't need to be cleared.
    ///
    /// Upon a background update, only item indices greater than or equal to the
    /// `index_of_first_change` are removed from this set.
    /// Not included in frontend serialization
    pub(crate) content_drawn_since_last_update: RangeSet<usize>,

    /// Same as `content_drawn_since_last_update`, but for the event **profiles** (avatar, username).
    /// Not included in frontend serialization
    pub(crate) profile_drawn_since_last_update: RangeSet<usize>,

    /// The channel receiver for timeline updates for this room.
    ///
    /// Here we use a synchronous (non-async) channel because the receiver runs
    /// in a sync context and the sender runs in an async context,
    /// which is okay because a sender on an unbounded channel never needs to block.
    /// Not included in frontend serialization
    pub(crate) update_receiver: crossbeam_channel::Receiver<TimelineUpdate>,

    /// The sender for timeline requests from a RoomScreen showing this room
    /// to the background async task that handles this room's timeline updates.
    /// Not included in frontend serialization
    pub(crate) request_sender: TimelineRequestSender,

    /// Info about the event currently being replied to, if any.
    // TODO: replace repliedtoinfo struct with the latest one from the SDK (this one is broken)
    // replying_to: Option<(EventTimelineItem, RepliedToInfo)>,

    /// The states relevant to the UI display of this timeline that are saved upon
    /// a `Hide` action and restored upon a `Show` action.
    /// Not included in frontend serialization
    pub(crate) saved_state: SavedState,

    /// The state of the message highlight animation.
    ///
    /// We need to run the animation once the scrolling, triggered by the click of of a
    /// a reply preview, ends. so we keep a small state for it.
    /// By default, it starts in Off.
    /// Once the scrolling is started, the state becomes Pending.
    /// If the animation was triggered, the state goes back to Off.
    // TODO: remove this if I'm sure I don't need it
    // message_highlight_animation_state: MessageHighlightAnimationState,

    /// The index of the timeline item that was most recently scrolled up past it.
    /// This is used to detect when the user has scrolled up past the second visible item (index 1)
    /// upwards to the first visible item (index 0), which is the top of the timeline,
    /// at which point we submit a backwards pagination request to fetch more events.
    pub(crate) last_scrolled_index: usize,

    /// The index of the first item shown in the timeline's PortalList from *before* the last "jump".
    ///
    /// This index is saved before the timeline undergoes any jumps, e.g.,
    /// receiving new items, major scroll changes, or other timeline view jumps.
    pub(crate) prev_first_index: Option<usize>,

    /// Whether the user has scrolled past their latest read marker.
    ///
    /// This is used to determine whether we should send a fully-read receipt
    /// after the user scrolls past their "read marker", i.e., their latest fully-read receipt.
    /// Its value is determined by comparing the fully-read event's timestamp with the
    /// first and last timestamp of displayed events in the timeline.
    /// When scrolling down, if the value is true, we send a fully-read receipt
    /// for the last visible event in the timeline.
    ///
    /// When new message come in, this value is reset to `false`.
    pub(crate) scrolled_past_read_marker: bool,
    pub(crate) latest_own_user_receipt: Option<Receipt>,
}

impl Serialize for TimelineUiState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TimelineUiState", 8)?;

        state.serialize_field("roomId", &self.room_id)?;
        state.serialize_field("userPower", &self.user_power)?;
        state.serialize_field("fullyPaginated", &self.fully_paginated)?;
        state.serialize_field(
            "items",
            &serialize_timeline_items(&self.items, &self.room_id),
        )?;
        state.serialize_field("lastScrolledIndex", &self.last_scrolled_index)?;
        state.serialize_field("prevFirstIndex", &self.prev_first_index)?;
        state.serialize_field("scrolledPastReadMarker", &self.scrolled_past_read_marker)?;
        state.serialize_field("latestOwnUserReceipt", &self.latest_own_user_receipt)?;

        state.end()
    }
}
fn serialize_timeline_items<'a>(
    items: &'a Vector<Arc<TimelineItem>>,
    room_id: &OwnedRoomId,
) -> Vec<FrontendTimelineItem<'a>> {
    items
        .iter()
        .map(|item| to_frontend_timeline_item(item, Some(room_id)))
        .collect()
}

/// Returns three channel endpoints related to the timeline for the given joined room.
///
/// 1. A timeline update sender.
/// 2. The timeline update receiver, which is a singleton, and can only be taken once.
/// 3. A `tokio::watch` sender that can be used to send requests to the timeline subscriber handler.
///
/// This will only succeed once per room, as only a single channel receiver can exist.
pub fn take_timeline_endpoints(
    room_id: &OwnedRoomId,
) -> Option<(
    crossbeam_channel::Sender<TimelineUpdate>,
    crossbeam_channel::Receiver<TimelineUpdate>,
    TimelineRequestSender,
)> {
    ALL_JOINED_ROOMS
        .lock()
        .unwrap()
        .get_mut(room_id)
        .and_then(|ri| {
            ri.timeline_singleton_endpoints
                .take()
                .map(|(receiver, request_sender)| {
                    (ri.timeline_update_sender.clone(), receiver, request_sender)
                })
        })
}

/// A per-room async task that listens for timeline updates and sends them to the UI thread.
///
/// One instance of this async task is spawned for each room the client knows about.
pub async fn timeline_subscriber_handler(
    room: Room,
    timeline: Arc<Timeline>,
    timeline_update_sender: crossbeam_channel::Sender<TimelineUpdate>,
    mut request_receiver: watch::Receiver<Vec<BackwardsPaginateUntilEventRequest>>,
) {
    /// An inner function that searches the given new timeline items for a target event.
    ///
    /// If the target event is found, it is removed from the `target_event_id_opt` and returned,
    /// along with the index/position of that event in the given iterator of new items.
    fn find_target_event<'a>(
        target_event_id_opt: &mut Option<OwnedEventId>,
        mut new_items_iter: impl Iterator<Item = &'a Arc<TimelineItem>>,
    ) -> Option<(usize, OwnedEventId)> {
        let found_index = target_event_id_opt.as_ref().and_then(|target_event_id| {
            new_items_iter.position(|new_item| {
                new_item
                    .as_event()
                    .is_some_and(|new_ev| new_ev.event_id() == Some(target_event_id))
            })
        });

        if let Some(index) = found_index {
            target_event_id_opt.take().map(|ev| (index, ev))
        } else {
            None
        }
    }

    let room_id = room.room_id().to_owned();
    println!("Starting timeline subscriber for room {room_id}...");
    let (mut timeline_items, mut subscriber) = timeline.subscribe().await;
    println!(
        "Received initial timeline update of {} items for room {room_id}.",
        timeline_items.len()
    );

    timeline_update_sender.send(TimelineUpdate::FirstUpdate {
        initial_items: timeline_items.clone(),
    }).unwrap_or_else(
        |_e| panic!("Error: timeline update sender couldn't send first update ({} items) to room {room_id}!", timeline_items.len())
    );

    let mut latest_event = timeline.latest_event().await;

    // the event ID to search for while loading previous items into the timeline.
    let mut target_event_id = None;
    // the timeline index and event ID of the target event, if it has been found.
    let mut found_target_event_id: Option<(usize, OwnedEventId)> = None;

    loop {
        tokio::select! {
            // we must check for new requests before handling new timeline updates.
            biased;

            // Handle updates to the current backwards pagination requests.
            Ok(()) = request_receiver.changed() => {
                let prev_target_event_id = target_event_id.clone();
                let new_request_details = request_receiver
                    .borrow_and_update()
                    .iter()
                    .find_map(|req| req.room_id
                        .eq(&room_id)
                        .then(|| (req.target_event_id.clone(), req.starting_index, req.current_tl_len))
                    );

                target_event_id = new_request_details.as_ref().map(|(ev, ..)| ev.clone());

                // If we received a new request, start searching backwards for the target event.
                if let Some((new_target_event_id, starting_index, current_tl_len)) = new_request_details {
                    if prev_target_event_id.as_ref() != Some(&new_target_event_id) {
                        let starting_index = if current_tl_len == timeline_items.len() {
                            starting_index
                        } else {
                            // The timeline has changed since the request was made, so we can't rely on the `starting_index`.
                            // Instead, we have no choice but to start from the end of the timeline.
                            timeline_items.len()
                        };
                        // println!("Received new request to search for event {new_target_event_id} in room {room_id} starting from index {starting_index} (tl len {}).", timeline_items.len());
                        // Search backwards for the target event in the timeline, starting from the given index.
                        if let Some(target_event_tl_index) = timeline_items
                            .focus()
                            .narrow(..starting_index)
                            .into_iter()
                            .rev()
                            .position(|i| i.as_event()
                                .and_then(|e| e.event_id())
                                .is_some_and(|ev_id| ev_id == new_target_event_id)
                            )
                            .map(|i| starting_index.saturating_sub(i).saturating_sub(1))
                        {
                            // println!("Found existing target event {new_target_event_id} in room {room_id} at index {target_event_tl_index}.");

                            // Nice! We found the target event in the current timeline items,
                            // so there's no need to actually proceed with backwards pagination;
                            // thus, we can clear the locally-tracked target event ID.
                            target_event_id = None;
                            found_target_event_id = None;
                            timeline_update_sender.send(
                                TimelineUpdate::TargetEventFound {
                                    target_event_id: new_target_event_id.clone(),
                                    index: target_event_tl_index,
                                }
                            ).unwrap_or_else(
                                |_e| panic!("Error: timeline update sender couldn't send TargetEventFound({new_target_event_id}, {target_event_tl_index}) to room {room_id}!")
                            );
                            // Update this room's timeline UI view.
                            broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                        }
                        else {
                            // println!("Target event not in timeline. Starting backwards pagination in room {room_id} to find target event {new_target_event_id} starting from index {starting_index}.");

                            // If we didn't find the target event in the current timeline items,
                            // we need to start loading previous items into the timeline.
                            submit_async_request(MatrixRequest::PaginateRoomTimeline {
                                room_id: room_id.clone(),
                                num_events: 50,
                                direction: PaginationDirection::Backwards,
                            });
                        }
                    }
                }
            }

            // Handle updates to the actual timeline content.
            batch_opt = subscriber.next() => {
                let Some(batch) = batch_opt else { break };
                let mut num_updates = 0;
                // For now we always requery the latest event, but this can be better optimized.
                let mut reobtain_latest_event = true;
                let mut index_of_first_change = usize::MAX;
                let mut index_of_last_change = usize::MIN;
                // whether to clear the entire cache of drawn items
                let mut clear_cache = false;
                // whether the changes include items being appended to the end of the timeline
                let mut is_append = false;
                for diff in batch {
                    num_updates += 1;
                    match diff {
                        VectorDiff::Append { values } => {
                            let _values_len = values.len();
                            index_of_first_change = min(index_of_first_change, timeline_items.len());
                            timeline_items.extend(values);
                            index_of_last_change = max(index_of_last_change, timeline_items.len());
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Append {_values_len}. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                            is_append = true;
                        }
                        VectorDiff::Clear => {
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Clear"); }
                            clear_cache = true;
                            timeline_items.clear();
                            reobtain_latest_event = true;
                        }
                        VectorDiff::PushFront { value } => {
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff PushFront"); }
                            if let Some((index, _ev)) = found_target_event_id.as_mut() {
                                *index += 1; // account for this new `value` being prepended.
                            } else {
                                found_target_event_id = find_target_event(&mut target_event_id, std::iter::once(&value));
                            }

                            clear_cache = true;
                            timeline_items.push_front(value);
                            reobtain_latest_event |= latest_event.is_none();
                        }
                        VectorDiff::PushBack { value } => {
                            index_of_first_change = min(index_of_first_change, timeline_items.len());
                            timeline_items.push_back(value);
                            index_of_last_change = max(index_of_last_change, timeline_items.len());
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff PushBack. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                            is_append = true;
                        }
                        VectorDiff::PopFront => {
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff PopFront"); }
                            clear_cache = true;
                            timeline_items.pop_front();
                            if let Some((i, _ev)) = found_target_event_id.as_mut() {
                                *i = i.saturating_sub(1); // account for the first item being removed.
                            }
                            // This doesn't affect whether we should reobtain the latest event.
                        }
                        VectorDiff::PopBack => {
                            timeline_items.pop_back();
                            index_of_first_change = min(index_of_first_change, timeline_items.len());
                            index_of_last_change = usize::MAX;
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff PopBack. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                        }
                        VectorDiff::Insert { index, value } => {
                            if index == 0 {
                                clear_cache = true;
                            } else {
                                index_of_first_change = min(index_of_first_change, index);
                                index_of_last_change = usize::MAX;
                            }
                            if index >= timeline_items.len() {
                                is_append = true;
                            }

                            if let Some((i, _ev)) = found_target_event_id.as_mut() {
                                // account for this new `value` being inserted before the previously-found target event's index.
                                if index <= *i {
                                    *i += 1;
                                }
                            } else {
                                found_target_event_id = find_target_event(&mut target_event_id, std::iter::once(&value))
                                    .map(|(i, ev)| (i + index, ev));
                            }

                            timeline_items.insert(index, value);
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Insert at {index}. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                        }
                        VectorDiff::Set { index, value } => {
                            index_of_first_change = min(index_of_first_change, index);
                            index_of_last_change  = max(index_of_last_change, index.saturating_add(1));
                            timeline_items.set(index, value);
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Set at {index}. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                        }
                        VectorDiff::Remove { index } => {
                            if index == 0 {
                                clear_cache = true;
                            } else {
                                index_of_first_change = min(index_of_first_change, index.saturating_sub(1));
                                index_of_last_change = usize::MAX;
                            }
                            if let Some((i, _ev)) = found_target_event_id.as_mut() {
                                // account for an item being removed before the previously-found target event's index.
                                if index <= *i {
                                    *i = i.saturating_sub(1);
                                }
                            }
                            timeline_items.remove(index);
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Remove at {index}. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                        }
                        VectorDiff::Truncate { length } => {
                            if length == 0 {
                                clear_cache = true;
                            } else {
                                index_of_first_change = min(index_of_first_change, length.saturating_sub(1));
                                index_of_last_change = usize::MAX;
                            }
                            timeline_items.truncate(length);
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Truncate to length {length}. Changes: {index_of_first_change}..{index_of_last_change}"); }
                            reobtain_latest_event = true;
                        }
                        VectorDiff::Reset { values } => {
                            if LOG_TIMELINE_DIFFS { println!("timeline_subscriber: room {room_id} diff Reset, new length {}", values.len()); }
                            clear_cache = true; // we must assume all items have changed.
                            timeline_items = values;
                            reobtain_latest_event = true;
                        }
                    }
                }


                if num_updates > 0 {
                    let new_latest_event = if reobtain_latest_event {
                        timeline.latest_event().await
                    } else {
                        None
                    };

                    let changed_indices = index_of_first_change..index_of_last_change;

                    if LOG_TIMELINE_DIFFS {
                        println!("timeline_subscriber: applied {num_updates} updates for room {room_id}, timeline now has {} items. is_append? {is_append}, clear_cache? {clear_cache}. Changes: {changed_indices:?}.", timeline_items.len());
                    }
                    timeline_update_sender.send(TimelineUpdate::NewItems {
                        new_items: timeline_items.clone(),
                        changed_indices,
                        clear_cache,
                        is_append,
                    }).expect("Error: timeline update sender couldn't send update with new items!");

                    // We must send this update *after* the actual NewItems update,
                    // otherwise the UI thread (RoomScreen) won't be able to correctly locate the target event.
                    if let Some((index, found_event_id)) = found_target_event_id.take() {
                        target_event_id = None;
                        timeline_update_sender.send(
                            TimelineUpdate::TargetEventFound {
                                target_event_id: found_event_id.clone(),
                                index,
                            }
                        ).unwrap_or_else(
                            |_e| panic!("Error: timeline update sender couldn't send TargetEventFound({found_event_id}, {index}) to room {room_id}!")
                        );
                    }

                    // Update the latest event for this room.
                    // We always do this in case a redaction or other event has changed the latest event.
                    if let Some(new_latest) = new_latest_event {
                        let _room_avatar_changed = update_latest_event(room_id.clone(), &new_latest, Some(&timeline_update_sender));
                        // if room_avatar_changed {
                        //     spawn_fetch_room_avatar(room.clone());
                        // }
                        latest_event = Some(new_latest);
                    }

                    // Update this room's timeline UI view.
                    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
                }
            }

            else => {
                break;
            }
        }
    }

    eprintln!("Error: unexpectedly ended timeline subscriber for room {room_id}.");
}

/// Handles the given updated latest event for the given room.
///
/// This currently includes checking the given event for:
/// * room name changes, in which it sends a `RoomsListUpdate`.
/// * room power level changes to see if the current user's permissions
///   have changed; if so, it sends a [`TimelineUpdate::UserPowerLevels`].
/// * room avatar changes, which is not handled here.
///   Instead, we return `true` such that other code can fetch the new avatar.
/// * membership changes to see if the current user has joined or left a room.
///
/// Finally, this function sends a `RoomsListUpdate::UpdateLatestEvent`
/// to update the latest event in the RoomsList's room preview for the given room.
///
/// Returns `true` if room avatar has changed and should be fetched and updated.
pub fn update_latest_event(
    room_id: OwnedRoomId,
    event_tl_item: &EventTimelineItem,
    timeline_update_sender: Option<&crossbeam_channel::Sender<TimelineUpdate>>,
) -> bool {
    let mut room_avatar_changed = false;

    let (timestamp, latest_message_text) = get_latest_event_details(event_tl_item, &room_id);
    match event_tl_item.content() {
        // Check for relevant state events.
        TimelineItemContent::OtherState(other) => {
            match other.content() {
                // Check for room name changes.
                AnyOtherFullStateEventContent::RoomName(FullStateEventContent::Original {
                    content,
                    ..
                }) => {
                    enqueue_rooms_list_update(RoomsListUpdate::UpdateRoomName {
                        room_id: room_id.clone(),
                        new_room_name: matrix_sdk::RoomDisplayName::Named(content.name.clone()),
                    });
                }
                // Check for room avatar changes.
                AnyOtherFullStateEventContent::RoomAvatar(_avatar_event) => {
                    room_avatar_changed = true;
                }
                // Check for if can user send message.
                AnyOtherFullStateEventContent::RoomPowerLevels(
                    FullStateEventContent::Original {
                        content,
                        prev_content: _,
                    },
                ) => {
                    if let (Some(sender), Some(user_id)) =
                        (timeline_update_sender, current_user_id())
                    {
                        match sender.send(TimelineUpdate::UserPowerLevels(UserPowerLevels::from(
                            &content.clone().into(),
                            &user_id,
                        ))) {
                            Ok(_) => {
                                broadcast_event(UIUpdateMessage::RefreshUI)
                                    .expect("Couldn't broadcast event to UI");
                            }
                            Err(e) => {
                                eprintln!("Failed to send the new RoomPowerLevels from an updated latest event: {e}");
                            }
                        }
                    }
                }
                _ => {} // TODO: implement behaviour
            }
        }
        TimelineItemContent::MembershipChange(room_membership_change) => {
            if matches!(
                room_membership_change.change(),
                Some(MembershipChange::InvitationAccepted | MembershipChange::Joined)
            ) {
                if current_user_id().as_deref() == Some(room_membership_change.user_id()) {
                    submit_async_request(MatrixRequest::GetRoomPowerLevels {
                        room_id: room_id.clone(),
                    });
                }
            }
        }
        _ => {} // TODO: implement behaviour
    }

    enqueue_rooms_list_update(RoomsListUpdate::UpdateLatestEvent {
        room_id,
        timestamp,
        latest_message_text,
    });
    room_avatar_changed
}
