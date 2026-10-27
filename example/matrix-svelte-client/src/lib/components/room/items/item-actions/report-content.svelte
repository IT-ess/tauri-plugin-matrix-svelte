<script lang="ts">
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as RadioGroup from '#lib/components/ui/radio-group/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Label } from '#lib/components/ui/label/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { Spinner } from '#lib/components/ui/spinner/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { toast } from 'svelte-sonner';
	import { reportEvent } from 'tauri-plugin-matrix-svelte-api';

	type Props = {
		open: boolean;
		roomId: string;
		eventId: string;
	};

	let { open = $bindable(false), roomId, eventId }: Props = $props();

	// `reason` is the English text sent to the homeserver admins, whatever the UI locale
	const categories = [
		{ value: 'spam', reason: 'Spam', label: m.report_content_category_spam },
		{
			value: 'harassment',
			reason: 'Harassment or abuse',
			label: m.report_content_category_harassment
		},
		{
			value: 'csam',
			reason: 'Child sexual abuse material (CSAM)',
			label: m.report_content_category_csam
		},
		{ value: 'illegal', reason: 'Other illegal content', label: m.report_content_category_illegal },
		{ value: 'violence', reason: 'Violence or threats', label: m.report_content_category_violence },
		{ value: 'other', reason: 'Other', label: m.report_content_category_other }
	];

	const id = $props.id();

	let category = $state('');
	let details = $state('');
	let isSubmitting = $state(false);

	const resetForm = () => {
		category = '';
		details = '';
	};

	const handleSubmit = async (event: SubmitEvent) => {
		event.preventDefault();
		const selected = categories.find((c) => c.value === category);
		if (!selected || isSubmitting) return;

		const trimmedDetails = details.trim();
		const reason = trimmedDetails ? `${selected.reason}: ${trimmedDetails}` : selected.reason;

		isSubmitting = true;
		try {
			await reportEvent(roomId, eventId, reason);
			toast.success(m.report_content_success());
			open = false;
		} catch (err) {
			console.error(err);
			toast.error(`${m.report_content_error()} ${err}`);
		} finally {
			isSubmitting = false;
		}
	};
</script>

<Dialog.Root
	bind:open={() => open, (value) => (open = isSubmitting ? true : value)}
	onOpenChangeComplete={(isOpen) => !isOpen && resetForm()}
>
	<Dialog.Content showCloseButton={!isSubmitting}>
		<form onsubmit={handleSubmit} class="flex flex-col gap-4">
			<Dialog.Header>
				<Dialog.Title>{m.report_content_title()}</Dialog.Title>
				<Dialog.Description>{m.report_content_description()}</Dialog.Description>
			</Dialog.Header>

			<div class="flex flex-col gap-3">
				<Label id="{id}-category">{m.report_content_category_label()}</Label>
				<RadioGroup.Root
					bind:value={category}
					aria-labelledby="{id}-category"
					disabled={isSubmitting}
					required
				>
					{#each categories as c (c.value)}
						<div class="flex items-center gap-3">
							<RadioGroup.Item value={c.value} id="{id}-{c.value}" />
							<Label for="{id}-{c.value}" class="font-normal">{c.label()}</Label>
						</div>
					{/each}
				</RadioGroup.Root>
			</div>

			<div class="flex flex-col gap-3">
				<Label for="{id}-details">{m.report_content_details_label()}</Label>
				<Textarea
					id="{id}-details"
					bind:value={details}
					placeholder={m.report_content_details_placeholder()}
					maxlength={1000}
					disabled={isSubmitting}
				/>
			</div>

			<Dialog.Footer>
				<Dialog.Close disabled={isSubmitting} class={buttonVariants({ variant: 'outline' })}
					>{m.button_cancel()}</Dialog.Close
				>
				<Button type="submit" variant="destructive" disabled={!category || isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{m.report_content_submit()}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
