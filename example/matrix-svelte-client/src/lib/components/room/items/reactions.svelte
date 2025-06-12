<script lang="ts">
	import {
		Tooltip,
		TooltipContent,
		TooltipProvider,
		TooltipTrigger
	} from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
	import type { ReactionsByKeyBySender } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		reactions: ReactionsByKeyBySender;
	};

	let { reactions }: Props = $props();

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

<div class="flex flex-wrap gap-1">
	{#each reactionKeys as reaction (reaction)}
		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					{#snippet child({ props: triggerProps })}
						<Button variant="secondary" size="sm" class="h-6 gap-1 px-2 text-xs" {...triggerProps}>
							<span>{reaction}</span>
							<span class="font-medium">{formatUsersList(Object.keys(reactions[reaction]))}</span>
						</Button>
					{/snippet}
				</TooltipTrigger>
				<TooltipContent>
					<p>{Object.keys(reactions[reaction])}</p>
				</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	{/each}
</div>
