<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { SvelteDate } from 'svelte/reactivity';
	import type { VirtualTimelineItem } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		timestamp?: number;
		data: VirtualTimelineItem;
		roomHasUnreadMessages: boolean;
	};

	let { timestamp, data, roomHasUnreadMessages }: Props = $props();

	// Format the date for the separator
	const formatDate = (timestamp: number) => {
		const date = new Date(timestamp);
		const now = new Date();
		const yesterday = new SvelteDate(now);
		yesterday.setDate(yesterday.getDate() - 1);

		// If it's today
		if (date.toDateString() === now.toDateString()) {
			return 'Today';
		}
		// If it's yesterday
		else if (date.toDateString() === yesterday.toDateString()) {
			return 'Yesterday';
		}
		// If it's within the last 7 days
		else if (now.getTime() - date.getTime() < 7 * 24 * 60 * 60 * 1000) {
			return date.toLocaleDateString(undefined, { weekday: 'long' });
		}
		// If it's this year
		else if (date.getFullYear() === now.getFullYear()) {
			return date.toLocaleDateString(undefined, {
				month: 'long',
				day: 'numeric'
			});
		}
		// If it's a different year
		else {
			return date.toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'long',
				day: 'numeric'
			});
		}
	};

	// Format the time
	const formatTime = (timestamp: number) => {
		return new Date(timestamp).toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	};
</script>

{#if data.kind === 'dateDivider'}
	<div class="relative my-6">
		<div class="absolute inset-0 flex items-center">
			<div class="w-full border-t"></div>
		</div>
		<div class="relative flex justify-center">
			<div class="bg-background text-muted-foreground px-2 text-xs">
				{formatDate(timestamp ?? 0)}
				{m.at()}
				{formatTime(timestamp ?? 0)}
			</div>
		</div>
	</div>
{:else if data.kind === 'timelineStart'}
	<p class="text-muted-foreground text-center text-sm">{m.room_no_more_messages()}</p>
{:else if data.kind === 'readMarker' && roomHasUnreadMessages}
	<div class="border-primary/80 w-full border-t"></div>
{/if}
