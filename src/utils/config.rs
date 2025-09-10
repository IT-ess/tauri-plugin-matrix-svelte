use crate::PluginConfig;
use tauri::{AppHandle, Runtime};

pub fn _get_plugin_config<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PluginConfig> {
    let plugin_config = app_handle.config().plugins.0.clone();
    let raw_matrix_config = plugin_config
        .get("matrix-svelte")
        .expect("Plugin 'matrix-svelte' configuration not found");
    let matrix_plugin_config: crate::PluginConfig =
        serde_json::from_value(raw_matrix_config.clone())?;
    Ok(matrix_plugin_config)
}
