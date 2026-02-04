<script lang="ts">
	import Reactions from './reactions.svelte';
	import { Popover, PopoverContent, PopoverTrigger } from '$lib/components/ui/popover';
	import { Tooltip, TooltipContent, TooltipProvider } from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
	import { Menu, ReplyIcon, SmilePlusIcon } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import type { MessageAbility, ReactionsByKeyBySender } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		isOwn: boolean;
		handleAddReaction: (emoji: string) => Promise<void>;
		handleReply: () => void;
		showActions: boolean;
		commonEmojis: string[];
		reactions: ReactionsByKeyBySender;
		currentUserId: string;
		handleShowdropdown: () => void;
		abilities: MessageAbility[];
	};

	let isReactionPopoverOpen = $state(false);

	let {
		isOwn,
		handleAddReaction,
		handleReply,
		showActions = $bindable(),
		commonEmojis,
		reactions,
		currentUserId,
		handleShowdropdown,
		abilities
	}: Props = $props();

	let reactionsArray = $derived(Object.keys(reactions));
</script>

<div class={['flex items-center gap-1', isOwn && 'flex-row-reverse']}>
	{#if (showActions && handleReply) || isReactionPopoverOpen}
		<!-- Other actions -->
		<TooltipProvider>
			<Tooltip>
				<Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => handleShowdropdown()}>
					<Menu class="size-4" />
				</Button>
				<TooltipContent>Other Actions</TooltipContent>
			</Tooltip>
		</TooltipProvider>
		{#if abilities.includes('canReplyTo')}
			<!-- Reply button -->
			<TooltipProvider>
				<Tooltip>
					<Button variant="ghost" size="icon" class="h-6 w-6" onclick={handleReply}>
						<ReplyIcon class="h-4 w-4" />
					</Button>
					<TooltipContent>{m.button_reply()}</TooltipContent>
				</Tooltip>
			</TooltipProvider>
		{/if}
		<!-- Reaction button -->
		{#if abilities.includes('canReact')}
			<TooltipProvider>
				<Popover bind:open={isReactionPopoverOpen}>
					<Tooltip>
						<PopoverTrigger>
							{#snippet child({ props: triggerProps })}
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6"
									onclick={() => (isReactionPopoverOpen = true)}
								>
									<SmilePlusIcon {...triggerProps} class="h-4 w-4" />
								</Button>
							{/snippet}
							<TooltipContent>Add reaction</TooltipContent>
						</PopoverTrigger>
					</Tooltip>
					<PopoverContent class="w-fit p-2">
						<div class="flex gap-1">
							{#each commonEmojis as emoji (emoji)}
								<Button
									variant={reactionsArray.includes(emoji)
										? Object.keys(reactions[emoji]).includes(currentUserId)
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
		{/if}
	{/if}

	{#if reactionsArray.length > 0}
		<Reactions {reactions} />
	{/if}
</div>
