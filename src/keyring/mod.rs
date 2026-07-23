use rand::RngExt;
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
    // The iOS Notification Service Extension reads the session while the
    // device may still be locked (pushes arrive any time), so the entry must
    // be accessible after first unlock instead of the default when-unlocked.
    #[cfg(target_os = "ios")]
    {
        let modifiers =
            std::collections::HashMap::from([("access-policy", "after-first-unlock")]);
        return Entry::new_with_modifiers("service.name", &entry_username, &modifiers)
            .map_err(Into::into);
    }
    #[cfg(not(target_os = "ios"))]
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

/// Install the platform-native keyring backend as the process-wide default
/// `keyring_core` store.
///
/// `ios_access_group` is only read on iOS: the keychain access group shared
/// between the app and its Notification Service Extension (the App Group id,
/// e.g. `group.com.example.app`), so both processes read/write the same
/// session entry. `None` keeps the app's default access group (no NSE
/// support). Other platforms ignore it.
///
/// Idempotent: the underlying `keyring_core::set_default_store` may only be
/// called once per process, and this has to be callable both from the plugin
/// `setup` (warm path) and from the background silent-push entries (cold
/// paths where `setup` never runs). A failed init is *not* latched — the
/// mutex flag is only set on success, so the next caller retries instead of
/// being handed a spurious `Ok(())` with no store installed.
pub fn init_keyring_store(ios_access_group: Option<&str>) -> anyhow::Result<()> {
    use std::sync::Mutex;
    static INIT: Mutex<bool> = Mutex::new(false);
    let mut done = INIT.lock().unwrap();
    if *done {
        return Ok(());
    }
    #[cfg(target_os = "ios")]
    init_keyring_store_inner(ios_access_group)?;
    #[cfg(not(target_os = "ios"))]
    {
        let _ = ios_access_group;
        init_keyring_store_inner()?;
    }
    *done = true;
    Ok(())
}

#[cfg(target_os = "ios")]
fn init_keyring_store_inner(access_group: Option<&str>) -> anyhow::Result<()> {
    use apple_native_keyring_store::protected::Store as IOSStore;
    let store = match access_group {
        Some(group) => {
            let config = std::collections::HashMap::from([("access-group", group)]);
            IOSStore::new_with_configuration(&config).map_err(anyhow::Error::from)?
        }
        None => IOSStore::new().map_err(anyhow::Error::from)?,
    };
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(not(target_os = "ios"))]
fn init_keyring_store_inner() -> anyhow::Result<()> {
    #[cfg(target_os = "android")]
    {
        use android_native_keyring_store::Store as AndroidStore;
        let store = AndroidStore::new().map_err(anyhow::Error::from)?;
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
