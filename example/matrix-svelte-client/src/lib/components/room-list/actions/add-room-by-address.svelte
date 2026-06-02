<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CircleCheck, CircleX, Loader } from '@lucide/svelte';
	import { Debounced } from 'runed';
	import { tryGetRoomPreviewFromAddress } from 'tauri-plugin-matrix-svelte-api';
	import { m } from '$lib/paraglide/messages';
	import { gotoRoomPreview } from '$lib/utils.svelte';

	interface Props {
		open: boolean;
	}

	let { open = $bindable(false) }: Props = $props();

	let inputValue = $state('');

	const resetState = (): undefined => {
		isValid = false;
		return undefined;
	};
	let isValid = $state(false);

	const debouncedResult = new Debounced(
		() =>
			inputValue
				? tryGetRoomPreviewFromAddress(inputValue).then((res) => {
						isValid = true;
						return res;
					})
				: resetState(),
		500
	);

	const handleSubmit = async () => {
		if (!debouncedResult.current) return;
		const [preview, via] = await debouncedResult.current;
		gotoRoomPreview(preview, via, null);
		inputValue = '';
		isValid = false;
		open = false;
	};

	const handleOpenChange = (newOpen: boolean) => {
		open = newOpen;
		if (!newOpen) {
			inputValue = '';
			isValid = false;
		}
	};
</script>

<Dialog {open} onOpenChange={handleOpenChange}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{m.join_room_by_address_title()}</DialogTitle>
			<DialogDescription>{m.join_room_by_address_desc()}</DialogDescription>
		</DialogHeader>

		<div class="flex flex-col gap-2">
			<Input placeholder="#room-alias:matrix.org" bind:value={inputValue} class="w-full" />

			<div class="flex gap-1">
				{#if debouncedResult.current}
					{#await debouncedResult.current}
						<Loader class="size-5 text-blue-500 animate-spin" />
					{:then}
						<CircleCheck class="size-5 text-green-500" />
					{:catch err}
						<CircleX class="size-5 text-red-500" />
						<Badge variant="destructive" class="w-fit">
							{err}
						</Badge>
					{/await}
				{/if}
			</div>
		</div>

		<DialogFooter class="flex gap-2">
			<Button variant="outline" onclick={() => (open = false)}>{m.button_cancel()}</Button>
			<Button onclick={handleSubmit} disabled={!isValid}>{m.button_next()}</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
