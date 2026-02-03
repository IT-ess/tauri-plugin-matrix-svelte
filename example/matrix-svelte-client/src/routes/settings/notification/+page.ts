import { pending } from '@choochmeque/tauri-plugin-notifications-api';
import type { PageLoad } from './$types';
import { platform } from '@tauri-apps/plugin-os';

export const load: PageLoad = async () => {
	const currentPlatform = platform();
	return {
		hasPending:
			currentPlatform === 'ios' || currentPlatform === 'android'
				? (await pending()).length > 0
				: false
	};
};
