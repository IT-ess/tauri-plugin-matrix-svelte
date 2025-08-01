<script lang="ts">
	import type { PageProps } from './$types';
	import RoomList from '$lib/components/room-list/room-list.svelte';
	import { goto } from '$app/navigation';
	import Profile from '$lib/components/room-list/profile.svelte';
	import { fetchMedia } from 'tauri-plugin-matrix-svelte-api';
	import { invoke } from '@tauri-apps/api/core';
	import SyncIndicator from '$lib/components/room-list/sync-indicator.svelte';

	let { data }: PageProps = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.loginStore.state);
	}

	function setCurrentActiveRoomAndGoToRoomRoute(id: string): undefined {
		data.roomsCollection.state.currentActiveRoom = id;
		goto(`/room`);
	}

	$effect(() => {
		if (data.loginStore.state.userId) {
			checkAvatarDataUri(data.loginStore.state.userId);
		}
	});

	const checkAvatarDataUri = async (key: string) => {
		if (data.profileStore.state?.[key] && data.profileStore.state?.[key].state == 'loaded') {
			if (
				data.profileStore.state[key].data.avatarUrl !== undefined &&
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
				roomId: undefined
			});
		}
	};
</script>

<div class="m-2"><SyncIndicator loginStore={data.loginStore} /></div>
<Profile
	loginStore={data.loginStore}
	profileStore={data.profileStore}
	currentUserId={data.loginStore.state.userId ?? ''}
/>
<RoomList
	roomsCollection={data.roomsCollection}
	onRoomClick={setCurrentActiveRoomAndGoToRoomRoute}
/>
