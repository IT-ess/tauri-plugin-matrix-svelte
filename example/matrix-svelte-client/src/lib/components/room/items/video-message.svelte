<script lang="ts">
	import { Play } from '@lucide/svelte';
	import {
		videoMessageInfoThumbnailSourceIsPlain,
		videoMessageSourceIsPlain,
		type VideoMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { getCustomMxcUriFromOriginal, getUrlFromEncryptedSource } from '$lib/utils.svelte';

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

	let thumnailSrc = $derived(
		itemContent.info
			? videoMessageInfoThumbnailSourceIsPlain(itemContent.info)
				? getCustomMxcUriFromOriginal(itemContent.info.thumbnail_url)
				: getUrlFromEncryptedSource(itemContent.info.thumbnail_file)
			: null
	);
	let videoSrc = $derived(
		videoMessageSourceIsPlain(itemContent)
			? (getCustomMxcUriFromOriginal(itemContent.url) as string)
			: getUrlFromEncryptedSource(itemContent.file)
	);

	let hasClickedToggleFullscreen = $state(false);
	const toggleFullscreen = () => {
		if (hasClickedToggleFullscreen) return;
		hasClickedToggleFullscreen = true;
		handleOpenMediaViewMode('video', videoSrc, {
			body: itemContent.body,
			size: itemContent.info?.size ?? 1
		});
		hasClickedToggleFullscreen = false;
	};
</script>

<div class="bg-card relative mt-1 overflow-hidden rounded-lg border">
	<!-- Blurhash Canvas as optimistic UI -->
	<!-- <div class="relative">
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
	</div> -->
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
			<Play class="text-primary size-12 p-2" />
		</div>
		<img src={thumnailSrc} {alt} class="w-full cursor-pointer object-cover" />
	</div>
</div>
