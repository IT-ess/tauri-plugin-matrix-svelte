import type { AudioInfo } from './bindings/AudioInfo.js';
import type { AudioMessageEventContent } from './bindings/AudioMessageEventContent.js';
import type { FileInfo } from './bindings/FileInfo.js';
import type { FileMessageEventContent } from './bindings/FileMessageEventContent.js';
import type { ImageInfo } from './bindings/ImageInfo.js';
import type { ImageMessageEventContent } from './bindings/ImageMessageEventContent.js';
import type { MsgLikeContent, StickerEventContent } from './bindings/MsgLikeContent.js';
import type { TimelineItem } from './bindings/TimelineItem.js';
import type { VideoInfo } from './bindings/VideoInfo.js';
import type { VideoMessageEventContent } from './bindings/VideoMessageEventContent.js';

/**
 * Type guard to check if an AudioMessageEventContent has a URL
 */
export function audioMessageSourceIsPlain(
	content: AudioMessageEventContent
): content is AudioMessageEventContent & { url: string } {
	return 'url' in content && typeof content.url === 'string';
}

/**
 * Type guard to check if an VideoMessageEventContent has a URL
 */
export function videoMessageSourceIsPlain(
	content: VideoMessageEventContent
): content is VideoMessageEventContent & { url: string } {
	return 'url' in content && typeof content.url === 'string';
}

/**
 * Type guard to check if an VideoMessageEventContent has a thumbnail with a plain URL source
 */
export function videoMessageInfoThumbnailSourceIsPlain(
	content: VideoInfo
): content is VideoInfo & { thumbnail_url: string } {
	return 'thumbnail_url' in content && typeof content.thumbnail_url === 'string';
}

export function isVideoOrImageInfo(
	content: AudioInfo | FileInfo | VideoInfo | ImageInfo
): content is (VideoInfo | ImageInfo) & { thumbnail_url: string } {
	return 'thumbnail_url' in content && typeof content.thumbnail_url === 'string';
}

/**
 * Type guard to check if an ImageMessageEventContent has a URL
 */
export function imageMessageSourceIsPlain(
	content: ImageMessageEventContent | StickerEventContent
): content is (ImageMessageEventContent | StickerEventContent) & { url: string } {
	return 'url' in content && typeof content.url === 'string';
}

/**
 * Type guard to check if an FileMessageEventContent has a URL
 */
export function fileMessageSourceIsPlain(
	content: FileMessageEventContent
): content is FileMessageEventContent & { url: string } {
	return 'url' in content && typeof content.url === 'string';
}

export function isTextMessage(
	content: MsgLikeContent
): content is MsgLikeContent & { kind: 'text' } {
	return content.kind === 'text';
}

export function isMsgLikeContent(item: TimelineItem): item is TimelineItem & { kind: 'msgLike' } {
	return item.kind === 'msgLike';
}
