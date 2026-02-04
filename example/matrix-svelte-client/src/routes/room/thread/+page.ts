import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ url }) => {
	const threadRoot = url.searchParams.get('threadRoot');
	if (threadRoot === null) {
		error(500, 'The current thread root has not been set properly');
	}
	return { threadRoot };
};
