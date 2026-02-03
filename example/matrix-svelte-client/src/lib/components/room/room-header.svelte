<script lang="ts">
	import { ChevronLeft } from '@lucide/svelte';
	import { Avatar } from '../ui/avatar';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import ActionBurgerMenu from './actions/action-burger-menu.svelte';
	import { gotoRoomsList, roomNameToPlainString } from '$lib/utils.svelte';
	import { roomsCollection } from '../../../hooks.client';
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

	let actionRoomDetailsOpen = $state(false);

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomId].isDirect);
	// It seems that DM rooms avatar behave differently that regular rooms, so we need to use
	// the user's avatar for direct rooms, and use the reactive one for regular rooms
	let avatarUrl = $derived(
		isDirect ? initialAvatarUrl : roomsCollection.state.allJoinedRooms[roomId].avatar
	);
</script>

<header
	class="bg-secondary pt-safe supports-backdrop-filter:bg-secondary/60 sticky top-0 z-50 w-full border-b backdrop-blur"
>
	<div class="relative flex h-16 items-center gap-3 px-4">
		<button
			onclick={() =>
				gotoRoomsList(
					roomsCollection.state.allJoinedRooms[roomStore.id].isDirect ? 'dm' : 'groups'
				)}
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

		<!-- svelte-ignore a11y_interactive_supports_focus -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div onclick={() => (actionRoomDetailsOpen = true)} role="button" class="min-w-0 flex-1">
			<h1 class="text-foreground truncate text-base font-semibold">
				{roomNameToPlainString(roomsCollection.state.allJoinedRooms[roomId].roomName)}
			</h1>
			<p class="text-muted/80 truncate text-sm">
				{roomsCollection.state.allJoinedRooms[roomId].topic}
			</p>
		</div>

		<ActionBurgerMenu {roomStore} {avatarUrl} bind:actionRoomDetailsOpen />
	</div>
</header>
