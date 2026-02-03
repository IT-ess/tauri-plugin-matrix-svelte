<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { CheckIcon, XIcon, LoaderIcon, SquarePen } from '@lucide/svelte';

	interface Props {
		label: string;
		value: string;
		isEditing: boolean;
		isSaving: boolean;
		onEdit: () => void;
		onSave: (newValue: string) => void;
		onCancel: () => void;
		canEdit: boolean;
	}

	let { label, value, isEditing, isSaving, onEdit, onSave, onCancel, canEdit }: Props = $props();
	// svelte-ignore state_referenced_locally
	let inputValue = $state(value);

	const handleSave = () => {
		if (inputValue.trim()) {
			onSave(inputValue.trim());
		}
	};

	const handleCancel = () => {
		inputValue = value;
		onCancel();
	};

	const handleKeydown = (e: KeyboardEvent) => {
		if (e.key === 'Enter') {
			handleSave();
		} else if (e.key === 'Escape') {
			handleCancel();
		}
	};
</script>

<div class="flex flex-col gap-2">
	<label class="text-sm font-medium text-muted-foreground" for={label}>{label}</label>

	{#if isEditing}
		<div class="flex gap-2">
			<Input
				type="text"
				bind:value={inputValue}
				onkeydown={handleKeydown}
				disabled={isSaving}
				placeholder={label}
				class="flex-1"
				autofocus
				id={label}
			/>
			<Button
				size="icon"
				variant="outline"
				onclick={handleSave}
				disabled={isSaving || !inputValue.trim()}
				class="shrink-0"
			>
				{#if isSaving}
					<LoaderIcon class="size-4 animate-spin" />
				{:else}
					<CheckIcon class="size-4" />
				{/if}
			</Button>
			<Button
				size="icon"
				variant="outline"
				onclick={handleCancel}
				disabled={isSaving}
				class="shrink-0"
			>
				<XIcon class="size-4" />
			</Button>
		</div>
	{:else}
		<div
			class="flex items-center justify-between gap-3 rounded-lg bg-muted/50 p-3 transition-colors hover:bg-muted"
		>
			<p class="font-medium text-foreground">{value}</p>
			<Button
				size="icon"
				variant="ghost"
				onclick={onEdit}
				disabled={isSaving || !canEdit}
				class="size-8 shrink-0"
			>
				<SquarePen class="size-4" />
			</Button>
		</div>
	{/if}
</div>
