import type { LayoutLoad } from './$types';
import { emit } from '@tauri-apps/api/event';
import { error } from '@sveltejs/kit';
import { roomNameToPlainString } from '$lib/utils.svelte';
import { roomsCollection, roomStoresMap } from '../../hooks.client';
import {
	MatrixSvelteEmitEvent,
	type UpdateCurrentActiveRoom
} from 'tauri-plugin-matrix-svelte-api';

export const load: LayoutLoad = async ({ url }) => {
	const roomId = url.searchParams.get('id');
	const avatarUri = url.searchParams.get('avatar');
	if (roomId === null) {
		error(500, 'The current room id has not been set properly');
	}
	const roomStore = roomStoresMap.get(roomId);
	if (roomStore === undefined) {
		error(500, 'The store of this room is not loaded');
	}
	const payload: UpdateCurrentActiveRoom = {
		roomId,
		// Kinda weird, but otherwise the room name is never initiated or
		// requires an additional fetch from the backend
		roomName: roomNameToPlainString(roomsCollection.state.allJoinedRooms[roomId].roomName)
	};
	await emit(MatrixSvelteEmitEvent.UpdateCurrentActiveRoom, payload);
	return { roomStore, avatarUri };
};
