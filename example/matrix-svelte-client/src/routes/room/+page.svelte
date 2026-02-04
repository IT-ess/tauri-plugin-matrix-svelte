<script lang="ts">
	import type { PageProps } from './$types';
	import Room from '$lib/components/room/room.svelte';
	import { checkUserInProfileStore } from '$lib/utils.svelte';

	let { data }: PageProps = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(data.roomStore.state);
	}

	let roomMembers = $derived(Object.keys(data.roomStore.state.members));

	// Fetch all avatarDataUrl depending on the room members
	$effect(() => {
		for (let key of roomMembers) {
			checkUserInProfileStore(key).catch((e) => console.error(e));
		}
	});
</script>

<Room roomStore={data.roomStore} roomAvatarUrl={data.avatarUri} />
