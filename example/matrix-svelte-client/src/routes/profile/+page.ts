import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { fetchUserProfile } from 'tauri-plugin-matrix-svelte-api';

export const load: PageLoad = async ({ url }) => {
	const matrixUserId = url.searchParams.get('id');
	if (matrixUserId === null) {
		error(500, 'The current room id has not been set properly');
	}

	return { profile: await fetchUserProfile(matrixUserId, null) };
};
