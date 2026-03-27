<script lang="ts">
	import RoomInfo from '$lib/components/room/info/room-info.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<RoomInfo avatar={data.avatarUri} roomStore={data.roomStore} />
