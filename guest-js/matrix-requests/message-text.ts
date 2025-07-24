import { EventId, RoomId } from './common';

// Main RoomMessageEventContent matching the Rust struct
export interface RoomMessageEventContent {
	// MessageType is flattened in Rust, so we flatten it here too
	msgtype: MessageType['msgtype'];
	body: string;
	formatted?: FormattedBody;

	// Optional relations and mentions
	relates_to?: Relation;
	'm.mentions'?: Mentions;
}

// Message format
type MessageFormat = 'org.matrix.custom.html'; // TODO: add custom formats

// Formatted body for rich text messages
export interface FormattedBody {
	format: MessageFormat;
	body: string;
}

// Text message content matching Rust TextMessageEventContent
interface TextMessageEventContent {
	msgtype: 'm.text';
	body: string;
	formatted?: FormattedBody;
	// url_previews: ... (unstable)
}

// Message type union (only Text for now, can be extended later)
type MessageType = TextMessageEventContent;

// Mentions structure
export interface Mentions {
	user_ids?: string[];
	room?: boolean;
}

// Relations for message threading/replies (simplified)
export interface Relation {
	rel_type?: string;
	event_id?: string;
	// Add other relation fields as needed
}

export interface SendMessageRequest {
	event: 'sendMessage';
	payload: {
		roomId: RoomId;
		message: RoomMessageEventContent;
		replyToEventId?: EventId;
	};
}

export const sendMessage = (payload: SendMessageRequest['payload']): SendMessageRequest => ({
	event: 'sendMessage',
	payload
});

export const sendTextMessage = (
	roomId: RoomId,
	body: string,
	options?: {
		replyToEventId?: EventId;
		formatted?: FormattedBody;
		mentions?: Mentions;
		relatesTo?: Relation;
	}
): SendMessageRequest => ({
	event: 'sendMessage',
	payload: {
		roomId,
		message: {
			msgtype: 'm.text',
			body,
			...(options?.formatted && { formatted: options.formatted }),
			...(options?.relatesTo && { relates_to: options.relatesTo }),
			...(options?.mentions && { 'm.mentions': options.mentions })
		},
		replyToEventId: options?.replyToEventId
	}
});
