<script lang="ts">
	import { decode } from 'blurhash';
	import { Button } from '$lib/components/ui/button';
	import type { events, VideoMessageEventContent } from 'tauri-plugin-matrix-svelte-api';
	import { Channel, invoke } from '@tauri-apps/api/core';

	type Props = {
		itemContent: VideoMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let blurhash = itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'; // Placeholder blurhash
	// let alt = itemContent.filename ?? itemContent.body;

	// State variables
	let isLoaded = $state(false);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let videoSrc = $state<string>('');
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	let progress = $derived(bytesReceived / totalSize);

	// Load image function
	const loadVideo = async () => {
		if (isLoaded || isLoading) return;

		isLoading = true;
		error = null;
		bytesReceived = 0;

		const chunks: Uint8Array[] = [];
		try {
			const onEvent = new Channel<events.MediaStreamEvent>();

			onEvent.onmessage = (message) => {
				if (message.event === 'started') {
					console.log(`Starting image fetch, total size: ${totalSize} bytes`);
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

					// Create blob URL for display
					const blob = new Blob([combined], { type: itemContent.info?.mimetype ?? 'image/jpeg' });
					videoSrc = URL.createObjectURL(blob);
					isLoaded = true;
					isLoading = false;
					console.log(`Image fetch completed: ${message.data.totalBytes} bytes`);
					return;
				}

				if (message.event === 'error') {
					error = message.data.message;
					isLoading = false;
					console.error('Image fetch error:', message.data.message);
					return;
				}
			};

			await invoke('plugin:matrix-svelte|fetch_media', {
				mediaRequest: {
					format: 'File',
					source: { file: itemContent.file }
				},
				onEvent
			});
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error occurred';
			isLoading = false;
			console.error('Invoke error:', err);
		}
	};

	// Auto-start loading when component mounts
	$effect(() => {
		loadVideo();
	});
</script>

<div class="bg-card relative mt-1 overflow-hidden rounded-lg border">
	{#if !isLoaded}
		<!-- Blurhash Canvas as optimistic UI -->
		<div class="relative">
			<canvas
				{@attach (canvas) => {
					console.log('Attaching canvas. Blurhash is', blurhash);
					const pixels = decode(blurhash, 200, 200);
					const context = canvas.getContext('2d');
					const imageData = context?.createImageData(200, 200);
					if (imageData) {
						imageData.data.set(pixels);
						context?.putImageData(imageData, 0, 0);
					}
				}}
				width={200}
				height={200}
				class="aspect-video w-full object-cover"
			></canvas>

			{#if isLoading}
				<div class="absolute inset-0 flex items-center justify-center bg-black/20">
					<div class="rounded-full bg-white/90 px-3 py-1 text-xs">
						{Math.round(progress * 100)}%
					</div>
				</div>
			{/if}

			{#if error}
				<div class="bg-destructive/80 absolute inset-0 flex items-center justify-center">
					<div class="text-center text-white">
						<p class="mb-2 text-sm">Failed to load</p>
						<Button variant="secondary" size="sm" onclick={loadVideo}>Retry</Button>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<!-- Loaded Video -->
		<!-- svelte-ignore a11y_media_has_caption -->
		<video
			src={videoSrc}
			class="aspect-video w-full cursor-pointer object-cover"
			role="button"
			tabindex="0"
			controls
		></video>
	{/if}
</div>
