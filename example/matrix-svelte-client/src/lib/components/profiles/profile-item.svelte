<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Card } from '$lib/components/ui/card';
	import { cn, getCustomMxcUriFromOriginal, getInitials, gotoProfile } from '$lib/utils.svelte';
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
			<Avatar class="size-10">
				<AvatarImage
					src={getCustomMxcUriFromOriginal(profile.avatarUrl)}
					alt={profile.displayName ?? '?'}
				/>
				<AvatarFallback>{getInitials(profile.displayName ?? '?')}</AvatarFallback>
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
