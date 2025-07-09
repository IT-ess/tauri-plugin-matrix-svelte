use anyhow::anyhow;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, HashMap},
    sync::LazyLock,
};

use crossbeam_queue::SegQueue;
use matrix_sdk::{
    room::RoomMember,
    ruma::{OwnedMxcUri, OwnedRoomId, OwnedUserId, RoomId},
};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};
use tauri_plugin_svelte::{ManagerExt, StoreState};
use tokio::sync::RwLock;

use crate::matrix::requests::{submit_async_request, MatrixRequest};

use super::singletons::{broadcast_event, UIUpdateMessage};

/// Information retrieved about a user: their displayable name, ID, and known avatar state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: OwnedUserId,
    /// The user's default display name, if set.
    /// Note that a user may have per-room display names,
    /// so this should be considered a fallback.
    pub username: Option<String>,
    pub avatar_url: Option<OwnedMxcUri>,
}
impl UserProfile {
    /// Returns the user's displayable name, using the user ID as a fallback.
    pub fn displayable_name(&self) -> &str {
        if let Some(un) = self.username.as_ref() {
            if !un.is_empty() {
                return un.as_str();
            }
        }
        self.user_id.as_str()
    }
}

/// The queue of user profile updates waiting to be processed by the UI thread's event handler.
static PENDING_USER_PROFILE_UPDATES: SegQueue<UserProfileUpdate> = SegQueue::new();

/// Enqueues a new user profile update and signals the UI that an update is available.
pub fn enqueue_user_profile_update(update: UserProfileUpdate) {
    PENDING_USER_PROFILE_UPDATES.push(update);
    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
}

/// A user profile update, which can include changes to a user's full profile
/// and/or room membership info.
pub enum UserProfileUpdate {
    /// A fully-fetched user profile, with info about the user's membership in a given room.
    Full {
        new_profile: UserProfile,
        room_id: OwnedRoomId,
        room_member: RoomMember,
    },
    /// An update to the user's room membership info only, without any profile changes.
    RoomMemberOnly {
        room_id: OwnedRoomId,
        room_member: RoomMember,
    },
    /// An update to the user's profile only, without changes to room membership info.
    UserProfileOnly(UserProfile),
}

impl UserProfileUpdate {
    /// Applies this update to the given user profile info cache.
    fn apply_to_cache(self, cache: &mut BTreeMap<OwnedUserId, UserProfileCacheEntry>) {
        match self {
            UserProfileUpdate::Full {
                new_profile,
                room_id,
                room_member: _,
            } => match cache.entry(new_profile.user_id.clone()) {
                Entry::Occupied(mut entry) => match entry.get_mut() {
                    e @ UserProfileCacheEntry::Requested => {
                        *e = UserProfileCacheEntry::Loaded {
                            user_profile: new_profile,
                            rooms: {
                                let mut rooms = BTreeSet::new();
                                rooms.insert(room_id);
                                rooms
                            },
                        };
                    }
                    UserProfileCacheEntry::Loaded {
                        user_profile,
                        rooms,
                    } => {
                        *user_profile = new_profile;
                        rooms.insert(room_id);
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(UserProfileCacheEntry::Loaded {
                        user_profile: new_profile,
                        rooms: {
                            let mut rooms = BTreeSet::new();
                            rooms.insert(room_id);
                            rooms
                        },
                    });
                }
            },
            UserProfileUpdate::RoomMemberOnly {
                room_id,
                room_member,
            } => {
                match cache.entry(room_member.user_id().to_owned()) {
                    Entry::Occupied(mut entry) => match entry.get_mut() {
                        e @ UserProfileCacheEntry::Requested => {
                            // This shouldn't happen, but we can still technically handle it correctly.
                            eprintln!("BUG: User profile cache entry was `Requested` for user {} when handling RoomMemberOnly update", room_member.user_id());
                            *e = UserProfileCacheEntry::Loaded {
                                user_profile: UserProfile {
                                    user_id: room_member.user_id().to_owned(),
                                    username: None,
                                    avatar_url: room_member.avatar_url().map(|url| url.to_owned()),
                                },
                                rooms: {
                                    let mut rooms = BTreeSet::new();
                                    rooms.insert(room_id);
                                    rooms
                                },
                            };
                        }
                        UserProfileCacheEntry::Loaded { rooms, .. } => {
                            rooms.insert(room_id);
                        }
                    },
                    Entry::Vacant(entry) => {
                        // This shouldn't happen, but we can still technically handle it correctly.
                        eprintln!("BUG: User profile cache entry not found for user {} when handling RoomMemberOnly update", room_member.user_id());
                        entry.insert(UserProfileCacheEntry::Loaded {
                            user_profile: UserProfile {
                                user_id: room_member.user_id().to_owned(),
                                username: None,
                                avatar_url: room_member.avatar_url().map(|url| url.to_owned()),
                            },
                            rooms: {
                                let mut rooms = BTreeSet::new();
                                rooms.insert(room_id);
                                rooms
                            },
                        });
                    }
                }
            }
            UserProfileUpdate::UserProfileOnly(new_profile) => {
                match cache.entry(new_profile.user_id.clone()) {
                    Entry::Occupied(mut entry) => match entry.get_mut() {
                        e @ UserProfileCacheEntry::Requested => {
                            *e = UserProfileCacheEntry::Loaded {
                                user_profile: new_profile,
                                rooms: BTreeSet::new(),
                            };
                        }
                        UserProfileCacheEntry::Loaded { user_profile, .. } => {
                            *user_profile = new_profile;
                        }
                    },
                    Entry::Vacant(entry) => {
                        entry.insert(UserProfileCacheEntry::Loaded {
                            user_profile: new_profile,
                            rooms: BTreeSet::new(),
                        });
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone, Serialize)]
struct UserProfileMap(BTreeMap<OwnedUserId, UserProfileCacheEntry>);

/// A cache of each user's profile and the rooms they are a member of, indexed by user ID.
static USER_PROFILE_CACHE: LazyLock<RwLock<UserProfileMap>> =
    LazyLock::new(|| RwLock::new(UserProfileMap(BTreeMap::new())));

static PROFILES_STORE_ID: &str = "profiles";

/// Processes all pending user profile updates in the queue.
pub async fn process_user_profile_updates<R: Runtime>(app_handle: &AppHandle<R>) -> bool {
    let mut updated = false;
    if PENDING_USER_PROFILE_UPDATES.is_empty() {
        return updated; // Return early if the queue is empty to avoid acquiring the lock.
    };
    {
        let mut lock = USER_PROFILE_CACHE.write().await;
        while let Some(update) = PENDING_USER_PROFILE_UPDATES.pop() {
            // Insert the updated info into the cache
            update.apply_to_cache(&mut lock.0);
            updated = true;
        }
    } // We drop the write lock here
    if updated {
        let lock = USER_PROFILE_CACHE.read().await;
        let json = serde_json::to_value(lock.0.clone()).expect("Couldn't serialize User Profiles");
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
                PROFILES_STORE_ID,
                state.expect("Wrong state sent to frontend"),
            )
            .expect("Couldn't patch the frontend state");
    }
    updated
}

/// Submit a request to retrieve the user profile or returns true if the entry is already requested.
pub async fn fetch_user_profile(user_id: OwnedUserId, room_id: Option<OwnedRoomId>) -> bool {
    let mut lock = USER_PROFILE_CACHE.write().await;
    match lock.0.entry(user_id) {
        Entry::Occupied(_) => true,
        Entry::Vacant(entry) => {
            submit_async_request(MatrixRequest::GetUserProfile {
                user_id: entry.key().clone(),
                room_id,
                local_only: false,
            });
            entry.insert(UserProfileCacheEntry::Requested);
            false
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "state",
    content = "data"
)]
enum UserProfileCacheEntry {
    /// A request has been issued and we're waiting for it to complete.
    Requested,
    /// The profile has been successfully loaded from the server.
    Loaded {
        #[serde(flatten)]
        user_profile: UserProfile,
        rooms: BTreeSet<OwnedRoomId>,
    },
}
