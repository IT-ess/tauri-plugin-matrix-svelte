//! Android-only background silent-push handling.
//!
//! When the app is killed, Firebase cold-starts the process and runs the
//! notifications plugin's messaging service *without* the Tauri runtime. The
//! plugin's `SilentPushHandler` (see `DemoSilentPushHandler.kt`) then calls the
//! JNI entry point below to "fetch" the notification content. There is no Tauri
//! `AppHandle` in this state, so we cannot use the plugin builder — we just
//! return the content as JSON and let Kotlin post the notification.
//!
//! In a real Matrix client, [`fetch_notification_event`] is where
//! `matrix_sdk::NotificationClient` would load and decrypt the event from the
//! on-disk store the main app shares.

// The shared helpers are `pub(crate)` for use from `lib.rs`; this module is
// private, so clippy flags that as redundant — it isn't, the parent needs them.
#![allow(clippy::redundant_pub_crate)]

use base64::Engine;
use std::collections::HashMap;
use tauri_plugin_matrix_svelte::FrontendNotificationStatus;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::jstring;

/// Pretend to fetch the event body from a homeserver. Returns `(sender, body, summary, room_display_name, is_dm, sender_avatar_url)`.
///
/// Shared by the warm path (`process_silent_push` in `lib.rs`) and the killed
/// path (the JNI entry below). The body is intentionally long so the expandable
/// `MessagingStyle` notification has something to show.
pub(crate) async fn fetch_notification_event(
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
    // Explicitly log *why* we fall back to the placeholder so the cold path is
    // debuggable in logcat (see `init_cold_path_logging`): an `Err` means the
    // fetch itself failed (e.g. keyring/session not initialized), while a
    // non-`Event` status (`NotFound`, …) means the event couldn't be resolved.
    match tauri_plugin_matrix_svelte::handle_silent_notification(data_dir, room_id, event_id).await {
        Ok(result) => match result.status {
            FrontendNotificationStatus::Event(item) => {
                tracing::info!("silent notification: resolved event, building real message");
                message.0 = item
                    .sender_display_name
                    .unwrap_or(item.room_display_name.clone());
                message.1 = item.body.unwrap_or(item.summary.clone());
                message.2 = item.summary;
                message.3 = item.room_display_name;
                message.4 = item.is_dm;
                message.5 = item.sender_avatar_url;
            }
            other => {
                tracing::warn!(
                    "silent notification fell back to placeholder: status = {other:?}"
                );
            }
        },
        Err(e) => {
            tracing::error!("silent notification fetch failed, using placeholder: {e}");
        }
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

/// JNI entry: `com.matrix.svelte.client.SilentPushBridge.nativeProcessSilentPush(Context, String, String): String`.
///
/// Inputs are an Android `Context`, the app data directory path, and the FCM
/// data payload as a JSON object (string → string). Output is the notification
/// content as JSON (`id`, `channelId`, `conversationTitle`, `selfName`, and a
/// `messages` array of `{ sender, personKey, text, timestamp, avatarBytes }`)
/// for Kotlin to post, or `null` on failure.
///
/// `context` is needed because the FCM service cold-starts the process *without*
/// the Tauri runtime, so none of the process-wide initializations the app does
/// at startup have run. Before fetching we replay the essential ones from this
/// `context` (NDK context, TLS platform verifier, keyring backend) — see
/// [`init_cold_path_context`].
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
    context: JObject<'local>,
    data_dir: JString<'local>,
    data_json: JString<'local>,
) -> jstring {
    // Route Rust `tracing` to logcat first, so everything below is visible
    // (view with e.g. `adb logcat -s MatrixSilentPush`).
    init_cold_path_logging();
    tracing::info!("nativeProcessSilentPush: cold-path entry");

    // Replay the startup initializations the Tauri runtime would normally do.
    // A failure here is logged but not fatal: `process` will simply fall back to
    // the placeholder message, which is exactly the symptom we want to surface.
    if let Err(e) = init_cold_path_context(&mut env, &context) {
        tracing::error!("cold-path context init failed: {e}");
    }

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

/// Install a `tracing` subscriber that writes to Android's logcat. Idempotent.
///
/// In the cold path there is no Tauri runtime, so the app's normal logging
/// setup never runs and every `tracing::*` call in this module would otherwise
/// be dropped. View the output with e.g. `adb logcat -s MatrixSilentPush`.
fn init_cold_path_logging() {
    use std::sync::Once;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::try_new(
            "info,matrix_svelte_client_lib=debug,matrix_ui_serializable=debug,matrix_sdk=warn",
        )
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
        // `try_init` fails only if a global subscriber is already set; ignore that.
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(paranoid_android::layer("MatrixSilentPush"))
            .try_init();
    });
}

/// Replay the process-wide initializations the Tauri runtime performs at startup
/// but which are skipped when the FCM service cold-starts the process:
///
/// 1. `ndk_context` — the Android keyring backend resolves its `Context` through
///    this global; without it, reading the saved Matrix session fails.
/// 2. `rustls-platform-verifier` — matrix-sdk fetches the event from the
///    homeserver over TLS, which needs the Android trust roots.
/// 3. the keyring backend (`init_keyring_store`) — sets the process-wide
///    `keyring_core` default store the session is read from.
///
/// All three are idempotent (guarded by `Once`/internally) so handling several
/// pushes in one process is safe. Mirrors `MainActivity.initNdkContext`
/// (`lib.rs`) and the matrix-svelte plugin `setup`.
fn init_cold_path_context(env: &mut JNIEnv, context: &JObject) -> Result<(), String> {
    use std::ffi::c_void;
    use std::sync::Once;

    static NDK_AND_TLS: Once = Once::new();
    let mut init_result: Result<(), String> = Ok(());
    NDK_AND_TLS.call_once(|| {
        init_result = (|| {
            let vm = env
                .get_java_vm()
                .map_err(|e| format!("getting JavaVM: {e}"))?;
            let context_ref = env
                .new_global_ref(context)
                .map_err(|e| format!("global-ref'ing context: {e}"))?;

            // 1. NDK context (used by the Android keyring backend).
            unsafe {
                ndk_context::initialize_android_context(
                    vm.get_java_vm_pointer().cast::<c_void>(),
                    context_ref.as_obj().as_raw().cast::<c_void>(),
                );
            }

            tracing::info!("cold-path: ndk_context initialized");

            // 2. TLS platform verifier (used by matrix-sdk's HTTP client).
            let loader = env
                .call_method(context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
                .and_then(|v| v.l())
                .map_err(|e| format!("getting ClassLoader: {e}"))?;
            let loader_ref = env
                .new_global_ref(&loader)
                .map_err(|e| format!("global-ref'ing ClassLoader: {e}"))?;
            rustls_platform_verifier::android::init_with_refs(vm, context_ref, loader_ref);

            tracing::info!("cold-path: rustls platform verifier initialized");
            Ok(())
        })();
    });
    init_result?;

    // 3. Keyring backend. Idempotent inside the plugin; cheap to call each time.
    tauri_plugin_matrix_svelte::init_keyring_store()
        .map_err(|e| format!("initializing keyring store: {e}"))?;
    tracing::info!("cold-path: keyring store initialized");

    Ok(())
}

/// Build the canonical Matrix URI (MSC2312) for an event in a room, e.g.
/// `matrix:roomid/abc:matrix.org/e/xyz` from `!abc:matrix.org` / `$xyz`. The
/// notification's tap fires `ACTION_VIEW` for this, which the app's `matrix:`
/// intent-filter routes to `tauri-plugin-deep-link`. Sigils (`!`/`$`) are
/// dropped; the spec keeps `:` literal in the path.
pub(crate) fn matrix_uri(room_id: &str, event_id: &str) -> String {
    let room = room_id.strip_prefix('!').unwrap_or(room_id);
    let event = event_id.strip_prefix('$').unwrap_or(event_id);
    format!("matrix:roomid/{room}/e/{event}")
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
    let (sender, body, summary, room_display_name, is_dm, sender_avatar_url) =
        runtime.block_on(async {
            fetch_notification_event(data_dir, room_id.clone(), event_id.clone()).await
        });

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
        // Tapping the notification opens this Matrix deep link (ACTION_VIEW),
        // routed by the app's `matrix:` intent-filter to tauri-plugin-deep-link
        // (Option B). This replaces the `notificationClicked` event for the tap.
        "deepLink": matrix_uri(&room_id, &event_id),
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
