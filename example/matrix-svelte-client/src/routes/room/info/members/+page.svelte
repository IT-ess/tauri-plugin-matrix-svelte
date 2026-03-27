<script lang="ts">
	import RoomMembers from '$lib/components/room/info/members/room-members.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<RoomMembers avatar={data.avatarUri} roomStore={data.roomStore} />
