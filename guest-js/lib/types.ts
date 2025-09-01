import type { RoomId } from './matrix-requests/common.js';
import { type TimelineItem } from './timeline-items/timeline-item.js';

export type MatrixClientConfig = {
	username: string;
	password: string;
	homeserver_url: string;
	client_name: string;
};

// Equivalent to RoomScreen Rust Struct
export type Room = {
	roomId: string;
	roomName: string;
	tlState?: TimelineState;
	members: Record<string, RoomMember>;
	typingUsers: string[];
	doneLoading: boolean;
};

export type TimelineState = {
	roomId: string;
	userPower: UserPowerLevels;
	fullyPaginated: boolean;
	items: TimelineItem[];
	scrolledPastReadMarker: boolean;
	latestOwnUserReceipt?: { ts: number };
};

type UserPowerLevel =
	| 'ban'
	| 'invite'
	| 'kick'
	| 'redact'
	| 'notifyRoom'
	| 'location'
	| 'message'
	| 'reaction'
	| 'roomMessage'
	| 'roomRedaction'
	| 'sticker'
	| 'roomPinnedEvents';

type UserPowerLevels = UserPowerLevel[];

export type RoomMember = {
	name: string;
	maxPowerLevel: number;
	displayNameAmbiguous: boolean;
	isIgnored: boolean;
};

// Equivalent to RoomsList Rust Struct
export type RoomsCollectionType = {
	invitedRooms: Record<string, InvitedRoomInfo>;
	allJoinedRooms: Record<string, JoinedRoomInfo>;
	// displayFilter: ???
	displayedInvitedRooms: string[];
	displayedJoinedRooms: string[];
	status: RoomsCollectionStatus;
	currentActiveRoom: string | null;
	maxKnownRooms?: number;
};

export type RoomsCollectionStatus =
	| { status: 'notLoaded'; message: string }
	| { status: 'loading'; message: string }
	| { status: 'loaded'; message: string }
	| { status: 'error'; message: string };

export type JoinedRoomInfo = {
	roomId: string;
	roomName: string;
	numUnreadMessages: number;
	numUnreadMentions: number;
	canonicalAlias: string | null;
	altAlias: string[] | null;
	tags: any;
	latest: [number, string] | null;
	avatar: string | null;
	hasBeenPaginated: boolean;
	isSelected: boolean;
};

export type InvitedRoomInfo = {
	roomId: string;
	roomName: string;
	canonicalAlias: string | null;
	altAlias: string[] | null;
	roomAvatar: string | null;
	inviterInfo: InviterInfo | null;
	latest: [number, string] | null;
	inviteState: InviteState;
	hasBeenPaginated: boolean;
	isSelected: boolean;
	isDirect: boolean;
};

export type InviteState =
	| 'waitingOnUserInput'
	| 'waitingForJoinResult'
	| 'waitingForLeaveResult'
	| 'waitingForJoinedRoom'
	| 'roomLeft';

export type InviterInfo = {
	userId: string;
	displayName: string;
	avatar: string | null;
};

export type Device = {
	deviceId: string;
	displayName: string;
	isVerified: boolean;
	isVerifiedWithCrossSigning: boolean;
	registrationDate: number;
	guessedType: 'ios' | 'android' | 'web' | 'desktop' | 'unknown';
	isCurrentDevice: boolean;
};

// Search messages

export type SearchConfig = {
	limit: number;
	before_limit: number;
	after_limit: number;
	order_by_recency: boolean;
	room_id: RoomId | null;
	keys: IndexedEventType[];
	next_batch: string | null; // UUID V4
};

type IndexedEventType = 'Message'; // We only index messages for the moment

export type SearchBatch = {
	count: number;
	results: SearchResult[];
	next_batch: string | null; // UUID V4
};

export type SearchResult = {
	score: number;
	event_source: string;
	events_before: string[];
	events_after: string[];
	profile_info: Record<string, SearchProfileInfo>;
};

// The event stored in index mapped to JS
export type SourceEvent = {
	body: string;
	eventId: string;
	senderId: string;
	timestamp: Date;
	roomId: string;
	msgtype: string;
};

export type SearchProfileInfo = {
	displayname: string | null;
	avatar_url: string | null;
};
