<script lang="ts">
	import RoomBookmarks from '$lib/components/room/info/bookmarks/room-bookmarks.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<RoomBookmarks roomId={data.roomId} avatar={data.avatarUri} />
