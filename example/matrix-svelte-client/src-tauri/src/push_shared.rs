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

/// Fetch the pushed event through the plugin's cold-path API and shape it for
/// display. Returns `(sender, body, summary, room_display_name, is_dm,
/// sender_avatar_base64, room_avatar_base64)`. The room avatar is only fetched
/// for group rooms (`is_dm == false`), where the notification brands as the
/// room; for DMs it is always `None`.
///
/// `data_dir` is the directory holding the Matrix store and salt file: the app
/// data dir on Android, the shared App Group container on iOS. On any failure
/// a placeholder message is returned so the notification still shows something
/// diagnosable.
pub(crate) async fn fetch_notification_event(
    data_dir: String,
    room_id: String,
    event_id: String,
) -> (
    String,
    String,
    String,
    String,
    bool,
    Option<String>,
    Option<String>,
) {
    let mut message = (
        "Alice".to_string(),
        format!("Nouveau message {data_dir} in {room_id} (event {event_id})"),
        format!("Summary"),
        format!("Test room"),
        true,
        None,
        None,
    );
    // Explicitly log *why* we fall back to the placeholder so the cold path is
    // debuggable (logcat on Android, Console on iOS): an `Err` means the
    // fetch itself failed (e.g. keyring/session not initialized), while a
    // non-`Event` status (`NotFound`, …) means the event couldn't be resolved.
    match tauri_plugin_matrix_svelte::handle_silent_notification(data_dir, room_id, event_id).await
    {
        Ok(status) => match status {
            FrontendNotificationStatus::Event(item) => {
                tracing::info!("silent notification: resolved event, building real message");
                message.0 = item
                    .sender_display_name
                    .unwrap_or(item.room_display_name.clone());
                message.1 = item.body.unwrap_or(item.summary.clone());
                message.2 = item.summary;
                message.3 = item.room_display_name;
                message.4 = item.is_dm;
                message.5 = item
                    .sender_avatar
                    .map(|buffer| base64::engine::general_purpose::STANDARD.encode(buffer));
                message.6 = item
                    .room_avatar
                    .map(|buffer| base64::engine::general_purpose::STANDARD.encode(buffer));
            }
            other => {
                tracing::warn!("silent notification fell back to placeholder: status = {other:?}");
            }
        },
        Err(e) => {
            tracing::error!("silent notification fetch failed, using placeholder: {e}");
        }
    };
    message
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
