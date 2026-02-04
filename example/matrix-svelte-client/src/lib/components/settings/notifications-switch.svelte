<script lang="ts">
	import { error } from '@sveltejs/kit';
	import {
		enableDailyRecapAndPushNotifications,
		requestPermissionsAndCreateChannel
	} from '$lib/notifications';
	import { m } from '$lib/paraglide/messages';
	import { Switch } from '$lib/components/ui/switch';
	import { cancelAll } from '@choochmeque/tauri-plugin-notifications-api';
	import { toast } from 'svelte-sonner';

	let { hasPending }: { hasPending: boolean } = $props();

	const enableRecap = async () => {
		try {
			await requestPermissionsAndCreateChannel();
			await enableDailyRecapAndPushNotifications(true);
		} catch (e) {
			error(400, `couldn't enable daily recap. ${e}`);
		}
	};

	// svelte-ignore state_referenced_locally
	let isActivated = $state(hasPending);

	const changeRecapState = () => {
		// Wait for isActivated value to be updated
		setTimeout(async () => {
			if (isActivated) {
				await enableRecap();
			} else {
				await cancelAll();
				toast.success(`Daily recap notification removed`);
			}
		}, 100);
	};
</script>

<div class="flex shrink flex-col">
	<div class="border-border mb-4 flex items-center justify-between border-b pb-6">
		<div>
			<p class="mb-1 font-medium">{m.notification_enable_notifications()}</p>
			<p class="text-muted-foreground text-sm">{m.notification_enable_desc()}</p>
		</div>
		<Switch onclick={changeRecapState} bind:checked={isActivated} />
	</div>
</div>
