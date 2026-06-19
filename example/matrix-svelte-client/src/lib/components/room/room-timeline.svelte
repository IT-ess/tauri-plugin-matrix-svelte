<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { LoaderIcon, ArrowDownIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import './room.css';
	import Item from './items/item.svelte';
	import { ScrollState } from 'runed';
	import { useDebounce } from 'runed';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { tick } from 'svelte';
	import { cn } from '$lib/utils.svelte';
	import { roomsCollection, roomStore } from '../../../hooks.client';
	import RoomInput from './room-input.svelte';
	import MediaViewer from '../common/media-viewer.svelte';
	import type { MediaViewerInfo } from '../media/utils';
	import { Spinner } from '../ui/spinner';
	import {
		awaitPaginateTimeline,
		createMatrixRequest,
		sendMediaMessage,
		submitAsyncRequest,
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

	let viewportElement = $state<HTMLElement | null>(null)!;
	const scroll = new ScrollState({
		element: () => viewportElement,
		idle: 100, // Shorter idle time for messaging
		offset: { top: 100 }, // Consider "on top" when within 100px
		onScroll: async () => {
			if (scroll.arrived.top && !isLoadingMore) {
				await loadMoreMessages();
			}
		},
		onStop: () => {
			if (
				scroll.arrived.bottom &&
				roomStore.state.tlState &&
				roomsCollection.state.allJoinedRooms[roomId] &&
				roomsCollection.state.allJoinedRooms[roomId].numUnreadMessages > 0
			) {
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
			}
		}
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

	let showScrollButton = $derived(!scroll.arrived.bottom && scroll.y > 100);

	// Load more messages when scrolling up with 1 sec debounce
	const loadMoreMessages = useDebounce(async () => {
		if (
			isLoadingMore ||
			roomStore.state.tlState?.fullyPaginated ||
			(roomStore.state.timelineKind?.kind == 'mainRoom' &&
				roomStore.state.tlState?.items[0].kind === 'virtual' &&
				roomStore.state.tlState?.items[0].data.kind === 'timelineStart')
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
		if (!viewportElement) return;

		// Find the element with the matching event ID
		let counter = 0;
		while (!viewportElement.querySelector(`[data-event-id="${eventId}"]`)) {
			// Paginate at most 200 events
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
		const messageElement = viewportElement.querySelector(`[data-event-id="${eventId}"]`);

		if (messageElement) {
			const messageRect = messageElement.getBoundingClientRect();
			const containerRect = viewportElement.getBoundingClientRect();

			// Calculate the element's position relative to the scroll container
			const elementTopInContainer = messageRect.top - containerRect.top + viewportElement.scrollTop;
			const containerHeight = viewportElement.clientHeight;

			// Scroll to center the message in the viewport
			const targetScrollTop =
				elementTopInContainer - containerHeight / 2 + messageElement.clientHeight / 2;

			scroll.scrollTo(0, Math.max(0, targetScrollTop));

			messageElement.classList.add('highlight-message');
			setTimeout(() => {
				messageElement.classList.remove('highlight-message');
			}, 3000);
		}
	};

	$effect.pre(() => {
		if (!viewportElement) return; // not yet mounted

		// reference `messages` array length so that this code re-runs whenever it changes
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		roomStore.state.tlState?.items.length;

		// autoscroll when new messages are added
		if (
			viewportElement.offsetHeight + viewportElement.scrollTop >
			viewportElement.scrollHeight - 20
		) {
			tick().then(() => {
				scroll.scrollTo(0, viewportElement.scrollHeight);
			});
		}
	});

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
		if (openingFocus) {
			// We wait for the viewportElement to be available
			setTimeout(() => {
				scrollToMessage(openingFocus);
			}, 100);
		}
	});
</script>

{#if roomStore.state.tlState}
	<div class={cn('w-full flex-1 overflow-hidden')}>
		<ScrollArea bind:viewportRef={viewportElement} class="h-full bg-white">
			<div class="flex flex-col gap-4 p-4 pb-2">
				{#if isLoadingMore}
					<div class="flex justify-center py-2" transition:fade|local>
						<LoaderIcon class="text-muted-foreground h-6 w-6 animate-spin" />
					</div>
				{/if}
				{#each roomStore.state.tlState.items as item (item.uniqueId)}
					<div transition:fade|local>
						<Item
							{item}
							{roomId}
							onReply={handleReplyTo}
							onScrollToMessage={scrollToMessage}
							repliedToMessage={item.kind === 'msgLike' && item.data.inReplyToId !== null
								? roomStore.state.tlState?.items.find((i) => i.eventId === item.data.inReplyToId)
								: undefined}
							{handleOpenMediaViewMode}
							roomAvatar={roomAvatarUrl}
							roomMembers={roomStore.state.members}
							threadRootEventId={threadRoot}
						/>
					</div>
				{:else}
					<p>No items yet</p>
				{/each}
				<div id="bottomscroll"></div>
			</div>
		</ScrollArea>
	</div>

	{#if showScrollButton && !replyingTo}
		<div transition:fade class="absolute right-4 bottom-32 z-10">
			<Button
				size="icon"
				variant="secondary"
				onclick={() => scroll.scrollToBottom()}
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
