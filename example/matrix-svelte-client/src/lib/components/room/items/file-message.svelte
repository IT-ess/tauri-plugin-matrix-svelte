<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Download, Paperclip } from '@lucide/svelte';
	import { writeFile, BaseDirectory, exists } from '@tauri-apps/plugin-fs';
	import { onMount } from 'svelte';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { appLocalDataDir } from '@tauri-apps/api/path';
	import {
		fileMessageSourceIsPlain,
		type FileMessageEventContent
	} from 'tauri-plugin-matrix-svelte-api';
	import { getCustomMxcUriFromOriginal } from '$lib/utils.svelte';
	import { Spinner } from '$lib/components/ui/spinner';

	type Props = {
		itemContent: FileMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let alt = $derived(itemContent.filename ?? itemContent.body);

	// State variables
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	let progress = $derived(bytesReceived / totalSize);
	let fileExistsInFs = $state(false);

	let src = $derived(
		(fileMessageSourceIsPlain(itemContent)
			? getCustomMxcUriFromOriginal(itemContent.url, {
					mime: itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.size ?? undefined
					// We force the http scheme, otherwise the tauri http-fetch denies the request.
				})
			: getCustomMxcUriFromOriginal(itemContent.file, {
					mime: itemContent.info?.mimetype ?? undefined,
					size: itemContent.info?.size ?? undefined
				})) as string
	);

	const handleWriteFile = async () => {
		isLoading = true;
		try {
			const res = await fetch(src);
			await writeFile(`download/${alt}`, await res.bytes(), {
				baseDir: BaseDirectory.AppLocalData
			});
			fileExistsInFs = true;
			await handleOpenFile();
		} catch (err) {
			console.error(err);
			error = error;
		} finally {
			isLoading = false;
		}
	};

	const handleOpenFile = async () => {
		const appDir = await appLocalDataDir();
		console.log(appDir);
		await openPath(appDir + '/download/' + alt); // TODO: handle windows path
	};

	onMount(async () => {
		try {
			fileExistsInFs = await exists(alt, {
				baseDir: BaseDirectory.AppLocalData
			});
		} catch (err) {
			console.error(err);
		}
	});
</script>

<div class="bg-card mt-1 overflow-hidden rounded-lg border">
	{#if !fileExistsInFs}
		<Button disabled={isLoading} size="lg" variant="outline" onclick={() => handleWriteFile()}
			><Download />{alt}
			{#if isLoading}
				<Spinner />
			{/if}
		</Button>
	{:else}
		<Button size="lg" variant="link" onclick={() => handleOpenFile()}><Paperclip />{alt}</Button>
	{/if}
	{#if error}
		<p class="text-destructive">Error: {error}</p>
	{/if}
</div>
