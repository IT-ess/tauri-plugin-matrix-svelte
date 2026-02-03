<script lang="ts" module>
	import { z } from 'zod/v4';

	const formSchema = z.object({
		passphrase: z.string()
	});
</script>

<script lang="ts">
	import { defaults, setError, superForm, type SuperValidated } from 'sveltekit-superforms';
	import { zod4 } from 'sveltekit-superforms/adapters';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';

	import { goto } from '$app/navigation';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { buttonVariants } from '$lib/components/ui/button/index.js';
	import type { PageProps } from './$types';
	import { m } from '$lib/paraglide/messages';
	import * as Card from '$lib/components/ui/card/index.js';
	import { toast } from 'svelte-sonner';
	import { ChevronLeft, DatabaseBackup, LoaderCircle, MonitorSmartphone } from '@lucide/svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { onDestroy } from 'svelte';
	import { loginStore } from '../../hooks.client';
	import {
		restoreBackupWithPassphrase,
		MatrixSvelteListenEvent,
		resetCrossSigning
	} from 'tauri-plugin-matrix-svelte-api';
	import { resolve } from '$app/paths';

	let { data }: PageProps = $props();

	type VerificationMethod = 'anotherDevice' | 'backup';

	let selectedVerificationMethod = $state<VerificationMethod | undefined>();

	const form = superForm(defaults(zod4(formSchema)), {
		validators: zod4(formSchema),
		SPA: true,
		onUpdate: ({ form: f }) => {
			if (f.valid) {
				console.log('form is valid');
				onSubmit(f);
			}
		}
	});

	const onSubmit = async (
		form: SuperValidated<
			{
				passphrase: string;
			},
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			any,
			{
				passphrase: string;
			}
		>
	) => {
		isBackupRestoreLoading = true;
		try {
			await restoreBackupWithPassphrase(form.data.passphrase.trim());
			toast.success(m.verification_success_toast());
		} catch (e) {
			isBackupRestoreLoading = false;
			backupRestoreErrors = e;
			console.error(e);
			setError(form, e as string);
		}
	};

	let isBackupRestoreLoading = $state(false);
	let backupRestoreErrors = $state();

	let resetCrossSigningUnlistener: UnlistenFn;

	const handleResetCrossSigning = async () => {
		// Setup event listener
		resetCrossSigningUnlistener = await listen<string>(
			MatrixSvelteListenEvent.ResetCrossSigningUrl,
			async (event) => {
				let url = event.payload;
				await openUrl(url);
			}
		);
		await resetCrossSigning(null);
	};

	const { form: formData, enhance } = form;

	$effect(() => {
		if (loginStore.state.verificationState === 'verified') {
			goto(resolve('/'));
		}
	});

	onDestroy(() => {
		if (resetCrossSigningUnlistener) {
			resetCrossSigningUnlistener();
		}
	});
</script>

<main class="py-safe-offset-12 px-safe-offset-4 flex h-full w-full flex-col justify-between">
	{#if selectedVerificationMethod}
		<button
			onclick={() => (selectedVerificationMethod = undefined)}
			class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
			aria-label="Go back"
		>
			<ChevronLeft class="text-foreground h-6 w-6" />
		</button>
	{/if}
	<div class="mt-8 text-center">
		<h1 class="text-primary text-4xl font-extrabold">{m.verification_page_title()}</h1>
		<h2 class="text-muted-foreground mt-4">
			<span class="font-semibold">{m.your_refs_are_protected()}</span>
			{m.verification_page_subtitle()}
		</h2>
	</div>

	{#if selectedVerificationMethod === undefined}
		<div class="flex space-x-4">
			{@render clickableCard('anotherDevice')}
			{#if data.hasBackupSetup}
				{@render clickableCard('backup')}
			{/if}
		</div>

		<div class="mx-auto mb-8">
			<AlertDialog.Root>
				<AlertDialog.Trigger
					class={buttonVariants({
						variant: 'ghost',
						size: 'lg',
						class: 'text-md text-destructive'
					})}
				>
					{m.verification_reset_cross_signing_button()}
				</AlertDialog.Trigger>
				<AlertDialog.Content>
					<AlertDialog.Header>
						<AlertDialog.Title>{m.verification_reset_cross_signing_title()}</AlertDialog.Title>
						<AlertDialog.Description>
							{m.verification_reset_cross_signing_description()}
							<span class="font-extrabold"
								>{m.verification_reset_cross_signing_description_warning()}</span
							>
						</AlertDialog.Description>
					</AlertDialog.Header>
					<AlertDialog.Footer>
						<AlertDialog.Cancel>{m.button_cancel()}</AlertDialog.Cancel>
						<AlertDialog.Action
							class={buttonVariants({ variant: 'destructive' })}
							onclick={handleResetCrossSigning}>{m.button_proceed()}</AlertDialog.Action
						>
					</AlertDialog.Footer>
				</AlertDialog.Content>
			</AlertDialog.Root>
		</div>
	{:else if selectedVerificationMethod === 'anotherDevice'}
		<div class="mx-auto max-w-md rounded-xl border border-gray-100 bg-white p-6 shadow-sm">
			<h3 class="mb-4 text-lg font-semibold">{m.verification_other_device_header()}</h3>

			<ol class="marker:text-secondary list-inside list-decimal space-y-3 marker:font-bold">
				<li>{m.verification_other_device_step1()}</li>
				<li>
					{m.verification_other_device_step2()}<span class="font-medium"
						>{m.profile_menu_devices()}</span
					>
				</li>
				<li>
					{m.verification_other_device_step3()}<span class="font-medium"
						>{m.devices_verify_device()}</span
					>
				</li>
			</ol>

			<p class="mt-8 border-l-4 border-gray-300 py-1 pl-3 text-sm text-gray-500 italic">
				<span class="font-semibold text-gray-700 not-italic">Note:</span>
				{m.verification_other_device_other_client_instructions()}
			</p>
		</div>
		<div class="mb-safe-offset-12">
			<!-- empty div for justify between -->
		</div>
	{:else}
		<div class="mx-auto max-w-md rounded-xl border border-gray-100 bg-white p-6 shadow-sm">
			<form method="POST" class="flex flex-col items-center space-y-6" use:enhance>
				<Form.Field {form} name="passphrase">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label class="mb-4 text-lg font-semibold"
								>{m.verification_backup_form_title()}</Form.Label
							>
							<Input placeholder="EsTa TgHL NHqf..." {...props} bind:value={$formData.passphrase} />
						{/snippet}
					</Form.Control>
					<Form.Description>{m.verification_backup_helper_text()}</Form.Description>
					<Form.FieldErrors />
					<p class="text-destructive font-bold">{backupRestoreErrors}</p>
				</Form.Field>
				<Form.Button disabled={isBackupRestoreLoading}
					>{m.verification_backup_button()}
					{#if isBackupRestoreLoading}
						<LoaderCircle class="animate-spin" />
					{/if}
				</Form.Button>
			</form>
		</div>
		<div class="mb-safe-offset-12">
			<!-- empty div for justify between -->
		</div>
	{/if}
</main>

{#snippet clickableCard(variant: VerificationMethod)}
	<Card.Root class="w-full max-w-sm" onclick={() => (selectedVerificationMethod = variant)}>
		<Card.Header class="text-center">
			<Card.Title>{variant === 'anotherDevice' ? m.device() : m.backup()}</Card.Title>
			<Card.Description
				>{variant === 'anotherDevice'
					? m.verification_other_device_method_description()
					: m.verification_backup_method_description()}</Card.Description
			>
		</Card.Header>
		<Card.Content>
			{#if variant === 'anotherDevice'}
				<MonitorSmartphone class="mx-auto size-12" />
			{:else}
				<DatabaseBackup class="mx-auto size-12" />
			{/if}
		</Card.Content>
	</Card.Root>
{/snippet}
