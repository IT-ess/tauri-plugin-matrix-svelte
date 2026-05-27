<script lang="ts">
	import { ChevronLeft, Globe, History, User, Users } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import { m } from '$lib/paraglide/messages';
	import {
		createMatrixRequest,
		MatrixSvelteListenEvent,
		submitAsyncRequest
	} from 'tauri-plugin-matrix-svelte-api';
	import {
		getCustomMxcUriFromOriginal,
		getInitials,
		goBack,
		gotoRoom,
		gotoRoomsList
	} from '$lib/utils.svelte';
	import { Spinner } from '$lib/components/ui/spinner';
	import type { PageProps } from './$types';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import { toast } from 'svelte-sonner';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy } from 'svelte';

	let { data }: PageProps = $props();

	let isLoading = $state(false);

	let newRoomUnlistener: UnlistenFn;

	const setupRoomListener = async (avatarUrl: string | null) => {
		newRoomUnlistener = await listen<string>(
			MatrixSvelteListenEvent.NewlyCreatedRoomId,
			(event) => {
				isLoading = false;
				gotoRoom(event.payload, avatarUrl);
			}
		);
	};

	const joinRoom = async (roomId: string, avatarUrl: string | null) => {
		isLoading = true;
		await setupRoomListener(avatarUrl);
		const request = createMatrixRequest.joinRoom({
			roomOrAliasId: roomId,
			via: null
		});
		await submitAsyncRequest(request);
	};

	const joinRoomWithAlias = async (
		aliasId: string,
		avatarUrl: string | null,
		serverNames: string[] | null
	) => {
		isLoading = true;
		await setupRoomListener(avatarUrl);
		const request = createMatrixRequest.joinRoom({
			roomOrAliasId: aliasId,
			via: serverNames
		});
		await submitAsyncRequest(request);
	};

	const knockRoom = async (
		roomOrAliasId: string,
		serverNames: string[],
		roomName: string | null
	) => {
		isLoading = true;
		const request = createMatrixRequest.knock({
			roomOrAliasId,
			reason: null,
			serverNames
		});
		await submitAsyncRequest(request);
		await gotoRoomsList('dm');
		toast.success(`Knocked room ${roomName}`);
	};

	onDestroy(() => {
		if (newRoomUnlistener) {
			newRoomUnlistener();
		}
	});
</script>

<div class="bg-background flex h-full w-full flex-col">
	<div class="pt-safe sticky top-0 right-0 left-0 z-50 w-full pl-2">
		<button
			onclick={goBack}
			class="hover:bg-accent flex size-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>
	</div>
	{#await data.preview}
		<div class="flex h-full w-full items-center justify-center">
			<Spinner class="size-16" />
		</div>
	{:then [{ avatar_url, name, canonical_alias, is_world_readable, is_direct, topic, join_rule, num_joined_members, state, room_id }, via]}
		<div class="flex h-full flex-col justify-between">
			<div class="flex flex-col items-center gap-4">
				<Avatar.Root class="size-16">
					<Avatar.Image src={getCustomMxcUriFromOriginal(avatar_url)} alt={name} />
					<Avatar.Fallback>{getInitials(name ?? '?')}</Avatar.Fallback>
				</Avatar.Root>
				<h1 class="text-2xl font-bold">
					{name}
				</h1>
				{#if canonical_alias}
					<h2 class="text-muted-foreground text-xl">
						{canonical_alias}
					</h2>
				{/if}
				<div class="flex gap-1">
					{#if is_direct}
						<Badge><User />{m.direct_message()}</Badge>
					{:else}
						<Badge><Users />{num_joined_members}</Badge>
					{/if}
					{#if join_rule && join_rule.join_rule == 'public'}
						<Badge><Globe />{m.public_room()}</Badge>
					{/if}
					{#if is_world_readable}
						<Badge><History />{m.world_readable_room()}</Badge>
					{/if}
				</div>
				{#if topic}
					<p class="mx-4 text-center">{topic}</p>
				{/if}
			</div>
			<div class="flex flex-col gap-2 pb-safe-offset-4 px-safe-offset-6">
				<Separator class="mt-4" />
				{#if state == 'Joined'}
					{@render summaryText(m.room_preview_already_joined())}
					{@render actionButton(m.button_goto_room(), () => gotoRoom(room_id, avatar_url))}
				{:else if state == 'Banned'}
					{@render summaryText(m.room_preview_banned())}
					{@render actionButton(m.room_preview_banned_action())}
				{:else if state == 'Invited'}
					{@render summaryText(m.room_preview_invited())}
					{@render actionButton(m.button_join_room(), () => joinRoom(room_id, avatar_url))}
				{:else if state == 'Knocked'}
					{@render summaryText(m.room_preview_already_knocked())}
					{@render actionButton(m.room_preview_knock_again(), () => knockRoom(room_id, via, name))}
				{:else if state == 'Left'}
					{@render summaryText(m.room_preview_previously_left())}
					{#if !join_rule}
						{@render actionButton(m.room_preview_not_allowed())}
					{:else if join_rule.join_rule == 'public'}
						{@render actionButton(
							m.room_preview_rejoin_room(),
							canonical_alias
								? () => joinRoomWithAlias(canonical_alias, avatar_url, via)
								: () => joinRoom(room_id, avatar_url)
						)}
					{:else if join_rule.join_rule == 'invite'}
						{@render actionButton(m.room_preview_rejoin_requires_invite())}
					{:else if join_rule.join_rule == 'knock' || join_rule.join_rule == 'knock_restricted'}
						{@render actionButton(m.room_preview_rejoin_knock(), () =>
							knockRoom(room_id, via, name)
						)}
					{:else}
						{@render actionButton(m.room_preview_not_allowed())}
					{/if}
				{:else if join_rule}
					<!-- The room is unknown to the user -->
					{#if join_rule.join_rule == 'public'}
						{@render actionButton(
							m.button_join_room(),
							canonical_alias
								? () => joinRoomWithAlias(canonical_alias, avatar_url, via)
								: () => joinRoom(room_id, avatar_url)
						)}
					{:else if join_rule.join_rule == 'invite'}
						{@render actionButton(m.room_preview_join_requires_invite())}
					{:else if join_rule.join_rule == 'knock' || join_rule.join_rule == 'knock_restricted'}
						{@render actionButton(m.room_preview_join_knock(), () => knockRoom(room_id, via, name))}
					{:else}
						{@render actionButton(m.room_preview_not_allowed())}
					{/if}
				{:else}
					{@render actionButton(m.room_preview_not_allowed())}
				{/if}
				<Button variant="outline" onclick={goBack}>{m.button_back()}</Button>
			</div>
		</div>
	{:catch err}
		<p class="text-destructive">{m.room_preview_error()} {err}</p>
	{/await}
</div>

{#snippet summaryText(text: string)}
	<p class="text-center py-8 font-medium">{text}</p>
{/snippet}

{#snippet actionButton(text: string, handler?: () => Promise<void>)}
	<Button onclick={handler} disabled={!handler || isLoading}
		>{text}
		{#if isLoading}
			<Spinner />
		{/if}</Button
	>
{/snippet}
