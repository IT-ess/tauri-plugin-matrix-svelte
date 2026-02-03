<script lang="ts">
	import { Card, CardContent, CardFooter } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { toast } from 'svelte-sonner';
	import EditableField from '$lib/components/common/editable-field.svelte';
	import { LoaderIcon } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import EditableAvatar from '$lib/components/common/editable-avatar.svelte';
	import { loginStore } from '../../../hooks.client';
	import { editUserInformation } from 'tauri-plugin-matrix-svelte-api';

	let tempDisplayName = $derived(loginStore.state.userDisplayName);
	let tempDeviceName = $derived(loginStore.state.deviceDisplayName);

	const handleSaveDisplayName = async (newValue: string) => {
		tempDisplayName = newValue;
		await saveChanges();
		toast.success(m.display_name() + ' ' + m.updated());
	};

	const handleSaveDeviceName = async (newValue: string) => {
		tempDeviceName = newValue;
		await saveChanges();
		toast.success(m.device_name() + ' ' + m.updated());
	};

	const handleCancelDisplayName = () => {
		toggleEditDisplay();
	};

	const handleCancelDeviceName = () => {
		toggleEditDevice();
	};

	const setUriCallback = async (uri: string) => {
		await editUserInformation({
			newAvatarUri: uri,
			newDeviceName: null,
			newDisplayName: null
		});
	};

	const saveChanges = async () => {
		isSaving = true;
		await editUserInformation({
			newDeviceName: isEditingDeviceName ? tempDeviceName : null,
			newDisplayName: isEditingDisplayName ? tempDisplayName : null,
			newAvatarUri: null
		});
		isSaving = false;
		isEditingDisplayName = false;
		isEditingDeviceName = false;
	};

	let isSaving = $state(false);
	let isEditingDisplayName = $state(false);
	const toggleEditDisplay = () => (isEditingDisplayName = !isEditingDisplayName);
	let isEditingDeviceName = $state(false);
	const toggleEditDevice = () => (isEditingDeviceName = !isEditingDeviceName);
</script>

<Card class="shadow-lg-lg mx-auto w-full max-w-md">
	<CardContent class="space-y-6">
		<div>
			<EditableAvatar
				{setUriCallback}
				previousAvatarUri={loginStore.state.userAvatar}
				displayName={tempDisplayName ?? '?'}
				canEdit
			/>
			<p class="mt-2 text-center text-lg font-semibold">{loginStore.state.userId}</p>
		</div>

		<div class="flex flex-col gap-4">
			<EditableField
				label={m.display_name()}
				value={tempDisplayName ?? ''}
				isEditing={isEditingDisplayName}
				{isSaving}
				onEdit={toggleEditDisplay}
				onSave={handleSaveDisplayName}
				onCancel={handleCancelDisplayName}
				canEdit
			/>

			<div class="flex flex-col gap-2">
				<EditableField
					label={m.device_name()}
					value={tempDeviceName ?? ''}
					isEditing={isEditingDeviceName}
					{isSaving}
					onEdit={toggleEditDevice}
					onSave={handleSaveDeviceName}
					onCancel={handleCancelDeviceName}
					canEdit
				/>
			</div>
		</div>
	</CardContent>

	<CardFooter class="flex gap-2 border-t pt-6">
		<Button class="flex-1" disabled={isSaving || (!isEditingDisplayName && !isEditingDeviceName)}>
			{#if isSaving}
				<LoaderIcon class="mr-2 size-4 animate-spin" />
			{/if}
			{m.button_save()}
		</Button>
	</CardFooter>
</Card>
