const COMMANDS: &[&str] = &[
    "ping",
    "login_and_create_new_session",
    "submit_async_request",
    "fetch_media",
    "fetch_user_profile",
    "watch_notifications",
    "get_devices",
    "verify_device",
    "search_messages",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
