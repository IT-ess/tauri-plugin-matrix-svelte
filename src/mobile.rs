use anyhow::anyhow;
use matrix_ui_serializable::{
    MatrixClientConfig, MatrixRequest, MediaRequestParameters, OwnedDeviceId, OwnedRoomId,
    OwnedUserId,
    commands::{SearchBatch, SearchConfig},
    models::events::{FrontendDevice, MediaStreamEvent},
};
use serde::de::DeserializeOwned;
use tauri::{
    AppHandle, Runtime,
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
};

use crate::{
    models::mobile::{GetTokenRequest, GetTokenResponse, WatchNotificationResult},
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

impl<R: Runtime> MatrixSvelte<R> {}
