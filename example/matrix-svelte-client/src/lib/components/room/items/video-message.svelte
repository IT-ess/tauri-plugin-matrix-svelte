<script lang="ts">
	import { decode } from 'blurhash';
	import { Button } from '$lib/components/ui/button';
	import {
		fetchMedia,
		MediaLoadingState,
		type VideoMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: VideoMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let blurhash = itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'; // Placeholder blurhash
	// let alt = itemContent.filename ?? itemContent.body;

	// State variables
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let videoSrc = $state<string>('');
	let loadingState = new MediaLoadingState(itemContent.info?.size ?? 1);

	// Load image function
	const loadVideo = async () => {
		if (loadingState.isLoaded || isLoading) return;

		isLoading = true;
		error = null;

		try {
			await fetchMedia(
				{
					format: 'File',
					source: { file: itemContent.file }
				},
				loadingState
			);
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
	{#if !loadingState.isLoaded}
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
						{Math.round(loadingState.progress * 100)}%
					</div>
				</div>
			{/if}

			{#if error}
				<div class="bg-destructive/80 absolute inset-0 flex items-center justify-center">
					<div class="text-center text-white">
						<p class="mb-2 text-sm">Failed to load</p>
						<Button variant="secondary" size="sm" onclick={() => loadVideo()}>Retry</Button>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<!-- Loaded Video -->
		<!-- svelte-ignore a11y_media_has_caption -->
		<video
			src={videoSrc}
			class="w-full cursor-pointer object-cover"
			role="button"
			tabindex="0"
			controls
		></video>
	{/if}
</div>
