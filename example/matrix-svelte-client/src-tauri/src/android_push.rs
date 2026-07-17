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
//!
//! ponytail: everything here except the notification formatting is
//! app-agnostic boilerplate (JNI entry, ndk_context/TLS/keyring init, push
//! runtime) that every consumer would copy; planned to move into the plugin
//! behind an `android_silent_push_handler!` macro (mirroring the notifications
//! plugin's `ios_silent_push_handler!`) in a follow-up PR.

// The shared helpers are `pub(crate)` for use from `lib.rs`; this module is
// private, so clippy flags that as redundant — it isn't, the parent needs them.
#![allow(clippy::redundant_pub_crate)]

use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

use jni::JNIEnv;
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::sys::jstring;
use tauri_plugin_notifications::{NotificationData, NotificationMessage};

use crate::push_shared::{fetch_notification_event, matrix_uri};

/// Keeps the global ref to the Android `Context` alive for the lifetime of the
/// process and guarantees `ndk_context::initialize_android_context` runs at most
/// once, regardless of which entry point reaches it first.
static NDK_CONTEXT_REF: OnceLock<GlobalRef> = OnceLock::new();

/// Initialize the `ndk_context` global exactly once per process.
///
/// `ndk_context::initialize_android_context` panics (`assert!(previous.is_none())`)
/// if called twice. Both the cold-push JNI entry ([`init_cold_path_context`]) and
/// `MainActivity.onCreate` (`initNdkContext` in `lib.rs`) need the context
/// initialized; because the FCM service shares the app's process and Android may
/// reuse that process to launch the activity (or it is already the warm app),
/// both can run in one process. This shared guard makes the second call a no-op
/// instead of an abort.
pub(crate) fn ensure_ndk_context(env: &mut JNIEnv, context: &JObject) {
    if NDK_CONTEXT_REF.get().is_some() {
        return;
    }
    let Ok(context_ref) = env.new_global_ref(context) else {
        tracing::error!("ensure_ndk_context: couldn't create global ref for context");
        return;
    };
    let Ok(vm) = env.get_java_vm() else {
        tracing::error!("ensure_ndk_context: couldn't get JavaVM");
        return;
    };
    // `get_or_init` makes the unsafe init + store atomic; if another thread won
    // the race the closure never runs and our spare `context_ref` is dropped.
    NDK_CONTEXT_REF.get_or_init(|| {
        unsafe {
            ndk_context::initialize_android_context(
                vm.get_java_vm_pointer().cast::<std::ffi::c_void>(),
                context_ref.as_obj().as_raw().cast::<std::ffi::c_void>(),
            );
        }
        tracing::info!("ndk_context initialized");
        context_ref
    });
}

/// Derive a stable, positive notification id from a conversation key (the room
/// id). Using the room as the key means every message in that room lands in the
/// same notification, so the plugin accumulates them into one conversation
/// instead of posting a separate notification per event.
// ponytail: 31-bit hash, two rooms can collide (~n²/2³² per pair) and would
// merge/dismiss together; map room id → id in persistent storage if that ever
// bites.
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
        // `try_init` fails only if a global subscriber is already set (e.g. the
        // app's own logging ran first in a warm process); ignore that.
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
    use std::sync::Mutex;

    // 1. NDK context (used by the Android keyring backend). Shared guard with
    //    `MainActivity.initNdkContext` so it is initialized at most once per
    //    process even when this push process is reused to launch the app.
    ensure_ndk_context(env, context);

    // 2. TLS platform verifier (used by matrix-sdk's HTTP client). Its
    //    `init_with_refs` is internally idempotent (`get_or_init`), but the JNI
    //    work to build the refs isn't free, so skip it once it has succeeded.
    //    The flag is only set on success so a failed attempt is retried on the
    //    next push instead of being latched as a silent permanent failure.
    static TLS_DONE: Mutex<bool> = Mutex::new(false);
    {
        let mut done = TLS_DONE.lock().unwrap();
        if !*done {
            let vm = env
                .get_java_vm()
                .map_err(|e| format!("getting JavaVM: {e}"))?;
            let context_ref = env
                .new_global_ref(context)
                .map_err(|e| format!("global-ref'ing context: {e}"))?;
            let loader = env
                .call_method(context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
                .and_then(|v| v.l())
                .map_err(|e| format!("getting ClassLoader: {e}"))?;
            let loader_ref = env
                .new_global_ref(&loader)
                .map_err(|e| format!("global-ref'ing ClassLoader: {e}"))?;
            rustls_platform_verifier::android::init_with_refs(vm, context_ref, loader_ref);
            tracing::info!("cold-path: rustls platform verifier initialized");
            *done = true;
        }
    }

    // 3. Keyring backend. Idempotent inside the plugin; cheap to call each time.
    //    The access-group argument is iOS-only.
    tauri_plugin_matrix_svelte::init_keyring_store(None)
        .map_err(|e| format!("initializing keyring store: {e}"))?;
    tracing::info!("cold-path: keyring store initialized");

    Ok(())
}

/// The tokio runtime driving cold-path (killed-app) push fetches. Process-wide
/// rather than per-push: matrix-ui-serializable caches the notification client
/// across pushes, and tasks the SDK spawns for it (connection pools, store
/// maintenance) must not be killed between two pushes by a runtime drop. In a
/// warm process the fetch hops onto the app's runtime anyway, so this one only
/// ever drives the cross-runtime await.
///
/// One worker thread is plenty: pushes are serialized by the FCM service.
static PUSH_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("couldn't build the push runtime")
});

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

    // A data message without event/room ids is not a message push: it's the
    // homeserver's badge update (unread counts only), sent e.g. after a read
    // receipt clears a room. There is no event to fetch, so tell Kotlin to
    // post nothing — and to clear the shade when everything has been read.
    let (Some(room_id), Some(event_id)) = (
        data.get("room_id").cloned(),
        data.get("event_id").cloned(),
    ) else {
        let all_read = data
            .get("unread")
            .and_then(|unread| unread.trim().parse::<u64>().ok())
            == Some(0);
        tracing::info!(
            "silent push (background/JNI): badge-only push (keys: {:?}), skipping; clearAll={all_read}",
            data.keys().collect::<Vec<_>>()
        );
        return Ok(serde_json::json!({ "skip": true, "clearAll": all_read }).to_string());
    };

    tracing::info!(
        "silent push (background/JNI): fetching {event_id} in {room_id} (data dir: {data_dir})"
    );

    let notif_id = notification_id_for(&room_id);
    let content = PUSH_RUNTIME.block_on(fetch_notification_event(
        data_dir,
        room_id.clone(),
        event_id.clone(),
    ));

    // MessagingStyle: the plugin decodes `avatarBytes` and renders a chat-style
    // notification with the sender's circular avatar and the room as the title.
    // The id is keyed by the room, and `appendMessages` lets the plugin stack
    // each new event onto the same conversation notification. `NotificationData`
    // is the plugin's own wire format (camelCase serde ↔ Jackson on the Kotlin
    // side), so the JSON stays in lockstep with the parser by construction.
    let mut message = NotificationMessage::new(content.body.clone())
        .sender(content.sender.clone())
        .person_key(content.sender);
    if let Some(avatar) = content.sender_avatar {
        message = message.avatar_bytes(avatar);
    }
    let mut builder = NotificationData::builder()
        .id(notif_id)
        // The high-importance channel the frontend creates at startup
        // (`MESSAGES_CHANNEL_ID` in `src/lib/notifications.ts`, also the
        // manifest's `default_notification_channel_id`) — required for
        // heads-up message notifications.
        .channel_id("messages")
        .title(content.summary)
        .body(content.body)
        .conversation_title(content.room_display_name)
        .self_name("Me")
        // Tapping the notification opens this Matrix deep link (ACTION_VIEW),
        // routed by the app's `matrix:` intent-filter to tauri-plugin-deep-link
        // (Option B). This replaces the `notificationClicked` event for the tap.
        .deep_link(matrix_uri(&room_id, &event_id))
        .auto_cancel()
        .message(message);
    if !content.is_dm {
        builder = builder.group_conversation();
        if let Some(avatar) = content.room_avatar {
            builder = builder.conversation_avatar_bytes(avatar);
        }
    }
    serde_json::to_string(&builder.build()).map_err(|e| format!("serializing notification: {e}"))
}
