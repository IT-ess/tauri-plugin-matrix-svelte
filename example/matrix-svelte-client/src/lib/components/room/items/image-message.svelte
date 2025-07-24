<script lang="ts">
	import { decode } from 'blurhash';
	import { scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import { XIcon } from '@lucide/svelte';
	import {
		fetchMedia,
		type events,
		type ImageMessageEventContent,
		type MediaRequestParameters,
		type StickerEventContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { Channel } from '@tauri-apps/api/core';
	import { onClickOutside } from 'runed';
	import { cn } from '$lib/utils';

	type Props = {
		itemContent: ImageMessageEventContent | StickerEventContent;
		isSticker?: boolean;
	};

	let { itemContent, isSticker = false }: Props = $props();

	let blurhash = itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'; // Placeholder blurhash
	let alt = itemContent.body;

	// State variables
	let isLoaded = $state(false);
	let isFullscreen = $state(false);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let imageSrc = $state<string>('');
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	let progress = $derived(bytesReceived / totalSize);

	let fullscreenContainer = $state<HTMLDivElement>()!;

	onClickOutside(
		() => fullscreenContainer,
		() => (isFullscreen = false)
	);

	// Load image function
	const loadImage = async () => {
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
					imageSrc = URL.createObjectURL(blob);
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

			let mediaRequest: MediaRequestParameters =
				itemContent.msgtype === 'm.image' // Images are encrypted while stickers aren't
					? {
							format: 'File',
							source: { file: itemContent.file }
						}
					: {
							format: 'File',
							source: { url: itemContent.url }
						};

			await fetchMedia(mediaRequest, onEvent);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error occurred';
			isLoading = false;
			console.error('Invoke error:', err);
		}
	};

	// Auto-start loading when component mounts
	$effect(() => {
		loadImage();
	});

	const toggleFullscreen = () => {
		if (isLoaded) {
			isFullscreen = !isFullscreen;
		}
	};
</script>

<div class={cn('bg-card relative mt-1 overflow-hidden', isSticker ? '' : 'rounded-lg border')}>
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
						<Button variant="secondary" size="sm" onclick={() => loadImage()}>Retry</Button>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<!-- Loaded Image -->
		<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
		<img
			src={imageSrc}
			{alt}
			class="w-full cursor-pointer object-cover"
			role="button"
			tabindex="0"
			onclick={toggleFullscreen}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					toggleFullscreen();
				}
			}}
		/>
	{/if}
</div>

<!-- Fullscreen Modal -->
{#if isFullscreen && imageSrc}
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
