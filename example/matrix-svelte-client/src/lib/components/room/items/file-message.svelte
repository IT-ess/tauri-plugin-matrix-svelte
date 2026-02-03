<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Channel, invoke } from '@tauri-apps/api/core';
	import { Download, Paperclip } from '@lucide/svelte';
	import { writeFile, BaseDirectory, exists } from '@tauri-apps/plugin-fs';
	import { onMount } from 'svelte';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { appLocalDataDir } from '@tauri-apps/api/path';
	import { m } from '$lib/paraglide/messages';
	import {
		fileMessageSourceIsPlain,
		type FileMessageEventContent,
		type MediaStreamEvent
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		itemContent: FileMessageEventContent;
	};

	let { itemContent }: Props = $props();

	let alt = $derived(itemContent.filename ?? itemContent.body);

	// State variables
	let isLoaded = $state(false);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let fileBuffer = $state<Uint8Array>();
	let totalSize = $derived(itemContent.info?.size ?? 1);
	let bytesReceived = $state(0);
	let progress = $derived(bytesReceived / totalSize);
	let fileExistsInFs = $state(false);

	const handleWriteFile = async () => {
		if (fileBuffer) {
			await writeFile(`download/${alt}`, fileBuffer, {
				baseDir: BaseDirectory.AppLocalData
			});
			fileExistsInFs = true;
			await handleOpenFile();
		}
	};

	const handleOpenFile = async () => {
		const appDir = await appLocalDataDir();
		console.log(appDir);
		await openPath(appDir + '/download/' + alt); // TODO: handle windows path
	};

	// Load image function
	const loadFile = async () => {
		if (isLoaded || isLoading) return;

		isLoading = true;
		error = null;
		bytesReceived = 0;

		const chunks: Uint8Array[] = [];
		try {
			const onEvent = new Channel<MediaStreamEvent>();

			onEvent.onmessage = (message) => {
				if (message.event === 'started') {
					console.log(`Starting image fetch, total size: ${totalSize} bytes`);
					return;
				}

				if (message.event === 'chunk') {
					chunks.push(new Uint8Array(message.data.data));
					bytesReceived = message.data.bytesReceived;
					console.log(
						`Received chunk: ${message.data.chunkSize} bytes, total: ${bytesReceived}/${totalSize}`
					);
					return;
				}

				if (message.event === 'finished') {
					// Combine all chunks into a single Uint8Array
					const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
					const combined = new Uint8Array(totalLength);
					let offset = 0;

					for (const chunk of chunks) {
						combined.set(chunk, offset);
						offset += chunk.length;
					}

					fileBuffer = combined;
					isLoaded = true;
					isLoading = false;
					console.log(`Image fetch completed: ${message.data.totalBytes} bytes`);
					return;
				}

				if (message.event === 'error') {
					error = message.data.message;
					isLoading = false;
					console.error('Image fetch error:', message.data.message);
					return;
				}
			};
			if (fileMessageSourceIsPlain(itemContent)) {
				await invoke('plugin:matrix-svelte|fetch_media', {
					mediaRequest: {
						format: 'File',
						source: { url: itemContent.url }
					},
					onEvent
				});
			} else {
				await invoke('plugin:matrix-svelte|fetch_media', {
					mediaRequest: {
						format: 'File',
						source: { file: itemContent.file }
					},
					onEvent
				});
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error occurred';
			isLoading = false;
			console.error('Invoke error:', err);
		}
	};

	onMount(async () => {
		fileExistsInFs = await exists(alt, {
			baseDir: BaseDirectory.AppLocalData
		});
	});
</script>

<div class="bg-card mt-1 overflow-hidden rounded-lg border">
	{#if !fileExistsInFs}
		{#if !isLoaded}
			<div class="bg-secondary inset-0 flex items-center justify-center">
				<div class="text-center text-white">
					<Button variant="default" size="lg" onclick={() => loadFile()}><Download />{alt}</Button>
				</div>
			</div>

			{#if isLoading}
				<div class="inset-0 flex items-center justify-center bg-black/20">
					<div class="rounded-full bg-white/90 px-3 py-1 text-xs">
						{Math.round(progress * 100)}%
					</div>
				</div>
			{/if}

			{#if error}
				<div class="bg-destructive/80 inset-0 flex items-center justify-center">
					<div class="text-center text-white">
						<p class="mb-2 text-sm">{m.failed_to_load()}</p>
						<Button variant="secondary" size="sm" onclick={() => loadFile()}
							>{m.button_retry()}</Button
						>
					</div>
				</div>
			{/if}
		{:else}
			<!-- Loaded File from backend but not written yet -->
			<Button size="lg" variant="link" onclick={() => handleWriteFile()}><Paperclip />{alt}</Button>
		{/if}
	{:else}
		<!-- File already present in FS -->
		<Button size="lg" variant="link" onclick={() => handleOpenFile()}><Paperclip />{alt}</Button>
	{/if}
</div>
