<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
	import RoomListItem from './room-list-item.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import ListActions from './actions/list-actions.svelte';
	import SearchRooms from './search-rooms.svelte';
	import { m } from '$lib/paraglide/messages';
	import InviteProfileItem from '../profiles/invite-profile-item.svelte';
	import { useSearchParams } from 'runed/kit';
	import { roomsListSearchParamsSchema } from '$lib/utils.svelte';
	import { roomsCollection } from '../../../hooks.client';
	import { filterRoomList } from 'tauri-plugin-matrix-svelte-api';

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(roomsCollection.state);
	}
	let searchQuery = $state<string>('');

	const params = useSearchParams(roomsListSearchParamsSchema);

	$effect(() => {
		filterRoomList(searchQuery);
	});

	let disabled = $derived(!['loaded', 'loading'].includes(roomsCollection.state.status.status));
</script>

<div
	class="pb-tauri-bottom-safe mx-auto mt-2 flex h-full w-full max-w-md flex-col overflow-y-auto px-1"
>
	<Tabs.Root bind:value={params.tab}>
		<Tabs.List class="mx-auto">
			<Tabs.Trigger value="dm">{m.direct_messages()}</Tabs.Trigger>
			<Tabs.Trigger value="groups">{m.groups()}</Tabs.Trigger>
			{#if roomsCollection.state.displayedInvitedRooms.length > 0}
				<Tabs.Trigger value="invites">{m.invites()}</Tabs.Trigger>
			{/if}
		</Tabs.List>
		<Tabs.Content value="dm">
			{@render roomsCard(roomsCollection.state.displayedDirectRooms)}
		</Tabs.Content>
		<Tabs.Content value="groups">
			{@render roomsCard(roomsCollection.state.displayedRegularRooms)}
		</Tabs.Content>
		{#if roomsCollection.state.displayedInvitedRooms.length > 0}
			<Tabs.Content value="invites">
				<Card.Root>
					<Card.Header>
						<Card.Title>{m.invites()}</Card.Title>
						<Card.Description>{m.invitation_rooms_desc()}</Card.Description>
					</Card.Header>
					<Card.Content class="grid gap-6">
						<div class="divide-border divide-y">
							{#each roomsCollection.state.displayedInvitedRooms as invitedRoomId (invitedRoomId)}
								<InviteProfileItem
									invitedRoomInfo={roomsCollection.state.invitedRooms[invitedRoomId]}
								/>
							{/each}
						</div>
					</Card.Content>
				</Card.Root>
			</Tabs.Content>
		{/if}
	</Tabs.Root>
	<ListActions />
</div>

{#snippet roomsCard(roomIds: string[])}
	<SearchRooms bind:searchQuery />
	<div class="divide-border divide-y overflow-y-scroll">
		{#each roomIds as id (id)}
			<RoomListItem room={roomsCollection.state.allJoinedRooms[id]} {disabled} />
		{/each}
	</div>
{/snippet}
