import { getAllDMRooms } from 'tauri-plugin-matrix-svelte-api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	const dmRooms = await getAllDMRooms();

	const openInviteDrawerOnLoad = !!url.searchParams.get('focusInvite');

	return { dmRooms, openInviteDrawerOnLoad };
};
