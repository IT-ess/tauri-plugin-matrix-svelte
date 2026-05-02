import type { SendTextMessagePayload } from '../bindings/SendTextMessagePayload.js';
import type { EventId, RoomId } from './common.js';

export interface SendTextMessageRequest {
	event: 'sendTextMessage';
	payload: SendTextMessagePayload;
}

export const sendTextMessage = (
	roomId: RoomId,
	threadRootEventId: EventId | null,
	body: string,
	replyToId?: EventId
): SendTextMessageRequest => ({
	event: 'sendTextMessage',
	payload: {
		roomId,
		threadRootEventId,
		message: body,
		replyToId: replyToId ?? null
	}
});
