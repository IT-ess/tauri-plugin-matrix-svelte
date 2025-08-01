import type { PageLoad } from './$types';
import { getDevices } from 'tauri-plugin-matrix-svelte-api';

export const load: PageLoad = async ({ parent }) => {
	const data = await parent();
	const userId = data.loginStore.state.userId ?? ''; // User id should be defined at this state

	return { devices: await getDevices(userId), userId };
};
