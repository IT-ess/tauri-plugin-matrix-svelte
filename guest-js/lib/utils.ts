import type { SourceEvent } from './types.js';

export const jsonSourceEventToObject = (json: string): SourceEvent => {
	const parsed = JSON.parse(json); // TODO: MAKE THIS SAFE
	return {
		body: parsed.body,
		eventId: parsed.event_id,
		senderId: parsed.sender_id,
		timestamp: new Date(parsed.timestamp),
		roomId: parsed.room_id,
		msgtype: parsed.msgtype
	};
};
