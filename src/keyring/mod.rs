use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::warn;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use keyring_core::Entry;

pub fn get_matrix_session_option(app_data_path: PathBuf) -> Option<String> {
    match get_session_from_keyring(app_data_path) {
        Ok(session) => Some(session),
        Err(e) => {
            warn!("Couldn't get session from keyring. {e}");
            None
        }
    }
}

fn create_entry(salt: &str) -> crate::Result<Entry> {
    let entry_username = format!("{}/{}/{}", "service.name", "current_user", salt);
    Entry::new("service.name", &entry_username).map_err(Into::into)
}

fn get_salt_or_create_it(app_data_path: PathBuf) -> io::Result<String> {
    let salt_path = app_data_path.join("salt");

    if salt_path.exists() {
        fs::read_to_string(&salt_path)
    } else {
        let salt: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let mut file = fs::File::create(&salt_path)?;
        file.write_all(salt.as_bytes())?;

        Ok(salt)
    }
}

fn get_session_from_keyring(app_data_path: PathBuf) -> crate::Result<String> {
    let salt = get_salt_or_create_it(app_data_path)?;
    let entry = create_entry(&salt)?;
    let encoded_bytes = entry.get_secret()?;
    let encoded_str = String::from_utf8(encoded_bytes).map_err(anyhow::Error::from)?;
    let decoded = BASE64.decode(encoded_str).map_err(anyhow::Error::from)?;
    let session_string = String::from_utf8_lossy(&decoded).into_owned();
    Ok(session_string)
}

pub(crate) fn set_session_in_keyring(
    session: Vec<u8>,
    app_data_path: PathBuf,
) -> crate::Result<()> {
    let salt = get_salt_or_create_it(app_data_path)?;
    let entry = create_entry(&salt)?;
    let encoded = BASE64.encode(&session);
    entry.set_secret(encoded.as_bytes()).map_err(Into::into)
}

pub(crate) fn clear_session_in_keyring(app_data_path: PathBuf) -> crate::Result<()> {
    let salt = get_salt_or_create_it(app_data_path.clone())?;
    let entry = create_entry(&salt)?;
    entry.delete_credential()?;
    // Remove salt so we do not use the previous DB.
    fs::remove_file(app_data_path.join("salt")).map_err(|e| e.into())
}

pub(crate) fn init_keyring_store() -> anyhow::Result<()> {
    #[cfg(target_os = "android")]
    {
        use android_native_keyring_store::credential::AndroidStore;
        let store = AndroidStore::from_ndk_context().map_err(anyhow::Error::from)?;
        keyring_core::set_default_store(store);
    }

    #[cfg(target_os = "ios")]
    {
        use apple_native_keyring_store::protected::Store as IOSStore;
        let store = IOSStore::new().map_err(anyhow::Error::from)?;
        keyring_core::set_default_store(store);
    }

    // Initialize platform-specific store
    #[cfg(target_os = "windows")]
    {
        use windows_native_keyring_store::Store as WindowsStore;
        let store = WindowsStore::new().map_err(anyhow::Error::from)?;
        keyring_core::set_default_store(store);
    }

    #[cfg(target_os = "macos")]
    {
        use apple_native_keyring_store::protected::Store as MacOSStore;
        let store = MacOSStore::new().map_err(anyhow::Error::from)?;
        keyring_core::set_default_store(store);
    }

    #[cfg(target_os = "linux")]
    {
        use dbus_secret_service_keyring_store::Store as LinuxStore;
        let store = LinuxStore::new().map_err(anyhow::Error::from)?;
        keyring_core::set_default_store(store);
    }
    Ok(())
}
