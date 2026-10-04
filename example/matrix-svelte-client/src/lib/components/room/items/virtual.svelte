<script lang="ts">
	import * as Marker from '#lib/components/ui/marker/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { SvelteDate } from 'svelte/reactivity';
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
	<Marker.Root variant="separator" class="text-xs">
		<Marker.Content>
			{formatDate(timestamp ?? 0)}
			{m.at()}
			{formatTime(timestamp ?? 0)}
		</Marker.Content>
	</Marker.Root>
{:else if data.kind === 'timelineStart'}
	<Marker.Root class="justify-center">
		<Marker.Content>{m.room_no_more_messages()}</Marker.Content>
	</Marker.Root>
{:else if data.kind === 'readMarker'}
	<Marker.Root variant="separator" class="text-primary before:bg-primary/80 after:bg-primary/80">
		<Marker.Content>{m.timeline_new_messages()}</Marker.Content>
	</Marker.Root>
{/if}
