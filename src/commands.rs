use matrix_sdk::media::MediaRequestParameters;
use matrix_sdk::ruma::{OwnedDeviceId, OwnedRoomId, OwnedUserId};
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Runtime};

use crate::matrix::login::MatrixClientConfig;
use crate::matrix::requests::MatrixRequest;
use crate::models::matrix::{FrontendDevice, MediaStreamEvent};
use crate::notifications::WatchNotificationResult;
use crate::Result;
use crate::{Error, MatrixSvelteExt};

#[command]
pub(crate) async fn login_and_create_new_session<R: Runtime>(
    app: AppHandle<R>,
    config: MatrixClientConfig,
) -> Result<()> {
    // TODO: create error variants to display rich error messages to frontend
    app.matrix_svelte()
        .login_and_create_new_session(config)
        .await
}

#[command]
pub(crate) fn submit_async_request<R: Runtime>(
    app: AppHandle<R>,
    request: MatrixRequest,
) -> Result<()> {
    // TODO: create add some sender / receiver mechanism to display rich error
    // messages to frontend in case of error
    app.matrix_svelte().submit_async_request(request)
}

#[tauri::command]
pub(crate) async fn fetch_media<R: Runtime>(
    app: AppHandle<R>,
    media_request: MediaRequestParameters,
    on_event: Channel<MediaStreamEvent>,
) -> Result<()> {
    on_event.send(MediaStreamEvent::Started).unwrap();

    match app
        .matrix_svelte()
        .fetch_media(media_request, &on_event)
        .await
    {
        Ok(total_bytes) => {
            on_event
                .send(MediaStreamEvent::Finished { total_bytes })
                .unwrap();
            Ok(())
        }
        Err(e) => {
            on_event
                .send(MediaStreamEvent::Error {
                    message: e.to_string(),
                })
                .unwrap();
            Err(Error::Anyhow(e))
        }
    }
}

#[command]
pub(crate) async fn fetch_user_profile<R: Runtime>(
    app: AppHandle<R>,
    user_id: OwnedUserId,
    room_id: Option<OwnedRoomId>,
) -> Result<bool> {
    app.matrix_svelte()
        .fetch_user_profile(user_id, room_id)
        .await
}

#[command]
pub(crate) async fn get_devices<R: Runtime>(
    app_handle: AppHandle<R>,
    user_id: OwnedUserId,
) -> Result<Vec<FrontendDevice>> {
    app_handle.matrix_svelte().get_devices(user_id).await
}

#[command]
pub(crate) async fn verify_device<R: Runtime>(
    app_handle: AppHandle<R>,
    user_id: OwnedUserId,
    device_id: OwnedDeviceId,
) -> Result<()> {
    app_handle
        .matrix_svelte()
        .verify_device(user_id, device_id)
        .await
}

// Mobile only

#[command]
pub(crate) async fn watch_notifications<R: Runtime>(
    app_handle: AppHandle<R>,
    channel: Channel,
) -> Result<WatchNotificationResult> {
    app_handle.matrix_svelte().watch_notifications(channel)
}
