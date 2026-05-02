import type { Thumbnail } from 'tauri-plugin-matrix-svelte-api';

export type MediaViewerInfo = {
	filename?: string;
	body?: string;
	mimeType?: string;
	size: number;
	thumbnailInfo: Promise<Thumbnail> | null;
};
