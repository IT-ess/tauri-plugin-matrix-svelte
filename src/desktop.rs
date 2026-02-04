use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<MatrixSvelte<R>> {
    Ok(MatrixSvelte(app.clone()))
}

/// Access to the Matrix Svelte APIs.
pub struct MatrixSvelte<R: Runtime>(AppHandle<R>);

impl<R: Runtime> MatrixSvelte<R> {}
