<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import { ping, events } from 'tauri-plugin-matrix-svelte-api';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { emit } from '@tauri-apps/api/event';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import type { PageProps } from './$types';
	import RoomList from '$lib/components/room-list/room-list.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let { data }: PageProps = $props();

	let displayEmojiVerificationModal = $state(false);
	let verificationEmojis = $state('');

	let matrixEvent = $state('');

	let emojisUnlistener: UnlistenFn;
	let messageUnlistener: UnlistenFn;

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
		messageUnlistener = await listen<events.MessageTextEventType>(
			events.MatrixSvelteListenEvent.MessageText,
			(event) => {
				console.log('Matrix event received:', event.payload);
				matrixEvent = event.payload.body;
			}
		);
	});

	onDestroy(() => {
		emojisUnlistener();
		messageUnlistener();
	});

	let response = $state('');

	function updateResponse(returnValue: any) {
		response +=
			`[${new Date().toLocaleTimeString()}] ` +
			(typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) +
			'<br>';
	}

	function setCurrentActiveRoomAndGoToRoomRoute(id: string): undefined {
		data.roomsCollection.state.currentActiveRoom = id;
		goto(`/room`);
	}

	function _ping() {
		ping('Pong!').then(updateResponse).catch(updateResponse);
	}
</script>

<main class="container">
	<h1>Welcome to Tauri + Svelte + Matrix</h1>
	<p>rooms</p>
	<RoomList
		roomsCollection={data.roomsCollection}
		onRoomClick={setCurrentActiveRoomAndGoToRoomRoute}
	/>
	<br />

	<div>
		<Button onclick={_ping}>Ping</Button>
		<div>{@html response}</div>
	</div>

	{#if matrixEvent !== ''}
		<div>
			<p>New message:</p>
			<pre>{matrixEvent}</pre>
		</div>
	{/if}

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
</main>
