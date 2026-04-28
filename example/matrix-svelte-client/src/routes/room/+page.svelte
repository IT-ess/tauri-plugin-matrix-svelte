<script lang="ts">
	import type { PageProps } from './$types';
	import RoomHeader from '$lib/components/room/room-header.svelte';
	import RoomTimeline from '$lib/components/room/room-timeline.svelte';

	let { data }: PageProps = $props();
</script>

<div class="bg-background pb-tauri-bottom-safe relative flex h-full flex-col">
	<RoomHeader initialAvatarUrl={data.avatarUri} roomId={data.roomId} />
	<RoomTimeline roomId={data.roomId} roomAvatarUrl={data.avatarUri} threadRoot={null} />
</div>
