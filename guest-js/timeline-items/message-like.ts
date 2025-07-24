import { UserId } from '../matrix-requests/common';
import { TimelineItem } from './timeline-item';
import { MsgLikeKind } from './message-kinds';

// Discriminated union for message-like content
export type MsgLikeContent = {
	threadRoot?: string; // OwnedEventId maps to string
	edited: boolean;
	reactions: ReactionsByKeyBySender;
	sender?: string;
	senderId: string;
} & MsgLikeKind;

// Type guards are now optional - TypeScript can infer types automatically
export const isMsgLikeContent = (
	item: TimelineItem
): item is TimelineItem & { kind: 'msgLike' } => {
	return item.kind === 'msgLike';
};

export type ReactionStatus = 'LocalToLocal' | 'LocalToRemote' | 'RemoteToRemote';

export interface ReactionInfo {
	timestamp: number; // assuming MilliSecondsSinceUnixEpoch is a UNIX timestamp in ms
	status: ReactionStatus;
}

// The inner map: sender → reaction info
export type ReactionsBySender = Record<UserId, ReactionInfo>;

// The full map: reaction key → sender map
export type ReactionsByKeyBySender = Record<string, ReactionsBySender>;
