<script lang="ts">
	import type { PageProps } from './$types';
	import Room from '$lib/components/room/room.svelte';
	import { onMount } from 'svelte';
	import {
		MatrixSvelteEmitEvent,
		type UpdateCurrentActiveRoom
	} from 'tauri-plugin-matrix-svelte-api';
	import { roomNameToPlainString } from '$lib/utils.svelte';
	import { roomsCollection } from '../../hooks.client';
	import { emit } from '@tauri-apps/api/event';

	let { data }: PageProps = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.roomStore.state);
	}

	onMount(() => {
		const payload: UpdateCurrentActiveRoom = {
			roomId: data.roomStore.id,
			// Kinda weird, but otherwise the room name is never initiated or
			// requires an additional fetch from the backend
			roomName: roomNameToPlainString(
				roomsCollection.state.allJoinedRooms[data.roomStore.id].roomName
			)
		};
		emit(MatrixSvelteEmitEvent.UpdateCurrentActiveRoom, payload);
	});
</script>

<Room roomStore={data.roomStore} roomAvatarUrl={data.avatarUri} />
