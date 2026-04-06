<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { m } from '$lib/paraglide/messages';
	import { getCustomMxcUriFromOriginal, getInitials, gotoRoomInfo } from '$lib/utils.svelte';
	import { ChevronLeft, Search } from '@lucide/svelte';
	import type { FrontendRoomMember, RoomStore } from 'tauri-plugin-matrix-svelte-api';
	import * as Item from '$lib/components/ui/item/index.js';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import MemberActions from './member-actions.svelte';

	let { avatar, roomStore }: { avatar: string | null; roomStore: RoomStore } = $props();

	let searchQuery = $state('');

	let members = $derived(
		Object.entries(roomStore.state.members).filter((m) => m[1].membership == 'join')
	);

	let filteredMembers = $derived(
		searchQuery.trim() == ''
			? members
			: members.filter(
					([id, info]) =>
						id.toLowerCase().includes(searchQuery.toLowerCase()) ||
						info.name.toLowerCase().includes(searchQuery.toLowerCase())
				)
	);

	type ClickedUser = {
		id: string;
		name: string;
		avatar: string | null;
		role: FrontendRoomMember['role'];
	};
	let openMemberActions = $state(false);
	let clickedUser: ClickedUser | undefined = $state();

	const handleClickOnItem = (user: ClickedUser) => {
		clickedUser = user;
		openMemberActions = true;
	};
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
			<h1 class="pl-2 text-lg font-semibold">{m.members()}</h1>
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
		<p class="ps-3 font-medium">{filteredMembers.length} {m.members()}</p>
		<Item.Group>
			{#each filteredMembers as [id, info], index (id)}
				<Item.Root
					onclick={() =>
						handleClickOnItem({ id, avatar: info.avatar, name: info.name, role: info.role })}
					size="sm"
				>
					<Item.Media>
						<Avatar.Root>
							<Avatar.Image src={getCustomMxcUriFromOriginal(info.avatar)} alt={info.name} />
							<Avatar.Fallback>{getInitials(info.name)}</Avatar.Fallback>
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
<MemberActions
	bind:openMemberActions
	clickedUser={clickedUser as ClickedUser}
	roomId={roomStore.id}
	userPower={roomStore.state.tlState?.userPower}
/>
