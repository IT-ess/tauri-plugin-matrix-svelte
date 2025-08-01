use std::time::SystemTime;

use crossbeam_queue::SegQueue;
use matrix_sdk::{
    deserialized_responses::RawAnySyncOrStrippedTimelineEvent,
    notification_settings::{IsEncrypted, IsOneToOne, NotificationSettings, RoomNotificationMode},
    ruma::{
        api::client::push::{Pusher, PusherIds, PusherInit, PusherKind},
        events::{room::message::MessageType, AnyMessageLikeEventContent, AnySyncTimelineEvent},
        push::HttpPusherData,
        serde::Raw,
        MilliSecondsSinceUnixEpoch,
    },
    sync::Notification,
    Client, Room,
};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    matrix::singletons::{broadcast_event, UIUpdateMessage},
    models::matrix::{MatrixSvelteEmitEvent, ToastNotificationRequest},
    notifications::GetTokenRequest,
    utils::config::get_plugin_config,
};

//
// TOAST Notifications (managed by Webview)
//

static TOAST_NOTIFICATION: SegQueue<ToastNotificationRequest> = SegQueue::new();

/// Displays a new toast notification with the given message.
///
/// Toast notifications will be shown in the order they were enqueued.
pub fn enqueue_toast_notification(notification: ToastNotificationRequest) {
    TOAST_NOTIFICATION.push(notification);
    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
}

pub fn process_toast_notifications<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<()> {
    if TOAST_NOTIFICATION.is_empty() {
        return Ok(());
    };
    while let Some(notif) = TOAST_NOTIFICATION.pop() {
        app_handle.emit(MatrixSvelteEmitEvent::ToastNotification.as_str(), notif)?;
    }
    Ok(())
}

//
// OS Notifications (push notifications for mobiles)
//

pub async fn register_notifications<R: Runtime>(app_handle: &AppHandle<R>, client: &Client) {
    #[cfg(mobile)]
    register_mobile_push_notifications(&app_handle, &client).await;
    #[cfg(desktop)]
    register_desktop_notifications(&app_handle, &client).await;
}

pub async fn register_mobile_push_notifications<R: Runtime>(
    app_handle: &AppHandle<R>,
    client: &Client,
) {
    let plugin_config = get_plugin_config(&app_handle).expect("The plugin config is not defined !");

    let http_pusher = HttpPusherData::new(plugin_config.sygnal_gateway_url);

    use crate::MatrixSvelteExt;
    let push_token = app_handle
        .matrix_svelte()
        .get_token(GetTokenRequest {})
        .unwrap();

    println!("Push token: {:?}", push_token);

    let identifier = app_handle.config().identifier.clone();
    #[cfg(target_os = "android")]
    let identifier = identifier.replace("-", "_"); // On android, - are replaced by _ in bundle names
    let pusher_ids = PusherIds::new(push_token.token, identifier);

    let pusher = PusherInit {
        ids: pusher_ids,
        app_display_name: "Matrix Svelte Client".to_string(),
        device_display_name: "Tauri app".to_string(),
        profile_tag: None,
        kind: PusherKind::Http(http_pusher),
        lang: "en".to_string(),
    };

    let pusher: Pusher = pusher.into();

    let _ = client
        .pusher()
        .set(pusher)
        .await
        .expect("Couldn't set the notification pusher correcly");
}

pub async fn register_desktop_notifications<R: Runtime>(
    app_handle: &AppHandle<R>,
    client: &Client,
) {
    let server_settings = client.notification_settings().await;
    let Some(startup_ts) = MilliSecondsSinceUnixEpoch::from_system_time(SystemTime::now()) else {
        return;
    };

    let notif_handler_app_handle = app_handle.clone();

    client
        .register_notification_handler(
            move |notification: Notification, room: Room, client: Client| {
                let server_settings = server_settings.clone();
                let inner_app_handle = notif_handler_app_handle.clone();
                async move {
                    let mode = global_or_room_mode(&server_settings, &room).await;
                    if mode == RoomNotificationMode::Mute {
                        return;
                    }

                    match notification.event {
                        RawAnySyncOrStrippedTimelineEvent::Sync(e) => {
                            match parse_full_notification(e, room, true).await {
                                Ok((summary, body, server_ts)) => {
                                    if server_ts < startup_ts {
                                        return;
                                    }

                                    if is_missing_mention(&body, mode, &client) {
                                        return;
                                    }

                                    send_notification(&inner_app_handle, &summary, body.as_deref())
                                        .await;
                                }
                                Err(err) => {
                                    eprintln!("Failed to extract notification data: {err}")
                                }
                            }
                        }
                        // Stripped events may be dropped silently because they're
                        // only relevant if we're not in a room, and we presumably
                        // don't want notifications for rooms we're not in.
                        RawAnySyncOrStrippedTimelineEvent::Stripped(_) => (),
                    }
                }
            },
        )
        .await;
}

pub async fn global_or_room_mode(
    settings: &NotificationSettings,
    room: &Room,
) -> RoomNotificationMode {
    let room_mode = settings
        .get_user_defined_room_notification_mode(room.room_id())
        .await;
    if let Some(mode) = room_mode {
        return mode;
    }
    let is_one_to_one = match room.is_direct().await {
        Ok(true) => IsOneToOne::Yes,
        _ => IsOneToOne::No,
    };
    let is_encrypted = match room.encryption_state().is_encrypted() {
        true => IsEncrypted::Yes,
        false => IsEncrypted::No,
    };
    settings
        .get_default_room_notification_mode(is_encrypted, is_one_to_one)
        .await
}

async fn send_notification<R: Runtime>(
    app_handle: &AppHandle<R>,
    summary: &str,
    body: Option<&str>,
) {
    match body {
        Some(body) => app_handle
            .notification()
            .builder()
            .title(summary)
            .body(body)
            .show()
            .unwrap(),
        None => app_handle
            .notification()
            .builder()
            .title(summary)
            .show()
            .unwrap(),
    }
}

fn is_missing_mention(body: &Option<String>, mode: RoomNotificationMode, client: &Client) -> bool {
    if let Some(body) = body {
        if mode == RoomNotificationMode::MentionsAndKeywordsOnly {
            let mentioned = match client.user_id() {
                Some(user_id) => body.contains(user_id.localpart()),
                _ => false,
            };
            return !mentioned;
        }
    }
    false
}

pub async fn parse_full_notification(
    event: Raw<AnySyncTimelineEvent>,
    room: Room,
    show_body: bool,
) -> anyhow::Result<(String, Option<String>, MilliSecondsSinceUnixEpoch)> {
    let event = event.deserialize().map_err(anyhow::Error::from)?;

    let server_ts = event.origin_server_ts();

    let sender_id = event.sender();
    let sender = room
        .get_member_no_sync(sender_id)
        .await
        .map_err(anyhow::Error::from)?;

    let sender_name = sender
        .as_ref()
        .and_then(|m| m.display_name())
        .unwrap_or_else(|| sender_id.localpart());

    let summary = if let Some(room_name) = room.cached_display_name() {
        if room.is_direct().await.map_err(anyhow::Error::from)?
            && sender_name == room_name.to_string()
        {
            sender_name.to_string()
        } else {
            format!("{sender_name} in {room_name}")
        }
    } else {
        sender_name.to_string()
    };

    let body = if show_body {
        event_notification_body(&event, sender_name).map(truncate)
    } else {
        None
    };

    return Ok((summary, body, server_ts));
}

pub fn event_notification_body(event: &AnySyncTimelineEvent, sender_name: &str) -> Option<String> {
    let AnySyncTimelineEvent::MessageLike(event) = event else {
        return None;
    };

    match event.original_content()? {
        AnyMessageLikeEventContent::RoomMessage(message) => {
            let body = match message.msgtype {
                MessageType::Audio(_) => {
                    format!("{sender_name} sent an audio file.")
                }
                MessageType::Emote(content) => content.body,
                MessageType::File(_) => {
                    format!("{sender_name} sent a file.")
                }
                MessageType::Image(_) => {
                    format!("{sender_name} sent an image.")
                }
                MessageType::Location(_) => {
                    format!("{sender_name} sent their location.")
                }
                MessageType::Notice(content) => content.body,
                MessageType::ServerNotice(content) => content.body,
                MessageType::Text(content) => content.body,
                MessageType::Video(_) => {
                    format!("{sender_name} sent a video.")
                }
                MessageType::VerificationRequest(_) => {
                    format!("{sender_name} sent a verification request.")
                }
                _ => {
                    format!("[Unknown message type: {:?}]", &message.msgtype)
                }
            };
            Some(body)
        }
        AnyMessageLikeEventContent::Sticker(_) => Some(format!("{sender_name} sent a sticker.")),
        _ => None,
    }
}

fn truncate(s: String) -> String {
    static MAX_LENGTH: usize = 5000;
    if s.graphemes(true).count() > MAX_LENGTH {
        let truncated: String = s.graphemes(true).take(MAX_LENGTH).collect();
        truncated + "..."
    } else {
        s
    }
}
