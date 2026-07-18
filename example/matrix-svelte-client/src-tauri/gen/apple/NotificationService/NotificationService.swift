import TauriPluginNotificationsNSE

/// Principal class of the Notification Service Extension. All the work —
/// flattening the push payload, resolving the App Group data directory, and
/// calling the Rust handler exported by `silent_push_handler!` in
/// `src/push_handler.rs` — happens in the plugin's base class.
final class NotificationService: TauriNotificationService {}
