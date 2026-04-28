<script lang="ts">
	import RoomTimeline from '$lib/components/room/room-timeline.svelte';
	import ThreadHeader from '$lib/components/room/thread/thread-header.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<div class="bg-background pb-tauri-bottom-safe relative flex h-full flex-col">
	<ThreadHeader roomId={data.roomId} initialAvatarUrl={data.avatarUri} />
	<RoomTimeline roomId={data.roomId} roomAvatarUrl={data.avatarUri} threadRoot={data.threadRoot} />
</div>
