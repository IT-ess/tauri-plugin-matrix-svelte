<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import {
		createMatrixRequest,
		ProfileStore,
		submitAsyncRequest,
		type MsgLikeContent
	} from 'tauri-plugin-matrix-svelte-api';
	import Reactions from './reactions.svelte';
	import { Popover, PopoverContent, PopoverTrigger } from '$lib/components/ui/popover';
	import { Tooltip, TooltipContent, TooltipProvider } from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
	import { SmileIcon, ReplyIcon } from '@lucide/svelte';
	import ImageMessage from './image-message.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getInitials } from '$lib/utils';
	import AudioMessage from './audio-message.svelte';
	import VideoMessage from './video-message.svelte';
	import FileMessage from './file-message.svelte';
	import { Badge } from '$lib/components/ui/badge';

	type Props = {
		data: MsgLikeContent;
		timestamp: number;
		isOwn: boolean;
		roomId: string;
		eventId: string;
		currentUserId: string;
		profileStore: ProfileStore;
		onReply?: (eventId: string, senderName: string, content: string) => void;
	};

	let { data, timestamp, isOwn, roomId, eventId, currentUserId, profileStore, onReply }: Props =
		$props();

	const { senderId, sender } = data;

	let reactionsArray = $derived(Object.keys(data.reactions));
	let showActions = $state(false);

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
	};

	// Handle reply action
	const handleReply = () => {
		if (!onReply) return;

		let content = '';

		// Extract content based on message type
		switch (data.kind) {
			case 'text':
			case 'emote':
				content = data.body.body;
				break;
			case 'image':
				content = data.body.body || 'Image';
				break;
			case 'audio':
				content = data.body.body || 'Audio message';
				break;
			case 'video':
				content = data.body.body || 'Video message';
				break;
			case 'file':
				content = data.body.body || 'File';
				break;
			case 'sticker':
				content = 'Sticker';
				break;
			case 'redacted':
				content = 'This message has been deleted';
				break;
			case 'unableToDecrypt':
				content = 'Encrypted message';
				break;
			default:
				content = `Unsupported message type: ${data.kind}`;
		}

		onReply(eventId, sender ?? 'Unknown', content);
		showActions = false;
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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class={['group flex gap-2', isOwn && 'flex-row-reverse']}
	onmouseenter={() => (showActions = true)}
	onmouseleave={() => (showActions = false)}
>
	<Avatar>
		<!-- Reactive store, once the profile is loaded we load the image -->
		{#if profileStore.state[senderId]?.state === 'loaded' && profileStore.state[senderId].data.avatarDataUrl}
			<AvatarImage src={profileStore.state[senderId].data.avatarDataUrl} alt={sender} />
		{:else}
			{@render avatarFallback(sender)}
		{/if}
	</Avatar>

	{#if data.kind === 'sticker'}
		<!-- Render sticker outside the block -->
		<div class="relative max-w-[30%] p-3">
			<ImageMessage itemContent={data.body} isSticker />

			<!-- Action buttons for stickers -->
			{#if showActions && onReply}
				<div
					class={[
						'absolute top-1 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100',
						isOwn ? 'left-1' : 'right-1'
					]}
				>
					<TooltipProvider>
						<Tooltip>
							<Button
								variant="secondary"
								size="icon"
								class="h-6 w-6 shadow-sm"
								onclick={handleReply}
							>
								<ReplyIcon class="h-3 w-3" />
							</Button>
							<TooltipContent>Reply</TooltipContent>
						</Tooltip>
					</TooltipProvider>
				</div>
			{/if}
		</div>
	{:else}
		<div class="relative max-w-[60%]">
			<div
				class={[
					'relative  rounded-lg p-3',
					isOwn ? 'bg-primary text-primary-foreground' : 'bg-muted'
				]}
			>
				<div class="flex items-center gap-2">
					<p class="text-sm font-medium">{data.sender}</p>
					<span class="text-xs opacity-70">{formatTime(timestamp ?? 0)}</span>
				</div>
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
					<ImageMessage itemContent={data.body} />
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
					<p class="text-muted text-sm">The message type: {data.kind} is not supported yet</p>
				{/if}
			</div>

			<!-- Action buttons for regular messages -->
			{#if showActions && onReply}
				<div
					class={[
						'absolute top-1 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100',
						isOwn ? 'left-1' : 'right-1'
					]}
				>
					<TooltipProvider>
						<Tooltip>
							<Button
								variant="secondary"
								size="icon"
								class="h-6 w-6 shadow-sm"
								onclick={handleReply}
							>
								<ReplyIcon class="h-3 w-3" />
							</Button>
							<TooltipContent>Reply</TooltipContent>
						</Tooltip>
					</TooltipProvider>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Reaction button and existing actions -->
	<div class={['flex items-center gap-1', isOwn && 'flex-row-reverse']}>
		<!-- Reaction button -->
		<TooltipProvider>
			<Popover>
				<PopoverTrigger>
					{#snippet child({ props: triggerProps })}
						<Tooltip>
							<Button variant="ghost" size="icon" class="h-6 w-6" {...triggerProps}>
								<SmileIcon class="h-4 w-4" />
							</Button>
						</Tooltip>
					{/snippet}
					<TooltipContent>Add reaction</TooltipContent>
				</PopoverTrigger>
				<PopoverContent class="w-fit p-2">
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
								onclick={() => handleAddReaction(emoji)}
							>
								{emoji}
							</Button>
						{/each}
					</div>
				</PopoverContent>
			</Popover>
		</TooltipProvider>

		{#if reactionsArray.length > 0}
			<Reactions reactions={data.reactions} />
		{/if}
	</div>
</div>

{#snippet avatarFallback(sender?: string)}
	<AvatarFallback>{getInitials(sender ?? 'John Doe')}</AvatarFallback>
{/snippet}
