<script lang="ts">
	import { decode } from 'blurhash';
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { Play } from '@lucide/svelte';
	import {
		fetchMedia,
		MediaLoadingState,
		videoMessageInfoThumbnailSourceIsPlain,
		videoMessageSourceIsPlain,
		type VideoMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';

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
	let alt = $derived(itemContent.body);

	// State variables
	// svelte-ignore state_referenced_locally
	let loadingState = $state(new MediaLoadingState(itemContent.info?.size ?? 1));
	// svelte-ignore state_referenced_locally
	let thumbnailLoadingState = $state(
		new MediaLoadingState(itemContent.info?.thumbnail_info?.size ?? 1)
	);

	const loadVideoThumbnail = () => {
		if (itemContent.info) {
			if (videoMessageInfoThumbnailSourceIsPlain(itemContent.info)) {
				return fetchMedia(
					{
						format: 'File',
						source: { url: itemContent.info.thumbnail_url }
					},
					thumbnailLoadingState
				);
			} else {
				return fetchMedia(
					{
						format: 'File',
						source: { file: itemContent.info.thumbnail_file }
					},
					thumbnailLoadingState
				);
			}
		} else {
			console.error('No thumbnail for this file');
		}
	};

	const loadVideoSource = () => {
		if (videoMessageSourceIsPlain(itemContent)) {
			return fetchMedia(
				{
					format: 'File',
					source: { url: itemContent.url }
				},
				loadingState
			);
		} else {
			return fetchMedia(
				{
					format: 'File',
					source: { file: itemContent.file }
				},
				loadingState
			);
		}
	};

	let hasClickedToggleFullscreen = $state(false);
	const toggleFullscreen = async () => {
		if (hasClickedToggleFullscreen) return;
		hasClickedToggleFullscreen = true;
		handleOpenMediaViewMode('video', await loadVideoSource(), {
			body: itemContent.body,
			size: itemContent.info?.size ?? 1
		});
		hasClickedToggleFullscreen = false;
	};
</script>

<div class="bg-card relative mt-1 overflow-hidden rounded-lg border">
	{#await loadVideoThumbnail()}
		<!-- Blurhash Canvas as optimistic UI -->
		<div class="relative">
			<canvas
				{@attach (canvas) => {
					if (import.meta.env.DEV) {
						console.log('Attaching canvas. Blurhash is', blurhash);
					}
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

			<div class="absolute inset-0 flex items-center justify-center bg-black/20">
				<div class="rounded-full bg-white/90 px-3 py-1 text-xs">
					{Math.round(thumbnailLoadingState.progress * 100)}%
				</div>
			</div>
		</div>
	{:then videoSrc}
		<div
			class="relative flex items-center justify-center"
			onclick={toggleFullscreen}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					toggleFullscreen();
				}
			}}
			role="button"
			tabindex="0"
		>
			<div class="absolute rounded-full bg-white/70">
				{#if hasClickedToggleFullscreen}
					<p class="text-primary text-xl">{(loadingState.progress * 100).toFixed(0)}%</p>
				{:else}
					<Play class="text-primary size-12 p-2" />
				{/if}
			</div>
			<img src={videoSrc} {alt} class="w-full cursor-pointer object-cover" />
		</div>
	{:catch}
		<div class="bg-destructive/80 absolute inset-0 flex items-center justify-center">
			<div class="text-center text-white">
				<p class="mb-2 text-sm">{m.failed_to_load()}</p>
				<Button variant="secondary" size="sm" onclick={() => loadVideoThumbnail()}
					>{m.button_retry()}</Button
				>
			</div>
		</div>
	{/await}
</div>
