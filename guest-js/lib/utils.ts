import type { SourceEvent } from './types.js';

export const jsonSourceEventToObject = (json: string): SourceEvent => {
	let parsed = JSON.parse(json); // TODO: MAKE THIS SAFE
	return {
		body: parsed.body,
		eventId: parsed.event_id,
		userId: parsed.user_id
	};
};
