<script lang="ts">
	import { error } from '@sveltejs/kit';
	import { enablePushNotifications, requestPermissionsAndCreateChannel } from '$lib/notifications';
	import { m } from '$lib/paraglide/messages';
	import { Switch } from '$lib/components/ui/switch';
	import { cancelAll } from '@choochmeque/tauri-plugin-notifications-api';
	import { toast } from 'svelte-sonner';

	let { hasPending }: { hasPending: boolean } = $props();

	const enablePushNotif = async () => {
		try {
			await requestPermissionsAndCreateChannel();
			await enablePushNotifications(true);
		} catch (e) {
			error(400, `couldn't enable push notifications. ${e}`);
		}
	};

	// svelte-ignore state_referenced_locally
	let isActivated = $state(hasPending);

	const changeToggleState = () => {
		// Wait for isActivated value to be updated
		setTimeout(async () => {
			if (isActivated) {
				await enablePushNotif();
			} else {
				await cancelAll();
				toast.success(`Push notifications removed`);
			}
		}, 100);
	};
</script>

<div class="flex shrink flex-col">
	<div class="mb-4 flex items-center justify-between border-b border-border pb-6">
		<div>
			<p class="mb-1 font-medium">{m.notification_enable_notifications()}</p>
			<p class="text-sm text-muted-foreground">{m.notification_enable_notifications_desc()}</p>
		</div>
		<Switch onclick={changeToggleState} bind:checked={isActivated} />
	</div>
</div>
