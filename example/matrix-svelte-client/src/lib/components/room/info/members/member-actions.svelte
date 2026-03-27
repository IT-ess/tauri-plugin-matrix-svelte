<script lang="ts">
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import { m } from '$lib/paraglide/messages';
	import { avatarFallback, fetchAvatar } from '$lib/snippets.svelte';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import { Button } from '$lib/components/ui/button';
	import { Ban, User, X } from '@lucide/svelte';
	import { gotoProfile } from '$lib/utils.svelte';
	import {
		createMatrixRequest,
		submitAsyncRequest,
		type FrontendRoomMember,
		type TimelineUiState
	} from 'tauri-plugin-matrix-svelte-api';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';

	type ClickedUser = {
		id: string;
		name: string;
		avatar: string | null;
		role: FrontendRoomMember['role'];
	};
	let {
		openMemberActions = $bindable(false),
		clickedUser,
		roomId,
		userPower
	}: {
		openMemberActions: boolean;
		clickedUser: ClickedUser;
		roomId: string;
		userPower: TimelineUiState['userPower'] | undefined;
	} = $props();

	let openKickOrBan = $state(false);
	let isBan = $state(false);
	const openKickOrBanAlert = (localIsBan: boolean) => {
		openKickOrBan = true;
		isBan = localIsBan;
	};
	let kickOrBanReason = $state('');

	const handleKickOrBan = async () => {
		const request = createMatrixRequest.kickOrBanUserFromRoom({
			isBan,
			roomId,
			userId: clickedUser.id,
			reason: kickOrBanReason.trim() == '' ? null : kickOrBanReason.trim()
		});
		await submitAsyncRequest(request);
		openKickOrBan = false;
		openMemberActions = false;
	};
</script>

<Drawer.Root bind:open={openMemberActions}>
	<Drawer.Content class="pb-safe">
		<Drawer.Header class="my-5 items-center gap-3">
			<Avatar.Root class="size-16">
				{#if clickedUser.avatar}
					{@render fetchAvatar(clickedUser.avatar, clickedUser.name)}
				{/if}
				{@render avatarFallback(clickedUser.name)}
			</Avatar.Root>
			<h2 class="mt-2 text-lg font-semibold">{clickedUser.name}</h2>
		</Drawer.Header>
		<Drawer.Footer class="gap-2">
			<Button
				onclick={() => gotoProfile(clickedUser.id)}
				class="h-12 w-full justify-baseline text-lg font-normal"
				variant="ghost"
				><User class="mr-2 size-5" />Voir le profil
			</Button>
			<!-- TODO: call real can_ban() and can_kick() functions -->
			{#if (userPower?.includes('kick') && clickedUser.role == 'moderator') || clickedUser.role == 'user'}
				<Button
					onclick={() => openKickOrBanAlert(false)}
					class="bg-background h-12 w-full justify-baseline text-lg font-normal"
					variant="destructive"
					><X class="mr-2 size-5" />{m.kick()}
				</Button>
			{/if}
			{#if (userPower?.includes('ban') && clickedUser.role == 'moderator') || clickedUser.role == 'user'}
				<Button
					onclick={() => openKickOrBanAlert(true)}
					class="bg-background h-12 w-full justify-baseline text-lg font-normal"
					variant="destructive"
					><Ban class="mr-2 size-5" />{m.ban()}
				</Button>
			{/if}
		</Drawer.Footer>
	</Drawer.Content>
</Drawer.Root>

<AlertDialog.Root bind:open={openKickOrBan}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{m.logout_are_you_sure()}</AlertDialog.Title>
		</AlertDialog.Header>
		<div>
			<Label for="reason" class="text-base font-semibold">{m.reason()}</Label>
			<Input
				id="reason"
				type="text"
				placeholder={m.optional()}
				bind:value={kickOrBanReason}
				maxlength={80}
				class="text-base"
			/>
		</div>

		<AlertDialog.Footer class="gap-4">
			<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" onclick={handleKickOrBan}
				>{m.button_confirm()}</AlertDialog.Action
			>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
