export type RedactMessagePayload = {
	roomId: string;
	threadRootEventId: string | null;
	timelineEventItemId: string;
	reason: string | null;
};
