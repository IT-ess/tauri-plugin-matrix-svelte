import type { LayoutLoad } from './$types';
import { locale } from '@tauri-apps/plugin-os';
import { setLocale } from '$lib/paraglide/runtime.js';

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
};
