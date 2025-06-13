use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use matrix_sdk::{ruma::OwnedRoomId, Client};
use matrix_sdk_ui::sync_service::SyncService;
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use super::{requests::MatrixRequest, rooms::JoinedRoomDetails};

/// The sender used by [`submit_async_request`] to send requests to the async worker thread.
/// Currently there is only one, but it can be cloned if we need more concurrent senders.
pub static REQUEST_SENDER: OnceLock<UnboundedSender<MatrixRequest>> = OnceLock::new();

/// The singleton sync service.
pub static SYNC_SERVICE: OnceLock<SyncService> = OnceLock::new();

pub fn get_sync_service() -> Option<&'static SyncService> {
    SYNC_SERVICE.get()
}

/// Information about all joined rooms that our client currently know about.
pub static ALL_JOINED_ROOMS: Mutex<BTreeMap<OwnedRoomId, JoinedRoomDetails>> =
    Mutex::new(BTreeMap::new());

pub static TOMBSTONED_ROOMS: Mutex<BTreeMap<OwnedRoomId, OwnedRoomId>> =
    Mutex::new(BTreeMap::new());

pub static LOG_ROOM_LIST_DIFFS: bool = true;

pub static LOG_TIMELINE_DIFFS: bool = true;

/// The logged-in Matrix client, which can be freely and cheaply cloned.
pub static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> Option<Client> {
    CLIENT.get().cloned()
}

pub static LOGIN_STORE_READY: OnceLock<bool> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum UIUpdateMessage {
    RefreshUI,
}

// Global broadcaster instance
static GLOBAL_BROADCASTER: OnceLock<GlobalBroadcaster> = OnceLock::new();

pub struct GlobalBroadcaster {
    sender: broadcast::Sender<UIUpdateMessage>,
}

pub static TEMP_DIR: OnceLock<PathBuf> = OnceLock::new();

impl GlobalBroadcaster {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    fn broadcast(
        &self,
        message: UIUpdateMessage,
    ) -> Result<usize, broadcast::error::SendError<UIUpdateMessage>> {
        self.sender.send(message)
    }

    fn subscribe(&self) -> broadcast::Receiver<UIUpdateMessage> {
        self.sender.subscribe()
    }
}

// Initialize the global broadcaster (call this once at startup)
pub fn init_broadcaster(capacity: usize) -> Result<(), &'static str> {
    GLOBAL_BROADCASTER
        .set(GlobalBroadcaster::new(capacity))
        .map_err(|_| "Broadcaster already initialized")
}

// Globally available function to broadcast messages
pub fn broadcast_event(message: UIUpdateMessage) -> Result<usize, Box<dyn std::error::Error>> {
    let broadcaster = GLOBAL_BROADCASTER
        .get()
        .ok_or("Broadcaster not initialized. Call init_broadcaster() first.")?;

    Ok(broadcaster.broadcast(message)?)
}

// Globally available function to create receivers
pub fn subscribe_to_events() -> Result<broadcast::Receiver<UIUpdateMessage>, &'static str> {
    let broadcaster = GLOBAL_BROADCASTER
        .get()
        .ok_or("Broadcaster not initialized. Call init_broadcaster() first.")?;

    Ok(broadcaster.subscribe())
}
