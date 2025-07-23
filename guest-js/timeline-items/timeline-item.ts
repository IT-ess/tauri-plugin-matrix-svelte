import { isTextMessage } from './message-kinds';
import { isMsgLikeContent, MsgLikeContent } from './message-like';
import { StateEvent } from './state-event';

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
	| { kind: 'stateChange'; data: StateEvent }
	| { kind: 'error'; data: TimelineErrorItem }
	| { kind: 'call' };

// Virtual timeline item (referenced but not defined in the Rust code)
export interface VirtualTimelineItem {
	kind: 'dateDivider' | 'timelineStart' | 'readMarker';
}

export interface TimelineErrorItem {
	error: string;
}

export const timelineDataGuards = {
	isMsgLikeContent,
	isTextMessage
} as const;
