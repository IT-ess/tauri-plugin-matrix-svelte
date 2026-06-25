<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { getCustomMxcUriFromOriginal, getInitials, gotoRoom } from '$lib/utils.svelte';
	import { Search } from '@lucide/svelte';
	import {
		searchBookmarksGlobally,
		searchBookmarksInRoom,
		type MatrixBookmarkItem
	} from 'tauri-plugin-matrix-svelte-api';
	import * as Item from '$lib/components/ui/item/index.js';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import * as Empty from '$lib/components/ui/empty/index.js';
	import { Spinner } from '$lib/components/ui/spinner';
	import TextMessage from '../room/items/text-message.svelte';
	import ImageMessage from '../room/items/image-message.svelte';
	import AudioMessage from '../room/items/audio-message.svelte';
	import FileMessage from '../room/items/file-message.svelte';
	import VideoMessage from '../room/items/video-message.svelte';

	// When there is no roomId passed, we consider we should use global bookmarks
	// search instead.
	let { searchedRoomId }: { searchedRoomId?: string } = $props();

	const PAGE_SIZE = 30;

	let searchQuery = $state('');
	let results = $state<MatrixBookmarkItem[]>([]);
	let loading = $state(false);
	let offset = $state(0);
	let hasMore = $state(false);

	let debounceTimer: ReturnType<typeof setTimeout> | undefined;

	const load = async (reset: boolean) => {
		loading = true;
		try {
			const currentOffset = reset ? 0 : offset;
			let page = [];
			if (searchedRoomId) {
				page = await searchBookmarksInRoom(
					searchQuery.trim(),
					PAGE_SIZE,
					currentOffset,
					searchedRoomId
				);
			} else {
				page = await searchBookmarksGlobally(searchQuery.trim(), PAGE_SIZE, currentOffset);
			}
			results = reset ? page : [...results, ...page];
			offset = currentOffset + page.length;
			hasMore = page.length === PAGE_SIZE;
		} catch (err) {
			console.error(err);
		} finally {
			loading = false;
		}
	};

	// Debounced reload whenever the query changes.
	$effect(() => {
		searchQuery;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => load(true), 250);
		return () => clearTimeout(debounceTimer);
	});

	const formatTime = (timestamp: number) =>
		new Date(timestamp).toLocaleDateString(getLocale(), {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
</script>

<div class="flex w-full flex-col">
	<div class="relative mx-4 mt-2 p-1">
		<Input
			class="h-12 rounded-2xl pl-4"
			type="text"
			placeholder={m.bookmarks_search_placeholder()}
			bind:value={searchQuery}
		/>
		<Search
			class="text-muted-foreground absolute top-1/2 right-4 size-4 -translate-y-1/2 transform"
		/>
	</div>

	{#if loading && results.length === 0}
		<div class="flex justify-center py-12">
			<Spinner />
		</div>
	{:else if results.length === 0}
		<Empty.Root class="mx-4 mt-4 border-0">
			<Empty.Title>{m.bookmarks_empty()}</Empty.Title>
		</Empty.Root>
	{:else}
		<div class="pb-safe-offset-4 mt-4 flex w-full flex-col gap-4">
			<Item.Group>
				{#each results as bookmark, index (bookmark.item.uniqueId)}
					<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
					{@const { item, roomAvatar, roomId, roomName, senderAvatar } = bookmark}
					{@const sender = item.data.sender}
					{@const data = item.data}
					<Item.Root onclick={() => gotoRoom(roomId, roomAvatar, item.eventId as string)} size="sm">
						<Item.Media>
							<Avatar.Root>
								<Avatar.Image src={getCustomMxcUriFromOriginal(senderAvatar)} alt={sender} />
								<Avatar.Fallback>{getInitials(sender ?? '?')}</Avatar.Fallback>
							</Avatar.Root>
						</Item.Media>
						<Item.Content class="gap-1">
							<Item.Title>{sender}</Item.Title>
							<Item.Description class="line-clamp-2">
								{#if data.kind == 'text'}
									<TextMessage textMessage={data.body} />
								{:else if data.kind === 'emote'}
									<p class="mt-1 text-sm">
										<b>{data.sender}:</b>{data.body.body}
										<!-- same as a text message, but with sender name in front -->
									</p>
								{:else if data.kind === 'image'}
									<ImageMessage
										itemContent={data.body}
										isSticker={false}
										handleOpenMediaViewMode={() => {}}
									/>
									{#if data.body.body}
										<TextMessage
											textMessage={{
												body: data.body.body,
												formatted_body: data.body.formatted_body,
												format: data.body.format,
												matched_urls: null
											}}
										/>
									{/if}
								{:else if data.kind == 'audio'}
									<AudioMessage itemContent={data.body} isOwn={false} />
								{:else if data.kind === 'video'}
									<VideoMessage itemContent={data.body} handleOpenMediaViewMode={() => {}} />
									{#if data.body.body}
										<TextMessage
											textMessage={{
												body: data.body.body,
												formatted_body: data.body.formatted_body,
												format: data.body.format,
												matched_urls: null
											}}
										/>
									{/if}
								{:else if data.kind === 'file'}
									<FileMessage itemContent={data.body} />
									{#if data.body.body}
										<TextMessage
											textMessage={{
												body: data.body.body,
												formatted_body: data.body.formatted_body,
												format: data.body.format,
												matched_urls: null
											}}
										/>
									{/if}
								{:else if data.kind === 'notice'}
									<TextMessage textMessage={data.body} />
								{:else if data.kind === 'serverNotice'}
									<TextMessage
										textMessage={{
											body: data.body.body,
											matched_urls: null
										}}
									/>
								{/if}
							</Item.Description>
							{#if !searchedRoomId}
								<Item.Description class="text-muted-foreground/70">
									{roomName}
								</Item.Description>
							{/if}
						</Item.Content>
						<Item.Actions>
							<span class="text-muted-foreground text-xs">
								{formatTime(bookmark.item.timestamp ?? 0)}
							</span>
						</Item.Actions>
					</Item.Root>
					{#if index !== results.length - 1}
						<Item.Separator />
					{/if}
				{/each}
			</Item.Group>

			{#if hasMore}
				<button
					class="text-muted-foreground hover:text-foreground py-2 text-sm"
					onclick={() => load(false)}
					disabled={loading}
				>
					{#if loading}
						<Spinner class="mx-auto" />
					{:else}
						…
					{/if}
				</button>
			{/if}
		</div>
	{/if}
</div>
