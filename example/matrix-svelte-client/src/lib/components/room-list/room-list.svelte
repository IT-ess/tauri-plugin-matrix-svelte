<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
	import type { RoomsCollection } from 'tauri-plugin-matrix-svelte-api';
	import RoomListItem from './room-list-item.svelte';
	import { getSortedRoomArray } from './utils.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import InviteListItem from './invite-list-item.svelte';
	import ListActions from './actions/list-actions.svelte';

	// Props type for conversations list
	type Props = {
		roomsCollection: RoomsCollection;
		onRoomClick: (id: string) => undefined;
	};
	let { roomsCollection, onRoomClick }: Props = $props();

	if (import.meta.env.DEV) {
		$inspect(roomsCollection.state);
	}
	let rooms = $derived.by(getSortedRoomArray(roomsCollection));
	let invitedRooms = $derived(Object.values(roomsCollection.state.invitedRooms));

	let disabled = $derived(roomsCollection.state.status.status !== 'loaded');
</script>

<div class="mx-auto flex w-full max-w-sm flex-col gap-6">
	{#if invitedRooms.length > 0}
		<Tabs.Root value="rooms">
			<Tabs.List>
				<Tabs.Trigger value="rooms">Rooms</Tabs.Trigger>
				<Tabs.Trigger value="invites">Invitations</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="rooms">
				{@render roomsCard()}
			</Tabs.Content>
			<Tabs.Content value="invites">
				<Card.Root>
					<Card.Header>
						<Card.Title>Invitations</Card.Title>
						<Card.Description>Rooms you've been invited to</Card.Description>
					</Card.Header>
					<Card.Content class="grid gap-6">
						<div class="divide-border divide-y">
							{#each invitedRooms as room (room.roomId)}
								<InviteListItem {room} {disabled} />
							{/each}
						</div>
					</Card.Content>
				</Card.Root>
			</Tabs.Content>
		</Tabs.Root>
	{:else}
		{@render roomsCard()}
	{/if}
	<ListActions />
</div>

{#snippet roomsCard()}
	<Card.Root class="w-full max-w-sm">
		<Card.Header>
			<Card.Title>Rooms</Card.Title>
			<Card.Description>All your discussions lay here</Card.Description>
		</Card.Header>
		<Card.Content>
			<div class="divide-border divide-y">
				{#each rooms as room (room.roomId)}
					<RoomListItem {room} {onRoomClick} {disabled} />
				{/each}
			</div>
		</Card.Content>
	</Card.Root>
{/snippet}
