<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { CheckIcon, XIcon } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { cn } from '$lib/utils';
	import { onMount } from 'svelte';

	interface Props {
		message: string;
		onEdit: (newMessage: string) => void;
		isEditing: boolean;
	}

	let { message, onEdit, isEditing = $bindable(true) }: Props = $props();

	let editedMessage = $state(message);
	let inputRef: HTMLInputElement | null = $state(null);

	onMount(() => {
		editedMessage = message;
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
	<Input
		bind:ref={inputRef}
		bind:value={editedMessage}
		onkeydown={handleKeydown}
		class="resize-none border-0 bg-transparent p-0 text-sm focus-visible:ring-0"
		placeholder="Type your message..."
	/>
	<div class="flex justify-end gap-1">
		<Button size="sm" variant="ghost" onclick={saveEdit} class={cn('size-6 p-0')}>
			<CheckIcon class="size-3" />
		</Button>
		<Button size="sm" variant="ghost" onclick={cancelEdit} class={cn('size-6 p-0')}>
			<XIcon class="size-3" />
		</Button>
	</div>
</div>
