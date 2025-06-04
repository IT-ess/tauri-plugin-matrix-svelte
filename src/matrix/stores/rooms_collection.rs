use matrix_sdk::ruma::OwnedRoomId;
use tauri::{AppHandle, Runtime};
use tauri_plugin_svelte::ManagerExt;

// Keep the same id as in JS package !
pub(crate) const ROOMS_COLLECTION_STORE_ID: &str = "rooms-collection";

// TODO: is this method useful ?
// pub fn update_current_active_room<R: Runtime>(
//     app_handle: &AppHandle<R>,
//     id: &Option<OwnedRoomId>,
// ) -> anyhow::Result<()> {
//     match id {
//         Some(id) => app_handle.svelte().set(
//             ROOMS_COLLECTION_STORE_ID,
//             "currentActiveRoom",
//             id.to_string(),
//         )?,
//         None => app_handle.svelte().set(
//             ROOMS_COLLECTION_STORE_ID,
//             "currentActiveRoom",
//             serde_json::Value::Null,
//         )?,
//     }
//     Ok(())
// }

// TODO: is this method useful ?
pub fn get_current_active_room<R: Runtime>(app_handle: &AppHandle<R>) -> Option<OwnedRoomId> {
    let value = app_handle
        .svelte()
        .get(ROOMS_COLLECTION_STORE_ID, "currentActiveRoom");
    match value {
        Some(json_room_id) => {
            if let Ok(room_id) = serde_json::from_value(json_room_id) {
                Some(room_id)
            } else {
                None
            }
        }
        None => None,
    }
}
