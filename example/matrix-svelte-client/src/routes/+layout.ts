import type { LayoutLoad } from './$types';
import { locale } from '@tauri-apps/plugin-os';
import { setLocale } from '$lib/paraglide/runtime.js';
import { loginState } from '$lib/login-state.svelte';
import { checkDeviceVerification, isLoggedIn } from 'tauri-plugin-matrix-svelte-api';

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load: LayoutLoad = async () => {
	const lang = await locale();
	console.log(`Locale is ${lang}`);
	if (lang !== null && lang !== document.documentElement.lang) {
		document.documentElement.lang = lang;
		setLocale(lang == 'fr-FR' ? 'fr' : 'en', { reload: false }); // TODO: use a pattern matcher instead
	}

	if (!loginState.isLoggedIn) {
		try {
			loginState.isLoggedIn = await isLoggedIn();
		} catch (err) {
			console.error(`Couldn't check login state. ${err}`);
		}
	}

	if (!loginState.isVerified) {
		try {
			loginState.isVerified = (await checkDeviceVerification()).verificationState === 'verified';
		} catch (err) {
			console.error(`Couldn't check verification state. ${err}`);
		}
	}
};
