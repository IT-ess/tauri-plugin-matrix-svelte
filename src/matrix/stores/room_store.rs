use anyhow::anyhow;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Listener, Runtime};
use tauri_plugin_svelte::{ManagerExt, StoreState};
use tokio::sync::oneshot;

use crate::models::matrix::{
    MatrixRoomStoreCreateRequest, MatrixRoomStoreCreatedRequest, MatrixSvelteEmitEvent,
    MatrixSvelteListenEvent,
};

pub fn get_room_store_state<R: Runtime>(
    app_handle: &AppHandle<R>,
    id: &str,
) -> anyhow::Result<StoreState> {
    app_handle
        .svelte()
        .state(id)
        .map_err(|e| anyhow!("Couldn't get store state from id {id}. Error : {e}"))
}

pub fn patch_room_store_state_from_reference<R: Runtime>(
    app_handle: &AppHandle<R>,
    id: &str,
    store_state: &StoreState,
) -> anyhow::Result<()> {
    app_handle
        .svelte()
        .patch(id, store_state.clone())
        .map_err(|e| anyhow!("Couldn't patch store state from id {id}. Error : {e}"))
}

pub async fn send_room_creation_request_and_await_response<R: Runtime>(
    app_handle: &AppHandle<R>,
    id: &str,
) -> anyhow::Result<()> {
    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    // Set up the event listener before emiting create event
    // We do not check the returned id
    let cloned_room_id = id.to_string();
    let unlisten = app_handle.listen(
        MatrixSvelteListenEvent::RoomCreated.as_str(),
        move |event| {
            if let Ok(payload) =
                serde_json::from_str::<MatrixRoomStoreCreatedRequest>(event.payload())
            {
                // Check that we got the correct id
                if payload.id == cloned_room_id {
                    let tx_clone = tx.clone();

                    let mut tx_lock = futures::executor::block_on(tx_clone.lock());
                    if let Some(tx) = tx_lock.take() {
                        let _ = tx.send(payload.id);
                    }
                }
            }
        },
    );

    app_handle.emit(
        MatrixSvelteEmitEvent::RoomCreate.as_str(),
        MatrixRoomStoreCreateRequest::new(id.to_string()),
    )?;

    // Wait for the event or return an error if the channel is closed. TODO: add a timeout
    rx.await?;

    // Clean up the listener
    app_handle.unlisten(unlisten);

    Ok(())
}
