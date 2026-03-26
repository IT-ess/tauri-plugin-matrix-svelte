import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { profileStore } from '../../hooks.client';

export const load: PageLoad = async ({ url }) => {
	const matrixUserId = url.searchParams.get('id');
	if (matrixUserId === null) {
		error(500, 'The current room id has not been set properly');
	}
	const avatar =
		profileStore.state[matrixUserId].state == 'loaded'
			? profileStore.state[matrixUserId].data.avatarUrl
			: undefined;

	const name =
		profileStore.state[matrixUserId].state == 'loaded'
			? profileStore.state[matrixUserId].data.username
			: '';

	return { matrixUserId, avatar, name };
};
