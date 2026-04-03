<script lang="ts">
	import Thread from '$lib/components/room/thread/thread.svelte';
	import { onMount } from 'svelte';
	import type { PageProps } from './$types';
	import {
		MatrixSvelteEmitEvent,
		type UpdateCurrentActiveRoom
	} from 'tauri-plugin-matrix-svelte-api';
	import { roomNameToPlainString } from '$lib/utils.svelte';
	import { roomsCollection } from '../../../hooks.client';
	import { emit } from '@tauri-apps/api/event';

	let { data }: PageProps = $props();

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

<Thread roomStore={data.roomStore} rootEventId={data.threadRoot} roomAvatarUrl={data.avatarUri} />
