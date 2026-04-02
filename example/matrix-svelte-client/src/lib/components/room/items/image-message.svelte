<script lang="ts">
	import { decode } from 'blurhash';
	import { cn, getCustomMxcUriFromOriginal, getUrlFromEncryptedSource } from '$lib/utils.svelte';
	import {
		imageMessageSourceIsPlain,
		type ImageMessageEventContent,
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
			}
		) => void;
	};

	let { itemContent, isSticker = false, handleOpenMediaViewMode }: Props = $props();

	let blurhash = $derived(
		itemContent.info?.['xyz.amorgan.blurhash'] ?? 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'
	); // Placeholder blurhash
	let alt = $derived(itemContent.body);

	let imageSrc = $derived(
		imageMessageSourceIsPlain(itemContent)
			? (getCustomMxcUriFromOriginal(itemContent.url) as string)
			: getUrlFromEncryptedSource(itemContent.file)
	);

	const toggleFullscreen = (imageSrc: string) => {
		handleOpenMediaViewMode('image', imageSrc, {
			body: itemContent.body,
			size: itemContent.info?.size ?? 1
		});
	};

	let isLoaded = $state(false);

	let imageWidthOrDefault = $derived(itemContent.info?.w ?? 200);
	let imageHeightOrDefault = $derived(itemContent.info?.h ?? 200);
</script>

<div class={cn('bg-card relative mt-1 overflow-hidden', isSticker ? '' : 'rounded-lg border')}>
	{#if !isLoaded}
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
	<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
	<img
		src={imageSrc}
		{alt}
		class="w-full cursor-pointer object-cover"
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
