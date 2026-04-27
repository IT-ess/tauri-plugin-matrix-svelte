<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { beforeNavigate } from '$app/navigation';
	import {
		XIcon,
		SendIcon,
		LoaderIcon,
		ZoomInIcon,
		ZoomOutIcon,
		Paperclip,
		Share,
		Download
	} from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { encodeImageToBlurhash } from './encode-blurhash.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	import { cn } from '$lib/utils.svelte';
	import { shareFile } from '@choochmeque/tauri-plugin-sharekit-api';
	import {
		androidShareMatrixMedia,
		silentSaveMatrixMediaToCacheDir,
		writeMediaToSelectedFolder,
		type FileInfo,
		type ImageInfo,
		type MediaRequestParameters,
		type RoomMessageEventContent,
		type VideoInfo
	} from 'tauri-plugin-matrix-svelte-api';

	type MediaType = 'image' | 'video' | 'file';
	type ViewerMode = 'send' | 'view';

	interface Props {
		src: string;
		mediaType?: MediaType;
		mode?: ViewerMode;
		text?: string;
		filename?: string;
		isLoading?: boolean;
		onClose: () => void;
		onSend: (
			msgtype: RoomMessageEventContent['msgtype'],
			mediaInfo?: VideoInfo | ImageInfo | FileInfo,
			additionalInfo?: { message?: string; waveform?: number[] }
		) => Promise<void>;
		mediaSource?: MediaRequestParameters['source']; // Only required in view mode
	}

	let {
		src,
		mediaType = 'image',
		mode = 'send',
		text = '',
		isLoading = false,
		filename,
		onClose,
		onSend,
		mediaSource
	}: Props = $props();

	let messageText = $state('');
	let zoom = $state(1);
	let showUI = $state(true);
	const MIN_ZOOM = 1;
	const MAX_ZOOM = 4;
	const ZOOM_STEP = 0.5;

	const msgtypeFromType = (type: MediaType): 'm.image' | 'm.video' | 'm.file' => {
		switch (type) {
			case 'image':
				return 'm.image';
			case 'video':
				return 'm.video';
			case 'file':
				return 'm.file';
		}
	};

	let videoEl = $state<HTMLVideoElement | undefined>();
	let imgEl = $state<HTMLImageElement | undefined>();

	const mediaInfoFromType = (type: MediaType): ImageInfo | VideoInfo | FileInfo | undefined => {
		switch (type) {
			case 'image': {
				if (imgEl) {
					const imageInfo: ImageInfo = {
						h: imgEl.naturalHeight,
						w: imgEl.naturalHeight,
						'xyz.amorgan.blurhash': encodeImageToBlurhash(imgEl) ?? null,
						// The size, mimetype and thumbnails are retrieved when reading the blob, not here
						mimetype: null,
						size: null,
						thumbnail_info: null,
						thumbnail_url: '',
						// Not used
						'org.matrix.msc4230.is_animated': false,
						'xyz.amorgan.thumbhash': null
					};
					return imageInfo;
				}
				break;
			}
			case 'video': {
				if (videoEl) {
					const videoInfo: VideoInfo = {
						h: videoEl.videoHeight,
						w: videoEl.videoWidth,
						duration: Math.ceil(videoEl.duration),
						// The size, mimetype and thumbnails are retrieved when reading the blob, not here
						mimetype: null,
						size: null,
						thumbnail_info: null,
						thumbnail_url: '',
						// Not used
						'xyz.amorgan.blurhash': null,
						'xyz.amorgan.thumbhash': null
					};
					return videoInfo;
				}
				break;
			}
			case 'file': {
				const fileInfo: FileInfo = {
					// The size and mimetypes are retrieved when reading the blob, not here
					mimetype: null,
					size: null,
					// Not used
					thumbnail_info: null
				};
				return fileInfo;
			}
		}
	};

	const handleSend = async () => {
		isLoading = true;
		await onSend(msgtypeFromType(mediaType), mediaInfoFromType(mediaType), {
			message: messageText === '' ? undefined : messageText
		});
		messageText = '';
		isLoading = false;
	};

	const handleKeydown = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			onClose?.();
		}
	};

	const handleZoomIn = () => {
		zoom = Math.min(zoom + ZOOM_STEP, MAX_ZOOM);
	};

	const handleZoomOut = () => {
		zoom = Math.max(zoom - ZOOM_STEP, MIN_ZOOM);
	};

	const handleResetZoom = () => {
		zoom = MIN_ZOOM;
	};

	const handleMediaClick = () => {
		showUI = !showUI;
	};

	const handleWheel = (e: WheelEvent) => {
		if (mediaType !== 'image') return;
		e.preventDefault();
		if (e.deltaY < 0) {
			handleZoomIn();
		} else {
			handleZoomOut();
		}
	};

	let exportActionOngoing = $state(false);
	const handleShare = async () => {
		if (!mediaSource) return;

		exportActionOngoing = true;
		try {
			const sanitizedFilename = filename?.replaceAll(' ', '_') ?? 'Refs_file';
			if (target == 'android') {
				await androidShareMatrixMedia({ format: 'File', source: mediaSource }, sanitizedFilename);
			} else if (target == 'ios') {
				const filePath = await silentSaveMatrixMediaToCacheDir(
					{
						format: 'File',
						source: mediaSource
					},
					sanitizedFilename
				);
				await shareFile(filePath);
			}
		} catch (err) {
			console.error(err);
		} finally {
			exportActionOngoing = false;
		}
	};

	const handleDownload = async () => {
		if (!mediaSource) return;
		exportActionOngoing = true;
		try {
			const sanitizedFilename = filename?.replaceAll(' ', '_') ?? 'Refs_file';
			await writeMediaToSelectedFolder({ format: 'File', source: mediaSource }, sanitizedFilename);
		} catch (err) {
			console.error(err);
		} finally {
			exportActionOngoing = false;
		}
	};

	// Before navigate interceptor

	beforeNavigate(({ cancel }) => {
		cancel();
		onClose();
	});

	const target = platform();
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-50 bg-black/95" transition:fade={{ duration: 200 }}>
	{#if showUI}
		<div
			class="pt-safe absolute top-0 right-0 left-0 z-60 flex items-center justify-between border-b border-white/10 bg-black/50 backdrop-blur-sm transition-all duration-200"
			transition:fade={{ duration: 200 }}
		>
			<Button
				size="icon"
				variant="ghost"
				class="size-10 rounded-full text-white hover:bg-white/10"
				onclick={(e) => {
					e.stopImmediatePropagation();
					onClose();
				}}
			>
				<XIcon size={24} />
			</Button>
			<div class="w-full">
				<p class={cn('mb-0.5 text-white', target == 'ios' && !mediaSource ? 'ml-1' : 'ml-10')}>
					{filename}
				</p>
			</div>
			{#if mediaSource}
				<div class="flex gap-1">
					<Button
						disabled={exportActionOngoing}
						onclick={handleShare}
						variant="ghost"
						class="text-white"><Share /></Button
					>
					{#if target != 'ios'}
						<Button
							disabled={exportActionOngoing}
							onclick={handleDownload}
							variant="ghost"
							class="text-white"><Download /></Button
						>
					{/if}
				</div>
			{/if}
		</div>
	{/if}

	<div
		class="absolute inset-0 flex items-center justify-center overflow-hidden"
		onwheel={mediaType === 'image' ? handleWheel : null}
	>
		{#if mediaType === 'image'}
			<div
				class="h-screen-safe py-safe-offset-8 relative flex w-full items-center justify-center overflow-auto"
			>
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<img
					bind:this={imgEl}
					{src}
					alt="Media"
					class="cursor-pointer object-contain transition-transform duration-200"
					style="transform: scale({zoom})"
					onclick={handleMediaClick}
					transition:scale={{ duration: 300 }}
				/>
			</div>

			{#if showUI}
				<div
					class="top-safe-offset-12 absolute right-4 flex flex-col gap-2 transition-all duration-200 sm:flex-row"
					transition:fade={{ duration: 200 }}
				>
					<Button
						size="icon"
						variant="ghost"
						class="size-10 rounded-full text-white hover:bg-white/20"
						onclick={handleZoomIn}
						title="Zoom in"
						disabled={zoom >= MAX_ZOOM}
					>
						<ZoomInIcon size={20} />
					</Button>
					{#if zoom > MIN_ZOOM}
						<Button
							size="icon"
							variant="ghost"
							class="size-10 rounded-full text-white hover:bg-white/20"
							onclick={handleResetZoom}
							title="Reset zoom"
						>
							<span class="text-xs font-semibold">{Math.round(zoom * 100)}%</span>
						</Button>
						<Button
							size="icon"
							variant="ghost"
							class="size-10 rounded-full text-white hover:bg-white/20"
							onclick={handleZoomOut}
							title="Zoom out"
						>
							<ZoomOutIcon size={20} />
						</Button>
					{/if}
				</div>
			{/if}

			{#if mode === 'view' && text && showUI}
				<div
					class="pointer-events-none absolute right-0 bottom-0 left-0 flex items-end justify-center"
					transition:fade={{ duration: 200 }}
				>
					<div
						class="pb-safe-offset-4 line-clamp-2 w-full rounded-lg bg-black/30 pt-4 text-center text-sm leading-relaxed text-white backdrop-blur-sm sm:text-base"
					>
						{text}
					</div>
				</div>
			{/if}
		{:else if mediaType === 'video'}
			<!-- svelte-ignore a11y_media_has_caption -->
			<video
				bind:this={videoEl}
				{src}
				controls
				autoplay
				class="h-full w-full object-contain"
				transition:scale={{ duration: 300 }}
				onclick={handleMediaClick}
			></video>
		{:else if mediaType === 'file'}
			<div class="flex h-full w-full flex-col items-center justify-center text-white">
				<Paperclip class="size-10" />
				<p class="truncate text-2xl">{filename}</p>
			</div>
		{/if}
	</div>

	{#if mode === 'send' && showUI}
		<div
			class="pb-safe-offset-2 absolute right-0 bottom-0 left-0 flex flex-col gap-3 border-t border-white/10 bg-black/50 p-3 backdrop-blur-sm transition-all duration-200"
			transition:fade={{ duration: 200 }}
		>
			<Textarea
				bind:value={messageText}
				placeholder="Add a message (optional)..."
				class="max-h-32 min-h-20 resize-none rounded-lg border-white/20 bg-white/10 text-sm text-white placeholder:text-white/50 sm:text-base"
				disabled={isLoading}
			/>
			<div class="flex gap-2">
				<Button
					variant="destructive"
					class="flex-1"
					onclick={(e) => {
						e.stopImmediatePropagation();
						onClose();
					}}
					disabled={isLoading}>{m.button_cancel()}</Button
				>
				<Button class="flex-1" onclick={handleSend} disabled={isLoading}>
					{#if isLoading}
						<LoaderIcon class="animate-spin" size={18} />
					{:else}
						<SendIcon size={18} />
					{/if}
					{m.button_send()}
				</Button>
			</div>
		</div>
	{/if}
</div>
