<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { gotoRoomInfo } from '$lib/utils.svelte';
	import { ChevronLeft } from '@lucide/svelte';
	import BookmarksList from '$lib/components/bookmarks/bookmarks-list.svelte';

	let { roomId, avatar }: { roomId: string; avatar: string | null } = $props();
</script>

<div class="bg-background flex h-full w-full flex-col overflow-x-hidden">
	<header class="pt-safe bg-background sticky top-0 right-0 left-0 z-50 flex w-full flex-col pl-2">
		<div class="flex items-center">
			<button
				onclick={() => gotoRoomInfo(roomId, avatar)}
				class="hover:bg-accent flex size-10 items-center justify-center rounded-full transition-colors"
				aria-label="Go back"
			>
				<ChevronLeft class="text-foreground size-6" />
			</button>
			<h1 class="pl-2 text-lg font-semibold">{m.bookmarks()}</h1>
		</div>
	</header>
	<BookmarksList {roomId} />
</div>
