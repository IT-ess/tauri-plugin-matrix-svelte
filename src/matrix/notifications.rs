use crossbeam_queue::SegQueue;
use tauri::{AppHandle, Emitter, Runtime};

use crate::{
    matrix::singletons::{broadcast_event, UIUpdateMessage},
    models::matrix::{MatrixSvelteEmitEvent, ToastNotificationRequest},
};

static TOAST_NOTIFICATION: SegQueue<ToastNotificationRequest> = SegQueue::new();

/// Displays a new toast notification with the given message.
///
/// Toast notifications will be shown in the order they were enqueued.
pub fn enqueue_toast_notification(notification: ToastNotificationRequest) {
    TOAST_NOTIFICATION.push(notification);
    broadcast_event(UIUpdateMessage::RefreshUI).expect("Couldn't broadcast event to UI");
}

pub fn process_toast_notifications<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<()> {
    if TOAST_NOTIFICATION.is_empty() {
        return Ok(());
    };
    while let Some(notif) = TOAST_NOTIFICATION.pop() {
        app_handle.emit(MatrixSvelteEmitEvent::ToastNotification.as_str(), notif)?;
    }
    Ok(())
}
