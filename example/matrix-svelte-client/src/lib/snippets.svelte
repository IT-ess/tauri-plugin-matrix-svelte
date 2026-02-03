<script lang="ts" module>
	import { getInitials } from '$lib/utils.svelte';
	import { AvatarFallback } from '$lib/components/ui/avatar';
	import AvatarImage from './components/ui/avatar/avatar-image.svelte';
	import { fetchMedia } from 'tauri-plugin-matrix-svelte-api';

	export { avatarFallback, fetchAvatar };
</script>

{#snippet avatarFallback(username: string | null | undefined)}
	<AvatarFallback>{getInitials(username ?? '?')}</AvatarFallback>
{/snippet}

{#snippet fetchAvatar(mxcUri: string, alt: string)}
	{#await fetchMedia({ source: { url: mxcUri }, format: 'File' }) then url}
		<AvatarImage src={url} {alt} />
	{/await}
{/snippet}
