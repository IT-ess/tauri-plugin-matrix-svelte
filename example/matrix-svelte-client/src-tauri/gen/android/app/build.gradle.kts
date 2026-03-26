import groovy.json.JsonSlurper
import java.util.Properties
import java.io.FileInputStream

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

repositories {
  rustlsPlatformVerifier()
}


fun RepositoryHandler.rustlsPlatformVerifier(): MavenArtifactRepository {
  @Suppress("UnstableApiUsage")
  val manifestPath = let {
    val dependencyJson = providers.exec {
      workingDir = File(project.rootDir, "../")
      commandLine(
        "cargo",
        "metadata",
        "--format-version", "1",
        "--filter-platform", "aarch64-linux-android",
        "--manifest-path", "../../../../Cargo.toml"
      )
    }.standardOutput.asText

    val parsed = JsonSlurper().parseText(dependencyJson.get()) as Map<String, Any>
    val packages = parsed["packages"] as List<Map<String, Any>>
    val path = packages.first { it["name"] == "rustls-platform-verifier-android" }["manifest_path"] as String

    File(path)
  }

  return maven {
    url = uri(File(manifestPath.parentFile, "maven").path)
    metadataSources.artifact()
  }
}


dependencies {
  // `rustls-platform-verifier` is a Rust crate, but it also has a Kotlin component.
  implementation("rustls:rustls-platform-verifier:0.1.1")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 36
    namespace = "com.matrix.svelte.client"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.matrix.svelte.client"
        minSdk = 34
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    signingConfigs {
      create("release") {
          val keystorePropertiesFile = rootProject.file("keystore.properties")
          val keystoreProperties = Properties()
          if (keystorePropertiesFile.exists()) {
              keystoreProperties.load(FileInputStream(keystorePropertiesFile))
          }

          keyAlias = keystoreProperties["keyAlias"] as String
          keyPassword = keystoreProperties["password"] as String
          storeFile = file(keystoreProperties["storeFile"] as String)
          storePassword = keystoreProperties["password"] as String
      }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation ("rustls:rustls-platform-verifier:0.1.1")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")
