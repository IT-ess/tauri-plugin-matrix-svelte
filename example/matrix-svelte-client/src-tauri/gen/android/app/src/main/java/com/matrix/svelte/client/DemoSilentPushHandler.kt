package com.matrix.svelte.client

import android.content.Context
import android.util.Log
import app.tauri.notification.Notification
import app.tauri.notification.NotificationPlugin
import app.tauri.notification.SilentPushHandler
import org.json.JSONObject

/**
 * Background silent-push handler for the demo, declared on the plugin's FCM
 * service via `<meta-data>` in the manifest. Invoked for every data-only push —
 * **including when the app has been killed** — with only an application context
 * (no Tauri runtime).
 *
 * It hands the data payload to Rust via [SilentPushBridge] (where a real client
 * would fetch/decrypt the Matrix event), then posts the resulting notification
 * with [NotificationPlugin.postBackgroundNotification], reusing the plugin's
 * channel and styling.
 */
class DemoSilentPushHandler : SilentPushHandler {
  override fun onSilentPush(
    context: Context,
    dataDir: String,
    data: Map<String, String>,
    messageId: String?
  ): Boolean {
    val resultJson = try {
      SilentPushBridge.nativeProcessSilentPush(dataDir, JSONObject(data).toString())
    } catch (e: Throwable) {
      Log.e(TAG, "native silent-push processing failed", e)
      null
    } ?: return false

    return try {
      val result = JSONObject(resultJson)
      val notification = Notification().apply {
        id = result.optInt("id", System.currentTimeMillis().toInt())
        title = result.optString("title", "")
        body = result.optString("body", null)
        channelId = result.optString("channelId", null)
      }
      NotificationPlugin.postBackgroundNotification(context, notification)
      Log.i(TAG, "posted background notification ${notification.id} from silent push")
      true
    } catch (e: Throwable) {
      Log.e(TAG, "failed to post background notification", e)
      false
    }
  }

  private companion object {
    const val TAG = "DemoSilentPushHandler"
  }
}