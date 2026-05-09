<script lang="ts">
	import * as RadioGroup from '$lib/components/ui/radio-group/index.js';
	import { Label } from '$lib/components/ui/label/index.js';

	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { LoaderCircle, LogInIcon, Server, XIcon } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import * as Form from '$lib/components/ui/form/index';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import { Spinner } from '../ui/spinner';
	import { listen } from '@tauri-apps/api/event';
	import { hostname, platform } from '@tauri-apps/plugin-os';
	import { authenticate } from 'tauri-plugin-web-auth-api';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import {
		setMessage,
		superForm,
		type Infer,
		type SuperValidated,
		type ValidationErrors
	} from 'sveltekit-superforms/client';
	import { zod4Client } from 'sveltekit-superforms/adapters';
	import { loginFormSchema, type LoginFormSchema } from '$lib/schemas/login';
	import { beforeNavigate } from '$app/navigation';
	import {
		buildClientFromHomeserverUrl,
		checkHomeserverAuthType,
		forwardOAuthLoginDeeplink,
		MatrixSvelteListenEvent,
		type AuthTypeResponse,
		type MatrixLoginPayload
	} from 'tauri-plugin-matrix-svelte-api';

	let {
		isSignup,
		onBack,
		awaitUntilLoggedIn,
		dataForm,
		isLoading = $bindable(),
		onSubmit
	}: {
		isSignup: boolean;
		onBack: () => void;
		awaitUntilLoggedIn: () => Promise<void>;
		dataForm: SuperValidated<Infer<LoginFormSchema>>;
		isLoading: boolean;
		onSubmit: (data: MatrixLoginPayload) => void;
	} = $props();

	let activeStep = $state<'homeserverSelection' | 'credentials'>('homeserverSelection');
	let authType = $state<AuthTypeResponse>();

	let homeserverSearch = $state('');

	let selectedValue = $state('matrix.org');
	let selectedHomeserver = $derived(homeserverSearch != '' ? homeserverSearch : selectedValue);
	let isValidUrl = $derived(selectedHomeserver.includes('.'));

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let customHomeserverError: any = $state()!;
	let isCheckingHomeserver = $state(false);
	let openOauthAlert = $state(false);
	let oauthUrl = $state<string>();

	const defineAndCheckHomeserver = async () => {
		isCheckingHomeserver = true;
		try {
			await buildClientFromHomeserverUrl(selectedHomeserver);

			await goToLoginStep(isSignup);

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} catch (e: any) {
			console.error(e);
			customHomeserverError = e;
		} finally {
			isCheckingHomeserver = false;
		}
	};

	const goToLoginStep = async (isSignup: boolean) => {
		const unlisten = await listen<string>(MatrixSvelteListenEvent.OAuthUrl, async (event) => {
			const platformName = platform();
			let url = event.payload;
			if (isSignup) {
				// Appending this query param to the auth url will force the creation form
				// if the user isn't already connected
				url = url + '&prompt=create';
			}
			if (platformName === 'ios') {
				// use the tauri-plugin-web-auth for native handling on ios
				const res = await authenticate({
					url,
					callbackScheme: 'https'
				});
				// Then forward the callback url to backend
				await forwardOAuthLoginDeeplink(res.callbackUrl);
			} else {
				console.log(url);
				oauthUrl = url;
				openOauthAlert = true;
				// Callback url will then be handled by the deeplink
				// and forwarded to backend through a tauri command.
			}
		});
		authType = await checkHomeserverAuthType();
		activeStep = 'credentials';
		if (authType === 'oauth') {
			await awaitUntilLoggedIn();
			console.log('finished awaiting');
			unlisten();
		} else {
			// We replace by a full URL, otherwise the matrix API isn't happy
			$formData.homeserver = 'https://' + selectedHomeserver;
			const rawHost = await hostname();
			$formData.clientName = rawHost === 'localhost' || rawHost === null ? 'MatrixSvelte' : rawHost;
		}
	};

	let formErrors = $state<
		| ValidationErrors<{
				username: string;
				password: string;
				homeserver: string;
				clientName: string;
		  }>
		| undefined
	>();
	// svelte-ignore state_referenced_locally
	const form = superForm(dataForm, {
		SPA: true,
		validators: zod4Client(loginFormSchema),
		onUpdate({ form }) {
			if (form.errors) {
				formErrors = form.errors;
			}
			// Form validation
			if (form.valid) {
				onSubmit({
					homeserverUrl: form.data.homeserver,
					username: '@' + form.data.username + ':' + selectedHomeserver,
					clientName: form.data.clientName,
					password: form.data.password
				});
				setMessage(form, 'Valid data!'); // is that still necessary ?
			}
		}
	});
	const { form: formData, enhance } = form;

	beforeNavigate(({ cancel, type }) => {
		if (type != 'goto') {
			cancel(); // Stops the navigation when the back button has been pressed
			if (activeStep == 'homeserverSelection') {
				onBack();
			}
		}
	});
</script>

<Card.Root class="mx-auto mb-4 min-h-124 w-full max-w-md">
	{#if activeStep == 'homeserverSelection'}
		<Card.Header>
			<Card.Title>{isSignup ? m.login_signup_title() : m.login_title()}</Card.Title>
			<Card.Description>{m.login_homeserver_desc_matrix()}</Card.Description>
			<Card.Action>
				<Button size="xs" variant="link" onclick={onBack}>{m.button_go_back()}</Button>
			</Card.Action>
		</Card.Header>
		<Card.Content class="flex h-full flex-col gap-4">
			<div class="relative">
				<Input
					autocorrect="off"
					autocapitalize="off"
					bind:value={homeserverSearch}
					placeholder={m.login_homeserver_search_placeholder()}
					class="py-3 pr-4 pl-10 text-base"
					autofocus={false}
				/>
				<Server class="text-muted-foreground absolute top-2 left-3 size-5" />
				{#if homeserverSearch.trim()}
					<XIcon
						onclick={(e) => {
							e.stopImmediatePropagation();
							homeserverSearch = '';
						}}
						class="text-muted-foreground absolute top-2.5 right-2 size-4"
					/>
				{/if}
			</div>
			<RadioGroup.Root bind:value={selectedValue} class="bg-muted gap-6 rounded-2xl p-4">
				{#if !homeserverSearch}
					{@render homeserverItem('matrix.org', m.login_homeserver_matrix_desc())}
					{@render homeserverItem('mozilla.org', m.login_homeserver_mozilla_desc())}
				{:else}
					{@render homeserverItem(homeserverSearch, m.login_homeserver_custom_desc())}
				{/if}
			</RadioGroup.Root>

			{#if customHomeserverError}
				<p class="text-destructive">
					{m.login_homeserver_error({ homeserver: selectedHomeserver })}
					{customHomeserverError}
				</p>
			{/if}
		</Card.Content>
		<Card.Footer class="flex-col gap-2">
			<Button
				class="w-full"
				disabled={isCheckingHomeserver ||
					!selectedValue ||
					!isValidUrl ||
					selectedValue !== selectedHomeserver}
				onclick={defineAndCheckHomeserver}
				>{m.button_next()}
				{#if isCheckingHomeserver}
					<Spinner />
				{/if}
			</Button>
		</Card.Footer>
	{:else if activeStep == 'credentials'}
		{#if authType === 'matrix'}
			<Card.Header>
				<Card.Title>{m.login_with_homeserver({ homeserver: selectedHomeserver })}</Card.Title>
				<Card.Description>{m.login_subtitle()}</Card.Description>
			</Card.Header>
			<Card.Content class="h-full">
				<form class="h-full justify-between" method="POST" use:enhance>
					<Form.Field {form} name="username">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{m.login_username()}</Form.Label>
								<InputGroup.Root>
									<InputGroup.Addon align="inline-start">@</InputGroup.Addon>
									<InputGroup.Input
										{...props}
										bind:value={$formData.username}
										placeholder={m.login_username_placeholder()}
									/>
									<InputGroup.Addon align="inline-end">
										<InputGroup.Text>:{selectedHomeserver}</InputGroup.Text>
									</InputGroup.Addon>
								</InputGroup.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<Form.Field {form} name="password">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{m.login_password()}</Form.Label>
								<Input type="password" {...props} bind:value={$formData.password} />
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<div class="mt-4 flex w-full justify-between">
						<Button
							disabled={isLoading}
							type="reset"
							onclick={() => {
								activeStep = 'homeserverSelection';
							}}>{m.button_back()}</Button
						>
						<Form.Button type="submit" disabled={isLoading}>
							{#if isLoading}
								<LoaderCircle class="animate-spin" />
								{m.button_loading()}
							{:else}
								<LogInIcon class="mr-1 h-4 w-4" />
								{m.button_login()}
							{/if}
						</Form.Button>
					</div>
					{#if formErrors}
						<p class="text-destructive">{JSON.stringify(formErrors)}</p>
					{/if}
				</form>
			</Card.Content>
		{:else if authType === 'oauth'}
			<Card.Header>
				<Card.Title
					>{isSignup
						? m.login_signup_with_homeserver({ homeserver: selectedHomeserver })
						: m.login_with_homeserver({ homeserver: selectedHomeserver })}</Card.Title
				>
			</Card.Header>
			<Card.Content class="flex flex-col items-center gap-4">
				<LoaderCircle class="text-primary size-24 animate-spin" />
				<p class="text-center text-xl font-medium">{m.login_loader_help_text()}</p>
			</Card.Content>
		{/if}
	{/if}
</Card.Root>

<AlertDialog.Root bind:open={openOauthAlert}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title
				>{isSignup
					? m.login_signup_with_homeserver({ homeserver: selectedHomeserver })
					: m.login_with_homeserver({ homeserver: selectedHomeserver })}</AlertDialog.Title
			>
			<AlertDialog.Description>
				{m.login_oauth_alert_desc()}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<!-- TODO: this cancel button is disabled right now, because the backend doesn't properly handle reconstructing the client right now. It should be improved in the future. -->
			<!-- <AlertDialog.Cancel
				disabled
				onclick={() => {
					openOauthAlert = false;
					authType = undefined;
					activeStep = 'homeserverSelection';
					isLoading = false; // Manually setting is loading to false should kill the "awaitUntilLogin"
				}}>{m.button_cancel()}</AlertDialog.Cancel
			> -->
			<AlertDialog.Action
				onclick={() => {
					openUrl(oauthUrl as string);
					openOauthAlert = false;
				}}>{m.button_proceed()}</AlertDialog.Action
			>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

{#snippet homeserverItem(address: string, description: string)}
	<div class="flex w-full items-center gap-2">
		<RadioGroup.Item value={address} id={address} />
		<Label class="flex w-full flex-col items-start" for={address}
			>{address} <span class="text-muted-foreground">{description}</span></Label
		>
	</div>
{/snippet}
