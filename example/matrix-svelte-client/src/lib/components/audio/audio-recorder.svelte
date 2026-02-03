<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { TrashIcon, PauseIcon, PlayIcon } from '@lucide/svelte';
	import AudioWaveform from 'svelte-audio-waveform';
	import RecordingWaveform from './recording-waveform.svelte';

	type State = 'recording' | 'recorded' | 'playing';

	let {
		recorderState = $bindable('recording'),
		audioUrl = $bindable(null),
		audioBlob = $bindable(null),
		duration = $bindable(1),
		peaks = $bindable(null),
		onDelete
	}: {
		recorderState: State;
		audioUrl: string | null;
		audioBlob: Blob | null;
		duration: number;
		peaks: number[] | null;
		onDelete: () => void;
	} = $props();

	let mediaRecorder: MediaRecorder | null = null;
	let audioChunks: Blob[] = $state([]);
	let audioContext: AudioContext | null = null;
	let analyser: AnalyserNode | null = null;
	let dataArray: Uint8Array = $state(new Uint8Array(0));
	let audioElement: HTMLAudioElement | null = null;
	let animationId: number | null = null;
	let currentTime: number = $state(0);
	let progress = $derived(currentTime / duration / 3);

	onMount(async () => {
		try {
			const stream = await navigator.mediaDevices.getUserMedia({
				audio: true,
				preferCurrentTab: true,
				video: false
			});
			// We force audio/mp4 format since WebKit iOS only supports this format.
			mediaRecorder = new MediaRecorder(stream, { mimeType: 'audio/mp4' });

			mediaRecorder.ondataavailable = (event) => {
				audioChunks.push(event.data);
			};

			mediaRecorder.onstop = async () => {
				audioBlob = new Blob(audioChunks, { type: 'audio/mp4' });
				audioBlob
					.arrayBuffer()
					.then((buffer) => {
						if (audioContext) {
							return audioContext?.decodeAudioData(buffer);
						}
					})
					.then((a) => (peaks = a ? getPeaksForPlaybackWaveform(a) : null));
				audioUrl = URL.createObjectURL(audioBlob);
				recorderState = 'recorded';
				audioChunks = [];
			};

			audioContext = new AudioContext();
			analyser = audioContext.createAnalyser();
			analyser.fftSize = 256;
			dataArray = new Uint8Array(analyser.frequencyBinCount);

			// Connect microphone stream to analyser
			const source = audioContext.createMediaStreamSource(stream);
			source.connect(analyser);

			startRecording();
		} catch (error) {
			console.error('Error accessing microphone:', error);
		}
	});

	onDestroy(() => {
		if (mediaRecorder?.stream) {
			mediaRecorder.stream.getTracks().forEach((track) => track.stop());
		}
		if (audioUrl) URL.revokeObjectURL(audioUrl);
	});

	const startRecording = () => {
		if (!mediaRecorder) return;
		audioChunks = [];
		mediaRecorder.start();
		recorderState = 'recording';
		animateWaveform();
	};

	const stopRecording = () => {
		if (!mediaRecorder) return;
		mediaRecorder.stop();
	};

	const animateWaveform = () => {
		if (!analyser) return;
		analyser.getByteFrequencyData(dataArray as Uint8Array<ArrayBuffer>);
		animationId = requestAnimationFrame(animateWaveform);
	};

	function getPeaksForPlaybackWaveform(buffer: AudioBuffer): number[] {
		if (0 >= buffer.numberOfChannels) {
			throw new Error(`Channel ${0} does not exist in audio buffer`);
		}

		const decodedAudioData = buffer.getChannelData(0);
		const bucketDataSize = Math.floor(decodedAudioData.length / 200);
		const buckets: number[] = [];

		for (let i = 0; i < 200; i++) {
			const startingPoint = i * bucketDataSize;
			const endingPoint = startingPoint + bucketDataSize;

			let max = 0;
			for (let j = startingPoint; j < endingPoint; j++) {
				const absolute = Math.abs(decodedAudioData[j]);
				if (absolute > max) {
					max = absolute;
				}
			}

			buckets.push(max / 2);
		}

		return buckets;
	}

	const togglePlayback = () => {
		if (!audioElement) return;

		if (recorderState === 'playing') {
			audioElement.pause();
			recorderState = 'recorded';
			if (animationId) cancelAnimationFrame(animationId);
		} else {
			audioElement.play();
			recorderState = 'playing';
		}
	};

	const handleAudioEnded = () => {
		recorderState = 'recorded';
		if (animationId) cancelAnimationFrame(animationId);
	};

	const deleteRecording = () => {
		if (audioElement) audioElement.pause();
		onDelete();
		if (audioUrl) URL.revokeObjectURL(audioUrl);
		audioUrl = '';
		audioChunks = [];
		currentTime = 0;
		duration = 0;
		if (animationId) cancelAnimationFrame(animationId);
	};

	const formatTime = (seconds: number) => {
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	};
	let formatedCurrentTime = $derived(formatTime(currentTime));
</script>

<audio
	bind:this={audioElement}
	src={audioUrl}
	onended={handleAudioEnded}
	ontimeupdate={(e) => {
		currentTime = e.currentTarget.currentTime;
	}}
	onloadedmetadata={(e) => {
		duration = e.currentTarget.duration;
	}}
></audio>

<div
	class="shadow-lg-lg-lg shadow-lg-lg-primary/5 flex flex-1 items-center gap-2 overflow-hidden pb-2"
>
	<Button
		size="icon"
		variant="ghost"
		class="size-9 shrink-0 rounded-full transition-colors hover:bg-destructive/10 hover:text-destructive"
		onclick={deleteRecording}
		title="Delete recording"
	>
		<TrashIcon size={18} />
	</Button>

	<div class="flex min-w-0 flex-1 items-center gap-2">
		{#if recorderState === 'recording'}
			<Button
				size="icon"
				class="size-9 shrink-0 rounded-full transition-all"
				onclick={stopRecording}
				title="Stop recording"
			>
				<div class="size-2 rounded-full bg-current"></div>
			</Button>
			<RecordingWaveform {dataArray} />
		{:else}
			<Button
				size="icon"
				variant="ghost"
				class="size-8 shrink-0 rounded-full transition-colors hover:bg-primary/10"
				onclick={togglePlayback}
				title={recorderState === 'playing' ? 'Pause' : 'Play'}
			>
				{#if recorderState === 'playing'}
					<PauseIcon size={18} />
				{:else}
					<PlayIcon size={18} />
				{/if}
			</Button>
			<AudioWaveform
				barWidth={2}
				peaks={peaks ?? []}
				height={120}
				width={550}
				position={progress}
				on:click={() => {}}
				progressColor="oklch(0.2273 0.0647 8.71)"
				gradientColors={[]}
				progressGradientColors={[]}
			/>
			<div class="flex w-full items-center justify-between">
				{#if currentTime === 0}
					{formatTime(duration)}
				{:else}
					{formatedCurrentTime}
				{/if}
			</div>
		{/if}
	</div>
</div>
