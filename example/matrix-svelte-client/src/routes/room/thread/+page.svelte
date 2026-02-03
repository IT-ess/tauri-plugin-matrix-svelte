<script lang="ts">
	import Thread from '$lib/components/room/thread/thread.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<Thread roomStore={data.roomStore} rootEventId={data.threadRoot} roomAvatarUrl={data.avatarUri} />
