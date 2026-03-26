<script lang="ts">
	import { SearchIcon, X } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import { m } from '$lib/paraglide/messages';

	let { searchQuery = $bindable() }: { searchQuery: string } = $props();
</script>

<div class="relative">
	<SearchIcon
		class="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2 transform"
		size={20}
	/>
	<Input
		bind:value={searchQuery}
		placeholder={m.rooms_search_bar_placeholder()}
		class="py-3 pr-4 pl-10 text-base"
	/>
	{#if searchQuery}
		<X
			onclick={() => (searchQuery = '')}
			class="text-muted-foreground absolute top-1/2 right-3 size-4 -translate-y-1/2 transform"
		/>
	{/if}
</div>
