use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<MatrixSvelte<R>> {
  Ok(MatrixSvelte(app.clone()))
}

/// Access to the matrix-svelte APIs.
pub struct MatrixSvelte<R: Runtime>(AppHandle<R>);

impl<R: Runtime> MatrixSvelte<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
