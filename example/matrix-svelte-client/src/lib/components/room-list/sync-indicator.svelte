<script lang="ts">
	import { CircleX, LoaderCircle, WifiOff } from '@lucide/svelte';
	import { loginStore } from '../../../hooks.client';
</script>

{#if loginStore.state.syncServiceState === 'running'}
	<LoaderCircle class="h-6 w-6 animate-spin text-primary" />
{:else if loginStore.state.syncServiceState === 'error'}
	<CircleX class="h-6 w-6 text-destructive" />
{:else if loginStore.state.syncServiceState === 'offline'}
	<WifiOff class="h-6 w-6 text-orange-400" />
{:else if loginStore.state.syncServiceState === 'idle'}
	<p class="text-sm text-muted">Messages synced ✔️</p>
	<!-- We ignore the terminated variant. -->
{/if}
