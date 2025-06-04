use matrix_sdk::Client;
use session::{restore_client_from_session, try_get_session};
use tauri::{AppHandle, Manager, Runtime};

pub mod emoji_verification;
pub mod event_preview;
pub mod events;
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
            Ok(Some(client_with_handlers))
        } // TODO : handle restore errors
        None => Ok(None),
    }
}
