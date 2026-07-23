use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};
use tracing::{trace, warn};

use crate::PluginConfig;

pub fn get_app_dir_or_create_it<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PathBuf> {
    // On iOS the Matrix store must live in the App Group container when one is
    // configured, so the Notification Service Extension (a separate process
    // with a disjoint sandbox) can open the same store and salt file.
    #[cfg(target_os = "ios")]
    let app_data_dir = match get_plugin_config(app_handle)?.ios_app_group {
        Some(group) => ios_app_group_container_dir(&group)?,
        None => app_handle.path().app_data_dir()?,
    };
    #[cfg(not(target_os = "ios"))]
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

/// Resolve the on-disk container of an App Group (e.g. `group.com.example.app`).
///
/// This is the same path the NSE's Swift side resolves through
/// `FileManager.containerURL(forSecurityApplicationGroupIdentifier:)` and
/// hands to the Rust silent-push handler as `data_dir`. Fails when the App
/// Group entitlement is missing or the identifier is not registered.
#[cfg(target_os = "ios")]
fn ios_app_group_container_dir(group: &str) -> anyhow::Result<PathBuf> {
    use objc2_foundation::{NSFileManager, NSString};

    let manager = NSFileManager::defaultManager();
    let group_id = NSString::from_str(group);
    let url = manager
        .containerURLForSecurityApplicationGroupIdentifier(&group_id)
        .ok_or_else(|| {
            anyhow::anyhow!("App Group container unavailable for {group} (entitlement missing?)")
        })?;
    let path = url
        .path()
        .ok_or_else(|| anyhow::anyhow!("App Group container URL for {group} has no path"))?;
    Ok(PathBuf::from(path.to_string()))
}

pub fn get_plugin_config<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PluginConfig> {
    let plugin_config = app_handle.config().plugins.0.clone();
    let raw_matrix_config = plugin_config
        .get("matrix-svelte")
        .expect("Plugin 'matrix-svelte' configuration not found");
    let matrix_plugin_config: crate::PluginConfig =
        serde_json::from_value(raw_matrix_config.clone())?;
    Ok(matrix_plugin_config)
}
