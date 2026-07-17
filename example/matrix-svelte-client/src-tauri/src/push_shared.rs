//! Platform-agnostic silent-push helpers shared by every entry point that
//! turns a Matrix push into notification content: Android's warm handler and
//! killed-state JNI entry (`android_push.rs`), and iOS's Notification Service
//! Extension entry (`ios_push.rs`).

// The shared helpers are `pub(crate)` for use from the sibling modules; this
// module is private, so clippy flags that as redundant — it isn't, the
// siblings need them.
#![allow(clippy::redundant_pub_crate)]

use base64::Engine;
use tauri_plugin_matrix_svelte::FrontendNotificationStatus;

/// Displayable content of a pushed Matrix event, shared by the Android and
/// iOS notification builders. Avatars are base64-encoded image bytes; the
/// room avatar is only fetched for group rooms (`is_dm == false`), where the
/// notification brands as the room — for DMs it is always `None`.
pub(crate) struct NotificationContent {
    pub(crate) sender: String,
    pub(crate) body: String,
    pub(crate) summary: String,
    pub(crate) room_display_name: String,
    pub(crate) is_dm: bool,
    pub(crate) sender_avatar: Option<String>,
    pub(crate) room_avatar: Option<String>,
}

impl NotificationContent {
    /// Neutral fallback shown when the event can't be fetched/decrypted. The
    /// actual failure cause is logged, not leaked into the notification.
    fn placeholder() -> Self {
        Self {
            sender: "Matrix".to_string(),
            body: "New message".to_string(),
            summary: "New message".to_string(),
            room_display_name: "Matrix".to_string(),
            is_dm: true,
            sender_avatar: None,
            room_avatar: None,
        }
    }
}

/// Fetch the pushed event through the plugin's cold-path API and shape it for
/// display.
///
/// `data_dir` is the directory holding the Matrix store and salt file: the app
/// data dir on Android, the shared App Group container on iOS. On any failure
/// a placeholder message is returned so the notification still shows something.
pub(crate) async fn fetch_notification_event(
    data_dir: String,
    room_id: String,
    event_id: String,
) -> NotificationContent {
    // Explicitly log *why* we fall back to the placeholder so the cold path is
    // debuggable (logcat on Android, Console on iOS): an `Err` means the
    // fetch itself failed (e.g. keyring/session not initialized), while a
    // non-`Event` status (`NotFound`, …) means the event couldn't be resolved.
    match tauri_plugin_matrix_svelte::handle_silent_notification(data_dir, room_id, event_id).await
    {
        Ok(FrontendNotificationStatus::Event(item)) => {
            tracing::info!("silent notification: resolved event, building real message");
            let encode = |buffer| base64::engine::general_purpose::STANDARD.encode(buffer);
            NotificationContent {
                sender: item
                    .sender_display_name
                    .unwrap_or(item.room_display_name.clone()),
                body: item.body.unwrap_or(item.summary.clone()),
                summary: item.summary,
                room_display_name: item.room_display_name,
                is_dm: item.is_dm,
                sender_avatar: item.sender_avatar.map(encode),
                room_avatar: item.room_avatar.map(encode),
            }
        }
        Ok(other) => {
            tracing::warn!("silent notification fell back to placeholder: status = {other:?}");
            NotificationContent::placeholder()
        }
        Err(e) => {
            tracing::error!("silent notification fetch failed, using placeholder: {e}");
            NotificationContent::placeholder()
        }
    }
}

/// Build the canonical Matrix URI (MSC2312) for an event in a room, e.g.
/// `matrix:roomid/abc:matrix.org/e/xyz` from `!abc:matrix.org` / `$xyz`.
/// Sigils (`!`/`$`) are dropped; the spec keeps `:` literal in the path.
///
/// On Android a notification tap fires `ACTION_VIEW` for this URI (see
/// `android_push.rs`); on iOS it rides in the notification's `userInfo` (via
/// `extra`) and reaches JS through the `notificationClicked` event.
pub(crate) fn matrix_uri(room_id: &str, event_id: &str) -> String {
    let room = room_id.strip_prefix('!').unwrap_or(room_id);
    let event = event_id.strip_prefix('$').unwrap_or(event_id);
    format!("matrix:roomid/{room}/e/{event}")
}
