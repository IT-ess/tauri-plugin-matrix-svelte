<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Card } from '$lib/components/ui/card';
	import { cn, getCustomMxcUriFromOriginal, getInitials } from '$lib/utils.svelte';
	import type { ProfileModel } from 'tauri-plugin-matrix-svelte-api';

	let {
		profile,
		onSelect,
		selected
	}: {
		profile: ProfileModel;
		onSelect: (user: ProfileModel) => void;
		selected?: boolean;
	} = $props();

	let alt = $derived(profile.displayName ?? '?');
</script>

<Card
	class={cn(
		'hover:bg-accent/50 cursor-pointer p-4 transition-colors',
		selected ? 'border-3 border-blue-600' : ''
	)}
>
	<button class="flex items-center gap-3" onclick={() => onSelect(profile)}>
		<Avatar class="size-10">
			<AvatarImage src={getCustomMxcUriFromOriginal(profile.avatarUrl)} {alt} />
			<AvatarFallback>{getInitials(alt)}</AvatarFallback>
		</Avatar>
		<div class="min-w-0 flex-1">
			<h3 class="text-foreground truncate font-medium">
				{profile.displayName}
			</h3>
			<p class="text-muted-foreground truncate text-sm">
				ID: {profile.userId}
			</p>
		</div>
	</button>
</Card>
