<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import HomeserverSelection from './homeserver-selection.svelte';
	import type { Infer, SuperValidated } from 'sveltekit-superforms/client';
	import type { LoginFormSchema } from '$lib/schemas/login';
	import type { MatrixLoginPayload } from 'tauri-plugin-matrix-svelte-api';

	let {
		dataForm,
		onSubmit,
		isLoading = $bindable(),
		awaitUntilLoggedIn,
		// eslint-disable-next-line no-useless-assignment
		skipVerification = $bindable()
	}: {
		dataForm: SuperValidated<Infer<LoginFormSchema>>;
		onSubmit: (data: MatrixLoginPayload) => void;
		isLoading: boolean;
		awaitUntilLoggedIn: () => Promise<void>;
		skipVerification: boolean;
	} = $props();

	let isSignup = $state<boolean | undefined>();

	// Step and loading states
	let currentStep: 'homepage' | 'homeserverSelection' = $state('homepage');

	const goToHomeserverSelection = (signup: boolean) => {
		currentStep = 'homeserverSelection';
		isSignup = signup;
	};

	$effect(() => {
		if (isSignup !== undefined) {
			skipVerification = isSignup;
		}
	});
</script>

<div class="px-safe-offset-4 pb-safe-offset-4 flex h-full w-full flex-col justify-start">
	{#if currentStep === 'homepage'}
		<div class="mb-6 flex h-full flex-col gap-8 px-8">
			<div class="h-full"></div>
			<Button
				variant="secondary"
				class="h-12 text-lg"
				onclick={() => goToHomeserverSelection(true)}
			>
				{m.button_signup()}</Button
			>
			<Button onclick={() => goToHomeserverSelection(false)} class="h-12 text-lg">
				{m.button_login()}</Button
			>
		</div>
	{:else if currentStep == 'homeserverSelection'}
		<HomeserverSelection
			isSignup={isSignup as boolean}
			onBack={() => {
				currentStep = 'homepage';
				isSignup = undefined;
				skipVerification = false;
			}}
			{awaitUntilLoggedIn}
			{dataForm}
			bind:isLoading
			{onSubmit}
		/>
	{/if}
</div>
