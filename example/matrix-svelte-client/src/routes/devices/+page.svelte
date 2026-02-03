<script lang="ts">
	import DeviceItem from './device-item.svelte';
	import { ChevronLeft } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { LoaderCircle, ShieldCheck, ShieldX } from '@lucide/svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Channel } from '@tauri-apps/api/core';
	import { Button } from '$lib/components/ui/button';
	import { emit } from '@tauri-apps/api/event';
	import { MatrixSvelteEmitEvent, verifyDevice } from 'tauri-plugin-matrix-svelte-api';
	import type { FrontendDevice as Device, VerifyDeviceEvent } from 'tauri-plugin-matrix-svelte-api';
	import { gotoRoomsList } from '$lib/utils.svelte';

	let { data } = $props();

	let displayVerificationModal = $state(false);
	let verificationDone = $state(false);
	let verificationCancelledReason = $state<string | undefined>();

	async function handleVerifyDevice(device: Device, verifiedCallback: () => void) {
		console.log('Verifying device:', device.deviceId);
		verificationDone = false;
		verificationCancelledReason = undefined;

		const onEvent = new Channel<VerifyDeviceEvent>();
		onEvent.onmessage = (message) => {
			switch (message.event) {
				case 'requested':
					console.log('Received requested');
					displayVerificationModal = true;
					break;
				case 'done':
					console.log('Verification is done');
					verificationDone = true;
					verifiedCallback();
					break;
				case 'cancelled':
					console.log('Verification is cancelled');
					verificationCancelledReason = message.data.reason;
					break;
			}
		};

		await verifyDevice(onEvent, data.userId, device.deviceId);
	}

	function sortDevices(device: Device[]): Device[] {
		let current = device.find((d) => d.isCurrentDevice);
		if (!current) return device;
		let sorted = device
			.filter((d) => d.deviceId !== current.deviceId && d.lastSeenTs)
			.sort(
				(a, b) =>
					new Date(b.lastSeenTs as number).getTime() - new Date(a.lastSeenTs as number).getTime()
			);
		sorted.unshift(current);
		return sorted;
	}
</script>

<div class="bg-background pt-safe pb-safe text-foreground min-h-screen">
	<div class="container mx-auto px-4 py-8">
		<div class="mx-auto max-w-2xl">
			<button
				onclick={() => gotoRoomsList('dm')}
				class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
				aria-label="Go back"
			>
				<ChevronLeft class="text-foreground h-6 w-6" />
			</button>
			<div class="mb-6 text-center">
				<h1 class="mb-2 text-2xl font-bold">{m.profile_menu_devices()}</h1>
				<p class="text-muted-foreground">{m.devices_subtitle()}</p>
			</div>
			<div class="space-y-2">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">{m.devices_connected_devices()}</h2>
					<span class="text-muted-foreground text-sm"
						>{data.devices.length} {m.profile_menu_devices().toLowerCase()}</span
					>
				</div>

				{#each sortDevices(data.devices) as device (device.deviceId)}
					<DeviceItem
						{device}
						onVerify={(verifiedCallback) => handleVerifyDevice(device, verifiedCallback)}
					/>
				{/each}
			</div>
		</div>
	</div>
</div>

<AlertDialog.Root bind:open={displayVerificationModal}>
	<AlertDialog.Content class="max-w-[80%] rounded-md">
		<AlertDialog.Header>
			<AlertDialog.Title>A verification has been requested</AlertDialog.Title>
			<AlertDialog.Description>
				{#if verificationCancelledReason}
					<div class="flex flex-col items-center text-center">
						<span class="mb-2"
							>{m.popup_verification_cancelled()} {verificationCancelledReason}
						</span><ShieldX class="h-10 w-10 text-red-600" />
					</div>
				{:else if verificationDone}
					<div class="flex flex-col items-center text-center">
						<span class="mb-2">{m.popup_verification_done()}</span>
						<ShieldCheck class="h-10 w-10 text-green-600" />
					</div>
				{:else}
					<div class="flex flex-col items-center text-center">
						<span class="mb-2">{m.popup_verification_awaiting()}</span>
						<LoaderCircle class="text-primary h-10 w-10 animate-spin" />
					</div>
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			{#if verificationDone || verificationCancelledReason}
				<Button
					type="submit"
					onclick={() => {
						displayVerificationModal = false;
					}}>{m.button_close()}</Button
				>
			{:else}
				<Button
					variant="destructive"
					type="submit"
					onclick={() => {
						emit(MatrixSvelteEmitEvent.CancelVerification, {});
						displayVerificationModal = false;
					}}>{m.button_cancel()}</Button
				>
			{/if}
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
