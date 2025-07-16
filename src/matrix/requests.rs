use matrix_sdk::{
    media::MediaRequestParameters,
    room::{edit::EditedContent, RoomMember},
    ruma::{
        events::room::message::RoomMessageEventContent, matrix_uri::MatrixId, OwnedEventId,
        OwnedRoomAliasId, OwnedRoomId, OwnedUserId,
    },
    OwnedServerName, RoomMemberships,
};
use matrix_sdk_ui::timeline::TimelineEventItemId;
use tokio::sync::oneshot;

use super::{singletons::REQUEST_SENDER, timeline::PaginationDirection};

/// Submits a request to the worker thread to be executed asynchronously.
pub fn submit_async_request(req: MatrixRequest) {
    REQUEST_SENDER
        .get()
        .unwrap() // this is initialized
        .send(req)
        .expect("BUG: async worker task receiver has died!");
}

/// The set of requests for async work that can be made to the worker thread.
#[allow(clippy::large_enum_variant)]
pub enum MatrixRequest {
    /// Request to paginate the older (or newer) events of a room's timeline.
    PaginateRoomTimeline {
        room_id: OwnedRoomId,
        /// The maximum number of timeline events to fetch in each pagination batch.
        num_events: u16,
        direction: PaginationDirection,
    },
    /// Request to edit the content of an event in the given room's timeline.
    EditMessage {
        room_id: OwnedRoomId,
        timeline_event_item_id: TimelineEventItemId,
        edited_content: EditedContent,
    },
    /// Request to fetch the full details of the given event in the given room's timeline.
    FetchDetailsForEvent {
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
    },
    /// Request to fetch profile information for all members of a room.
    /// This can be *very* slow depending on the number of members in the room.
    SyncRoomMemberList {
        room_id: OwnedRoomId,
    },
    /// Request to join the given room.
    JoinRoom {
        room_id: OwnedRoomId,
    },
    /// Request to leave the given room.
    LeaveRoom {
        room_id: OwnedRoomId,
    },
    /// Request to get the actual list of members in a room.
    /// This returns the list of members that can be displayed in the UI.
    GetRoomMembers {
        room_id: OwnedRoomId,
        memberships: RoomMemberships,
        /// * If `true` (not recommended), only the local cache will be accessed.
        /// * If `false` (recommended), details will be fetched from the server.
        local_only: bool,
    },
    /// Request to fetch profile information for the given user ID.
    GetUserProfile {
        user_id: OwnedUserId,
        /// * If `Some`, the user is known to be a member of a room, so this will
        ///   fetch the user's profile from that room's membership info.
        /// * If `None`, the user's profile info will be fetched from the server
        ///   in a room-agnostic manner, and no room membership info will be returned.
        room_id: Option<OwnedRoomId>,
        /// * If `true` (not recommended), only the local cache will be accessed.
        /// * If `false` (recommended), details will be fetched from the server.
        local_only: bool,
    },
    /// Request to fetch the number of unread messages in the given room.
    GetNumberUnreadMessages {
        room_id: OwnedRoomId,
    },
    /// Request to ignore/block or unignore/unblock a user.
    IgnoreUser {
        /// Whether to ignore (`true`) or unignore (`false`) the user.
        ignore: bool,
        /// The room membership info of the user to (un)ignore.
        room_member: RoomMember,
        /// The room ID of the room where the user is a member,
        /// which is only needed because it isn't present in the `RoomMember` object.
        room_id: OwnedRoomId,
    },
    /// Request to resolve a room alias into a room ID and the servers that know about that room.
    ResolveRoomAlias(OwnedRoomAliasId),
    /// Request to fetch an Avatar image from the server.
    /// Upon completion of the async media request, the `on_fetched` function
    /// will be invoked with the content of an `AvatarUpdate`.
    // FetchAvatar {
    //     mxc_uri: OwnedMxcUri,
    //     on_fetched: fn(AvatarUpdate),
    // },
    /// Request to fetch media from the server.
    /// Upon completion of the async media request, the `on_fetched` function
    /// will be invoked with four arguments: the `destination`, the `media_request`,
    /// the result of the media fetch, and the `update_sender`.
    FetchMedia {
        media_request: MediaRequestParameters,
        content_sender: oneshot::Sender<Result<Vec<u8>, matrix_sdk::Error>>,
    },
    /// Request to send a message to the given room.
    SendMessage {
        room_id: OwnedRoomId,
        message: RoomMessageEventContent,
        // replied_to: Option<RepliedToInfo>,
    },
    /// Sends a notice to the given room that the current user is or is not typing.
    ///
    /// This request does not return a response or notify the UI thread, and
    /// furthermore, there is no need to send a follow-up request to stop typing
    /// (though you certainly can do so).
    SendTypingNotice {
        room_id: OwnedRoomId,
        typing: bool,
    },
    /// Subscribe to typing notices for the given room.
    ///
    /// This request does not return a response or notify the UI thread.
    SubscribeToTypingNotices {
        room_id: OwnedRoomId,
        /// Whether to subscribe or unsubscribe from typing notices for this room.
        subscribe: bool,
    },
    /// Subscribe to changes in the read receipts of our own user.
    ///
    /// This request does not return a response or notify the UI thread.
    SubscribeToOwnUserReadReceiptsChanged {
        room_id: OwnedRoomId,
        /// Whether to subscribe or unsubscribe to changes in the read receipts of our own user for this room
        subscribe: bool,
    },
    /// Sends a read receipt for the given event in the given room.
    ReadReceipt {
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
    },
    /// Sends a fully-read receipt for the given event in the given room.
    FullyReadReceipt {
        room_id: OwnedRoomId,
        event_id: OwnedEventId,
    },
    /// Sends a request to obtain the power levels for this room.
    ///
    /// The response is delivered back to the main UI thread via [`TimelineUpdate::UserPowerLevels`].
    GetRoomPowerLevels {
        room_id: OwnedRoomId,
    },
    /// Toggles the given reaction to the given event in the given room.
    ToggleReaction {
        room_id: OwnedRoomId,
        timeline_event_id: TimelineEventItemId,
        reaction: String,
    },
    /// Redacts (deletes) the given event in the given room.
    #[doc(alias("delete"))]
    RedactMessage {
        room_id: OwnedRoomId,
        timeline_event_id: TimelineEventItemId,
        reason: Option<String>,
    },
    /// Sends a request to obtain the room's pill link info for the given Matrix ID.
    ///
    /// The MatrixLinkPillInfo::Loaded variant is sent back to the main UI thread via.
    GetMatrixRoomLinkPillInfo {
        matrix_id: MatrixId,
        via: Vec<OwnedServerName>,
    },
    CreateDMRoom {
        user_id: OwnedUserId,
    },
}
// Deserialize trait is implemented in models/matrix_requests.rs
