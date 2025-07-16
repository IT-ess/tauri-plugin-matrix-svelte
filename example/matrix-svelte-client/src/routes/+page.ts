import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { createDMRoomFormSchema } from '$lib/schemas/matrix-id';

export const load: PageLoad = async () => {
	return {
		createDMRoomForm: await superValidate(zod4(createDMRoomFormSchema))
	};
};
