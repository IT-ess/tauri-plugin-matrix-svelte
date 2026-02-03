export type MediaViewerInfo = {
	filename?: string;
	body?: string;
	mimeType?: string;
	size: number;
	thumbnailInfo?: Promise<{
		blob: Blob | null;
		w: number;
		h: number;
	}>;
};
