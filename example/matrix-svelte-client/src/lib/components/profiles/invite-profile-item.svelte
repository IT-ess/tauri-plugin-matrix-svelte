<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Card } from '$lib/components/ui/card';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { roomNameToPlainString } from '$lib/utils.svelte';
	import { m } from '$lib/paraglide/messages';
	import { buttonVariants } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { User, Users } from '@lucide/svelte';
	import { profileStore } from '../../../hooks.client';
	import {
		createMatrixRequest,
		submitAsyncRequest,
		type InvitedRoomInfo
	} from 'tauri-plugin-matrix-svelte-api';

	let {
		invitedRoomInfo
	}: {
		invitedRoomInfo: InvitedRoomInfo;
	} = $props();

	const joinRoom = async () => {
		const request = createMatrixRequest.joinRoom({
			roomId: invitedRoomInfo.roomId
		});
		await submitAsyncRequest(request);
		openAlertDialog = false;
	};

	const rejectRoom = async () => {
		const request = createMatrixRequest.leaveRoom({
			roomId: invitedRoomInfo.roomId
		});
		await submitAsyncRequest(request);
		openAlertDialog = false;
	};

	let openAlertDialog = $state(false);
	// svelte-ignore state_referenced_locally
	const inviterUserId = invitedRoomInfo.inviterInfo ? invitedRoomInfo.inviterInfo.userId : null;
	onMount(async () => {
		if (inviterUserId && profileStore.state[inviterUserId] === undefined) {
			await invoke('fetch_user_profile', {
				userId: inviterUserId,
				roomId: invitedRoomInfo.roomId
			});
		}
	});
</script>

<Card class="hover:bg-accent/50 cursor-pointer p-4 transition-colors">
	<button class="relative flex items-center gap-3" onclick={() => (openAlertDialog = true)}>
		<Avatar class="h-10 w-10">
			{#if inviterUserId && profileStore.state[inviterUserId] && profileStore.state[inviterUserId].state === 'loaded' && profileStore.state[inviterUserId].data.avatarUrl}
				{@render fetchAvatar(
					profileStore.state[inviterUserId].data.avatarUrl,
					profileStore.state[inviterUserId].data.username ?? '?'
				)}
			{/if}
			{@render avatarFallback(roomNameToPlainString(invitedRoomInfo.roomName))}
		</Avatar>

		<div class="min-w-0 flex-1">
			<h3 class="text-foreground truncate font-medium">
				{roomNameToPlainString(invitedRoomInfo.roomName)}
			</h3>
			<p class="text-muted-foreground truncate text-sm">
				{m.invitation_user_invited_you({
					user:
						invitedRoomInfo?.inviterInfo?.displayName ??
						roomNameToPlainString(invitedRoomInfo.roomName),
					userId: invitedRoomInfo?.inviterInfo?.userId ?? '?'
				})}
			</p>
		</div>
		<Badge class="absolute top-0 right-0">
			{#if invitedRoomInfo.isDirect}
				<User />
			{:else}
				<Users />
			{/if}
		</Badge>
	</button>
</Card>

<AlertDialog.Root bind:open={openAlertDialog}>
	<AlertDialog.Portal>
		<AlertDialog.Overlay />
		<AlertDialog.Content>
			<AlertDialog.Title>{m.invite()}</AlertDialog.Title>
			<AlertDialog.Description class="text-lg">
				{#if invitedRoomInfo.isDirect}
					{m.invitation_direct_description({
						user:
							invitedRoomInfo?.inviterInfo?.displayName ??
							roomNameToPlainString(invitedRoomInfo.roomName),
						userId: invitedRoomInfo?.inviterInfo?.userId ?? '?'
					})}
				{:else}
					{m.invitation_room_description({
						room: roomNameToPlainString(invitedRoomInfo.roomName),
						user:
							invitedRoomInfo?.inviterInfo?.displayName ??
							invitedRoomInfo?.inviterInfo?.userId ??
							'?',
						userId: invitedRoomInfo?.inviterInfo?.userId ?? '?'
					})}
				{/if}
			</AlertDialog.Description>
			<AlertDialog.Action onclick={() => joinRoom()} type="submit"
				>{m.button_confirm()}</AlertDialog.Action
			>
			<AlertDialog.Action
				class={buttonVariants({ variant: 'destructive' })}
				onclick={() => rejectRoom()}
				type="reset">{m.button_remove()}</AlertDialog.Action
			>
			<AlertDialog.Cancel type="button">{m.button_cancel()}</AlertDialog.Cancel>
			<Badge class="absolute top-4 right-4">
				{#if invitedRoomInfo.isDirect}
					{m.direct_message()} <User />
				{:else}
					{m.group()} <Users />
				{/if}
			</Badge>
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>
