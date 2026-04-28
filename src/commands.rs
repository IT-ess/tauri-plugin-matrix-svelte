use anyhow::anyhow;
use matrix_ui_serializable::commands::{OwnedEventId, VerifyDeviceEvent};
use matrix_ui_serializable::models::events::{
    FrontendDevice, MatrixLoginPayload, MediaStreamEvent,
};
use matrix_ui_serializable::models::misc::{
    EditRoomInformationPayload, EditUserInformationPayload,
};
use matrix_ui_serializable::models::profile::ProfileModel;
use matrix_ui_serializable::{
    FrontendTimelineItem, FrontendVerificationState, MatrixRequest, MediaRequestParameters,
    OwnedDeviceId, OwnedMxcUri, OwnedRoomId, OwnedUserId, UserProfile, oneshot,
};
use mime_serde_shim::Wrapper as MimeWrapper;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::{AppHandle, Listener, Runtime, command};
use url::Url;

use crate::keyring::clear_session_in_keyring;
use crate::state_updaters::Updaters;
use crate::utils::{get_app_dir_or_create_it, get_plugin_config};
use crate::{AUTH_DEEPLINK_SENDER, Error};
use crate::{LOGIN_SENDER, Result};

#[command]
pub(crate) async fn submit_matrix_login_request(request: MatrixLoginPayload) -> Result<()> {
    let sender = LOGIN_SENDER
        .get()
        .expect("sender should be defined at this point");
    sender
        .send(request)
        .await
        .expect("couldn't send login request to receiver");
    Ok(())
}

#[command]
pub(crate) fn forward_oauth_login_deeplink(url: Url) {
    let sender = AUTH_DEEPLINK_SENDER
        .get()
        .expect("sender should be defined at this point");
    sender
        .blocking_send(url)
        .expect("couldn't send deeplink payload to receiver")
}

#[command]
pub async fn build_client_from_homeserver_url(homeserver: String) -> Result<()> {
    matrix_ui_serializable::commands::build_temp_client_from_homeserver_url(homeserver)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub async fn check_homeserver_auth_type()
-> Result<matrix_ui_serializable::commands::FrontendAuthTypeResponse> {
    matrix_ui_serializable::commands::check_homeserver_auth_type()
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn submit_async_request(request: MatrixRequest) {
    matrix_ui_serializable::commands::submit_async_request(request)
}

async fn fetch_media_helper(
    media_request: MediaRequestParameters,
    on_event: &Channel<MediaStreamEvent>,
) -> anyhow::Result<usize> {
    let (tx, rx) = matrix_ui_serializable::oneshot::channel();
    matrix_ui_serializable::commands::submit_async_request(MatrixRequest::FetchMedia {
        media_request,
        content_sender: tx,
    });

    let image_data: Vec<u8> = match rx.await {
        Ok(data) => match data {
            Ok(data) => data,
            Err(e) => return Err(anyhow!("Failed to fetch image: {}", e)),
        },
        Err(e) => return Err(anyhow!("Media receiver failed: {}", e)),
    };

    // Stream the image in chunks of 8KB
    const CHUNK_SIZE: usize = 8192;
    let mut bytes_sent = 0;

    for chunk in image_data.chunks(CHUNK_SIZE) {
        bytes_sent += chunk.len();

        if let Err(e) = on_event.send(MediaStreamEvent::Chunk {
            data: chunk.to_vec(),
            chunk_size: chunk.len(),
            bytes_received: bytes_sent,
        }) {
            return Err(anyhow!("Failed to send media chunk: {}", e));
        }
    }
    Ok(bytes_sent)
}

#[tauri::command]
pub(crate) async fn fetch_media(
    media_request: MediaRequestParameters,
    on_event: Channel<MediaStreamEvent>,
) -> Result<()> {
    on_event
        .send(MediaStreamEvent::Started)
        .map_err(anyhow::Error::from)?;

    match fetch_media_helper(media_request, &on_event).await {
        Ok(total_bytes) => {
            on_event
                .send(MediaStreamEvent::Finished { total_bytes })
                .map_err(anyhow::Error::from)?;
            Ok(())
        }
        Err(e) => {
            on_event
                .send(MediaStreamEvent::Error {
                    message: e.to_string(),
                })
                .map_err(anyhow::Error::from)?;
            Err(Error::Anyhow(e))
        }
    }
}

#[command]
pub(crate) async fn fetch_user_profile(
    user_id: OwnedUserId,
    room_id: Option<OwnedRoomId>,
) -> Result<UserProfile> {
    matrix_ui_serializable::commands::fetch_user_profile(user_id, room_id.as_ref())
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) async fn get_devices(user_id: OwnedUserId) -> Result<Vec<FrontendDevice>> {
    matrix_ui_serializable::commands::get_devices(&user_id)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn get_dm_room_from_user_id(user_id: OwnedUserId) -> Result<Option<OwnedRoomId>> {
    matrix_ui_serializable::commands::get_dm_room_from_user_id(&user_id).map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn check_device_verification() -> FrontendVerificationState {
    matrix_ui_serializable::commands::check_device_verification()
}

#[command]
pub(crate) async fn has_backup_setup() -> Result<bool> {
    matrix_ui_serializable::commands::has_backup_setup()
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) async fn restore_backup_with_passphrase(passphrase: String) -> Result<()> {
    matrix_ui_serializable::commands::restore_backup_with_passphrase(passphrase)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) async fn setup_new_backup() -> Result<String> {
    matrix_ui_serializable::commands::setup_new_backup()
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) async fn verify_device<R: Runtime>(
    app_handle: AppHandle<R>,
    on_event: Channel<VerifyDeviceEvent>,
    user_id: OwnedUserId,
    device_id: OwnedDeviceId,
) -> Result<()> {
    let (cancel_verif_tx, cancel_verif_rx) = matrix_ui_serializable::oneshot::channel();
    let cancel_tx = Arc::new(Mutex::new(Some(cancel_verif_tx)));
    let cancel_tx_clone = cancel_tx.clone();
    app_handle.listen("cancel-verification", move |_| {
        let mut tx_guard = cancel_tx_clone.lock().expect("Failed to lock mutex");
        if let Some(tx) = tx_guard.take() {
            tx.send(()).expect("couldn't cancel running verification");
        }
    });

    let (status_tx, status_rx) = channel::<VerifyDeviceEvent>();

    tauri::async_runtime::spawn(async move {
        while let Ok(ev) = status_rx.recv() {
            on_event.send(ev).expect("couldn't send event");
        }
    });
    matrix_ui_serializable::commands::verify_device(user_id, device_id, cancel_verif_rx, status_tx)
        .await
        .map_err(|e| e.into())
}

#[command]
pub(crate) async fn search_users(search_term: String, limit: u64) -> Result<Vec<ProfileModel>> {
    let (tx, rx) = matrix_ui_serializable::oneshot::channel();
    matrix_ui_serializable::commands::submit_async_request(MatrixRequest::SearchUsers {
        search_term,
        limit,
        content_sender: tx,
    });

    Ok(rx
        .await
        .map_err(anyhow::Error::from)?
        .map_err(anyhow::Error::from)?)
}

#[command]
pub(crate) async fn disconnect_and_clear_session<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<()> {
    matrix_ui_serializable::commands::disconnect_user().await?;
    clear_session_in_keyring(
        get_app_dir_or_create_it(&app_handle).expect("app data dir should be defined"),
    )
}

#[command]
pub(crate) async fn check_if_last_device() -> Result<bool> {
    matrix_ui_serializable::commands::check_if_last_device()
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn is_logged_in() -> bool {
    matrix_ui_serializable::commands::is_logged_in()
}

#[command(async)]
// Run in a new thread so we don't deadlock
// the frontend
pub(crate) fn has_session_stored() -> bool {
    matrix_ui_serializable::commands::has_session_stored()
}

#[command]
pub(crate) async fn reset_cross_signing(password: Option<String>) -> Result<()> {
    matrix_ui_serializable::commands::reset_cross_signing(password)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) async fn edit_user_information<R: Runtime>(
    app_handle: AppHandle<R>,
    payload: EditUserInformationPayload,
) -> Result<()> {
    let updaters = Updaters::new(app_handle);
    matrix_ui_serializable::commands::edit_user_information(payload, Arc::new(Box::new(updaters)))
        .await
        .map_err(|e| e.into())
}

#[command]
pub(crate) async fn upload_media(content_type: MimeWrapper, data: Vec<u8>) -> Result<OwnedMxcUri> {
    matrix_ui_serializable::commands::upload_media(content_type.0, data)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn filter_room_list(keywords: String) {
    matrix_ui_serializable::commands::filter_room_list(keywords)
}

#[command]
pub(crate) async fn define_room_informations(payload: EditRoomInformationPayload) -> Result<()> {
    matrix_ui_serializable::commands::define_room_informations(payload)
        .await
        .map_err(Error::MatrixLib)
}

#[command]
pub(crate) fn get_dm_room_id_or_create_it(user_id: OwnedUserId) -> Option<OwnedRoomId> {
    matrix_ui_serializable::commands::get_dm_room_id_or_create_it(user_id)
}

async fn get_media_and_infer_filename<'a>(
    media_request: MediaRequestParameters,
    filename: String,
) -> Result<(Vec<u8>, &'a str, &'a str, String)> {
    let (tx, rx) = oneshot::channel();
    matrix_ui_serializable::commands::submit_async_request(MatrixRequest::FetchMedia {
        media_request,
        content_sender: tx,
    });
    let contents = rx
        .await
        .map_err(anyhow::Error::from)?
        .map_err(anyhow::Error::from)?;
    let (kind, mimetype) = infer::get(&contents)
        .map(|k| (k.extension(), k.mime_type()))
        .unwrap_or(("", ""));

    let filename_path = Path::new(&filename).with_extension(kind);
    let filename = filename_path.to_str().unwrap_or("refs_file").to_owned();
    Ok((contents, kind, mimetype, filename))
}

#[command(async)]
pub(crate) async fn write_media_to_selected_folder<R: Runtime>(
    app_handle: AppHandle<R>,
    media_request: MediaRequestParameters,
    filename: String,
) -> Result<String> {
    let (contents, _kind, _mimetype, filename) =
        get_media_and_infer_filename(media_request, filename).await?;

    // Android File API is more complex, so we use a dedicated plugin.
    #[cfg(target_os = "android")]
    {
        use std::io::Write;
        use tauri_plugin_android_fs::{AndroidFsExt, PublicGeneralPurposeDir};
        let android_api = app_handle.android_fs_async();

        if !android_api
            .public_storage()
            .request_permission()
            .await
            .map_err(anyhow::Error::from)?
        {
            return Err(Error::Anyhow(anyhow!("Permission denied by user")));
        }

        let initial_location = android_api
            .public_storage()
            .resolve_initial_location(None, PublicGeneralPurposeDir::Download, "", true)
            .await
            .map_err(anyhow::Error::from)?;

        let selected_path = android_api
            .file_picker()
            .save_file(Some(&initial_location), filename, Some(_mimetype), true)
            .await
            .map_err(anyhow::Error::from)?;

        if let Some(path) = selected_path {
            let mut file: std::fs::File = android_api
                .open_file_writable(&path)
                .await
                .map_err(anyhow::Error::from)?;

            file.write_all(&contents)?;
            Ok(path.uri)
        } else {
            Err(Error::Anyhow(anyhow!("No file path has been selected")))
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        use tauri::Manager;
        use tauri_plugin_dialog::DialogExt;

        let selected_path = app_handle
            .dialog()
            .file()
            .set_directory(app_handle.path().download_dir()?)
            .set_file_name(filename)
            .add_filter("Refs", &[_kind])
            .blocking_save_file()
            .ok_or(anyhow!("No path was selected"))?
            .into_path()
            .map_err(anyhow::Error::from)?;

        if let Err(e) = std::fs::write(&selected_path, contents) {
            tracing::error!("Couldn't write file to given path. {e}");
            Err(Error::Io(e))
        } else {
            Ok(selected_path.to_string_lossy().to_string())
        }
    }
}

#[command(async)]
pub(crate) async fn silent_save_matrix_media_to_cache_dir<R: Runtime>(
    app_handle: AppHandle<R>,
    media_request: MediaRequestParameters,
    filename: String,
) -> Result<String> {
    let (contents, _, _mimetype, filename) =
        get_media_and_infer_filename(media_request, filename).await?;
    // Android File API is more complex, so we use a dedicated plugin.
    #[cfg(target_os = "android")]
    {
        use std::io::Write;
        use tauri_plugin_android_fs::{AndroidFsExt, FileUri};

        let android_api = app_handle.android_fs_async();

        let app_storage = android_api.app_storage();
        let cache_dir = app_storage
            .resolve_path(None, tauri_plugin_android_fs::AppDir::Cache)
            .await
            .map_err(anyhow::Error::from)?;
        let cache_dir_uri = FileUri::from_path(cache_dir);
        let file_uri = android_api
            .create_new_file(&cache_dir_uri, filename, Some(_mimetype))
            .await
            .map_err(|e| Error::Anyhow(anyhow::Error::from(e)))?;
        let mut file: std::fs::File = android_api
            .open_file_writable(&file_uri)
            .await
            .map_err(anyhow::Error::from)?;

        file.write_all(&contents)?;
        Ok(file_uri.uri)
    }
    #[cfg(not(target_os = "android"))]
    {
        use tauri::Manager;

        let path = app_handle.path().app_cache_dir()?.join(filename);
        std::fs::write(&path, &contents)?;
        Ok(path.to_string_lossy().to_string())
    }
}

#[cfg(target_os = "android")]
#[command(async)]
pub(crate) async fn android_share_matrix_media<R: Runtime>(
    app_handle: AppHandle<R>,
    media_request: MediaRequestParameters,
    filename: String,
) -> Result<()> {
    use tauri_plugin_android_fs::{AndroidFsExt, PublicGeneralPurposeDir};

    let android_api = app_handle.android_fs_async();
    let (contents, _, mimetype, filename) =
        get_media_and_infer_filename(media_request, filename).await?;

    if !android_api
        .public_storage()
        .request_permission()
        .await
        .map_err(anyhow::Error::from)?
    {
        return Err(Error::Anyhow(anyhow!("Permission denied by user")));
    }

    let file_uri = android_api
        .public_storage()
        .write_new(
            None, // Storage volume (e.g. internal storage, SD card). If none, primary one
            PublicGeneralPurposeDir::Download,
            filename,
            Some(mimetype),
            &contents,
        )
        .await
        .map_err(anyhow::Error::from)?;

    android_api
        .file_opener()
        .share_file(&file_uri)
        .await
        .map_err(|e| Error::Anyhow(anyhow::Error::from(e)))
}

#[command(async)]
/// Get or fetch an event from a room's main timeline
/// This can be used to get the thread root event when displaying
/// a thread timeline.
pub(crate) async fn get_event_from_main_timeline(
    room_id: OwnedRoomId,
    event_id: OwnedEventId,
) -> Result<FrontendTimelineItem> {
    matrix_ui_serializable::commands::get_event_from_main_timeline(room_id, event_id)
        .await
        .map_err(Into::into)
}

#[command]
/// For mobile we require a token and the user language (i.e. en or en-EN)
pub(crate) async fn register_notifications<R: Runtime>(
    app_handle: AppHandle<R>,
    token: String,
    user_language: String,
) -> Result<()> {
    let app_id = app_handle.config().identifier.clone();
    let plugin_config = get_plugin_config(&app_handle)?;
    matrix_ui_serializable::commands::register_notifications(
        token,
        user_language,
        plugin_config.android_sygnal_gateway_url,
        plugin_config.ios_sygnal_gateway_url,
        app_id,
    )
    .await
    .map_err(|e| e.into())
}
