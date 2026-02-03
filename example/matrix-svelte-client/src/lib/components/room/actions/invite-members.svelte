<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { Spinner } from '$lib/components/ui/spinner';
	import RoomSelector from '$lib/components/room-selector/room-selector.svelte';
	import { roomsCollection } from '../../../../hooks.client';
	import { createMatrixRequest, submitAsyncRequest } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		actionInviteMembersOpen: boolean;
		roomId: string;
		previousUsersIdsList: string[];
	};

	let {
		actionInviteMembersOpen = $bindable(false),
		roomId,
		previousUsersIdsList
	}: Props = $props();

	let isLoading = $state(false);
	let selectedRoomsIds = $state<string[]>([]);

	const handleInviteMembers = async () => {
		isLoading = true;

		try {
			const invitedUserIds = selectedRoomsIds
				.map((id) => roomsCollection.state.allJoinedRooms[id].directUserId)
				.filter((userId) => userId !== null);
			let request = createMatrixRequest.inviteUsersInRoom({
				roomId,
				invitedUserIds
			});

			await submitAsyncRequest(request);
		} catch (error) {
			console.error(error);
		} finally {
			isLoading = false;
			actionInviteMembersOpen = false;
			selectedRoomsIds = [];
		}
	};
</script>

<Dialog.Root bind:open={actionInviteMembersOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.invites()}</Dialog.Title>
			<Dialog.Description>{m.room_invite_members_subtitle()}</Dialog.Description>
		</Dialog.Header>

		<RoomSelector
			bind:selectedRoomsIds
			hideGroups
			hideDirectUsersList={previousUsersIdsList}
			height={72}
		/>

		<Button onclick={handleInviteMembers} disabled={isLoading}>
			{#if isLoading}
				<Spinner />
			{/if}
			{m.invite()}
		</Button>
		<Dialog.Close disabled={isLoading} class={buttonVariants({ variant: 'outline' })}
			>{m.button_cancel()}</Dialog.Close
		>
	</Dialog.Content>
</Dialog.Root>
