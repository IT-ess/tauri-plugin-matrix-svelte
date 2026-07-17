package com.matrix.svelte.client

import android.content.Context
import android.util.Log
import androidx.core.app.NotificationManagerCompat
import app.tauri.notification.Notification
import app.tauri.notification.NotificationPlugin
import app.tauri.notification.SilentPushHandler
import com.fasterxml.jackson.databind.ObjectMapper
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
      SilentPushBridge.nativeProcessSilentPush(context.applicationContext, dataDir, JSONObject(data).toString())
    } catch (e: Throwable) {
      Log.e(TAG, "native silent-push processing failed", e)
      null
    } ?: return false

    return try {
      // Badge-only push (no event to display): Rust asks us to post nothing,
      // and to clear the shade once every message has been read.
      val result = JSONObject(resultJson)
      if (result.optBoolean("skip", false)) {
        if (result.optBoolean("clearAll", false)) {
          NotificationManagerCompat.from(context).cancelAll()
          // Everything is read: also drop the stored chat histories so the
          // next message starts a fresh thread instead of resurrecting the
          // cleared conversation.
          NotificationPlugin.clearAllConversations(context)
          Log.i(TAG, "badge reset push: cleared active notifications")
        }
        return true
      }
      // Rust serializes the plugin's own `NotificationData` wire format
      // (camelCase serde), which is exactly what `Notification` deserializes —
      // same pair the JS invoke path and `NotificationStorage` use.
      val notification = ObjectMapper().readValue(resultJson, Notification::class.java)
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
