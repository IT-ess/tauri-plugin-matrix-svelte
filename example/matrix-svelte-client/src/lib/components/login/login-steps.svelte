<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { ArrowLeftIcon, ArrowRightIcon, LoaderCircle, LogInIcon } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import type { MatrixClientConfig } from 'tauri-plugin-matrix-svelte-api';
	import * as Form from '$lib/components/ui/form/index';

	import { type SuperValidated, type Infer, superForm, setMessage } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { loginFormSchema, type LoginFormSchema } from '$lib/schemas/login';
	import { onMount } from 'svelte';

	let {
		dataForm,
		onSubmit,
		hostname,
		isLoading = $bindable()
	}: {
		dataForm: SuperValidated<Infer<LoginFormSchema>>;
		onSubmit: (data: MatrixClientConfig) => void;
		hostname: string;
		isLoading: boolean;
	} = $props();

	onMount(() => {
		$formData.homeserver = 'https://matrix.org';
		$formData.clientName = hostname;
	});

	const form = superForm(dataForm, {
		SPA: true,
		validators: zodClient(loginFormSchema),
		onUpdate({ form }) {
			// Form validation
			if (form.valid) {
				onSubmit({
					homeserver_url: form.data.homeserver,
					client_name: form.data.clientName,
					...form.data
				});
				setMessage(form, 'Valid data!'); // is that still necessary ?
			}
		}
	});

	const { form: formData, enhance } = form;

	// Step and loading states
	let currentStep: 'credentials' | 'client' = $state('credentials');

	// Derive if we can proceed to next step
	const canProceedToClient = $derived(
		$formData.homeserver && $formData.username && $formData.password
	);

	// Navigate between steps
	const goToClientStep = () => {
		currentStep = 'client';
	};

	const goToCredentials = () => {
		currentStep = 'credentials';
	};
</script>

<Card>
	<CardHeader>
		<CardTitle>Welcome to Matrix</CardTitle>
		<CardDescription>
			{#if currentStep === 'credentials'}
				Enter your Matrix account credentials
			{:else}
				Set your client name to continue
			{/if}
		</CardDescription>
	</CardHeader>
	<CardContent>
		<form method="POST" use:enhance>
			{#if currentStep === 'credentials'}
				<div transition:fade class="flex flex-col gap-4">
					<Form.Field {form} name="homeserver">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Homeserver</Form.Label>
								<Input {...props} bind:value={$formData.homeserver} />
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<Form.Field {form} name="username">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Username</Form.Label>
								<Input {...props} bind:value={$formData.username} />
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<Form.Field {form} name="password">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Password</Form.Label>
								<Input type="password" {...props} bind:value={$formData.password} />
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<Button type="button" disabled={!canProceedToClient} onclick={goToClientStep}>
						Next Step
						<ArrowRightIcon class="ml-2 h-4 w-4" />
					</Button>
				</div>
			{:else}
				<div transition:fade class="flex flex-col gap-4">
					<Form.Field {form} name="clientName">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Client Name</Form.Label>
								<Input {...props} bind:value={$formData.clientName} />
							{/snippet}
						</Form.Control>
						<Form.Description>Name your session</Form.Description>
						<Form.FieldErrors />
					</Form.Field>
					<div class="flex gap-2">
						<Button
							type="button"
							variant="outline"
							class="flex-1"
							onclick={goToCredentials}
							disabled={isLoading}
						>
							<ArrowLeftIcon class="mr-2 h-4 w-4" />
							Back
						</Button>

						<Form.Button type="submit" class="flex-1" disabled={isLoading}>
							{#if isLoading}
								<LoaderCircle class="animate-spin" />
								Logging in...
							{:else}
								<LogInIcon class="mr-2 h-4 w-4" />
								Login
							{/if}
						</Form.Button>
					</div>
				</div>
			{/if}
		</form>
	</CardContent>
</Card>
