<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { CheckIcon, XIcon } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils.svelte';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		message: string;
		onEdit: (newMessage: string) => void;
		isEditing: boolean;
	}

	let { message, onEdit, isEditing = $bindable(true) }: Props = $props();

	// svelte-ignore state_referenced_locally
	let editedMessage = $state(message);
	let inputRef: HTMLTextAreaElement | null = $state(null);

	onMount(() => {
		// Focus input after DOM update
		setTimeout(() => {
			if (inputRef === null) return;
			inputRef.focus();
			inputRef.select();
		}, 50);
	});

	// Handle save
	function saveEdit() {
		if (editedMessage.trim() === '') {
			toast.error('Message cannot be empty');
			return;
		}

		if (editedMessage.trim() === message) {
			cancelEdit();
			return;
		}

		onEdit(editedMessage.trim());
		isEditing = false;
	}

	// Handle cancel
	function cancelEdit() {
		isEditing = false;
		editedMessage = message;
	}

	// Handle keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			saveEdit();
		} else if (event.key === 'Escape') {
			cancelEdit();
		}
	}
</script>

<div class="flex flex-col gap-2">
	<textarea
		bind:this={inputRef}
		bind:value={editedMessage}
		onkeydown={handleKeydown}
		placeholder={m.message_input_placeholder()}
		rows="1"
		class="field-sizing-content max-h-24 flex-1 resize-none overflow-y-auto rounded-md border p-2"
	></textarea>
	<div class="flex justify-end gap-1">
		<Button size="sm" variant="ghost" onclick={saveEdit} class={cn('size-6 p-0')}>
			<CheckIcon class="size-3" />
		</Button>
		<Button size="sm" variant="ghost" onclick={cancelEdit} class={cn('size-6 p-0')}>
			<XIcon class="size-3" />
		</Button>
	</div>
</div>
