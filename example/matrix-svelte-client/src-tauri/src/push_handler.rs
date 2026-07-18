//! The one silent-push handler for both mobile platforms.
//!
//! Registered with `silent_push_handler!`, which exports:
//! * the C entry point the iOS Notification Service Extension resolves via
//!   `dlsym` (the NSE links the same staticlib as the app; linking does not
//!   start Tauri), and
//! * the JNI entry point the notifications plugin's Android FCM service calls
//!   after loading this library (named by the `SILENT_PUSH_LIB` manifest
//!   meta-data) — in every app state, including a killed-app cold start.
//!
//! There is no Tauri `AppHandle` in either context, so the handler returns the
//! notification content and the plugin renders and posts it.
//! [`crate::push_shared::fetch_notification_event`] does the actual work:
//! matrix-ui-serializable loads and decrypts the pushed event from the store
//! under `data_dir` — the App Group container on iOS, the app data directory
//! on Android.
//!
//! On Android, a cold-started push process skipped every process-wide
//! initialization the app performs at startup; [`android_push_init`] (the
//! macro's `android_init` hook) replays them before each handler run.

// The shared helpers are `pub(crate)` for use from `lib.rs`; this module is
// private, so clippy flags that as redundant — it isn't, the parent needs them.
#![allow(clippy::redundant_pub_crate)]

use std::collections::HashMap;

use tauri_plugin_notifications::{NotificationData, NotificationMessage, SilentPushResponse};

use crate::push_shared::{fetch_notification_event, matrix_uri};

/// Must match `ios_app_group` in `tauri.conf.json` and the App Group in both
/// targets' entitlements.
#[cfg(target_os = "ios")]
const APP_GROUP: &str = "group.com.matrix.svelte.client";

/// Decode a Matrix push (`room_id` / `event_id` custom keys) into the
/// notification to display, or the clear-all directive for a badge reset.
// The by-value map is the `SilentPushHandler` contract (the handler owns the data).
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_silent_push(
    data_dir: &str,
    data: HashMap<String, String>,
) -> SilentPushResponse {
    // The NSE process never runs the plugin `setup`, so replay the keyring
    // init here (idempotent), scoped to the shared access group the app
    // saved the session into. (The Android counterpart happens in
    // `android_push_init`, which also needs the JNI context.)
    #[cfg(target_os = "ios")]
    if let Err(e) = tauri_plugin_matrix_svelte::init_keyring_store(Some(APP_GROUP)) {
        tracing::error!("NSE: couldn't init keyring store: {e}");
        return SilentPushResponse::Decline;
    }

    // A data message without event/room ids is not a message push: it's the
    // homeserver's badge update (unread counts only), sent e.g. after a read
    // receipt clears a room. There is no event to fetch — and when everything
    // has been read, ask the plugin to clear the shade. (iOS never delivers
    // badge pushes to the NSE; `ClearActive` degrades to `Decline` there.)
    let (Some(room_id), Some(event_id)) =
        (data.get("room_id").cloned(), data.get("event_id").cloned())
    else {
        let all_read = data
            .get("unread")
            .and_then(|unread| unread.trim().parse::<u64>().ok())
            == Some(0);
        tracing::info!(
            "silent push: badge-only push (keys: {:?}), nothing to display; clearAll={all_read}",
            data.keys().collect::<Vec<_>>()
        );
        return if all_read {
            SilentPushResponse::ClearActive
        } else {
            SilentPushResponse::Decline
        };
    };

    tracing::info!("silent push: fetching {event_id} in {room_id} (data dir: {data_dir})");

    let fetch = fetch_notification_event(data_dir.to_owned(), room_id.clone(), event_id.clone());
    #[cfg(target_os = "android")]
    let content = PUSH_RUNTIME.block_on(fetch);
    // A per-push runtime is fine on iOS: the multi-process NSE mode restores a
    // fresh client on every call anyway (no cached client/tasks to keep alive,
    // unlike the Android path's process-wide runtime).
    #[cfg(target_os = "ios")]
    let content = {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return SilentPushResponse::Decline;
        };
        runtime.block_on(fetch)
    };

    let mut message = NotificationMessage::new(content.body.clone())
        .sender(content.sender.clone())
        .person_key(content.sender);
    if let Some(avatar) = content.sender_avatar {
        message = message.avatar_bytes(avatar);
    }

    // MessagingStyle: the plugin decodes the avatar bytes and renders a
    // chat-style notification with the sender's circular avatar and the room
    // as the title. The id is keyed by the room so each new event stacks onto
    // the same conversation notification. `channel_id` is the high-importance
    // channel the frontend creates at startup (`MESSAGES_CHANNEL_ID` in
    // `src/lib/notifications.ts`, also the manifest's
    // `default_notification_channel_id`). Tapping the notification opens the
    // Matrix deep link (ACTION_VIEW), routed by the app's `matrix:`
    // intent-filter to tauri-plugin-deep-link.
    #[cfg(target_os = "android")]
    let mut builder = NotificationData::builder()
        .id(notification_id_for(&room_id))
        .channel_id("messages")
        .title(content.summary)
        .body(content.body)
        .conversation_title(content.room_display_name)
        .self_name("Me")
        .deep_link(matrix_uri(&room_id, &event_id))
        .auto_cancel()
        .message(message);

    // No `.id(...)`: the extension cannot change the identifier APNs assigned.
    // `group` (→ `threadIdentifier`) stacks the room's notifications together
    // instead. The deep link and the Matrix ids ride in `extra`, whose string
    // values surface in the `notificationClicked` event's `data` when tapped —
    // the frontend routes `deepLink` through `handleMatrixUri`. The NSE renders
    // the message fields as a communication notification.
    #[cfg(target_os = "ios")]
    let mut builder = NotificationData::builder()
        .title(content.summary)
        .body(content.body)
        .group(room_id.as_str())
        .summary(content.room_display_name.clone())
        .message(message)
        .conversation_title(content.room_display_name)
        .extra("deepLink", matrix_uri(&room_id, &event_id))
        .extra("room_id", room_id)
        .extra("event_id", event_id);

    // DMs draw the sender's avatar; group rooms brand as the room instead —
    // room name as the conversation title, room avatar (when it has one) as
    // the icon.
    if !content.is_dm {
        builder = builder.group_conversation();
        if let Some(avatar) = content.room_avatar {
            builder = builder.conversation_avatar_bytes(avatar);
        }
    }
    SilentPushResponse::Notification(builder.build())
}

tauri_plugin_notifications::silent_push_handler!(handle_silent_push, android_init = android_push_init);

/// Derive a stable, positive notification id from a conversation key (the room
/// id). Using the room as the key means every message in that room lands in the
/// same notification, so the plugin accumulates them into one conversation
/// instead of posting a separate notification per event.
// ponytail: 31-bit hash, two rooms can collide (~n²/2³² per pair) and would
// merge/dismiss together; map room id → id in persistent storage if that ever
// bites.
#[cfg(target_os = "android")]
pub(crate) fn notification_id_for(key: &str) -> i32 {
    let hash = key.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u32::from(b))
    }) & 0x7fff_ffff;
    i32::try_from(hash).unwrap_or(0)
}

/// The tokio runtime driving Android push fetches. Process-wide rather than
/// per-push: matrix-ui-serializable caches the notification client across
/// pushes, and tasks the SDK spawns for it (connection pools, store
/// maintenance) must not be killed between two pushes by a runtime drop. In a
/// warm process the fetch hops onto the app's runtime anyway, so this one only
/// ever drives the cross-runtime await.
///
/// One worker thread is plenty: pushes are serialized by the FCM service.
#[cfg(target_os = "android")]
static PUSH_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("couldn't build the push runtime")
    });

/// The `android_init` hook of `silent_push_handler!`: runs on the FCM service
/// thread before every handler invocation — warm or cold — with the
/// Application context. Replays the process-wide initializations the Tauri
/// runtime performs at startup but which are skipped when Firebase cold-starts
/// the process (all idempotent, so warm runs are no-ops):
///
/// 1. logcat logging, so the fetch below is debuggable (`adb logcat -s
///    MatrixSilentPush`);
/// 2. `ndk_context` — the Android keyring backend resolves its `Context`
///    through this global;
/// 3. `rustls-platform-verifier` — matrix-sdk fetches the event from the
///    homeserver over TLS, which needs the Android trust roots;
/// 4. the keyring backend (`init_keyring_store`) — sets the process-wide
///    `keyring_core` default store the session is read from.
///
/// A failure is logged but not fatal: the fetch then falls back to the
/// placeholder message, which is exactly the symptom we want to surface.
#[cfg(target_os = "android")]
pub(crate) fn android_push_init(env: &mut jni::JNIEnv, context: &jni::objects::JObject) {
    init_cold_path_logging();
    if let Err(e) = init_cold_path_context(env, context) {
        tracing::error!("cold-path context init failed: {e}");
    }
}

/// Keeps the global ref to the Android `Context` alive for the lifetime of the
/// process and guarantees `ndk_context::initialize_android_context` runs at most
/// once, regardless of which entry point reaches it first.
#[cfg(target_os = "android")]
static NDK_CONTEXT_REF: std::sync::OnceLock<jni::objects::GlobalRef> = std::sync::OnceLock::new();

/// Initialize the `ndk_context` global exactly once per process.
///
/// `ndk_context::initialize_android_context` panics (`assert!(previous.is_none())`)
/// if called twice. Both the push init hook ([`android_push_init`]) and
/// `MainActivity.onCreate` (`initNdkContext` in `lib.rs`) need the context
/// initialized; because the FCM service shares the app's process and Android may
/// reuse that process to launch the activity (or it is already the warm app),
/// both can run in one process. This shared guard makes the second call a no-op
/// instead of an abort.
#[cfg(target_os = "android")]
pub(crate) fn ensure_ndk_context(env: &mut jni::JNIEnv, context: &jni::objects::JObject) {
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

/// Install a `tracing` subscriber that writes to Android's logcat. Idempotent.
///
/// In the cold path there is no Tauri runtime, so the app's normal logging
/// setup never runs and every `tracing::*` call in this module would otherwise
/// be dropped. View the output with e.g. `adb logcat -s MatrixSilentPush`.
#[cfg(target_os = "android")]
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

/// See [`android_push_init`] for what is replayed and why. All steps are
/// idempotent (guarded by `Once`/internally) so handling several pushes in one
/// process is safe. Mirrors `MainActivity.initNdkContext` (`lib.rs`) and the
/// matrix-svelte plugin `setup`.
#[cfg(target_os = "android")]
fn init_cold_path_context(
    env: &mut jni::JNIEnv,
    context: &jni::objects::JObject,
) -> Result<(), String> {
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
