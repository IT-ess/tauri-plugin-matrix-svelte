<script lang="ts">
	import '../app.css';
	import { setupViewTransition } from 'sveltekit-view-transition';
	import { onDestroy, onMount } from 'svelte';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { Button } from '$lib/components/ui/button/';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Toaster, toast } from 'svelte-sonner';
	import { MediaQuery } from 'svelte/reactivity';
	import type { LayoutProps } from './$types';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { loginState } from '$lib/login-state.svelte';
	import { m } from '$lib/paraglide/messages';
	import '@saurl/tauri-plugin-safe-area-insets-css-api';
	import { loginStore } from '../hooks.client';
	import { platform } from '@tauri-apps/plugin-os';
	import {
		isLoggedIn,
		MatrixSvelteEmitEvent,
		MatrixSvelteListenEvent,
		type ToastNotificationEventType,
		type VerificationEmojisEventType
	} from 'tauri-plugin-matrix-svelte-api';

	let { children }: LayoutProps = $props();

	if (platform() !== 'linux') {
		setupViewTransition();
	}

	let displayEmojiVerificationModal = $state(false);

	let verificationEmojis = $state('');
	const isDesktop = new MediaQuery('(min-width: 768px)');

	let emojisUnlistener: UnlistenFn;
	let toastUnlistener: UnlistenFn;

	onMount(async () => {
		// Necessary for first init
		if (!loginState.isLoggedIn && page.route.id !== '/login') {
			while (!(await isLoggedIn())) {
				if (loginStore.state.state == 'awaitingForHomeserver') {
					await goto('/login');
					break;
				}
				const sleep = () => {
					return new Promise((resolve) => setTimeout(resolve, 300));
				};
				console.log('awaiting for login state');
				await sleep();
			}
			// If the client is active we set the context to loggedIn
			// (by calling the function again to avoid breaking the loop and setting true)
			loginState.isLoggedIn = await isLoggedIn();
		}

		emojisUnlistener = await listen<VerificationEmojisEventType>(
			MatrixSvelteListenEvent.VerificationStart,
			(event) => {
				console.log(
					'Matrix verification event received. Beginning verification. Emojis:',
					event.payload.emojis
				);
				displayEmojiVerificationModal = true;
				verificationEmojis = event.payload.emojis;
			}
		);

		toastUnlistener = await listen<ToastNotificationEventType>(
			MatrixSvelteListenEvent.ToastNotification,
			(event) => {
				switch (event.payload.variant) {
					case 'success':
						return toast.success(event.payload.message);
					case 'error':
						return toast.error(event.payload.message);
					case 'info':
						return toast.info(event.payload.message);
					case 'warning':
						return toast.warning(event.payload.message);
					case 'description':
						return toast.message(event.payload.message, {
							description: event.payload.description ?? 'Missing description'
						});
					default:
						return toast(event.payload.message);
				}
			}
		);
	});

	onDestroy(() => {
		emojisUnlistener();
		toastUnlistener();
	});
</script>

<main class="h-screen w-screen">
	{@render children()}
</main>

<Toaster
	toastOptions={{ class: 'mt-safe' }}
	richColors
	expand={isDesktop.current}
	position={isDesktop.current ? 'top-right' : 'top-center'}
	closeButton
/>

<Dialog.Root bind:open={displayEmojiVerificationModal}>
	<Dialog.Content class="z-60 max-w-[80%] rounded-md">
		<Dialog.Header>
			<Dialog.Title>{m.popup_verification_please_verify()}</Dialog.Title>
			<Dialog.Description>
				{m.popup_verification_instructions()}
				<br />
				<span class="text-3xl">{verificationEmojis}</span>
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button
				variant="default"
				type="submit"
				onclick={() => {
					emit(MatrixSvelteEmitEvent.VerificationResult, {
						confirmed: true
					});
					displayEmojiVerificationModal = false;
				}}>{m.button_confirm()}</Button
			>
			<Button
				variant="destructive"
				type="submit"
				onclick={() => {
					emit(MatrixSvelteEmitEvent.VerificationResult, {
						confirmed: false
					});
					displayEmojiVerificationModal = false;
				}}>{m.button_cancel()}</Button
			>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
