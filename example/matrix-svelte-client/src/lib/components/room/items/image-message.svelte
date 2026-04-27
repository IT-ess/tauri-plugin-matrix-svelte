<script lang="ts">
	import { decode } from 'blurhash';
	import { cn, getCustomMxcUriFromOriginal } from '$lib/utils.svelte';
	import {
		imageMessageSourceIsPlain,
		type ImageMessageEventContent,
		type MediaRequestParameters,
		type StickerEventContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: ImageMessageEventContent | StickerEventContent;
		isSticker: boolean;
		handleOpenMediaViewMode: (
			type: 'image' | 'video',
			src: string,
			info: {
				filename?: string;
				body?: string;
				size: number;
			},
			mediaSource: MediaRequestParameters['source']
		) => void;
	};

	let { itemContent, isSticker = false, handleOpenMediaViewMode }: Props = $props();

	let blurhash = $derived(
		itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'
	); // Placeholder blurhash
	let alt = $derived(itemContent.body);

	let imageSrc = $derived(
		(imageMessageSourceIsPlain(itemContent)
			? getCustomMxcUriFromOriginal(itemContent.url, {
					mime:
						itemContent.info?.thumbnail_info?.mimetype ?? itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.thumbnail_info?.size ?? itemContent.info?.size ?? undefined
				})
			: getCustomMxcUriFromOriginal(itemContent.file, {
					mime:
						itemContent.info?.thumbnail_info?.mimetype ?? itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.thumbnail_info?.size ?? itemContent.info?.size ?? undefined
				})) as string
	);

	const toggleFullscreen = (imageSrc: string) => {
		handleOpenMediaViewMode(
			'image',
			imageSrc,
			{
				body: itemContent.body,
				size: itemContent.info?.size ?? 1
			},
			imageMessageSourceIsPlain(itemContent) ? { url: itemContent.url } : { file: itemContent.file }
		);
	};

	let isLoaded = $state(false);

	let imageWidthOrDefault = $derived(itemContent.info?.w ?? 200);
	let imageHeightOrDefault = $derived(itemContent.info?.h ?? 200);
</script>

<div
	class={cn('bg-card relative mt-1 overflow-hidden', isSticker ? '' : 'rounded-lg border')}
	style="content-visibility: auto;"
>
	{#if !isLoaded}
		<canvas
			{@attach (canvas) => {
				const pixels = decode(blurhash, 32, 32);
				const context = canvas.getContext('2d');
				const imageData = context?.createImageData(32, 32);
				if (imageData) {
					imageData.data.set(pixels);
					context?.putImageData(imageData, 0, 0);
				}
			}}
			width={32}
			height={32}
			class="h-full w-full object-cover"
			style="image-rendering: auto;"
		></canvas>
	{/if}
	<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
	<img
		src={imageSrc}
		loading="lazy"
		decoding="async"
		width={imageWidthOrDefault}
		height={imageHeightOrDefault}
		{alt}
		class="cursor-pointer object-cover"
		role="button"
		tabindex="0"
		onload={() => (isLoaded = true)}
		onclick={() => toggleFullscreen(imageSrc)}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				toggleFullscreen(imageSrc);
			}
		}}
	/>
</div>
