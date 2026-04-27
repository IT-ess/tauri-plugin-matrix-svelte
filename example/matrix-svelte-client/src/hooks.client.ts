import type { ClientInit } from '@sveltejs/kit';
import { goto } from '$app/navigation';
import {
	hasSessionStored,
	LoginStore,
	RoomsCollection,
	RoomStore
} from 'tauri-plugin-matrix-svelte-api';

// Create the store that will track the login state
const loginStore = new LoginStore();
// Init the room ids store that syncs available rooms with backend
const roomsCollection = new RoomsCollection();
const roomStore = new RoomStore();

export const init: ClientInit = async () => {
	// The login store start allows the rust lib to pursue its init.
	await loginStore.start();
	await roomsCollection.start();
	await roomStore.start();

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

export { roomsCollection, roomStore, loginStore };
