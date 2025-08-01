use anyhow::anyhow;
use matrix_sdk::{
    media::MediaRequestParameters,
    ruma::{OwnedDeviceId, OwnedRoomId, OwnedUserId},
};
use serde::de::DeserializeOwned;
use tauri::{
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{
    matrix::{
        create_session_to_state, get_devices,
        login::{LoginRequest, MatrixClientConfig},
        requests::{submit_async_request, MatrixRequest},
        user_profile::fetch_user_profile,
        verify_device,
    },
    models::matrix::{FrontendDevice, MediaStreamEvent},
};

use crate::models::notifications::{
    GetTokenRequest, GetTokenResponse, WatchNotificationResult, WatchNotificationsArgs,
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
        create_session_to_state(&self.0.app(), LoginRequest::LoginByPassword(config)).await?;
        Ok(())
    }

    pub fn submit_async_request(&self, request: MatrixRequest) -> crate::Result<()> {
        submit_async_request(request);
        Ok(())
    }

    pub async fn fetch_media(
        &self,
        media_request: MediaRequestParameters,
        on_event: &Channel<MediaStreamEvent>,
    ) -> anyhow::Result<usize> {
        crate::matrix::fetch_media(media_request, on_event).await
    }

    pub async fn fetch_user_profile(
        &self,
        user_id: OwnedUserId,
        room_id: Option<OwnedRoomId>,
    ) -> crate::Result<bool> {
        Ok(fetch_user_profile(user_id, room_id).await)
    }

    pub async fn get_devices(&self, user_id: OwnedUserId) -> crate::Result<Vec<FrontendDevice>> {
        get_devices(&user_id).await
    }

    pub async fn verify_device(
        &self,
        user_id: OwnedUserId,
        device_id: OwnedDeviceId,
    ) -> crate::Result<()> {
        verify_device(&self.0.app(), &user_id, &device_id).await
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
