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
{:else if item.kind === 'call'}
	<!-- TODO: add styling  -->
	<p class="text-muted text-center">Someone started a call</p>
{:else if item.kind === 'stateChange'}
	{#if item.data.kind === 'profileChange'}
		<p class="text-center text-slate-400">{item.data.body.user_id} profile update</p>
	{:else if item.data.kind === 'membershipChange'}
		<p class="text-center text-slate-400">
			{item.data.body.content.content.displayname} membership update: {item.data.body.content
				.content.membership}
		</p>
	{:else if item.data.kind === 'otherState'}
		<p class="text-center text-slate-400">State change: {Object.keys(item.data.body)[0]}</p>
		<!-- TODO: implement full mapping -->
	{/if}
{:else if item.kind === 'error'}
	<p class="text-destructivetext-center">Received error: {item.data.error}</p>
{/if}
