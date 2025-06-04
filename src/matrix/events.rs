use matrix_sdk::{
    encryption::VerificationState,
    ruma::{
        events::{
            key::verification::request::ToDeviceKeyVerificationRequestEvent,
            room::message::{MessageType, OriginalSyncRoomMessageEvent},
        },
        MilliSecondsSinceUnixEpoch, OwnedRoomId,
    },
    Client,
};
use matrix_sdk_ui::timeline::EventTimelineItem;
use tauri::{AppHandle, Runtime};

use super::{
    emoji_verification::request_verification_handler, event_preview::text_preview_of_timeline_item,
};

// These event handlers handle only the verification events. Other events are managed by the matrix_sdk_ui sync service.
pub fn add_event_handlers<R: Runtime>(
    client: Client,
    app_handle: &AppHandle<R>,
) -> anyhow::Result<Client> {
    let first_app_handle = app_handle.clone();

    let mut verification_state_subscriber = client.encryption().verification_state();
    println!(
        "Initial verification state is {:?}",
        verification_state_subscriber.get()
    );
    tauri::async_runtime::spawn(async move {
        while let Some(state) = verification_state_subscriber.next().await {
            println!("Received a verification state update: {state:?}");
            // Cx::post_action(VerificationStateAction::Update(state));
            // TODO: display verification state
            if let VerificationState::Verified = state {
                break;
            }
        }
    });
    client.add_event_handler(
        |ev: ToDeviceKeyVerificationRequestEvent, client: Client| async move {
            if let Some(request) = client
                .encryption()
                .get_verification_request(&ev.sender, &ev.content.transaction_id)
                .await
            {
                tauri::async_runtime::spawn(request_verification_handler(
                            client,
                            request,
                            first_app_handle,
                        ));
            }
            else {
                eprintln!("Skipping invalid verification request from {}, transaction ID: {}\n   Content: {:?}",
                    ev.sender, ev.content.transaction_id, ev.content,
                );
            }
        },
    );

    let second_app_handle = app_handle.clone();
    client.add_event_handler(
        |ev: OriginalSyncRoomMessageEvent, client: Client| async move {
            if let MessageType::VerificationRequest(_) = &ev.content.msgtype {
                if let Some(request) = client
                    .encryption()
                    .get_verification_request(&ev.sender, &ev.event_id)
                    .await
                {
                    tauri::async_runtime::spawn(request_verification_handler(
                                client,
                                request,
                                second_app_handle,
                            ));
                }
                else {
                    eprintln!("Skipping invalid verification request from {}, event ID: {}\n   Content: {:?}",
                        ev.sender, ev.event_id, ev.content,
                    );
                }
            }
        }
    );
    Ok(client)
}

/// Returns the timestamp and text preview of the given `latest_event` timeline item.
///
/// If the sender profile of the event is not yet available, this function will
/// generate a preview using the sender's user ID instead of their display name,
/// and will submit a background async request to fetch the details for this event.
pub fn get_latest_event_details(
    latest_event: &EventTimelineItem,
    room_id: &OwnedRoomId,
) -> (MilliSecondsSinceUnixEpoch, String) {
    #[cfg(debug_assertions)]
    println!("Formating event coming from: {:?}", latest_event.sender());

    let sender_username = super::utils::get_or_fetch_event_sender(latest_event, Some(room_id));
    (
        latest_event.timestamp(),
        text_preview_of_timeline_item(latest_event.content(), &sender_username)
            .format_with(&sender_username, true),
    )
}
