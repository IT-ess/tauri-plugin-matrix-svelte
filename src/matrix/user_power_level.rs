use bitflags::bitflags;
use matrix_sdk::ruma::{
    events::{room::power_levels::RoomPowerLevels, MessageLikeEventType, StateEventType},
    UserId,
};
use serde::{Deserialize, Serialize};

bitflags! {
    /// The powers that a user has in a given room.
    #[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct UserPowerLevels: u64 {
        const Ban = 1 << 0;
        const Invite = 1 << 1;
        const Kick = 1 << 2;
        const Redact = 1 << 3;
        const NotifyRoom = 1 << 4;
        // -------------------------------------
        // -- Copied from TimelineEventType ----
        // -- Unused powers are commented out --
        // -------------------------------------
        // const CallAnswer = 1 << 5;
        // const CallInvite = 1 << 6;
        // const CallHangup = 1 << 7;
        // const CallCandidates = 1 << 8;
        // const CallNegotiate = 1 << 9;
        // const CallReject = 1 << 10;
        // const CallSdpStreamMetadataChanged = 1 << 11;
        // const CallSelectAnswer = 1 << 12;
        // const KeyVerificationReady = 1 << 13;
        // const KeyVerificationStart = 1 << 14;
        // const KeyVerificationCancel = 1 << 15;
        // const KeyVerificationAccept = 1 << 16;
        // const KeyVerificationKey = 1 << 17;
        // const KeyVerificationMac = 1 << 18;
        // const KeyVerificationDone = 1 << 19;
        const Location = 1 << 20;
        const Message = 1 << 21;
        // const PollStart = 1 << 22;
        // const UnstablePollStart = 1 << 23;
        // const PollResponse = 1 << 24;
        // const UnstablePollResponse = 1 << 25;
        // const PollEnd = 1 << 26;
        // const UnstablePollEnd = 1 << 27;
        // const Beacon = 1 << 28;
        const Reaction = 1 << 29;
        // const RoomEncrypted = 1 << 30;
        const RoomMessage = 1 << 31;
        const RoomRedaction = 1 << 32;
        const Sticker = 1 << 33;
        // const CallNotify = 1 << 34;
        // const PolicyRuleRoom = 1 << 35;
        // const PolicyRuleServer = 1 << 36;
        // const PolicyRuleUser = 1 << 37;
        // const RoomAliases = 1 << 38;
        // const RoomAvatar = 1 << 39;
        // const RoomCanonicalAlias = 1 << 40;
        // const RoomCreate = 1 << 41;
        // const RoomEncryption = 1 << 42;
        // const RoomGuestAccess = 1 << 43;
        // const RoomHistoryVisibility = 1 << 44;
        // const RoomJoinRules = 1 << 45;
        // const RoomMember = 1 << 46;
        // const RoomName = 1 << 47;
        const RoomPinnedEvents = 1 << 48;
        // const RoomPowerLevels = 1 << 49;
        // const RoomServerAcl = 1 << 50;
        // const RoomThirdPartyInvite = 1 << 51;
        // const RoomTombstone = 1 << 52;
        // const RoomTopic = 1 << 53;
        // const SpaceChild = 1 << 54;
        // const SpaceParent = 1 << 55;
        // const BeaconInfo = 1 << 56;
        // const CallMember = 1 << 57;
        // const MemberHints = 1 << 58;
    }
}
impl UserPowerLevels {
    pub fn from(power_levels: &RoomPowerLevels, user_id: &UserId) -> Self {
        let mut retval = UserPowerLevels::empty();
        let user_power = power_levels.for_user(user_id);
        retval.set(UserPowerLevels::Ban, user_power >= power_levels.ban);
        retval.set(UserPowerLevels::Invite, user_power >= power_levels.invite);
        retval.set(UserPowerLevels::Kick, user_power >= power_levels.kick);
        retval.set(UserPowerLevels::Redact, user_power >= power_levels.redact);
        retval.set(
            UserPowerLevels::NotifyRoom,
            user_power >= power_levels.notifications.room,
        );
        retval.set(
            UserPowerLevels::Location,
            user_power >= power_levels.for_message(MessageLikeEventType::Location),
        );
        retval.set(
            UserPowerLevels::Message,
            user_power >= power_levels.for_message(MessageLikeEventType::Message),
        );
        retval.set(
            UserPowerLevels::Reaction,
            user_power >= power_levels.for_message(MessageLikeEventType::Reaction),
        );
        retval.set(
            UserPowerLevels::RoomMessage,
            user_power >= power_levels.for_message(MessageLikeEventType::RoomMessage),
        );
        retval.set(
            UserPowerLevels::RoomRedaction,
            user_power >= power_levels.for_message(MessageLikeEventType::RoomRedaction),
        );
        retval.set(
            UserPowerLevels::Sticker,
            user_power >= power_levels.for_message(MessageLikeEventType::Sticker),
        );
        retval.set(
            UserPowerLevels::RoomPinnedEvents,
            user_power >= power_levels.for_state(StateEventType::RoomPinnedEvents),
        );
        retval
    }

    pub fn can_ban(self) -> bool {
        self.contains(UserPowerLevels::Ban)
    }

    pub fn can_unban(self) -> bool {
        self.can_ban() && self.can_kick()
    }

    pub fn can_invite(self) -> bool {
        self.contains(UserPowerLevels::Invite)
    }

    pub fn can_kick(self) -> bool {
        self.contains(UserPowerLevels::Kick)
    }

    pub fn can_redact(self) -> bool {
        self.contains(UserPowerLevels::Redact)
    }

    pub fn can_notify_room(self) -> bool {
        self.contains(UserPowerLevels::NotifyRoom)
    }

    pub fn can_redact_own(self) -> bool {
        self.contains(UserPowerLevels::RoomRedaction)
    }

    pub fn can_redact_others(self) -> bool {
        self.can_redact_own() && self.contains(UserPowerLevels::Redact)
    }

    pub fn can_send_location(self) -> bool {
        self.contains(UserPowerLevels::Location)
    }

    pub fn can_send_message(self) -> bool {
        self.contains(UserPowerLevels::RoomMessage) || self.contains(UserPowerLevels::Message)
    }

    pub fn can_send_reaction(self) -> bool {
        self.contains(UserPowerLevels::Reaction)
    }

    pub fn can_send_sticker(self) -> bool {
        self.contains(UserPowerLevels::Sticker)
    }

    #[doc(alias("unpin"))]
    pub fn can_pin(self) -> bool {
        self.contains(UserPowerLevels::RoomPinnedEvents)
    }
}
