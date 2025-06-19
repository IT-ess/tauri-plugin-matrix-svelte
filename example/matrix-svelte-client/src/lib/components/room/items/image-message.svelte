<script lang="ts">
	import { decode } from 'blurhash';
	import { fade, scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import { ImageIcon, Loader, XIcon } from '@lucide/svelte';
	import type { events, ImageMessageEventContent } from 'tauri-plugin-matrix-svelte-api';
	import { Channel, invoke } from '@tauri-apps/api/core';
	import { onClickOutside } from 'runed';

	type Props = {
		itemContent: ImageMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let blurhash = itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'; // use this blurhash as a placeholder
	let alt = itemContent.body;

	// State variables
	let isLoaded = $state(false);
	let isFullscreen = $state(false);
	let imageSrc = $state<string>('');
	let isLoading = $state(false);
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let error = $state<string | null>(null);
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let progress = $derived(bytesReceived / totalSize);

	let fullscreenContainer = $state<HTMLDivElement>()!;

	onClickOutside(
		() => fullscreenContainer,
		() => (isFullscreen = false)
	);

	// Load image when button is clicked
	const loadImage = async () => {
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
					imageSrc = URL.createObjectURL(blob);

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
					format: 'File', // We do not handle thumbnails yet
					source: { file: itemContent.file }
				},
				onEvent
			});
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error occurred';
			isLoading = false;
			console.error('Invoke error:', err);
		}

		isLoaded = true;
	};

	// Toggle fullscreen view
	const toggleFullscreen = () => {
		if (isLoaded) {
			isFullscreen = !isFullscreen;
		}
	};
</script>

<div class="bg-card relative overflow-hidden rounded-lg border">
	<!-- Blurhash Canvas -->
	<canvas
		{@attach (canvas) => {
			// TODO: optimise this because this is ran multiple times since the blurhash store is updated. See attachments docs
			console.log('Attaching canvas. Blurhash is', blurhash);
			const pixels = decode(blurhash, itemContent.info?.w, itemContent.info?.h);
			const context = canvas.getContext('2d');
			const imageData = context?.createImageData(itemContent.info?.w, itemContent.info?.h);
			if (imageData) {
				imageData.data.set(pixels);
				context?.putImageData(imageData, 0, 0);
			}
		}}
		width={itemContent.info?.w ?? 400 / 2}
		height={itemContent.info?.h ?? 400 / 2}
		class:hidden={isLoaded}
	></canvas>

	<!-- Main Image -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<img
		src={imageSrc}
		{alt}
		class="aspect-video w-full cursor-pointer object-cover"
		onclick={toggleFullscreen}
		class:hidden={!isLoaded}
	/>

	<!-- Load Button -->
	{#if !isLoaded}
		<div class="absolute inset-0 flex items-center justify-center" transition:fade>
			<Button onclick={() => loadImage()}>
				{#if isLoading}
					<Loader class="mr-2 h-4 w-4" />
				{:else}
					<ImageIcon class="mr-2 h-4 w-4" />
				{/if}
				Load Image
			</Button>
		</div>
	{/if}
</div>

<!-- Fullscreen Modal -->
<!-- TODO: add onOutsideClick -->
{#if isFullscreen}
	<div
		class="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm"
		transition:scale
	>
		<div bind:this={fullscreenContainer} class="relative max-h-[90vh] max-w-[90vw]">
			<Button
				variant="ghost"
				size="icon"
				class="absolute -top-4 -right-4 rounded-full"
				onclick={() => (isFullscreen = false)}
			>
				<XIcon class="h-4 w-4" />
			</Button>
			<img src={imageSrc} {alt} class="max-h-[90vh] max-w-[90vw] rounded-lg object-contain" />
		</div>
	</div>
{/if}
