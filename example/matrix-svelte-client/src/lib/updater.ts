import { platform } from '@tauri-apps/plugin-os';
import { check } from '@tauri-apps/plugin-updater';
import { ask } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { m } from '#lib/paraglide/messages.js';

// Desktop only: the updater plugin isn't registered on mobile (store updates there).
export async function checkForAppUpdates() {
	const currentPlatform = platform();
	if (currentPlatform === 'android' || currentPlatform === 'ios') return;

	try {
		const update = await check();
		if (!update) return;
		const yes = await ask(m.update_available_body({ version: update.version }), {
			title: m.update_available_title(),
			kind: 'info',
			okLabel: m.update_install(),
			cancelLabel: m.update_later()
		});
		if (yes) {
			await update.downloadAndInstall();
			await relaunch();
		}
	} catch (e) {
		// No published release yet, offline, etc. — never block the app on this.
		console.warn('Update check failed:', e);
	}
}
