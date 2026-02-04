<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import { Badge } from '$lib/components/ui/badge';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { roomNameToPlainString } from '$lib/utils.svelte';
	import type { JoinedRoomInfo } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		room: JoinedRoomInfo;
		disabled: boolean;
	};

	let { room, disabled }: Props = $props();

	const getLocalTimeAsFormattedString = (timestamp: number) => {
		const date = new Date(timestamp);
		const month = String(date.getMonth() + 1).padStart(2, '0'); // Months are 0-indexed
		const day = String(date.getDate()).padStart(2, '0');
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		return `${hours}h${minutes} ${day}-${month}`;
	};

	let latestEvent = $derived(room.latest ? room.latest[1] : 'Placeholder for last message');

	let avatarUri = $derived.by(() => {
		if (room.directUserId && room.heroes.length > 0) {
			return room.heroes[0].avatar_url;
		} else {
			return room.avatar;
		}
	});
</script>

<div
	class={{
		'flex items-center gap-4 p-4 transition-colors': true,
		'hover:bg-muted/50 cursor-pointer': !disabled,
		'cursor-not-allowed opacity-50': disabled
	}}
	role="button"
	tabindex={disabled ? -1 : 0}
>
	<Avatar>
		{#if avatarUri}
			{@render fetchAvatar(avatarUri, roomNameToPlainString(room.roomName))}
		{/if}
		{@render avatarFallback(roomNameToPlainString(room.roomName))}
	</Avatar>
	<a
		class="flex-1 space-y-1"
		data-sveltekit-preload-data="tap"
		href={`/room?id=${encodeURIComponent(room.roomId)}${avatarUri ? '&avatar=' + encodeURIComponent(avatarUri) : ''}#bottomscroll`}
	>
		<div class="flex items-center justify-between">
			<h4 class="font-semibold">{roomNameToPlainString(room.roomName)}</h4>
			<span class="text-muted-foreground text-sm"
				>{getLocalTimeAsFormattedString(room.latest ? room.latest[0] : 0)}</span
			>
		</div>
		<div class="flex items-center justify-between">
			<p class="text-muted-foreground line-clamp-1 text-sm">
				<!-- the latest event is sanitized by the backend before being rendered -->
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html latestEvent}
			</p>
			{#if room.numUnreadMessages > 0}
				<Badge variant="default" class="ml-2">{room.numUnreadMessages}</Badge>
			{/if}
		</div>
	</a>
</div>
