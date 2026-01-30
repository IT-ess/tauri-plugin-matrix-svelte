use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};
use tracing::{trace, warn};

use crate::PluginConfig;

pub fn get_app_dir_or_create_it<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PathBuf> {
    let app_data_dir = app_handle.path().app_data_dir()?;

    match std::fs::create_dir(&app_data_dir) {
        Ok(_) => trace!("Directory created"),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Do nothing if the directory already exists
            trace!("App data directory already exists.")
        }
        Err(e) => {
            // Handle other errors
            warn!("Error creating directory: {}", e);
        }
    }
    Ok(app_data_dir)
}

pub fn _get_temp_dir_or_create_it<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<PathBuf> {
    let temp_data_dir = app_handle.path().temp_dir()?;

    match std::fs::create_dir(&temp_data_dir) {
        Ok(_) => trace!("Directory created"),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Do nothing if the directory already exists
            trace!("Temp data directory already exists.")
        }
        Err(e) => {
            // Handle other errors
            warn!("Error creating directory: {}", e);
        }
    }
    Ok(temp_data_dir)
}

pub fn _get_plugin_config<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PluginConfig> {
    let plugin_config = app_handle.config().plugins.0.clone();
    let raw_matrix_config = plugin_config
        .get("matrix-svelte")
        .expect("Plugin 'matrix-svelte' configuration not found");
    let matrix_plugin_config: crate::PluginConfig =
        serde_json::from_value(raw_matrix_config.clone())?;
    Ok(matrix_plugin_config)
}
