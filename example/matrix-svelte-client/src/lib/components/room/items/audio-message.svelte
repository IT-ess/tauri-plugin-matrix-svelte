<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		type AudioMessageEventContent,
		fetchMedia,
		MediaLoadingState
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: AudioMessageEventContent;
	};

	let { itemContent }: Props = $props();

	// let alt = itemContent.filename ?? itemContent.body;

	// State variables
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let audioSrc = $state<string>('');
	let loadingState = new MediaLoadingState(itemContent.info?.size ?? 1);

	// Load image function
	const loadAudio = async () => {
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
		loadAudio();
	});
</script>

<div class="bg-card relative mt-1 overflow-hidden rounded-lg border">
	{#if !loadingState.isLoaded}
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
					<Button variant="secondary" size="sm" onclick={() => loadAudio()}>Retry</Button>
				</div>
			</div>
		{/if}
	{:else}
		<!-- Loaded Audio -->
		<audio src={audioSrc} controls></audio>
	{/if}
</div>
