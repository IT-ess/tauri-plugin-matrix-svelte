<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index';
	import { Button, buttonVariants } from '$lib/components/ui/button/index';
	import { Avatar } from '../ui/avatar';
	import {
		BellRing,
		MonitorSmartphone,
		ShieldAlert,
		ShieldQuestion,
		User,
		ShieldCheck,
		DatabaseBackup,
		LogOut,
		Pencil
	} from '@lucide/svelte';
	import { cn } from '$lib/utils.svelte';
	import Badge from '../ui/badge/badge.svelte';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import { m } from '$lib/paraglide/messages';
	import { toast } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { loginStore } from '../../../hooks.client';
	import { checkIfLastDevice, disconnectAndClearSession } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		currentInvitesNumber: number;
	};
	let { currentInvitesNumber }: Props = $props();

	const logoutAndGoToLogin = async () => {
		try {
			await disconnectAndClearSession();
		} catch (err) {
			toast.error(String(err));
		} finally {
			goto('/login');
		}
	};
</script>

<Sheet.Root>
	<Sheet.Trigger class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'relative mb-1')}>
		{@render avatar(false)}
	</Sheet.Trigger>
	<Sheet.Content side="right">
		<Sheet.Header class="mt-safe">
			<Sheet.Title>{m.profile_menu_profile()}</Sheet.Title>
			<Sheet.Description>{m.profile_menu_subtitle()}</Sheet.Description>
		</Sheet.Header>
		<div class="grid flex-1 auto-rows-min gap-6 px-4">
			<div class="mx-auto">
				{@render avatar(true)}
				<p
					class="mt-2 mb-8 scroll-m-20 pb-2 text-center text-2xl font-semibold tracking-tight transition-colors"
				>
					{loginStore.state.userDisplayName}
				</p>
			</div>
			<div class="flex w-full flex-col gap-2">
				{#if loginStore.state.verificationState !== 'verified'}
					<Button class="w-full" size="lg" variant="destructive" href="/verification"
						><ShieldCheck />{m.profile_menu_verify_device()}</Button
					>
				{/if}
				{#if loginStore.state.recoveryState !== 'enabled' && loginStore.state.verificationState === 'verified'}
					<Button class="w-full" size="lg" variant="destructive" href="/settings/recovery"
						><DatabaseBackup />{m.profile_menu_enable_recovery()}</Button
					>
				{/if}
				<Button class="w-full" size="lg" variant="outline" href="/devices"
					><MonitorSmartphone />{m.profile_menu_devices()}</Button
				>
				<Button class="relative w-full" size="lg" variant="outline" href="/profiles"
					><User />{m.profile_menu_contacts()}
					{#if currentInvitesNumber > 0}
						<Badge class="absolute -top-1 -right-1">{currentInvitesNumber}</Badge>
					{/if}</Button
				>
				<Button class="w-full" size="lg" variant="outline" href="/settings/notification"
					><BellRing />{m.profile_menu_notifications()}</Button
				>
				<AlertDialog.Root>
					<AlertDialog.Trigger
						class={buttonVariants({ variant: 'outline', size: 'lg', class: 'w-full' })}
					>
						<LogOut class="text-destructive" />{m.logout()}
					</AlertDialog.Trigger>
					<AlertDialog.Content>
						<AlertDialog.Header>
							<AlertDialog.Title>{m.logout_are_you_sure()}</AlertDialog.Title>
							<AlertDialog.Description>
								{m.logout_are_you_sure_desc()}
							</AlertDialog.Description>
						</AlertDialog.Header>
						{#await checkIfLastDevice()}
							<AlertDialog.Footer>
								<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
								<AlertDialog.Action disabled>{m.logout()}</AlertDialog.Action>
							</AlertDialog.Footer>
						{:then res}
							{#if res && loginStore.state.recoveryState !== 'enabled'}
								<span class="font-extrabold text-red-500"
									>{m.logout_are_you_sure_last_device_warn()}
									<a class="underline" href="/settings/recovery"
										>{m.logout_please_setup_backup_before()}</a
									></span
								>
								<AlertDialog.Footer>
									<AlertDialog.Action onclick={() => goto('/settings/recovery')}
										>{m.setup_backup()}</AlertDialog.Action
									>
									<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
									<AlertDialog.Action
										class={buttonVariants({ variant: 'destructive' })}
										onclick={logoutAndGoToLogin}>{m.logout_anyway()}</AlertDialog.Action
									>
								</AlertDialog.Footer>
							{:else}
								<AlertDialog.Footer>
									<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
									<AlertDialog.Action onclick={logoutAndGoToLogin}>{m.logout()}</AlertDialog.Action>
								</AlertDialog.Footer>
							{/if}
						{/await}
					</AlertDialog.Content>
				</AlertDialog.Root>
			</div>
		</div>
	</Sheet.Content>
</Sheet.Root>

{#snippet avatar(isBig: boolean)}
	<div class="relative">
		<Avatar class={cn('border-primary rounded-full border-2', isBig ? 'size-36' : 'size-10')}>
			{#if loginStore.state.userAvatar}
				{@render fetchAvatar(loginStore.state.userAvatar, loginStore.state.userDisplayName ?? '?')}
			{/if}
			{@render avatarFallback(loginStore.state.userDisplayName)}
		</Avatar>
		<div class="absolute -bottom-1 -left-1 flex">
			{@render verificationState(isBig)}
		</div>
		{#if isBig}
			<a
				href="/settings/profile"
				class="shadow-lg-lg-md hover:shadow-lg-lg-lg bg-primary text-primary-foreground absolute right-0 bottom-0 flex size-8 items-center justify-center rounded-full transition-all duration-200 hover:scale-110 disabled:opacity-50"
				title="Change avatar"
			>
				<Pencil class="size-4" />
			</a>
		{/if}
		{#if currentInvitesNumber > 0 && !isBig}
			<Badge variant="default" class="absolute -right-1 -bottom-1">{currentInvitesNumber}</Badge>
		{/if}
	</div>
{/snippet}

{#snippet verificationState(isBig: boolean)}
	{#if loginStore.state.verificationState === 'verified'}
		<!-- We no longer display the shield as it should be the norm -->
		<!-- <ShieldUser
			class={cn('rounded-3xl bg-background/60 text-green-700', isBig ? 'size-12' : 'size-5')}
		/> -->
	{:else if loginStore.state.verificationState === 'unverified'}
		<ShieldAlert
			class={cn('bg-background/60 rounded-3xl text-red-500', isBig ? 'size-12' : 'size-5')}
		/>
	{:else}
		<ShieldQuestion
			class={cn('bg-background/60 rounded-3xl text-slate-500', isBig ? 'size-12' : 'size-5')}
		/>
	{/if}
{/snippet}
