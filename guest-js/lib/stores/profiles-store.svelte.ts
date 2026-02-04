import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import type { UserId } from '../matrix-requests/common.js';

export const PROFILES_STORE_ID = 'profiles';

export type ProfileType =
	| { state: 'requested' }
	| {
			state: 'loaded';
			data: {
				userId: string;
				username: string;
				avatarUrl?: string;
				rooms: string[];
			};
	  };

export type ProfileRecord = Record<UserId, ProfileType>;

export class ProfileStore extends RuneStore<ProfileRecord> {
	/**
	 * Creates a new ProfileStore instance
	 */

	constructor() {
		const hooks: StoreHooks<ProfileRecord> = {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			error: (err: any) => console.error(err)
		};
		const options: TauriPluginSvelteRuneStoreOptions<ProfileRecord> = {
			hooks,
			syncStrategy: 'debounce',
			syncInterval: 100
		};
		super(PROFILES_STORE_ID, {}, options);
	}
}
