use std::borrow::Cow;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};

use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use matrix_sdk_ui::timeline::{EventTimelineItem, TimelineDetails};

use super::{
    requests::{submit_async_request, MatrixRequest},
    singletons::CLIENT,
};

/// Returns the sender's display name if available.
///
/// If not available, and if the `room_id` is provided, this function will
/// submit an async request to fetch the event details.
/// In this case, this will return the event sender's user ID as a string.
pub fn get_or_fetch_event_sender(
    event_tl_item: &EventTimelineItem,
    room_id: Option<&OwnedRoomId>,
) -> String {
    let sender_username = match event_tl_item.sender_profile() {
        TimelineDetails::Ready(profile) => profile.display_name.as_deref(),
        TimelineDetails::Unavailable => {
            if let Some(room_id) = room_id {
                if let Some(event_id) = event_tl_item.event_id() {
                    // TODO: handle
                    submit_async_request(MatrixRequest::FetchDetailsForEvent {
                        room_id: room_id.clone(),
                        event_id: event_id.to_owned(),
                    });
                }
            }
            None
        }
        _ => None,
    }
    .unwrap_or_else(|| event_tl_item.sender().as_str());
    sender_username.to_owned()
}

/// Returns the user ID of the currently logged-in user, if any.
pub fn current_user_id() -> Option<OwnedUserId> {
    CLIENT
        .get()
        .and_then(|c| c.session_meta().map(|m| m.user_id.clone()))
}

/// Removes leading whitespace and HTML whitespace tags (`<p>` and `<br>`) from the given `text`.
pub fn trim_start_html_whitespace(mut text: &str) -> &str {
    let mut prev_text_len = text.len();
    loop {
        text = text
            .trim_start_matches("<p>")
            .trim_start_matches("<br>")
            .trim_start_matches("<br/>")
            .trim_start_matches("<br />")
            .trim_start();

        if text.len() == prev_text_len {
            break;
        }
        prev_text_len = text.len();
    }
    text
}

/// Looks for bare links in the given `text` and converts them into proper HTML links.
pub fn linkify(text: &str, is_html: bool) -> Cow<'_, str> {
    use linkify::{LinkFinder, LinkKind};
    let mut links = LinkFinder::new().links(text).peekable();
    if links.peek().is_none() {
        return Cow::Borrowed(text);
    }

    // A closure to escape text if it's not HTML.
    let escaped = |text| {
        if is_html {
            Cow::from(text)
        } else {
            htmlize::escape_text(text)
        }
    };

    let mut linkified_text = String::new();
    let mut last_end_index = 0;
    for link in links {
        let link_txt = link.as_str();
        // Only linkify the URL if it's not already part of an HTML href attribute.
        let is_link_within_href_attr = text.get(..link.start()).is_some_and(ends_with_href);
        let is_link_within_html_tag = text
            .get(link.end()..)
            .is_some_and(|after| after.trim_end().starts_with("</a>"));

        if is_link_within_href_attr || is_link_within_html_tag {
            linkified_text = format!(
                "{linkified_text}{}",
                text.get(last_end_index..link.end()).unwrap_or_default(),
            );
        } else {
            match link.kind() {
                LinkKind::Url => {
                    linkified_text = format!(
                        "{linkified_text}{}<a href=\"{}\">{}</a>",
                        escaped(text.get(last_end_index..link.start()).unwrap_or_default()),
                        htmlize::escape_attribute(link_txt),
                        htmlize::escape_text(link_txt),
                    );
                }
                LinkKind::Email => {
                    linkified_text = format!(
                        "{linkified_text}{}<a href=\"mailto:{}\">{}</a>",
                        escaped(text.get(last_end_index..link.start()).unwrap_or_default()),
                        htmlize::escape_attribute(link_txt),
                        htmlize::escape_text(link_txt),
                    );
                }
                _ => return Cow::Borrowed(text), // unreachable
            }
        }
        last_end_index = link.end();
    }
    linkified_text.push_str(&escaped(text.get(last_end_index..).unwrap_or_default()));
    Cow::Owned(linkified_text)
}

/// Returns true if the given `text` string ends with a valid href attribute opener.
///
/// An href attribute looks like this: `href="http://example.com"`,.
/// so we look for `href="` at the end of the given string.
///
/// Spaces are allowed to exist in between the `href`, `=`, and `"`.
/// In addition, the quotation mark is optional, and can be either a single or double quote,
/// so this function takes those into account as well.
pub fn ends_with_href(text: &str) -> bool {
    // let mut idx = text.len().saturating_sub(1);
    let mut substr = text.trim_end();
    // Search backwards for a single quote, double quote, or an equals sign.
    match substr.as_bytes().last() {
        Some(b'\'' | b'"') => {
            if substr
                .get(..substr.len().saturating_sub(1))
                .map(|s| {
                    substr = s.trim_end();
                    substr.as_bytes().last() == Some(&b'=')
                })
                .unwrap_or(false)
            {
                substr = &substr[..substr.len().saturating_sub(1)];
            } else {
                return false;
            }
        }
        Some(b'=') => {
            substr = &substr[..substr.len().saturating_sub(1)];
        }
        _ => return false,
    }

    // Now we have found the equals sign, so search backwards for the `href` attribute.
    substr.trim_end().ends_with("href")
}

/// Returns a string representation of the room name or ID.
pub fn room_name_or_id(
    room_name: Option<impl Into<String>>,
    room_id: impl AsRef<RoomId>,
) -> String {
    room_name.map_or_else(
        || format!("Room ID {}", room_id.as_ref()),
        |name| name.into(),
    )
}

pub fn debounce_broadcast<T: Clone + Send + 'static>(
    mut input: broadcast::Receiver<T>,
    duration: Duration,
) -> mpsc::Receiver<T> {
    let (tx, rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let mut last_item: Option<T> = None;

        loop {
            tokio::select! {
                result = input.recv() => {
                    match result {
                        Ok(item) => last_item = Some(item),
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(i)) => {
                            eprintln!("Broadcast receiver missed {i} updates");
                            // Handle lagged receiver - you might want to log this
                            // The receiver was too slow and missed some messages
                            continue;
                        }
                    }
                }

                _ = sleep(duration), if last_item.is_some() => {
                    if let Some(item) = last_item.take() {
                        if tx.send(item).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                }
            }
        }
    });

    rx
}
