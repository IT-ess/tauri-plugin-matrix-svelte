<script lang="ts">
	import { onMount } from 'svelte';

	let { dataArray = $bindable(new Uint8Array(0)) }: Props = $props();

	let canvasRef: HTMLCanvasElement | null = null;

	type Props = {
		dataArray: Uint8Array;
	};

	onMount(() => {
		const canvas = canvasRef;
		if (!canvas) return;

		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const resizeCanvas = () => {
			const rect = canvas.getBoundingClientRect();
			canvas.width = Math.floor(rect.width * window.devicePixelRatio);
			canvas.height = Math.floor(rect.height * window.devicePixelRatio);
			ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
		};

		resizeCanvas();

		const drawWaveform = () => {
			const width = canvas.width / window.devicePixelRatio;
			const height = canvas.height / window.devicePixelRatio;

			// Clear canvas with transparent background
			ctx.clearRect(0, 0, width, height);

			// Draw waveform bars
			const barWidth = 3;
			const barGap = 2;
			const barCount = Math.floor(width / (barWidth + barGap));

			ctx.fillStyle = 'oklch(0.2273 0.0647 8.71)';

			for (let i = 0; i < barCount; i++) {
				const dataIndex = Math.floor((i / barCount) * dataArray.length);
				const value = dataArray[dataIndex] || 0;
				const barHeight = (value / 255) * height * 0.85;
				const x = i * (barWidth + barGap);
				const y = (height - barHeight) / 2;

				// Rounded top corners
				ctx.beginPath();
				ctx.roundRect(x, y, barWidth, barHeight, 2);
				ctx.fill();
			}

			requestAnimationFrame(drawWaveform);
		};

		drawWaveform();

		const resizeObserver = new ResizeObserver(() => {
			resizeCanvas();
		});

		resizeObserver.observe(canvas);

		return () => {
			resizeObserver.disconnect();
		};
	});
</script>

<canvas
	bind:this={canvasRef}
	class="h-8 min-w-0 flex-1 self-center rounded-lg border border-primary/10 bg-muted/30"
></canvas>
