<script lang="ts">
	import RoomInfo from '$lib/components/room/info/room-info.svelte';
	import { roomStore } from '../../../hooks.client';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<RoomInfo roomId={data.roomId} avatar={data.avatarUri} {roomStore} />
