import { SvelteMap } from 'svelte/reactivity';
import type { RoomStore } from 'tauri-plugin-matrix-svelte-api';

export const roomStoresMap: SvelteMap<string, RoomStore> = new SvelteMap();
