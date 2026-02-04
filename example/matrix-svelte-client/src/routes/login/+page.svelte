<script lang="ts">
	import LoginSteps from '$lib/components/login/login-steps.svelte';
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';
	import { loginState } from '$lib/login-state.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onDestroy, onMount } from 'svelte';
	import {
		type MatrixLoginPayload,
		submitMatrixLoginRequest,
		isLoggedIn
	} from 'tauri-plugin-matrix-svelte-api';
	import { gotoRoomsList } from '$lib/utils.svelte';
	import { resolve } from '$app/paths';

	let { data }: { data: PageData } = $props();

	// svelte-ignore state_referenced_locally
	let { form, host } = data;

	let isLoading = $state(false);
	let skipVerification = $state(false);

	const onSubmit = async (formData: MatrixLoginPayload) => {
		isLoading = true;
		await submitMatrixLoginRequest(formData);
		await awaitUntilLoggedIn();
	};

	let focusUnlistener: UnlistenFn;

	const awaitUntilLoggedIn = async () => {
		// We use the verification state instead of login state because command based calls
		// are more reliable.
		while (!(await isLoggedIn())) {
			const sleep = () => {
				return new Promise((resolve) => setTimeout(resolve, 100));
			};
			console.log('awaiting verification state');
			await sleep();
		}
		loginState.isLoggedIn = true;

		console.log('logged in');

		isLoading = false;
		if (skipVerification) {
			await gotoRoomsList('dm');
		} else {
			await goto(resolve('/verification'));
		}
	};

	onMount(async () => {
		const currentPlatform = platform();
		// For iOS, focusing back on the window causes issues, so we add another event to manage this
		if (currentPlatform === 'ios') {
			focusUnlistener = await listen('new-intent', async () => {
				setTimeout(async () => {
					if (await isLoggedIn()) {
						loginState.isLoggedIn = true;

						console.log('logged in');

						isLoading = false;
						await goto(resolve('/verification'));
					}
				}, 400);
			});
		}
	});

	onDestroy(() => {
		if (focusUnlistener) {
			focusUnlistener();
		}
	});
</script>

<main class="pb-safe relative flex h-full w-full flex-col items-center justify-baseline space-y-10">
	<LoginSteps
		dataForm={form}
		{onSubmit}
		hostname={host}
		bind:isLoading
		{awaitUntilLoggedIn}
		bind:skipVerification
	/>
</main>
