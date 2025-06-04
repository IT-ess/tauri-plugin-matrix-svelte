use std::sync::Arc;

use crossbeam_queue::SegQueue;
use matrix_sdk::{
    room::RoomMember,
    ruma::{OwnedMxcUri, OwnedRoomId, OwnedUserId},
};

use super::singletons::{broadcast_event, UIUpdateMessage};

/// The currently-known state of a user's avatar.
#[derive(Clone, Debug)]
#[allow(unused)]
pub enum AvatarState {
    /// It isn't yet known if this user has an avatar.
    Unknown,
    /// It is known that this user does or does not have an avatar.
    Known(Option<OwnedMxcUri>),
    /// This user does have an avatar, and it has been fetched successfully.
    Loaded(Arc<[u8]>),
    /// This user does have an avatar, but we failed to fetch it.
    Failed,
}
impl AvatarState {
    /// Returns the avatar data, if in the `Loaded` state.
    pub fn data(&self) -> Option<&Arc<[u8]>> {
        if let AvatarState::Loaded(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Returns the avatar URI, if in the `Known` state and it exists.
    pub fn uri(&self) -> Option<&OwnedMxcUri> {
        if let AvatarState::Known(Some(uri)) = self {
            Some(uri)
        } else {
            None
        }
    }
}

/// Information retrieved about a user: their displayable name, ID, and known avatar state.
#[derive(Clone, Debug)]
pub struct UserProfile {
    pub user_id: OwnedUserId,
    /// The user's default display name, if set.
    /// Note that a user may have per-room display names,
    /// so this should be considered a fallback.
    pub username: Option<String>,
    pub avatar_state: AvatarState,
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
