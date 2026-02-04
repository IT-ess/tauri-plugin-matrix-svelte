import type { ClientInit } from '@sveltejs/kit';
import { type UnlistenFn } from '@tauri-apps/api/event';
import { roomStoresMap } from '$lib/stores/rooms.map.svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
	LoginStore,
	MatrixSvelteListenEvent,
	ProfileStore,
	RoomsCollection,
	RoomStore,
	type RoomCreateEventType
} from 'tauri-plugin-matrix-svelte-api';

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
	storeListener = await getCurrentWebviewWindow().listen<RoomCreateEventType>(
		MatrixSvelteListenEvent.RoomCreate,
		async ({ payload }) => {
			const { id } = payload;
			console.log(
				`Matrix room creation event received. Begining creation of store room with id ${id}`
			);
			const newRoom = new RoomStore(id);
			await newRoom.startStoreAndSendConfirmationEvent(id);
			roomStoresMap.set(id, newRoom);
		}
	);

	// The login store start allows the rust lib to pursue its init.
	await loginStore.start();
	while (loginStore.state.state === 'initiating') {
		const sleep = () => {
			return new Promise((resolve) => setTimeout(resolve, 200));
		};
		console.log('awaiting for init');
		await sleep();
	}

	// May not be necessary anymore since we will rather provide a loader in the room list view
	const splashscreenEvent = new CustomEvent('app-init-done');
	window.dispatchEvent(splashscreenEvent);
};

export { roomsCollection, roomStoresMap, storeListener, loginStore, profileStore };
