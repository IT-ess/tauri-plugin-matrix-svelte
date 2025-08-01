use tauri::Manager;

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
        .setup(|app| {
            #[cfg(desktop)]
            {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::TrayIconBuilder
            };

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let _tray = TrayIconBuilder::new()
              .icon(app.default_window_icon().unwrap().clone())
              .menu(&menu)
              .show_menu_on_left_click(true)
              .on_menu_event(|app, event| match event.id.as_ref() {
                "quit" => {
                  println!("quit menu item was clicked");
                  app.exit(0);
                }
                _ => {
                  println!("menu item {:?} not handled", event.id);
                }
              })
              .build(app)?;
            }

            // Create download dir for files
            let path = app.path().app_local_data_dir().expect("Couldn't get app local data dir");
            let path = path.join("download");
            match std::fs::create_dir(&path) {
                Ok(_) => println!("Directory created"),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Do nothing if the directory already exists
                    println!("App data directory already exists.")
                }
                Err(e) => {
                    // Handle other errors
                    eprintln!("Error creating directory: {}", e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
