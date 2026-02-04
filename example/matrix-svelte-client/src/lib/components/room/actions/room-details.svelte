<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { toast } from 'svelte-sonner';
	import EditableAvatar from '$lib/components/common/editable-avatar.svelte';
	import EditableField from '$lib/components/common/editable-field.svelte';
	import ProfileItem from '$lib/components/profiles/profile-item.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { UserPlus } from '@lucide/svelte';
	import { profileStore, roomsCollection } from '../../../../hooks.client';
	import {
		defineRoomInformations,
		type ProfileModel,
		type RoomStore
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		actionRoomDetailsOpen: boolean;
		roomStore: RoomStore;
		switchToAddUser: () => void;
		avatarUrl: string | null;
	};
	let {
		actionRoomDetailsOpen = $bindable(false),
		roomStore,
		switchToAddUser,
		avatarUrl
	}: Props = $props();

	let groupName = $derived(roomStore.state.roomName);
	let groupTopic = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].topic);
	let isSaving = $state(false);

	let isEditingName = $state(false);
	const toggleEditName = () => (isEditingName = !isEditingName);
	const onSaveDisplayName = async (newName: string) => {
		if (!newName.trim()) {
			toast.error('Please enter a group name');
			return;
		}
		isSaving = true;
		await defineRoomInformations({
			roomId: roomStore.id,
			topic: null,
			newAvatarUri: null,
			newDisplayName: newName
		});
		isSaving = false;
		toggleEditName();
	};
	const handleCancelDisplayName = () => {
		toggleEditName();
	};

	let isEditingTopic = $state(false);
	const toggleEditTopic = () => (isEditingTopic = !isEditingTopic);
	const onSaveTopic = async (newTopic: string) => {
		isSaving = true;
		await defineRoomInformations({
			roomId: roomStore.id,
			topic: newTopic,
			newAvatarUri: null,
			newDisplayName: null
		});
		isSaving = false;
		toggleEditTopic();
	};
	const handleCancelTopic = () => {
		toggleEditTopic();
	};

	const setUriCallback = async (uri: string) => {
		isSaving = true;
		await defineRoomInformations({
			roomId: roomStore.id,
			topic: null,
			newAvatarUri: uri,
			newDisplayName: null
		});
		isSaving = false;
	};

	let roomMembers = $derived(Object.entries(roomStore.state.members));

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const onSelectProfile = async (profile: ProfileModel) => {
		// Trying to go to the room doesn't refresh the room header.
		// Plus, a better action would be to display the user's profile (the feature doesn't exist right now)
		// let roomId = await getDMRoomFromUserId(profile.userId);
		// if (roomId) {
		// 	await gotoRoom(roomId, profile.avatarUrl);
		// } else {
		// 	console.error("Selected user doesn't have DM room yet");
		// }
	};

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomStore.id].isDirect);
</script>

<Dialog.Root bind:open={actionRoomDetailsOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.room_action_info_title()}</Dialog.Title>
			<Dialog.Description>{m.room_action_info_subtitle()}</Dialog.Description>
		</Dialog.Header>
		<Tabs value="basic" class="w-full">
			<TabsList class="grid w-full grid-cols-2">
				<TabsTrigger value="basic">{m.create_room_tab_info()}</TabsTrigger>
				<TabsTrigger value="members">{m.create_room_tab_members()}</TabsTrigger>
			</TabsList>

			<TabsContent value="basic" class="min-h-96 space-y-4">
				<div class="flex justify-center pt-2">
					<EditableAvatar
						{setUriCallback}
						previousAvatarUri={avatarUrl}
						displayName={groupName}
						hideHelpText
						canEdit={(roomStore.state.tlState?.userPower.includes('roomAvatar') ?? false) &&
							!isDirect}
					/>
				</div>

				<div class="flex flex-col gap-4 pb-8">
					<EditableField
						label={m.create_room_name_label()}
						value={groupName}
						isEditing={isEditingName}
						{isSaving}
						onEdit={toggleEditName}
						onSave={onSaveDisplayName}
						onCancel={handleCancelDisplayName}
						canEdit={(roomStore.state.tlState?.userPower.includes('roomName') ?? false) &&
							!isDirect}
					/>

					<div class="flex flex-col gap-2">
						<EditableField
							label={m.topic()}
							value={groupTopic ?? ''}
							isEditing={isEditingTopic}
							{isSaving}
							onEdit={() => toggleEditTopic()}
							onSave={onSaveTopic}
							onCancel={handleCancelTopic}
							canEdit={roomStore.state.tlState?.userPower.includes('roomTopic') ?? false}
						/>
					</div>
				</div>
			</TabsContent>

			<TabsContent value="members" class="flex min-h-96 flex-col">
				{#if roomStore.state.tlState?.userPower.includes('invite') && !isDirect}
					<Button onclick={switchToAddUser}
						><UserPlus class="text-primary-foreground group-hover:text-primary" />
						{m.room_action_info_add_contact_to_group()}</Button
					>
				{/if}
				<ScrollArea class="h-84">
					<div class="space-y-2 p-2">
						{#each roomMembers as member (member[0])}
							{@const userId = member[0]}
							{@const profileStoreEntry =
								profileStore.state[userId].state === 'loaded'
									? profileStore.state[userId].data
									: null}
							{@const profile: ProfileModel = {userId, displayName: profileStoreEntry ? profileStoreEntry.username : null, avatarUrl: profileStoreEntry ? profileStoreEntry.avatarUrl ?? null : null }}
							<ProfileItem {profile} onSelect={onSelectProfile} />
						{:else}
							<p class="text-muted-foreground">{m.contact_selection_no_results()}</p>
							<p class="text-sm text-muted-foreground mt-1">
								{m.contact_selection_no_results_help_text()}
							</p>
						{/each}
					</div>
				</ScrollArea>
			</TabsContent>
		</Tabs>
		<Dialog.Close disabled={isSaving} class={buttonVariants({ variant: 'outline' })}
			>{m.button_close()}</Dialog.Close
		>
	</Dialog.Content>
</Dialog.Root>
