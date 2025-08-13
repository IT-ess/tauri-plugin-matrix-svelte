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
