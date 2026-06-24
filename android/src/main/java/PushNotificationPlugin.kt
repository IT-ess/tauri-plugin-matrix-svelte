package com.plugin.matrix.svelte

import android.app.Activity
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin


@TauriPlugin
class PushNotificationPlugin(private val activity: Activity) : Plugin(activity) {

}
