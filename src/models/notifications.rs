use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenRequest {}

// TODO: adapt to the exact iOS body
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct WatchNotificationsArgs {
    pub channel: Channel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchNotificationResult {
    pub success: bool,
}
