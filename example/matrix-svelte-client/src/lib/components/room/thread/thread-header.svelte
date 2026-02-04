<script lang="ts">
	import { ChevronLeft } from '@lucide/svelte';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { gotoRoom, roomNameToPlainString } from '$lib/utils.svelte';
	import { Avatar } from '$lib/components/ui/avatar';
	import { roomsCollection } from '../../../../hooks.client';
	import { m } from '$lib/paraglide/messages';
	import type { RoomStore } from 'tauri-plugin-matrix-svelte-api';

	let {
		roomStore,
		initialAvatarUrl
	}: {
		roomStore: RoomStore;
		initialAvatarUrl: string | null;
	} = $props();

	// svelte-ignore state_referenced_locally
	const roomId = roomStore.id;

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomId].isDirect);
	// It seems that DM rooms avatar behave differently that regular rooms, so we need to use
	// the user's avatar for direct rooms, and use the reactive one for regular rooms
	let avatarUrl = $derived(
		isDirect ? initialAvatarUrl : roomsCollection.state.allJoinedRooms[roomId].avatar
	);
</script>

<header class="pt-safe sticky top-0 z-50 w-full border-b">
	<div class="relative flex h-16 items-center gap-3 px-4">
		<button
			onclick={() => gotoRoom(roomStore.id, avatarUrl)}
			class="hover:bg-accent flex h-10 w-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>

		<Avatar>
			{#if avatarUrl}
				{@render fetchAvatar(
					avatarUrl,
					roomNameToPlainString(roomsCollection.state.allJoinedRooms[roomId].roomName)
				)}
			{/if}
			{@render avatarFallback(
				roomNameToPlainString(roomsCollection.state.allJoinedRooms[roomId].roomName)
			)}
		</Avatar>
		<h1 class="text-foreground truncate text-base font-semibold">{m.room_thread_header()}</h1>
	</div>
</header>
