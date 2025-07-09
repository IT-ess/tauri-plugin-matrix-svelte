import {
	loginStore,
	profileStore,
	roomsCollection,
	roomStoresMap,
	storeListener
} from '../hooks.client';
import type { LayoutLoad } from './$types';

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load: LayoutLoad = () => {
	return { roomsCollection, roomStoresMap, storeListener, loginStore, profileStore };
};
