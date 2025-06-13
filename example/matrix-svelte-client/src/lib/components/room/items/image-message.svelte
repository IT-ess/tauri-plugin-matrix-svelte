<script lang="ts">
	import { decode } from 'blurhash';
	import { fade, scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import { ImageIcon, XIcon } from '@lucide/svelte';

	type Props = {
		src: string;
		blurhash: string;
		alt: string;
	};

	// let json = {
	// 	body: 'a capitalist dream.jpg',
	// 	file: {
	// 		hashes: { sha256: 'nOAhf17EI29KSsw2VCPmx8r13QRDIZ77KpKeN1lWY4I' },
	// 		iv: '/wIzIcHJ/m8AAAAAAAAAAA',
	// 		key: {
	// 			alg: 'A256CTR',
	// 			ext: true,
	// 			k: '3oQUwh6yknodlreZSBf7NZKeZdzwTJnLyaZlOZRT5ho',
	// 			key_ops: ['encrypt', 'decrypt'],
	// 			kty: 'oct'
	// 		},
	// 		url: 'mxc://matrix.lucide.space/qEWPqaI06FzhPNGvhunR6Qsv1s1cBRqJ',
	// 		v: 'v2'
	// 	},
	// 	info: {
	// 		h: 898,
	// 		mimetype: 'image/jpeg',
	// 		size: 245733,
	// 		w: 650,
	// 		'xyz.amorgan.blurhash': 'LQHx$:t8*JEj*0WqtlNd9@WUIVsT'
	// 	},
	// 	msgtype: 'm.image'
	// };

	let { src, blurhash, alt }: Props = $props();

	// State variables
	let isLoaded = $state(false);
	let isFullscreen = $state(false);

	// Handle blurhash decoding
	// const decodeBlurhash = () => {
	// 	const pixels = decode(blurhash, 32, 32);
	// 	const ctx = canvas?.getContext('2d');
	// 	const imageData = ctx?.createImageData(32, 32);
	// 	if (imageData) {
	// 		imageData.data.set(pixels);
	// 		ctx?.putImageData(imageData, 0, 0);
	// 	}
	// };

	// const blurhashCanvas: Attachment = (element) => {
	// 	const pixels = decode(blurhash, 32, 32);
	// 	const ctx = element.getContext('2d');

	// 	return () => {
	// 		console.log('cleaning up');
	// 	};
	// };

	// Load image when button is clicked
	// const loadImage = () => {
	// 	imageElement.src = src;
	// 	imageElement.onload = () => {
	// 		isLoaded = true;
	// 	};
	// };

	// Toggle fullscreen view
	const toggleFullscreen = () => {
		if (isLoaded) {
			isFullscreen = !isFullscreen;
		}
	};

	// Initialize blurhash
	// $effect(() => {
	// 	if (canvas) {
	// 		decodeBlurhash();
	// 	}
	// });
</script>

<div class="bg-card relative overflow-hidden rounded-lg border">
	<!-- Blurhash Canvas -->
	<canvas
		{@attach (canvas) => {
			console.log('Attaching canvas. Blurhash is', blurhash);
			const pixels = decode(blurhash, 200, 200);
			const context = canvas.getContext('2d');
			const imageData = context.createImageData(200, 200);
			if (imageData) {
				imageData.data.set(pixels);
				context.putImageData(imageData, 0, 0);
			}
		}}
		width="200"
		height="200"
		class:hidden={isLoaded}
	></canvas>

	<!-- Main Image -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<img
		{alt}
		class="aspect-video w-full cursor-pointer object-cover"
		onclick={toggleFullscreen}
		class:hidden={!isLoaded}
	/>

	<!-- Load Button -->
	{#if !isLoaded}
		<div class="absolute inset-0 flex items-center justify-center" transition:fade>
			<Button onclick={() => null}>
				<ImageIcon class="mr-2 h-4 w-4" />
				Load Image
			</Button>
		</div>
	{/if}
</div>

<!-- Fullscreen Modal -->
{#if isFullscreen}
	<div
		class="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm"
		transition:scale
	>
		<div class="relative max-h-[90vh] max-w-[90vw]">
			<Button
				variant="ghost"
				size="icon"
				class="absolute -top-4 -right-4 rounded-full"
				onclick={() => (isFullscreen = false)}
			>
				<XIcon class="h-4 w-4" />
			</Button>
			<img {src} {alt} class="max-h-[90vh] max-w-[90vw] rounded-lg object-contain" />
		</div>
	</div>
{/if}
