<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import { Card } from '$lib/components/ui/card';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { checkUserInProfileStore, cn } from '$lib/utils.svelte';
	import { onMount } from 'svelte';
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

	onMount(async () => {
		await checkUserInProfileStore(profile.userId);
	});
</script>

<Card
	class={cn(
		'hover:bg-accent/50 cursor-pointer p-4 transition-colors',
		selected ? 'border-3 border-blue-600' : ''
	)}
>
	<button class="flex items-center gap-3" onclick={() => onSelect(profile)}>
		<Avatar class="h-10 w-10">
			{#if profile.avatarUrl}
				{@render fetchAvatar(profile.avatarUrl, profile.displayName ?? '?')}
			{/if}
			{@render avatarFallback(profile.displayName)}
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
