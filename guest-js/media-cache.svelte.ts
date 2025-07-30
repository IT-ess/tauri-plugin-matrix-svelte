import { invoke } from '@tauri-apps/api/core';
import { Channel } from '@tauri-apps/api/core';
import { fileTypeFromBuffer } from 'file-type';
import type { MediaStreamEvent } from './tauri-events';
import { isPlainMediaSource, MediaRequestParameters } from './matrix-requests/media';

const CACHE_NAME = 'matrix-media-cache';
const CACHE_VERSION = 'v1';
const FULL_CACHE_NAME = `${CACHE_NAME}-${CACHE_VERSION}`;

// Cache configuration
const CACHE_CONFIG = {
	maxAge: 365 * 24 * 60 * 60 * 1000, // 1 year in milliseconds
	maxEntries: 1000 // Maximum number of cached images
};

export type LoadingState = {
	progress: number;
	isLoaded: boolean;
	totalSize: number;
};

export async function fetchMedia(
	mediaRequest: MediaRequestParameters,
	loadingState?: LoadingState
) {
	return mediaCache.get(mediaRequest, loadingState);
}

export class MediaCache {
	private cache: Cache | null = null;
	private pendingRequests: Map<string, Promise<string>> = new Map();

	/**
	 * Initialize the cache
	 */
	async init(): Promise<void> {
		try {
			this.cache = await caches.open(FULL_CACHE_NAME);
			await this.cleanupExpiredEntries();
		} catch (error) {
			console.error('Failed to initialize image cache:', error);
		}
	}

	/**
	 * Main method: Get media from cache or fetch it if not available
	 */
	async get(request: MediaRequestParameters, loadingState?: LoadingState): Promise<string> {
		// Initialize cache if not already done
		if (!this.cache) {
			await this.init();
		}

		let mxcUri = isPlainMediaSource(request.source) ? request.source.url : request.source.file.url;

		// Check if image is already cached
		const cachedImage = await this.getCachedMedia(mxcUri);
		if (cachedImage) {
			console.log(`Avatar loaded from cache: ${mxcUri}`);
			return cachedImage;
		}

		// Check if image is currently being fetched
		const pendingRequest = this.pendingRequests.get(mxcUri);
		if (pendingRequest) {
			console.log(`Avatar request already in progress, waiting: ${mxcUri}`);
			return pendingRequest;
		}

		// Create new fetch request
		const fetchPromise = this.fetchAndCache(mxcUri, request, loadingState);

		// Register the pending request
		this.pendingRequests.set(mxcUri, fetchPromise);

		try {
			const result = await fetchPromise;
			return result;
		} finally {
			// Always remove the pending request when done (success or failure)
			this.pendingRequests.delete(mxcUri);
		}
	}

	/**
	 * Get cached image as blob URL (private method)
	 */
	private async getCachedMedia(mxcUri: string): Promise<string | null> {
		if (!this.cache) return null;

		try {
			const cacheKey = this.getCacheKey(mxcUri);
			const response = await this.cache.match(cacheKey);

			if (!response) return null;

			// Check if still valid
			const cachedTime = response.headers.get('x-cached-time');
			if (cachedTime) {
				const age = Date.now() - parseInt(cachedTime);
				if (age > CACHE_CONFIG.maxAge) {
					await this.cache.delete(cacheKey);
					return null;
				}
			}

			const blob = await response.blob();
			return URL.createObjectURL(blob);
		} catch (error) {
			console.error('Error retrieving from cache:', error);
			return null;
		}
	}

	/**
	 * Fetch image from backend and cache it (private method)
	 */
	private async fetchAndCache(
		mxcUri: string,
		request: MediaRequestParameters,
		loadingState?: LoadingState
	): Promise<string> {
		const chunks: Uint8Array[] = [];

		return new Promise((resolve, reject) => {
			const onEvent = new Channel<MediaStreamEvent>();

			let bytesReceived = $state(0);
			let localProgress = $derived(bytesReceived / (loadingState?.totalSize ?? 1));

			onEvent.onmessage = async (message) => {
				try {
					if (message.event === 'started') {
						return;
					}

					if (message.event === 'chunk') {
						chunks.push(new Uint8Array(message.data.data));
						bytesReceived = message.data.bytesReceived;
						if (loadingState) {
							loadingState.progress = localProgress;
						}
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

						// Store in cache
						await this.cacheMedia(mxcUri, combined);

						// Create blob URL for display
						const blob = new Blob([combined]);
						const imageSrc = URL.createObjectURL(blob);
						console.log(`Image fetch completed and cached: ${message.data.totalBytes} bytes`);

						resolve(imageSrc);
						if (loadingState) {
							loadingState.isLoaded = true;
						}
						return;
					}

					if (message.event === 'error') {
						console.error('Image fetch error:', message.data.message);
						reject(new Error(`Image fetch error: ${message.data.message}`));
						return;
					}
				} catch (error) {
					console.error('Error processing message:', error);
					reject(error);
				}
			};

			// Start the fetch
			invoke('plugin:matrix-svelte|fetch_media', {
				mediaRequest: request,
				onEvent
			}).catch(reject);
		});
	}

	/**
	 * Store image data in cache (private method)
	 */
	private async cacheMedia(mxcUri: string, mediaData: Uint8Array): Promise<void> {
		if (!this.cache) return;

		try {
			const cacheKey = this.getCacheKey(mxcUri);
			const fileType = await fileTypeFromBuffer(mediaData);
			const blob = new Blob([mediaData], { type: fileType?.mime });

			// Create response with metadata
			const response = new Response(blob, {
				headers: {
					'Content-Type': fileType?.mime ?? 'image/jpeg',
					'x-cached-time': Date.now().toString(),
					'x-mxc-uri': mxcUri
				}
			});

			await this.cache.put(cacheKey, response);

			// Cleanup if we exceed max entries
			await this.enforceCacheSize();
		} catch (error) {
			console.error('Error storing in cache:', error);
		}
	}

	/**
	 * Generate a cache key from MXC URI (private method)
	 */
	private getCacheKey(mxcUri: string): string {
		// Cache API requires HTTP/HTTPS URLs, so we create a fake URL
		// Using a consistent domain to avoid issues
		const encoded = encodeURIComponent(mxcUri);
		return `https://cache.local/media/${encoded}`;
	}

	/**
	 * Remove expired entries from cache (private method)
	 */
	private async cleanupExpiredEntries(): Promise<void> {
		if (!this.cache) return;

		try {
			const keys = await this.cache.keys();
			const now = Date.now();

			for (const request of keys) {
				const response = await this.cache.match(request);
				if (response) {
					const cachedTime = response.headers.get('x-cached-time');
					if (cachedTime) {
						const age = now - parseInt(cachedTime);
						if (age > CACHE_CONFIG.maxAge) {
							await this.cache.delete(request);
						}
					}
				}
			}
		} catch (error) {
			console.error('Error during cache cleanup:', error);
		}
	}

	/**
	 * Enforce maximum cache size by removing oldest entries (private method)
	 */
	private async enforceCacheSize(): Promise<void> {
		if (!this.cache) return;

		try {
			const keys = await this.cache.keys();

			if (keys.length <= CACHE_CONFIG.maxEntries) return;

			// Get all entries with their timestamps
			const entries: { request: Request; timestamp: number }[] = [];

			for (const request of keys) {
				const response = await this.cache.match(request);
				if (response) {
					const cachedTime = response.headers.get('x-cached-time');
					const timestamp = cachedTime ? parseInt(cachedTime) : 0;
					entries.push({ request, timestamp });
				}
			}

			// Sort by timestamp (oldest first)
			entries.sort((a, b) => a.timestamp - b.timestamp);

			// Remove oldest entries
			const entriesToRemove = entries.length - CACHE_CONFIG.maxEntries;
			for (let i = 0; i < entriesToRemove; i++) {
				await this.cache.delete(entries[i].request);
			}
		} catch (error) {
			console.error('Error enforcing cache size:', error);
		}
	}

	/**
	 * Clear all cached images and pending requests
	 */
	async clear(): Promise<void> {
		try {
			await caches.delete(FULL_CACHE_NAME);
			this.cache = await caches.open(FULL_CACHE_NAME);
			this.pendingRequests.clear();
		} catch (error) {
			console.error('Error clearing cache:', error);
		}
	}

	/**
	 * Get cache statistics
	 */
	async getStats(): Promise<{ count: number; size: number; pending: number }> {
		if (!this.cache) return { count: 0, size: 0, pending: 0 };

		try {
			const keys = await this.cache.keys();
			let totalSize = 0;

			for (const request of keys) {
				const response = await this.cache.match(request);
				if (response) {
					const blob = await response.blob();
					totalSize += blob.size;
				}
			}

			return {
				count: keys.length,
				size: totalSize,
				pending: this.pendingRequests.size
			};
		} catch (error) {
			console.error('Error getting cache stats:', error);
			return { count: 0, size: 0, pending: 0 };
		}
	}
}

// Create singleton instance
export const mediaCache = new MediaCache();

// Initialize cache when module loads
mediaCache.init();
