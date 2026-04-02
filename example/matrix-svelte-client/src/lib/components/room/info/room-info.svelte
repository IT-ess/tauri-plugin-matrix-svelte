<script lang="ts">
	import { getCustomMxcUriFromOriginal, getInitials, gotoRoom } from '$lib/utils.svelte';
	import { ChevronLeft, SquareArrowRightExit, SquarePen, User, UserPlus } from '@lucide/svelte';
	import { type RoomStore } from 'tauri-plugin-matrix-svelte-api';
	import { roomsCollection } from '../../../../hooks.client';
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';
	import * as Item from '$lib/components/ui/item/index.js';
	import InviteMembers from './members/invite-members.svelte';
	import LeaveRoom from './leave-room.svelte';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import RoomDetails from './room-details.svelte';

	let { avatar, roomStore }: { avatar: string | null; roomStore: RoomStore } = $props();

	let roomTopic = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].topic);

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].isDirect);

	let actionInviteMembersOpen = $state(false);
	let actionLeaveRoomOpen = $state(false);
	let actionRoomDetailsOpen = $state(false);

	let userPowers = $derived(roomStore.state.tlState?.userPower);

	// No need to filter the membership (join, leave...) of each member,
	// since we already filter it on the backend side.
	let membersIds = $derived(Object.keys(roomStore.state.members));
</script>

<div class="bg-background flex h-full w-full flex-col">
	<div class="pt-safe sticky top-0 right-0 left-0 z-50 w-full pl-2">
		<button
			onclick={() => gotoRoom(roomStore.id, avatar)}
			class="hover:bg-accent flex size-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>
	</div>
	<div class="flex min-h-3/8 flex-col justify-between">
		<div class="flex flex-col items-center gap-4">
			<Avatar class="size-24">
				<AvatarImage src={getCustomMxcUriFromOriginal(avatar)} alt={roomStore.state.roomName} />
				<AvatarFallback>{getInitials(roomStore.state.roomName)}</AvatarFallback>
			</Avatar>
			<h1 class="text-2xl font-bold">
				{roomStore.state.roomName}
			</h1>
			<div class="my-4 flex gap-6">
				{#if userPowers && userPowers.includes('roomAvatar')}
					<Button
						onclick={() => (actionRoomDetailsOpen = true)}
						class="flex flex-col gap-2 text-base"
						size="icon-lg"
						variant="ghost"
					>
						<SquarePen class="size-7 pl-px" />
						{m.button_edit()}
					</Button>
				{/if}
				<Button
					onclick={() => (actionInviteMembersOpen = true)}
					class="flex flex-col gap-2 text-base"
					size="icon-lg"
					variant="ghost"
				>
					<UserPlus class="size-7 pl-px" />
					{m.to_invite()}
				</Button>
			</div>
		</div>

		<div class="px-4">
			<Label for="group-topic" class="text-base font-semibold">{m.topic()}</Label>
			<p class="text-muted-foreground leading-7 not-first:mt-6">{roomTopic}</p>
		</div>
	</div>
	<Separator class="mt-4" />
	<Item.Root>
		{#snippet child({ props })}
			<a
				href={`/room/info/members?id=${encodeURIComponent(roomStore.id)}${avatar ? '&avatar=' + encodeURIComponent(avatar) : ''}`}
				{...props}
			>
				<Item.Media>
					<User class="text-muted-foreground size-6" />
				</Item.Media>
				<Item.Content>
					<Item.Title class="text-base">{m.members()}</Item.Title>
				</Item.Content>
				<Item.Actions>
					{membersIds.length}
				</Item.Actions>
			</a>
		{/snippet}
	</Item.Root>
	<Item.Root>
		{#snippet child({ props })}
			<button onclick={() => (actionLeaveRoomOpen = true)} {...props}>
				<Item.Media>
					<SquareArrowRightExit class="text-destructive size-6" />
				</Item.Media>
				<Item.Content>
					<Item.Title class="text-destructive text-base"
						>{m.room_action_menu_leave_room()}</Item.Title
					>
				</Item.Content>
			</button>
		{/snippet}
	</Item.Root>
</div>

<InviteMembers
	bind:actionInviteMembersOpen
	roomId={roomStore.id}
	previousUsersIdsList={membersIds}
/>
<LeaveRoom bind:open={actionLeaveRoomOpen} roomId={roomStore.id} {isDirect} />
<RoomDetails bind:actionRoomDetailsOpen {roomStore} avatarUrl={avatar} />
