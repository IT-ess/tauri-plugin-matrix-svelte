import type {
	EncryptedMediaSource,
	MediaSource,
	PlainMediaSource
} from '../matrix-requests/media.js';
import { type MsgLikeContent } from './message-like.js';

export type MsgLikeKind =
	| { kind: 'text'; body: TextMessageEventContent }
	| { kind: 'image'; body: ImageMessageEventContent }
	| { kind: 'audio'; body: AudioMessageEventContent }
	| { kind: 'file'; body: FileMessageEventContent }
	| { kind: 'video'; body: VideoMessageEventContent }
	| { kind: 'sticker'; body: StickerEventContent }
	| { kind: 'emote'; body: EmoteMessageEventContent }
	| { kind: 'location'; body: any }
	| { kind: 'notice'; body: any }
	| { kind: 'serverNotice'; body: any }
	| { kind: 'verificationRequest'; body: any }
	| { kind: 'redacted' }
	| { kind: 'unableToDecrypt' }
	| { kind: 'unknown' };

//
// Kinds
//

// Text & Emote

export type TextMessageEventContent = {
	body: string;
} & FormattedBody;

export type EmoteMessageEventContent = {
	// Exactly the same as text message except msgtype
	body: string;
} & FormattedBody;

// Images

export type ImageMessageEventContent = {
	body: string;
	info?: ImageInfo;
} & EncryptedMediaSource & // We assume that we'll only use encrypted media.
	FormattedBody;

export interface ImageInfo {
	h?: number;
	w?: number;
	mimetype?: string;
	size?: number;
	thumbnail_info?: ThumbnailInfo;
	thumbnail_source?: MediaSource;
	'xyz.amorgan.blurhash'?: string;
	is_animated?: boolean;
}

// Audio

export type AudioMessageEventContent = {
	body: string;
	filename?: string;
	info?: AudioInfo;
} & EncryptedMediaSource & // We assume that we'll only use encrypted media.
	FormattedBody;

export type AudioInfo = {
	duration?: {
		secs: number;
		nanos: number;
	};
	mimetype?: string;
	size?: number;
};

// File

export type FileMessageEventContent = {
	body: string;
	filename?: string;
	info?: FileInfo;
} & EncryptedMediaSource & // We assume that we'll only use encrypted media.
	FormattedBody;

export type FileInfo = {
	mimetype?: string;
	size?: number;
	thumbnail_info: ThumbnailInfo;
} & MediaSource; // The thumbnail source

// Video

export type VideoMessageEventContent = {
	body: string;
	filename?: string;
	info?: VideoInfo;
} & EncryptedMediaSource & // We assume that we'll only use encrypted media.
	FormattedBody;

export type VideoInfo = {
	duration?: {
		secs: number;
		nanos: number;
	};
	mimetype?: string;
	size?: number;
	h?: number;
	w?: number;
	thumbnail_info: ThumbnailInfo;
	'xyz.amorgan.blurhash'?: string;
} & MediaSource; // The thumbnail source

// Sticker

export type StickerEventContent = {
	body: string;
	info: ImageInfo;
} & PlainMediaSource; // Stickers aren't encrypted

//
// Common
//

export interface FormattedBody {
	format?: string;
	formatted_body?: string;
}

export interface ThumbnailInfo {
	h?: number;
	w?: number;
	mimetype?: string;
	size?: number;
}

//
// Type Guards
//

export const isTextMessage = (
	content: MsgLikeContent
): content is MsgLikeContent & { kind: 'text' } => {
	return content.kind === 'text';
};
