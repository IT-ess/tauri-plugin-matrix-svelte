# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }

# The silent-push handler is only reachable through the SILENT_PUSH_HANDLER
# manifest meta-data + Class.forName, and the JNI bridge only through the
# handler — R8 can't see either reference, so without these keeps release
# builds strip them and lose the killed-state (cold) push path.
-keep class com.matrix.svelte.client.DemoSilentPushHandler { <init>(); }
-keep class com.matrix.svelte.client.SilentPushBridge { *; }