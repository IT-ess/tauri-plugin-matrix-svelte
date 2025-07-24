use tauri::{AppHandle, Runtime};

static SESSION_KEY: &str = "MATRIX_SVELTE_CLIENT_SESSION";

// TODO: replace these methods with the tauri-plugin-os-secrets ones.

pub fn save_session<R: Runtime>(app_handle: &AppHandle<R>, session: String) -> anyhow::Result<()> {
    #[cfg(mobile)]
    {
        use tauri_plugin_keychain::{KeychainExt, KeychainRequest};
        let request = KeychainRequest {
            key: Some(SESSION_KEY.to_string()),
            password: Some(session),
        };
        app_handle.save_item().save_item(request)?;
    }
    Ok(())
}

pub fn retrieve_session<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<Option<String>> {
    let mut session = None;
    #[cfg(mobile)]
    {
        use tauri_plugin_keychain::{KeychainExt, KeychainRequest};
        let request = KeychainRequest {
            key: Some(SESSION_KEY.to_string()),
            password: None,
        };
        session = app_handle
            .get_item()
            .get_item(request)
            .expect("Couldn't get item")
            .password;
    }
    Ok(session)
}
