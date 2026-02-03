<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import { InfoIcon, LoaderCircle, LogInIcon, ServerCog, ServerIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import * as Form from '$lib/components/ui/form/index';

	import { type SuperValidated, type Infer, superForm, setMessage } from 'sveltekit-superforms';
	import { zod4Client } from 'sveltekit-superforms/adapters';
	import { loginFormSchema, type LoginFormSchema } from '$lib/schemas/login';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { listen } from '@tauri-apps/api/event';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { platform } from '@tauri-apps/plugin-os';
	import {
		buildClientFromHomeserverUrl,
		checkHomeserverAuthType,
		forwardOAuthLoginDeeplink,
		MatrixSvelteListenEvent,
		type MatrixLoginPayload
	} from 'tauri-plugin-matrix-svelte-api';
	import { authenticate } from 'tauri-plugin-web-auth-api';

	let {
		dataForm,
		onSubmit,
		hostname,
		isLoading = $bindable(),
		awaitUntilLoggedIn,
		skipVerification = $bindable()
	}: {
		dataForm: SuperValidated<Infer<LoginFormSchema>>;
		onSubmit: (data: MatrixLoginPayload) => void;
		hostname: string;
		isLoading: boolean;
		awaitUntilLoggedIn: () => Promise<void>;
		skipVerification: boolean;
	} = $props();

	onMount(() => {
		$formData.homeserver = 'refs.rs';
		$formData.clientName = hostname;
	});

	// svelte-ignore state_referenced_locally
	const form = superForm(dataForm, {
		SPA: true,
		validators: zod4Client(loginFormSchema),
		onUpdate({ form }) {
			// Form validation
			if (form.valid) {
				onSubmit({
					homeserverUrl: form.data.homeserver,
					...form.data
				});
				setMessage(form, 'Valid data!'); // is that still necessary ?
			}
		}
	});

	const { form: formData, enhance } = form;

	// Step and loading states
	let currentStep: 'homeserverSelection' | 'login' = $state('homeserverSelection');

	let authType = $state<'matrix' | 'oauth' | 'wrongUrl'>()!;
	// Navigate between steps
	// TODO: add command to rebuild destroy the client and rebuild a new one
	const goToLoginStep = async (isSignup: boolean) => {
		authType = await checkHomeserverAuthType();
		currentStep = 'login';
		if (authType === 'oauth') {
			const unlisten = await listen<string>(MatrixSvelteListenEvent.OAuthUrl, async (event) => {
				const platformName = platform();
				let url = event.payload;
				if (isSignup) {
					// Appending this query param to the auth url will force the creation form
					// if the user isn't already connected
					url = url + '&prompt=create';
					skipVerification = true;
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
					// open connection URL in a new window
					openUrl(url);
					// Callback url will then be handled by the deeplink
					// and forwarded to backend through a tauri command.
				}
			});
			await awaitUntilLoggedIn();
			unlisten();
		}
		// matrix login has its own flow
	};

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let customHomeserverError: any = $state()!;
	let hasSubmittedLogin = $state(false);
	let hasSubmittedSignup = $state(false);
	let disableButtons = $derived(hasSubmittedLogin || hasSubmittedSignup);
	const defineAndCheckHomeserver = async (signup: boolean) => {
		if (signup) {
			hasSubmittedSignup = true;
		} else {
			hasSubmittedLogin = true;
		}
		try {
			await buildClientFromHomeserverUrl($formData.homeserver);

			await goToLoginStep(signup);

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} catch (e: any) {
			customHomeserverError = e;
			hasSubmittedLogin = false;
			hasSubmittedSignup = false;
		}
	};
</script>

<Dialog.Root>
	<Dialog.Trigger
		disabled={disableButtons}
		class={buttonVariants({
			variant: 'ghost',
			size: 'icon-lg',
			class: 'top-safe-offset-2 right-safe-offset-2 text-primary absolute'
		})}><ServerCog class="size-7" /></Dialog.Trigger
	>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{m.login_custom_homeserver()}</Dialog.Title>
			<Dialog.Description
				>{m.login_refs_works_with()}
				<a class="text-blue-500 underline" href="https://matrix.org">Matrix</a>. {m.login_keep_the_default()}</Dialog.Description
			>
		</Dialog.Header>
		<Tooltip.Provider>
			<InputGroup.Root>
				<InputGroup.Input bind:value={$formData.homeserver} />
				<InputGroup.Addon>
					<ServerIcon />
				</InputGroup.Addon>
				<InputGroup.Addon align="inline-end">
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<InputGroup.Button {...props} class="rounded-full" size="icon-xs">
									<InfoIcon />
								</InputGroup.Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content
							>{m.login_homeserver_tooltip()} (@alice:<mark>matrix.org</mark>)</Tooltip.Content
						>
					</Tooltip.Root>
				</InputGroup.Addon>
			</InputGroup.Root>
		</Tooltip.Provider>
		{#if customHomeserverError}
			<p class="text-destructive">
				The selected homeserver isn't valid or reachable. Error: {customHomeserverError}
			</p>
		{/if}
		<Dialog.Footer>
			<Button
				disabled={disableButtons}
				type="submit"
				onclick={() => defineAndCheckHomeserver(false)}
			>
				{#if hasSubmittedLogin}
					<LoaderCircle class="animate-spin" />
				{/if}
				{m.button_login()}</Button
			>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<div class="px-safe-offset-4 pb-safe-offset-4 flex h-full w-full flex-col justify-between">
	{#if currentStep === 'homeserverSelection'}
		<div transition:fade>
			<h2 class="text-primary mt-8 text-center font-serif text-5xl">{m.login_refs_motto()}</h2>
		</div>
		<div class="flex flex-col space-y-5 px-8 pb-6">
			<Button
				size="lg"
				disabled={disableButtons}
				onclick={() => defineAndCheckHomeserver(false)}
				class="text-lg"
				>{#if hasSubmittedLogin}
					<LoaderCircle class="animate-spin" />
				{/if}{m.button_login()}</Button
			>
			<Button
				size="lg"
				variant="link"
				class="text-lg"
				disabled={disableButtons}
				onclick={() => defineAndCheckHomeserver(true)}
				>{#if hasSubmittedSignup}
					<LoaderCircle class="animate-spin" />
				{/if}{m.button_signup()}</Button
			>
		</div>
	{:else}
		<div transition:fade>
			{#if authType === 'matrix'}
				<Card>
					<CardHeader>
						<CardTitle>{m.login_title()}</CardTitle>
						<CardDescription>{m.login_subtitle()}</CardDescription>
					</CardHeader>
					<CardContent>
						<form method="POST" use:enhance>
							<div transition:fade class="flex flex-col gap-4">
								<Form.Field {form} name="username">
									<Form.Control>
										{#snippet children({ props })}
											<Form.Label>{m.login_username()}</Form.Label>
											<Input {...props} bind:value={$formData.username} />
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
							</div>

							<Form.Button type="submit" class="flex-1" disabled={isLoading}>
								{#if isLoading}
									<LoaderCircle class="animate-spin" />
									{m.button_loading()}
								{:else}
									<LogInIcon class="mr-1 h-4 w-4" />
									{m.button_login()}
								{/if}
							</Form.Button>
						</form>
					</CardContent>
				</Card>
			{:else if authType === 'oauth'}
				<!-- oauth -->
				<div class="flex items-center justify-center">
					<LoaderCircle class="text-primary size-24 animate-spin" />
				</div>
			{:else}
				<!-- wrong url -->
				<p>Wrong homeserver url</p>
			{/if}
		</div>
	{/if}
</div>
