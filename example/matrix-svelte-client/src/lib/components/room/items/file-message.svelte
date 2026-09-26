<script lang="ts">
	import * as Attachment from '$lib/components/ui/attachment';
	import { Spinner } from '$lib/components/ui/spinner';
	import { DownloadIcon, ExternalLinkIcon, FileIcon, RotateCwIcon } from '@lucide/svelte';
	import { getLocale } from '$lib/paraglide/runtime';
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

	const formatSize = (bytes: number) => {
		const i = Math.max(0, Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 3));
		return new Intl.NumberFormat(getLocale(), {
			style: 'unit',
			unit: ['byte', 'kilobyte', 'megabyte', 'gigabyte'][i],
			unitDisplay: 'narrow',
			maximumFractionDigits: 1
		}).format(bytes / 1024 ** i);
	};

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

<Attachment.Root state={error ? 'error' : isLoading ? 'processing' : 'done'} class="mt-1">
	<Attachment.Media>
		{#if isLoading}<Spinner />{:else}<FileIcon />{/if}
	</Attachment.Media>
	<Attachment.Content>
		<Attachment.Title>{alt}</Attachment.Title>
		<Attachment.Description>
			{error
				? `${m.failed_to_load()} ${error}`
				: itemContent.info?.size
					? formatSize(itemContent.info.size)
					: (itemContent.info?.mimetype ?? '')}
		</Attachment.Description>
	</Attachment.Content>
	<Attachment.Actions>
		<Attachment.Action
			aria-label={fileExistsInFs ? `Open ${alt}` : `Download ${alt}`}
			disabled={isLoading}
			onclick={() => (fileExistsInFs ? handleOpenFile() : loadFile())}
		>
			{#if fileExistsInFs}
				<ExternalLinkIcon />
			{:else if error}
				<RotateCwIcon />
			{:else}
				<DownloadIcon />
			{/if}
		</Attachment.Action>
	</Attachment.Actions>
</Attachment.Root>
