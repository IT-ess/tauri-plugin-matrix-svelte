import { RuneStore, type StoreHooks } from '@tauri-store/svelte';
import type { RoomScreen } from '../bindings/RoomScreen.js';

export const ROOM_STORE_ID = 'room-store';

export class RoomStore extends RuneStore<RoomScreen> {
	/**
	 * Creates a new RoomStore instance
	 * @param id The Matrix room id used to name the store
	 * @param options Optional configuration options
	 */
	constructor() {
		// Initialize with default empty Room state
		const defaultRoom: RoomScreen = {
			roomId: 'Not known yet',
			roomName: 'Not known yet',
			tlState: null,
			doneLoading: false,
			typingUsers: [],
			members: {}
		};

		const hooks: StoreHooks = {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			error: (err: any) => console.error(err)
		};

		super(ROOM_STORE_ID, defaultRoom, {
			hooks,
			save: false,
			syncStrategy: 'debounce',
			syncInterval: 10
		});
	}
}
