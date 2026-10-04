<script lang="ts">
	import { Popover, PopoverContent, PopoverTrigger } from '#lib/components/ui/popover/index.js';
	import { Tooltip, TooltipContent, TooltipProvider } from '#lib/components/ui/tooltip/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Menu, ReplyIcon, SmilePlusIcon } from '@lucide/svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { MessageAbility, ReactionsByKeyBySender } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		isOwn: boolean;
		handleAddReaction: (emoji: string) => Promise<void>;
		handleReply: () => void;
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
		commonEmojis,
		reactions,
		currentUserId,
		handleShowdropdown,
		abilities
	}: Props = $props();

	let reactionsArray = $derived(Object.keys(reactions));
</script>

<!-- Shown on message hover, and kept visible while the emoji picker is open -->
<div
	class={[
		// Buttons have `transition-all`, so they keep painting while their inherited visibility
		// transitions out: the pill must transition too, or it vanishes before its icons
		'bg-background flex items-center gap-1 rounded-full border p-0.5 shadow-sm transition-[opacity,visibility]',
		isOwn && 'flex-row-reverse',
		!isReactionPopoverOpen &&
			'invisible opacity-0 group-hover/message:visible group-hover/message:opacity-100'
	]}
>
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
</div>
