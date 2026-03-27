<script lang="ts">
	import { ChevronLeft, MessageCircle } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { m } from '$lib/paraglide/messages';
	import {
		getDmRoomIdOrCreateIt,
		MatrixSvelteListenEvent,
		type ProfileModel
	} from 'tauri-plugin-matrix-svelte-api';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { gotoRoom } from '$lib/utils.svelte';
	import { onDestroy } from 'svelte';
	import { Spinner } from '../ui/spinner';

	let { profile }: { profile: ProfileModel } = $props();
	let { avatarUrl, displayName, userId } = $derived(profile);

	let unlistenCreateRoom: UnlistenFn;
	let isCreatingRoom = $state(false);
	const handleGotoMessage = async () => {
		unlistenCreateRoom = await listen<string>(MatrixSvelteListenEvent.NewlyCreatedRoomId, (id) => {
			gotoRoom(id.payload, null);
		});
		const maybeRoomId = await getDmRoomIdOrCreateIt(userId);

		if (maybeRoomId) {
			gotoRoom(maybeRoomId, null);
		} else {
			isCreatingRoom = true;
		}
	};

	onDestroy(() => {
		if (unlistenCreateRoom) {
			unlistenCreateRoom();
		}
	});
</script>

<div class="bg-background flex h-full w-full flex-col">
	<div class="pt-safe sticky top-0 right-0 left-0 z-50 w-full pl-2">
		<button
			onclick={() => (window.navigation ? window.navigation.back() : window.history.back())}
			class="hover:bg-accent flex size-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>
	</div>
	<div class="flex min-h-3/8 flex-col justify-between">
		<div class="flex flex-col items-center gap-4">
			<Avatar.Root class="size-16">
				{#if avatarUrl}
					{@render fetchAvatar(avatarUrl, displayName)}
				{/if}
				{@render avatarFallback(displayName)}
			</Avatar.Root>
			<h1 class="text-2xl font-bold">
				{displayName}
			</h1>
			<h2 class="text-muted-foreground text-xl">
				{userId}
			</h2>
			<div class="my-4 flex gap-4">
				<Button
					onclick={handleGotoMessage}
					class="flex flex-col gap-2 text-base"
					size="icon-lg"
					variant="ghost"
				>
					{#if isCreatingRoom}
						<Spinner class="size-7" />
					{:else}
						<MessageCircle class="size-7 pl-px" />
					{/if}
					{m.message()}
				</Button>
			</div>
		</div>
	</div>
	<Separator class="mt-4" />
</div>
