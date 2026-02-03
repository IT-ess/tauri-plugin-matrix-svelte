import { hasBackupSetup } from 'tauri-plugin-matrix-svelte-api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	return { hasBackupSetup: await hasBackupSetup() };
};
