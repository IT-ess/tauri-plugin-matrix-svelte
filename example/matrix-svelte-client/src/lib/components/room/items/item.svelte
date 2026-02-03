<script lang="ts">
	import type { TimelineItem } from 'tauri-plugin-matrix-svelte-api';
	import { roomsCollection } from '../../../../hooks.client';
	import MessageLike from './message-like.svelte';
	import Virtual from './virtual.svelte';

	type Props = {
		item: TimelineItem;
		roomId: string;
		currentUserId: string;
		repliedToMessage?: TimelineItem;
		onReply?: (eventId: string, senderName: string, content: string) => void;
		onScrollToMessage?: (eventId: string) => void;
		handleOpenMediaViewMode: (
			type: 'image' | 'video',
			src: string,
			info: {
				filename?: string;
				body?: string;
				size: number;
			}
		) => void;
		isInThread?: boolean;
		roomAvatar: string | null;
	};

	let {
		item,
		roomId,
		currentUserId,
		onReply,
		repliedToMessage,
		onScrollToMessage,
		handleOpenMediaViewMode,
		isInThread,
		roomAvatar
	}: Props = $props();
</script>

{#if item.kind === 'msgLike'}
	<!-- msg like always have event ids -->
	<div data-event-id={item.eventId as string}>
		<MessageLike
			data={item.data}
			timestamp={item.timestamp ?? 0}
			isOwn={item.isOwn}
			{roomId}
			eventId={item.eventId ?? ''}
			timelineItemId={item.timelineItemId}
			isLocal={item.isLocal}
			{currentUserId}
			{onReply}
			{onScrollToMessage}
			repliedToMessage={repliedToMessage?.kind === 'msgLike' ? repliedToMessage.data : undefined}
			abilities={item.abilities}
			{handleOpenMediaViewMode}
			{isInThread}
			{roomAvatar}
		/>
	</div>
{:else if item.kind === 'virtual'}
	<Virtual
		timestamp={item.timestamp ?? undefined}
		data={item.data}
		roomHasUnreadMessages={roomsCollection.state.allJoinedRooms[roomId].numUnreadMessages > 0}
	/>
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
