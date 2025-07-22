package com.plugin.matrix.svelte


import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import com.google.gson.Gson
import java.time.Instant
import kotlin.random.Random
import kotlin.time.Duration.Companion.milliseconds

class FCMService : FirebaseMessagingService() {
    @SuppressLint("LaunchActivityFromNotification")
    override fun onMessageReceived(remoteMessage: RemoteMessage) {

        val gson = Gson() // should we keep a reference ?
        val dataPayload = gson.toJson(remoteMessage.data)
        Log.d("NOTIFDATA", dataPayload)

        // We no longer use Notification Handler for the moment. We'll try again later to process notifications body.
        //val intent = Intent(this, NotificationHandler::class.java).apply {
        //    putExtra("data", dataPayload)
        //    putExtra("sent_at", remoteMessage.sentTime)
        //}

        //val requestCode = Random.nextInt()
        //val pendingIntent = PendingIntent.getBroadcast(
        //    this,
        //    requestCode,
        //    intent,
        //    PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_IMMUTABLE
        //)

        this.sendNotification(remoteMessage.data["sender_display_name"] ?: "Unknown Sender")


    }

        private fun sendNotification(sender: String) {
            val mainActivityClass = Class.forName("${applicationContext.packageName}.MainActivity")
            val intent = Intent(this, mainActivityClass)
            intent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)

            val requestCode = Random.nextInt()
            val pendingIntent = PendingIntent.getActivity(this, requestCode, intent,  PendingIntent.FLAG_IMMUTABLE)

            val channelId = "messages"
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

            val channel = NotificationChannel(
                channelId,
                "Messages",
                NotificationManager.IMPORTANCE_HIGH,).apply { description = "Notifications for Matrix Svelte client !" }

            notificationManager.createNotificationChannel(channel)


            val notificationBuilder = NotificationCompat.Builder(this, channelId)
                .setSmallIcon(getAppIconResourceId())
                .setContentTitle(sender)
                .setContentText("Click to view message")
                .setAutoCancel(true)
                .setContentIntent(pendingIntent)
                .setPriority(NotificationCompat.PRIORITY_HIGH)
                .setCategory(NotificationCompat.CATEGORY_EVENT)
                .setFullScreenIntent(pendingIntent, true)

            notificationManager.notify(Random.nextInt(), notificationBuilder.build())
        }


    private fun getAppIconResourceId(): Int {
        val packageManager = packageManager
        val packageName = applicationContext.packageName
        try {
            val appInfo = packageManager.getApplicationInfo(packageName, 0)
            return appInfo.icon
        } catch (e: PackageManager.NameNotFoundException) {
            e.printStackTrace()
        }
        return android.R.drawable.sym_def_app_icon // fallback icon
    }
}
