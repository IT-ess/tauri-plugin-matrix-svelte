<script lang="ts">
	import { Play } from '@lucide/svelte';
	import {
		videoMessageInfoThumbnailSourceIsPlain,
		videoMessageSourceIsPlain,
		type VideoMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { getCustomMxcUriFromOriginal } from '$lib/utils.svelte';
	import { decode } from 'blurhash';

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
				? getCustomMxcUriFromOriginal(itemContent.info.thumbnail_url, {
						mime: itemContent.info?.thumbnail_info?.mimetype ?? undefined,
						size: itemContent.info?.thumbnail_info?.size ?? undefined,
						th: itemContent.info.thumbnail_info?.h ?? undefined,
						tw: itemContent.info.thumbnail_info?.w ?? undefined,
						tm: 'crop'
					})
				: getCustomMxcUriFromOriginal(itemContent.info.thumbnail_file, {
						mime: itemContent.info?.mimetype ?? undefined,
						size: itemContent.info?.size ?? undefined
					})
			: null
	);
	let videoSrc = $derived(
		(videoMessageSourceIsPlain(itemContent)
			? getCustomMxcUriFromOriginal(itemContent.url, {
					mime: itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.size ?? undefined
				})
			: getCustomMxcUriFromOriginal(itemContent.file, {
					mime: itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.size ?? undefined
				})) as string
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

	let isThumbLoaded = $state(false);
	let imageWidthOrDefault = $derived(itemContent.info?.thumbnail_info?.w ?? 200);
	let imageHeightOrDefault = $derived(itemContent.info?.thumbnail_info?.h ?? 200);
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
		onclick={toggleFullscreen}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				toggleFullscreen();
			}
		}}
		onload={() => (isThumbLoaded = true)}
		role="button"
		tabindex="0"
	>
		<img loading="lazy" src={thumnailSrc} {alt} class="w-full cursor-pointer object-cover" />
	</div>

	<div class="absolute rounded-full bg-white/70">
		<Play class="text-primary size-12 p-2" />
	</div>
</div>
