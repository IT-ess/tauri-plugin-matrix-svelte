use mime_serde_shim::Wrapper as MimeWrapper;
use serde::Deserialize;
use tauri::http::{self, HeaderValue, Uri};
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_matrix_svelte::{
    AUTH_DEEPLINK_SENDER, Base64, EncryptedFile, EncryptedFileHashes, EncryptedFileInfo,
    MatrixRequest, MediaFormat, MediaRequestParameters, MediaSource, MediaThumbnailSettings,
    Method, OwnedMxcUri, Standard, UInt, UrlSafe, V2EncryptedFileInfo, oneshot,
    submit_async_request,
};
use tauri_plugin_svelte::CborMarshaler;
use tracing::{debug, error, trace};

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
                    android_windows_to_common_uri(raw_uri)
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

                let mut response = http::Response::builder()
                    .header(http::header::CACHE_CONTROL, "public, max-age=3600");

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

                let (tx, rx) = oneshot::channel();
                submit_async_request(MatrixRequest::FetchMedia {
                    media_request,
                    content_sender: tx,
                });

                match rx.await {
                    Ok(Ok(data)) => match response.status(200).body(data) {
                        Ok(res) => {
                            responder.respond(res);
                            trace!("responded to uri request {}", request.uri());
                        }
                        Err(e) => {
                            error!("Cannot build response. {e}")
                        }
                    },
                    Ok(Err(e)) => {
                        error!("Media error: {e}");
                        responder.respond(
                            http::Response::builder()
                                .status(http::StatusCode::BAD_REQUEST)
                                .header(http::header::CONTENT_TYPE, "text/plain")
                                .body("failed to get media".as_bytes().to_vec())
                                .unwrap(),
                        );
                    }
                    Err(e) => {
                        error!("Channel error: {e}");
                        responder.respond(
                            http::Response::builder()
                                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .header(http::header::CONTENT_TYPE, "text/plain")
                                .body("failed to get media, channel error".as_bytes().to_vec())
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
          debug!("a new app instance was opened with {argv:?} and the deep link event was already triggered");
          // when defining deep link schemes at runtime, you must also check `argv` here

          let _ = app.get_webview_window("main")
                     .expect("no main window")
                     .set_focus();
        }));
    }

    // Init deeplink plugin before tauri_plugin_mobile_sharetarget
    builder = builder.plugin(tauri_plugin_deep_link::init());

    #[cfg(target_os = "ios")]
    {
        builder = builder.plugin(tauri_plugin_web_auth::init());
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
                        tauri_plugin_matrix_svelte::LOGIN_STORE_READY
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
                            debug!("quit menu item was clicked");
                            app.exit(0);
                        }
                        _ => {
                            warn!("menu item {:?} not handled", event.id);
                        }
                    })
                    .build(app)?;
            }

            // Handle scheme:// deeplink

            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link()
                    .register_all()
                    .expect("couldn't register deeplink");
            }

            let deeplink_handle = app.app_handle().clone();

            app.deep_link().on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
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

fn android_windows_to_common_uri(raw_uri: &Uri) -> Uri {
    let mut split_iter = raw_uri.path_and_query().unwrap().as_str().split("/");

    // burn the first /
    split_iter.next();
    let new_uri = Uri::builder()
        .scheme("mxc")
        .authority(split_iter.next().unwrap())
        .path_and_query(format!("/{}", split_iter.next().unwrap()));
    new_uri.build().unwrap()
}

#[test]
fn reconstruct_custom_uri() {
    let uri =
        Uri::from_static("http://mxc.localhost/matrix.org/mysuperid?iv=MRQMwnE55C0AAAAAAAAAAA");
    let common_uri = Uri::from_static("mxc://matrix.org/mysuperid?iv=MRQMwnE55C0AAAAAAAAAAA");

    let new_uri = android_windows_to_common_uri(&uri);

    assert_eq!(new_uri, common_uri);
}
