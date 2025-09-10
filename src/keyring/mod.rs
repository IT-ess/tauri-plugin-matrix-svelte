use std::{fs, path::PathBuf};

use anyhow::anyhow;
use tauri::{AppHandle, Runtime};
use tauri_plugin_keyring::KeyringExt;

use crate::utils::fs::get_app_dir_or_create_it;

static CURRENT_USERNAME_FILENAME: &str = "current_user.txt";

pub async fn get_matrix_session_option<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<Option<String>> {
    let username = get_current_username(app_handle)?;
    if username.is_none() {
        return Ok(None);
    }
    match app_handle.keyring().get(
        &username.unwrap(),
        tauri_plugin_keyring::CredentialType::Secret,
    ) {
        Ok(tauri_plugin_keyring::CredentialValue::Secret(raw_session)) => {
            let session_string = String::from_utf8_lossy(&raw_session).into_owned();
            Ok(Some(session_string))
        }
        Err(e) => match e {
            tauri_plugin_keyring::Error::EntryNotFound => Ok(None),
            _e => Err(anyhow!(_e)),
        },
        _ => Err(anyhow!("Invalid credential type".to_string())),
    }
}

pub fn set_current_username(path: PathBuf, username: &str) -> anyhow::Result<()> {
    let username_pathbuf = path.join(CURRENT_USERNAME_FILENAME);
    fs::write(username_pathbuf, username)?;
    Ok(())
}

fn get_current_username<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<Option<String>> {
    let app_data_dir = get_app_dir_or_create_it(app_handle)?;

    let username_pathbuf = app_data_dir.join(CURRENT_USERNAME_FILENAME);

    let raw_username = fs::read(username_pathbuf);
    match raw_username {
        Ok(name) => {
            if name.is_empty() {
                return Ok(None);
            };
            Ok(Some(String::from_utf8_lossy(&name).into_owned()))
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => Err(anyhow!(e)),
        },
    }
}
