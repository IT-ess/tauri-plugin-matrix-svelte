<script lang="ts">
	import AudioPlayer from '$lib/components/audio/audio-player.svelte';
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import {
		audioMessageSourceIsPlain,
		fetchMedia,
		MediaLoadingState,
		type AudioMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: AudioMessageEventContent;
		isOwn: boolean;
	};

	let { itemContent, isOwn }: Props = $props();

	// State variables
	// svelte-ignore state_referenced_locally
	let loadingState = $state(new MediaLoadingState(itemContent.info?.size ?? 1));

	// Load image function
	const loadAudio = () => {
		if (audioMessageSourceIsPlain(itemContent)) {
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
</script>

<div class="relative mt-1 h-full min-h-8 w-full overflow-x-hidden">
	{#await loadAudio()}
		<div class="absolute inset-0 flex items-center justify-center bg-black/20">
			<div class="rounded-full bg-white/90 px-3 py-1 text-xs">
				{Math.round(loadingState.progress * 100)}%
			</div>
		</div>
	{:then src}
		<AudioPlayer
			{src}
			peaks={itemContent['org.matrix.msc1767.audio']?.waveform}
			initialDuration={itemContent.info?.duration}
			{isOwn}
		/>
	{:catch error}
		<div class="bg-destructive/80 absolute inset-0 flex items-center justify-center">
			<div class="text-center text-white">
				<p class="mb-2 text-sm">{m.failed_to_load()}</p>
				<p class="text-destructive">{error}</p>
				<Button variant="secondary" size="sm" onclick={() => loadAudio()}>{m.button_retry()}</Button
				>
			</div>
		</div>
	{/await}
</div>
