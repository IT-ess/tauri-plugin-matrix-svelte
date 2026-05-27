<script lang="ts">
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { Hash, Plus, UserPen, Users } from '@lucide/svelte';
	import CreateDmRoom from './create-dm-room.svelte';
	import { m } from '$lib/paraglide/messages';
	import CreateRoom from './create-room.svelte';
	import AddRoomByAddress from './add-room-by-address.svelte';

	// Track button rotation state
	let isActionButtonOpen = $state(false);

	let actionCreateDMRoomOpen = $state(false);
	let actionCreateRoomOpen = $state(false);
	let actionAddRoomByAddressOpen = $state(false);
</script>

<div class="bottom-safe-offset-28 absolute right-6 z-50">
	<DropdownMenu bind:open={isActionButtonOpen}>
		<DropdownMenuTrigger>
			{#snippet child({ props: triggerProps })}
				<Button {...triggerProps} size="icon" variant="secondary" class="h-12 w-12">
					<div style:transform={isActionButtonOpen ? 'rotate(45deg)' : 'rotate(0deg)'}>
						<Plus class="h-8 w-8 transition-transform duration-200" />
					</div>
				</Button>
			{/snippet}
		</DropdownMenuTrigger>
		<DropdownMenuContent align="end" side="top">
			<DropdownMenuItem class="text-md" onSelect={() => (actionCreateDMRoomOpen = true)}
				><UserPen />{m.create_dm_room()}</DropdownMenuItem
			>
			<DropdownMenuItem class="text-md" onSelect={() => (actionCreateRoomOpen = true)}
				><Users />{m.create_room()}</DropdownMenuItem
			>
			<DropdownMenuItem class="text-md" onSelect={() => (actionAddRoomByAddressOpen = true)}
				><Hash />{m.join_room_from_address()}</DropdownMenuItem
			>
		</DropdownMenuContent>
	</DropdownMenu>
</div>

<CreateDmRoom bind:actionCreateDMRoomOpen />
<CreateRoom bind:actionCreateRoomOpen />
<AddRoomByAddress bind:open={actionAddRoomByAddressOpen} />
