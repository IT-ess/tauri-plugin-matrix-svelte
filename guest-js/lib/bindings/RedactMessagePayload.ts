export type RedactMessagePayload = {
	roomId: string;
	threadRootEventId: string | null;
	timelineEventId: string;
	reason: string | null;
};
