<script lang="ts">
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuLabel,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { BookUser, EllipsisVertical, LogOut, UserPlus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import LeaveRoom from './leave-room.svelte';
	import InviteMembers from './invite-members.svelte';
	import RoomDetails from './room-details.svelte';
	import { roomsCollection } from '../../../../hooks.client';
	import type { RoomStore } from 'tauri-plugin-matrix-svelte-api';

	let {
		roomStore,
		actionRoomDetailsOpen = $bindable(false),
		avatarUrl
	}: {
		roomStore: RoomStore;
		actionRoomDetailsOpen: boolean;
		avatarUrl: string | null;
	} = $props();

	let isActionButtonOpen = $state(false);

	let actionLeaveRoomOpen = $state(false);
	let actionInviteMembersOpen = $state(false);

	let membersIds = $derived(Object.keys(roomStore.state.members));

	const switchToAddUser = () => {
		actionRoomDetailsOpen = false;
		actionInviteMembersOpen = true;
	};

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].isDirect);
</script>

<div class="absolute right-6 z-50">
	<DropdownMenu bind:open={isActionButtonOpen}>
		<DropdownMenuTrigger>
			<EllipsisVertical class="text-foreground h-5 w-5" />
		</DropdownMenuTrigger>
		<DropdownMenuContent align="end" side="bottom">
			<DropdownMenuLabel class="text-md">{m.room_action_menu_header()}</DropdownMenuLabel>
			<DropdownMenuItem class="text-md" onSelect={() => (actionRoomDetailsOpen = true)}
				><BookUser />{m.room_action_menu_room_info()}</DropdownMenuItem
			>
			{#if roomStore.state.tlState?.userPower.includes('invite') && !isDirect}
				<DropdownMenuItem class="text-md" onSelect={() => (actionInviteMembersOpen = true)}
					><UserPlus />{m.room_action_menu_invite_members()}</DropdownMenuItem
				>
			{/if}
			{#if !isDirect}
				<DropdownMenuItem
					variant="destructive"
					class="text-md"
					onSelect={() => (actionLeaveRoomOpen = true)}
					><LogOut />{m.room_action_menu_leave_room()}</DropdownMenuItem
				>
			{/if}
		</DropdownMenuContent>
	</DropdownMenu>
</div>
<LeaveRoom bind:open={actionLeaveRoomOpen} roomId={roomStore.id} {isDirect} />
<InviteMembers
	bind:actionInviteMembersOpen
	roomId={roomStore.id}
	previousUsersIdsList={membersIds}
/>
<RoomDetails {roomStore} bind:actionRoomDetailsOpen {switchToAddUser} {avatarUrl} />
