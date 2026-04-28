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
	import { loginStore, roomsCollection, roomStore } from '../../../hooks.client';
	import RoomInput from './room-input.svelte';
	import MediaViewer from '../common/media-viewer.svelte';
	import type { MediaViewerInfo } from '../media/utils';
	import { Spinner } from '../ui/spinner';
	import {
		createMatrixRequest,
		isVideoOrImageInfo,
		submitAsyncRequest,
		uploadMedia,
		type AudioInfo,
		type FileInfo,
		type ImageInfo,
		type MediaRequestParameters,
		type RoomMessageEventContent,
		type VideoInfo
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		roomId: string;
		roomAvatarUrl: string | null;
		threadRoot: string | null;
	};
	let { roomId, roomAvatarUrl, threadRoot }: Props = $props();

	if (import.meta.env.DEV) {
		// eslint-disable-next-line svelte/no-inspect
		$inspect(roomStore.state);
	}

	let isLoadingMore = $state(false);
	let prevScrollHeight = $state(0);

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
				roomsCollection.state.allJoinedRooms[roomId].numUnreadMessages > 0
			) {
				const request = createMatrixRequest.markRoomAsRead({
					roomId,
					threadRootEventId: threadRoot
				});
				submitAsyncRequest(request);
			}
		}
	});

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
		prevScrollHeight = scroll.y || 0;
		console.log('Loading more messages !');

		try {
			const request = createMatrixRequest.paginateTimeline({
				roomId,
				threadRootEventId: threadRoot,
				numEvents: 20,
				direction: 'backwards'
			});
			await submitAsyncRequest(request);
			setTimeout(() => {
				const newScrollHeight = scroll.y;
				scroll.scrollTo(undefined, newScrollHeight - prevScrollHeight);
			}, 100);
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
		while (!viewportElement.querySelector(`[data-event-id="${eventId}"]`)) {
			await loadMoreMessages();
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
	let mediaViewerMxcUri = $state<Promise<string> | undefined>();
	let mediaViewerSource = $state<MediaRequestParameters['source']>();
	let mediaViewerInfo = $state<MediaViewerInfo | undefined>();
	let viewerMode: 'send' | 'view' = $state('send');
	let viewedMediaType: 'image' | 'video' | 'file' = $state('image');
	const handleOpenMediaSendMode = (
		type: 'image' | 'video' | 'file',
		src: string,
		mxcUri: Promise<string>,
		info: MediaViewerInfo
	) => {
		viewedMediaType = type;
		mediaViewerSrc = src;
		mediaViewerMxcUri = mxcUri;
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

		mediaViewerInfo = { ...info };
		viewerMode = 'view';
		mediaViewerSource = mediaSource;
		showMediaViewer = true;
	};

	const handleSendMedia = async (
		msgtype: RoomMessageEventContent['msgtype'],
		mediaInfo?: AudioInfo | VideoInfo | ImageInfo | FileInfo,
		additionalInfo?: { message?: string; waveform?: number[] }
	) => {
		let completeInfo = mediaInfo;
		// Consolidate media info from the blob info
		if (mediaViewerInfo && completeInfo) {
			completeInfo.size = mediaViewerInfo.size;
			completeInfo.mimetype = mediaViewerInfo.mimeType ?? null;
			// If the media supports thumbnails and we successfully generated it,
			// add them to the message
			if (isVideoOrImageInfo(completeInfo) && mediaViewerInfo.thumbnailInfo) {
				let thumbInfo = await mediaViewerInfo.thumbnailInfo;
				if (thumbInfo.blob) {
					completeInfo.thumbnail_info = {
						h: thumbInfo.h,
						w: thumbInfo.w,
						mimetype: thumbInfo.blob.type,
						size: thumbInfo.blob.size
					};
					completeInfo.thumbnail_url = await uploadMedia(
						thumbInfo.blob.type,
						await thumbInfo.blob.arrayBuffer()
					);
				}
			}
		}
		if (!mediaViewerMxcUri) throw Error('Missing media URI');
		let request = createMatrixRequest.sendMessage({
			roomId,
			message: {
				msgtype,
				body: additionalInfo?.message ?? '', // The body must be defined for some reason.
				// TODO: use those two fields ?
				'm.mentions': null,
				filename: mediaViewerInfo?.filename ?? null,
				info: completeInfo ?? null,
				url: await mediaViewerMxcUri,
				'org.matrix.msc1767.audio':
					msgtype === 'm.audio'
						? {
								duration: (completeInfo as AudioInfo).duration ?? 1,
								waveform: additionalInfo?.waveform
							}
						: null
			} as RoomMessageEventContent, // TODO: Remove assertion
			replyToId: replyingTo?.eventId ?? null,
			threadRootEventId: threadRoot // We cannot send thread messages from here
		});

		await submitAsyncRequest(request);

		replyingTo = null; // Clear reply state after sending
		showMediaViewer = false;
		mediaViewerSrc = null;
		mediaViewerMxcUri = undefined;
		mediaViewerInfo = undefined;
	};

	const handleSendAudioMessage = async (
		blob: Blob,
		duration: number,
		waveform: number[] | null
	) => {
		mediaViewerMxcUri = uploadMedia(blob.type, await blob.arrayBuffer());
		mediaViewerInfo = {
			filename: 'audio-recording_' + new Date().toISOString() + '.' + blob.type.split('/').pop(),
			size: blob.size,
			mimeType: blob.type
		};
		const info: AudioInfo = {
			mimetype: blob.type,
			size: blob.size,
			duration: Math.ceil(duration)
		};
		await handleSendMedia('m.audio', info, {
			waveform: waveform?.map((f) => Math.floor(f * 1000))
		});
	};

	const handleCloseMediaViewer = () => {
		showMediaViewer = false;
	};
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
							currentUserId={loginStore.state.userId ?? 'shouldbedefined'}
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
	/>
{/if}
