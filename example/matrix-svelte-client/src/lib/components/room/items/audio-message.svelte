<script lang="ts">
	import AudioPlayer from '$lib/components/audio/audio-player.svelte';
	import { getCustomMxcUriFromOriginal, getUrlFromEncryptedSource } from '$lib/utils.svelte';
	import {
		audioMessageSourceIsPlain,
		type AudioMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: AudioMessageEventContent;
		isOwn: boolean;
	};

	let { itemContent, isOwn }: Props = $props();

	let src = $derived(
		audioMessageSourceIsPlain(itemContent)
			? (getCustomMxcUriFromOriginal(itemContent.url) as string)
			: getUrlFromEncryptedSource(itemContent.file)
	);
</script>

<div class="relative mt-1 h-full min-h-8 w-full overflow-x-hidden">
	<AudioPlayer
		{src}
		peaks={itemContent['org.matrix.msc1767.audio']?.waveform}
		initialDuration={itemContent.info?.duration}
		{isOwn}
	/>
</div>
