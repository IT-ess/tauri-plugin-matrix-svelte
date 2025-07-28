import { events } from 'tauri-plugin-matrix-svelte-api';
import type { PageLoad } from './$types';
import { emit } from '@tauri-apps/api/event';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ parent }) => {
	const data = await parent();
	const roomId = data.roomsCollection.state.currentActiveRoom;
	if (roomId === null) {
		error(500, 'The current room id has not been set properly');
	}
	const roomStore = data.roomStoresMap.get(roomId);
	if (roomStore === undefined) {
		error(500, 'The store of this room is not loaded');
	}
	const payload: events.UpdateCurrentActiveRoom = {
		roomId,
		roomName: roomStore.state.roomName
	};
	await emit(events.MatrixSvelteEmitEvent.UpdateCurrentActiveRoom, payload);
	return { roomStore };
};
