package com.matrix.svelte.client

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Debug-only receiver that simulates a silent (data-only) push without needing a
 * Firebase backend. Because it is manifest-registered, `adb am broadcast` with
 * `FLAG_INCLUDE_STOPPED_PACKAGES` will **cold-start the killed app** and run the
 * exact same path the FCM service would: [DemoSilentPushHandler] → JNI fetch →
 * background notification.
 *
 * Example:
 * ```
 * adb shell am broadcast -a com.test.app.DEBUG_SILENT_PUSH -f 0x01000020 \
 *   --es room_id '!demo:matrix.org' --es event_id '$1700000000' \
 *   -n com.test.app/.DebugSilentPushReceiver
 * ```
 */
class DebugSilentPushReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    val data = mutableMapOf<String, String>()
    intent.getStringExtra("room_id")?.let { data["room_id"] = it }
    intent.getStringExtra("event_id")?.let { data["event_id"] = it }
    Log.i(TAG, "debug silent push received: $data (app may have been killed)")
    val appContext = context.applicationContext
    DemoSilentPushHandler().onSilentPush(appContext, appContext.dataDir.absolutePath, data, null)
  }

  private companion object {
    const val TAG = "DebugSilentPushReceiver"
  }
}
