import { Channel, invoke } from '@tauri-apps/api/core';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { events } from 'tauri-plugin-matrix-svelte-api';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

// Load avatar from plain mxcUri
export const loadAvatar = async (mxcUri: string) => {
	const chunks: Uint8Array[] = [];
	try {
		let imageSrc: string = '';
		const onEvent = new Channel<events.MediaStreamEvent>();

		onEvent.onmessage = (message) => {
			if (message.event === 'started') {
				return;
			}

			if (message.event === 'chunk') {
				chunks.push(new Uint8Array(message.data.data));
				console.log(`Received chunk: ${message.data.chunkSize} bytes`);
				return;
			}

			if (message.event === 'finished') {
				// Combine all chunks into a single Uint8Array
				const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
				const combined = new Uint8Array(totalLength);
				let offset = 0;

				for (const chunk of chunks) {
					combined.set(chunk, offset);
					offset += chunk.length;
				}

				// Create blob URL for display
				const blob = new Blob([combined]);
				imageSrc = URL.createObjectURL(blob);
				console.log(`Image fetch completed: ${message.data.totalBytes} bytes`);
				return;
			}

			if (message.event === 'error') {
				console.error('Image fetch error:', message.data.message);
				return;
			}
		};

		await invoke('plugin:matrix-svelte|fetch_media', {
			mediaRequest: {
				format: 'File', // Maybe switch to a thumbnail instead ?
				source: { url: mxcUri }
			},
			onEvent
		});
		return imageSrc;
	} catch (err) {
		throw new Error(`Invoke error: ${err}`);
	}
};
