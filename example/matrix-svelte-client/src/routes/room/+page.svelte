<script lang="ts">
	import type { PageProps } from './$types';
	import Room from '$lib/components/room/room.svelte';

	let { data }: PageProps = $props();
</script>

<Room roomId={data.roomId} roomAvatarUrl={data.avatarUri} />
