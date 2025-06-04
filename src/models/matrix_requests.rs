use matrix_sdk::ruma::{
    events::room::message::RoomMessageEventContent, OwnedEventId, OwnedRoomId, OwnedUserId,
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::matrix::{requests::MatrixRequest, timeline::PaginationDirection};

impl<'de> Deserialize<'de> for MatrixRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize into a generic Value to inspect the structure
        let value = Value::deserialize(deserializer)?;

        // Extract the "event" field to determine the variant
        let event = value
            .get("event")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("event"))?;

        // Extract the "payload" field containing the variant data
        let payload = value
            .get("payload")
            .ok_or_else(|| serde::de::Error::missing_field("payload"))?;

        // Match on the event type and deserialize the appropriate variant
        match event {
            "paginateRoomTimeline" => {
                let data: PaginateRoomTimelinePayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::PaginateRoomTimeline {
                    room_id: data.room_id,
                    num_events: data.num_events,
                    direction: data.direction,
                })
            }
            // "editMessage" => {
            //     let data: EditMessagePayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::EditMessage {
            //         room_id: data.room_id,
            //         timeline_event_item_id: data.timeline_event_item_id,
            //         edited_content: data.edited_content,
            //     })
            // }
            "fetchDetailsForEvent" => {
                let data: FetchDetailsForEventPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::FetchDetailsForEvent {
                    room_id: data.room_id,
                    event_id: data.event_id,
                })
            }
            // "syncRoomMemberList" => {
            //     let data: SyncRoomMemberListPayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::SyncRoomMemberList {
            //         room_id: data.room_id,
            //     })
            // }
            "joinRoom" => {
                let data: JoinRoomPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::JoinRoom {
                    room_id: data.room_id,
                })
            }
            "leaveRoom" => {
                let data: LeaveRoomPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::LeaveRoom {
                    room_id: data.room_id,
                })
            }
            // "getRoomMembers" => {
            //     let data: GetRoomMembersPayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::GetRoomMembers {
            //         room_id: data.room_id,
            //         memberships: data.memberships,
            //         local_only: data.local_only,
            //     })
            // }
            "getUserProfile" => {
                let data: GetUserProfilePayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::GetUserProfile {
                    user_id: data.user_id,
                    room_id: data.room_id,
                    local_only: data.local_only,
                })
            }
            "getNumberUnreadMessages" => {
                let data: GetNumberUnreadMessagesPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::GetNumberUnreadMessages {
                    room_id: data.room_id,
                })
            }
            // "ignoreUser" => {
            //     let data: IgnoreUserPayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::IgnoreUser {
            //         ignore: data.ignore,
            //         room_member: data.room_member,
            //         room_id: data.room_id,
            //     })
            // }
            "resolveRoomAlias" => {
                let alias =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::ResolveRoomAlias(alias))
            }
            "sendMessage" => {
                let data: SendMessagePayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::SendMessage {
                    room_id: data.room_id,
                    message: data.message,
                })
            }
            "sendTypingNotice" => {
                let data: SendTypingNoticePayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::SendTypingNotice {
                    room_id: data.room_id,
                    typing: data.typing,
                })
            }
            "subscribeToTypingNotices" => {
                let data: SubscribeToTypingNoticesPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::SubscribeToTypingNotices {
                    room_id: data.room_id,
                    subscribe: data.subscribe,
                })
            }
            "subscribeToOwnUserReadReceiptsChanged" => {
                let data: SubscribeToOwnUserReadReceiptsChangedPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::SubscribeToOwnUserReadReceiptsChanged {
                    room_id: data.room_id,
                    subscribe: data.subscribe,
                })
            }
            "readReceipt" => {
                let data: ReadReceiptPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::ReadReceipt {
                    room_id: data.room_id,
                    event_id: data.event_id,
                })
            }
            "fullyReadReceipt" => {
                let data: FullyReadReceiptPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::FullyReadReceipt {
                    room_id: data.room_id,
                    event_id: data.event_id,
                })
            }
            "getRoomPowerLevels" => {
                let data: GetRoomPowerLevelsPayload =
                    serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
                Ok(MatrixRequest::GetRoomPowerLevels {
                    room_id: data.room_id,
                })
            }
            // "toggleReaction" => {
            //     let data: ToggleReactionPayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::ToggleReaction {
            //         room_id: data.room_id,
            //         timeline_event_id: data.timeline_event_id,
            //         reaction: data.reaction,
            //     })
            // }
            // "redactMessage" => {
            //     let data: RedactMessagePayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::RedactMessage {
            //         room_id: data.room_id,
            //         timeline_event_id: data.timeline_event_id,
            //         reason: data.reason,
            //     })
            // }
            // "getMatrixRoomLinkPillInfo" => {
            //     let data: GetMatrixRoomLinkPillInfoPayload =
            //         serde_json::from_value(payload.clone()).map_err(serde::de::Error::custom)?;
            //     Ok(MatrixRequest::GetMatrixRoomLinkPillInfo {
            //         matrix_id: data.matrix_id,
            //         via: data.via,
            //     })
            // }
            _ => Err(serde::de::Error::unknown_variant(
                event,
                &[
                    "paginateRoomTimeline",
                    // "editMessage",
                    "fetchDetailsForEvent",
                    // "syncRoomMemberList",
                    "joinRoom",
                    "leaveRoom",
                    // "getRoomMembers",
                    "getUserProfile",
                    "getNumberUnreadMessages",
                    // "ignoreUser",
                    "resolveRoomAlias",
                    "sendMessage",
                    "sendTypingNotice",
                    "subscribeToTypingNotices",
                    "subscribeToOwnUserReadReceiptsChanged",
                    "readReceipt",
                    "fullyReadReceipt",
                    "getRoomPowerLevels",
                    // "toggleReaction",
                    // "redactMessage",
                    // "getMatrixRoomLinkPillInfo",
                ],
            )),
        }
    }
}

// Helper structs for deserializing payloads
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaginateRoomTimelinePayload {
    room_id: OwnedRoomId,
    num_events: u16,
    direction: PaginationDirection,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct EditMessagePayload {
//     room_id: OwnedRoomId,
//     timeline_event_item_id: TimelineEventItemId,
//     edited_content: EditedContent,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchDetailsForEventPayload {
    room_id: OwnedRoomId,
    event_id: OwnedEventId,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct SyncRoomMemberListPayload {
//     room_id: OwnedRoomId,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinRoomPayload {
    room_id: OwnedRoomId,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LeaveRoomPayload {
    room_id: OwnedRoomId,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct GetRoomMembersPayload {
//     room_id: OwnedRoomId,
//     memberships: RoomMemberships,
//     local_only: bool,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetUserProfilePayload {
    user_id: OwnedUserId,
    room_id: Option<OwnedRoomId>,
    local_only: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetNumberUnreadMessagesPayload {
    room_id: OwnedRoomId,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct IgnoreUserPayload {
//     ignore: bool,
//     room_member: RoomMember,
//     room_id: OwnedRoomId,
// }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendMessagePayload {
    room_id: OwnedRoomId,
    message: RoomMessageEventContent,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendTypingNoticePayload {
    room_id: OwnedRoomId,
    typing: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeToTypingNoticesPayload {
    room_id: OwnedRoomId,
    subscribe: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeToOwnUserReadReceiptsChangedPayload {
    room_id: OwnedRoomId,
    subscribe: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadReceiptPayload {
    room_id: OwnedRoomId,
    event_id: OwnedEventId,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullyReadReceiptPayload {
    room_id: OwnedRoomId,
    event_id: OwnedEventId,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetRoomPowerLevelsPayload {
    room_id: OwnedRoomId,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct ToggleReactionPayload {
//     room_id: OwnedRoomId,
//     timeline_event_id: TimelineEventItemId,
//     reaction: String,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct RedactMessagePayload {
//     room_id: OwnedRoomId,
//     timeline_event_id: TimelineEventItemId,
//     reason: Option<String>,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct GetMatrixRoomLinkPillInfoPayload {
//     matrix_id: MatrixId,
//     via: Vec<OwnedServerName>,
// }
