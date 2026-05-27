import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import { tryGetRoomPreviewFromAddress, type RoomPreview } from 'tauri-plugin-matrix-svelte-api';

export const load: PageLoad = async ({ url }) => {
	const prefetchedData = url.searchParams.get('data');
	if (prefetchedData) {
		return {
			preview: new Promise<[RoomPreview, string[]]>((resolve, reject) => {
				try {
					const via = url.searchParams.get('via');
					if (!via) throw Error('Missing `via` server names');
					const data: RoomPreview = JSON.parse(prefetchedData);
					const parsedVia: string[] = JSON.parse(via);
					resolve([data, parsedVia]);
				} catch (err) {
					console.error(err);
					reject(err);
				}
			})
		};
	} else {
		const matrixId = url.searchParams.get('matrixId');
		if (!matrixId) {
			error(400, 'Missing MatrixId to access this preview');
		} else {
			return { preview: tryGetRoomPreviewFromAddress(matrixId) };
		}
	}
};
