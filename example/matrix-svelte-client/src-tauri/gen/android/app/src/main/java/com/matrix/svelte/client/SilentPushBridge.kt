package com.matrix.svelte.client

import android.content.Context

/**
 * Bridge to the app's native library for background silent-push handling.
 *
 * The `init` block loads `notifications_demo_lib` — the same `.so` the Tauri
 * runtime uses — but loading it does NOT start Tauri (only `Rust.create()` /
 * `start()` do). That lets the FCM service call [nativeProcessSilentPush] after
 * a cold start, with no Activity/WebView, to "fetch" notification content in
 * Rust (where matrix-rust-sdk would run).
 *
 * Input: an Android `Context` (used by Rust to initialize the NDK context, TLS
 * platform verifier and keyring backend that the Tauri runtime would normally
 * set up at startup), the app data directory path, and the FCM data payload as
 * a JSON object string.
 * Output: JSON `{ id, title, body, channelId }`, or `null` on failure.
 */
object SilentPushBridge {
  init {
    System.loadLibrary("matrix_svelte_client_lib")
  }

  @JvmStatic
  external fun nativeProcessSilentPush(context: Context, dataDir: String, dataJson: String): String?
}
