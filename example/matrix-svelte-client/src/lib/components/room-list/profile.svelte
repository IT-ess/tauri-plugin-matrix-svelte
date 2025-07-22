<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index';
	import { buttonVariants } from '$lib/components/ui/button/index';
	import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
	import { cn, getInitials } from '$lib/utils';
	import type { ProfileStore } from 'tauri-plugin-matrix-svelte-api';
	import type { ClassValue } from 'clsx';

	type Props = {
		profileStore: ProfileStore;
		currentUserId: string;
	};
	let { profileStore, currentUserId }: Props = $props();
</script>

<Sheet.Root>
	<Sheet.Trigger
		class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'fixed top-4 right-4')}
		>{@render avatar('size-10')}</Sheet.Trigger
	>
	<Sheet.Content side="right">
		<Sheet.Header>
			<Sheet.Title>Profile</Sheet.Title>
			<Sheet.Description>Your current profile.</Sheet.Description>
		</Sheet.Header>
		<div class="grid flex-1 auto-rows-min gap-6 px-4">
			<div class="mx-auto">
				{@render avatar('size-36')}
				{#if profileStore.state[currentUserId]?.state === 'loaded'}
					<p
						class="mt-2 mb-8 scroll-m-20 pb-2 text-center text-2xl font-semibold tracking-tight transition-colors"
					>
						{profileStore.state[currentUserId].data.username}
					</p>
				{/if}
			</div>
		</div>
		<Sheet.Footer>
			<Sheet.Close class={buttonVariants({ variant: 'outline' })}>Close</Sheet.Close>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>

{#snippet avatar(size: ClassValue)}
	<Avatar class={cn('rounded-full border-2 border-blue-500', size)}>
		<!-- Reactive store, once the profile is loaded we load the image -->
		{#if profileStore.state[currentUserId]?.state === 'loaded' && profileStore.state[currentUserId].data.avatarDataUrl}
			<AvatarImage
				src={profileStore.state[currentUserId].data.avatarDataUrl}
				alt={profileStore.state[currentUserId]?.data.username}
			/>
		{:else if profileStore.state[currentUserId]?.state === 'loaded'}
			<AvatarFallback
				>{getInitials(profileStore.state[currentUserId]?.data.username)}</AvatarFallback
			>
		{:else}
			<AvatarFallback>?</AvatarFallback>
		{/if}
	</Avatar>
{/snippet}
