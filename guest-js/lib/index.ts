import { invoke } from '@tauri-apps/api/core';
import type {
	Device,
	InvitedRoomInfo,
	InviterInfo,
	InviteState,
	JoinedRoomInfo,
	MatrixClientConfig,
	Room,
	RoomsCollectionType,
	SearchBatch,
	SearchConfig,
	SearchResult,
	TimelineState
} from './types.js';
import { RoomStore } from './stores/room-store.svelte';
import { RoomsCollection } from './stores/rooms-collection.svelte';
import { LoginStore } from './stores/login-store.svelte';
import { ProfileStore } from './stores/profiles-store.svelte';
import * as events from './tauri-events.js';
import { type MatrixRequest, createMatrixRequest } from './matrix-requests/requests.js';
import {
	timelineDataGuards,
	type TimelineItem,
	type TimelineItemData,
	type VirtualTimelineItem
} from './timeline-items/timeline-item.js';
import type { MsgLikeContent, ReactionsByKeyBySender } from './timeline-items/message-like.js';
import type { MediaRequestParameters } from './matrix-requests/media.js';
import { fetchMedia, MediaLoadingState } from './media-cache.svelte.js';
import { watchNotifications } from './notifications/ios-notifications.js';
import type {
	AudioMessageEventContent,
	EmoteMessageEventContent,
	FileMessageEventContent,
	ImageMessageEventContent,
	StickerEventContent,
	TextMessageEventContent,
	VideoMessageEventContent
} from './timeline-items/message-kinds.js';
import type { StateEvent } from './timeline-items/state-event.js';
import type { UserId, DeviceId } from './matrix-requests/common.js';
import { jsonSourceEventToObject } from './utils.js';

export async function loginAndCreateNewSession(config: MatrixClientConfig): Promise<null> {
	return await invoke('plugin:matrix-svelte|login_and_create_new_session', {
		config
	});
}

export async function submitAsyncRequest(request: MatrixRequest): Promise<null> {
	return await invoke('plugin:matrix-svelte|submit_async_request', {
		request
	});
}

export async function getDevices(userId: UserId): Promise<Device[]> {
	return await invoke('plugin:matrix-svelte|get_devices', { userId });
}

export async function verifyDevice(userId: UserId, deviceId: DeviceId): Promise<null> {
	return await invoke('plugin:matrix-svelte|verify_device', { userId, deviceId });
}

export async function searchMessages(
	searchTerm: string,
	searchConfig: SearchConfig
): Promise<SearchBatch> {
	return await invoke<SearchBatch>('plugin:matrix-svelte|search_messages', {
		searchTerm,
		searchConfig
	});
}

export {
	type Room,
	type RoomsCollectionType,
	type MatrixClientConfig,
	RoomStore,
	RoomsCollection,
	type JoinedRoomInfo,
	LoginStore,
	ProfileStore,
	events,
	type TimelineItem,
	type TimelineItemData,
	timelineDataGuards,
	type MsgLikeContent,
	type StateEvent,
	type TextMessageEventContent,
	type EmoteMessageEventContent,
	type ImageMessageEventContent,
	type AudioMessageEventContent,
	type FileMessageEventContent,
	type VideoMessageEventContent,
	type StickerEventContent,
	type MediaRequestParameters,
	type ReactionsByKeyBySender,
	type VirtualTimelineItem,
	type TimelineState,
	createMatrixRequest,
	type MatrixRequest,
	type InvitedRoomInfo,
	type InviteState,
	type InviterInfo,
	watchNotifications,
	MediaLoadingState,
	fetchMedia,
	type Device,

	// Search
	type SearchConfig,
	type SearchBatch,
	type SearchResult,
	jsonSourceEventToObject
};
