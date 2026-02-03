<script lang="ts">
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { CheckIcon, ChevronLeft, CopyIcon, LoaderCircle } from '@lucide/svelte';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Button } from '$lib/components/ui/button';
	import { m } from '$lib/paraglide/messages';
	import { toast } from 'svelte-sonner';
	import { setupNewBackup } from 'tauri-plugin-matrix-svelte-api';
	import { gotoRoomsList } from '$lib/utils.svelte';

	let hasCopied = $state(false);
	let canProceed = $state(false);
	const copyBackupToClipboard = async (secretKey: string) => {
		await writeText(secretKey);
		hasCopied = true;
		setTimeout(() => (canProceed = true), 4000);
	};

	const handleClickNext = () => {
		if (canProceed) {
			return gotoRoomsList('dm');
		}
		if (hasCopied) {
			toast(m.recovery_save_key_first());
		} else {
			toast(m.recovery_copy_key_first());
		}
	};

	let userHasEnabled = $state(false);
	let secretKey = $state<undefined | string>();
	let errors = $state();
	const fetchRecovery = async (e: Event) => {
		e.preventDefault();
		userHasEnabled = true;
		try {
			let key = await setupNewBackup();
			secretKey = key;
		} catch (err) {
			console.error(err);
			errors = err;
		}
	};
</script>

<main class="py-safe-offset-12 px-safe-offset-4 flex h-full w-full flex-col justify-between">
	<button
		onclick={() => gotoRoomsList('dm')}
		class="top-safe-offset-2 hover:bg-accent absolute left-3 flex h-10 w-10 items-center justify-center rounded-full transition-colors"
		aria-label="Go back"
	>
		<ChevronLeft class="text-foreground h-6 w-6" />
	</button>
	<div class="flex flex-col space-y-6">
		<div class="mt-8 text-center">
			<h1 class="text-primary text-4xl font-extrabold">{m.recovery_page_title()}</h1>
			<h2 class="text-muted-foreground mt-4">
				<span class="font-semibold">{m.your_refs_are_protected()}</span>
				{m.recovery_page_subtitle()}
			</h2>
		</div>
		<div class="flex items-center justify-start space-x-4">
			<Switch bind:checked={userHasEnabled} onclick={fetchRecovery} disabled={userHasEnabled} />
			<h3 class="text-lg font-semibold">{m.recovery_page_title()}</h3>
		</div>
		{#if userHasEnabled}
			<div class="flex flex-col">
				{#if errors}
					<p class="text-destructive font-semibold">{errors}</p>
				{/if}
				{#if !secretKey}
					<LoaderCircle class="mx-auto size-8 animate-spin" />
				{:else}
					<Label>{m.secret_key()}</Label>
					<InputGroup.Root>
						<InputGroup.Input placeholder={secretKey} readonly />
						<InputGroup.Addon align="inline-end">
							<InputGroup.Button
								aria-label="Copy"
								title="Copy"
								size="icon-xs"
								onclick={() => copyBackupToClipboard(secretKey ?? '')}
							>
								{#if hasCopied}
									<CheckIcon />
								{:else}
									<CopyIcon />
								{/if}
							</InputGroup.Button>
						</InputGroup.Addon>
					</InputGroup.Root>
					<p class="mt-4 text-lg font-semibold">
						{m.recovery_save_instructions()}
					</p>
				{/if}
			</div>
		{/if}
	</div>
	<div class="flex items-center justify-center-safe">
		<Button
			class={canProceed ? '' : 'opacity-50'}
			disabled={!userHasEnabled}
			onclick={handleClickNext}>{m.button_next()}</Button
		>
	</div>
</main>
