<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { PageProps } from './$types';
	import { events, fetchMedia } from 'tauri-plugin-matrix-svelte-api';
	import { emit } from '@tauri-apps/api/event';
	import Room from '$lib/components/room/room.svelte';
	import { invoke } from '@tauri-apps/api/core';

	let { data }: PageProps = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.roomStore.state);
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.profileStore.state);
	}

	onDestroy(async () => {
		// TODO: verify the behaviour
		let payload: events.UpdateCurrentActiveRoom = {
			roomId: null,
			roomName: null
		};
		await emit(events.MatrixSvelteEmitEvent.UpdateCurrentActiveRoom, payload);
	});

	let roomMembers = $derived(Object.keys(data.roomStore.state.members));

	// Fetch all avatarDataUrl depending on the room members
	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		roomMembers;
		for (let key of roomMembers) {
			checkAvatarDataUri(key).catch((e) => console.error(e));
		}
	});

	const checkAvatarDataUri = async (key: string) => {
		if (data.profileStore.state?.[key] && data.profileStore.state?.[key].state == 'loaded') {
			if (
				data.profileStore.state[key].data.avatarUrl !== undefined &&
					data.profileStore.state[key].data.avatarUrl !== null &&
				data.profileStore.state[key].data.avatarDataUrl === undefined
			) {
				data.profileStore.state[key].data.avatarDataUrl = await fetchMedia({
					format: 'File',
					source: { url: data.profileStore.state[key].data.avatarUrl }
				});
			}
		} else if (
			data.profileStore.state?.[key] &&
			data.profileStore.state?.[key].state === 'requested'
		) {
			return;
		} else {
			await invoke('plugin:matrix-svelte|fetch_user_profile', {
				userId: key,
				roomId: data.roomStore.id
			});
		}
	};
</script>

<h1>Room with id {data.roomStore.id}</h1>
<Room
	roomStore={data.roomStore}
	profileStore={data.profileStore}
	currentUserId={data.loginStore.state.userId ?? ''}
/>
<!-- userId should be defined at this point -->
