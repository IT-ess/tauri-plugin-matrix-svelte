<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { events } from 'tauri-plugin-matrix-svelte-api';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

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

	let emojisUnlistener: UnlistenFn;

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
	});

	onDestroy(() => {
		emojisUnlistener();
	});
</script>

{@render children()}

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
