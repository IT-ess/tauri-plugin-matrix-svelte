import { platform } from '@tauri-apps/plugin-os';
import {
	createChannel,
	Importance,
	isPermissionGranted,
	registerForPushNotifications,
	requestPermission,
	Schedule,
	sendNotification,
	Visibility
} from '@choochmeque/tauri-plugin-notifications-api';
import { m } from '$lib/paraglide/messages';
import { toast } from 'svelte-sonner';
import { getLocale } from './paraglide/runtime';
import { registerNotifications } from 'tauri-plugin-matrix-svelte-api';

export const DAILY_NOTIFICATIONS_CHANNEL_ID = 'daily_notifications';
export const MESSAGES_CHANNEL_ID = 'messages';

export async function requestPermissionsAndCreateChannel() {
	// Do we have permission to send a notification?
	let permissionGranted = await isPermissionGranted();
	console.log(`Is notification permission granted: ${permissionGranted}`);

	// If not we need to request it
	if (!permissionGranted) {
		const permission = await requestPermission();
		permissionGranted = permission === 'granted';
	}

	const currentPlatform = platform();
	if (currentPlatform === 'android') {
		if (permissionGranted) {
			await createChannel({
				id: DAILY_NOTIFICATIONS_CHANNEL_ID,
				name: m.notification_daily_recap(),
				description: m.notification_daily_recap_description(),
				importance: Importance.High,
				visibility: Visibility.Public,
				lights: true,
				lightColor: '#ff0000',
				vibration: true
				//sound: 'notification_sound'
			});

			await createChannel({
				id: MESSAGES_CHANNEL_ID,
				name: m.notification_messages_channel(),
				description: m.notification_messages_channel_desc(),
				importance: Importance.High,
				visibility: Visibility.Public,
				lights: true,
				lightColor: '#ff0000',
				vibration: true
				//sound: 'notification_sound'
			});
		}
	}
}

export async function enableDailyRecapAndPushNotifications(areNotificationsAllowed: boolean) {
	try {
		if (!areNotificationsAllowed) {
			await requestPermissionsAndCreateChannel();
		}

		sendNotification({
			title: m.notification_daily_recap(),
			schedule: Schedule.interval({
				day: 1
			}),
			channelId: DAILY_NOTIFICATIONS_CHANNEL_ID,
			autoCancel: true
		});

		const currentPlatform = platform();
		const token =
			currentPlatform === 'android' || currentPlatform === 'ios'
				? await registerForPushNotifications()
				: '';
		await registerNotifications(token, getLocale());
	} catch (e) {
		return toast.error(`Error while setting notification ${e}`);
	}

	toast.success(m.notification_set_success());
}
