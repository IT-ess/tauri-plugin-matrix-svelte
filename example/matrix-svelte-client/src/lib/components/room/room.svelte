<script lang="ts">
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Button } from '$lib/components/ui/button';
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

	type Props = {
		roomStore: RoomStore;
		profileStore: ProfileStore;
		currentUserId: string;
	};
	let { roomStore, profileStore, currentUserId }: Props = $props();

	let isLoadingMore = $state(false);
	let showScrollToBottom = $state(false);
	let viewport: HTMLDivElement | null = $state(null);
	let prevScrollHeight = $state(0);
	let newMessage: string = $state('');

	const scrollState = new ScrollState({
		element: () => viewport
	});

	if (import.meta.env.DEV) {
		$inspect(scrollState);
	}

	// Load more messages when scrolling up
	const loadMoreMessages = async () => {
		if (
			isLoadingMore ||
			roomStore.state.tlState?.fullyPaginated ||
			(roomStore.state.tlState?.items[0].kind === 'virtual' &&
				roomStore.state.tlState?.items[0].data.kind === 'timelineStart')
		)
			return;

		isLoadingMore = true;
		prevScrollHeight = viewport?.scrollHeight || 0;
		console.log('Loading more messages !');

		try {
			const request = createMatrixRequest.paginateRoomTimeline({
				roomId: roomStore.id,
				numEvents: 50,
				direction: 'backwards'
			});
			await submitAsyncRequest(request);
			// const olderMessages = await fetchMessages(page + 1);
			// Check if we have more messages
			// if (olderMessages.length < PAGE_SIZE) {
			//   hasMoreMessages = false;
			// }
			// page++;
			// messages = [...olderMessages, ...messages]; // newer messages passed through a dedicated field by the backend ?
			// Maintain scroll position after loading more messages
			setTimeout(() => {
				if (viewport) {
					const newScrollHeight = viewport.scrollHeight;
					viewport.scrollTop = newScrollHeight - prevScrollHeight;
				}
			}, 100);
		} finally {
			isLoadingMore = false;
		}
	};

	// Handle scroll to check when we need to load more messages
	const handleScroll = async (e: Event) => {
		const target = e.target as HTMLDivElement;
		const scrollTop = target.scrollTop;
		const threshold = 100;

		showScrollToBottom = scrollTop < -threshold;

		if (scrollTop <= 100 && !isLoadingMore) {
			// Load more when near top
			console.log('Almost reached the top !');
			await loadMoreMessages();
		}
	};

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

	// TODO: rework this behaviour
	// $effect.pre(() => {
	// 	if (!viewport) return; // not yet mounted

	// 	// reference `messages` array length so that this code re-runs whenever it changes
	// 	// eslint-disable-next-line @typescript-eslint/no-unused-expressions
	// 	roomStore.state.tlState?.items.length;

	// 	// autoscroll when new messages are added
	// 	if (viewport.offsetHeight + viewport.scrollTop > viewport.scrollHeight - 20) {
	// 		tick().then(() => {
	// 			viewport?.scrollTo(0, viewport.scrollHeight);
	// 		});
	// 	}
	// });
</script>

<div class="bg-background relative flex h-[600px] flex-col rounded-lg border">
	<!-- Chat messages container -->
	<div class="flex-1 overflow-hidden">
		<ScrollArea class="h-full" bind:ref={viewport} onscroll={handleScroll}>
			<div class="flex flex-col gap-4 p-4 pb-0">
				<!-- Loading indicator -->
				{#if isLoadingMore}
					<div class="flex justify-center py-2" transition:fade|local>
						<LoaderIcon class="text-muted-foreground h-6 w-6 animate-spin" />
					</div>
				{:else if roomStore.state.tlState?.fullyPaginated}
					<p class="text-muted-foreground text-center text-sm">No more messages</p>
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
		</ScrollArea>
	</div>

	<!-- Scroll to bottom button: TODO: FIXME -->
	{#if showScrollToBottom}
		<div transition:fade class="absolute right-4 bottom-20 z-10">
			<Button
				size="icon"
				variant="secondary"
				onclick={scrollState.scrollToBottom}
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
