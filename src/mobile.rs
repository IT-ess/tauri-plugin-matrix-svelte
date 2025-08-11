use anyhow::anyhow;
use matrix_ui_serializable::{
    MatrixClientConfig, MediaRequestParameters, OwnedDeviceId, OwnedRoomId, OwnedUserId,
    device::FrontendDevice, models::requests::MediaStreamEvent, requests::MatrixRequest,
};
use serde::de::DeserializeOwned;
use tauri::{
    AppHandle, Manager, Runtime,
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
};

use crate::{
    models::mobile::{GetTokenRequest, GetTokenResponse, WatchNotificationResult},
    stronghold::{SnapshotPath, StrongholdCollection, utils::BytesDto},
    utils::fs::get_app_dir_or_create_it,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_matrix_svelte);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<MatrixSvelte<R>> {
    #[cfg(target_os = "android")]
    let handle =
        api.register_android_plugin("com.plugin.matrix.svelte", "PushNotificationPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_matrix_svelte)?;
    Ok(MatrixSvelte(handle))
}

/// Access to the matrix-svelte APIs.
pub struct MatrixSvelte<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MatrixSvelte<R> {
    pub async fn login_and_create_new_session(
        &self,
        config: MatrixClientConfig,
    ) -> crate::Result<()> {
        let app_data_dir = get_app_dir_or_create_it(&self.0.app())?;

        let snapshot_path = &self.0.app().state::<SnapshotPath>().0.clone();
        let collection_state = &self.0.app().state::<StrongholdCollection>();
        let client_key = BytesDto::Text("matrix_session".to_string());

        crate::stronghold::client::load_stronghold_client_or_create_it(
            collection_state.clone(),
            snapshot_path.clone(),
            client_key.clone(),
        )
        .await?;

        let session_string = matrix_ui_serializable::commands::login_and_create_new_session(
            config,
            None,
            app_data_dir,
        )
        .await?;

        crate::stronghold::store::save_store_record(
            collection_state.clone(),
            snapshot_path.clone(),
            client_key,
            "current".to_string(),
            session_string.into(),
            None,
        )
        .await?;

        crate::stronghold::client::save(collection_state.clone(), snapshot_path.clone()).await?;
        Ok(())
    }

    pub fn submit_async_request(&self, request: MatrixRequest) -> crate::Result<()> {
        matrix_ui_serializable::commands::submit_async_request(request)?;
        Ok(())
    }

    pub async fn fetch_media(
        &self,
        media_request: MediaRequestParameters,
        on_event: &Channel<MediaStreamEvent>,
    ) -> anyhow::Result<usize> {
        let (tx, rx) = matrix_ui_serializable::oneshot::channel();
        matrix_ui_serializable::commands::submit_async_request(MatrixRequest::FetchMedia {
            media_request,
            content_sender: tx,
        })?;

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

    pub async fn fetch_user_profile(
        &self,
        user_id: OwnedUserId,
        room_id: Option<OwnedRoomId>,
    ) -> crate::Result<bool> {
        matrix_ui_serializable::commands::fetch_user_profile(user_id, room_id)
            .await
            .map_err(|e| crate::Error::MatrixLib(e))
    }

    pub async fn get_devices(&self, user_id: OwnedUserId) -> crate::Result<Vec<FrontendDevice>> {
        matrix_ui_serializable::commands::get_devices(&user_id)
            .await
            .map_err(|e| crate::Error::MatrixLib(e))
    }

    pub async fn verify_device(
        &self,
        user_id: OwnedUserId,
        device_id: OwnedDeviceId,
    ) -> crate::Result<()> {
        matrix_ui_serializable::commands::verify_device(user_id, device_id)
            .await
            .map_err(|e| crate::Error::MatrixLib(e))
    }

    // Mobile only

    pub fn get_token(&self, payload: GetTokenRequest) -> crate::Result<GetTokenResponse> {
        self.0
            .run_mobile_plugin("getToken", payload)
            .map_err(Into::into)
    }

    pub fn watch_notifications(&self, channel: Channel) -> crate::Result<WatchNotificationResult> {
        #[cfg(target_os = "android")]
        return Err(crate::Error::Anyhow(anyhow!(
            "Not implemented on Android yet"
        )));
        #[cfg(target_os = "ios")]
        self.0
            .run_mobile_plugin("watchNotifications", WatchNotificationsArgs { channel })
            .map_err(Into::into)
    }
}
