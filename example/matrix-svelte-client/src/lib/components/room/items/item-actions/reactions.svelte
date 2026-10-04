<script lang="ts">
	import {
		Tooltip,
		TooltipContent,
		TooltipProvider,
		TooltipTrigger
	} from '#lib/components/ui/tooltip/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { ReactionsByKeyBySender } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		reactions: ReactionsByKeyBySender;
		currentUserId: string;
		onToggle: (emoji: string) => void;
	};

	let { reactions, currentUserId, onToggle }: Props = $props();

	let reactionKeys = $derived(Object.keys(reactions));

	// Format users list for tooltip
	const formatUsersList = (users: string[]) => {
		if (users.length === 0) return '';
		let mapped = users.map(fullUserIdToName);
		if (mapped.length === 1) return mapped[0];
		if (mapped.length === 2) return `${mapped[0]} and ${mapped[1]}`;
		return `${mapped[0]}, ${mapped[1]} and ${mapped.length - 2} others`;
	};
	const fullUserIdToName = (user: string) => {
		const regex = /@(\w+):/;
		const matchArray = user.match(regex) ?? [];
		return matchArray[1];
	};
</script>

<TooltipProvider>
	{#each reactionKeys as reaction (reaction)}
		{@const users = Object.keys(reactions[reaction])}
		<Tooltip>
			<TooltipTrigger>
				{#snippet child({ props: triggerProps })}
					<Button
						{...triggerProps}
						variant={users.includes(currentUserId) ? 'secondary' : 'ghost'}
						size="xs"
						aria-label={`${reaction} ${formatUsersList(users)}`}
						onclick={() => onToggle(reaction)}
					>
						{reaction}
						{users.length}
					</Button>
				{/snippet}
			</TooltipTrigger>
			<TooltipContent>
				<p>{formatUsersList(users)}</p>
			</TooltipContent>
		</Tooltip>
	{/each}
</TooltipProvider>
