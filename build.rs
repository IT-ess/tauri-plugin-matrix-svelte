const COMMANDS: &[&str] = &[
    "submit_async_request",
    "fetch_media",
    "fetch_user_profile",
    "get_devices",
    "verify_device",
    "submit_matrix_login_request",
    "forward_oauth_login_deeplink",
    "build_client_from_homeserver_url",
    "check_homeserver_auth_type",
    "get_dm_room_from_user_id",
    "check_device_verification",
    "has_backup_setup",
    "restore_backup_with_passphrase",
    "setup_new_backup",
    "search_users",
    "disconnect_and_clear_session",
    "check_if_last_device",
    "is_logged_in",
    "has_session_stored",
    "reset_cross_signing",
    "edit_user_information",
    "upload_media",
    "filter_room_list",
    "define_room_informations",
    "get_dm_room_id_or_create_it",
    "write_media_to_selected_folder",
    "silent_save_matrix_media_to_cache_dir",
    "android_share_matrix_media",
    "register_notifications",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
