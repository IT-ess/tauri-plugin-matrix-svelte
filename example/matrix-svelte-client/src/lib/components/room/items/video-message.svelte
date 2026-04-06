<script lang="ts">
	import { decode } from 'blurhash';
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { Play } from '@lucide/svelte';
	import {
		videoMessageInfoThumbnailSourceIsPlain,
		videoMessageSourceIsPlain,
		type MediaStreamEvent,
		type VideoMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { Channel, invoke } from '@tauri-apps/api/core';
	import { getCustomMxcUriFromOriginal } from '$lib/utils.svelte';

	type Props = {
		itemContent: VideoMessageEventContent;
		handleOpenMediaViewMode: (
			type: 'image' | 'video',
			src: string,
			info: {
				filename?: string;
				body?: string;
				size: number;
			}
		) => void;
	};

	let { itemContent, handleOpenMediaViewMode }: Props = $props();

	let blurhash = $derived(
		itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'
	);
	let thumnailSrc = $derived(
		itemContent.info
			? videoMessageInfoThumbnailSourceIsPlain(itemContent.info)
				? getCustomMxcUriFromOriginal(itemContent.info.thumbnail_url, {
						mime: itemContent.info?.thumbnail_info?.mimetype ?? undefined,
						size: itemContent.info?.thumbnail_info?.size ?? undefined
					})
				: getCustomMxcUriFromOriginal(itemContent.info.thumbnail_file, {
						mime: itemContent.info?.thumbnail_info?.mimetype ?? undefined,
						size: itemContent.info?.thumbnail_info?.size ?? undefined
					})
			: null
	);
	let imageWidthOrDefault = $derived(itemContent.info?.thumbnail_info?.w ?? 200);
	let imageHeightOrDefault = $derived(itemContent.info?.thumbnail_info?.h ?? 200);
	let isThumbLoaded = $state(false);

	let alt = $derived(itemContent.body);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let videoSrc = $state<string | undefined>();
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	let progress = $derived(bytesReceived / totalSize);

	const loadVideoSource = async () => {
		if (isLoading) return;

		isLoading = true;
		error = null;
		bytesReceived = 0;

		const chunks: Uint8Array[] = [];
		try {
			const onEvent = new Channel<MediaStreamEvent>();

			onEvent.onmessage = (message) => {
				if (message.event === 'started') {
					console.log(`Starting file fetch, total size: ${totalSize} bytes`);
					return;
				}

				if (message.event === 'chunk') {
					chunks.push(new Uint8Array(message.data.data));
					bytesReceived = message.data.bytesReceived;
					console.log(
						`Received chunk: ${message.data.chunkSize} bytes, total: ${bytesReceived}/${totalSize}`
					);
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

					const blob = new Blob([combined]);
					videoSrc = URL.createObjectURL(blob);
					isLoading = false;
					console.log(`File fetch completed: ${message.data.totalBytes} bytes`);
					handleOpenMediaViewMode('video', videoSrc, {
						body: itemContent.body,
						size: itemContent.info?.size ?? 1
					});
					return;
				}

				if (message.event === 'error') {
					error = message.data.message;
					isLoading = false;
					console.error('File fetch error:', message.data.message);
					return;
				}
			};
			if (videoMessageSourceIsPlain(itemContent)) {
				await invoke('plugin:matrix-svelte|fetch_media', {
					mediaRequest: {
						format: 'File',
						source: { url: itemContent.url }
					},
					onEvent
				});
			} else {
				await invoke('plugin:matrix-svelte|fetch_media', {
					mediaRequest: {
						format: 'File',
						source: { file: itemContent.file }
					},
					onEvent
				});
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error occurred';
			isLoading = false;
			console.error('Invoke error:', err);
		}
	};

	const startVideoLoadOrOpen = async () => {
		if (videoSrc) {
			handleOpenMediaViewMode('video', videoSrc, {
				body: itemContent.body,
				size: itemContent.info?.size ?? 1
			});
		} else {
			try {
				await loadVideoSource();
			} catch (err) {
				console.error(err);
			}
		}
	};
</script>

<div
	class="bg-card relative mt-1 flex items-center justify-center overflow-hidden rounded-lg border"
>
	{#if !isThumbLoaded}
		<canvas
			{@attach (canvas) => {
				const pixels = decode(blurhash, imageWidthOrDefault, imageHeightOrDefault);
				const context = canvas.getContext('2d');
				const imageData = context?.createImageData(imageWidthOrDefault, imageHeightOrDefault);
				if (imageData) {
					imageData.data.set(pixels);
					context?.putImageData(imageData, 0, 0);
				}
			}}
			width={imageWidthOrDefault}
			height={imageHeightOrDefault}
			class="w-full object-cover"
		></canvas>
	{/if}

	<div
		class="relative flex items-center justify-center"
		onclick={startVideoLoadOrOpen}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				startVideoLoadOrOpen();
			}
		}}
		role="button"
		tabindex="0"
	>
		<img
			loading="lazy"
			onload={() => (isThumbLoaded = true)}
			src={thumnailSrc}
			{alt}
			class="w-full cursor-pointer object-cover"
		/>
	</div>

	<button onclick={startVideoLoadOrOpen} class="absolute rounded-full bg-white/70">
		{#if isLoading}
			{progress.toFixed(0)}%
		{:else}
			<Play class="text-primary size-12 p-2" />
		{/if}
	</button>

	{#if error}
		<div class="bg-destructive/80 inset-0 flex items-center justify-center">
			<div class="text-center text-white">
				<p class="mb-2 text-sm">{m.failed_to_load()}</p>
				<Button variant="secondary" size="sm" onclick={() => loadVideoSource()}
					>{m.button_retry()}</Button
				>
			</div>
		</div>
	{/if}
</div>
