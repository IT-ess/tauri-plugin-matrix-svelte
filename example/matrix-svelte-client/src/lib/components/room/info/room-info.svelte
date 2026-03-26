<script lang="ts">
	import EditableAvatar from '$lib/components/common/editable-avatar.svelte';
	import { gotoRoom } from '$lib/utils.svelte';
	import { ChevronLeft, User, UserPlus } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { defineRoomInformations, type RoomStore } from 'tauri-plugin-matrix-svelte-api';
	import { roomsCollection } from '../../../../hooks.client';
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import * as Item from '$lib/components/ui/item/index.js';
	import InviteMembers from './members/invite-members.svelte';

	let { avatar, roomStore }: { avatar: string | null; roomStore: RoomStore } = $props();

	let roomTopic = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].topic);

	const setUriCallback = async (uri: string) => {
		try {
			await defineRoomInformations({
				roomId: roomStore.id,
				topic: null,
				newAvatarUri: uri,
				newDisplayName: null
			});
		} catch (err) {
			console.error(err);
			toast.error(`Error ${err}`);
		}
	};
	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].isDirect);

	let actionInviteMembersOpen = $state(false);
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
			<EditableAvatar
				{setUriCallback}
				previousAvatarUri={avatar}
				displayName={roomStore.state.roomName}
				hideHelpText
				canEdit={(roomStore.state.tlState?.userPower.includes('roomAvatar') ?? false) && !isDirect}
			/>
			<h1 class="text-2xl font-bold">
				{roomStore.state.roomName}
			</h1>
			<div class="my-4 flex gap-4">
				<Button
					onclick={() => (actionInviteMembersOpen = true)}
					class="flex flex-col gap-2 text-base"
					size="icon-lg"
					variant="ghost"
				>
					<UserPlus class="size-7 pl-px" />
					Invite
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
					<Item.Title class="text-base">People</Item.Title>
				</Item.Content>
				<Item.Actions>
					{membersIds.length}
				</Item.Actions>
			</a>
		{/snippet}
	</Item.Root>
</div>

<InviteMembers
	bind:actionInviteMembersOpen
	roomId={roomStore.id}
	previousUsersIdsList={membersIds}
/>
