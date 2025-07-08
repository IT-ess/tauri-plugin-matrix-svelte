use matrix_sdk::media::MediaRequestParameters;
use serde::de::DeserializeOwned;
use tauri::{ipc::Channel, plugin::PluginApi, AppHandle, Runtime};

use crate::{
    matrix::{
        create_session_to_state,
        login::{LoginRequest, MatrixClientConfig},
        requests::{submit_async_request, MatrixRequest},
    },
    models::matrix::MediaStreamEvent,
    ping::{PingRequest, PingResponse},
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
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

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
}
