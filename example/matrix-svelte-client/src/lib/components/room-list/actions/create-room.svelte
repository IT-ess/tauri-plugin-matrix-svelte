<script lang="ts">
	import { MediaQuery } from 'svelte/reactivity';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Form from '$lib/components/ui/form/index';

	import { buttonVariants } from '$lib/components/ui/button';

	import { type SuperValidated, type Infer, superForm, setMessage } from 'sveltekit-superforms';
	import { zod4Client } from 'sveltekit-superforms/adapters';
	import { createDMRoomFormSchema, type CreateDMRoomFormSchema } from '$lib/schemas/matrix-id';
	import { onMount } from 'svelte';
	import { createMatrixRequest, submitAsyncRequest } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		actionCreateRoomOpen: boolean;
		isActionButtonOpen: boolean;
		// TODO: remove this param since we don't really need +page.ts
		// https://superforms.rocks/concepts/spa#without-a-pagets-file
		dataForm: SuperValidated<Infer<CreateDMRoomFormSchema>>;
	};

	let {
		actionCreateRoomOpen = $bindable(false),
		isActionButtonOpen = $bindable(),
		dataForm
	}: Props = $props();

	const isDesktop = new MediaQuery('(min-width: 768px)');

	const form = superForm(dataForm, {
		SPA: true,
		validators: zod4Client(createDMRoomFormSchema),
		onUpdate({ form }) {
			// Form validation
			if (form.valid) {
				// Send request (handleCreateRoom)
				handleCreateRoom();
				setMessage(form, 'Valid data!'); // is that still necessary ?
				actionCreateRoomOpen = false;
			}
		}
	});

	const { form: formData, enhance } = form;

	onMount(() => {
		$formData.username = '@johndoe:matrix.org';
	});

	/** Handle create room action */
	const handleCreateRoom = async () => {
		const request = createMatrixRequest.createDMRoom({
			userId: $formData.username
		});
		await submitAsyncRequest(request);
	};
</script>

{#if isDesktop.current}
	<Dialog.Root bind:open={actionCreateRoomOpen}>
		<Dialog.Content class="sm:max-w-[425px]">
			<Dialog.Header>
				<Dialog.Title>Create DM Room</Dialog.Title>
				<Dialog.Description>Invite a friend to discuss.</Dialog.Description>
			</Dialog.Header>
			{@render createRoomForm()}
		</Dialog.Content>
		<Dialog.Close class={buttonVariants({ variant: 'outline' })}>Cancel</Dialog.Close>
	</Dialog.Root>
{:else}
	<Drawer.Root bind:open={actionCreateRoomOpen}>
		<Drawer.Content>
			<Drawer.Header class="text-left">
				<Drawer.Title>Create DM Room</Drawer.Title>
				<Drawer.Description>Invite a friend to discuss.</Drawer.Description>
			</Drawer.Header>
			<div class="mx-4">
				{@render createRoomForm()}
			</div>
			<Drawer.Footer class="pt-2">
				<Drawer.Close class={buttonVariants({ variant: 'outline' })}>Cancel</Drawer.Close>
			</Drawer.Footer>
		</Drawer.Content>
	</Drawer.Root>
{/if}

{#snippet createRoomForm()}
	<form method="POST" use:enhance class="grid items-start gap-4">
		<Form.Field {form} name="username">
			<Form.Control>
				{#snippet children({ props })}
					<Form.Label>Matrix ID</Form.Label>
					<Input {...props} bind:value={$formData.username} />
				{/snippet}
			</Form.Control>
			<Form.FieldErrors />
		</Form.Field>
		<Form.Button type="submit" class="flex-1">Send invite</Form.Button>
	</form>
{/snippet}
