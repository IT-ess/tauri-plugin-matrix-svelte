<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { ScrollArea } from 'bits-ui';
	import { Input } from '$lib/components/ui/input';
	import { SendIcon, LoaderIcon, ArrowDownIcon, XIcon, ReplyIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import './room.css';
	import {
		createMatrixRequest,
		ProfileStore,
		submitAsyncRequest,
		type RoomStore
	} from 'tauri-plugin-matrix-svelte-api';
	import Item from './items/item.svelte';
	import { ScrollState } from 'runed';
	import { useDebounce } from 'runed';
	import { tick } from 'svelte';
	import SearchBar from './search-bar.svelte';

	type Props = {
		roomStore: RoomStore;
		profileStore: ProfileStore;
		currentUserId: string;
	};
	let { roomStore, profileStore, currentUserId }: Props = $props();

	let isLoadingMore = $state(false);
	let prevScrollHeight = $state(0);
	let newMessage: string = $state('');

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
				!roomStore.state.tlState?.scrolledPastReadMarker
			) {
				const request = createMatrixRequest.fullyReadReceipt({
					roomId: roomStore.id,
					eventId:
						roomStore.state.tlState.items[roomStore.state.tlState.items.length - 1].eventId ?? ''
				});
				roomStore.state.tlState.scrolledPastReadMarker = true;
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
			(roomStore.state.tlState?.items[0].kind === 'virtual' &&
				roomStore.state.tlState?.items[0].data.kind === 'timelineStart')
		)
			return;

		isLoadingMore = true;
		prevScrollHeight = scroll.y || 0;
		console.log('Loading more messages !');

		try {
			const request = createMatrixRequest.paginateRoomTimeline({
				roomId: roomStore.id,
				numEvents: 50,
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

	// Cancel reply
	const cancelReply = () => {
		replyingTo = null;
	};

	// Handle sending new message
	const handleSendMessage = async () => {
		if (!newMessage.trim()) return;

		let request;
		if (replyingTo) {
			// Send reply message
			request = createMatrixRequest.sendTextMessage(roomStore.id, newMessage, {
				replyToEventId: replyingTo.eventId
			});
		} else {
			// Send regular message
			request = createMatrixRequest.sendTextMessage(roomStore.id, newMessage, {});
		}

		await submitAsyncRequest(request);
		newMessage = '';
		replyingTo = null; // Clear reply state after sending
	};

	// Handle enter key press
	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSendMessage();
		} else if (e.key === 'Escape' && replyingTo) {
			e.preventDefault();
			cancelReply();
		}
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
</script>

<div class="bg-background relative flex h-[600px] flex-col rounded-lg border">
	<SearchBar roomId={roomStore.id} />
	<!-- Chat messages container -->
	<div class="flex-1 overflow-hidden">
		<ScrollArea.Root class="h-full bg-white">
			<ScrollArea.Viewport bind:ref={viewportElement} class="h-full">
				<div class="flex flex-col gap-4 p-4 pb-0">
					<!-- Loading indicator -->
					{#if isLoadingMore}
						<div class="flex justify-center py-2" transition:fade|local>
							<LoaderIcon class="text-muted-foreground h-6 w-6 animate-spin" />
						</div>
					{/if}

					<!-- Messages list -->
					{#if roomStore.state.tlState?.items}
						{#each roomStore.state.tlState?.items as item (item.eventId ?? crypto.randomUUID())}
							<div transition:fade|local>
								<Item
									{item}
									{profileStore}
									roomId={roomStore.id}
									{currentUserId}
									onReply={handleReplyTo}
									onScrollToMessage={scrollToMessage}
									repliedToMessage={item.kind === 'msgLike' && item.data.threadRoot !== null
										? roomStore.state.tlState?.items.find((i) => i.eventId === item.data.threadRoot)
										: undefined}
								/>
							</div>
						{/each}
					{:else}
						<b>Error: timeline state should be defined</b>
					{/if}

					<!-- Spacer to ensure last message isn't hidden behind input -->
					<div class="h-2"></div>
				</div>
			</ScrollArea.Viewport>
			<ScrollArea.Scrollbar
				class="flex h-full w-2.5 touch-none border-l border-l-transparent p-px transition-colors select-none"
				orientation="vertical"
			>
				<ScrollArea.Thumb class="bg-border relative flex-1 rounded-full" />
			</ScrollArea.Scrollbar>

			<ScrollArea.Corner />
		</ScrollArea.Root>
	</div>

	{#if showScrollButton && !replyingTo}
		<div transition:fade class="absolute right-4 bottom-20 z-10">
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

	<!-- Message input - fixed at bottom -->
	<div class="bg-background border-t">
		<!-- Reply preview -->
		{#if replyingTo}
			<div class="bg-muted/50 border-b p-3" transition:fade>
				<div class="flex items-start justify-between gap-2">
					<div class="flex min-w-0 flex-1 items-start gap-2">
						<ReplyIcon class="text-muted-foreground mt-0.5 h-4 w-4 flex-shrink-0" />
						<div class="min-w-0 flex-1">
							<div class="text-foreground text-sm font-medium">
								Replying to {replyingTo.senderName}
							</div>
							<div class="text-muted-foreground truncate text-sm">
								{replyingTo.content}
							</div>
						</div>
					</div>
					<Button size="icon" variant="ghost" onclick={cancelReply} class="h-6 w-6 flex-shrink-0">
						<XIcon class="h-3 w-3" />
						<span class="sr-only">Cancel reply</span>
					</Button>
				</div>
			</div>
		{/if}

		<div class="p-4">
			<div class="flex gap-2">
				<Input
					bind:value={newMessage}
					onkeydown={handleKeyDown}
					placeholder={replyingTo ? `Reply to ${replyingTo.senderName}...` : 'Type a message...'}
					class="flex-1"
				/>
				<Button onclick={handleSendMessage} disabled={!newMessage.trim()}>
					<SendIcon class="h-4 w-4" />
					<span class="sr-only">Send message</span>
				</Button>
			</div>
		</div>
	</div>
</div>
