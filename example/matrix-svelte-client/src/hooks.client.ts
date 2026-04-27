import type { ClientInit } from '@sveltejs/kit';
import { type UnlistenFn } from '@tauri-apps/api/event';
import { roomStoresMap } from '$lib/stores/rooms.map.svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { goto } from '$app/navigation';
import {
	hasSessionStored,
	LoginStore,
	MatrixSvelteListenEvent,
	RoomsCollection,
	RoomStore,
	type RoomCreateEventType
} from 'tauri-plugin-matrix-svelte-api';

// Create the store that will track the login state
const loginStore = new LoginStore();
// Init the room ids store that syncs available rooms with backend
const roomsCollection = new RoomsCollection();

let storeListener: UnlistenFn;

export const init: ClientInit = async () => {
	// The login store start allows the rust lib to pursue its init.
	await loginStore.start();

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

	const hasSessionStoredBool = await hasSessionStored();
	if (!hasSessionStoredBool) {
		loginStore.state.state = 'awaitingForHomeserver';
	}

	const splashscreenEvent = new CustomEvent('app-init-done');
	window.dispatchEvent(splashscreenEvent);

	if (!hasSessionStoredBool) {
		goto('/login');
	}
};

export { roomsCollection, roomStoresMap, storeListener, loginStore };
