use anyhow::anyhow;
use matrix_sdk::{
    media::MediaRequestParameters,
    ruma::{OwnedDeviceId, OwnedRoomId, OwnedUserId},
};
use serde::de::DeserializeOwned;
use tauri::{ipc::Channel, plugin::PluginApi, AppHandle, Runtime};

use crate::{
    matrix::{
        create_session_to_state, get_devices,
        login::{LoginRequest, MatrixClientConfig},
        requests::{submit_async_request, MatrixRequest},
        user_profile::fetch_user_profile,
        verify_device,
    },
    models::matrix::{FrontendDevice, MediaStreamEvent},
    notifications::{GetTokenRequest, GetTokenResponse, WatchNotificationResult},
};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<MatrixSvelte<R>> {
    Ok(MatrixSvelte(app.clone()))
}

/// Access to the Matrix Svelte APIs.
pub struct MatrixSvelte<R: Runtime>(AppHandle<R>);

impl<R: Runtime> MatrixSvelte<R> {
    pub async fn login_and_create_new_session(
        &self,
        config: MatrixClientConfig,
    ) -> crate::Result<()> {
        create_session_to_state(&self.0, LoginRequest::LoginByPassword(config)).await?;
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
        verify_device(&self.0, &user_id, &device_id).await
    }

    // Not implemented on desktop

    pub fn get_token(&self, _payload: GetTokenRequest) -> crate::Result<GetTokenResponse> {
        Err(crate::Error::Anyhow(anyhow!("Not implemented on desktop")))
    }

    pub fn watch_notifications(&self, _channel: Channel) -> crate::Result<WatchNotificationResult> {
        Err(crate::Error::Anyhow(anyhow!("Not implemented on desktop")))
    }
}
