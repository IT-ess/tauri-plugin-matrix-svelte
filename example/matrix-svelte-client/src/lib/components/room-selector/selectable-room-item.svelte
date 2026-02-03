<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { checkUserInProfileStore, cn, roomNameToPlainString } from '$lib/utils.svelte';
	import { onMount } from 'svelte';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { profileStore } from '../../../hooks.client';
	import type { JoinedRoomInfo } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		room: JoinedRoomInfo;
		toggleItem: (roomId: string) => void;
	};

	let { room, toggleItem }: Props = $props();

	// svelte-ignore state_referenced_locally
	const directUserId = room.directUserId;

	let avatarUri = $derived.by(() => {
		if (
			directUserId &&
			profileStore.state[directUserId] &&
			profileStore.state[directUserId].state === 'loaded' &&
			profileStore.state[directUserId].data.avatarUrl
		) {
			return profileStore.state[directUserId].data.avatarUrl;
		} else {
			return room.avatar;
		}
	});

	onMount(async () => {
		if (directUserId) {
			await checkUserInProfileStore(directUserId);
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
		{#if avatarUri}
			{@render fetchAvatar(avatarUri, roomNameToPlainString(room.roomName))}
		{/if}
		{@render avatarFallback(roomNameToPlainString(room.roomName))}
	</Avatar>
	<div
		class="hover:bg-muted flex w-full items-center justify-between rounded-lg p-2 transition-colors"
	>
		<span class="flex-1 text-left text-sm">{roomNameToPlainString(room.roomName)}</span>
		<Checkbox bind:checked class="pointer-events-none" />
	</div>
</button>
