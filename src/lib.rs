use std::sync::OnceLock;

use matrix_ui_serializable::{LibConfig, models::events::MatrixLoginPayload, mpsc};
use serde::Deserialize;
use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod events;
mod keyring;
mod state_updaters;
mod utils;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::MatrixSvelte;
#[cfg(mobile)]
use mobile::MatrixSvelte;
use tracing::{debug, error, info};
use url::Url;

use crate::{
    events::handle_incoming_events,
    state_updaters::Updaters,
    utils::{get_app_dir_or_create_it, get_plugin_config},
};

pub static LOGIN_SENDER: OnceLock<mpsc::Sender<MatrixLoginPayload>> = OnceLock::new();

pub static AUTH_DEEPLINK_SENDER: OnceLock<mpsc::Sender<Url>> = OnceLock::new();

/// Plugin config to be set in tauri.conf.json
#[derive(Debug, Deserialize)]
pub struct PluginConfig {
    /// The Sygnal Push Notification gateway URL (android)
    pub android_sygnal_gateway_url: Url,
    /// The Sygnal Push Notification gateway URL (iOS)
    pub ios_sygnal_gateway_url: Url,
    /// The client URL for the OAuth flow
    pub oauth_client_uri: Url,
    /// The redirect URI called at the end of the OAuth flow.
    pub oauth_redirect_uri: Url,
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the Matrix Svelte APIs.
pub trait MatrixSvelteExt<R: Runtime> {
    fn matrix_svelte(&self) -> &MatrixSvelte<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MatrixSvelteExt<R> for T {
    fn matrix_svelte(&self) -> &MatrixSvelte<R> {
        self.state::<MatrixSvelte<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R, PluginConfig> {
    Builder::<R, PluginConfig>::new("matrix-svelte")
        .invoke_handler(tauri::generate_handler![
            commands::submit_async_request,
            commands::fetch_media,
            commands::fetch_user_profile,
            commands::get_devices,
            commands::verify_device,
            commands::submit_matrix_login_request,
            commands::forward_oauth_login_deeplink,
            commands::build_client_from_homeserver_url,
            commands::check_homeserver_auth_type,
            commands::get_dm_room_from_user_id,
            commands::check_device_verification,
            commands::has_backup_setup,
            commands::restore_backup_with_passphrase,
            commands::setup_new_backup,
            commands::search_users,
            commands::disconnect_and_clear_session,
            commands::check_if_last_device,
            commands::is_logged_in,
            commands::reset_cross_signing,
            commands::edit_user_information,
            commands::upload_media,
            commands::filter_room_list,
            commands::define_room_informations,
            commands::register_notifications
        ])
        .setup(|app, api| {
            let init_app_handle = app.app_handle().clone();

            let app_data_dir = get_app_dir_or_create_it(&init_app_handle)?;

            // keyring
            keyring::init_keyring_store().expect("couldn't init keyring store");

            // Create download dir for files
            let path = init_app_handle
                .path()
                .app_local_data_dir()
                .expect("Couldn't get app local data dir");
            let path = path.join("download");
            match std::fs::create_dir(&path) {
                Ok(_) => info!("Download directory created"),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Do nothing if the directory already exists
                    debug!("Download directory already exists.")
                }
                Err(e) => {
                    // Handle other errors
                    error!("Error creating directory: {}", e);
                }
            }

            let forwarder_handle = app.app_handle().clone();

            let _monitor = tauri::async_runtime::spawn(async move {
                let session_option = keyring::get_matrix_session_option(app_data_dir.clone());

                let event_receivers = handle_incoming_events(&init_app_handle);

                let updaters_handle = init_app_handle.clone();
                let updaters = Updaters::new(updaters_handle);

                let plugin_config = get_plugin_config(&init_app_handle)
                    .expect("Some plugin configuration is missing");

                let config = LibConfig::new(
                    Box::new(updaters),
                    event_receivers,
                    session_option,
                    app_data_dir,
                    plugin_config.oauth_client_uri,
                    plugin_config.oauth_redirect_uri,
                );
                let receiver = matrix_ui_serializable::init(config);
                tauri::async_runtime::spawn(async move {
                    events::event_forwarder(forwarder_handle, receiver).await
                });
            });

            #[cfg(mobile)]
            let matrix_svelte = mobile::init(app, api)?;
            #[cfg(desktop)]
            let matrix_svelte = desktop::init(app, api)?;
            app.manage(matrix_svelte);
            Ok(())
        })
        .build()
}

// Re-export for app
pub use crate::state_updaters::LOGIN_STATE_STORE_ID;
pub use matrix_ui_serializable::LOGIN_STORE_READY;
