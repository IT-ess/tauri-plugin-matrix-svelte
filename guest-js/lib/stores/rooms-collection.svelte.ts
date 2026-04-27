import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import type { RoomsList } from '../bindings/RoomsList.js';

export const ROOMS_COLLECTION_STORE_ID = 'rooms-collection';

export class RoomsCollection extends RuneStore<RoomsList> {
	/**
	 * Creates a new RoomsCollections instance
	 */

	constructor() {
		const hooks: StoreHooks = {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			error: (err: any) => console.error(err)
		};
		const options: TauriPluginSvelteRuneStoreOptions<RoomsList> = {
			hooks,
			save: false,
			syncStrategy: 'debounce',
			syncInterval: 10
		};

		super(
			ROOMS_COLLECTION_STORE_ID,
			{
				invitedRooms: {},
				allJoinedRooms: {},
				displayedInvitedRooms: [],
				displayedDirectRooms: [],
				displayedRegularRooms: [],
				status: { status: 'notLoaded', message: 'Initiating' },
				currentActiveRoom: null,
				maxKnownRooms: null,
				filterKeywords: ''
			},
			options
		);
	}
}
