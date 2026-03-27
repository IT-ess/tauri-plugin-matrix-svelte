<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { cn, roomNameToPlainString } from '$lib/utils.svelte';
	import { onMount } from 'svelte';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { fetchUserProfile, type JoinedRoomInfo } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		room: JoinedRoomInfo;
		toggleItem: (roomId: string) => void;
	};

	let { room, toggleItem }: Props = $props();

	// svelte-ignore state_referenced_locally
	const directUserId = room.directUserId;

	let avatarUri = $derived.by(async () => {
		if (directUserId) {
			return fetchUserProfile(directUserId, room.roomId).then((p) => p.avatarUrl);
		} else {
			return room.avatar;
		}
	});

	let checked = $state(false);
	const handleToggle = () => {
		checked = !checked;
		toggleItem(room.roomId);
	};
</script>

<button
	class={cn(
		'border-muted hover:bg-muted/50 flex w-full cursor-pointer items-center gap-4 border-b p-4 transition-colors',
		checked ? 'bg-primary/20' : ''
	)}
	onclick={handleToggle}
>
	<Avatar>
		{#await avatarUri then uri}
			{#if uri}
				{@render fetchAvatar(uri, roomNameToPlainString(room.roomName))}
			{/if}
		{/await}
		{@render avatarFallback(roomNameToPlainString(room.roomName))}
	</Avatar>
	<div
		class="hover:bg-muted flex w-full items-center justify-between rounded-lg p-2 transition-colors"
	>
		<span class="flex-1 text-left text-sm">{roomNameToPlainString(room.roomName)}</span>
		<Checkbox bind:checked class="pointer-events-none" />
	</div>
</button>
