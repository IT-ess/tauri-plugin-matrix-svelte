<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index';
	import { buttonVariants } from '$lib/components/ui/button/index';
	import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar';
	import { cn, getInitials } from '$lib/utils';
	import type { LoginStore, ProfileStore } from 'tauri-plugin-matrix-svelte-api';
	import { ShieldAlert, ShieldQuestion, ShieldUser } from '@lucide/svelte';

	type Props = {
		profileStore: ProfileStore;
		currentUserId: string;
		loginStore: LoginStore;
	};
	let { profileStore, currentUserId, loginStore }: Props = $props();
</script>

<Sheet.Root>
	<Sheet.Trigger
		class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'top-safe-or-4 fixed right-4')}
	>
		{@render avatar(false)}
	</Sheet.Trigger>
	<Sheet.Content side="right">
		<Sheet.Header class="mt-safe">
			<Sheet.Title>Profile</Sheet.Title>
			<Sheet.Description>Your current profile.</Sheet.Description>
		</Sheet.Header>
		<div class="grid flex-1 auto-rows-min gap-6 px-4">
			<div class="mx-auto">
				{@render avatar(true)}
				{#if profileStore.state[currentUserId]?.state === 'loaded'}
					<p
						class="mt-2 mb-8 scroll-m-20 pb-2 text-center text-2xl font-semibold tracking-tight transition-colors"
					>
						{profileStore.state[currentUserId].data.username}
					</p>
				{/if}
			</div>
			<div>
				<a href="/devices">Devices</a>
			</div>
		</div>
		<Sheet.Footer>
			<Sheet.Close class={buttonVariants({ variant: 'outline' })}>Close</Sheet.Close>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>

{#snippet avatar(isBig: boolean)}
	<div class="relative">
		<Avatar class={cn('rounded-full border-2 border-blue-500', isBig ? 'size-36' : 'size-10')}>
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
		<div class="absolute -bottom-1 -left-1 flex">
			{@render verificationState(isBig)}
		</div>
	</div>
{/snippet}

{#snippet verificationState(isBig: boolean)}
	{#if loginStore.state.verificationState === 'verified'}
		<ShieldUser
			class={cn('bg-background rounded-3xl text-green-700', isBig ? 'size-12' : 'size-5')}
		/>
	{:else if loginStore.state.verificationState === 'unverified'}
		<ShieldAlert
			class={cn('bg-background rounded-3xl text-red-500', isBig ? 'size-12' : 'size-5')}
		/>
	{:else}
		<ShieldQuestion
			class={cn('bg-background rounded-3xl text-slate-500', isBig ? 'size-12' : 'size-5')}
		/>
	{/if}
{/snippet}
