<script lang="ts">
	import LoginSteps from '$lib/components/login/login-steps.svelte';
	import {
		loginAndCreateNewSession,
		type MatrixClientConfig
	} from 'tauri-plugin-matrix-svelte-api';
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';
	import Button from '$lib/components/ui/button/button.svelte';

	let { data }: { data: PageData } = $props();

	let { form, host } = data;

	let isLoading = $state(false);

	const onSubmit = async (formData: MatrixClientConfig) => {
		isLoading = true;
		await loginAndCreateNewSession(formData);
		// TODO: handle connection errors gracefully
		isLoading = false;
		await goto('/');
	};
</script>

<main class="container">
	<LoginSteps dataForm={form} {onSubmit} hostname={host} bind:isLoading></LoginSteps>
	<Button onclick={() => goto('/')}>Goto home (debug)</Button>
</main>
