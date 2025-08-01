<script lang="ts">
	import { Card } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import DeviceActions from './device-actions.svelte';
	import {
		ShieldCheckIcon,
		ShieldIcon,
		MonitorIcon,
		CircleQuestionMarkIcon,
		TabletSmartphoneIcon,
		PanelsTopLeftIcon
	} from '@lucide/svelte';
	import type { Device } from 'tauri-plugin-matrix-svelte-api';

	interface Props {
		device: Device;
		onVerify: () => void;
	}

	let { device, onVerify }: Props = $props();

	const formatDate = (timestamp: number) => {
		const date = new Date(timestamp);
		const now = new Date();
		const diffTime = Math.abs(now.getTime() - date.getTime());
		const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

		if (diffDays === 1) return '1 day ago';
		if (diffDays < 7) return `${diffDays} days ago`;
		if (diffDays < 14) return '1 week ago';
		if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
		return date.toLocaleDateString();
	};

	// Get device icon based on display name
	const getDeviceIcon = (device: Device) => {
		switch (device.guessedType) {
			case 'ios':
			case 'android':
				return TabletSmartphoneIcon;
			case 'web':
				return PanelsTopLeftIcon;
			case 'desktop':
				return MonitorIcon;
			default:
				return CircleQuestionMarkIcon;
		}
	};

	const DeviceIcon = getDeviceIcon(device);
</script>

<Card class="cursor-pointer p-4 transition-shadow hover:shadow-md">
	<div class="flex items-center justify-between">
		<div class="flex flex-1 items-center gap-3">
			<div class="bg-muted rounded-lg p-2">
				<DeviceIcon class="text-muted-foreground h-5 w-5" />
			</div>

			<div class="min-w-0 flex-1">
				<div class="mb-1 flex items-center gap-2">
					<h3 class="truncate font-medium">{device.displayName}</h3>

					{#if device.isVerifiedWithCrossSigning}
						<Badge variant="default" class="bg-green-100 text-green-800 hover:bg-green-100">
							<ShieldCheckIcon class="mr-1 h-3 w-3" />
							Cross-signed
						</Badge>
					{:else if device.isVerified}
						<Badge variant="secondary" class="bg-blue-100 text-blue-800 hover:bg-blue-100">
							<ShieldIcon class="mr-1 h-3 w-3" />
							Verified
						</Badge>
					{:else}
						<Badge variant="outline" class="border-orange-200 text-orange-600">
							<ShieldIcon class="mr-1 h-3 w-3" />
							Unverified
						</Badge>
					{/if}
					{#if device.isCurrentDevice}
						<Badge variant="outline" class="bg-primary text-primary-foreground">Current</Badge>
					{/if}
				</div>

				<div class="text-muted-foreground flex items-center gap-4 text-sm">
					<span>ID: {device.deviceId}</span>
					<span>•</span>
					<span>Added {formatDate(device.registrationDate)}</span>
				</div>
			</div>
		</div>

		<DeviceActions {device} {onVerify} />
	</div>
</Card>
