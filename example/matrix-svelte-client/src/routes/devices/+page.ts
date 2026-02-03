import { getDevices } from 'tauri-plugin-matrix-svelte-api';
import { loginStore } from '../../hooks.client';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const userId = loginStore.state.userId ?? ''; // User id should be defined at this state

	return { devices: await getDevices(userId), userId };
};
