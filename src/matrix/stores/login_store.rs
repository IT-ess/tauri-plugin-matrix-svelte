use matrix_sdk::ruma::UserId;
use tauri::{AppHandle, Runtime};
use tauri_plugin_svelte::ManagerExt;

#[derive(Debug, PartialEq)]
pub enum LoginState {
    Initiating,
    Restored,
    AwaitingForLogin,
    LoggedIn,
}

impl LoginState {
    // Method to convert LoginState to camelCase string
    fn to_camel_case(&self) -> String {
        match self {
            LoginState::Initiating => "initiating".to_string(),
            LoginState::Restored => "restored".to_string(),
            LoginState::AwaitingForLogin => "awaitingForLogin".to_string(),
            LoginState::LoggedIn => "loggedIn".to_string(),
        }
    }

    // Method to convert camelCase string to LoginState
    fn _from_camel_case(s: &str) -> LoginState {
        match s {
            "initiating" => LoginState::Initiating,
            "restored" => LoginState::Restored,
            "awaitingForLogin" => LoginState::AwaitingForLogin,
            "loggedIn" => LoginState::LoggedIn,
            _ => LoginState::Initiating, // Default fallback
        }
    }
}

pub const LOGIN_STATE_STORE_ID: &str = "login-state";

pub fn update_login_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: LoginState,
    user_id: Option<String>,
) -> anyhow::Result<()> {
    app_handle
        .svelte()
        .set(LOGIN_STATE_STORE_ID, "state", state.to_camel_case())?;
    app_handle
        .svelte()
        .set(LOGIN_STATE_STORE_ID, "userId", user_id)?;
    Ok(())
}
