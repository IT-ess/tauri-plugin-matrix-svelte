package com.matrix.svelte.client

import android.content.Context
import android.util.Log
import app.tauri.notification.Notification
import app.tauri.notification.NotificationMessage
import app.tauri.notification.NotificationPlugin
import app.tauri.notification.SilentPushHandler
import org.json.JSONArray
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
      val result = JSONObject(resultJson)
      val notification = Notification().apply {
        id = result.optInt("id", System.currentTimeMillis().toInt())
        title = result.optString("title", "")
        body = result.optString("body", null)
        channelId = result.optString("channelId", null)
        // Chat-style fields: the plugin renders these as MessagingStyle and
        // decodes each message's base64 avatar into a circular icon.
        conversationTitle = result.optString("conversationTitle", null)
        groupConversation = result.optBoolean("groupConversation", false)
        selfName = result.optString("selfName", null)
        appendMessages = result.optBoolean("appendMessages", true)
        messages = parseMessages(result.optJSONArray("messages"))
        // Dismiss the notification when tapped (FLAG_AUTO_CANCEL). Without this the
        // posted notification lingers in the shade after launching the app.
        isAutoCancel = result.optBoolean("autoCancel", true)
        // Tapping the notification opens this `matrix:` deep link (ACTION_VIEW),
        // which the app's intent-filter routes to tauri-plugin-deep-link.
        deepLink = result.optString("deepLink", null)
      }
      NotificationPlugin.postBackgroundNotification(context, notification)
      Log.i(TAG, "posted background notification ${notification.id} from silent push")
      true
    } catch (e: Throwable) {
      Log.e(TAG, "failed to post background notification", e)
      false
    }
  }

  private fun parseMessages(array: JSONArray?): List<NotificationMessage>? {
    if (array == null) return null
    val messages = mutableListOf<NotificationMessage>()
    for (i in 0 until array.length()) {
      val obj = array.optJSONObject(i) ?: continue
      messages.add(NotificationMessage().apply {
        sender = obj.optString("sender", null)
        personKey = obj.optString("personKey", null)
        avatarBytes = obj.optString("avatarBytes", null)
        text = obj.optString("text", null)
        timestamp = obj.optLong("timestamp", 0)
      })
    }
    return messages
  }

  private companion object {
    const val TAG = "DemoSilentPushHandler"
  }
}
