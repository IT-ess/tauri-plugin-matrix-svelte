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
	import { m } from '$lib/paraglide/messages';
	import type { FrontendDevice as Device } from 'tauri-plugin-matrix-svelte-api';

	interface Props {
		device: Device;
		onVerify: (verifiedCallback: () => void) => void;
	}

	let { device, onVerify }: Props = $props();

	let isDeviceNowVerified = $state(false);

	const innerOnVerify = () => {
		onVerify(markDeviceNowVerified);
	};

	const markDeviceNowVerified = () => {
		isDeviceNowVerified = true;
	};

	const formatDate = (timestamp: number | null) => {
		const date = timestamp ? new Date(timestamp) : new Date();
		const now = new Date();
		const diffTime = Math.abs(now.getTime() - date.getTime());
		const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

		if (diffDays === 1) return m.devices_day_ago();
		if (diffDays < 7) return m.devices_days_ago({ days: diffDays });
		if (diffDays < 14) return m.devices_week_ago();
		if (diffDays < 30) return m.devices_weeks_ago({ weeks: Math.floor(diffDays / 7) });
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

	let DeviceIcon = $derived(getDeviceIcon(device));
</script>

<Card class="cursor-pointer p-4 transition-shadow hover:shadow-md">
	<div class="flex items-center justify-between">
		<div class="flex flex-1 items-center gap-3">
			<div class="bg-muted rounded-lg p-2">
				<DeviceIcon class="text-muted-foreground h-5 w-5" />
			</div>

			<div class="min-w-0 flex-1">
				<h3 class="truncate font-medium">{device.displayName}</h3>
				<div class="mb-1 flex items-center gap-2">
					{#if device.isVerifiedWithCrossSigning || device.isVerified || isDeviceNowVerified}
						<Badge variant="default" class="bg-green-100 text-green-800 hover:bg-green-100">
							<ShieldCheckIcon class="mr-1 h-3 w-3" />
							{m.verified()}
						</Badge>
					{:else}
						<Badge variant="outline" class="border-orange-200 text-orange-600">
							<ShieldIcon class="mr-1 h-3 w-3" />
							{m.unverified()}
						</Badge>
					{/if}
					{#if device.isCurrentDevice}
						<Badge variant="outline" class="bg-primary text-primary-foreground">{m.current()}</Badge
						>
					{/if}
				</div>

				<div class="text-muted-foreground flex items-center gap-4 text-sm">
					<span>ID: {device.deviceId}</span>
					<span>{m.devices_last_seen()} {formatDate(device.lastSeenTs)}</span>
				</div>
			</div>
		</div>

		<DeviceActions {device} onVerify={innerOnVerify} />
	</div>
</Card>
