<script lang="ts">
	import type { PageProps } from './$types';
	import Room from '$lib/components/room/room.svelte';

	let { data }: PageProps = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.roomStore.state);
	}
</script>

<Room roomStore={data.roomStore} roomAvatarUrl={data.avatarUri} />
