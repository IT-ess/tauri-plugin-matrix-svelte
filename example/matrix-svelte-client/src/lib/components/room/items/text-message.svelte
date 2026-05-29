<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { adaptBaseUriToPlatform } from '$lib/utils.svelte';
	import type { Attachment } from 'svelte/attachments';
	import { fetchMatrixPillInfo, type FrontendTextMessage } from 'tauri-plugin-matrix-svelte-api';

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
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	export function handleMatrixLinks(formattedBody: string | undefined): Attachment<HTMLElement> {
		return (element) => {
			// Query links inside the freshly injected/updated DOM element
			const links = element.querySelectorAll<HTMLAnchorElement>('a[href]');

			links.forEach(async (a) => {
				// Avoid preloading any data
				a.setAttribute('data-sveltekit-preload-data', 'off');
				a.setAttribute('data-sveltekit-preload-code', 'off');

				if (a.href.startsWith('https://matrix.to') || a.href.startsWith('matrix:')) {
					try {
						const info = await fetchMatrixPillInfo(a.href);

						a.className = `mx-pill mx-pill--${info.kind}`;
						// Matrix links are already found by the backend and should not have target or rel attributes
						// a.removeAttribute('target'); // Matrix links stay inside or update app state
						// a.removeAttribute('rel');
						let avatarHtml = '';
						let nameHtml = '';
						if (info.kind == 'room') {
							const [preview, via] = info.payload;
							avatarHtml = preview.avatar_url
								? `<img class="mx-pill__avatar" src="${adaptBaseUriToPlatform(preview.avatar_url)}" alt="${preview.name}" />`
								: `<span class="mx-pill__avatar-fallback">${(preview.name ?? '?').charAt(0)}</span>`;
							nameHtml = `<span class="mx-pill__name">${preview.name ?? preview.canonical_alias ?? preview.room_id}</span>`;

							if (preview.state && preview.state == 'Joined') {
								// Point to the room if already joined
								a.href = `/room?id=${encodeURIComponent(preview.room_id)}${preview.avatar_url ? '&avatar=' + encodeURIComponent(preview.avatar_url) : ''}#bottomscroll`;
							} else {
								// display preview instead
								a.href = `/room-preview?data=${encodeURIComponent(JSON.stringify(preview))}&via=${encodeURIComponent(JSON.stringify(via))}`;
							}
						} else {
							// Point to the user profile
							a.href = `/profile?id=${encodeURIComponent(info.payload.user_id)}`;
							avatarHtml = info.payload.avatar_url
								? `<img class="mx-pill__avatar" src="${adaptBaseUriToPlatform(info.payload.avatar_url)}" alt="${info.payload.username}" />`
								: `<span class="mx-pill__avatar-fallback">${(info.payload.username ?? '?').charAt(0)}</span>`;
							nameHtml = `<span class="mx-pill__name">${info.payload.username ?? info.payload.user_id}</span>`;
						}

						// Swap the content smoothly without breaking browser node flows
						a.innerHTML = `${avatarHtml}${nameHtml}`;

						return; // Skip adding _blank
					} catch (err) {
						console.error(err);
					}
				}
			});
		};
	}
</script>

<div class="mt-1">
	{#if textMessage.format == 'org.matrix.custom.html' && textMessage.formatted_body}
		<div class="matrix-message" {@attach handleMatrixLinks(textMessage.formatted_body)}>
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

	/* Base Pill Styling */
	.matrix-message :global(.mx-pill) {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0 0.6rem 0 0.25rem;

		/* Using theme radius token */
		border-radius: var(--radius);

		font-weight: 500;
		font-size: 0.85rem;
		line-height: 1.4rem;
		text-decoration: none !important;
		vertical-align: middle;

		/* Clean transition state mapping */
		transition:
			background-color 0.15s ease,
			color 0.15s ease,
			border-color 0.15s ease;
		max-width: 220px;
		border: 1px solid transparent;
	}

	/* Hover Interaction: Switches safely to your system's accent tokens */
	.matrix-message :global(.mx-pill:hover) {
		background-color: var(--accent);
		color: var(--accent-foreground);
		border-color: var(--border);
	}

	/* 1. User Pills (Uses your colorful Primary token) */
	.matrix-message :global(.mx-pill--user) {
		/* Mixes 12% of primary color with transparency for a modern translucent tag look */
		background-color: color-mix(in oklch, var(--primary) 12%, transparent);
		color: var(--primary);
	}

	/* 2. Room Pills (Uses your elegant structural Secondary token) */
	.matrix-message :global(.mx-pill--room) {
		background-color: var(--secondary);
		color: var(--secondary-foreground);
		border-color: var(--border);
	}

	/* 3. Space Pills (Uses Muted structural tokens to feel like a folder/utility) */
	.matrix-message :global(.mx-pill--space) {
		background-color: var(--muted);
		color: var(--muted-foreground);
		border-color: var(--border);
	}

	/* Avatar Layout */
	.matrix-message :global(.mx-pill__avatar) {
		width: 16px;
		height: 16px;
		/* Nested border-radius calculation for perfect visual harmony */
		border-radius: calc(var(--radius) - 2px);
		object-fit: cover;
		display: inline-block;
	}

	/* Fallback Initials Avatar */
	.matrix-message :global(.mx-pill__avatar-fallback) {
		width: 16px;
		height: 16px;
		border-radius: calc(var(--radius) - 2px);
		font-size: 0.65rem;
		font-weight: 700;
		display: flex;
		align-items: center;
		justify-content: center;
		text-transform: uppercase;
	}

	/* Context-aware colors for fallback avatars to ensure high contrast */
	.matrix-message :global(.mx-pill--user .mx-pill__avatar-fallback) {
		background-color: var(--primary);
		color: var(--primary-foreground);
	}

	.matrix-message :global(.mx-pill--room .mx-pill__avatar-fallback),
	.matrix-message :global(.mx-pill--space .mx-pill__avatar-fallback) {
		background-color: var(--foreground);
		color: var(--background);
	}

	/* Truncated Name Text */
	.matrix-message :global(.mx-pill__name) {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
