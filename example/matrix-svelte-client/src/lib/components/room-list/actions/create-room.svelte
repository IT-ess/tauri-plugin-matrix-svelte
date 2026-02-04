<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { InfoIcon } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { Spinner } from '$lib/components/ui/spinner';
	import EditableAvatar from '$lib/components/common/editable-avatar.svelte';
	import RoomSelector from '$lib/components/room-selector/room-selector.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { gotoRoom } from '$lib/utils.svelte';
	import { roomsCollection } from '../../../../hooks.client';
	import { createMatrixRequest, submitAsyncRequest, MatrixSvelteListenEvent } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		actionCreateRoomOpen: boolean;
	};
	let { actionCreateRoomOpen = $bindable(false) }: Props = $props();

	let groupName = $state('');
	let groupTopic = $state('');
	let isLoading = $state(false);
	let selectedRoomsIds = $state<string[]>([]);

	let definedAvatarUri = $state<string | undefined>();

	const setUriCallback = async (uri: string) => {
		definedAvatarUri = uri;
	};

	const handleCreateGroup = async () => {
		if (!groupName.trim()) {
			toast.error('Please enter a group name');
			return;
		}

		isLoading = true;

		try {
			const invitedUserIds = selectedRoomsIds
				.map((id) => roomsCollection.state.allJoinedRooms[id].directUserId)
				.filter((userId) => userId !== null);
			let request = createMatrixRequest.createRoom({
				roomName: groupName,
				topic: groupTopic || null,
				roomAvatar: definedAvatarUri ?? null,
				invitedUserIds
			});

			await submitAsyncRequest(request);
		} catch (error) {
			console.error(error);
		} finally {
			groupName = '';
			groupTopic = '';
		}
	};

	let newRoomUnlistener: UnlistenFn;
	onMount(async () => {
		newRoomUnlistener = await listen<string>(
			MatrixSvelteListenEvent.NewlyCreatedRoomId,
			(event) => {
				isLoading = false;
				actionCreateRoomOpen = false;
				// Goto the room 250 ms after so the room store can be created
				setTimeout(async () => {
					await gotoRoom(event.payload, definedAvatarUri ?? null);
				}, 250);
			}
		);
	});

	onDestroy(() => {
		newRoomUnlistener();
	});
</script>

<Dialog.Root bind:open={actionCreateRoomOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.create_room()}</Dialog.Title>
			<Dialog.Description>{m.create_room_subtitle()}</Dialog.Description>
		</Dialog.Header>
		<Tabs value="basic" class="w-full">
			<TabsList class="grid w-full grid-cols-2">
				<TabsTrigger value="basic">{m.create_room_tab_info()}</TabsTrigger>
				<TabsTrigger value="members">{m.create_room_tab_members()}</TabsTrigger>
			</TabsList>

			<TabsContent value="basic" class="space-y-4">
				<div class="flex justify-center pt-2">
					<EditableAvatar
						{setUriCallback}
						previousAvatarUri={null}
						displayName={groupName}
						hideHelpText
						canEdit
					/>
				</div>

				<div class="flex flex-col gap-2">
					<Label for="group-name" class="text-base font-semibold"
						>{m.create_room_name_label()} *</Label
					>
					<div class="relative">
						<Input
							id="group-name"
							type="text"
							placeholder={m.create_room_name_placeholder()}
							bind:value={groupName}
							maxlength={50}
							disabled={isLoading}
							class="text-base"
						/>
						<p class="text-muted-foreground absolute top-1/2 right-1 text-xs">
							{groupName.length}/50
						</p>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<div class="flex items-center gap-2">
						<Label for="group-topic" class="text-base font-semibold">{m.topic()}</Label>
						<span class="text-muted-foreground text-xs">{m.optional()}</span>
					</div>
					<div class="relative">
						<Input
							id="group-topic"
							type="text"
							placeholder={m.create_room_topic_placeholder()}
							bind:value={groupTopic}
							maxlength={100}
							disabled={isLoading}
							class="text-base"
						/>
						<p class="text-muted-foreground absolute top-1/2 right-1 text-xs">
							{groupTopic.length}/100
						</p>
					</div>
				</div>

				<div class="flex items-center gap-2 rounded-lg bg-blue-50 p-3 dark:bg-blue-950/30">
					<InfoIcon class="mt-0.5 size-4 shrink-0 text-blue-600 dark:text-blue-400" />
					<p class="text-xs text-blue-700 dark:text-blue-300">
						{m.create_room_add_members_later()}
					</p>
				</div>
			</TabsContent>

			<TabsContent value="members" class="flex flex-col gap-4">
				<p class="text-muted-foreground text-sm">
					{m.create_room_select_members()}
				</p>
				<RoomSelector bind:selectedRoomsIds hideGroups height={72} />
			</TabsContent>
		</Tabs>
		<Button onclick={handleCreateGroup} disabled={isLoading || !groupName.trim()}>
			{#if isLoading}
				<Spinner />
			{/if}
			{m.create_room()}
		</Button>
		<Dialog.Close disabled={isLoading} class={buttonVariants({ variant: 'outline' })}
			>{m.button_cancel()}</Dialog.Close
		>
	</Dialog.Content>
</Dialog.Root>
