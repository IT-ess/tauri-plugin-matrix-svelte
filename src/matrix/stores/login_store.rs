use std::ops::{Deref, DerefMut};

use matrix_sdk::encryption::VerificationState;
use matrix_sdk_ui::sync_service;
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

#[derive(Debug, Clone)]
pub struct FrontendVerificationState(VerificationState);

impl Deref for FrontendVerificationState {
    type Target = VerificationState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendVerificationState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FrontendVerificationState {
    pub fn new(state: VerificationState) -> Self {
        Self(state)
    }

    fn to_camel_case(&self) -> &str {
        match self {
            FrontendVerificationState(VerificationState::Unknown) => "unknown",
            FrontendVerificationState(VerificationState::Verified) => "verified",
            FrontendVerificationState(VerificationState::Unverified) => "unverified",
        }
    }
}

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

pub fn update_verification_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    verification_state: FrontendVerificationState,
) -> anyhow::Result<()> {
    app_handle.svelte().set(
        LOGIN_STATE_STORE_ID,
        "verificationState",
        verification_state.to_camel_case(),
    )?;
    Ok(())
}

pub struct FrontendSyncServiceState(sync_service::State);

impl Deref for FrontendSyncServiceState {
    type Target = sync_service::State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendSyncServiceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FrontendSyncServiceState {
    pub fn new(state: sync_service::State) -> Self {
        Self(state)
    }

    fn to_camel_case(&self) -> &str {
        match self {
            FrontendSyncServiceState(sync_service::State::Error) => "error",
            FrontendSyncServiceState(sync_service::State::Idle) => "idle",
            FrontendSyncServiceState(sync_service::State::Offline) => "offline",
            FrontendSyncServiceState(sync_service::State::Running) => "running",
            FrontendSyncServiceState(sync_service::State::Terminated) => "terminated",
        }
    }
}

pub fn update_sync_service_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: FrontendSyncServiceState,
) -> anyhow::Result<()> {
    app_handle.svelte().set(
        LOGIN_STATE_STORE_ID,
        "syncServiceState",
        state.to_camel_case(),
    )?;
    Ok(())
}
