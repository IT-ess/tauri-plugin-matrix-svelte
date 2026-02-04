import type { RoomModel } from 'tauri-plugin-matrix-svelte-api';
import { roomsCollection } from '../../hooks.client';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	const fullDMRooms = Object.values(roomsCollection.state.allJoinedRooms).filter(
		(room) => room.isDirect && !room.isTombstoned
	);

	const dmRooms: RoomModel[] = fullDMRooms.map((fullRoom) => ({
		avatarUrl: fullRoom.avatar,
		displayName: fullRoom.roomName.kind !== 'empty' ? fullRoom.roomName.name : 'Not known yet',
		dmUserId: fullRoom.directUserId,
		id: fullRoom.roomId
	}));

	const openInviteDrawerOnLoad = !!url.searchParams.get('focusInvite');

	return { dmRooms, openInviteDrawerOnLoad };
};
