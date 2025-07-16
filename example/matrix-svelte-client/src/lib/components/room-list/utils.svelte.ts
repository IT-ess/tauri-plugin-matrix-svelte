import type { JoinedRoomInfo, RoomsCollection } from 'tauri-plugin-matrix-svelte-api';

export function getSortedRoomArray(roomsCollection: RoomsCollection): () => JoinedRoomInfo[] {
	const entriesArray = $derived.by(() =>
		Array.from(Object.values(roomsCollection.state.allJoinedRooms))
	);
	// TODO: use displayedJoinedRooms to filter the array

	// Sort the array by latestUpdate (newest first)
	return () =>
		entriesArray.sort(
			(a, b) =>
				new Date(b.latest ? b.latest[0] : 0).getTime() -
				new Date(a.latest ? a.latest[0] : 0).getTime()
		);
}
