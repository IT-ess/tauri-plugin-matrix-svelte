<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { events } from 'tauri-plugin-matrix-svelte-api';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Toaster, toast } from 'svelte-sonner';
	import { MediaQuery } from 'svelte/reactivity';

	let { children, data } = $props();

	$effect(() => {
		if (data.loginStore.state.state == 'awaitingForLogin') {
			goto('/login');
		} else {
			data.loginStore.state.state = 'loggedIn';
		}
	});

	let displayEmojiVerificationModal = $state(false);
	let verificationEmojis = $state('');
	const isDesktop = new MediaQuery('(min-width: 768px)');

	let emojisUnlistener: UnlistenFn;
	let toastUnlistener: UnlistenFn;

	onMount(async () => {
		// Do not register if already verified ?
		emojisUnlistener = await listen<events.VerificationEmojisEventType>(
			events.MatrixSvelteListenEvent.VerificationStart,
			(event) => {
				console.log(
					'Matrix verification event received. Beginning verification. Emojis:',
					event.payload.emojis
				);
				displayEmojiVerificationModal = true;
				verificationEmojis = event.payload.emojis;
			}
		);

		toastUnlistener = await listen<events.ToastNotificationEventType>(
			events.MatrixSvelteListenEvent.ToastNotification,
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

<main class="container">
	{@render children()}
</main>

<Toaster
	richColors
	expand={isDesktop.current}
	position={isDesktop.current ? 'top-right' : 'top-center'}
	closeButton
/>

<Dialog.Root bind:open={displayEmojiVerificationModal}>
	<Dialog.Content class="max-w-[80%] rounded-md">
		<Dialog.Header>
			<Dialog.Title>Please verify this device</Dialog.Title>
			<Dialog.Description>
				Check if the emojis match between the devices:
				<br />
				<span class="text-3xl">{verificationEmojis}</span>
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button
				variant="default"
				type="submit"
				onclick={() => {
					emit<events.VerificationResultEventType>(
						events.MatrixSvelteEmitEvent.VerificationResult,
						{ confirmed: true }
					);
					displayEmojiVerificationModal = false;
				}}>Confirm</Button
			>
			<Button
				variant="destructive"
				type="submit"
				onclick={() => {
					emit<events.VerificationResultEventType>(
						events.MatrixSvelteEmitEvent.VerificationResult,
						{ confirmed: false }
					);
					displayEmojiVerificationModal = false;
				}}>Cancel</Button
			>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<p>
	Current status: {data.roomsCollection.state.status.status}. Message: {data.roomsCollection.state
		.status.message}
</p>
