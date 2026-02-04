<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { XIcon } from '@lucide/svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { m } from '$lib/paraglide/messages';
	import SelectableRoomItem from './selectable-room-item.svelte';
	import { cn, roomNameToPlainString } from '$lib/utils.svelte';
	import SearchRooms from '$lib/components/room-list/search-rooms.svelte';
	import { roomsCollection } from '../../../hooks.client';
	import { filterRoomList } from 'tauri-plugin-matrix-svelte-api';

	let {
		selectedRoomsIds = $bindable([]),
		hideGroups,
		hideDirectUsersList,
		height
	}: {
		selectedRoomsIds: string[];
		hideGroups: boolean;
		hideDirectUsersList?: string[];
		height?: number | string;
	} = $props();

	const toggleItem = (roomId: string) => {
		const isSelected = selectedRoomsIds.some((r) => r === roomId);
		if (isSelected) {
			selectedRoomsIds = selectedRoomsIds.filter((r) => r !== roomId);
		} else {
			selectedRoomsIds = [...selectedRoomsIds, roomId];
		}
	};

	const removeMember = (roomId: string) => {
		selectedRoomsIds = selectedRoomsIds.filter((r) => r !== roomId);
	};

	let searchQuery = $state<string>('');

	$effect(() => {
		filterRoomList(searchQuery);
	});
</script>

<div class="flex flex-col gap-2">
	<!-- Wrapper div so the size of the ui doesn't change when we select a room -->
	<div class="flex min-h-6">
		{#if selectedRoomsIds.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each selectedRoomsIds as id (id)}
					<Badge variant="secondary" class="flex items-center gap-2 pl-2">
						<span class="text-xs"
							>{roomNameToPlainString(roomsCollection.state.allJoinedRooms[id].roomName)}</span
						>
						<button onclick={() => removeMember(id)} class="hover:text-destructive ml-1">
							<XIcon class="size-3" />
						</button>
					</Badge>
				{/each}
			</div>
		{/if}
	</div>

	{#if hideGroups}
		{#if hideDirectUsersList}
			{@render roomCard(
				roomsCollection.state.displayedDirectRooms.filter(
					(id) =>
						!hideDirectUsersList.includes(
							roomsCollection.state.allJoinedRooms[id].directUserId ?? ''
						)
				)
			)}
		{:else}
			{@render roomCard(roomsCollection.state.displayedDirectRooms)}
		{/if}
	{:else}
		<Tabs.Root value="dm">
			<Tabs.List>
				<Tabs.Trigger value="dm">{m.direct_messages()}</Tabs.Trigger>
				<Tabs.Trigger value="groups">{m.groups()}</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="dm">
				{@render roomCard(roomsCollection.state.displayedDirectRooms)}
			</Tabs.Content>
			<Tabs.Content value="groups">
				{@render roomCard(roomsCollection.state.displayedRegularRooms)}
			</Tabs.Content>
		</Tabs.Root>
	{/if}

	<p class="text-muted-foreground text-xs">
		{selectedRoomsIds.length} member{selectedRoomsIds.length !== 1 ? 's' : ''} selected
	</p>
</div>

{#snippet roomCard(roomIds: string[])}
	<div class="space-y-4">
		<SearchRooms bind:searchQuery />
		<ScrollArea class={cn('rounded-lg border', `h-${height ?? 48}`)}>
			{#each roomIds as id (id)}
				<SelectableRoomItem room={roomsCollection.state.allJoinedRooms[id]} {toggleItem} />
			{/each}
		</ScrollArea>
	</div>
{/snippet}
