import type { EncryptedFile } from '../bindings/EncryptedFile.js';
import type { UInt, MxcUri, RoomId, EventId } from './common.js';
import type { MediaSource } from '../bindings/MediaSource.js';
import type { AttachmentInfo } from '$lib/bindings/AttachementInfo.js';
import type { Thumbnail } from '$lib/bindings/Thumbnail.js';

export type PlainMediaSource = { url: MxcUri }; // Corresponds to Plain variant, renamed to "url". We don't support Plain for now

export type EncryptedMediaSource = { file: EncryptedFile }; // Corresponds to Encrypted variant, renamed to "file"

// Manual typing needed for ImageInfo and VideoInfo
export type ThumbnailMediaSource = { thumbnail_file: EncryptedFile } | { thumbnail_url: MxcUri };

export function isPlainMediaSource(source: MediaSource): source is PlainMediaSource {
	return typeof source === 'object' && source !== null && 'url' in source;
}

export type MediaFormat = 'File' | { Thumbnail: MediaThumbnailSettings };

export interface MediaThumbnailSettings {
	method: Method;
	width: UInt;
	height: UInt;
	animated: boolean;
}

export type Method = 'crop' | 'scale';

export interface MediaRequestParameters {
	source: MediaSource;
	format: MediaFormat;
}

export type SendMediaMessageRequest = {
	roomId: RoomId;
	threadRoot: EventId | null;
	buffer: string | ArrayBuffer;
	filename: string;
	mimeType: string;
	caption: string | null;
	inReplyTo: EventId | null;
	info: AttachmentInfo;
	thumbnail: Thumbnail | null;
};
