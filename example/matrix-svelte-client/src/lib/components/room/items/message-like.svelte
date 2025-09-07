<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import {
		createMatrixRequest,
		ProfileStore,
		submitAsyncRequest,
		type MsgLikeContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { MessagesSquare, ReplyIcon, Share2Icon } from '@lucide/svelte';
	import ImageMessage from './image-message.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { cn, getInitials } from '$lib/utils';
	import AudioMessage from './audio-message.svelte';
	import VideoMessage from './video-message.svelte';
	import FileMessage from './file-message.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { platform } from '@tauri-apps/plugin-os';
	import DesktopActions from './item-actions/desktop-actions.svelte';
	import { press, swipe, type GestureCustomEvent } from 'svelte-gestures';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { Popover, PopoverContent } from '$lib/components/ui/popover';
	import { Button } from '$lib/components/ui/button';
	import PopoverTrigger from '$lib/components/ui/popover/popover-trigger.svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';

	type Props = {
		data: MsgLikeContent;
		timestamp: number;
		isOwn: boolean;
		roomId: string;
		eventId: string;
		currentUserId: string;
		profileStore: ProfileStore;
		repliedToMessage?: MsgLikeContent;
		onReply?: (eventId: string, senderName: string, content: string) => void;
		onScrollToMessage?: (eventId: string) => void;
	};

	let {
		data,
		timestamp,
		isOwn,
		roomId,
		eventId,
		currentUserId,
		profileStore,
		onReply,
		repliedToMessage,
		onScrollToMessage
	}: Props = $props();

	const { senderId, sender } = data;
	let reactionsArray = $derived(Object.keys(data.reactions));

	let showActions = $state(false);
	let showDropdown = $state(false);
	let reactionsPopoverAnchor = $state<HTMLElement>(null!);

	// Format timestamp
	const formatTime = (timestamp: number) => {
		return new Date(timestamp).toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	};

	// Common emojis for reactions
	const commonEmojis = ['👍', '❤️', '😂', '😮', '😢', '🎉', '👎', '💪'];

	// Add reaction to message
	const handleAddReaction = async (emoji: string) => {
		const request = createMatrixRequest.toggleReaction({
			reaction: emoji,
			roomId,
			timelineEventId: eventId
		});
		await submitAsyncRequest(request);
		showDropdown = false;
	};

	// Handle reply action
	const handleReply = () => {
		if (!onReply) return;

		let content = extractContentFromMsg(data);

		onReply(eventId, sender ?? 'Unknown', content);
	};

	const handleShare = async () => {
		if (data.kind === 'text') {
			await shareText(data.body.body, { mimeType: 'plain/text' });
		}
	};

	const handleShowdropdown = () => {
		showDropdown = true;
	};

	// SWIPE TO REPLY

	// Animation state
	const swipeOffset = new Tween(0, { duration: 200, easing: cubicOut });
	const replyOpacity = new Tween(0, { duration: 150, easing: cubicOut });

	let isSwipeActive = $state(false);
	let isDragging = $state(false);
	let startX = $state(0);
	let currentX = $state(0);

	// Swipe threshold for triggering reply
	const SWIPE_THRESHOLD = 100; // TODO: adapt the threshold for responsive ?
	const MAX_SWIPE = 150;

	function handleSwipeStart(event: GestureCustomEvent) {
		isDragging = true;
		startX = event.detail.x;
		currentX = event.detail.x;
	}

	function handleSwipeMove(event: GestureCustomEvent) {
		if (!isDragging) return;

		currentX = event.detail.x;
		const deltaX = currentX - startX;

		// Only allow swipe from left to right (for reply action)
		if (deltaX > 0) {
			const clampedDelta = Math.min(deltaX, MAX_SWIPE);
			swipeOffset.set(clampedDelta, { duration: 0 });

			// Show reply icon with opacity based on swipe distance
			const opacity = Math.min(clampedDelta / SWIPE_THRESHOLD, 1);
			replyOpacity.set(opacity, { duration: 0 });

			isSwipeActive = clampedDelta > SWIPE_THRESHOLD / 2;
		}
	}

	function handleSwipeEnd() {
		if (!isDragging) return;

		const deltaX = currentX - startX;
		const shouldTriggerReply = deltaX >= SWIPE_THRESHOLD;

		if (shouldTriggerReply) {
			// Trigger reply action
			handleReply();
		}

		// Reset animation state
		swipeOffset.set(0);
		replyOpacity.set(0);
		isSwipeActive = false;
		isDragging = false;
	}

	const extractContentFromMsg = (msg: MsgLikeContent): string => {
		switch (msg.kind) {
			case 'text':
			case 'emote':
				return msg.body.body;
			case 'image':
				return msg.body.body || 'Image';
			case 'audio':
				return msg.body.body || 'Audio message';
			case 'video':
				return msg.body.body || 'Video message';
			case 'file':
				return msg.body.body || 'File';
			case 'sticker':
				return 'Sticker';
			case 'redacted':
				return 'This message has been deleted';
			case 'unableToDecrypt':
				return 'Encrypted message';
			default:
				return `Unsupported message type: ${msg.kind}`;
		}
	};

	const currentPlatform = platform();

	const handleReplyClick = () => {
		if (onScrollToMessage && data.threadRoot) {
			onScrollToMessage(data.threadRoot);
		}
	};

	onMount(async () => {
		if (profileStore.state[senderId] === undefined) {
			await invoke('plugin:matrix-svelte|fetch_user_profile', {
				userId: senderId,
				roomId
			});
		}
	});
</script>

<Popover bind:open={showDropdown}>
	<div
		onmouseenter={() => (showActions = true)}
		onmouseleave={() => (showActions = false)}
		use:press={() => ({ timeframe: 300, triggerBeforeFinished: true })}
		onpress={() => {
			showDropdown = true;
		}}
		style="transform: translateX({swipeOffset.current}px)"
		use:swipe={() => ({ timeframe: 300, minSwipeDistance: 60, touchAction: 'pan-y' })}
		onswipedown={handleSwipeStart}
		onswipemove={handleSwipeMove}
		onswipeup={handleSwipeEnd}
		class={cn(
			'group flex gap-2 transition-transform duration-200',
			isOwn && 'flex-row-reverse',
			`translate-[${swipeOffset.current}]px`
		)}
		role="button"
		tabindex="0"
		aria-label="Swipe right to reply"
	>
		<PopoverTrigger />
		<Avatar>
			<!-- Reactive store, once the profile is loaded we load the image -->
			{#if profileStore.state[senderId]?.state === 'loaded' && profileStore.state[senderId].data.avatarDataUrl}
				<AvatarImage src={profileStore.state[senderId].data.avatarDataUrl} alt={sender} />
			{:else}
				{@render avatarFallback(sender)}
			{/if}
		</Avatar>
		<DropdownMenu bind:open={showDropdown}>
			<DropdownMenuTrigger />
			{#if data.kind === 'sticker'}
				<!-- Render sticker outside the block -->
				<div class={cn('relative max-w-[30%] p-3', isSwipeActive ? 'ring-2 ring-blue-300' : '')}>
					<ImageMessage itemContent={data.body} isSticker />
				</div>
			{:else}
				<div bind:this={reactionsPopoverAnchor} class="relative max-w-[60%]">
					<div
						class={[
							'relative  rounded-lg p-3',
							isOwn ? 'bg-primary text-primary-foreground' : 'bg-muted',
							isSwipeActive ? 'ring-2 ring-blue-300' : ''
						]}
					>
						<div class="flex items-center gap-2">
							<p class="text-sm font-medium">{data.sender}</p>
							<span class="text-xs opacity-70">{formatTime(timestamp ?? 0)}</span>
						</div>
						{#if repliedToMessage}
							<div
								class="relative mt-1 cursor-pointer rounded-lg bg-white p-2 text-sm text-black transition-colors hover:bg-gray-100"
								onclick={handleReplyClick}
								role="button"
								tabindex="0"
								onkeydown={(e) => e.key === 'Enter' && handleReplyClick()}
							>
								<MessagesSquare class="absolute top-1 right-1" />
								<p class="mr-8 text-sm font-medium">{repliedToMessage.sender}</p>
								<p class="text-sm">{extractContentFromMsg(repliedToMessage)}</p>
							</div>
						{/if}
						{#if data.kind === 'text'}
							<p class="mt-1 text-sm">
								{data.body.body}
							</p>
						{:else if data.kind === 'emote'}
							<p class="mt-1 text-sm">
								<b>{data.sender}:</b>{data.body.body}
								<!-- same as a text message, but with sender name in front -->
							</p>
						{:else if data.kind === 'image'}
							<ImageMessage itemContent={data.body} isSticker={false} />
						{:else if data.kind === 'audio'}
							<AudioMessage itemContent={data.body} />
						{:else if data.kind === 'video'}
							<VideoMessage itemContent={data.body} />
						{:else if data.kind === 'file'}
							<FileMessage itemContent={data.body} />
						{:else if data.kind === 'redacted'}
							<Badge variant="destructive">This message has been deleted</Badge>
						{:else if data.kind === 'unableToDecrypt'}
							<Badge variant={isOwn ? 'secondary' : 'default'}>Encrypted message</Badge>
						{:else}
							<p class="text-muted text-sm">
								The message type: {data.kind} is not supported yet
							</p>
						{/if}
					</div>
				</div>
			{/if}

			{#if currentPlatform !== 'android' && currentPlatform !== 'ios'}
				<DesktopActions
					bind:showActions
					{commonEmojis}
					{currentUserId}
					{isOwn}
					{handleAddReaction}
					{handleReply}
					reactions={data.reactions}
					{handleShowdropdown}
				/>
			{/if}
			<DropdownMenuContent
				customAnchor={reactionsPopoverAnchor}
				align="end"
				side={isOwn ? 'left' : 'right'}
			>
				<DropdownMenuItem onclick={handleReply} class="text-md">
					<ReplyIcon class="h-4 w-4" />
					Reply</DropdownMenuItem
				>
				{#if (currentPlatform === 'android' || currentPlatform === 'ios') && data.kind === 'text'}
					<DropdownMenuItem onclick={handleShare} class="text-md">
						<Share2Icon class="h-4 w-4" />
						Share</DropdownMenuItem
					>
				{/if}
			</DropdownMenuContent>
		</DropdownMenu>
	</div>
	<PopoverContent
		side="top"
		align={isOwn ? 'end' : 'start'}
		customAnchor={reactionsPopoverAnchor}
		class="relative top-0 w-fit p-2"
	>
		<div class="flex gap-1">
			{#each commonEmojis as emoji (emoji)}
				<Button
					variant={reactionsArray.includes(emoji)
						? Object.keys(data.reactions[emoji]).includes(currentUserId)
							? 'secondary'
							: 'ghost'
						: 'ghost'}
					size="icon"
					class="h-8 w-8"
					onclick={() => {
						handleAddReaction(emoji);
					}}
				>
					{emoji}
				</Button>
			{/each}
		</div>
	</PopoverContent>
</Popover>

{#snippet avatarFallback(sender?: string)}
	<AvatarFallback>{getInitials(sender ?? 'John Doe')}</AvatarFallback>
{/snippet}
