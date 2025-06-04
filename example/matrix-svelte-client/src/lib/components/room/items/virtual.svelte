<script lang="ts">
	import type { VirtualTimelineItem } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		timestamp?: number;
		data: VirtualTimelineItem;
	};

	let { timestamp, data }: Props = $props();

	// Format the date for the separator
	const formatDate = (timestamp: number) => {
		const date = new Date(timestamp);
		const now = new Date();
		const yesterday = new Date(now);
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
				{formatDate(timestamp ?? 0)} at {formatTime(timestamp ?? 0)}
			</div>
		</div>
	</div>
{:else if data.kind === 'timelineStart'}
	<p>No more messages</p>
{:else if data.kind === 'readMarker'}
	<p>Read</p>
{/if}
