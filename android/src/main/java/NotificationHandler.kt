package com.plugin.matrix.svelte

import android.app.ActivityOptions
import android.app.ActivityOptions.MODE_BACKGROUND_ACTIVITY_START_ALLOWED
import android.app.ActivityOptions.MODE_BACKGROUND_ACTIVITY_START_ALLOW_ALWAYS
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent


class NotificationHandler : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val mainActivityClass = Class.forName("${context.packageName}.MainActivity")
        val newIntent = Intent(context, mainActivityClass).apply {
            flags =
                Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
        }
        newIntent.putExtra("data", intent.getStringExtra("data"))
        newIntent.putExtra("sent_at", intent.getLongExtra("sent_at", 0))
        newIntent.putExtra("opened_at", System.currentTimeMillis())




//        val options = ActivityOptions
//            .makeBasic()
//            .setPendingIntentCreatorBackgroundActivityStartMode(
//                MODE_BACKGROUND_ACTIVITY_START_ALLOWED
//            )

        context.startActivity(newIntent)
    }
}
