<script lang="ts">
	import { CircleX, LoaderCircle, WifiOff } from '@lucide/svelte';
	import type { LoginStore } from 'tauri-plugin-matrix-svelte-api';

	let { loginStore }: { loginStore: LoginStore } = $props();
</script>

{#if loginStore.state.syncServiceState === 'running'}
	<LoaderCircle class="text-primary h-6 w-6 animate-spin" />
{:else if loginStore.state.syncServiceState === 'error'}
	<CircleX class="text-destructive h-6 w-6" />
{:else if loginStore.state.syncServiceState === 'offline'}
	<WifiOff class="h-6 w-6 text-orange-400" />
{:else if loginStore.state.syncServiceState === 'idle'}
	<p class="text-muted text-sm">Messages synced ✔️</p>
	<!-- We ignore the terminated variant. -->
{/if}
