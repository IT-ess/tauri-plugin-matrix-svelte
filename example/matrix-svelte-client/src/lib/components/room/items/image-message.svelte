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
</script>

<div class={cn('bg-card relative mt-1 overflow-hidden', isSticker ? '' : 'rounded-lg border')}>
	<!-- {#await loadImage()}
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
					{Math.round(loadingState.progress * 100)}%
				</div>
			</div>
		</div>
	{:then imageSrc} -->
	<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
	<img
		src={imageSrc}
		{alt}
		class="w-full cursor-pointer object-cover"
		role="button"
		tabindex="0"
		onclick={() => toggleFullscreen(imageSrc)}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				toggleFullscreen(imageSrc);
			}
		}}
	/>
</div>
