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
	{@render stateMessage('Someone started a call')}
{:else if item.kind === 'stateChange'}
	{#if item.data.kind === 'profileChange'}
		{@render stateMessage(`${item.data.body.user_id} profile update`)}
	{:else if item.data.kind === 'membershipChange'}
		{@render stateMessage(
			`${item.data.body.content.content.displayname} membership update: ${
				item.data.body.content.content.membership
			}`
		)}
	{:else if item.data.kind === 'otherState'}
		{@render stateMessage(`State change: ${Object.keys(item.data.body)[0]}`)}
		<!-- TODO: implement full mapping -->
	{/if}
{:else if item.kind === 'error'}
	<p class="text-destructive text-center">Received error: {item.data.error}</p>
{/if}

{#snippet stateMessage(text: string)}
	<p class="text-center text-sm text-slate-400">{text}</p>
{/snippet}
