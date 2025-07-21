use anyhow::anyhow;
use matrix_sdk::{media::MediaRequestParameters, Client};
use session::{restore_client_from_session, try_get_session};
use tauri::{ipc::Channel, AppHandle, Manager, Runtime};
use tokio::sync::oneshot;

#[cfg(mobile)]
use crate::matrix::notifications::register_mobile_push_notifications;
use crate::{
    matrix::{notifications::register_desktop_notifications, requests::MatrixRequest},
    models::matrix::MediaStreamEvent,
};

pub mod emoji_verification;
pub mod event_preview;
pub mod events;
pub mod invited_room;
pub mod login;
pub mod notifications;
pub mod requests;
pub mod room;
pub mod rooms;
pub mod session;
pub mod singletons;
pub mod stores;
pub mod sync;
pub mod timeline;
pub mod user_power_level;
pub mod user_profile;
pub mod utils;
pub mod workers;

pub async fn create_session_to_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    request: login::LoginRequest,
) -> crate::Result<Client> {
    let snapshot_path = app_handle
        .state::<crate::stronghold::SnapshotPath>()
        .0
        .clone();

    let initial_client =
        login::get_client_from_new_session(&app_handle, request, &snapshot_path).await?;
    let client_with_handlers = events::add_event_handlers(initial_client, &app_handle)?;
    #[cfg(mobile)]
    register_mobile_push_notifications(&app_handle, &client_with_handlers).await;
    #[cfg(desktop)]
    register_desktop_notifications(&app_handle, &client_with_handlers).await;
    Ok(client_with_handlers)
}

pub async fn try_restore_session_to_state<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> crate::Result<Option<Client>> {
    let snapshot_path = app_handle
        .state::<crate::stronghold::SnapshotPath>()
        .0
        .clone();

    let session_option = try_get_session(&app_handle, snapshot_path).await?;

    match session_option {
        Some(session) => {
            let initial_client = restore_client_from_session(session).await?;
            let client_with_handlers = events::add_event_handlers(initial_client, &app_handle)?;
            #[cfg(mobile)]
            register_mobile_push_notifications(&app_handle, &client_with_handlers).await;
            #[cfg(desktop)]
            register_desktop_notifications(&app_handle, &client_with_handlers).await;
            Ok(Some(client_with_handlers))
        } // TODO : handle restore errors
        None => Ok(None),
    }
}

pub(crate) async fn fetch_media(
    media_request: MediaRequestParameters,
    on_event: &Channel<MediaStreamEvent>,
) -> anyhow::Result<usize> {
    let (tx, rx) = oneshot::channel();
    crate::matrix::requests::submit_async_request(MatrixRequest::FetchMedia {
        media_request,
        content_sender: tx,
    });

    let image_data: Vec<u8> = match rx.await {
        Ok(data) => match data {
            Ok(data) => data,
            Err(e) => return Err(anyhow!("Failed to fetch image: {}", e)),
        },
        Err(e) => return Err(anyhow!("Media receiver failed: {}", e)),
    };

    // Stream the image in chunks of 8KB
    const CHUNK_SIZE: usize = 8192;
    let mut bytes_sent = 0;

    for chunk in image_data.chunks(CHUNK_SIZE) {
        bytes_sent += chunk.len();

        if let Err(e) = on_event.send(MediaStreamEvent::Chunk {
            data: chunk.to_vec(),
            chunk_size: chunk.len(),
            bytes_received: bytes_sent,
        }) {
            return Err(anyhow!("Failed to send media chunk: {}", e));
        }
    }

    Ok(bytes_sent)
}
