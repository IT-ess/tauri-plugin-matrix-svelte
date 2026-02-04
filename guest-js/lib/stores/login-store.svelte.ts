import {
	RuneStore,
	type StoreHooks,
	type TauriPluginSvelteRuneStoreOptions
} from '@tauri-store/svelte';
import type { LoginState } from '../bindings/LoginState.js';

export const LOGIN_STATE_STORE_ID = 'login-state';

export type LoginStateType = {
	state: LoginState;
	userId: string | null;
	verificationState: 'unknown' | 'verified' | 'unverified';
	syncServiceState: 'idle' | 'error' | 'terminated' | 'running' | 'offline';
	recoveryState: 'unknown' | 'enabled' | 'disabled' | 'incomplete';
	userAvatar: string | null;
	userDisplayName: string | null;
	deviceDisplayName: string | null;
};

export class LoginStore extends RuneStore<LoginStateType> {
	/**
	 * Creates a new LoginState instance
	 */
	constructor() {
		const hooks: StoreHooks = {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			error: (err: any) => console.error(err)
		};
		const options: TauriPluginSvelteRuneStoreOptions<LoginStateType> = {
			hooks,
			syncStrategy: 'debounce',
			syncInterval: 50
		};

		super(
			LOGIN_STATE_STORE_ID,
			{
				state: 'initiating',
				userId: null,
				verificationState: 'unknown',
				syncServiceState: 'offline',
				deviceDisplayName: null,
				recoveryState: 'unknown',
				userAvatar: null,
				userDisplayName: null
			},
			options
		);
	}
}
