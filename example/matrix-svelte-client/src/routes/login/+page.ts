import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { loginFormSchema } from '#lib/schemas/login.js';
import { hostname } from '@tauri-apps/plugin-os';

export const load: PageLoad = async () => {
	return {
		form: await superValidate(zod4(loginFormSchema))
	};
};
