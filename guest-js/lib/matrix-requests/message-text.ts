import type { Mentions } from '../bindings/Mentions.js';
import type { Relation } from '../bindings/Relation.js';
import type { RoomMessageEventContentWithoutRelation } from '../bindings/RoomMessageEventContentWithoutRelation.js';
import type { SendMessagePayload } from '../bindings/SendMessagePayload.js';
import type { EventId, RoomId } from './common.js';

export interface SendMessageRequest {
	event: 'sendMessage';
	payload: SendMessagePayload;
}

export const sendMessage = (payload: SendMessageRequest['payload']): SendMessageRequest => ({
	event: 'sendMessage',
	payload
});

export const sendTextMessage = (
	roomId: RoomId,
	body: string,
	options?: {
		replyToId?: EventId;
		threadRootId?: EventId;
		// formatted?: FormattedBody; TODO: support formatted messages
		mentions?: Mentions;
		relatesTo?: Relation<RoomMessageEventContentWithoutRelation>;
	}
): SendMessageRequest => ({
	event: 'sendMessage',
	payload: {
		roomId,
		message: {
			msgtype: 'm.text',
			body,
			// ...(options?.formatted && { formatted: options.formatted }),
			...(options?.relatesTo && { relates_to: options.relatesTo }),
			...(options?.mentions && { 'm.mentions': options.mentions })
		},
		replyToId: options?.replyToId ?? null,
		threadRootId: options?.threadRootId ?? null
	}
});
