import { emit } from '@tauri-apps/api/event';
import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import { MatrixSvelteEmitEvent } from '../tauri-events.js';
import { type RoomsCollectionType } from '../types.js';

export const ROOMS_COLLECTION_STORE_ID = 'rooms-collection';

export class RoomsCollection extends RuneStore<RoomsCollectionType> {
	/**
	 * Creates a new RoomsCollections instance
	 */

	constructor() {
		const hooks: StoreHooks = {
			error: (err: any) => console.error(err)
		};
		const options: TauriPluginSvelteRuneStoreOptions<RoomsCollectionType> = {
			hooks,
			syncStrategy: 'debounce',
			syncInterval: 1000
		};

		super(
			ROOMS_COLLECTION_STORE_ID,
			{
				invitedRooms: {},
				allJoinedRooms: {},
				displayedInvitedRooms: [],
				displayedJoinedRooms: [],
				status: { status: 'notLoaded', message: 'Initiating' },
				currentActiveRoom: null,
				maxKnownRooms: undefined
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
	 * Gets the current joined rooms ids
	 * @returns An array of joined rooms
	 */
	getDisplayedJoinedRoomsIds(): string[] {
		const ids = this.state.displayedJoinedRooms; // pass by value
		return ids;
	}

	/**
	 * Adds a room id to the joined rooms list
	 * @returns The added id
	 */
	addDisplayedJoinedRoomId(id: string): string {
		if (!this.isDisplayedRoomRunning(id)) {
			this.state.displayedJoinedRooms.push(id);
		}
		return id;
	}

	/**
	 * Checks from a room id if its store is running already
	 */
	isDisplayedRoomRunning(id: string): boolean {
		return this.state.displayedJoinedRooms.includes(id);
	}
}
