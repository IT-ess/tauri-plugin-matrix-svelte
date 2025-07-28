use matrix_sdk::ruma::{MilliSecondsSinceUnixEpoch, OwnedDeviceId};
use serde::{Deserialize, Serialize};

// Listen to events
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MatrixSvelteListenEvent {
    RoomsCollectionStarted,
    RoomCreated,
    VerificationResult,
    MatrixUpdateCurrentActiveRoom,
}

impl MatrixSvelteListenEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatrixSvelteListenEvent::RoomsCollectionStarted => {
                "matrix-svelte://rooms-collection-started"
            }
            MatrixSvelteListenEvent::RoomCreated => "matrix-svelte://room-created",
            MatrixSvelteListenEvent::VerificationResult => "matrix-svelte://verification-result",
            MatrixSvelteListenEvent::MatrixUpdateCurrentActiveRoom => {
                "matrix-svelte://update-current-active-room"
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixVerificationResponse {
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixRoomStoreCreatedRequest {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixUpdateCurrentActiveRoom {
    // if the frontend sends null then it will be None
    pub room_id: Option<String>,
    pub room_name: Option<String>,
}

// Emit events

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MatrixSvelteEmitEvent {
    RoomCreate,
    VerificationStart,
    ToastNotification,
}

impl MatrixSvelteEmitEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatrixSvelteEmitEvent::RoomCreate => "matrix-svelte://room-create",
            MatrixSvelteEmitEvent::VerificationStart => "matrix-svelte://verification-start",
            MatrixSvelteEmitEvent::ToastNotification => "matrix-svelte://toast-notification",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixVerificationEmojis {
    emojis: String,
}

impl MatrixVerificationEmojis {
    pub fn new(emojis: String) -> Self {
        Self { emojis }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixRoomStoreCreateRequest {
    id: String,
}

impl MatrixRoomStoreCreateRequest {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToastNotificationRequest {
    message: String,
    description: Option<String>,
    variant: ToastNotificationVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ToastNotificationVariant {
    Default,
    Description,
    Success,
    Info,
    Warning,
    Error,
}

impl ToastNotificationRequest {
    pub fn new(
        message: String,
        description: Option<String>,
        variant: ToastNotificationVariant,
    ) -> Self {
        if description.is_some() {
            // If there is a description, force the description variant.
            Self {
                message,
                description,
                variant: ToastNotificationVariant::Description,
            }
        } else {
            Self {
                message,
                description: None,
                variant,
            }
        }
    }
}

// Channel events

#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum MediaStreamEvent {
    Started,
    Chunk {
        data: Vec<u8>,
        chunk_size: usize,
        bytes_received: usize,
    },
    Finished {
        total_bytes: usize,
    },
    Error {
        message: String,
    },
}

// Commands
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendDevice {
    pub device_id: OwnedDeviceId,
    pub is_verified: bool,
    pub is_verified_with_cross_signing: bool,
    pub display_name: Option<String>,
    pub registration_date: MilliSecondsSinceUnixEpoch,
    pub guessed_type: DeviceGuessedType,
    pub is_current_device: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DeviceGuessedType {
    Android,
    Ios,
    Web,
    Desktop,
    Unknown,
}
