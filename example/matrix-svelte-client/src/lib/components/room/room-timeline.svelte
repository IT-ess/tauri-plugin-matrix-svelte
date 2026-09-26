<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Marker from '$lib/components/ui/marker';
	import { ArrowDownIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import './room.css';
	import Item from './items/item.svelte';
	import { useDebounce } from 'runed';
	import SvelteVirtualChat from '@humanspeak/svelte-virtual-chat';
	import { tick, untrack } from 'svelte';
	import { loginStore, roomsCollection, roomStore } from '../../../hooks.client';
	import RoomInput from './room-input.svelte';
	import MediaViewer from '../common/media-viewer.svelte';
	import type { MediaViewerInfo } from '../media/utils';
	import { Spinner } from '../ui/spinner';
	import {
		awaitPaginateTimeline,
		createMatrixRequest,
		sendMediaMessage,
		submitAsyncRequest,
		type TimelineItem,
		type AttachmentInfo,
		type BaseAudioInfo,
		type MediaRequestParameters
	} from 'tauri-plugin-matrix-svelte-api';
	import { toast } from 'svelte-sonner';
	import { afterNavigate } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		roomId: string;
		roomAvatarUrl: string | null;
		threadRoot: string | null;
		openingFocus: string | null;
	};
	let { roomId, roomAvatarUrl, threadRoot, openingFocus }: Props = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(roomStore.state);
	}

	let isLoadingMore = $state(false);

	// Reply state
	let replyingTo = $state<{
		eventId: string;
		senderName: string;
		content: string;
	} | null>(null);

	let chat = $state<SvelteVirtualChat<TimelineItem>>();
	let isFollowing = $state(true);
	let highlightedEventId = $state<string | null>(null);

	let items = $derived(roomStore.state.tlState?.items ?? []);
	let hasItems = $derived(items.length > 0);
	let itemsByEventId = $derived(new Map(items.map((i) => [i.eventId, i])));
	let unreadCount = $derived(roomsCollection.state.allJoinedRooms[roomId]?.numUnreadMessages ?? 0);

	// Consecutive messages from the same sender within 5 minutes are visually grouped
	const sameSender = (a?: TimelineItem, b?: TimelineItem) =>
		a?.kind === 'msgLike' &&
		b?.kind === 'msgLike' &&
		a.data.senderId === b.data.senderId &&
		Math.abs((b.timestamp ?? 0) - (a.timestamp ?? 0)) < 5 * 60 * 1000;

	// Send a read receipt when reaching the bottom, or when a message arrives while at the bottom
	$effect(() => {
		if (!isFollowing || unreadCount === 0 || !hasItems) return;
		untrack(() => {
			try {
				const request = createMatrixRequest.readReceipt({
					eventId: getLatestEventId(),
					receiptType: 'm.read',
					roomId,
					threadRootEventId: threadRoot
				});
				submitAsyncRequest(request);
			} catch (err) {
				console.error(err);
				toast.error(err as string);
			}
		});
	});

	const getLatestEventId = (): string => {
		if (roomStore.state.tlState?.items && roomStore.state.tlState.items.length > 0) {
			const timelineLength = roomStore.state.tlState.items.length;
			let newArray = Array.from(
				{ length: timelineLength },
				(value, index) => timelineLength - index - 1
			);
			for (const i of newArray) {
				const item = roomStore.state.tlState.items[i];
				if (item.kind == 'msgLike' && !item.isOwn) {
					return roomStore.state.tlState.items[i].eventId as string; // All remote msgLike events have eventIds
				}
			}
		}
		throw Error('No message like event to read in this room');
	};

	// Load more messages when scrolling up with 1 sec debounce
	// (onNeedHistory fires on every scroll event near the top)
	const loadMoreMessages = useDebounce(async () => {
		if (
			isLoadingMore ||
			roomStore.state.tlState?.fullyPaginated ||
			(roomStore.state.timelineKind?.kind == 'mainRoom' &&
				items[0]?.kind === 'virtual' &&
				items[0].data.kind === 'timelineStart')
		)
			return;

		isLoadingMore = true;
		console.log('Loading more messages !');

		try {
			const request = createMatrixRequest.paginateTimeline({
				roomId,
				threadRootEventId: threadRoot,
				numEvents: 50,
				direction: 'backwards'
			});
			await submitAsyncRequest(request);
		} finally {
			isLoadingMore = false;
		}
	}, 1000);

	// Handle reply to message
	const handleReplyTo = (eventId: string, senderName: string, content: string) => {
		replyingTo = {
			eventId,
			senderName,
			content: content.length > 100 ? content.substring(0, 100) + '...' : content
		};
	};

	const scrollToMessage = async (eventId: string) => {
		let counter = 0;
		while (!itemsByEventId.has(eventId)) {
			// Paginate at most 250 events
			if (counter > 4) {
				toast.error(m.timeline_focus_error());
				return;
			}
			counter++;
			try {
				isLoadingMore = true;
				await awaitPaginateTimeline({
					roomId,
					threadRootEventId: threadRoot,
					numEvents: 50,
					direction: 'backwards'
				});
			} catch (err) {
				console.error(err);
				toast.error(err as string);
			} finally {
				isLoadingMore = false;
			}
		}
		await tick();
		chat?.scrollToMessage(itemsByEventId.get(eventId)!.uniqueId);

		highlightedEventId = eventId;
		setTimeout(() => {
			if (highlightedEventId === eventId) highlightedEventId = null;
		}, 3000);
	};

	// Media viewer
	let showMediaViewer = $state(false);
	let mediaViewerSrc = $state<string | null>(null);
	let mediaViewerBuffer = $state<ArrayBuffer | undefined>();
	let mediaViewerSource = $state<MediaRequestParameters['source']>();
	let mediaViewerInfo = $state<MediaViewerInfo | undefined>();
	let viewerMode: 'send' | 'view' = $state('send');
	let viewedMediaType: 'image' | 'video' | 'file' = $state('image');
	const handleOpenMediaSendMode = (
		type: 'image' | 'video' | 'file',
		src: string,
		buffer: ArrayBuffer,
		info: MediaViewerInfo
	) => {
		viewedMediaType = type;
		mediaViewerSrc = src;
		mediaViewerBuffer = buffer;
		mediaViewerInfo = info;
		viewerMode = 'send';
		showMediaViewer = true;
	};

	const handleOpenMediaViewMode = (
		type: 'image' | 'video' | 'file',
		src: string,
		info: {
			filename?: string;
			body?: string;
			size: number;
		},
		mediaSource: MediaRequestParameters['source']
	) => {
		viewedMediaType = type;
		mediaViewerSrc = src;

		mediaViewerInfo = { thumbnailInfo: null, ...info };
		viewerMode = 'view';
		mediaViewerSource = mediaSource;
		showMediaViewer = true;
	};

	const handleSendMedia = async (mediaInfo: AttachmentInfo, caption: string | null) => {
		if (!mediaViewerBuffer || !mediaViewerInfo?.mimeType) {
			toast.error('No buffer available to send');
			return;
		}
		console.log('called send media');

		await sendMediaMessage({
			roomId,
			inReplyTo: replyingTo?.eventId ?? null,
			threadRoot,
			info: mediaInfo,
			caption,
			filename: mediaViewerInfo?.filename ?? 'Media',
			buffer: mediaViewerBuffer,
			mimeType: mediaViewerInfo.mimeType,
			thumbnail: mediaViewerInfo?.thumbnailInfo ? await mediaViewerInfo.thumbnailInfo : null
		});

		console.log('Sent media !');

		replyingTo = null; // Clear reply state after sending
		showMediaViewer = false;
		mediaViewerSrc = null;
		mediaViewerBuffer = undefined;
		mediaViewerInfo = undefined;
	};

	const handleSendAudioMessage = async (
		blob: Blob,
		duration: number,
		waveform: number[] | null
	) => {
		mediaViewerBuffer = await blob.arrayBuffer();
		mediaViewerInfo = {
			filename: 'audio-recording_' + new Date().toISOString() + '.' + blob.type.split('/').pop(),
			size: blob.size,
			mimeType: blob.type,
			thumbnailInfo: null
		};

		const info: BaseAudioInfo = {
			size: blob.size,
			duration: {
				secs: Math.floor(duration),
				nanos: Math.floor((duration % 1) * 1e9)
			},
			waveform: waveform?.map((val) => val / 256) ?? null
		};
		await handleSendMedia({ kind: 'voice', info }, null);
	};

	const handleCloseMediaViewer = () => {
		showMediaViewer = false;
	};

	// We use afterNavigate instead of onMount because sometimes the navigation
	// is done between rooms, thus this component is already mounted
	afterNavigate(() => {
		// The keyed timeline remounts pinned to the bottom without emitting onFollowBottomChange
		isFollowing = true;
		if (openingFocus) {
			// We wait for the timeline to be mounted
			setTimeout(() => {
				scrollToMessage(openingFocus);
			}, 100);
		}
	});
</script>

{#if roomStore.state.tlState}
	{#key `${roomId}|${threadRoot}`}
		<SvelteVirtualChat
			bind:this={chat}
			messages={items}
			getMessageId={(item) => item.uniqueId}
			onNeedHistory={() => void loadMoreMessages()}
			onFollowBottomChange={(following) => (isFollowing = following)}
			containerClass="w-full flex-1 min-h-0"
			viewportClass="bg-white px-4"
		>
			{#snippet header()}
				{#if isLoadingMore}
					<Marker.Root role="status" class="justify-center pt-2">
						<Marker.Icon><Spinner /></Marker.Icon>
					</Marker.Root>
				{/if}
			{/snippet}
			{#snippet renderMessage(item, index)}
				<Item
					{item}
					{roomId}
					currentUserId={loginStore.state.userId ?? 'shouldbedefined'}
					onReply={handleReplyTo}
					onScrollToMessage={scrollToMessage}
					repliedToMessage={item.kind === 'msgLike' && item.data.inReplyToId !== null
						? itemsByEventId.get(item.data.inReplyToId)
						: undefined}
					groupedWithPrev={sameSender(items[index - 1], item)}
					groupedWithNext={sameSender(item, items[index + 1])}
					highlighted={item.eventId !== null && item.eventId === highlightedEventId}
					{handleOpenMediaViewMode}
					roomAvatar={roomAvatarUrl}
					roomMembers={roomStore.state.members}
					threadRootEventId={threadRoot}
				/>
			{/snippet}
			{#snippet footer()}
				<div id="bottomscroll" class="h-2"></div>
			{/snippet}
		</SvelteVirtualChat>
	{/key}

	{#if !isFollowing && !replyingTo}
		<div transition:fade class="absolute right-4 bottom-32 z-10">
			<Button
				size="icon"
				variant="secondary"
				onclick={() => chat?.scrollToBottom({ smooth: true })}
				class="rounded-full shadow-lg"
			>
				<ArrowDownIcon class="h-4 w-4" />
			</Button>
		</div>
	{/if}

	<RoomInput
		{roomId}
		bind:replyingTo
		{handleOpenMediaSendMode}
		{handleSendAudioMessage}
		threadRootEventId={threadRoot}
	/>
{:else}
	<div class="m-auto">
		<Spinner class="size-8" />
	</div>
{/if}

{#if showMediaViewer && mediaViewerSrc}
	<MediaViewer
		src={mediaViewerSrc}
		text={mediaViewerInfo?.body}
		mediaType={viewedMediaType}
		mode={viewerMode}
		onClose={handleCloseMediaViewer}
		onSend={handleSendMedia}
		filename={mediaViewerInfo?.filename}
		mediaSource={mediaViewerSource}
		mediaSize={mediaViewerInfo?.size ?? 0}
	/>
{/if}
