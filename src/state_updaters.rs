use std::collections::HashMap;

use anyhow::anyhow;
use async_trait::async_trait;
use matrix_ui_serializable::{
    AuthSession, FrontendSyncServiceState, FrontendVerificationState, FullMatrixSession,
    LoginState, OwnedMxcUri, OwnedUserId, RecoveryState, RoomScreen, RoomsList,
    models::state_updater::{StateUpdater, StateUpdaterFunctions},
};
use serde_json::Value;
use tauri::{AppHandle, Runtime};
#[cfg(target_os = "ios")]
use tauri_plugin_notifications::NotificationsExt;
use tauri_plugin_svelte::{ManagerExt, StoreState};

use crate::{
    keyring::{get_matrix_session_option, set_session_in_keyring},
    utils::get_app_dir_or_create_it,
};

// Keep the same ids as in the JS package !
const ROOM_STORE_ID: &str = "room-store";
const ROOMS_COLLECTION_STORE_ID: &str = "rooms-collection";
pub const LOGIN_STATE_STORE_ID: &str = "login-state";

#[derive(Debug)]
pub struct Updaters<R: Runtime> {
    app_handle: AppHandle<R>,
    /// Last badge count sent to the OS, to skip redundant bridge hops on the
    /// frequent rooms-list updates. `-1` sentinel: the first sync always
    /// fires, so a stale badge left by a killed app is corrected (even to 0).
    #[cfg(target_os = "ios")]
    last_badge: std::sync::atomic::AtomicI64,
}

impl<R: Runtime> Updaters<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self {
            app_handle,
            #[cfg(target_os = "ios")]
            last_badge: std::sync::atomic::AtomicI64::new(-1),
        }
    }

    /// Mirror the total unread count onto the iOS app icon badge.
    ///
    /// Reads the serialized `RoomsList` JSON because the struct's fields are
    /// deliberately private to adapters. `is_marked_unread` is not counted:
    /// the server-side `aps.badge` pushes that correct the badge while the
    /// app is killed know nothing about that local flag, and counting it
    /// would make the badge flip-flop between warm and killed states.
    #[cfg(target_os = "ios")]
    fn sync_ios_badge(&self, rooms_json: &Value) {
        use std::sync::atomic::Ordering;

        let total: u64 = rooms_json
            .get("allJoinedRooms")
            .and_then(Value::as_object)
            .map(|rooms| {
                rooms
                    .values()
                    .filter_map(|room| room.get("numUnreadMessages").and_then(Value::as_u64))
                    .sum()
            })
            .unwrap_or(0);
        let badge = i64::try_from(total).unwrap_or(i64::MAX);
        if self.last_badge.swap(badge, Ordering::Relaxed) == badge {
            return;
        }
        // Saturating clamp: the badge is cosmetic.
        let count = i32::try_from(badge).unwrap_or(i32::MAX);
        // Log-and-continue: a badge failure must never break store patching.
        if let Err(e) = self.app_handle.notifications().set_badge_count(count) {
            tracing::warn!("failed to set app badge to {count}: {e}");
        }
    }
}

#[async_trait]
impl<R: Runtime> StateUpdaterFunctions for Updaters<R> {
    fn update_rooms_list(&self, rooms_list: &RoomsList) -> anyhow::Result<()> {
        let json = serde_json::to_value(rooms_list).expect("Couldn't serialize Rooms List");
        #[cfg(target_os = "ios")]
        self.sync_ios_badge(&json);
        let mut empty_state = StoreState::new();
        let state = match json {
            Value::Object(map) => {
                let hashmap: HashMap<String, Value> = map.into_iter().collect();
                empty_state.patch(hashmap);
                Ok(empty_state)
            }
            _ => Err(anyhow!(
                "Unexpected JSON object received during serialization"
            )),
        };
        self.app_handle.svelte().patch(
            ROOMS_COLLECTION_STORE_ID,
            state.expect("Wrong state sent to frontend"),
        )?;
        Ok(())
    }
    fn update_room(&self, room: &RoomScreen) -> anyhow::Result<()> {
        let json = serde_json::to_value(room).expect("Couldn't serialize Rooms List");
        let mut empty_state = StoreState::new();
        let state = match json {
            Value::Object(map) => {
                let hashmap: HashMap<String, Value> = map.into_iter().collect();
                empty_state.patch(hashmap);
                Ok(empty_state)
            }
            _ => Err(anyhow!(
                "Unexpected JSON object received during serialization"
            )),
        };

        self.app_handle
            .svelte()
            .patch(ROOM_STORE_ID, state.expect("Wrong state sent to frontend"))?;
        Ok(())
    }
    fn update_sync_service(
        &self,
        sync_service_state: FrontendSyncServiceState,
    ) -> anyhow::Result<()> {
        self.app_handle.svelte().set(
            LOGIN_STATE_STORE_ID,
            "syncServiceState",
            sync_service_state.to_camel_case(),
        )?;
        Ok(())
    }
    fn update_login_state(
        &self,
        login_state: LoginState,
        user_id: Option<String>,
    ) -> anyhow::Result<()> {
        self.app_handle
            .svelte()
            .set(LOGIN_STATE_STORE_ID, "state", login_state.to_camel_case())?;
        self.app_handle
            .svelte()
            .set(LOGIN_STATE_STORE_ID, "userId", user_id)?;
        Ok(())
    }

    fn update_verification_state(
        &self,
        verification_state: FrontendVerificationState,
    ) -> anyhow::Result<()> {
        self.app_handle.svelte().set(
            LOGIN_STATE_STORE_ID,
            "verificationState",
            verification_state.to_camel_case(),
        )?;
        Ok(())
    }

    fn update_recovery_state(&self, recovery_state: RecoveryState) -> anyhow::Result<()> {
        let serialized: &str = match recovery_state {
            RecoveryState::Enabled => "enabled",
            RecoveryState::Disabled => "disabled",
            RecoveryState::Incomplete => "incomplete",
            RecoveryState::Unknown => "unknown",
        };
        self.app_handle
            .svelte()
            .set(LOGIN_STATE_STORE_ID, "recoveryState", serialized)?;
        Ok(())
    }

    fn update_current_user_info(
        &self,
        current_user_id: Option<OwnedUserId>,
        avatar: Option<OwnedMxcUri>,
        user_display_name: Option<String>,
        device_display_name: Option<String>,
    ) -> anyhow::Result<()> {
        if let Some(user_id) = current_user_id {
            self.app_handle
                .svelte()
                .set(LOGIN_STATE_STORE_ID, "userId", user_id.to_string())?;
        }
        if let Some(avatar_uri) = avatar {
            self.app_handle.svelte().set(
                LOGIN_STATE_STORE_ID,
                "userAvatar",
                serde_json::to_value(avatar_uri).expect("Couldn't serialize MXC URI to value"),
            )?;
        }
        if let Some(username) = user_display_name {
            self.app_handle
                .svelte()
                .set(LOGIN_STATE_STORE_ID, "userDisplayName", username)?;
        }
        if let Some(device_name) = device_display_name {
            self.app_handle
                .svelte()
                .set(LOGIN_STATE_STORE_ID, "deviceDisplayName", device_name)?;
        }
        Ok(())
    }

    async fn persist_refreshed_session(
        &self,
        refreshed_session: AuthSession,
    ) -> anyhow::Result<()> {
        let app_data_dir =
            get_app_dir_or_create_it(&self.app_handle).expect("app data dir should be defined");
        let serialized_session = get_matrix_session_option(app_data_dir.clone()).ok_or(anyhow!(
            "We should be able to get previous session since the user is logged in"
        ))?;
        let mut session = serde_json::from_str::<FullMatrixSession>(&serialized_session)?;

        session.user_session = refreshed_session.into();

        let serialized_session = serde_json::to_string(&session)?;

        set_session_in_keyring(serialized_session.into_bytes(), app_data_dir)?;
        Ok(())
    }

    async fn persist_login_session(&self, session: String) -> anyhow::Result<()> {
        let app_data_dir =
            get_app_dir_or_create_it(&self.app_handle).expect("app data dir should be defined");
        set_session_in_keyring(session.into_bytes(), app_data_dir)?;
        Ok(())
    }
}

impl<R: Runtime> StateUpdater for Updaters<R> {}
