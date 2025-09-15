use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

pub fn get_app_dir_or_create_it<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<PathBuf> {
    let app_data_dir = app_handle.path().app_data_dir()?;

    match std::fs::create_dir(&app_data_dir) {
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
    Ok(app_data_dir)
}

pub fn _get_temp_dir_or_create_it<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<PathBuf> {
    let temp_data_dir = app_handle.path().temp_dir()?;

    match std::fs::create_dir(&temp_data_dir) {
        Ok(_) => println!("Directory created"),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Do nothing if the directory already exists
            println!("Temp data directory already exists.")
        }
        Err(e) => {
            // Handle other errors
            eprintln!("Error creating directory: {}", e);
        }
    }
    Ok(temp_data_dir)
}
