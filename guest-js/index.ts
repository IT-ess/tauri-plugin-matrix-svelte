import { invoke } from '@tauri-apps/api/core';
import {
	InvitedRoomInfo,
	InviterInfo,
	InviteState,
	JoinedRoomInfo,
	MatrixClientConfig,
	Room,
	RoomsCollectionType,
	TimelineState
} from './types';
import { RoomStore } from './stores/room-store.svelte';
import { RoomsCollection } from './stores/rooms-collection.svelte';
import { LoginStore } from './stores/login-store.svelte';
import { ProfileStore } from './stores/profiles-store.svelte';
import * as events from './tauri-events';
import { MatrixRequest, createMatrixRequest } from './matrix-requests/requests';
import {
	timelineDataGuards,
	TimelineItem,
	TimelineItemData,
	VirtualTimelineItem
} from './timeline-items/timeline-item';
import { MsgLikeContent, ReactionsByKeyBySender } from './timeline-items/message-like';
import { MediaRequestParameters } from './matrix-requests/media';
import { fetchMedia, LoadingState, mediaCache } from './media-cache.svelte.js';
import { watchNotifications } from './notifications/ios-notifications';
import {
	AudioMessageEventContent,
	EmoteMessageEventContent,
	FileMessageEventContent,
	ImageMessageEventContent,
	StickerEventContent,
	TextMessageEventContent,
	VideoMessageEventContent
} from './timeline-items/message-kinds';
import { StateEvent } from './timeline-items/state-event';

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

export {
	Room,
	RoomsCollectionType,
	MatrixClientConfig,
	RoomStore,
	RoomsCollection,
	JoinedRoomInfo,
	LoginStore,
	ProfileStore,
	events,
	TimelineItem,
	TimelineItemData,
	timelineDataGuards,
	MsgLikeContent,
	StateEvent,
	TextMessageEventContent,
	EmoteMessageEventContent,
	ImageMessageEventContent,
	AudioMessageEventContent,
	FileMessageEventContent,
	VideoMessageEventContent,
	StickerEventContent,
	MediaRequestParameters,
	ReactionsByKeyBySender,
	VirtualTimelineItem,
	TimelineState,
	createMatrixRequest,
	MatrixRequest,
	mediaCache,
	InvitedRoomInfo,
	InviteState,
	InviterInfo,
	watchNotifications,
	LoadingState,
	fetchMedia
};
