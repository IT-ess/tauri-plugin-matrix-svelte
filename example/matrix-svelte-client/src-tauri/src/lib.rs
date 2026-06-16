use anyhow::anyhow;
use mime_serde_shim::Wrapper as MimeWrapper;
use serde::Deserialize;
use tauri::http::{self, HeaderValue, Uri};
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_matrix_svelte::{
    AUTH_DEEPLINK_SENDER, Base64, CLIENT, EncryptedFile, EncryptedFileHashes, EncryptedFileInfo,
    LOGIN_STORE_READY, MediaFormat, MediaRequestParameters, MediaSource, MediaThumbnailSettings,
    Method, OwnedMxcUri, Standard, UInt, UrlSafe, V2EncryptedFileInfo,
};
#[cfg(target_os = "android")]
use tauri_plugin_notifications::NotificationsExt;
use tauri_plugin_svelte::CborMarshaler;
use tracing::{error, trace};

mod logging;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().register_asynchronous_uri_scheme_protocol(
        "mxc",
        |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                // Looks like:
                // On android and windows: http://mxc.localhost/matrix.org/mediaid?iv=...
                // On others: mxc://matrix.org/mediaid?iv=...
                let raw_uri = request.uri();
                // Android and Windows doesn't support directly using a custom protocol.
                // So we reconstruct a new_uri matching the common pattern.
                let uri = if cfg!(any(target_os = "android", target_os = "windows")) {
                    match android_windows_to_common_uri(raw_uri) {
                        Ok(common_uri) => common_uri,
                        Err(e) => {
                            error!("URI {raw_uri} couldn't be converted to android format. {e}.");
                            responder.respond(
                                http::Response::builder()
                                    .status(http::StatusCode::BAD_REQUEST)
                                    .header(http::header::CONTENT_TYPE, "text/plain")
                                    .body("failed to get media, wrong url".as_bytes().to_vec())
                                    .unwrap(),
                            );
                            return;
                        }
                    }
                } else {
                    raw_uri.to_owned()
                };

                let mxc_uri = OwnedMxcUri::from(uri.to_string().split('?').next().unwrap());

                let (media_request, mime, size) = if let Some(query_str) = uri.query() {
                    match serde_urlencoded::from_str(query_str) {
                        Ok(MediaQueryParams {
                            k,
                            iv,
                            hash,
                            mime,
                            size,
                            th,
                            tw,
                            tm,
                        }) => {
                            let media_format = if let Some(thumb_height) = th
                                && let Some(thumb_width) = tw
                                && let Some(method) = tm
                            {
                                MediaFormat::Thumbnail(MediaThumbnailSettings {
                                    method,
                                    width: thumb_width,
                                    height: thumb_height,
                                    animated: false,
                                })
                            } else {
                                MediaFormat::File
                            };

                            if let Some(k) = k
                                && let Some(iv) = iv
                                && let Some(hash) = hash
                            {
                                let info = EncryptedFileInfo::V2(V2EncryptedFileInfo::new(k, iv));
                                let hashes = EncryptedFileHashes::with_sha256(hash.into_inner());
                                let encrypted_file = EncryptedFile::new(mxc_uri, info, hashes);

                                (
                                    MediaRequestParameters {
                                        source: MediaSource::Encrypted(Box::new(encrypted_file)),
                                        format: media_format,
                                    },
                                    mime,
                                    size,
                                )
                            } else {
                                (
                                    MediaRequestParameters {
                                        source: MediaSource::Plain(mxc_uri),
                                        format: media_format,
                                    },
                                    mime,
                                    size,
                                )
                            }
                        }
                        Err(e) => {
                            error!("Cannot deserialize encrypted media info. {e}");
                            return;
                        }
                    }
                } else {
                    (
                        MediaRequestParameters {
                            source: MediaSource::Plain(mxc_uri),
                            format: MediaFormat::File,
                        },
                        None,
                        None,
                    )
                };

                // TODO: even if we send this header, the webview doesn't
                // cache the content for some reason. I should find a way
                // to reliably cache the data.
                let mut response = http::Response::builder().header(
                    http::header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable",
                );

                if let Some(content_length) = size {
                    response.headers_mut().unwrap().append(
                        http::header::CONTENT_LENGTH,
                        HeaderValue::from_str(content_length.to_string().as_str()).unwrap(),
                    );
                }

                if let Some(mime) = mime {
                    response.headers_mut().unwrap().append(
                        http::header::CONTENT_TYPE,
                        HeaderValue::from_str(mime.essence_str()).unwrap(),
                    );
                }

                let Some(client) = CLIENT.get() else {
                    responder.respond(
                        http::Response::builder()
                            .status(http::StatusCode::BAD_REQUEST)
                            .header(http::header::CONTENT_TYPE, "text/plain")
                            .body("failed to get media. Client not ready.".as_bytes().to_vec())
                            .unwrap(),
                    );
                    return;
                };

                match client.media().get_media_content(&media_request, true).await {
                    Ok(data) => match response.status(200).body(data) {
                        Ok(res) => {
                            responder.respond(res);
                            trace!("responded to uri request {}", request.uri());
                        }
                        Err(e) => {
                            error!("Cannot build response. {e}")
                        }
                    },
                    Err(e) => {
                        error!("Media error: {e}");
                        responder.respond(
                            http::Response::builder()
                                .status(http::StatusCode::BAD_REQUEST)
                                .header(http::header::CONTENT_TYPE, "text/plain")
                                .body("failed to get media".as_bytes().to_vec())
                                .unwrap(),
                        );
                    }
                }
            });
        },
    );

    builder = logging::setup_logging(builder);

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
          tracing::debug!("a new app instance was opened with {argv:?} and the deep link event was already triggered");
          // when defining deep link schemes at runtime, you must also check `argv` here

          let _ = app.get_webview_window("main")
                     .expect("no main window")
                     .set_focus();
        }));
    }

    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_sharekit::init());
    }

    // Init deeplink plugin before tauri_plugin_mobile_sharetarget
    builder = builder.plugin(tauri_plugin_deep_link::init());

    #[cfg(target_os = "ios")]
    {
        builder = builder.plugin(tauri_plugin_web_auth::init());
    }

    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(tauri_plugin_android_fs::init());
    }

    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_safe_area_insets_css::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notifications::init())
        .plugin(
            tauri_plugin_svelte::Builder::new()
                .marshaler(Box::new(CborMarshaler))
                .on_load(|store| {
                    if store.id().to_string() == tauri_plugin_matrix_svelte::LOGIN_STATE_STORE_ID {
                        LOGIN_STORE_READY
                            .set(true)
                            .expect("LOGIN_STORE_READY has already been set !");
                    }
                    Ok(())
                })
                .build(),
        )
        .plugin(tauri_plugin_matrix_svelte::init())
        .setup(|app| {
            #[cfg(target_os = "android")]
            let _ = app
                .get_webview_window("main")
                .unwrap()
                .with_webview(|webview| {
                    webview.jni_handle().exec(|env, context, _webview| {
                        use tauri::wry::prelude::JObject;
                        let loader = env
                            .call_method(
                                context,
                                "getClassLoader",
                                "()Ljava/lang/ClassLoader;",
                                &[],
                            )
                            .unwrap();

                        rustls_platform_verifier::android::init_with_refs(
                            env.get_java_vm().unwrap(),
                            env.new_global_ref(context).unwrap(),
                            env.new_global_ref(JObject::try_from(loader).unwrap())
                                .unwrap(),
                        );
                    })
                });
            // Register the Rust-only silent-push handler. On Android, data-only
            // FCM messages are routed here so we can fetch content and raise the
            // notification ourselves — the Matrix client pattern.
            #[cfg(target_os = "android")]
            {
                let handle = app.handle().clone();
                if let Err(e) = app.notifications().on_silent_push(move |push| {
                    tracing::info!("silent push received: {:?}", push.data);
                    process_silent_push(&handle, &push.data);
                }) {
                    tracing::error!("failed to register silent push handler: {e}");
                }
            }
            // Tray icon stuff
            #[cfg(desktop)]
            {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::TrayIconBuilder,
                };
                use tracing::warn;

                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&quit_i])?;
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            tracing::debug!("quit menu item was clicked");
                            app.exit(0);
                        }
                        _ => {
                            warn!("menu item {:?} not handled", event.id);
                        }
                    })
                    .build(app)?;
            }

            // Handle scheme:// deeplink

            let deeplink_manager = app.deep_link();
            #[cfg(any(windows, target_os = "linux"))]
            {
                deeplink_manager
                    .register_all()
                    .expect("couldn't register deeplink");
            }

            let deeplink_handle = app.app_handle().clone();

            deeplink_manager.on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
                    // Matches matrix: URIs
                    if url.scheme().eq("matrix") {
                        tauri_plugin_matrix_svelte::handle_matrix_uri(url);
                        return;
                    }

                    // Matches scheme://auth-callback
                    if url.host_str().is_some_and(|s| s.eq("auth-callback")) {
                        // Wake up the UI (for iOS only)
                        deeplink_handle.emit("new-intent", ()).unwrap();
                        let sender = AUTH_DEEPLINK_SENDER
                            .get()
                            .expect("sender should be defined at this point");
                        sender
                            .blocking_send(url.to_owned())
                            .expect("couldn't send deeplink payload to receiver");
                    }

                    // Matches https://oauth-client-uri-domain/auth-callback
                    let plugin_config = deeplink_handle.config().plugins.0.clone();
                    let raw_matrix_config = plugin_config
                        .get("matrix-svelte")
                        .expect("Plugin 'matrix-svelte' configuration not found");
                    let matrix_plugin_config: tauri_plugin_matrix_svelte::PluginConfig =
                        serde_json::from_value(raw_matrix_config.clone())
                            .expect("Missing fields in plugin configuration");
                    if url.host_str().is_some_and(|s| {
                        s.eq(matrix_plugin_config
                            .oauth_client_uri
                            .domain()
                            .expect("this url should have a domain"))
                    }) & url.path().contains("auth-callback")
                    {
                        let sender = AUTH_DEEPLINK_SENDER
                            .get()
                            .expect("sender should be defined at this point");
                        sender
                            .blocking_send(url.to_owned())
                            .expect("couldn't send deeplink payload to receiver");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Deserialize)]
/// Used to deserialize the custom URIs
struct MediaQueryParams {
    /// The Base64 URL-safe Key
    k: Option<Base64<UrlSafe, [u8; 32]>>,
    /// The Base64 Standard IV
    iv: Option<Base64<Standard, [u8; 16]>>,
    /// The SHA-256 Hash
    hash: Option<Base64<Standard, [u8; 32]>>,
    /// The optional mime-type
    mime: Option<MimeWrapper>,
    /// Optional content length
    size: Option<UInt>,
    /// Thumbnail height
    th: Option<UInt>,
    /// Thumbnail width
    tw: Option<UInt>,
    /// Thumbnail method,
    tm: Option<Method>,
}

fn android_windows_to_common_uri(raw_uri: &Uri) -> anyhow::Result<Uri> {
    let mut split_iter = raw_uri.path_and_query().unwrap().as_str().split("/");

    // burn the first /
    split_iter.next();
    let new_uri = Uri::builder()
        .scheme("mxc")
        .authority(
            split_iter
                .next()
                .ok_or(anyhow!("Missing authority in URI"))?,
        )
        .path_and_query(format!(
            "/{}",
            split_iter
                .next()
                .ok_or(anyhow!("Missing path and query in URI"))?
        ));
    new_uri.build().map_err(anyhow::Error::from)
}

/// Initialize the NDK context.
///
/// This JNI function prepares the NDK context for use by this crate.
/// (Tauri no longer does this for us.)
///
/// This function is called from the main activity's `onCreate` method,
/// which has to be altered after being generated by Tauri. It's defined
/// as part of the MainActivity class.
/// ```
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_matrix_svelte_client_MainActivity_initNdkContext(
    env: jni::JNIEnv,
    _class: jni::objects::JObject,
    context: jni::objects::JObject,
) {
    use jni::objects::GlobalRef;
    use std::ffi::c_void;
    use std::sync::OnceLock;
    static REF: OnceLock<Option<GlobalRef>> = OnceLock::new();
    REF.get_or_init(|| match env.new_global_ref(&context) {
        Ok(ref_) => {
            let vm = env.get_java_vm().unwrap();
            let vm = vm.get_java_vm_pointer() as *mut c_void;
            unsafe {
                ndk_context::initialize_android_context(vm, ref_.as_obj().as_raw() as _);
            }
            Some(ref_)
        }
        Err(e) => {
            tracing::error!(%e, "error creating global reference for context");
            tracing::debug!(?e);
            None
        }
    });
}

// Android-only background (killed-state) silent-push handling: a JNI entry the
// FCM service calls when there is no Tauri runtime. See the module docs.
#[cfg(target_os = "android")]
mod android_push;

/// Simulates handling a *silent* (data-only) push for a Matrix-style client on
/// the **warm** path — i.e. while the app/Tauri runtime is alive, driven by
/// `on_silent_push`. Here we can use the plugin builder directly.
///
/// The **killed** path can't use the builder (no `AppHandle`); it goes through
/// `android_push`'s JNI entry instead, but shares the same `simulate_matrix_fetch`.
#[cfg(target_os = "android")]
fn process_silent_push<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    data: &std::collections::HashMap<String, String>,
) {
    let room_id = data
        .get("room_id")
        .cloned()
        .unwrap_or_else(|| "!unknown:matrix.org".to_string());
    let event_id = data
        .get("event_id")
        .cloned()
        .unwrap_or_else(|| "$unknown".to_string());

    tracing::info!("silent push (warm): fetching event {event_id} in room {room_id}");

    // Stand-in for `GET /_matrix/client/v3/rooms/{room_id}/event/{event_id}`.
    let (sender, body) = android_push::simulate_matrix_fetch(&room_id, &event_id);
    let id = android_push::notification_id_for(&event_id);

    let builder = app
        .notifications()
        .builder()
        .id(id)
        .title(sender)
        .body(body)
        .extra("room_id", room_id)
        .extra("event_id", event_id);

    // `show()` is async on mobile; the silent-push handler runs on a background
    // thread, so spawn the display work rather than blocking it.
    tauri::async_runtime::spawn(async move {
        if let Err(e) = builder.show().await {
            tracing::error!("failed to show notification from silent push: {e}");
        }
    });
}

/// Demo-only command: feed a fake silent push through the same handler the FCM
/// data message would, so the flow is testable without a Firebase backend. In
/// production the identical `process_silent_push` runs from `on_silent_push`.
// Tauri command handlers take owned args by convention (see the plugin's own
// `commands.rs`), and these are only consumed on Android.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(not(target_os = "android"), allow(unused_variables))]
fn simulate_silent_push(app: tauri::AppHandle, room_id: String, event_id: String) {
    #[cfg(target_os = "android")]
    {
        let mut data = std::collections::HashMap::new();
        data.insert("room_id".to_string(), room_id);
        data.insert("event_id".to_string(), event_id);
        process_silent_push(&app, &data);
    }
    #[cfg(not(target_os = "android"))]
    tracing::warn!("simulate_silent_push is Android-only");
}

#[test]
fn reconstruct_custom_uri() {
    let uri =
        Uri::from_static("http://mxc.localhost/matrix.org/mysuperid?iv=MRQMwnE55C0AAAAAAAAAAA");
    let common_uri = Uri::from_static("mxc://matrix.org/mysuperid?iv=MRQMwnE55C0AAAAAAAAAAA");

    let new_uri = android_windows_to_common_uri(&uri).unwrap();

    assert_eq!(new_uri, common_uri);
}
