<script lang="ts">
	import { PauseIcon, PlayIcon } from '@lucide/svelte';
	import AudioWaveform from 'svelte-audio-waveform';
	import { Button } from '../ui/button';
	import { MediaQuery } from 'svelte/reactivity';

	let {
		src,
		peaks,
		initialDuration,
		isOwn
	}: { src: string; peaks?: number[]; initialDuration?: number | null; isOwn: boolean } = $props();

	type State = 'idle' | 'playing';
	const large = new MediaQuery('min-width: 800px');

	let currentState = $state<State>('idle');
	// svelte-ignore state_referenced_locally
	let audioUrl = $state(src);
	let audioElement: HTMLAudioElement | null = null;
	// svelte-ignore state_referenced_locally
	let duration = $state(initialDuration ? initialDuration / 1000 : 1);
	let currentTime = $state(0);
	// For some reason the progress goes three times too fast ??
	// Don't know why it works but it does 🥴
	let progress = $derived(currentTime / duration / (large.current ? 2 : 3));
	let formatedCurrentTime = $derived(formatTime(currentTime));

	function formatTime(seconds: number): string {
		if (!isFinite(seconds) || seconds < 0) {
			return '00:00';
		}

		const hrs = Math.floor(seconds / 3600);
		const mins = Math.floor((seconds % 3600) / 60);
		const secs = Math.floor(seconds % 60);

		const formattedMins = hrs > 0 ? String(mins).padStart(2, '0') : String(mins);
		const formattedSecs = String(secs).padStart(2, '0');

		return hrs > 0
			? `${hrs}:${formattedMins}:${formattedSecs}` // HH:MM:SS
			: `${formattedMins}:${formattedSecs}`; // MM:SS
	}

	function handleSeek(event: MouseEvent) {
		if (!audioElement) return;

		const target = (event.currentTarget as HTMLDivElement).getBoundingClientRect();
		const position = event.pageX - target.left;
		const rate = position / target.width;
		const newTime = rate * duration;
		audioElement.currentTime = newTime;
	}

	const handleAudioEnded = () => {
		currentState = 'idle';
	};

	const togglePlayback = () => {
		if (!audioElement) return;

		if (currentState === 'playing') {
			audioElement.pause();
		} else {
			audioElement.play();
			currentState = 'playing';
		}
	};
</script>

<audio
	bind:this={audioElement}
	src={audioUrl}
	ontimeupdate={(e) => {
		currentTime = e.currentTarget.currentTime;
	}}
	onloadedmetadata={(e) => {
		duration = e.currentTarget.duration;
	}}
	onended={handleAudioEnded}
	controls={peaks === undefined}
></audio>

{#if peaks}
	<div class="flex w-full min-w-0 flex-1 items-center">
		<Button
			size="icon"
			variant="ghost"
			class="size-8 shrink-0 rounded-full transition-colors hover:bg-primary/10"
			onclick={togglePlayback}
			title={currentState === 'playing' ? 'Pause' : 'Play'}
		>
			{#if currentState === 'playing'}
				<PauseIcon size={18} />
			{:else}
				<PlayIcon size={18} />
			{/if}
		</Button>
		<AudioWaveform
			barWidth={2}
			{peaks}
			height={80}
			width={large.current ? 600 : 350}
			position={progress}
			on:click={handleSeek}
			progressColor={isOwn ? 'oklch(0.9263 0.0395 295.1)' : 'oklch(0.2273 0.0647 8.71)'}
			gradientColors={[]}
			progressGradientColors={[]}
		/>
		<div class="ml-2 flex w-full items-center justify-between">
			{#if currentTime === 0}
				{formatTime(duration)}
			{:else}
				{formatedCurrentTime}
			{/if}
		</div>
	</div>
{/if}
