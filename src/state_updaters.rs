use std::collections::HashMap;

use anyhow::anyhow;
use matrix_ui_serializable::{
    FrontendSyncServiceState, FrontendVerificationState, LoginState, RoomScreen, RoomsList,
    UserProfileMap,
    models::state_updater::{StateUpdater, StateUpdaterFunctions},
};
use serde_json::Value;
use tauri::{AppHandle, Runtime};
use tauri_plugin_svelte::{ManagerExt, StoreState};

// Keep the same ids as in the JS package !
const ROOMS_COLLECTION_STORE_ID: &str = "rooms-collection";
pub const LOGIN_STATE_STORE_ID: &str = "login-state";
const PROFILES_STORE_ID: &str = "profiles";

#[derive(Debug)]
pub struct Updaters<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> Updaters<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<R: Runtime> StateUpdaterFunctions for Updaters<R> {
    fn update_rooms_list(&self, rooms_list: &RoomsList) -> anyhow::Result<()> {
        let json = serde_json::to_value(rooms_list).expect("Couldn't serialize Rooms List");
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
    fn update_room(&self, room: &RoomScreen, room_id: &str) -> anyhow::Result<()> {
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
            .patch(room_id, state.expect("Wrong state sent to frontend"))?;
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
    fn update_profile(&self, user_profiles: &UserProfileMap) -> anyhow::Result<()> {
        let json = serde_json::to_value(user_profiles)?;
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
            PROFILES_STORE_ID,
            state.expect("Wrong state sent to frontend"),
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
}

impl<R: Runtime> StateUpdater for Updaters<R> {}
