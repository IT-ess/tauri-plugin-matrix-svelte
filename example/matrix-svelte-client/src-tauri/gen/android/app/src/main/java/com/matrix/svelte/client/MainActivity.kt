package com.matrix.svelte.client

import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  companion object {
    // Loaded here too (idempotent): `claimNdkContext` runs before WryActivity
    // touches `Rust`, which is what normally loads the library.
    init {
      System.loadLibrary("matrix_svelte_client_lib")
    }

    // Tells the silent-push path that tao is about to own `ndk_context`
    // (see `push_handler.rs`). Must run before `super.onCreate`.
    @JvmStatic private external fun claimNdkContext()
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    claimNdkContext()
  ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { view: View, insets: WindowInsetsCompat ->
            val bottomInset = insets.getInsets(WindowInsetsCompat.Type.ime()).bottom
            view.setPadding(0, 0, 0, bottomInset)
            insets
  }
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
