<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Card } from '$lib/components/ui/card';
	import { SearchIcon, MessageCircleIcon } from '@lucide/svelte';
	import {
		jsonSourceEventToObject,
		searchMessages,
		type SearchConfig
	} from 'tauri-plugin-matrix-svelte-api';

	let { roomId }: { roomId: string } = $props();

	let searchQuery = $state('');
	let isOpen = $state(false);

	// Filter results based on search query
	const filteredResults = $derived.by(async () => {
		if (!searchQuery.trim()) return [];
		const searchConfig: SearchConfig = {
			limit: 10,
			keys: ['Message'],
			room_id: roomId,
			after_limit: 0,
			before_limit: 0,
			next_batch: null,
			order_by_recency: false
		};
		let searchBatch = await searchMessages(searchQuery, searchConfig);
		console.log(`Matched items for term "${searchQuery}" :`);
		console.log(searchBatch);
		return searchBatch.results.map((res) => {
			const mapped = jsonSourceEventToObject(res.event_source);
			return { matchedEvent: mapped };
		});
	});

	// Handle input focus
	function handleFocus() {
		isOpen = true;
	}

	// Handle input blur with delay to allow for clicks
	function handleBlur() {
		setTimeout(() => {
			isOpen = false;
		}, 150);
	}
</script>

<div class="relative w-full">
	<!-- Search Input -->
	<div class="relative">
		<SearchIcon
			class="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2 transform"
			size={20}
		/>
		<Input
			bind:value={searchQuery}
			placeholder="Search messages, senders, or content..."
			class="py-3 pr-4 pl-10 text-base"
			onfocus={handleFocus}
			onblur={handleBlur}
		/>
	</div>

	<!-- Dropdown Results -->
	{#await filteredResults}
		<p>loading</p>
	{:then results}
		{#if isOpen && results.length > 0}
			<Card
				class="absolute top-full right-0 left-0 z-50 mt-2 max-h-96 overflow-y-auto border shadow-lg"
			>
				<div class="p-2">
					{#each results as message (message.matchedEvent.eventId)}
						<button
							class="hover:bg-accent hover:text-accent-foreground flex w-full items-start gap-3 rounded-lg p-3 text-left transition-colors"
						>
							<div class="text-muted-foreground mt-0.5 flex-shrink-0">
								<MessageCircleIcon size={16} />
							</div>
							<div class="min-w-0 flex-1">
								<div class="mb-1 flex items-center gap-2">
									<span class="text-foreground text-sm font-medium"
										>{message.matchedEvent.senderId}</span
									>
									<span class="text-muted-foreground text-xs"
										>{message.matchedEvent.timestamp.toLocaleDateString()}</span
									>
								</div>
								<div class="text-muted-foreground line-clamp-2 text-sm leading-relaxed">
									{message.matchedEvent.body}
								</div>
							</div>
						</button>
					{/each}
				</div>
			</Card>
		{/if}

		<!-- No Results Message -->
		{#if isOpen && searchQuery.trim() && results.length === 0}
			<Card class="absolute top-full right-0 left-0 z-50 mt-2 border shadow-lg">
				<div class="text-muted-foreground p-4 text-center">
					<SearchIcon class="mx-auto mb-2" size={24} />
					<p class="text-sm">No results found for "{searchQuery}"</p>
				</div>
			</Card>
		{/if}
	{/await}
</div>
