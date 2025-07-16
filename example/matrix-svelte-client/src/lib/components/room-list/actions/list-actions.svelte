<script lang="ts">
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { PlusIcon, UserPen } from '@lucide/svelte';
	import CreateRoom from './create-room.svelte';
	import type { CreateDMRoomFormSchema } from '$lib/schemas/matrix-id';
	import { type SuperValidated, type Infer } from 'sveltekit-superforms';

	let { createDMRoomForm }: { createDMRoomForm: SuperValidated<Infer<CreateDMRoomFormSchema>> } =
		$props();

	// Track button rotation state
	let isActionButtonOpen = $state(false);

	let actionCreateRoomOpen = $state(false);
</script>

<div class="absolute right-4 bottom-4">
	<DropdownMenu bind:open={isActionButtonOpen}>
		<DropdownMenuTrigger>
			{#snippet child({ props: triggerProps })}
				<Button {...triggerProps} size="icon" variant="secondary" class="h-10 w-10">
					<div style:transform={isActionButtonOpen ? 'rotate(45deg)' : 'rotate(0deg)'}>
						<PlusIcon class="h-6 w-6 transition-transform duration-200" />
					</div>
				</Button>
			{/snippet}
		</DropdownMenuTrigger>
		<DropdownMenuContent align="end" side="top">
			<DropdownMenuItem onSelect={() => (actionCreateRoomOpen = true)}
				><UserPen />Create DM Room</DropdownMenuItem
			>
		</DropdownMenuContent>
	</DropdownMenu>
</div>

<CreateRoom {actionCreateRoomOpen} dataForm={createDMRoomForm} />
