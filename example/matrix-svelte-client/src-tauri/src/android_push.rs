//! Android-only background silent-push handling.
//!
//! When the app is killed, Firebase cold-starts the process and runs the
//! notifications plugin's messaging service *without* the Tauri runtime. The
//! plugin's `SilentPushHandler` (see `DemoSilentPushHandler.kt`) then calls the
//! JNI entry point below to "fetch" the notification content. There is no Tauri
//! `AppHandle` in this state, so we cannot use the plugin builder — we just
//! return the content as JSON and let Kotlin post the notification.
//!
//! In a real Matrix client, [`simulate_matrix_fetch`] is where
//! `matrix_sdk::NotificationClient` would load and decrypt the event from the
//! on-disk store the main app shares.

// The shared helpers are `pub(crate)` for use from `lib.rs`; this module is
// private, so clippy flags that as redundant — it isn't, the parent needs them.
#![allow(clippy::redundant_pub_crate)]

use base64::Engine;
use std::collections::HashMap;
use tauri_plugin_matrix_svelte::FrontendNotificationStatus;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

/// Pretend to fetch the event body from a homeserver. Returns `(sender, body, summary, room_display_name, is_dm, sender_avatar_url)`.
///
/// Shared by the warm path (`process_silent_push` in `lib.rs`) and the killed
/// path (the JNI entry below). The body is intentionally long so the expandable
/// `MessagingStyle` notification has something to show.
pub(crate) async fn simulate_matrix_fetch(
    data_dir: String,
    room_id: String,
    event_id: String,
) -> (String, String, String, String, bool, Option<String>) {
    let mut message = (
        "Alice".to_string(),
        format!("Nouveau message {data_dir} in {room_id} (event {event_id})"),
        format!("Summary"),
        format!("Test room"),
        true,
        Some(format!("mxc://")),
    );
    if let Ok(result) =
        tauri_plugin_matrix_svelte::handle_silent_notification(data_dir, room_id, event_id).await
        && let FrontendNotificationStatus::Event(item) = result.status
    {
        message.0 = item
            .sender_display_name
            .unwrap_or(item.room_display_name.clone());
        message.1 = item.body.unwrap_or(item.summary.clone());
        message.2 = item.summary;
        message.3 = item.room_display_name;
        message.4 = item.is_dm;
        message.5 = item.sender_avatar_url;
    };
    message
}

/// Base64-encoded demo avatar. Stands in for the bytes a real client gets from
/// matrix-sdk's media store after downloading the sender/room `mxc://` avatar;
/// here we just reuse the app icon so no extra asset is committed.
pub(crate) fn demo_avatar_base64() -> String {
    const AVATAR_PNG: &[u8] = include_bytes!("../icons/testavatar.png");
    base64::engine::general_purpose::STANDARD.encode(AVATAR_PNG)
}

/// Derive a stable, positive notification id from a conversation key (the room
/// id). Using the room as the key means every message in that room lands in the
/// same notification, so the plugin accumulates them into one conversation
/// instead of posting a separate notification per event.
pub(crate) fn notification_id_for(key: &str) -> i32 {
    let hash = key.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    }) & 0x7fff_ffff;
    i32::try_from(hash).unwrap_or(0)
}

/// JNI entry: `com.matrix.svelte.client.SilentPushBridge.nativeProcessSilentPush(String, String): String`.
///
/// Inputs are the app data directory path and the FCM data payload as a JSON
/// object (string → string). Output is the notification content as JSON
/// (`id`, `channelId`, `conversationTitle`, `selfName`, and a `messages` array of
/// `{ sender, personKey, text, timestamp, avatarBytes }`) for Kotlin to post, or
/// `null` on failure.
///
/// `data_dir` is the app's data directory (the same path Tauri's path API
/// resolves to on Android); a real client opens its on-disk store (e.g. the
/// Matrix SDK database) under it to decrypt the event.
///
/// # Safety
/// Called by the JVM with valid JNI references; not invoked from Rust.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_matrix_svelte_client_SilentPushBridge_nativeProcessSilentPush<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
    data_json: JString<'local>,
) -> jstring {
    match process(&mut env, &data_dir, &data_json) {
        Ok(json) => env
            .new_string(json)
            .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw),
        Err(e) => {
            tracing::error!("nativeProcessSilentPush failed: {e}");
            std::ptr::null_mut()
        }
    }
}

fn process(env: &mut JNIEnv, data_dir: &JString, data_json: &JString) -> Result<String, String> {
    let data_dir: String = env
        .get_string(data_dir)
        .map_err(|e| format!("reading dataDir JString: {e}"))?
        .into();
    let input: String = env
        .get_string(data_json)
        .map_err(|e| format!("reading JString: {e}"))?
        .into();
    let data: HashMap<String, String> =
        serde_json::from_str(&input).map_err(|e| format!("parsing data JSON: {e}"))?;

    let room_id = data
        .get("room_id")
        .cloned()
        .unwrap_or_else(|| "!unknown:matrix.org".to_string());
    let event_id = data
        .get("event_id")
        .cloned()
        .unwrap_or_else(|| "$unknown".to_string());

    tracing::info!(
        "silent push (background/JNI): fetching {event_id} in {room_id} (data dir: {data_dir})"
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("building runtime: {e}"))?;
    let notif_id = notification_id_for(&room_id);
    let (sender, body, summary, room_display_name, is_dm, sender_avatar_url) = runtime
        .block_on(async { simulate_matrix_fetch(data_dir, room_id, event_id.clone()).await });

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0));

    // MessagingStyle: the plugin decodes `avatarBytes` and renders a chat-style
    // notification with the sender's circular avatar and the room as the title.
    // The id is keyed by the room, and `appendMessages` lets the plugin stack
    // each new event onto the same conversation notification.
    let out = serde_json::json!({
        "id": notif_id,
        "channelId": "default",
        "title": summary,
        "body": body,
        "conversationTitle": room_display_name,
        "groupConversation": !is_dm,
        "selfName": "Me",
        "appendMessages": true,
        "autoCancel": true,
        "messages": [{
            "sender": sender,
            "personKey": sender,
            "text": body,
            "timestamp": now_ms,
            "avatarBytes": demo_avatar_base64(),
        }],
    });
    Ok(out.to_string())
}
