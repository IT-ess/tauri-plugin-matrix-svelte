<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	let { body }: { body: string } = $props();

	let textElement = $state<HTMLParagraphElement>();
	let needsToggle = $derived.by(() => {
		if (textElement) {
			return textElement.scrollHeight > textElement.clientHeight;
		} else {
			return false;
		}
	});
	let isExpanded = $state(false);
</script>

<div class="mt-1">
	<p
		bind:this={textElement}
		class="wrap-break-words w-full overflow-hidden text-sm wrap-anywhere hyphens-auto {isExpanded
			? ''
			: 'line-clamp-5'}"
	>
		{body}
	</p>

	{#if needsToggle || isExpanded}
		<button
			onclick={() => (isExpanded = !isExpanded)}
			class="mt-1 text-xs font-semibold text-blue-500"
		>
			{isExpanded ? m.message_show_less() : m.message_view_more()}
		</button>
	{/if}
</div>
