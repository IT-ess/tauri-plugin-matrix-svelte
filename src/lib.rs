use std::{thread, time::Duration};

use matrix::{
    requests::MatrixRequest,
    room::rooms_list::{enqueue_rooms_list_update, RoomsListUpdate},
    singletons::{CLIENT, LOGIN_STORE_READY, REQUEST_SENDER},
    stores::login_store::{update_login_state, LoginState},
    try_restore_session_to_state,
    workers::{async_main_loop, async_worker},
};
use serde::Deserialize;
use stronghold::init_stronghold_client;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;

pub mod matrix;
pub mod models;
pub mod stronghold;
pub mod utils;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::MatrixSvelte;
#[cfg(mobile)]
use mobile::MatrixSvelte;

use crate::{
    matrix::{
        notifications::enqueue_toast_notification, room::rooms_list::RoomsCollectionStatus,
        singletons::TEMP_DIR,
    },
    models::matrix::{ToastNotificationRequest, ToastNotificationVariant},
    utils::fs::get_temp_dir_or_create_it,
};

// Plugin config
#[derive(Deserialize)]
pub struct PluginConfig {
    stronghold_password: String,
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
            commands::watch_notifications
        ])
        .setup(|app, api| {
            // Create a channel to be used between UI thread(s) and the async worker thread.
            crate::matrix::singletons::init_broadcaster(16)
                .expect("Couldn't init the UI broadcaster"); // TODO: adapt capacity if needed

            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<MatrixRequest>();
            REQUEST_SENDER
                .set(sender)
                .expect("BUG: REQUEST_SENDER already set!");

            let init_app_handle = app.app_handle().clone();
            let stronghold_app_handle = app.app_handle().clone();
            let main_loop_app_handle = app.app_handle().clone();

            let temp_dir = get_temp_dir_or_create_it(&init_app_handle)?;

            TEMP_DIR.set(temp_dir).expect("Couldn't set temporary dir");

            // use tauri_plugin_notification::NotificationExt;
            // //Check notifications permissions based on status.
            // match app.notification().permission_state().unwrap() {
            //     PermissionState::Prompt
            //     | PermissionState::PromptWithRationale
            //     | PermissionState::Denied => {
            //         app.notification().request_permission().unwrap();
            //         app.notification()
            //             .builder()
            //             .title("Tauri")
            //             .body("Tauri is awesome")
            //             .show()
            //             .unwrap();
            //     }
            // PermissionState::Granted => {
            //     app.notification()
            //         .builder()
            //         .title("Tauri")
            //         .body("Tauri is awesome")
            //         .show()
            //         .unwrap();
            // }
            // }

            let stronghold_handle = tauri::async_runtime::spawn(async move {
                init_stronghold_client(&stronghold_app_handle)
                    .expect("Couldn't init stronghold client")
            });

            let _monitor = tauri::async_runtime::spawn(async move {
                stronghold_handle
                    .await
                    .expect("Couldn't init stronghold client");
                let client = try_restore_session_to_state(&init_app_handle)
                    .await
                    .expect("Couldn't try to restore session");

                LOGIN_STORE_READY.wait();
                let client = match client {
                    Some(new_login) => {
                        // Should check that the login store is available before. With the on_load store hook ?
                        update_login_state(
                            &init_app_handle,
                            LoginState::Restored,
                            new_login
                                .user_id()
                                .map_or(None, |val| Some(val.to_string())),
                        )
                        .expect("Couldn't update login state");
                        new_login
                    }
                    None => {
                        println!("Waiting for login request...");
                        thread::sleep(Duration::from_secs(3)); // Block the thread for 3 secs to let the frontend init itself.
                        update_login_state(&init_app_handle, LoginState::AwaitingForLogin, None)
                            .expect("Couldn't update login state");
                        // We await frontend to call the login command and set the client
                        // loop until client is set
                        CLIENT.wait();
                        let client = CLIENT.get().unwrap().clone();
                        update_login_state(
                            &init_app_handle,
                            LoginState::LoggedIn,
                            client
                                .user_id()
                                .clone()
                                .map_or(None, |val| Some(val.to_string())),
                        )
                        .expect("Couldn't update login state");
                        client
                    }
                };

                let mut ui_event_receiver = crate::matrix::singletons::subscribe_to_events()
                    .expect("Couldn't get UI event receiver"); // subscribe to events so the sender(s) never fail

                // client.register_notification_handler();

                // Spawn the actual async worker thread.
                let mut worker_join_handle = tauri::async_runtime::spawn(async_worker(receiver));

                // // Start the main loop that drives the Matrix client SDK.
                let mut main_loop_join_handle =
                    tauri::async_runtime::spawn(async_main_loop(main_loop_app_handle, client));

                #[allow(clippy::never_loop)] // unsure if needed, just following tokio's examples.
                loop {
                    tokio::select! {
                        result = &mut main_loop_join_handle => {
                            match result {
                                Ok(Ok(())) => {
                                    eprintln!("BUG: main async loop task ended unexpectedly!");
                                }
                                Ok(Err(e)) => {
                                    eprintln!("Error: main async loop task ended:\n\t{e:?}");
                                    enqueue_rooms_list_update(RoomsListUpdate::Status {
                                        status: RoomsCollectionStatus::Error(e.to_string()),
                                    });
                                    enqueue_toast_notification(ToastNotificationRequest::new(
                                        format!("Rooms list update error: {e}"),
                                        None,
                                        ToastNotificationVariant::Error,
                                    ));
                                },
                                Err(e) => {
                                    eprintln!("BUG: failed to join main async loop task: {e:?}");
                                }
                            }
                            break;
                        }
                        result = &mut worker_join_handle => {
                            match result {
                                Ok(Ok(())) => {
                                    eprintln!("BUG: async worker task ended unexpectedly!");
                                }
                                Ok(Err(e)) => {
                                    eprintln!("Error: async worker task ended:\n\t{e:?}");
                                    enqueue_rooms_list_update(RoomsListUpdate::Status {
                                        status: RoomsCollectionStatus::Error(e.to_string()),
                                    });
                                    enqueue_toast_notification(ToastNotificationRequest::new(
                                        format!("Rooms list update error: {e}"),
                                        None,
                                        ToastNotificationVariant::Error,
                                    ));
                                },
                                Err(e) => {
                                    eprintln!("BUG: failed to join async worker task: {e:?}");
                                }
                            }
                            break;
                        }
                        _ = ui_event_receiver.recv() => {
                            #[cfg(debug_assertions)]
                            println!("Received UI update event");
                        }
                    }
                }
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
