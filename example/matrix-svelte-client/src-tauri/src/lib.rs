#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_svelte::Builder::new()
                .on_load(|store| {
                    if store.id().to_string() == tauri_plugin_matrix_svelte::matrix::stores::login_store::LOGIN_STATE_STORE_ID {
                        tauri_plugin_matrix_svelte::matrix::singletons::LOGIN_STORE_READY.set(true).expect("LOGIN_STORE_READY has already been set !");
                    }
                    Ok(())
                })
                .build(),
        )
        .plugin(tauri_plugin_matrix_svelte::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
