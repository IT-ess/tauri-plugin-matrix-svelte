<script lang="ts">
	import type { Device } from 'tauri-plugin-matrix-svelte-api';
	import DeviceItem from './device-item.svelte';

	let { data } = $props();

	function handleVerifyDevice(device: Device) {
		console.log('Verifying device:', device.deviceId);
		// Handle device verification logic here
	}

	function sortDevices(device: Device[]): Device[] {
		let current = device.find((d) => d.isCurrentDevice);
		if (!current) return device;
		let sorted = device
			.filter((d) => d.deviceId !== current.deviceId)
			.sort(
				(a, b) => new Date(b.registrationDate).getTime() - new Date(a.registrationDate).getTime()
			);
		sorted.unshift(current);
		return sorted;
	}
</script>

<div class="bg-background text-foreground min-h-screen">
	<div class="container mx-auto px-4 py-8">
		<div class="mx-auto max-w-2xl">
			<div class="mb-6">
				<h1 class="mb-2 text-2xl font-bold">Device Management</h1>
				<p class="text-muted-foreground">Manage and verify your connected devices</p>
			</div>
			<div class="space-y-2">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Connected Devices</h2>
					<span class="text-muted-foreground text-sm">{data.devices.length} devices</span>
				</div>

				{#each sortDevices(data.devices) as device (device.deviceId)}
					<DeviceItem {device} onVerify={() => handleVerifyDevice(device)} />
				{/each}
			</div>
		</div>
	</div>
</div>
