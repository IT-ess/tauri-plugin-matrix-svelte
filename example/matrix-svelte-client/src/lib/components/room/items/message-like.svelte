<script lang="ts">
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import type { MsgLikeContent } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		data: MsgLikeContent;
		timestamp: number;
		isOwn: boolean;
	};

	let { data, timestamp, isOwn }: Props = $props();

	// Get initials for avatar fallback
	const getInitials = (name: string) => {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	};

	// Format timestamp
	const formatTime = (timestamp: number) => {
		return new Date(timestamp).toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	};
</script>

<div class={['flex gap-2', isOwn && 'flex-row-reverse']}>
	<Avatar>
		<!-- <AvatarImage src={message.sender.avatar} alt={message.sender.name} /> -->
		<AvatarFallback>{getInitials(data.sender ?? 'John Doe')}</AvatarFallback>
	</Avatar>

	<div
		class={[
			'max-w-[80%] rounded-lg p-3',
			isOwn ? 'bg-primary text-primary-foreground' : 'bg-muted'
		]}
	>
		<div class="flex items-center gap-2">
			<p class="text-sm font-medium">{data.sender}</p>
			<span class="text-xs opacity-70">{formatTime(timestamp ?? 0)}</span>
		</div>
		<p class="mt-1 text-sm">
			{data.body.body}
		</p>
	</div>
</div>
