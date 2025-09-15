import { isTextMessage } from './message-kinds.js';
import { isMsgLikeContent, type MsgLikeContent } from './message-like.js';
import { type StateEvent } from './state-event.js';

// Base timeline item structure
export type TimelineItem = {
	eventId?: string;
	timestamp?: number; // UInt maps to number in TypeScript
	isOwn: boolean;
	isLocal: boolean;
	abilities: MessageAbilities;
} & TimelineItemData;

export type MessageAbility =
	| 'canReact'
	| 'canReplyTo'
	| 'canEdit'
	| 'canPin'
	| 'canUnpin'
	| 'canDelete'
	| 'hasHtml';

export type MessageAbilities = MessageAbility[];

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
