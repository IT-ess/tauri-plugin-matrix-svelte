<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { ReplyIcon, XIcon, SendIcon, Mic } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import { open } from '@tauri-apps/plugin-dialog';
	import type { ChangeEventHandler } from 'svelte/elements';
	import AudioRecorder from '$lib/components/audio/audio-recorder.svelte';
	import { getMediaFromFSPath, getThumbnailInfoFromBlob } from '$lib/utils.svelte';
	import AddMediaDrawer from '../media/add-media-drawer.svelte';
	import {
		createMatrixRequest,
		submitAsyncRequest,
		uploadMedia,
		type RoomStore
	} from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		roomStore: RoomStore;
		// Only for the small timeline in a Ref details
		replyingTo: {
			eventId: string;
			senderName: string;
			content: string;
		} | null;
		handleOpenMediaSendMode: (
			type: 'image' | 'video' | 'file',
			src: string,
			mxcUri: Promise<string>,
			info: {
				filename?: string;
				body?: string;
				mimeType?: string;
				size: number;
				thumbnailInfo?: Promise<{
					blob: Blob | null;
					w: number;
					h: number;
				}>;
			}
		) => void;
		handleSendAudioMessage: (
			blob: Blob,
			duration: number,
			waveform: number[] | null
		) => Promise<void>;
		threadRootEventId?: string;
	};
	let {
		roomStore,
		replyingTo = $bindable(null),
		handleOpenMediaSendMode,
		handleSendAudioMessage,
		threadRootEventId
	}: Props = $props();

	let newMessage = $state('');
	let openMediaDrawer = $state(false);
	let textboxHasChars = $derived(newMessage !== '');

	// Cancel reply
	const cancelReply = () => {
		replyingTo = null;
	};

	// Handle sending new message
	const handleSendTextMessage = async () => {
		if (!newMessage.trim()) return;

		const request = createMatrixRequest.sendTextMessage(roomStore.id, newMessage, {
			replyToId: replyingTo?.eventId,
			threadRootId: threadRootEventId
		});

		await submitAsyncRequest(request);

		newMessage = '';
		replyingTo = null; // Clear reply state after sending
	};

	// Audio related
	let recorderState = $state<'recording' | 'recorded' | 'playing'>('recording');
	let audioPeaks = $state<number[] | null>(null);
	let audioBlob = $state<Blob | null>(null);
	let audioUrl = $state<string | null>(null);
	let audioDuration: number = $state(1);
	let isSendingAudioMessage = $derived(false);

	const sendAudio = async () => {
		if (!audioBlob) return;
		handleSendAudioMessage(audioBlob, audioDuration * 1000, audioPeaks);
		isSendingAudioMessage = false;
	};

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSendTextMessage();
		} else if (e.key === 'Escape' && replyingTo) {
			e.preventDefault();
			cancelReply();
		}
	};

	const handleMediaSendFromPath = async (path: string) => {
		const { filename, mediaSrc, mediaType, mxcUriPromise, blob, mime } =
			await getMediaFromFSPath(path);

		handleOpenMediaSendMode(mediaType, mediaSrc, mxcUriPromise, {
			body: newMessage,
			filename,
			mimeType: mime,
			size: blob.size,
			thumbnailInfo: getThumbnailInfoFromBlob(mediaType, blob) ?? undefined
		});
	};

	const handleMediaSendBlob = async (file: File) => {
		const mediaSrc = URL.createObjectURL(file);
		const mxcUriPromise = uploadMedia(file.type, await file.arrayBuffer());
		const mediaType = file.type.includes('video')
			? 'video'
			: file.type.includes('image')
				? 'image'
				: 'file';

		handleOpenMediaSendMode(mediaType, mediaSrc, mxcUriPromise, {
			body: newMessage,
			filename: file.name,
			mimeType: file.type,
			size: file.size,
			thumbnailInfo: getThumbnailInfoFromBlob(mediaType, file) ?? undefined
		});
	};

	const handleInputFile: ChangeEventHandler<HTMLInputElement> = async (e: Event) => {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (file) {
			await handleMediaSendBlob(file);
		}
		openMediaDrawer = false;
	};

	// Drawer handlers

	const handleOpenGallery = async () => {
		const media = await open({
			multiple: false,
			directory: false,
			title: 'Pick a media',
			// Mobile only
			pickerMode: 'media'
		});
		if (media) {
			await handleMediaSendFromPath(media);
		}
		openMediaDrawer = false;
	};

	const handleTakePhoto = () => {
		const input = document.getElementById('cameraImageInput');
		input?.click();
		openMediaDrawer = false;
	};

	const handleTakeVideo = async () => {
		const input = document.getElementById('cameraVideoInput');
		input?.click();
		openMediaDrawer = false;
	};

	const handleJoinFile = async () => {
		const file = await open({
			multiple: false,
			directory: false,
			title: 'Pick a file',
			// Mobile only
			pickerMode: 'document'
		});
		if (file) {
			await handleMediaSendFromPath(file);
		}
		openMediaDrawer = false;
	};
</script>

<div class="bg-background border-t">
	{#if replyingTo}
		<div class="bg-muted/50 border-b p-3" transition:fade>
			<div class="flex items-start justify-between gap-2">
				<div class="flex min-w-0 flex-1 items-start gap-2">
					<ReplyIcon class="text-muted-foreground mt-0.5 h-4 w-4 shrink-0" />
					<div class="min-w-0 flex-1">
						<div class="text-foreground text-sm font-medium">
							{m.room_input_bar_reply_to()}
							{replyingTo.senderName}
						</div>
						<div class="text-muted-foreground truncate text-sm">
							{replyingTo.content}
						</div>
					</div>
				</div>
				<Button size="icon" variant="ghost" onclick={cancelReply} class="h-6 w-6 shrink-0">
					<XIcon class="h-3 w-3" />
					<span class="sr-only">{m.button_cancel()} {m.button_reply()}</span>
				</Button>
			</div>
		</div>
	{/if}

	<div class="h-16 px-4 pt-4 pb-2">
		{#if isSendingAudioMessage}
			<div class="flex space-x-2">
				<AudioRecorder
					bind:recorderState
					bind:audioUrl
					bind:audioBlob
					bind:duration={audioDuration}
					bind:peaks={audioPeaks}
					onDelete={() => (isSendingAudioMessage = false)}
				/>
				<Button size="icon-lg" onclick={sendAudio} disabled={!audioUrl}>
					<SendIcon class="h-4 w-4" />
					<span class="sr-only">{m.button_send()}</span>
				</Button>
			</div>
		{:else}
			<div class="flex items-center">
				<AddMediaDrawer
					bind:openMediaDrawer
					{handleOpenGallery}
					{handleJoinFile}
					{handleTakePhoto}
					{handleTakeVideo}
					displayTrigger
				/>
				<textarea
					bind:value={newMessage}
					onkeydown={handleKeyDown}
					placeholder={replyingTo
						? m.message_input_reply_placeholder({ name: replyingTo.senderName })
						: m.message_input_placeholder()}
					rows="1"
					class="field-sizing-content max-h-24 flex-1 resize-none overflow-y-auto rounded-md border p-2"
				></textarea>
				{#if textboxHasChars}
					<Button size="icon-lg" onclick={handleSendTextMessage} disabled={!newMessage.trim()}>
						<SendIcon class="h-4 w-4" />
						<span class="sr-only">{m.button_send()}</span>
					</Button>
				{:else}
					<Button variant="ghost" size="icon-lg" onclick={() => (isSendingAudioMessage = true)}>
						<Mic class="size-6" />
					</Button>
				{/if}
			</div>
		{/if}
	</div>
</div>

<!-- Hidden inputs to handle camera results -->
<input
	id="cameraImageInput"
	type="file"
	accept="image/*"
	capture="environment"
	style="display: none"
	multiple={false}
	onchange={handleInputFile}
/>
<input
	id="cameraVideoInput"
	type="file"
	accept="video/*"
	capture="environment"
	style="display: none"
	multiple={false}
	onchange={handleInputFile}
/>
