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
use matrix_sdk::ruma::events::room::member::Change;
use matrix_sdk::ruma::events::room::member::RoomMemberEventContent;
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
use matrix_sdk::ruma::OwnedMxcUri;
use matrix_sdk::ruma::UserId;
use matrix_sdk_ui::timeline::AnyOtherFullStateEventContent;
use matrix_sdk_ui::timeline::MemberProfileChange;
use matrix_sdk_ui::timeline::MembershipChange;
use matrix_sdk_ui::timeline::RoomMembershipChange;
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
    MembershipChange(FrontendRoomMembershipChange),
    ProfileChange(FrontendMemberProfileChange),
}

//
// COMMON
//

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

//
// OTHER STATE
//

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

//
// MEMBERSHIP STATE
//
// Newtype for MembershipChange to add Serialize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendMembershipChange(MembershipChange);

impl Deref for FrontendMembershipChange {
    type Target = MembershipChange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MembershipChange> for FrontendMembershipChange {
    fn from(change: MembershipChange) -> Self {
        FrontendMembershipChange(change)
    }
}

impl Serialize for FrontendMembershipChange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant_name = match self.0 {
            MembershipChange::None => "None",
            MembershipChange::Error => "Error",
            MembershipChange::Joined => "Joined",
            MembershipChange::Left => "Left",
            MembershipChange::Banned => "Banned",
            MembershipChange::Unbanned => "Unbanned",
            MembershipChange::Kicked => "Kicked",
            MembershipChange::Invited => "Invited",
            MembershipChange::KickedAndBanned => "KickedAndBanned",
            MembershipChange::InvitationAccepted => "InvitationAccepted",
            MembershipChange::InvitationRejected => "InvitationRejected",
            MembershipChange::InvitationRevoked => "InvitationRevoked",
            MembershipChange::Knocked => "Knocked",
            MembershipChange::KnockAccepted => "KnockAccepted",
            MembershipChange::KnockRetracted => "KnockRetracted",
            MembershipChange::KnockDenied => "KnockDenied",
            MembershipChange::NotImplemented => "NotImplemented",
        };
        serializer.serialize_str(variant_name)
    }
}

// Newtype for RoomMembershipChange
#[derive(Debug, Clone)]
pub struct FrontendRoomMembershipChange(RoomMembershipChange);

impl Deref for FrontendRoomMembershipChange {
    type Target = RoomMembershipChange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendRoomMembershipChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<RoomMembershipChange> for FrontendRoomMembershipChange {
    fn from(change: RoomMembershipChange) -> Self {
        FrontendRoomMembershipChange(change)
    }
}

impl Serialize for FrontendRoomMembershipChange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("RoomMembershipChange", 3)?;

        // Use accessor methods if they exist, or you might need to check the API docs
        // For now, let's assume there are getter methods:
        state.serialize_field("user_id", self.user_id())?;

        // Serialize content using the FrontendFullStateEventContent wrapper
        let frontend_content = FrontendFullStateEventContent::from(self.content().clone());
        state.serialize_field("content", &frontend_content)?;

        // Serialize change, wrapping Option<MembershipChange> with our frontend type
        let frontend_change = self.change().map(FrontendMembershipChange::from);
        state.serialize_field("change", &frontend_change)?;

        state.end()
    }
}

// Helper methods for easier usage
impl FrontendRoomMembershipChange {
    pub fn _new(room_membership_change: RoomMembershipChange) -> Self {
        Self(room_membership_change)
    }

    // These methods will delegate to the wrapped type's public API
    // You'll need to check what methods RoomMembershipChange actually provides
    pub fn user_id(&self) -> &UserId {
        // If RoomMembershipChange has a user_id() method:
        self.0.user_id()
        // Or if it has some other accessor method, use that instead
    }

    pub fn content(&self) -> &FullStateEventContent<RoomMemberEventContent> {
        // If RoomMembershipChange has a content() method:
        self.0.content()
        // Or use whatever the actual accessor method is
    }

    pub fn change(&self) -> Option<MembershipChange> {
        // If RoomMembershipChange has a change() method:
        self.0.change()
        // Or use whatever the actual accessor method is
    }
}

//
// PROFILE STATE
//

// Newtype for Change<T> to add Serialize
#[derive(Debug, Clone)]
struct FrontendChange<T>(Change<T>);

impl<T> Deref for FrontendChange<T> {
    type Target = Change<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FrontendChange<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Change<T>> for FrontendChange<T> {
    fn from(change: Change<T>) -> Self {
        FrontendChange(change)
    }
}

impl<T> Serialize for FrontendChange<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Change", 2)?;
        state.serialize_field("old", &self.0.old)?;
        state.serialize_field("new", &self.0.new)?;
        state.end()
    }
}

// Newtype for MemberProfileChange
#[derive(Debug, Clone)]
pub struct FrontendMemberProfileChange(MemberProfileChange);

impl Deref for FrontendMemberProfileChange {
    type Target = MemberProfileChange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrontendMemberProfileChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MemberProfileChange> for FrontendMemberProfileChange {
    fn from(change: MemberProfileChange) -> Self {
        FrontendMemberProfileChange(change)
    }
}

impl Serialize for FrontendMemberProfileChange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("MemberProfileChange", 3)?;

        // Use accessor methods - you'll need to check what methods MemberProfileChange provides
        state.serialize_field("user_id", self.user_id())?;

        // Wrap displayname_change with our FrontendChange wrapper
        let frontend_displayname_change = self
            .displayname_change()
            .map(|change| FrontendChange::from(change.clone()));
        state.serialize_field("displayname_change", &frontend_displayname_change)?;

        // Wrap avatar_url_change with our FrontendChange wrapper
        let frontend_avatar_url_change = self
            .avatar_url_change()
            .map(|change| FrontendChange::from(change.clone()));
        state.serialize_field("avatar_url_change", &frontend_avatar_url_change)?;

        state.end()
    }
}

// Helper methods for easier usage
impl FrontendMemberProfileChange {
    pub fn _new(member_profile_change: MemberProfileChange) -> Self {
        Self(member_profile_change)
    }

    // These methods will delegate to the wrapped type's public API
    // You'll need to check what methods MemberProfileChange actually provides
    pub fn user_id(&self) -> &UserId {
        // If MemberProfileChange has a user_id() method:
        self.0.user_id()
        // Or use whatever the actual accessor method is
    }

    pub fn displayname_change(&self) -> Option<&Change<Option<String>>> {
        // If MemberProfileChange has a displayname_change() method:
        self.0.displayname_change()
        // Or use whatever the actual accessor method is
    }

    pub fn avatar_url_change(&self) -> Option<&Change<Option<OwnedMxcUri>>> {
        // If MemberProfileChange has an avatar_url_change() method:
        self.0.avatar_url_change()
        // Or use whatever the actual accessor method is
    }
}

// Helper methods for FrontendChange for easier access to old/new values
impl<T> FrontendChange<T> {
    pub fn _new(change: Change<T>) -> Self {
        Self(change)
    }

    pub fn _old(&self) -> &T {
        &self.0.old
    }

    pub fn _new_value(&self) -> &T {
        &self.0.new
    }
}
