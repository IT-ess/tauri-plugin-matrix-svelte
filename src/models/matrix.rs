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
    MessageText,
}

impl MatrixSvelteEmitEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatrixSvelteEmitEvent::RoomCreate => "matrix-svelte://room-create",
            MatrixSvelteEmitEvent::VerificationStart => "matrix-svelte://verification-start",
            MatrixSvelteEmitEvent::MessageText => "matrix-svelte://message-text",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixMessage {
    sender: String,
    body: String,
}

impl MatrixMessage {
    pub fn new(sender: String, body: String) -> Self {
        Self { sender, body }
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
