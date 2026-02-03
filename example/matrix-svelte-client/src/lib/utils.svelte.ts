import { invoke } from '@tauri-apps/api/core';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { m } from './paraglide/messages';
import { goto } from '$app/navigation';
import z from 'zod/v4';
import { profileStore } from '../hooks.client';
import { readFile } from '@tauri-apps/plugin-fs';
import { fileTypeFromBlob } from 'file-type';
import { type RoomDisplayName, uploadMedia } from 'tauri-plugin-matrix-svelte-api';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

// Get initials for avatar fallback
export const getInitials = (name: string) => {
	return name
		.split(' ')
		.map((n) => n[0])
		.join('')
		.toUpperCase();
};

export const checkUserInProfileStore = async (userId: string) => {
	if (!profileStore.state?.[userId]) {
		await invoke('fetch_user_profile', {
			userId: userId,
			roomId: undefined
		});
	}
};

export function roomNameToPlainString(rawName: RoomDisplayName): string {
	switch (rawName.kind) {
		case 'named':
		case 'aliased':
		case 'calculated':
			return rawName.name;
		case 'empty_was':
			return `${m.empty_room_name()} (${rawName.name})`;
		case 'empty':
			return m.empty_room_name();
	}
}

export const roomsListSearchParamsSchema = z.object({
	tab: z.enum(['dm', 'groups', 'invites']).default('dm')
});

export async function gotoRoomsList(selectedTab: 'dm' | 'groups' | 'invites') {
	return await goto(`/?tab=${selectedTab}`);
}

export async function gotoRoom(roomId: string, avatarUri: string | null) {
	return await goto(
		`/room?id=${encodeURIComponent(roomId)}${avatarUri ? '&avatar=' + encodeURIComponent(avatarUri) : ''}#bottomscroll`
	);
}

export async function gotoThread(roomId: string, threadRoot: string, avatarUri: string | null) {
	return await goto(
		`/room/thread?id=${encodeURIComponent(roomId)}&threadRoot=${encodeURIComponent(threadRoot)}${avatarUri ? '&avatar=' + encodeURIComponent(avatarUri) : ''}#bottomscroll`
	);
}

export async function getImageThumbnailBlob(
	blob: Blob,
	width = 300
): Promise<{ blob: Blob | null; w: number; h: number }> {
	const bitmap = await createImageBitmap(blob, {
		resizeWidth: width,
		resizeQuality: 'high'
	});

	const canvas = document.createElement('canvas');
	canvas.width = bitmap.width;
	canvas.height = bitmap.height;
	const ctx = canvas.getContext('bitmaprenderer');
	ctx?.transferFromImageBitmap(bitmap);
	return new Promise<{ blob: Blob | null; w: number; h: number }>((resolve, reject) => {
		try {
			canvas.toBlob(
				(blob) => resolve({ blob, w: canvas.width, h: canvas.height }),
				'image/webp',
				0.8
			);
		} catch (err) {
			reject(err);
		}
	});
}

export async function getVideoThumbnailBlob(
	videoBlob: Blob,
	seekTime = 1
): Promise<{ blob: Blob | null; w: number; h: number; duration: number }> {
	return new Promise<{ blob: Blob | null; w: number; h: number; duration: number }>(
		(resolve, reject) => {
			const url = URL.createObjectURL(videoBlob);
			const video = document.createElement('video');
			video.src = url;
			video.preload = 'metadata';
			video.muted = true;

			video.onloadedmetadata = () => (video.currentTime = seekTime);

			video.onseeked = () => {
				const canvas = document.createElement('canvas');
				canvas.width = video.videoWidth;
				canvas.height = video.videoHeight;
				canvas.getContext('2d')?.drawImage(video, 0, 0);

				canvas.toBlob(
					(blob) => {
						URL.revokeObjectURL(url);
						video.remove();
						resolve({ blob, w: canvas.width, h: canvas.height, duration: video.duration });
					},
					'image/webp',
					0.8
				);
			};

			video.onerror = () => {
				URL.revokeObjectURL(url);
				reject('Video error');
			};
		}
	);
}

export function getThumbnailInfoFromBlob(mediaType: 'video' | 'image' | 'file', blob: Blob) {
	let thumbnailInfo: Promise<{
		blob: Blob | null;
		w: number;
		h: number;
		duration?: number;
	}> | null = null;

	switch (mediaType) {
		case 'video':
			thumbnailInfo = getVideoThumbnailBlob(blob);
			break;
		case 'image':
			thumbnailInfo = getImageThumbnailBlob(blob);
			break;
		case 'file':
			break;
	}
	return thumbnailInfo;
}

export async function getMediaFromFSPath(path: string): Promise<{
	filename: string | undefined;
	mediaSrc: string;
	mxcUriPromise: Promise<string>;
	mediaType: 'video' | 'image' | 'file';
	blob: Blob;
	mime: string;
}> {
	const mediaFile = await readFile(path);
	const filename = path.split('/').pop();
	const blob = new Blob([mediaFile]);
	const res = await fileTypeFromBlob(blob);
	const mediaSrc = URL.createObjectURL(blob);
	if (!res) throw Error("couldn't get filetype for this file");
	const mxcUriPromise = uploadMedia(res.mime, mediaFile.buffer);
	const mediaType = res.mime.includes('video')
		? 'video'
		: res.mime.includes('image')
			? 'image'
			: 'file';
	return { filename, mediaSrc, mxcUriPromise, mediaType, blob, mime: res.mime };
}
