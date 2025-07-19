package com.matrix_svelte.client

//import android.content.ContentValues.TAG
//import android.os.Bundle
//import android.util.Log
//import android.widget.Toast
//import com.google.android.gms.tasks.OnCompleteListener
//import com.google.firebase.messaging.FirebaseMessaging

class MainActivity : TauriActivity() 
//{
//  override fun onCreate(savedInstanceState: Bundle?) {
//    super.onCreate(savedInstanceState)
//
//    FirebaseMessaging.getInstance().token.addOnCompleteListener(OnCompleteListener { task ->
//      if (!task.isSuccessful) {
//        Log.w("FCM", "Fetching FCM registration token failed", task.exception)
//        return@OnCompleteListener
//      }
//
//      // Get new FCM registration token
//      val token = task.result
//
//      
//      Log.d("FCM", token)
//      Toast.makeText(baseContext, token, Toast.LENGTH_LONG).show()
//    })
//  }
//}