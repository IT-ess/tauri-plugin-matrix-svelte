use bitflags::bitflags;
use matrix_sdk::ruma::OwnedEventId;
use matrix_sdk_ui::timeline::EventTimelineItem;

use crate::matrix::user_power_level::UserPowerLevels;

bitflags! {
    /// Possible actions that the user can perform on a message.
    ///
    /// This is used to determine which buttons to show in the message context menu.
    #[derive(Copy, Clone, Debug)]
    pub struct MessageAbilities: u8 {
        /// Whether the user can react to this message.
        const CanReact = 1 << 0;
        /// Whether the user can reply to this message.
        const CanReplyTo = 1 << 1;
        /// Whether the user can edit this message.
        const CanEdit = 1 << 2;
        /// Whether the user can pin this message.
        const CanPin = 1 << 3;
        /// Whether the user can unpin this message.
        const CanUnpin = 1 << 4;
        /// Whether the user can delete/redact this message.
        const CanDelete = 1 << 5;
        /// Whether this message contains HTML content that the user can copy.
        const HasHtml = 1 << 6;
    }
}
impl MessageAbilities {
    pub fn from_user_power_and_event(
        user_power_levels: &UserPowerLevels,
        event_tl_item: &EventTimelineItem,
        has_html: bool,
    ) -> Self {
        let mut abilities = Self::empty();
        abilities.set(Self::CanEdit, event_tl_item.is_editable());
        // Currently we only support deleting one's own messages.
        if event_tl_item.is_own() {
            abilities.set(Self::CanDelete, user_power_levels.can_redact_own());
        }
        abilities.set(Self::CanReplyTo, event_tl_item.can_be_replied_to());
        abilities.set(Self::CanPin, user_power_levels.can_pin());
        // TODO: currently we don't differentiate between pin and unpin,
        //       but we should first check whether the given message is already pinned
        //       before deciding which ability to set.
        // abilities.set(Self::CanUnPin, user_power_levels.can_pin_unpin());
        abilities.set(Self::CanReact, user_power_levels.can_send_reaction());
        abilities.set(Self::HasHtml, has_html);
        abilities
    }
}

/// Details about the message that define its context menu content.
#[derive(Clone, Debug)]
pub struct MessageDetails {
    /// The Event ID of the message. If `None`, it is an unsent local event.
    pub event_id: Option<OwnedEventId>,
    /// The index of this message in its room's timeline.
    pub item_id: usize,
    /// The event ID of the message that this message is related to, if any,
    /// such as the replied-to message.
    pub related_event_id: Option<OwnedEventId>,
    /// The abilities that the user has on this message.
    pub abilities: MessageAbilities,
}
