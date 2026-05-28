<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Action } from 'svelte/action';
	import type { FrontendTextMessage } from 'tauri-plugin-matrix-svelte-api';

	let { textMessage }: { textMessage: FrontendTextMessage } = $props();

	let textElement = $state<HTMLParagraphElement>();
	let needsToggle = $derived.by(() => {
		if (textElement) {
			return textElement.scrollHeight > textElement.clientHeight;
		} else {
			return false;
		}
	});
	let isExpanded = $state(false);

	// Small AI generated utility that checks the presence of anchor elements,
	// and add _blank target to it.
	const handleMatrixLinks: Action<HTMLElement, string | undefined> = (
		node,
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		formattedBody
	) => {
		const updateLinks = (): void => {
			// Query only anchor tags that have an href attribute
			const links = node.querySelectorAll<HTMLAnchorElement>('a[href]');

			links.forEach((a) => {
				// Force external links to open in a new tab
				a.target = '_blank';

				// Security best practice for target="_blank"
				// Prevents the new page from accessing window.opener
				a.rel = 'noopener noreferrer';
			});
		};

		// Initial run
		updateLinks();

		return {
			// eslint-disable-next-line @typescript-eslint/no-unused-vars
			update(newBody) {
				// Re-run when the content of the message changes
				updateLinks();
			}
		};
	};
</script>

<div class="mt-1">
	{#if textMessage.format == 'org.matrix.custom.html' && textMessage.formatted_body}
		<div class="matrix-message" use:handleMatrixLinks>
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html textMessage.formatted_body}
		</div>
	{:else}
		<p
			bind:this={textElement}
			class="wrap-break-words w-full overflow-hidden text-sm wrap-normal hyphens-auto {isExpanded
				? ''
				: 'line-clamp-5'}"
		>
			{textMessage.body}
		</p>
	{/if}

	{#if needsToggle || isExpanded}
		<button
			onclick={() => (isExpanded = !isExpanded)}
			class="mt-1 text-xs font-semibold text-blue-500"
		>
			{isExpanded ? m.message_show_less() : m.message_view_more()}
		</button>
	{/if}
</div>

<style>
	/* AI generated styles. That should do the job for now */
	.matrix-message {
		line-height: 1.5;
		font-size: 0.95rem;
		word-wrap: break-word;
	}

	/* Headings */
	.matrix-message :global(h1),
	.matrix-message :global(h2),
	.matrix-message :global(h3) {
		margin-top: 1rem;
		margin-bottom: 0.5rem;
		font-weight: 600;
		line-height: 1.25;
	}

	.matrix-message :global(h1) {
		font-size: 1.5rem;
		border-bottom: 1px solid var(--border-color, #333);
		padding-bottom: 0.3rem;
	}
	.matrix-message :global(h2) {
		font-size: 1.25rem;
	}
	.matrix-message :global(h3) {
		font-size: 1.1rem;
	}

	/* Paragraphs & Spacing */
	.matrix-message :global(p) {
		margin-bottom: 0.8rem;
	}

	/* Lists */
	.matrix-message :global(ul),
	.matrix-message :global(ol) {
		margin-bottom: 1rem;
		padding-left: 1.5rem;
	}

	.matrix-message :global(ul ul) {
		list-style-type: circle;
		margin: 0.5rem 0;
	}

	.matrix-message :global(li) {
		display: list-item;
		list-style-type: disc;
		margin-bottom: 0.25rem;
	}

	/* Links */
	.matrix-message :global(a) {
		color: var(--accent);
		text-decoration: none;
	}

	.matrix-message :global(a[target='_blank']::after) {
		content: ' ↗';
		font-size: 0.8em;
		opacity: 0.7;
	}

	.matrix-message :global(a:hover) {
		text-decoration: underline;
	}

	/* Inline Formatting */
	.matrix-message :global(em) {
		font-style: italic;
	}
	.matrix-message :global(strong) {
		font-weight: bold;
	}

	/* Code Blocks & Inline Code */
	.matrix-message :global(code) {
		font-family: 'Fira Code', monospace;
		background-color: rgba(0, 0, 0, 0.2);
		padding: 0.2rem 0.4rem;
		border-radius: 4px;
		font-size: 0.9em;
	}

	.matrix-message :global(pre) {
		background-color: #1d1f21;
		padding: 1rem;
		border-radius: 8px;
		overflow-x: auto;
		margin: 1rem 0;
	}

	.matrix-message :global(pre) :global(code) {
		background-color: transparent;
		padding: 0;
	}

	/* Blockquotes (Common in Matrix replies) */
	.matrix-message :global(blockquote) {
		margin: 0.5rem 0;
		padding-left: 1rem;
		border-left: 4px solid var(--color-accent);
		color: var(--text-muted, #a0a0a0);
		font-style: italic;
	}
</style>
