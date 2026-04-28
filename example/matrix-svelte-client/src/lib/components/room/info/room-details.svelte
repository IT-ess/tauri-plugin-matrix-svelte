<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { buttonVariants } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { toast } from 'svelte-sonner';
	import EditableAvatar from '$lib/components/common/editable-avatar.svelte';
	import EditableField from '$lib/components/common/editable-field.svelte';
	import { defineRoomInformations, type RoomStore } from 'tauri-plugin-matrix-svelte-api';
	import { roomsCollection } from '../../../../hooks.client';

	type Props = {
		roomId: string;
		actionRoomDetailsOpen: boolean;
		roomStore: RoomStore;
		avatarUrl: string | null;
	};
	let { roomId, actionRoomDetailsOpen = $bindable(false), roomStore, avatarUrl }: Props = $props();

	let groupName = $derived(roomStore.state.roomName);
	let groupTopic = $derived(roomsCollection.state.allJoinedRooms[roomId].topic);
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
			roomId,
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
			roomId,
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
		try {
			await defineRoomInformations({
				roomId,
				topic: null,
				newAvatarUri: uri,
				newDisplayName: null
			});
		} catch (err) {
			console.error(err);
			toast.error(`Error ${err}`);
		} finally {
			isSaving = false;
		}
	};

	let isDirect = $derived(roomsCollection.state.allJoinedRooms[roomId].isDirect);
</script>

<Dialog.Root bind:open={actionRoomDetailsOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.room_action_info_title()}</Dialog.Title>
			<Dialog.Description>{m.room_action_info_subtitle()}</Dialog.Description>
		</Dialog.Header>
		<div class="flex justify-center pt-2">
			<EditableAvatar
				{setUriCallback}
				previousAvatarUri={avatarUrl}
				displayName={groupName}
				hideHelpText
				canEdit={(roomStore.state.tlState?.userPower.includes('roomAvatar') ?? false) && !isDirect}
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
				canEdit={(roomStore.state.tlState?.userPower.includes('roomName') ?? false) && !isDirect}
			/>

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
		<Dialog.Close disabled={isSaving} class={buttonVariants({ variant: 'outline' })}
			>{m.button_close()}</Dialog.Close
		>
	</Dialog.Content>
</Dialog.Root>
