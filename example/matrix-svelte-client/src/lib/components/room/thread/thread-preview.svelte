<script lang="ts">
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import { Button } from '$lib/components/ui/button';
	import { gotoThread } from '$lib/utils.svelte';
	import { MessagesSquare } from '@lucide/svelte';
	import type { ThreadSummary } from 'tauri-plugin-matrix-svelte-api';

	let {
		threadSummary,
		rootId,
		roomId,
		roomAvatar
	}: { threadSummary: ThreadSummary; rootId: string; roomId: string; roomAvatar: string | null } =
		$props();
</script>

<Button
	onclick={() => gotoThread(roomId, rootId, roomAvatar)}
	class="mt-1 flex w-full justify-start px-0.5"
	variant="ghost"
>
	<div class="relative">
		<MessagesSquare class="size-6" />
		<Badge class="absolute top-0 right-0 h-3.5 min-w-3.5 rounded-full px-0 font-mono tabular-nums"
			>{threadSummary.numReplies}</Badge
		>
	</div>
	<!-- the string is sanitized by the backend before being rendered -->
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	<p class="truncate text-sm">{@html threadSummary.eventFormattedSummary}</p>
</Button>
