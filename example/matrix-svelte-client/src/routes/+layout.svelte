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
	import { beforeNavigate, goto } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import '@saurl/tauri-plugin-safe-area-insets-css-api';
	import { loginStore, roomsCollection } from '../hooks.client';
	import { platform } from '@tauri-apps/plugin-os';
	import { getCurrent } from '@tauri-apps/plugin-deep-link';
	import {
		handleMatrixUri,
		isLoggedIn,
		MatrixSvelteEmitEvent,
		MatrixSvelteListenEvent,
		type MatrixUriIntent,
		type ToastNotificationEventType,
		type VerificationEmojisEventType
	} from 'tauri-plugin-matrix-svelte-api';
	import { gotoProfile, gotoRoom, gotoRoomPreview, pollWithBackoff } from '$lib/utils.svelte';
	import { onNotificationClicked } from '@choochmeque/tauri-plugin-notifications-api';
	import type { PluginListener } from '@tauri-apps/api/core';

	let { children }: LayoutProps = $props();

	if (platform() !== 'linux') {
		setupViewTransition();
	}

	let displayEmojiVerificationModal = $state(false);

	let verificationEmojis = $state('');
	const isDesktop = new MediaQuery('(min-width: 768px)');

	let emojisUnlistener: UnlistenFn;
	let toastUnlistener: UnlistenFn;
	let matrixIntentUnlistener: UnlistenFn;
	let notificationClickedListener: PluginListener | undefined;

	onMount(async () => {
		// iOS: notifications posted by the Notification Service Extension carry
		// the Matrix deep link in their userInfo (`deepLink` extra) instead of
		// Android's ACTION_VIEW intent. This listener also replays a pending
		// tap when the app was cold-started from a notification.
		notificationClickedListener = await onNotificationClicked(({ data }) => {
			const deepLink = data?.deepLink;
			if (deepLink && deepLink.startsWith('matrix:')) {
				handleMatrixUri(deepLink);
			}
		});

		matrixIntentUnlistener = await listen<MatrixUriIntent>(
			MatrixSvelteListenEvent.MatrixUriIntent,
			async (event) => {
				if (event.payload.kind == 'room') {
					// eslint-disable-next-line @typescript-eslint/no-unused-vars
					const [roomId, _viaServers, eventId] = event.payload.payload;

					// If the intent points to an event, we directly open the room without the preview
					if (eventId) {
						// We need to await the room list to be populated before trying to go to the room
						await pollWithBackoff(
							() => !!roomsCollection.state.allJoinedRooms[roomId],
							() =>
								gotoRoom(
									roomId,
									roomsCollection.state.allJoinedRooms[roomId]?.avatar ?? null,
									eventId
								),
							{ initialDelay: 20, maxRetries: 100, factor: 1.1 }
						);
					} else {
						gotoRoomPreview(null, null, event.payload.payload[0]);
					}
				} else {
					gotoProfile(event.payload.payload);
				}
			}
		);

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

		// We check if the app has been launched with a specific intent (i.e. a deep link)
		const launchUris = await getCurrent();
		if (launchUris && launchUris[0] && launchUris[0].startsWith('matrix:')) {
			handleMatrixUri(launchUris[0]);
		}
	});

	$effect(() => {
		if (loginStore.state.state == 'awaitingForHomeserver') {
			// Double check because sometimes the login store is lagging.
			isLoggedIn().then((isLogged) => {
				if (!isLogged) {
					goto('/login');
				}
			});
		}
	});

	onDestroy(() => {
		if (matrixIntentUnlistener) {
			matrixIntentUnlistener();
		}
		if (emojisUnlistener) {
			emojisUnlistener();
		}
		if (toastUnlistener) {
			toastUnlistener();
		}
		notificationClickedListener?.unregister();
	});

	beforeNavigate(({ cancel, to }) => {
		// Current bug: `matrix:` URIs aren't supported by the browser so it doesn't
		// even trigger navigation correctly and this handler isn't reached.
		if (to && (to.url.protocol == 'matrix:' || to.url.hostname == 'matrix.to')) {
			cancel();
			handleMatrixUri(to.url.toString());
		}
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
