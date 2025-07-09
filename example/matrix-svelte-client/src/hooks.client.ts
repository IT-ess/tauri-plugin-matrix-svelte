import type { ClientInit } from '@sveltejs/kit';
import {
	RoomStore,
	RoomsCollection,
	LoginStore,
	ProfileStore,
	events
} from 'tauri-plugin-matrix-svelte-api';
import { type UnlistenFn } from '@tauri-apps/api/event';
import { roomStoresMap } from '$lib/stores/rooms.map.svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

// Create the store that will track the login state
const loginStore = new LoginStore();
// Create the store that gathers the user profiles
const profileStore = new ProfileStore();
// Init the room ids store that syncs available rooms with backend
const roomsCollection = new RoomsCollection();

let storeListener: UnlistenFn;

export const init: ClientInit = async () => {
	await loginStore.start();
	await profileStore.start();
	await roomsCollection.startStoreAndSendConfirmationEvent();
	const roomIds = roomsCollection.getDisplayedJoinedRoomsIds();
	console.log(roomIds);
	// TODO: we keep this for the moment, but it may be not necessary since the stores are created quickly already
	for (const id of roomIds) {
		const newRoom = new RoomStore(id);
		await newRoom.startStoreAndSendConfirmationEvent(id, roomsCollection);
		roomStoresMap.set(id, newRoom);
		console.log(`Room Store with id ${id} has been restored`);
	}
	storeListener = await getCurrentWebviewWindow().listen<events.RoomCreateEventType>(
		events.MatrixSvelteListenEvent.RoomCreate,
		async ({ payload }) => {
			let { id } = payload;
			console.log(
				`Matrix room creation event received. Begining creation of store room with id ${id}`
			);
			const newRoom = new RoomStore(id);
			await newRoom.startStoreAndSendConfirmationEvent(id, roomsCollection);
			roomStoresMap.set(id, newRoom);
			roomsCollection.addDisplayedJoinedRoomId(id);
		}
	);

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
