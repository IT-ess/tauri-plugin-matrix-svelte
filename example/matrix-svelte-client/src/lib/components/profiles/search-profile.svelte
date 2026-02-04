<script lang="ts">
	import { LoaderCircle, Search } from '@lucide/svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Input } from '$lib/components/ui/input';
	import ProfileItem from './profile-item.svelte';
	import { m } from '$lib/paraglide/messages';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import {
		createMatrixRequest,
		searchUsers,
		submitAsyncRequest,
		type ProfileModel
	} from 'tauri-plugin-matrix-svelte-api';

	let { parentOpen = $bindable(true) }: { parentOpen: boolean } = $props();

	let searchQuery = $state('');
	let openAlertDialog = $state(false);
	let selectedProfile = $state<ProfileModel | null>(null);

	// Filter contacts based on search query
	const matchingProfiles = $derived.by(async () => {
		if (!searchQuery.trim()) return [];
		return await searchUsers(searchQuery, 10);
	});

	const matrixIdRegExp = new RegExp(/^@[a-z0-9._=\-/]+:[a-z0-9.-]+\.[a-z]{2,}$/i);
	let isMatrixId = $derived.by(() => matrixIdRegExp.test(searchQuery));

	const onSelect = (profile: ProfileModel) => {
		selectedProfile = profile;
		openAlertDialog = true;
	};

	const handleInviteUser = () => {
		if (selectedProfile === null) {
			return;
		}
		const request = createMatrixRequest.createDMRoom({ userId: selectedProfile.userId });
		submitAsyncRequest(request);
		selectedProfile = null;
		openAlertDialog = false;
		parentOpen = false;
	};
</script>

<div class="mx-auto flex h-full max-w-md flex-col">
	<div class="bg-background/95 supports-backdrop-filter:bg-background/60 border-b backdrop-blur">
		<div class="relative p-1">
			<Search
				class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform"
			/>
			<Input
				type="text"
				placeholder={m.profiles_search_bar_placeholder()}
				bind:value={searchQuery}
				class="pl-10"
			/>
		</div>
		<p class="text-muted-foreground my-2 text-sm">
			{#await matchingProfiles}
				0 contact
			{:then profiles}
				{#if profiles !== undefined}
					{profiles.length} contact{profiles.length !== 1 ? 's' : ''}
				{:else}
					0 contact
				{/if}
			{/await}
		</p>
	</div>

	<!-- Scrollable Contact List -->
	<ScrollArea class="h-full w-full rounded-md">
		<div class="space-y-2 py-2">
			{#await matchingProfiles}
				<LoaderCircle class="animate-spin" />
			{:then profiles}
				{#if profiles !== undefined}
					{#each profiles as profile (profile.userId)}
						<ProfileItem {profile} {onSelect} />
					{:else}
						{#if isMatrixId}
							<ProfileItem
								profile={{ userId: searchQuery, avatarUrl: null, displayName: null }}
								{onSelect}
							/>
						{:else}
							{@render noContactFound()}
						{/if}
					{/each}
				{:else}
					{@render noContactFound()}
				{/if}
			{/await}
		</div>
	</ScrollArea>
</div>

{#snippet noContactFound()}
	<div class="py-8 text-center">
		<p class="text-muted-foreground">{m.contact_selection_no_results()}</p>
		<p class="text-muted-foreground mt-1 text-sm">{m.contact_selection_no_results_help_text()}</p>
	</div>
{/snippet}

<AlertDialog.Root bind:open={openAlertDialog}>
	<AlertDialog.Portal>
		<AlertDialog.Overlay />
		<AlertDialog.Content>
			<AlertDialog.Title>{m.invite()}</AlertDialog.Title>
			<AlertDialog.Description
				>{m.invitation_send_confirmation({
					user: selectedProfile?.displayName ?? '?'
				})}</AlertDialog.Description
			>
			<AlertDialog.Action onclick={() => handleInviteUser()} type="submit"
				>{m.invite()}</AlertDialog.Action
			>
			<AlertDialog.Cancel type="button">{m.button_cancel()}</AlertDialog.Cancel>
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>
