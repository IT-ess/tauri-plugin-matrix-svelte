import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import { emit } from '@tauri-apps/api/event';
import { MatrixSvelteEmitEvent, type RoomCreatedEventType } from '../tauri-events.js';
import type { RoomScreen } from '../bindings/RoomScreen.js';

export class RoomStore extends RuneStore<RoomScreen> {
	/**
	 * Creates a new RoomStore instance
	 * @param id The Matrix room id used to name the store
	 * @param options Optional configuration options
	 */
	constructor(id: string, options: TauriPluginSvelteRuneStoreOptions<RoomScreen> = {}) {
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

		options = {
			hooks,
			syncStrategy: 'debounce',
			syncInterval: 300
		};

		super(id, defaultRoom, options);
	}

	/**
	 * Start the Rune store
	 */
	async startStoreAndSendConfirmationEvent(id: string): Promise<void> {
		// start the store
		await this.start();

		// send event to rust to confirm that the store is ready
		const payload: RoomCreatedEventType = {
			id,
			message: `Room store with id ${id} has been sucessfully created and started`
		};
		await emit(MatrixSvelteEmitEvent.RoomCreated, payload);
	}
}
