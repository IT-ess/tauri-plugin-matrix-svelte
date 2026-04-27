<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Download, Paperclip } from '@lucide/svelte';
	import { BaseDirectory, exists } from '@tauri-apps/plugin-fs';
	import { onMount } from 'svelte';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { appCacheDir } from '@tauri-apps/api/path';
	import { m } from '$lib/paraglide/messages';
	import { shareFile } from '@choochmeque/tauri-plugin-sharekit-api';
	import { platform } from '@tauri-apps/plugin-os';
	import {
		fileMessageSourceIsPlain,
		silentSaveMatrixMediaToCacheDir,
		type FileMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: FileMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let alt = $derived(itemContent.filename ?? itemContent.body);

	// State variables
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let fileExistsInFs = $state(false);
	let filePath = $state<string>();

	const handleOpenFile = async () => {
		if (filePath) {
			const target = platform();
			if (target == 'android' || target == 'ios') {
				await shareFile(filePath, {
					mimeType: itemContent.info?.mimetype ?? '',
					title: itemContent.body
				});
			} else {
				await openPath(filePath);
			}
		} else {
			const cache = await appCacheDir();
			await openPath(cache + '/' + alt);
		}
	};

	// Load image function
	const loadFile = async () => {
		if (isLoading) return;

		isLoading = true;
		error = null;
		try {
			filePath = await silentSaveMatrixMediaToCacheDir(
				{
					format: 'File',
					source: fileMessageSourceIsPlain(itemContent)
						? { url: itemContent.url }
						: { file: itemContent.file }
				},
				alt
			);
			handleOpenFile();
			fileExistsInFs = true;
		} catch (err) {
			error = err instanceof Error ? err.message : (err as string);
			console.error('Invoke error:', err);
		} finally {
			isLoading = false;
		}
	};

	onMount(async () => {
		// We only check the downloads folder just in case
		fileExistsInFs = await exists(alt, {
			baseDir: BaseDirectory.AppCache
		});
	});
</script>

<div class="bg-card mt-1 overflow-hidden rounded-lg border">
	{#if !fileExistsInFs}
		<div class="bg-secondary inset-0 flex items-center justify-center">
			<div class="text-center text-white">
				<Button variant="default" size="lg" onclick={() => loadFile()}><Download />{alt}</Button>
			</div>
		</div>

		{#if error}
			<div class="bg-destructive/80 inset-0 flex items-center justify-center">
				<div class="text-center text-white">
					<p class="mb-2 text-sm">
						{m.failed_to_load()} <span class="text-destructive">{error}</span>
					</p>
					<Button variant="secondary" size="sm" onclick={() => loadFile()}
						>{m.button_retry()}</Button
					>
				</div>
			</div>
		{/if}
	{:else}
		<Button size="lg" variant="link" onclick={() => handleOpenFile()}><Paperclip />{alt}</Button>
	{/if}
</div>
