import {
	MatrixId,
	EventId,
	RoomAliasId,
	RoomId,
	ServerName,
	UserId,
	TimelineEventItemId
} from './common';
import {
	RoomMessageEventContent,
	sendMessage,
	SendMessageRequest,
	sendTextMessage
} from './message-text';

// Enums and other types
type PaginationDirection = 'backwards' | 'forwards';

interface RoomMemberships {
	// Define based on your Rust RoomMemberships type
	[key: string]: any;
}

interface EditedContent {
	// Define based on your Rust EditedContent type
	[key: string]: any;
}

interface RoomMember {
	// Define based on your Rust RoomMember type
	userId: string;
	displayName?: string;
	avatarUrl?: string;
	// Add other fields as needed
}

// Individual request types matching the Rust enum variants
interface PaginateRoomTimelineRequest {
	event: 'paginateRoomTimeline';
	payload: {
		roomId: RoomId;
		numEvents: number;
		direction: PaginationDirection;
	};
}

interface EditMessageRequest {
	event: 'editMessage';
	payload: {
		roomId: RoomId;
		timelineEventItemId: TimelineEventItemId;
		editedContent: EditedContent;
	};
}

interface FetchDetailsForEventRequest {
	event: 'fetchDetailsForEvent';
	payload: {
		roomId: RoomId;
		eventId: EventId;
	};
}

interface SyncRoomMemberListRequest {
	event: 'syncRoomMemberList';
	payload: {
		roomId: RoomId;
	};
}

interface JoinRoomRequest {
	event: 'joinRoom';
	payload: {
		roomId: RoomId;
	};
}

interface LeaveRoomRequest {
	event: 'leaveRoom';
	payload: {
		roomId: RoomId;
	};
}

interface GetRoomMembersRequest {
	event: 'getRoomMembers';
	payload: {
		roomId: RoomId;
		memberships: RoomMemberships;
		localOnly: boolean;
	};
}

interface GetUserProfileRequest {
	event: 'getUserProfile';
	payload: {
		userId: UserId;
		roomId?: RoomId;
		localOnly: boolean;
	};
}

interface GetNumberUnreadMessagesRequest {
	event: 'getNumberUnreadMessages';
	payload: {
		roomId: RoomId;
	};
}

interface IgnoreUserRequest {
	event: 'ignoreUser';
	payload: {
		ignore: boolean;
		roomMember: RoomMember;
		roomId: RoomId;
	};
}

interface ResolveRoomAliasRequest {
	event: 'resolveRoomAlias';
	payload: RoomAliasId;
}

interface SendTypingNoticeRequest {
	event: 'sendTypingNotice';
	payload: {
		roomId: RoomId;
		typing: boolean;
	};
}

interface SubscribeToTypingNoticesRequest {
	event: 'subscribeToTypingNotices';
	payload: {
		roomId: RoomId;
		subscribe: boolean;
	};
}

interface SubscribeToOwnUserReadReceiptsChangedRequest {
	event: 'subscribeToOwnUserReadReceiptsChanged';
	payload: {
		roomId: RoomId;
		subscribe: boolean;
	};
}

interface ReadReceiptRequest {
	event: 'readReceipt';
	payload: {
		roomId: RoomId;
		eventId: EventId;
	};
}

interface FullyReadReceiptRequest {
	event: 'fullyReadReceipt';
	payload: {
		roomId: RoomId;
		eventId: EventId;
	};
}

interface GetRoomPowerLevelsRequest {
	event: 'getRoomPowerLevels';
	payload: {
		roomId: RoomId;
	};
}

interface ToggleReactionRequest {
	event: 'toggleReaction';
	payload: {
		roomId: RoomId;
		timelineEventId: TimelineEventItemId;
		reaction: string;
	};
}

interface RedactMessageRequest {
	event: 'redactMessage';
	payload: {
		roomId: RoomId;
		timelineEventId: TimelineEventItemId;
		reason?: string;
	};
}

interface GetMatrixRoomLinkPillInfoRequest {
	event: 'getMatrixRoomLinkPillInfo';
	payload: {
		matrixId: MatrixId;
		via: ServerName[];
	};
}

// Union type combining all request types
export type MatrixRequest =
	| PaginateRoomTimelineRequest
	// | EditMessageRequest
	| FetchDetailsForEventRequest
	// | SyncRoomMemberListRequest
	| JoinRoomRequest
	| LeaveRoomRequest
	// | GetRoomMembersRequest
	| GetUserProfileRequest
	| GetNumberUnreadMessagesRequest
	// | IgnoreUserRequest
	| ResolveRoomAliasRequest
	| SendMessageRequest
	| SendTypingNoticeRequest
	| SubscribeToTypingNoticesRequest
	| SubscribeToOwnUserReadReceiptsChangedRequest
	| ReadReceiptRequest
	| FullyReadReceiptRequest
	| GetRoomPowerLevelsRequest
	| ToggleReactionRequest;
// | RedactMessageRequest
// | GetMatrixRoomLinkPillInfoRequest;

// Export individual types as well for convenience
export type {
	PaginateRoomTimelineRequest,
	// EditMessageRequest,
	FetchDetailsForEventRequest,
	// SyncRoomMemberListRequest,
	JoinRoomRequest,
	LeaveRoomRequest,
	// GetRoomMembersRequest,
	GetUserProfileRequest,
	GetNumberUnreadMessagesRequest,
	// IgnoreUserRequest,
	ResolveRoomAliasRequest,
	SendMessageRequest,
	SendTypingNoticeRequest,
	SubscribeToTypingNoticesRequest,
	SubscribeToOwnUserReadReceiptsChangedRequest,
	ReadReceiptRequest,
	FullyReadReceiptRequest,
	GetRoomPowerLevelsRequest,
	ToggleReactionRequest,
	// RedactMessageRequest,
	// GetMatrixRoomLinkPillInfoRequest,
	// Base types
	PaginationDirection,
	RoomMemberships,
	EditedContent,
	RoomMember,
	RoomMessageEventContent
};

// Helper function to create type-safe requests
export const createMatrixRequest = {
	paginateRoomTimeline: (
		payload: PaginateRoomTimelineRequest['payload']
	): PaginateRoomTimelineRequest => ({
		event: 'paginateRoomTimeline',
		payload
	}),

	editMessage: (payload: EditMessageRequest['payload']): EditMessageRequest => ({
		event: 'editMessage',
		payload
	}),

	fetchDetailsForEvent: (
		payload: FetchDetailsForEventRequest['payload']
	): FetchDetailsForEventRequest => ({
		event: 'fetchDetailsForEvent',
		payload
	}),

	syncRoomMemberList: (
		payload: SyncRoomMemberListRequest['payload']
	): SyncRoomMemberListRequest => ({
		event: 'syncRoomMemberList',
		payload
	}),

	joinRoom: (payload: JoinRoomRequest['payload']): JoinRoomRequest => ({
		event: 'joinRoom',
		payload
	}),

	leaveRoom: (payload: LeaveRoomRequest['payload']): LeaveRoomRequest => ({
		event: 'leaveRoom',
		payload
	}),

	getRoomMembers: (payload: GetRoomMembersRequest['payload']): GetRoomMembersRequest => ({
		event: 'getRoomMembers',
		payload
	}),

	getUserProfile: (payload: GetUserProfileRequest['payload']): GetUserProfileRequest => ({
		event: 'getUserProfile',
		payload
	}),

	getNumberUnreadMessages: (
		payload: GetNumberUnreadMessagesRequest['payload']
	): GetNumberUnreadMessagesRequest => ({
		event: 'getNumberUnreadMessages',
		payload
	}),

	ignoreUser: (payload: IgnoreUserRequest['payload']): IgnoreUserRequest => ({
		event: 'ignoreUser',
		payload
	}),

	resolveRoomAlias: (payload: ResolveRoomAliasRequest['payload']): ResolveRoomAliasRequest => ({
		event: 'resolveRoomAlias',
		payload
	}),

	sendMessage,

	/**
  Send message with type m.text
  */
	sendTextMessage,

	sendTypingNotice: (payload: SendTypingNoticeRequest['payload']): SendTypingNoticeRequest => ({
		event: 'sendTypingNotice',
		payload
	}),

	subscribeToTypingNotices: (
		payload: SubscribeToTypingNoticesRequest['payload']
	): SubscribeToTypingNoticesRequest => ({
		event: 'subscribeToTypingNotices',
		payload
	}),

	subscribeToOwnUserReadReceiptsChanged: (
		payload: SubscribeToOwnUserReadReceiptsChangedRequest['payload']
	): SubscribeToOwnUserReadReceiptsChangedRequest => ({
		event: 'subscribeToOwnUserReadReceiptsChanged',
		payload
	}),

	readReceipt: (payload: ReadReceiptRequest['payload']): ReadReceiptRequest => ({
		event: 'readReceipt',
		payload
	}),

	fullyReadReceipt: (payload: FullyReadReceiptRequest['payload']): FullyReadReceiptRequest => ({
		event: 'fullyReadReceipt',
		payload
	}),

	getRoomPowerLevels: (
		payload: GetRoomPowerLevelsRequest['payload']
	): GetRoomPowerLevelsRequest => ({
		event: 'getRoomPowerLevels',
		payload
	}),

	toggleReaction: (payload: ToggleReactionRequest['payload']): ToggleReactionRequest => ({
		event: 'toggleReaction',
		payload
	}),

	redactMessage: (payload: RedactMessageRequest['payload']): RedactMessageRequest => ({
		event: 'redactMessage',
		payload
	}),

	getMatrixRoomLinkPillInfo: (
		payload: GetMatrixRoomLinkPillInfoRequest['payload']
	): GetMatrixRoomLinkPillInfoRequest => ({
		event: 'getMatrixRoomLinkPillInfo',
		payload
	})
};
