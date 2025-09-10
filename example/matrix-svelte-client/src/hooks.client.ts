import type { ClientInit } from '@sveltejs/kit';
import {
	RoomStore,
	RoomsCollection,
	LoginStore,
	ProfileStore,
	events,
	watchNotifications
} from 'tauri-plugin-matrix-svelte-api';
import { type UnlistenFn } from '@tauri-apps/api/event';
import { roomStoresMap } from '$lib/stores/rooms.map.svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { platform } from '@tauri-apps/plugin-os';
import {
	createChannel,
	Importance,
	isPermissionGranted,
	requestPermission,
	sendNotification,
	Visibility
} from '@tauri-apps/plugin-notification';

// Create the store that will track the login state
const loginStore = new LoginStore();
// Create the store that gathers the user profiles
const profileStore = new ProfileStore();
// Init the room ids store that syncs available rooms with backend
const roomsCollection = new RoomsCollection();

let storeListener: UnlistenFn;

export const init: ClientInit = async () => {
	await profileStore.start();
	await roomsCollection.startStoreAndSendConfirmationEvent();
	storeListener = await getCurrentWebviewWindow().listen<events.RoomCreateEventType>(
		events.MatrixSvelteListenEvent.RoomCreate,
		async ({ payload }) => {
			const { id } = payload;
			console.log(
				`Matrix room creation event received. Begining creation of store room with id ${id}`
			);
			const newRoom = new RoomStore(id);
			await newRoom.startStoreAndSendConfirmationEvent(id, roomsCollection);
			roomStoresMap.set(id, newRoom);
			roomsCollection.addDisplayedJoinedRoomId(id);
		}
	);
	// The login store start allows the rust lib to pursue its init.
	await loginStore.start();

	// Do we have permission to send a notification?
	let permissionGranted = await isPermissionGranted();
	console.log(`Is notification permission granted: ${permissionGranted}`);

	// If not we need to request it
	if (!permissionGranted) {
		const permission = await requestPermission();
		permissionGranted = permission === 'granted';
	}

	if (permissionGranted) {
		const currentPlatform = platform();
		if (currentPlatform === 'android') {
			await createChannel({
				id: 'messages',
				name: 'Messages',
				description: 'Notifications for new messages',
				importance: Importance.High,
				visibility: Visibility.Public,
				lights: true,
				lightColor: '#ff0000',
				vibration: true
				//sound: 'notification_sound'
			});
			sendNotification({
				title: 'Matrix-Svelte',
				body: 'App is launched !',
				channelId: 'messages'
			});
		} else if (currentPlatform === 'ios') {
			const watchResult = await watchNotifications((event) => {
				switch (event.type) {
					case 'BACKGROUND_TAP':
						console.log('User tapped notification in background');
						break;
					case 'FOREGROUND_TAP':
						console.log('User tapped notification in foreground');
						break;
					case 'FOREGROUND_DELIVERY':
						console.log('Notification received in foreground');
						break;
					case 'BACKGROUND_DELIVERY':
						console.log('Notification received in background');
						break;
				}
				console.log('Notification payload:', event.payload);
			});

			if (watchResult.success) {
				console.log('Successfully set up notification listener');
				sendNotification({ title: 'Matrix-Svelte', body: 'App is launched !' });
			}
		} else {
			sendNotification({ title: 'Matrix-Svelte', body: 'App is launched !' });
		}
	}

	while (loginStore.state.state === 'initiating') {
		const sleep = () => {
			return new Promise((resolve) => setTimeout(resolve, 1000));
		};
		await sleep();
	}

	// May not be necessary anymore since we will rather provide a loader in the room list view
	const splashscreenEvent = new CustomEvent('app-init-done');
	window.dispatchEvent(splashscreenEvent);
};

export { roomsCollection, roomStoresMap, storeListener, loginStore, profileStore };
