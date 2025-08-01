<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Popover, PopoverContent, PopoverTrigger } from '$lib/components/ui/popover';
	import { Separator } from '$lib/components/ui/separator';
	import { MoreVerticalIcon, ShieldCheckIcon, TrashIcon, InfoIcon } from '@lucide/svelte';
	import type { Device } from 'tauri-plugin-matrix-svelte-api';

	interface Props {
		device: Device;
		onVerify: () => void;
	}

	let { device, onVerify }: Props = $props();
	let isOpen = $state(false);

	function handleVerify() {
		onVerify();
		isOpen = false;
	}

	function handleViewDetails() {
		console.log('View device details:', device.deviceId);
		// TODO
		isOpen = false;
	}

	function handleRemoveDevice() {
		console.log('Remove device:', device.deviceId);
		// TODO
		isOpen = false;
	}
</script>

<Popover bind:open={isOpen}>
	<PopoverTrigger>
		{#snippet child({ props })}
			<Button variant="ghost" size="sm" class="h-8 w-8 p-0" {...props}>
				<MoreVerticalIcon class="h-4 w-4" />
				<span class="sr-only">Open device actions</span>
			</Button>
		{/snippet}
	</PopoverTrigger>
	<PopoverContent class="w-48 p-1" align="end">
		<div class="space-y-1">
			<!-- Verify Device Action -->
			{#if !device.isVerified}
				<button
					class="hover:bg-accent hover:text-accent-foreground flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
					onclick={handleVerify}
				>
					<ShieldCheckIcon class="h-4 w-4" />
					Verify this device
				</button>
				<Separator />
			{/if}

			<!-- View Details Action -->
			<button
				class="hover:bg-accent hover:text-accent-foreground flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
				onclick={handleViewDetails}
			>
				<InfoIcon class="h-4 w-4" />
				View details
			</button>

			<Separator />

			<!-- Remove Device Action -->
			<button
				class="text-destructive hover:bg-destructive hover:text-destructive-foreground flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
				onclick={handleRemoveDevice}
			>
				<TrashIcon class="h-4 w-4" />
				Remove device
			</button>
		</div>
	</PopoverContent>
</Popover>
