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

use std::collections::HashMap;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

/// Pretend to fetch the event body from a homeserver. Returns `(sender, body)`.
///
/// Shared by the warm path (`process_silent_push` in `lib.rs`) and the killed
/// path (the JNI entry below).
pub(crate) fn simulate_matrix_fetch(room_id: &str, event_id: &str) -> (String, String) {
    (
        "Alice".to_string(),
        format!("New message in {room_id} (event {event_id})"),
    )
}

/// Derive a stable, positive notification id from an event id, so re-delivery of
/// the same event updates rather than stacks.
pub(crate) fn notification_id_for(event_id: &str) -> i32 {
    let hash = event_id.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    }) & 0x7fff_ffff;
    i32::try_from(hash).unwrap_or(0)
}

/// JNI entry: `com.matrix.svelte.client.SilentPushBridge.nativeProcessSilentPush(String): String`.
///
/// Input is the FCM data payload as a JSON object (string → string). Output is
/// JSON `{ "id", "title", "body", "channelId" }` for Kotlin to post, or `null`
/// on failure.
///
/// # Safety
/// Called by the JVM with valid JNI references; not invoked from Rust.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_matrix_svelte_client_SilentPushBridge_nativeProcessSilentPush<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_json: JString<'local>,
) -> jstring {
    match process(&mut env, &data_json) {
        Ok(json) => env
            .new_string(json)
            .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw),
        Err(e) => {
            tracing::error!("nativeProcessSilentPush failed: {e}");
            std::ptr::null_mut()
        }
    }
}

fn process(env: &mut JNIEnv, data_json: &JString) -> Result<String, String> {
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

    tracing::info!("silent push (background/JNI): fetching {event_id} in {room_id}");

    let (sender, body) = simulate_matrix_fetch(&room_id, &event_id);

    let out = serde_json::json!({
        "id": notification_id_for(&event_id),
        "title": sender,
        "body": body,
        "channelId": "default",
    });
    Ok(out.to_string())
}
