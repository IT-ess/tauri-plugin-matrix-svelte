<script lang="ts">
	import type { ProfileStore, TimelineItem } from 'tauri-plugin-matrix-svelte-api';
	import MessageLike from './message-like.svelte';
	import Virtual from './virtual.svelte';

	type Props = {
		item: TimelineItem;
		roomId: string;
		currentUserId: string;
		profileStore: ProfileStore;
	};

	let { item, roomId, currentUserId, profileStore }: Props = $props();
</script>

{#if item.kind === 'msgLike'}
	<MessageLike
		data={item.data}
		timestamp={item.timestamp ?? 0}
		isOwn={item.isOwn}
		{roomId}
		eventId={item.eventId ?? ''}
		{currentUserId}
		{profileStore}
	/>
	<!-- eventId should always be defined in msgLike -->
{:else if item.kind === 'virtual'}
	<Virtual timestamp={item.timestamp} data={item.data} />
{:else}
	Nope
{/if}
