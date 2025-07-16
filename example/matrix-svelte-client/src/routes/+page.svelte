<script lang="ts">
	import type { PageProps } from './$types';
	import RoomList from '$lib/components/room-list/room-list.svelte';
	import { goto } from '$app/navigation';

	let { data }: PageProps = $props();

	function setCurrentActiveRoomAndGoToRoomRoute(id: string): undefined {
		data.roomsCollection.state.currentActiveRoom = id;
		goto(`/room`);
	}
</script>

<main class="container">
	<RoomList
		roomsCollection={data.roomsCollection}
		onRoomClick={setCurrentActiveRoomAndGoToRoomRoute}
		createDMRoomForm={data.createDMRoomForm}
	/>
</main>
