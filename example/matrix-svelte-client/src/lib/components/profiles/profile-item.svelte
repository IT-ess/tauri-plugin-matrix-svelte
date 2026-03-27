<script lang="ts">
	import { Avatar } from '$lib/components/ui/avatar';
	import { Card } from '$lib/components/ui/card';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { cn, gotoProfile } from '$lib/utils.svelte';
	import { fetchUserProfile } from 'tauri-plugin-matrix-svelte-api';
	import { Skeleton } from '../ui/skeleton';

	let {
		userId,
		selected
	}: {
		userId: string;
		selected?: boolean;
	} = $props();
</script>

{#await fetchUserProfile(userId, null)}
	<Skeleton class="h-12 w-full" />
{:then profile}
	<Card
		class={cn(
			'hover:bg-accent/50 cursor-pointer p-4 transition-colors',
			selected ? 'border-3 border-blue-600' : ''
		)}
	>
		<button class="flex items-center gap-3" onclick={() => gotoProfile(userId)}>
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
					ID: {userId}
				</p>
			</div>
		</button>
	</Card>
{:catch err}
	<p class="text-destructive">Error {err}</p>
{/await}
