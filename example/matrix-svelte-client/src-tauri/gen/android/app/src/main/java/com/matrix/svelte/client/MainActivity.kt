package com.matrix.svelte.client

import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
  ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { view: View, insets: WindowInsetsCompat ->
            val bottomInset = insets.getInsets(WindowInsetsCompat.Type.ime()).bottom
            view.setPadding(0, 0, 0, bottomInset)
            insets
  }
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
