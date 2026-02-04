use matrix_ui_serializable::{
    EventReceivers,
    models::{
        event_bridge::broadcast,
        events::{
            EmitEvent, MatrixLoginPayload, MatrixRoomStoreCreatedRequest,
            MatrixUpdateCurrentActiveRoom, MatrixVerificationResponse,
        },
    },
};
use tauri::{AppHandle, Emitter, Listener, Runtime};
use tauri_plugin_notifications::NotificationsExt;
use url::Url;

use crate::{AUTH_DEEPLINK_SENDER, LOGIN_SENDER};

// Outgoing events (lib -> tauri -> frontend)

pub async fn event_forwarder<R: Runtime>(
    app_handle: AppHandle<R>,
    receiver: broadcast::Receiver<EmitEvent>,
) -> anyhow::Result<()> {
    while let Ok(event) = receiver.resubscribe().recv().await {
        match event {
            EmitEvent::RoomCreate(e) => {
                app_handle.emit("matrix-svelte://room-create", e)?;
            }
            EmitEvent::VerificationStart(e) => {
                app_handle.emit("matrix-svelte://verification-start", e)?;
            }
            EmitEvent::ToastNotification(e) => {
                app_handle.emit("matrix-svelte://toast-notification", e)?;
            }
            EmitEvent::OsNotification(e) => match e.body {
                Some(body) => {
                    app_handle
                        .notifications()
                        .builder()
                        .title(e.summary)
                        .body(body)
                        .show()
                        .await?
                }
                None => {
                    app_handle
                        .notifications()
                        .builder()
                        .title(e.summary)
                        .show()
                        .await?
                }
            },
            EmitEvent::OAuthUrl(e) => {
                app_handle.emit("matrix-svelte://oauth-url", e)?;
            }
            EmitEvent::ResetCrossSigngingUrl(url) => {
                app_handle.emit("matrix-svelte://reset-cross-signing-url", url)?;
            }
            EmitEvent::NewlyCreatedRoomId(room_id) => {
                app_handle.emit("matrix-svelte://newly-created-room-id", room_id)?;
            }
        }
    }
    Ok(())
}

// Incoming events (lib <- tauri <- frontend)

const DEFAULT_BUFFER_SIZE: usize = 20;

pub fn handle_incoming_events<R: Runtime>(app_handle: &AppHandle<R>) -> EventReceivers {
    // Event based
    let (tx_room_created, rx_room_created) =
        tauri::async_runtime::channel::<MatrixRoomStoreCreatedRequest>(DEFAULT_BUFFER_SIZE);
    let (tx_verif, rx_verif) =
        tauri::async_runtime::channel::<MatrixVerificationResponse>(DEFAULT_BUFFER_SIZE);
    let (tx_update_room, rx_update_room) =
        tauri::async_runtime::channel::<MatrixUpdateCurrentActiveRoom>(DEFAULT_BUFFER_SIZE);

    let room_created_handle = app_handle.clone();
    // The listeners should be alive for the entire lifetime of the app
    room_created_handle.listen("matrix-svelte://room-created", move |e| {
        if let Ok(payload) = serde_json::from_str::<MatrixRoomStoreCreatedRequest>(e.payload()) {
            futures::executor::block_on(tx_room_created.send(payload))
                .expect("Couldn't forward event to lib");
        }
    });
    room_created_handle.listen("matrix-svelte://verification-result", move |e| {
        if let Ok(payload) = serde_json::from_str::<MatrixVerificationResponse>(e.payload()) {
            futures::executor::block_on(tx_verif.send(payload))
                .expect("Couldn't forward event to lib");
        }
    });
    room_created_handle.listen("matrix-svelte://update-current-active-room", move |e| {
        if let Ok(payload) = serde_json::from_str::<MatrixUpdateCurrentActiveRoom>(e.payload()) {
            futures::executor::block_on(tx_update_room.send(payload))
                .expect("Couldn't forward event to lib");
        }
    });

    // Command based
    let (tx_matrix_login, rx_matrix_login) = tauri::async_runtime::channel::<MatrixLoginPayload>(1);
    let (tx_oauth_deeplink, rx_oauth_deeplink) = tauri::async_runtime::channel::<Url>(1);

    LOGIN_SENDER
        .set(tx_matrix_login)
        .expect("login sender already set");

    AUTH_DEEPLINK_SENDER
        .set(tx_oauth_deeplink)
        .expect("oauth deeplink sender already set");

    EventReceivers::new(
        rx_room_created,
        rx_verif,
        rx_update_room,
        rx_matrix_login,
        rx_oauth_deeplink,
    )
}
