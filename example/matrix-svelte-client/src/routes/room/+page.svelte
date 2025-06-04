<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { PageProps } from './$types';
	import { events } from 'tauri-plugin-matrix-svelte-api';
	import { emit } from '@tauri-apps/api/event';
	import Room from '$lib/components/room/room.svelte';

	let { data }: PageProps = $props();

	$inspect(data.roomStore.state);

	onDestroy(async () => {
		// TODO: verify the behaviour
		let payload: events.UpdateCurrentActiveRoom = {
			roomId: null,
			roomName: null
		};
		await emit(events.MatrixSvelteEmitEvent.UpdateCurrentActiveRoom, payload);
	});
</script>

<h1>Room with id {data.roomStore.id}</h1>
<Room roomStore={data.roomStore} />
