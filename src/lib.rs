use matrix_ui_serializable::{LibConfig, notifications::MobilePushNotificationConfig};
use serde::Deserialize;
use stronghold::init_stronghold_client;
use tauri::{
    AppHandle, Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod events;
mod models;
mod state_updaters;
mod stronghold;
mod utils;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::MatrixSvelte;
#[cfg(mobile)]
use mobile::MatrixSvelte;

use crate::{
    events::{event_forwarder, handle_incoming_events},
    state_updaters::Updaters,
    stronghold::get_matrix_session_option,
    utils::fs::get_temp_dir_or_create_it,
};

// Plugin config
#[derive(Deserialize)]
pub struct PluginConfig {
    stronghold_password: String,
    #[cfg(mobile)]
    sygnal_gateway_url: String,
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
            commands::login_and_create_new_session,
            commands::submit_async_request,
            commands::fetch_media,
            commands::fetch_user_profile,
            commands::watch_notifications,
            commands::get_devices,
            commands::verify_device
        ])
        .setup(|app, api| {
            let init_app_handle = app.app_handle().clone();
            let stronghold_app_handle = app.app_handle().clone();

            let temp_dir = get_temp_dir_or_create_it(&init_app_handle)?;

            let stronghold_handle = tauri::async_runtime::spawn(async move {
                init_stronghold_client(&stronghold_app_handle)
                    .expect("Couldn't init stronghold client")
            });

            let _monitor = tauri::async_runtime::spawn(async move {
                stronghold_handle
                    .await
                    .expect("Couldn't init stronghold client");

                let session_option = get_matrix_session_option(&init_app_handle)
                    .await
                    .expect("Couldn't get session option");

                let event_receivers = handle_incoming_events(&init_app_handle);

                let push_config = get_push_config(&init_app_handle);

                let updaters_handle = init_app_handle.clone();
                let updaters = Updaters::new(updaters_handle);

                let config = LibConfig::new(
                    Box::new(updaters),
                    push_config,
                    event_receivers,
                    session_option,
                    temp_dir,
                );
                let receiver = matrix_ui_serializable::init(config);

                let inner_app_handle = init_app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    futures::executor::block_on(event_forwarder(inner_app_handle, receiver))
                })
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

fn get_push_config<R: Runtime>(_app_handle: &AppHandle<R>) -> Option<MobilePushNotificationConfig> {
    #[cfg(desktop)]
    return None;
    #[cfg(mobile)]
    {
        use crate::MatrixSvelteExt;
        use crate::mobile::GetTokenRequest;
        if let Ok(push_token) = _app_handle.matrix_svelte().get_token(GetTokenRequest {}) {
            let plugin_config =
                get_plugin_config(_app_handle).expect("The plugin config is not defined !");
            let identifier = _app_handle.config().identifier;
            #[cfg(target_os = "android")]
            let identifier = identifier.replace("-", "_"); // On android, - are replaced by _ in bundle names

            return Some(MobilePushNotificationConfig::new(
                push_token.token,
                plugin_config.sygnal_gateway_url,
                identifier,
            ));
        } else {
            return None;
        }
    }
}

// Re-export for app
pub use crate::state_updaters::LOGIN_STATE_STORE_ID;
pub use matrix_ui_serializable::LOGIN_STORE_READY;
