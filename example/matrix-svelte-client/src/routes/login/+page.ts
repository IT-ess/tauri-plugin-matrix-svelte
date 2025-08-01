import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { loginFormSchema } from '$lib/schemas/login';
import { hostname } from '@tauri-apps/plugin-os';

export const load: PageLoad = async () => {
	const host = (await hostname()) ?? 'Matrix client';
	return {
		form: await superValidate(zod4(loginFormSchema)),
		host
	};
};
