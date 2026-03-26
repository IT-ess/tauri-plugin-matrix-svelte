<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { m } from '$lib/paraglide/messages';
	import { gotoRoomInfo } from '$lib/utils.svelte';
	import { ChevronLeft, Search } from '@lucide/svelte';
	import type { RoomStore } from 'tauri-plugin-matrix-svelte-api';
	import * as Item from '$lib/components/ui/item/index.js';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import Plus from '@lucide/svelte/icons/plus';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';

	let { avatar, roomStore }: { avatar: string | null; roomStore: RoomStore } = $props();

	let searchQuery = $state('');

	let members = $derived(Object.entries(roomStore.state.members));

	let filteredMembers = $derived(
		searchQuery.trim() == ''
			? members
			: members.filter(
					([id, info]) =>
						id.toLowerCase().includes(searchQuery.toLowerCase()) ||
						info.name.toLowerCase().includes(searchQuery.toLowerCase())
				)
	);
</script>

<div class="bg-background flex h-full w-full flex-col overflow-x-hidden">
	<header class="pt-safe bg-background sticky top-0 right-0 left-0 z-50 flex w-full flex-col pl-2">
		<div class="flex items-center">
			<button
				onclick={() => gotoRoomInfo(roomStore.id, avatar)}
				class="hover:bg-accent flex size-10 items-center justify-center rounded-full transition-colors"
				aria-label="Go back"
			>
				<ChevronLeft class="text-foreground size-6" />
			</button>
			<h1 class="pl-2 text-lg font-semibold">People</h1>
		</div>

		<div class="relative mx-4 mt-2 p-1">
			<Input
				class="h-12 rounded-2xl pl-4"
				type="text"
				placeholder={m.profiles_search_bar_placeholder()}
				bind:value={searchQuery}
			/>
			<Search
				class="text-muted-foreground absolute top-1/2 right-4 size-4 -translate-y-1/2 transform"
			/>
		</div>
	</header>
	<div class="pb-safe-offset-4 mt-4 flex w-full max-w-md flex-col gap-4">
		<p class="ps-3 font-medium">{filteredMembers.length} Members</p>
		<Item.Group>
			{#each filteredMembers as [id, info], index (id)}
				<Item.Root size="sm">
					<Item.Media>
						<Avatar.Root>
							{#if info.avatar}
								{@render fetchAvatar(info.avatar, info.name)}
							{/if}
							{@render avatarFallback(info.name)}
						</Avatar.Root>
					</Item.Media>
					<Item.Content class="gap-1">
						<Item.Title>{info.name}</Item.Title>
						<Item.Description>{id}</Item.Description>
					</Item.Content>
					<Item.Actions>
						{#if info.role !== 'user'}
							{m[info.role]()}
						{/if}
					</Item.Actions>
				</Item.Root>
				{#if index !== members.length - 1}
					<Item.Separator />
				{/if}
			{/each}
		</Item.Group>
	</div>
</div>
