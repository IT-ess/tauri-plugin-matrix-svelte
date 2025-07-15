<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { ScrollArea } from 'bits-ui';
	import { Input } from '$lib/components/ui/input';
	import { SendIcon, LoaderIcon, ArrowDownIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
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

	type Props = {
		roomStore: RoomStore;
		profileStore: ProfileStore;
		currentUserId: string;
	};
	let { roomStore, profileStore, currentUserId }: Props = $props();

	let isLoadingMore = $state(false);
	let prevScrollHeight = $state(0);
	let newMessage: string = $state('');

	let viewportElement = $state<HTMLElement | null>(null)!;
	const scroll = new ScrollState({
		element: () => viewportElement,
		idle: 100, // Shorter idle time for messaging
		offset: { top: 100 }, // Consider "on top" when within 100px
		onScroll: () => {
			if (scroll.arrived.top && !isLoadingMore) {
				loadMoreMessages();
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

	// Handle sending new message
	const handleSendMessage = async () => {
		if (!newMessage.trim()) return;

		let request = createMatrixRequest.sendTextMessage(
			roomStore.id,
			newMessage,
			{} // TODO: handle replies and other stuff
		);
		await submitAsyncRequest(request);
		newMessage = '';
	};

	// Handle enter key press
	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSendMessage();
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
	<!-- Chat messages container -->
	<div class="flex-1 overflow-hidden">
		<ScrollArea.Root class="h-full bg-white">
			<ScrollArea.Viewport bind:ref={viewportElement} class="h-full p-4">
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
								<Item {item} {profileStore} roomId={roomStore.id} {currentUserId} />
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

	{#if showScrollButton}
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
	<div class="bg-background border-t p-4">
		<div class="flex gap-2">
			<Input
				bind:value={newMessage}
				onkeydown={handleKeyDown}
				placeholder="Type a message..."
				class="flex-1"
			/>
			<Button onclick={handleSendMessage} disabled={!newMessage.trim()}>
				<SendIcon class="h-4 w-4" />
				<span class="sr-only">Send message</span>
			</Button>
		</div>
	</div>
</div>
