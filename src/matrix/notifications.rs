use crossbeam_queue::SegQueue;

static POPUP_NOTIFICATION: SegQueue<String> = SegQueue::new();

/// Displays a new popup notification with the given message.
///
/// Popup notifications will be shown in the order they were enqueued,
/// and are currently only removed when manually closed by the user.
pub fn enqueue_popup_notification(message: String) {
    POPUP_NOTIFICATION.push(message);
    // Cx::post_action(PopupNotificationAction::Open);
}
