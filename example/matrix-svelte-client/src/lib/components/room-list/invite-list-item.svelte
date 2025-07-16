<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import {
		createMatrixRequest,
		mediaCache,
		submitAsyncRequest,
		type InvitedRoomInfo
	} from 'tauri-plugin-matrix-svelte-api';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';

	type Props = {
		room: InvitedRoomInfo;
		disabled: boolean;
	};

	let { room, disabled }: Props = $props();

	// Get initials from name for avatar fallback
	const getInitials = (name: string) => {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	};

	const getLocalTimeAsFormattedString = (timestamp: number) => {
		const date = new Date(timestamp);
		const month = String(date.getMonth() + 1).padStart(2, '0'); // Months are 0-indexed
		const day = String(date.getDate()).padStart(2, '0');
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		return `${hours}h${minutes} ${day}-${month}`;
	};

	let latestEvent = $derived(room.latest ? room.latest[1] : 'Placeholder for last message');

	const joinRoom = async () => {
		const request = createMatrixRequest.joinRoom({
			roomId: room.roomId
		});
		await submitAsyncRequest(request);
		openAlertDialog = false;
	};

	const rejectRoom = async () => {
		const request = createMatrixRequest.leaveRoom({
			roomId: room.roomId
		});
		await submitAsyncRequest(request);
		openAlertDialog = false;
	};

	let openAlertDialog = $state(false);
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
		{#if room.roomAvatar !== null}
			{#await mediaCache.get(room.roomAvatar)}
				{@render avatarFallback(room.roomName)}
			{:then url}
				<AvatarImage src={url} alt={room.roomName} />
			{:catch}
				{@render avatarFallback(room.roomName)}
			{/await}
		{:else}
			{@render avatarFallback(room.roomName)}
		{/if}
	</Avatar>
	<button {disabled} class="flex-1 space-y-1" onclick={() => (openAlertDialog = true)}>
		<div class="flex items-center justify-between">
			<h4 class="font-semibold">{room.roomName}</h4>
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
		</div>
	</button>
</div>

<AlertDialog.Root bind:open={openAlertDialog}>
	<AlertDialog.Portal>
		<AlertDialog.Overlay />
		<AlertDialog.Content>
			<AlertDialog.Title>Join room</AlertDialog.Title>
			<AlertDialog.Description>You've been invited to {room.roomName}</AlertDialog.Description>
			<AlertDialog.Cancel type="button">Cancel</AlertDialog.Cancel>
			<AlertDialog.Action onclick={() => rejectRoom()} type="reset">Refuse</AlertDialog.Action>
			<AlertDialog.Action onclick={() => joinRoom()} type="submit">Accept</AlertDialog.Action>
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>

{#snippet avatarFallback(roomName: string)}
	<AvatarFallback>{getInitials(roomName)}</AvatarFallback>
{/snippet}
