import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	const openInviteDrawerOnLoad = !!url.searchParams.get('focusInvite');

	return { openInviteDrawerOnLoad };
};
