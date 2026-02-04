<script lang="ts">
	import { PlusIcon, SearchIcon } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import ProfileItem from './profile-item.svelte';
	import * as m from '$lib/paraglide/messages';
	import InviteProfileItem from './invite-profile-item.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import SearchProfile from './search-profile.svelte';
	import { buttonVariants } from '$lib/components/ui/button';
	import Button from '../ui/button/button.svelte';
	import { roomsCollection } from '../../../hooks.client';
	import { onMount } from 'svelte';
	import type { ProfileModel, RoomModel } from 'tauri-plugin-matrix-svelte-api';

	let {
		dmRooms,
		openInviteDrawerOnLoad
	}: {
		dmRooms: RoomModel[];
		openInviteDrawerOnLoad: boolean;
	} = $props();

	let searchQuery = $state('');

	// Todo: do something on click
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const onSelect = (user: ProfileModel) => {
		return;
	};

	// Filter contacts based on search query
	const filteredProfiles: ProfileModel[] = $derived.by(() => {
		if (!searchQuery.trim()) {
			return dmRooms.map((room) => ({
				userId: room.dmUserId as string,
				displayName: room.displayName,
				avatarUrl: room.avatarUrl
			}));
		}

		const query = searchQuery.toLowerCase();
		return dmRooms
			.filter(
				(profile) =>
					(profile.displayName && profile.displayName.toLowerCase().includes(query)) ||
					(profile.dmUserId && profile.dmUserId.toLowerCase().includes(query))
			)
			.map((room) => ({
				userId: room.dmUserId as string,
				displayName: room.displayName,
				avatarUrl: room.avatarUrl
			}));
	});

	let invitedRooms = $derived(Object.values(roomsCollection.state.invitedRooms));

	let openInviteDrawer = $state(false);

	onMount(() => {
		// we delay the opening of the drawer so it is clear that this is the intended action
		if (openInviteDrawerOnLoad) {
			setTimeout(() => (openInviteDrawer = true), 500);
		}
	});
</script>

<div class="mx-auto flex h-full w-full max-w-md flex-col">
	<div
		class="bg-background/95 supports-backdrop-filter:bg-background/60 border-b p-4 backdrop-blur"
	>
		<div class="relative m-2">
			<SearchIcon
				class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform"
			/>
			<Input
				type="text"
				placeholder={m.profiles_search_bar_placeholder()}
				bind:value={searchQuery}
				class="pl-10"
			/>
		</div>
	</div>

	{#if invitedRooms.length > 0}
		<div class="mt-2 border-b">
			<h2 class="scroll-m-20 pb-2 pl-4 text-xl font-semibold tracking-tight transition-colors">
				{m.invites()}
			</h2>
			<div class="space-y-2 p-4 pt-2">
				{#each invitedRooms as invitedRoomInfo (invitedRoomInfo.roomId)}
					<InviteProfileItem {invitedRoomInfo} />
				{/each}
			</div>
		</div>
	{/if}

	<div class="h-full w-full flex-1 overflow-y-auto">
		<p class="text-muted-foreground mt-2 ml-3 text-sm">
			{filteredProfiles.length} contact{filteredProfiles.length !== 1 ? 's' : ''}
		</p>
		<div class="space-y-2 p-4">
			<Button class="w-full" onclick={() => (openInviteDrawer = true)}
				><PlusIcon class="text-primary-foreground group-hover:text-primary size-5" />
				{m.search_profiles_add_new()}</Button
			>
			<Dialog.Root bind:open={openInviteDrawer}>
				<Dialog.Content>
					<Dialog.Header>
						<Dialog.Title>{m.search_profiles_add_new()}</Dialog.Title>
						<Dialog.Description>
							{m.search_profiles_subtitle()}
						</Dialog.Description>
					</Dialog.Header>
					<div class="mx-4 h-72 overflow-y-hidden pb-20">
						<SearchProfile bind:parentOpen={openInviteDrawer} />
					</div>
					<Dialog.Footer>
						<Dialog.Close class={buttonVariants({ variant: 'outline' })}
							>{m.button_cancel()}</Dialog.Close
						>
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>

			{#each filteredProfiles as profile (profile.userId)}
				<ProfileItem {profile} {onSelect} />
			{:else}
				<div class="text-center py-8">
					<p class="text-muted-foreground">{m.contact_selection_no_results()}</p>
					<p class="text-sm text-muted-foreground mt-1">
						{m.contact_selection_no_results_help_text()}
					</p>
				</div>
			{/each}
		</div>
	</div>
</div>
