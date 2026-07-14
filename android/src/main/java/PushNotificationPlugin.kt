package com.plugin.matrix.svelte

import android.app.Activity
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin

@InvokeArg
class SubscribeToTopicArgs {
    lateinit var topic: String
}

@TauriPlugin
class PushNotificationPlugin(private val activity: Activity) : Plugin(activity) {}
