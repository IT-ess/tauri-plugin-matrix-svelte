import { emit } from '@tauri-apps/api/event';
import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import { MatrixSvelteEmitEvent } from '../tauri-events.js';
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
			syncStrategy: 'debounce',
			syncInterval: 300
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

	/**
	 * Start the Rune store
	 */
	async startStoreAndSendConfirmationEvent(): Promise<void> {
		await this.start();
		await this.save();
		await emit(MatrixSvelteEmitEvent.RoomsCollectionStarted);
	}

	/**
	 * Gets the current joined regular rooms ids
	 * @returns An array of joined rooms ids
	 */
	getDisplayedRegularRoomsIds(): string[] {
		const ids = this.state.displayedRegularRooms; // pass by value
		return ids;
	}

	/**
	 * Gets the current joined DM rooms ids
	 * @returns An array of joined rooms ids
	 */
	getDisplayedDMRoomsIds(): string[] {
		const ids = this.state.displayedDirectRooms; // pass by value
		return ids;
	}
}
