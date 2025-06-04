<script lang="ts">
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import { Badge } from '$lib/components/ui/badge';
	import type { JoinedRoomInfo } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		room: JoinedRoomInfo;
		onRoomClick: (id: string) => undefined;
		disabled: boolean;
	};

	let { room, onRoomClick, disabled }: Props = $props();

	// Get initials from name for avatar fallback
	const getInitials = (name: string) => {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	};

	const getLocalTimeAsFormattedString = (timestamp: number) => {
		const date = new Date(timestamp);
		const month = String(date.getMonth() + 1).padStart(2, '0'); // Months are 0-indexed
		const day = String(date.getDate()).padStart(2, '0');
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		return `${hours}h${minutes} ${day}-${month}`;
	};

	let latestEvent = $derived(room.latest ? room.latest[1] : 'Placeholder for last message');
</script>

<div
	class={{
		'flex items-center gap-4 p-4 transition-colors': true,
		'hover:bg-muted/50 cursor-pointer': !disabled,
		'cursor-not-allowed opacity-50': disabled
	}}
	role="button"
	tabindex={disabled ? -1 : 0}
>
	<!-- onclick={onClick}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === "Space") {
      onClick?.();
    }}} -->
	<Avatar>
		<!-- <AvatarImage src={room.state.avatar} alt={room.state.name} /> -->
		<AvatarFallback>{getInitials(room.roomName)}</AvatarFallback>
	</Avatar>
	<button {disabled} class="flex-1 space-y-1" onclick={() => onRoomClick(room.roomId)}>
		<div class="flex items-center justify-between">
			<h4 class="font-semibold">{room.roomName}</h4>
			<span class="text-muted-foreground text-sm"
				>{getLocalTimeAsFormattedString(room.latest ? room.latest[0] : 0)}</span
			>
		</div>
		<div class="flex items-center justify-between">
			<p class="text-muted-foreground line-clamp-1 text-sm">
				{@html latestEvent}
			</p>
			{#if room.numUnreadMessages > 0}
				<Badge variant="default" class="ml-2">{room.numUnreadMessages}</Badge>
			{/if}
		</div>
	</button>
</div>
