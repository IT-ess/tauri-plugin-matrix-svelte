import { isMsgLikeContent, isTextMessage, MsgLikeContent } from './message-like';

// Base timeline item structure
export type TimelineItem = {
	eventId?: string;
	timestamp?: number; // UInt maps to number in TypeScript
	isOwn: boolean;
	isLocal: boolean;
} & TimelineItemData;

// Discriminated union for the different timeline item data variants
export type TimelineItemData =
	| { kind: 'msgLike'; data: MsgLikeContent }
	| { kind: 'virtual'; data: VirtualTimelineItem }
	| { kind: 'stateChange'; data: {} } // TODO - empty for now
	| { kind: 'error'; data: {} } // TODO - empty for now
	| { kind: 'call'; data: {} }; // TODO - empty for now

// Virtual timeline item (referenced but not defined in the Rust code)
export interface VirtualTimelineItem {
	kind: 'dateDivider' | 'timelineStart' | 'readMarker';
}

export const timelineDataGuards = {
	isMsgLikeContent,
	isTextMessage
} as const;
