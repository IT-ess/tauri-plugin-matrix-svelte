use std::ops::Deref;
use std::ops::DerefMut;

use matrix_sdk::ruma::events::policy::rule::room::PolicyRuleRoomEventContent;
use matrix_sdk::ruma::events::policy::rule::server::PolicyRuleServerEventContent;
use matrix_sdk::ruma::events::policy::rule::user::PolicyRuleUserEventContent;
use matrix_sdk::ruma::events::room::aliases::RoomAliasesEventContent;
use matrix_sdk::ruma::events::room::avatar::RoomAvatarEventContent;
use matrix_sdk::ruma::events::room::canonical_alias::RoomCanonicalAliasEventContent;
use matrix_sdk::ruma::events::room::create::RoomCreateEventContent;
use matrix_sdk::ruma::events::room::encryption::RoomEncryptionEventContent;
use matrix_sdk::ruma::events::room::guest_access::RoomGuestAccessEventContent;
use matrix_sdk::ruma::events::room::history_visibility::RoomHistoryVisibilityEventContent;
use matrix_sdk::ruma::events::room::join_rules::RoomJoinRulesEventContent;
use matrix_sdk::ruma::events::room::name::RoomNameEventContent;
use matrix_sdk::ruma::events::room::pinned_events::RoomPinnedEventsEventContent;
use matrix_sdk::ruma::events::room::power_levels::RoomPowerLevelsEventContent;
use matrix_sdk::ruma::events::room::server_acl::RoomServerAclEventContent;
use matrix_sdk::ruma::events::room::third_party_invite::RoomThirdPartyInviteEventContent;
use matrix_sdk::ruma::events::room::tombstone::RoomTombstoneEventContent;
use matrix_sdk::ruma::events::room::topic::RoomTopicEventContent;
use matrix_sdk::ruma::events::space::child::SpaceChildEventContent;
use matrix_sdk::ruma::events::space::parent::SpaceParentEventContent;
use matrix_sdk::ruma::events::FullStateEventContent;
use matrix_sdk::ruma::events::RedactContent;
use matrix_sdk::ruma::events::StaticStateEventContent;
use matrix_sdk_ui::timeline::AnyOtherFullStateEventContent;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind",
    content = "body"
)]
pub enum FrontendStateEvent {
    OtherState(FrontendAnyOtherFullStateEventContent),
}

// Newtype for FullStateEventContent
#[derive(Debug, Clone)]
struct FrontendFullStateEventContent<C>(FullStateEventContent<C>)
where
    C: StaticStateEventContent + RedactContent,
    C::Redacted: std::fmt::Debug + Clone,
    C::PossiblyRedacted: std::fmt::Debug + Clone;

impl<C> Deref for FrontendFullStateEventContent<C>
where
    C: StaticStateEventContent + RedactContent,
    C::Redacted: std::fmt::Debug + Clone,
    C::PossiblyRedacted: std::fmt::Debug + Clone,
{
    type Target = FullStateEventContent<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for FrontendFullStateEventContent<C>
where
    C: StaticStateEventContent + RedactContent,
    C::Redacted: std::fmt::Debug + Clone,
    C::PossiblyRedacted: std::fmt::Debug + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C> From<FullStateEventContent<C>> for FrontendFullStateEventContent<C>
where
    C: StaticStateEventContent + RedactContent,
    C::Redacted: std::fmt::Debug + Clone,
    C::PossiblyRedacted: std::fmt::Debug + Clone,
{
    fn from(content: FullStateEventContent<C>) -> Self {
        FrontendFullStateEventContent(content)
    }
}

// Implement Serialize for the FullStateEventContent wrapper
impl<C> Serialize for FrontendFullStateEventContent<C>
where
    C: StaticStateEventContent + RedactContent + Serialize,
    C::PossiblyRedacted: Serialize + std::fmt::Debug + Clone,
    C::Redacted: Serialize + std::fmt::Debug + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            FullStateEventContent::Original {
                content,
                prev_content,
            } => {
                use serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct("FullStateEventContent", 2)?;
                state.serialize_field("content", content)?;
                state.serialize_field("prev_content", prev_content)?;
                state.end()
            }
            FullStateEventContent::Redacted(redacted) => redacted.serialize(serializer),
        }
    }
}

// Newtype for AnyOtherFullStateEventContent
#[derive(Debug, Clone)]
pub struct FrontendAnyOtherFullStateEventContent(AnyOtherFullStateEventContent);

impl Deref for FrontendAnyOtherFullStateEventContent {
    type Target = AnyOtherFullStateEventContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendAnyOtherFullStateEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<AnyOtherFullStateEventContent> for FrontendAnyOtherFullStateEventContent {
    fn from(content: AnyOtherFullStateEventContent) -> Self {
        FrontendAnyOtherFullStateEventContent(content)
    }
}

// Updated enum using the wrapped FullStateEventContent
#[derive(Clone, Debug, Serialize)]
enum FrontendAnyOtherFullStateEventContentEnum {
    /// m.policy.rule.room
    PolicyRuleRoom(FrontendFullStateEventContent<PolicyRuleRoomEventContent>),

    /// m.policy.rule.server
    PolicyRuleServer(FrontendFullStateEventContent<PolicyRuleServerEventContent>),

    /// m.policy.rule.user
    PolicyRuleUser(FrontendFullStateEventContent<PolicyRuleUserEventContent>),

    /// m.room.aliases
    RoomAliases(FrontendFullStateEventContent<RoomAliasesEventContent>),

    /// m.room.avatar
    RoomAvatar(FrontendFullStateEventContent<RoomAvatarEventContent>),

    /// m.room.canonical_alias
    RoomCanonicalAlias(FrontendFullStateEventContent<RoomCanonicalAliasEventContent>),

    /// m.room.create
    RoomCreate(FrontendFullStateEventContent<RoomCreateEventContent>),

    /// m.room.encryption
    RoomEncryption(FrontendFullStateEventContent<RoomEncryptionEventContent>),

    /// m.room.guest_access
    RoomGuestAccess(FrontendFullStateEventContent<RoomGuestAccessEventContent>),

    /// m.room.history_visibility
    RoomHistoryVisibility(FrontendFullStateEventContent<RoomHistoryVisibilityEventContent>),

    /// m.room.join_rules
    RoomJoinRules(FrontendFullStateEventContent<RoomJoinRulesEventContent>),

    /// m.room.name
    RoomName(FrontendFullStateEventContent<RoomNameEventContent>),

    /// m.room.pinned_events
    RoomPinnedEvents(FrontendFullStateEventContent<RoomPinnedEventsEventContent>),

    /// m.room.power_levels
    RoomPowerLevels(FrontendFullStateEventContent<RoomPowerLevelsEventContent>),

    /// m.room.server_acl
    RoomServerAcl(FrontendFullStateEventContent<RoomServerAclEventContent>),

    /// m.room.third_party_invite
    RoomThirdPartyInvite(FrontendFullStateEventContent<RoomThirdPartyInviteEventContent>),

    /// m.room.tombstone
    RoomTombstone(FrontendFullStateEventContent<RoomTombstoneEventContent>),

    /// m.room.topic
    RoomTopic(FrontendFullStateEventContent<RoomTopicEventContent>),

    /// m.space.child
    SpaceChild(FrontendFullStateEventContent<SpaceChildEventContent>),

    /// m.space.parent
    SpaceParent(FrontendFullStateEventContent<SpaceParentEventContent>),

    // Custom
    Custom,
}

// Implement Serialize for the main wrapper by converting to the serializable enum
impl Serialize for FrontendAnyOtherFullStateEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the inner enum to our serializable version
        let serializable_enum = match &self.0 {
            AnyOtherFullStateEventContent::PolicyRuleRoom(content) => {
                FrontendAnyOtherFullStateEventContentEnum::PolicyRuleRoom(content.clone().into())
            }
            AnyOtherFullStateEventContent::PolicyRuleServer(content) => {
                FrontendAnyOtherFullStateEventContentEnum::PolicyRuleServer(content.clone().into())
            }
            AnyOtherFullStateEventContent::PolicyRuleUser(content) => {
                FrontendAnyOtherFullStateEventContentEnum::PolicyRuleUser(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomAliases(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomAliases(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomAvatar(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomAvatar(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomCanonicalAlias(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomCanonicalAlias(
                    content.clone().into(),
                )
            }
            AnyOtherFullStateEventContent::RoomCreate(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomCreate(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomEncryption(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomEncryption(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomGuestAccess(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomGuestAccess(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomHistoryVisibility(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomHistoryVisibility(
                    content.clone().into(),
                )
            }
            AnyOtherFullStateEventContent::RoomJoinRules(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomJoinRules(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomName(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomName(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomPinnedEvents(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomPinnedEvents(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomPowerLevels(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomPowerLevels(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomServerAcl(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomServerAcl(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomThirdPartyInvite(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomThirdPartyInvite(
                    content.clone().into(),
                )
            }
            AnyOtherFullStateEventContent::RoomTombstone(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomTombstone(content.clone().into())
            }
            AnyOtherFullStateEventContent::RoomTopic(content) => {
                FrontendAnyOtherFullStateEventContentEnum::RoomTopic(content.clone().into())
            }
            AnyOtherFullStateEventContent::SpaceChild(content) => {
                FrontendAnyOtherFullStateEventContentEnum::SpaceChild(content.clone().into())
            }
            AnyOtherFullStateEventContent::SpaceParent(content) => {
                FrontendAnyOtherFullStateEventContentEnum::SpaceParent(content.clone().into())
            }
            _ => FrontendAnyOtherFullStateEventContentEnum::Custom,
        };

        serializable_enum.serialize(serializer)
    }
}
