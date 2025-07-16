use matrix_sdk::ruma::{
    MilliSecondsSinceUnixEpoch, OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, OwnedUserId,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct InvitedRoomInfo {
    /// The matrix ID of this room.
    pub room_id: OwnedRoomId,
    /// The displayable name of this room, if known.
    pub room_name: Option<String>,
    /// The canonical alias for this room, if any.
    pub canonical_alias: Option<OwnedRoomAliasId>,
    /// The alternative aliases for this room, if any.
    pub alt_aliases: Vec<OwnedRoomAliasId>,
    /// The avatar for this room: either an array of bytes holding the avatar image
    /// or a string holding the first Unicode character of the room name.
    pub room_avatar: Option<OwnedMxcUri>,
    /// Info about the user who invited us to this room, if available.
    pub inviter_info: Option<InviterInfo>,
    /// The timestamp and Html text content of the latest message in this room.
    pub latest: Option<(MilliSecondsSinceUnixEpoch, String)>,
    /// The state of this how this invite is being handled by the client backend
    /// and what should be shown in the UI.
    pub invite_state: InviteState,
    /// Whether this room is currently selected in the UI.
    pub is_selected: bool, // TODO: remove if not needed
    /// Whether this is an invite to a direct room.
    pub is_direct: bool,
}

/// Info about the user who invited us to a room.
#[derive(Clone, Serialize)]
pub struct InviterInfo {
    pub user_id: OwnedUserId,
    pub display_name: Option<String>,
    pub avatar: Option<OwnedMxcUri>,
}
impl std::fmt::Debug for InviterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InviterInfo")
            .field("user_id", &self.user_id)
            .field("display_name", &self.display_name)
            .field("avatar?", &self.avatar.is_some())
            .finish()
    }
}

/// The state of a pending invite.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum InviteState {
    /// Waiting for the user to accept or decline the invite.
    #[default]
    WaitingOnUserInput,
    /// Waiting for the server to respond to the user's "join room" action.
    WaitingForJoinResult,
    /// Waiting for the server to respond to the user's "leave room" action.
    WaitingForLeaveResult,
    /// The invite was accepted and the room was successfully joined.
    /// We're now waiting for our client to receive the joined room from the homeserver.
    WaitingForJoinedRoom,
    /// The invite was declined and the room was successfully left.
    /// This should result in the InviteScreen being closed.
    RoomLeft,
}
