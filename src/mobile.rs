use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_matrix_svelte);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<MatrixSvelte<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.matrix-svelte", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_matrix_svelte)?;
  Ok(MatrixSvelte(handle))
}

/// Access to the matrix-svelte APIs.
pub struct MatrixSvelte<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MatrixSvelte<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }
}
