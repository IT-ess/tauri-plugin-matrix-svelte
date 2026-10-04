<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '#lib/components/ui/avatar/index.js';
	import * as Message from '#lib/components/ui/message/index.js';
	import * as Bubble from '#lib/components/ui/bubble/index.js';
	import {
		Copy,
		MessageSquareReply,
		MessagesSquare,
		ReplyIcon,
		SquarePenIcon,
		Trash2Icon
	} from '@lucide/svelte';
	import ImageMessage from './image-message.svelte';
	import {
		getCustomMxcUriFromOriginal,
		getInitials,
		gotoProfile,
		gotoThread
	} from '#lib/utils.svelte.js';
	import AudioMessage from './audio-message.svelte';
	import VideoMessage from './video-message.svelte';
	import FileMessage from './file-message.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { platform } from '@tauri-apps/plugin-os';
	import DesktopActions from './item-actions/desktop-actions.svelte';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '#lib/components/ui/dropdown-menu/index.js';
	import { Popover, PopoverContent } from '#lib/components/ui/popover/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import PopoverTrigger from '#lib/components/ui/popover/popover-trigger.svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import EditTextMessage from './item-actions/edit-text-message.svelte';
	import Reactions from './item-actions/reactions.svelte';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import TextMessage from './text-message.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import ThreadPreview from '../thread/thread-preview.svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { usePress, useSwipe, type GestureCustomEvent } from 'svelte-gestures';
	import {
		createMatrixRequest,
		submitAsyncRequest,
		type FrontendRoomMember,
		type MediaRequestParameters,
		type MessageAbility,
		type MsgLikeContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		data: MsgLikeContent;
		timestamp: number;
		isOwn: boolean;
		roomId: string;
		eventId: string;
		timelineItemId: string;
		isLocal: boolean;
		currentUserId: string;
		repliedToMessage?: MsgLikeContent;
		onReply?: (eventId: string, senderName: string, content: string) => void;
		onScrollToMessage?: (eventId: string) => void;
		abilities: MessageAbility[];
		handleOpenMediaViewMode: (
			type: 'image' | 'video',
			src: string,
			info: {
				filename?: string;
				body?: string;
				size: number;
			},
			mediaSource: MediaRequestParameters['source']
		) => void;
		threadRootEventId: string | null;
		roomAvatar: string | null;
		roomMembers: Record<string, FrontendRoomMember>;
		groupedWithPrev: boolean;
		groupedWithNext: boolean;
		highlighted: boolean;
	};

	let {
		data,
		timestamp,
		isOwn,
		roomId,
		eventId,
		timelineItemId,
		isLocal,
		currentUserId,
		onReply,
		repliedToMessage,
		onScrollToMessage,
		abilities,
		handleOpenMediaViewMode,
		threadRootEventId,
		roomAvatar,
		roomMembers,
		groupedWithPrev,
		groupedWithNext,
		highlighted
	}: Props = $props();

	let senderId = $derived(data.senderId);
	let sender = $derived(data.sender);
	let reactionsArray = $derived(Object.keys(data.reactions));

	let showDropdown = $state(false);
	let isEditing = $state(false);
	let reactionsPopoverAnchor = $state<HTMLElement | null>(null);

	// Format timestamp
	const formatTime = (timestamp: number) => {
		return new Date(timestamp).toLocaleTimeString(getLocale(), {
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
			threadRootEventId,
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

	const onSubmitEditMessage = (newMessage: string) => {
		// We only support editing text messages right now
		if (data.kind !== 'text') return;
		let request = createMatrixRequest.editMessage({
			roomId,
			threadRootEventId,
			timelineEventItemId: { timelineItemId, isLocal },
			editedContent: {
				msgtype: 'm.text',
				body: newMessage,
				'm.mentions': null,
				'com.beeper.linkpreviews': null
			}
		});
		submitAsyncRequest(request);
	};

	const handleRedactMessage = () => {
		const request = createMatrixRequest.redactMessage({
			roomId,
			threadRootEventId,
			timelineEventId: eventId,
			reason: null
		});
		submitAsyncRequest(request);
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
	const isDesktop = currentPlatform !== 'android' && currentPlatform !== 'ios';

	const handleReplyClick = () => {
		if (onScrollToMessage && data.inReplyToId) {
			onScrollToMessage(data.inReplyToId);
		}
	};

	let canReplyTo = $derived(abilities.includes('canReplyTo'));
</script>

<!-- Overlaid on the bubble's top edge (not the content's, which starts with the sender name)
     so it never takes layout space and grouped rows stay tight -->
{#snippet desktopActions()}
	{#if isDesktop}
		<Message.Header
			class={['absolute top-0 z-10 -translate-y-1/2 px-0', isOwn ? 'left-0' : 'right-0']}
		>
			<DesktopActions
				{commonEmojis}
				{currentUserId}
				{isOwn}
				{handleAddReaction}
				{handleReply}
				reactions={data.reactions}
				{handleShowdropdown}
				{abilities}
			/>
		</Message.Header>
	{/if}
{/snippet}

<Popover bind:open={showDropdown}>
	<Message.Root
		align={isOwn ? 'end' : 'start'}
		{...usePress(
			() => {
				showDropdown = true;
			},
			() => ({
				timeframe: 300,
				triggerBeforeFinished: true
			})
		)}
		{...useSwipe(
			() => {},
			() => ({ timeframe: 300, minSwipeDistance: 50, touchAction: 'pan-y' }),
			{
				onswipeup: handleSwipeEnd,
				onswipedown: handleSwipeStart,
				onswipemove: handleSwipeMove
			}
		)}
		style="transform: translateX({swipeOffset.current}px)"
		class={['rounded-3xl transition-transform duration-200', highlighted && 'highlight-message']}
		role="button"
		tabindex={0}
		aria-label="Swipe to reply"
	>
		<PopoverTrigger />
		{#if !isOwn}
			<Message.Avatar>
				{#if !groupedWithNext}
					<Avatar onclick={() => gotoProfile(senderId)} class="border-primary border">
						<AvatarImage
							src={getCustomMxcUriFromOriginal(roomMembers[senderId]?.avatar)}
							alt={sender}
						/>
						<AvatarFallback>{getInitials(sender ?? '?')}</AvatarFallback>
					</Avatar>
				{/if}
			</Message.Avatar>
		{/if}
		<DropdownMenu bind:open={showDropdown}>
			<DropdownMenuTrigger />
			<Message.Content class="w-fit max-w-[80%]">
				{#if !groupedWithPrev && !isOwn}
					<Message.Header>{sender}</Message.Header>
				{/if}
				{#if data.kind === 'sticker'}
					<!-- Render sticker outside the bubble -->
					<div
						bind:this={reactionsPopoverAnchor}
						class={[
							'relative w-40 rounded-lg [&_img]:h-auto [&_img]:w-full',
							isSwipeActive && 'ring-ring ring-2'
						]}
					>
						{@render desktopActions()}
						<ImageMessage itemContent={data.body} isSticker {handleOpenMediaViewMode} />
					</div>
					{#if reactionsArray.length > 0}
						<div class="flex flex-wrap gap-1">
							<Reactions reactions={data.reactions} {currentUserId} onToggle={handleAddReaction} />
						</div>
					{/if}
				{:else}
					<Bubble.Root
						bind:ref={reactionsPopoverAnchor}
						variant={isOwn ? 'default' : 'muted'}
						class="max-w-full has-data-[slot=bubble-reactions]:mb-5"
					>
						{@render desktopActions()}
						<Bubble.Content class={isSwipeActive ? 'ring-ring ring-2' : undefined}>
							{#if repliedToMessage && !threadRootEventId}
								<div
									class="relative mt-1 cursor-pointer rounded-lg bg-white p-2 text-sm text-black transition-colors hover:bg-gray-100"
									onclick={handleReplyClick}
									role="button"
									tabindex="0"
									onkeydown={(e) => e.key === 'Enter' && handleReplyClick()}
								>
									<MessageSquareReply class="absolute top-1 right-1" />
									<p class="mr-8 text-sm font-medium">{repliedToMessage.sender}</p>
									<p class="text-sm">{extractContentFromMsg(repliedToMessage)}</p>
								</div>
							{/if}
							{#if data.kind === 'text'}
								{#if isEditing}
									<EditTextMessage
										bind:isEditing
										message={data.body.body}
										onEdit={onSubmitEditMessage}
									/>
								{:else}
									<TextMessage textMessage={data.body} />
								{/if}
							{:else if data.kind === 'emote'}
								<p class="mt-1 text-sm">
									<b>{data.sender}:</b>{data.body.body}
									<!-- same as a text message, but with sender name in front -->
								</p>
							{:else if data.kind === 'image'}
								<ImageMessage itemContent={data.body} isSticker={false} {handleOpenMediaViewMode} />
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
							{:else if data.kind === 'audio'}
								<AudioMessage itemContent={data.body} {isOwn} />
							{:else if data.kind === 'video'}
								<VideoMessage itemContent={data.body} {handleOpenMediaViewMode} />
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
							{:else if data.kind === 'redacted'}
								<Badge variant="destructive">{m.message_has_been_deleted()}</Badge>
							{:else if data.kind === 'unableToDecrypt'}
								<Badge variant={isOwn ? 'secondary' : 'default'}>{m.message_encrypted()}</Badge>
							{:else}
								<p class="text-muted text-sm">
									The message type: {data.kind} is not supported yet
								</p>
							{/if}
						</Bubble.Content>
						{#if reactionsArray.length > 0}
							<Bubble.Reactions align={isOwn ? 'end' : 'start'}>
								<Reactions
									reactions={data.reactions}
									{currentUserId}
									onToggle={handleAddReaction}
								/>
							</Bubble.Reactions>
						{/if}
					</Bubble.Root>
				{/if}
				{#if data.threadSummary && !threadRootEventId}
					<ThreadPreview
						threadSummary={data.threadSummary}
						{roomId}
						rootId={eventId}
						{roomAvatar}
					/>
				{/if}
				{#if !groupedWithNext}
					<Message.Footer>{formatTime(timestamp)}</Message.Footer>
				{/if}
			</Message.Content>

			<DropdownMenuContent
				customAnchor={reactionsPopoverAnchor}
				align={isOwn ? 'end' : 'start'}
				side="bottom"
			>
				{#if canReplyTo}
					<DropdownMenuItem onclick={handleReply} class="text-md">
						<ReplyIcon class="size-4" />
						{m.button_reply()}</DropdownMenuItem
					>
				{/if}
				{#if !threadRootEventId && canReplyTo}
					<DropdownMenuItem class="text-md" onclick={() => gotoThread(roomId, eventId, roomAvatar)}
						><MessagesSquare class="size-4" />{m.button_reply_in_thread()}</DropdownMenuItem
					>
				{/if}
				{#if data.kind == 'text'}
					<DropdownMenuItem onclick={() => writeText(data.body.body)} class="text-md">
						<Copy class="size-4" />
						{m.button_copy()}</DropdownMenuItem
					>
				{/if}
				{#if abilities.includes('canEdit')}
					<DropdownMenuItem onclick={() => (isEditing = true)} class="text-md">
						<SquarePenIcon class="size-4" />
						{m.button_edit()}</DropdownMenuItem
					>
				{/if}
				{#if abilities.includes('canDelete')}
					<DropdownMenuItem onclick={handleRedactMessage} class="text-md">
						<Trash2Icon class="size-4" />
						{m.button_delete()}</DropdownMenuItem
					>
				{/if}
			</DropdownMenuContent>
		</DropdownMenu>
	</Message.Root>
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
