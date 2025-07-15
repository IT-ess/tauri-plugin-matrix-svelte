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
	import { SmileIcon } from '@lucide/svelte';
	import ImageMessage from './image-message.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type Props = {
		data: MsgLikeContent;
		timestamp: number;
		isOwn: boolean;
		roomId: string;
		eventId: string;
		currentUserId: string;
		profileStore: ProfileStore;
	};

	let { data, timestamp, isOwn, roomId, eventId, currentUserId, profileStore }: Props = $props();

	const { senderId, sender } = data;

	let reactionsArray = $derived(Object.keys(data.reactions));

	// Get initials for avatar fallback
	const getInitials = (name: string) => {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	};

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

	onMount(async () => {
		if (profileStore.state[senderId] === undefined) {
			await invoke('plugin:matrix-svelte|fetch_user_profile', {
				userId: senderId,
				roomId
			});
		}
	});
</script>

<div class={['flex gap-2', isOwn && 'flex-row-reverse']}>
	<Avatar>
		<!-- Reactive store, once the profile is loaded we load the image -->
		{#if profileStore.state[senderId]?.state === 'loaded' && profileStore.state[senderId].data.avatarDataUrl}
			<AvatarImage src={profileStore.state[senderId].data.avatarDataUrl} alt={sender} />
		{:else}
			{@render avatarFallback(sender)}
		{/if}
	</Avatar>

	<div
		class={[
			'max-w-[80%] rounded-lg p-3',
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
		{:else if data.kind === 'image'}
			<ImageMessage itemContent={data.body} />
		{/if}
	</div>

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
	<div class={['flex items-center gap-2', isOwn && 'flex-row-reverse']}>
		{#if reactionsArray.length > 0}
			<Reactions reactions={data.reactions} />
		{/if}
	</div>
</div>

{#snippet avatarFallback(sender?: string)}
	<AvatarFallback>{getInitials(sender ?? 'John Doe')}</AvatarFallback>
{/snippet}
