import { Channel, invoke } from '@tauri-apps/api/core';
import type { EditUserInformationPayload } from './bindings/EditUserInformationPayload.js';
import type { FrontendDevice } from './bindings/FrontendDevice.js';
import type { UserId, DeviceId, RoomId, MxcUri } from './matrix-requests/common.js';
import type { EditRoomInformationPayload } from './bindings/EditRoomInformationPayload.js';
import type { MatrixLoginPayload } from './bindings/MatrixLoginPayload.js';
import type { ProfileModel } from './bindings/ProfileModel.js';
import type { VerifyDeviceEvent } from './bindings/VerifyDeviceEvent.js';
import { createMatrixRequest, type MatrixRequest } from './matrix-requests/requests.js';
import type { RoomModel } from './bindings/RoomModel.js';
import { LoginStore } from './stores/login-store.svelte.js';
import { ProfileStore } from './stores/profiles-store.svelte.js';
import { RoomStore } from './stores/room-store.svelte.js';
import { RoomsCollection } from './stores/rooms-collection.svelte.js';

export function submitMatrixLoginRequest(request: MatrixLoginPayload): Promise<null> {
	return invoke('plugin:matrix-svelte|submit_matrix_login_request', {
		request
	});
}

export function buildClientFromHomeserverUrl(homeserver: string): Promise<null> {
	return invoke('plugin:matrix-svelte|build_client_from_homeserver_url', {
		homeserver
	});
}

export type AuthTypeResponse = 'matrix' | 'oauth' | 'wrongUrl';

export function checkHomeserverAuthType(): Promise<AuthTypeResponse> {
	return invoke<AuthTypeResponse>('plugin:matrix-svelte|check_homeserver_auth_type', {});
}

export function forwardOAuthLoginDeeplink(url: string): Promise<null> {
	return invoke('plugin:matrix-svelte|forward_oauth_login_deeplink', { url });
}

export function submitAsyncRequest(request: MatrixRequest): Promise<null> {
	return invoke('plugin:matrix-svelte|submit_async_request', {
		request
	});
}

export async function getDevices(userId: UserId): Promise<FrontendDevice[]> {
	return await invoke('plugin:matrix-svelte|get_devices', { userId });
}

export function checkDeviceVerification(): Promise<{
	verificationState: 'unknown' | 'verified' | 'unverified';
}> {
	return invoke('plugin:matrix-svelte|check_device_verification', {});
}

export async function hasBackupSetup(): Promise<boolean> {
	return invoke<boolean>('plugin:matrix-svelte|has_backup_setup', {});
}

export function restoreBackupWithPassphrase(passphrase: string): Promise<null> {
	return invoke('plugin:matrix-svelte|restore_backup_with_passphrase', { passphrase });
}

export function setupNewBackup(): Promise<string> {
	return invoke<string>('plugin:matrix-svelte|setup_new_backup', {});
}

export async function verifyDevice(
	onEvent: Channel<VerifyDeviceEvent>,
	userId: UserId,
	deviceId: DeviceId
): Promise<null> {
	return await invoke('plugin:matrix-svelte|verify_device', { onEvent, userId, deviceId });
}

export function getDMRoomFromUserId(userId: UserId): Promise<RoomId | null> {
	return invoke<RoomId | null>('plugin:matrix-svelte|get_dm_room_from_user_id', { userId });
}

export async function searchUsers(searchTerm: string, limit: number): Promise<ProfileModel[]> {
	return invoke<ProfileModel[]>('plugin:matrix-svelte|search_users', {
		searchTerm,
		limit
	});
}

export async function getAllUserProfiles(): Promise<ProfileModel[]> {
	return invoke<ProfileModel[]>('plugin:matrix-svelte|get_all_user_profiles', {});
}

export async function getAllDMRooms(): Promise<RoomModel[]> {
	return invoke<RoomModel[]>('plugin:matrix-svelte|get_all_dm_rooms', {});
}

/**
 *
 * Logout the current user and clear the session in keyring
 */
export function disconnectAndClearSession(): Promise<null> {
	return invoke('plugin:matrix-svelte|disconnect_and_clear_session', {});
}

/**
 *
 * @returns whether the current device is the user's last one he is connected to
 */
export function checkIfLastDevice(): Promise<boolean> {
	return invoke<boolean>('plugin:matrix-svelte|check_if_last_device', {});
}

/**
 *
 * @returns whether the current client is "active" (~ logged in)
 */
export function isLoggedIn(): Promise<boolean> {
	return invoke<boolean>('plugin:matrix-svelte|is_logged_in', {});
}

/**
 *
 * Give the ability to the user to reset its identity, losing all its historic
 * messages in the process.
 */
export function resetCrossSigning(password: string | null): Promise<null> {
	return invoke('plugin:matrix-svelte|reset_cross_signing', { password });
}

/**
 *
 * Give the ability to the user to reset its identity, losing all its historic
 * messages in the process.
 */
export function editUserInformation(payload: EditUserInformationPayload): Promise<null> {
	return invoke('plugin:matrix-svelte|edit_user_information', { payload });
}

/**
 *
 * @returns an MxcUri of the uploaded media
 */
export function uploadMedia(contentType: string, data: string | ArrayBuffer): Promise<MxcUri> {
	return invoke<MxcUri>('plugin:matrix-svelte|upload_media', { contentType, data });
}

/**
 *
 * @returns an MxcUri of the uploaded media
 */
export function filterRoomList(keywords: string): Promise<null> {
	return invoke('plugin:matrix-svelte|filter_room_list', { keywords });
}

/**
 *
 * Update informations for a given room. Fields that are "null" are just ignored (no-op).
 */
export function defineRoomInformations(payload: EditRoomInformationPayload): Promise<null> {
	return invoke('plugin:matrix-svelte|define_room_informations', { payload });
}

/**
 *
 * Register push notifications on mobile and OS notifications on desktop. On desktop just send empty strings.
 */
export function registerNotifications(token: string, userLanguage: string): Promise<null> {
	return invoke('plugin:matrix-svelte|register_notifications', { token, userLanguage });
}

export { LoginStore, ProfileStore, RoomStore, RoomsCollection, createMatrixRequest };

export * from './bindings/AudioInfo.js';
export * from './bindings/AudioMessageEventContent.js';
export * from './bindings/DeviceGuessedType.js';
export * from './bindings/EditRoomInformationPayload.js';
export * from './bindings/EditUserInformationPayload.js';
export * from './bindings/EmoteMessageEventContent.js';
export * from './bindings/EncryptedFile.js';
export * from './bindings/FileInfo.js';
export * from './bindings/FileMessageEventContent.js';
export * from './bindings/FrontendDevice.js';
export * from './bindings/FrontendRoomMember.js';
export * from './bindings/ImageInfo.js';
export * from './bindings/ImageMessageEventContent.js';
export * from './bindings/InReplyTo.js';
export * from './bindings/InvitedRoomInfo.js';
export * from './bindings/InviterInfo.js';
export * from './bindings/InviteState.js';
export * from './bindings/JoinedRoomInfo.js';
export * from './bindings/JsonWebKey.js';
export * from './bindings/LocationMessageEventContent.js';
export * from './bindings/LoginState.js';
export * from './bindings/MatrixLoginPayload.js';
export * from './bindings/MediaSource.js';
export * from './bindings/Mentions.js';
export * from './bindings/MilliSecondsSinceUnixEpoch.js';
export * from './bindings/MsgLikeContent.js';
export * from './bindings/NoticeMessageEventContent.js';
export * from './bindings/OGTagsModel.js';
export * from './bindings/ProfileModel.js';
export * from './bindings/ReactionInfo.js';
export * from './bindings/ReactionsByKeyBySender.js';
export * from './bindings/RedactMessagePayload.js';
export * from './bindings/Relation.js';
export * from './bindings/Replacement.js';
export * from './bindings/RoomDisplayName.js';
export * from './bindings/RoomHero.js';
export * from './bindings/RoomMessageEventContent.js';
export * from './bindings/RoomMessageEventContentWithoutRelation.js';
export * from './bindings/RoomModel.js';
export * from './bindings/RoomsCollectionStatus.js';
export * from './bindings/RoomScreen.js';
export * from './bindings/RoomsList.js';
export * from './bindings/SendMessagePayload.js';
export * from './bindings/ServerNoticeMessageEventContent.js';
export * from './bindings/TagInfo.js';
export * from './bindings/TagName.js';
export * from './bindings/TextMessageEventContent.js';
export * from './bindings/Thread.js';
export * from './bindings/ThreadSummary.js';
export * from './bindings/ThumbnailInfo.js';
export * from './bindings/TimelineItem.js';
export * from './bindings/TimelineUiState.js';
export * from './bindings/UnstableAmplitude.js';
export * from './bindings/UnstableAudioDetailsContentBlock.js';
export * from './bindings/UnstableVoiceContentBlock.js';
export * from './bindings/VerifyDeviceEvent.js';
export * from './bindings/VideoInfo.js';
export * from './bindings/VideoMessageEventContent.js';
export * from './bindings/VirtualTimelineItem.js';
export * from './tauri-events.js';
export * from './type-guards.js';
export * from './media-cache.svelte.js';
