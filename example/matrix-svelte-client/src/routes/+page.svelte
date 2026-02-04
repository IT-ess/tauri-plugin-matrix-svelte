<script lang="ts">
	import Header from '$lib/components/common/header.svelte';
	import RoomList from '$lib/components/room-list/room-list.svelte';
	import { roomsCollection } from '../hooks.client';

	let currentInvitesNumber = $derived(Object.keys(roomsCollection.state.invitedRooms).length);
</script>

<div class="relative flex h-full w-full flex-col overflow-hidden">
	<Header {currentInvitesNumber} />
	<RoomList />
</div>
