<script lang="ts">
	import { Card } from '$lib/components/ui/card';
	import type { RoomsCollection } from 'tauri-plugin-matrix-svelte-api';
	import RoomListItem from './room-list-item.svelte';
	import { getSortedRoomArray } from './utils.svelte';

	// Props type for conversations list
	type Props = {
		roomsCollection: RoomsCollection;
		onRoomClick: (id: string) => undefined;
	};
	let { roomsCollection, onRoomClick }: Props = $props();

	if (import.meta.env.DEV) {
		$inspect(roomsCollection.state);
	}
	let rooms = $derived.by(getSortedRoomArray(roomsCollection));

	let disabled = $derived(roomsCollection.state.status.status !== 'loaded');
</script>

<Card>
	<div class="divide-border divide-y">
		{#each rooms as room (room.roomId)}
			<RoomListItem {room} {onRoomClick} {disabled} />
		{/each}
	</div>
</Card>
