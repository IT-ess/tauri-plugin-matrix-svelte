use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_matrix_svelte::AUTH_DEEPLINK_SENDER;
use tracing::debug;

mod logging;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    builder = logging::setup_logging(builder);

    // Init deeplink plugin before tauri_plugin_mobile_sharetarget
    builder = builder.plugin(tauri_plugin_deep_link::init());

    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_mobile_sharetarget::init());
    }

    #[cfg(target_os = "ios")]
    {
        builder = builder.plugin(tauri_plugin_web_auth::init());
    }

    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_safe_area_insets_css::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notifications::init())
        .plugin(
            tauri_plugin_svelte::Builder::new()
                .on_load(|store| {
                    if store.id().to_string() == tauri_plugin_matrix_svelte::LOGIN_STATE_STORE_ID {
                        tauri_plugin_matrix_svelte::LOGIN_STORE_READY
                            .set(true)
                            .expect("LOGIN_STORE_READY has already been set !");
                    }
                    Ok(())
                })
                .build(),
        )
        .plugin(tauri_plugin_matrix_svelte::init())
        .setup(|app| {
            // Tray icon stuff
            #[cfg(desktop)]
            {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::TrayIconBuilder,
                };
                use tracing::warn;

                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&quit_i])?;
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            debug!("quit menu item was clicked");
                            app.exit(0);
                        }
                        _ => {
                            warn!("menu item {:?} not handled", event.id);
                        }
                    })
                    .build(app)?;
            }

            // Handle refs:// deeplink

            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link()
                    .register_all()
                    .expect("couldn't register deeplink");
            }

            #[cfg(target_os = "ios")]
            {
                use tauri_plugin_mobile_sharetarget::{IOS_DEEP_LINK_SCHEME, push_new_intent};
                let start_urls = app.deep_link().get_current()?;
                if let Some(urls) = start_urls {
                    debug!("deep link URLs: {:?}", urls);
                    if let Some(url) = urls.first() {
                        if url.scheme().eq(IOS_DEEP_LINK_SCHEME.wait()) {
                            push_new_intent(url.to_string());
                        }
                    }
                }
            }

            let deeplink_handle = app.app_handle().clone();

            app.deep_link().on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
                    // Matches refs://share
                    #[cfg(target_os = "ios")]
                    {
                        use tauri_plugin_mobile_sharetarget::{
                            IOS_DEEP_LINK_SCHEME, push_new_intent,
                        };
                        if url.scheme().eq(IOS_DEEP_LINK_SCHEME.wait()) {
                            // We can't rely on tauri://focus for iOS for some reason.
                            deeplink_handle.emit("new-intent", ()).unwrap();
                            push_new_intent(url.to_string());
                        }
                    }

                    // Matches refs://auth-callback
                    if url.host_str().is_some_and(|s| s.eq("auth-callback")) {
                        // Wake up the UI (for iOS only)
                        deeplink_handle.emit("new-intent", ()).unwrap();
                        let sender = AUTH_DEEPLINK_SENDER
                            .get()
                            .expect("sender should be defined at this point");
                        sender
                            .blocking_send(url.to_owned())
                            .expect("couldn't send deeplink payload to receiver");
                    }
                    // Matches https://refs.rs/auth-callback
                    if url.host_str().is_some_and(|s| s.eq("refs.rs"))
                        & url.path().contains("auth-callback")
                    {
                        let sender = AUTH_DEEPLINK_SENDER
                            .get()
                            .expect("sender should be defined at this point");
                        sender
                            .blocking_send(url.to_owned())
                            .expect("couldn't send deeplink payload to receiver");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
