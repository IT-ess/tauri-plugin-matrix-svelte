export type RedactMessagePayload = {
	roomId: string;
	timelineEventId: string;
	reason: string | null;
};
